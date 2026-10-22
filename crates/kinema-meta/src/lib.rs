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
    /// Certification ("PG-13", "TV-14"), US first; None when TMDB has none.
    pub certification: Option<String>,
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
    /// Movies, with `append_to_response=release_dates`.
    #[serde(default)]
    release_dates: Option<Page<RawCountryReleases>>,
    /// Shows, with `append_to_response=content_ratings`.
    #[serde(default)]
    content_ratings: Option<Page<RawCountryRating>>,
}

#[derive(Deserialize)]
struct RawCountryReleases {
    iso_3166_1: String,
    #[serde(default)]
    release_dates: Vec<RawRelease>,
}

#[derive(Deserialize)]
struct RawRelease {
    #[serde(default)]
    certification: String,
}

#[derive(Deserialize)]
struct RawCountryRating {
    iso_3166_1: String,
    #[serde(default)]
    rating: String,
}

/// Extra TMDB sections per details call: the certification source differs for movies and shows.
fn append_for(path: &str) -> &'static str {
    if path.starts_with("/tv/") {
        "credits,content_ratings"
    } else {
        "credits,release_dates"
    }
}

/// The US certification, else the first other country's, as one string per country.
fn pick_certification(by_country: Vec<(String, String)>) -> Option<String> {
    let mut rated: Vec<(String, String)> = by_country
        .into_iter()
        .map(|(c, r)| (c, r.trim().to_string()))
        .filter(|(_, r)| !r.is_empty())
        .collect();
    rated.sort_by_key(|(c, _)| c != "US");
    rated.into_iter().next().map(|(_, r)| r)
}

fn certification(d: &RawDetails) -> Option<String> {
    if let Some(page) = &d.release_dates {
        return pick_certification(
            page.results
                .iter()
                .map(|c| {
                    let cert = c
                        .release_dates
                        .iter()
                        .map(|r| r.certification.trim())
                        .find(|r| !r.is_empty())
                        .unwrap_or_default();
                    (c.iso_3166_1.clone(), cert.to_string())
                })
                .collect(),
        );
    }
    d.content_ratings.as_ref().and_then(|page| {
        pick_certification(
            page.results
                .iter()
                .map(|c| (c.iso_3166_1.clone(), c.rating.clone()))
                .collect(),
        )
    })
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
            .get(
                path,
                &[("append_to_response", append_for(path).to_string())],
            )
            .await?;
        let certification = certification(&d);
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
            certification,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn certification_prefers_us() {
        let movie: RawDetails = serde_json::from_str(
            r#"{"id": 1, "release_dates": {"results": [
                {"iso_3166_1": "DE", "release_dates": [{"certification": "12"}]},
                {"iso_3166_1": "US", "release_dates": [{"certification": ""}, {"certification": "PG-13"}]}
            ]}}"#,
        )
        .unwrap();
        assert_eq!(certification(&movie).as_deref(), Some("PG-13"));
        let show: RawDetails = serde_json::from_str(
            r#"{"id": 2, "content_ratings": {"results": [{"iso_3166_1": "GB", "rating": "15"}]}}"#,
        )
        .unwrap();
        assert_eq!(certification(&show).as_deref(), Some("15"));
        let none: RawDetails = serde_json::from_str(r#"{"id": 3}"#).unwrap();
        assert_eq!(certification(&none), None);
        assert_eq!(append_for("/tv/5"), "credits,content_ratings");
        assert_eq!(append_for("/movie/5"), "credits,release_dates");
    }
}
