pub mod error;

pub use error::MetaError;
use reqwest::Client;
use serde::Deserialize;
use serde::de::DeserializeOwned;

const API: &str = "https://api.themoviedb.org/3";
const IMAGES: &str = "https://image.tmdb.org/t/p";

/// Image sizes the server asks TMDB for; also the allow-list for the image route.
pub const IMAGE_SIZES: &[&str] = &["w300", "w500", "w780", "w1280"];

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
        }
    }

    async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<T, MetaError> {
        let mut req = self.client.get(format!("{API}{path}")).query(query);
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
        let d: RawDetails = self.get(path, &[]).await?;
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
