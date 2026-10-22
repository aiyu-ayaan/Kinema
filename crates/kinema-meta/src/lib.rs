pub mod error;
pub mod tvmaze;

pub use error::MetaError;
use reqwest::Client;
use serde::Deserialize;
use serde::de::DeserializeOwned;

const API: &str = "https://api.themoviedb.org/3";
const IMAGES: &str = "https://image.tmdb.org/t/p";

/// Image sizes the server asks TMDB for; also the allow-list for the image route.
pub const IMAGE_SIZES: &[&str] = &["w185", "w300", "w500", "w780", "w1280"];

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default, PartialEq, Eq)]
pub struct CastMember {
    pub id: i64,
    pub name: String,
    pub role: String,
    pub image: Option<String>,
}

/// Metadata for a movie, show or episode. Images are TMDB paths like "/abc.jpg".
#[derive(Debug, Clone, Default)]
pub struct Details {
    pub tmdb_id: i64,
    pub title: String,
    pub year: Option<i64>,
    pub overview: Option<String>,
    pub rating: Option<f64>,
    pub runtime_min: Option<i64>,
    pub poster: Option<String>,
    pub backdrop: Option<String>,
    pub genres: Vec<String>,
    pub cast: Vec<CastMember>,
    /// The franchise TMDB groups a movie in: (collection id, name).
    pub collection: Option<(i64, String)>,
}

#[derive(Debug, Clone, Default)]
pub struct Season {
    pub poster: Option<String>,
    pub episodes: Vec<Episode>,
}

#[derive(Debug, Clone, Default)]
pub struct Episode {
    pub number: i64,
    pub title: String,
    pub overview: Option<String>,
    pub still: Option<String>,
    pub rating: Option<f64>,
    pub runtime_min: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct TmdbClient {
    client: Client,
    api_key: String,
    language: Option<String>,
}

#[derive(Deserialize)]
struct Page<T> {
    results: Vec<T>,
}

#[derive(Deserialize)]
struct Hit {
    id: i64,
    title: Option<String>,
    name: Option<String>,
    release_date: Option<String>,
    first_air_date: Option<String>,
    overview: Option<String>,
    poster_path: Option<String>,
}

/// One search result, for the fix-match picker.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Candidate {
    pub tmdb_id: i64,
    pub title: String,
    pub year: Option<i64>,
    pub overview: Option<String>,
    pub poster: Option<String>,
}

#[derive(Deserialize)]
struct Genre {
    name: String,
}

#[derive(Deserialize, Default)]
struct RawCredits {
    #[serde(default)]
    cast: Vec<RawCast>,
    #[serde(default)]
    crew: Vec<RawCrew>,
}

#[derive(Deserialize)]
struct RawCast {
    id: i64,
    name: String,
    character: Option<String>,
    profile_path: Option<String>,
    #[serde(default)]
    order: i64,
}

#[derive(Deserialize)]
struct RawCrew {
    id: i64,
    name: String,
    job: Option<String>,
    profile_path: Option<String>,
}

#[derive(Deserialize)]
struct RawDetails {
    id: i64,
    title: Option<String>,
    name: Option<String>,
    release_date: Option<String>,
    first_air_date: Option<String>,
    overview: Option<String>,
    vote_average: Option<f64>,
    runtime: Option<i64>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    #[serde(default)]
    genres: Vec<Genre>,
    #[serde(default)]
    credits: Option<RawCredits>,
    #[serde(default)]
    belongs_to_collection: Option<RawCollection>,
}

#[derive(Deserialize)]
struct RawCollection {
    id: i64,
    name: String,
}

#[derive(Deserialize)]
struct RawSeason {
    poster_path: Option<String>,
    #[serde(default)]
    episodes: Vec<RawEpisode>,
}

#[derive(Deserialize)]
struct RawEpisode {
    episode_number: i64,
    name: Option<String>,
    overview: Option<String>,
    still_path: Option<String>,
    vote_average: Option<f64>,
    runtime: Option<i64>,
}

fn year(date: Option<&str>) -> Option<i64> {
    date?.get(..4)?.parse().ok()
}

fn non_empty(s: Option<String>) -> Option<String> {
    s.filter(|s| !s.trim().is_empty())
}

impl TmdbClient {
    /// Accepts a v3 API key or a v4 read access token.
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::builder()
                .user_agent(concat!("Kinema/", env!("CARGO_PKG_VERSION")))
                .timeout(std::time::Duration::from_secs(20))
                .build()
                .unwrap_or_default(),
            api_key,
            language: None,
        }
    }

    /// Titles and summaries in this language ("hi-IN"); TMDB falls back to the original.
    pub fn language(mut self, language: String) -> Self {
        self.language = Some(language).filter(|l| !l.is_empty());
        self
    }

    async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<T, MetaError> {
        let mut req = self.client.get(format!("{API}{path}")).query(query);
        if let Some(lang) = &self.language {
            req = req.query(&[("language", lang)]);
        }
        // v4 tokens are JWTs; v3 keys are 32 hex chars.
        req = if self.api_key.starts_with("eyJ") {
            req.bearer_auth(&self.api_key)
        } else {
            req.query(&[("api_key", &self.api_key)])
        };
        let res = req.send().await?;
        if !res.status().is_success() {
            return Err(MetaError::Tmdb(format!("{} {}", res.status(), path)));
        }
        Ok(res.json().await?)
    }

    /// Checks the key with a cheap call.
    pub async fn verify(&self) -> Result<(), MetaError> {
        self.get::<serde_json::Value>("/configuration", &[]).await?;
        Ok(())
    }

    /// Best match by title, trying with the year first.
    async fn search(
        &self,
        kind: &str,
        year_param: &str,
        title: &str,
        year: Option<i64>,
    ) -> Result<Option<i64>, MetaError> {
        let mut query = vec![("query", title.to_string())];
        if let Some(y) = year {
            query.push((year_param, y.to_string()));
            let page: Page<Hit> = self.get(&format!("/search/{kind}"), &query).await?;
            if let Some(hit) = page.results.first() {
                return Ok(Some(hit.id));
            }
            query.pop();
        }
        let page: Page<Hit> = self.get(&format!("/search/{kind}"), &query).await?;
        Ok(page.results.first().map(|h| h.id))
    }

    /// Up to 20 matches, best first. `kind` is "movie" or "show".
    pub async fn candidates(
        &self,
        kind: &str,
        query: &str,
        year_hint: Option<i64>,
    ) -> Result<Vec<Candidate>, MetaError> {
        let (path, year_param) = if kind == "show" {
            ("/search/tv", "first_air_date_year")
        } else {
            ("/search/movie", "year")
        };
        let mut q = vec![("query", query.to_string())];
        if let Some(y) = year_hint {
            q.push((year_param, y.to_string()));
        }
        let page: Page<Hit> = self.get(path, &q).await?;
        Ok(page
            .results
            .into_iter()
            .map(|h| Candidate {
                tmdb_id: h.id,
                title: h.title.or(h.name).unwrap_or_default(),
                year: year(h.release_date.as_deref().or(h.first_air_date.as_deref())),
                overview: non_empty(h.overview),
                poster: h.poster_path,
            })
            .collect())
    }

    /// Details by TMDB id, for a match the user picked.
    pub async fn by_id(&self, kind: &str, id: i64) -> Result<Details, MetaError> {
        let path = if kind == "show" {
            format!("/tv/{id}")
        } else {
            format!("/movie/{id}")
        };
        self.details(&path).await
    }

    async fn details(&self, path: &str) -> Result<Details, MetaError> {
        let d: RawDetails = self
            .get(path, &[("append_to_response", "credits".to_string())])
            .await?;
        let mut cast = Vec::new();
        if let Some(credits) = d.credits {
            let mut raw_cast = credits.cast;
            raw_cast.sort_by_key(|c| c.order);
            for c in raw_cast.into_iter().take(20) {
                cast.push(CastMember {
                    id: c.id,
                    name: c.name,
                    role: c
                        .character
                        .filter(|s| !s.trim().is_empty())
                        .unwrap_or_else(|| "Cast".to_string()),
                    image: c
                        .profile_path
                        .map(|p| format!("w185/{}", p.trim_start_matches('/'))),
                });
            }
            for cr in credits.crew {
                let is_key_crew = matches!(
                    cr.job.as_deref(),
                    Some("Director" | "Series Director" | "Creator" | "Writer")
                );
                if is_key_crew && !cast.iter().any(|c| c.id == cr.id) {
                    cast.push(CastMember {
                        id: cr.id,
                        name: cr.name,
                        role: cr.job.unwrap_or_else(|| "Director".to_string()),
                        image: cr
                            .profile_path
                            .map(|p| format!("w185/{}", p.trim_start_matches('/'))),
                    });
                }
            }
        }
        Ok(Details {
            tmdb_id: d.id,
            title: d.title.or(d.name).unwrap_or_default(),
            year: year(d.release_date.as_deref().or(d.first_air_date.as_deref())),
            overview: non_empty(d.overview),
            rating: d.vote_average.filter(|r| *r > 0.0),
            runtime_min: d.runtime.filter(|r| *r > 0),
            poster: d.poster_path,
            backdrop: d.backdrop_path,
            genres: d.genres.into_iter().map(|g| g.name).collect(),
            cast,
            collection: d.belongs_to_collection.map(|c| (c.id, c.name)),
        })
    }

    pub async fn movie(
        &self,
        title: &str,
        year: Option<i64>,
    ) -> Result<Option<Details>, MetaError> {
        match self.search("movie", "year", title, year).await? {
            Some(id) => Ok(Some(self.details(&format!("/movie/{id}")).await?)),
            None => Ok(None),
        }
    }

    pub async fn show(&self, title: &str, year: Option<i64>) -> Result<Option<Details>, MetaError> {
        match self
            .search("tv", "first_air_date_year", title, year)
            .await?
        {
            Some(id) => Ok(Some(self.details(&format!("/tv/{id}")).await?)),
            None => Ok(None),
        }
    }

    pub async fn season(&self, show_id: i64, season: i64) -> Result<Season, MetaError> {
        let s: RawSeason = self
            .get(&format!("/tv/{show_id}/season/{season}"), &[])
            .await?;
        Ok(Season {
            poster: s.poster_path,
            episodes: s
                .episodes
                .into_iter()
                .map(|e| Episode {
                    number: e.episode_number,
                    title: e.name.unwrap_or_default(),
                    overview: non_empty(e.overview),
                    still: e.still_path,
                    rating: e.vote_average.filter(|r| *r > 0.0),
                    runtime_min: e.runtime.filter(|r| *r > 0),
                })
                .collect(),
        })
    }
}

/// Downloads one TMDB image. Needs no key.
pub async fn fetch_image(client: &Client, size: &str, name: &str) -> Result<Vec<u8>, MetaError> {
    let res = client.get(format!("{IMAGES}/{size}/{name}")).send().await?;
    if !res.status().is_success() {
        return Err(MetaError::NotFound(name.to_string()));
    }
    Ok(res.bytes().await?.to_vec())
}
