//! TVmaze: keyless TV metadata, used for shows when TMDB has no key or no match.
//! Data is CC BY-SA. English only. Limit is about 20 calls per 10 s per IP.

use crate::{CastMember, Details, Episode, MetaError, Season};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::time::Duration;

const API: &str = "https://api.tvmaze.com";
const IMAGES: &str = "https://static.tvmaze.com/uploads/images";

/// The `size` part of a TVmaze image key ("tvmaze/original_untouched-501-1253519.jpg").
pub const IMAGE_PREFIX: &str = "tvmaze";
const VARIANTS: &[&str] = &["original_untouched", "medium_portrait", "medium_landscape"];

/// A show with every season and episode, from one call.
#[derive(Debug, Clone, Default)]
pub struct Show {
    pub id: i64,
    /// `tmdb_id` is unused. Image fields are full image keys, not TMDB paths.
    pub details: Details,
    /// By season number.
    pub seasons: HashMap<i64, Season>,
}

#[derive(Clone, Debug)]
pub struct TvmazeClient {
    client: Client,
}

#[derive(Deserialize)]
struct SearchHit {
    show: RawShow,
}

#[derive(Deserialize, Default)]
struct Image {
    medium: Option<String>,
    original: Option<String>,
}

#[derive(Deserialize, Default)]
struct Rating {
    average: Option<f64>,
}

#[derive(Deserialize)]
struct RawShow {
    id: i64,
    name: String,
    premiered: Option<String>,
    summary: Option<String>,
    #[serde(default)]
    genres: Vec<String>,
    #[serde(rename = "averageRuntime")]
    average_runtime: Option<i64>,
    runtime: Option<i64>,
    #[serde(default)]
    rating: Rating,
    image: Option<Image>,
    #[serde(rename = "_embedded")]
    embedded: Option<Embedded>,
}

#[derive(Deserialize, Default)]
struct Embedded {
    #[serde(default)]
    seasons: Vec<RawSeason>,
    #[serde(default)]
    episodes: Vec<RawEpisode>,
    #[serde(default)]
    images: Vec<RawImage>,
    #[serde(default)]
    cast: Vec<RawCast>,
}

#[derive(Deserialize)]
struct RawSeason {
    number: i64,
    image: Option<Image>,
}

#[derive(Deserialize)]
struct RawEpisode {
    season: i64,
    number: Option<i64>,
    name: Option<String>,
    summary: Option<String>,
    runtime: Option<i64>,
    #[serde(default)]
    rating: Rating,
    image: Option<Image>,
}

#[derive(Deserialize)]
struct RawImage {
    #[serde(rename = "type")]
    kind: Option<String>,
    #[serde(default)]
    main: bool,
    resolutions: Resolutions,
}

#[derive(Deserialize)]
struct Resolutions {
    original: Option<Url>,
}

#[derive(Deserialize)]
struct Url {
    url: String,
}

#[derive(Deserialize)]
struct RawCast {
    person: Person,
    character: Option<Person>,
}

#[derive(Deserialize)]
struct Person {
    id: i64,
    name: String,
    image: Option<Image>,
}

/// "https://static.tvmaze.com/uploads/images/original_untouched/501/1253519.jpg"
/// to the image key "tvmaze/original_untouched-501-1253519.jpg".
fn image_key(url: Option<&str>) -> Option<String> {
    let rest = url?.strip_prefix(IMAGES)?.strip_prefix('/')?;
    let mut parts = rest.split('/');
    let (variant, dir, file) = (parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() || !VARIANTS.contains(&variant) {
        return None;
    }
    Some(format!("{IMAGE_PREFIX}/{variant}-{dir}-{file}"))
}

/// TVmaze summaries are HTML ("<p><b>Title</b> is ...</p>").
fn plain_text(html: Option<String>) -> Option<String> {
    let html = html?;
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    let out = out
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ");
    let out = out.trim();
    (!out.is_empty()).then(|| out.to_string())
}

fn year(date: Option<&str>) -> Option<i64> {
    date?.get(..4)?.parse().ok()
}

impl Default for TvmazeClient {
    fn default() -> Self {
        Self::new()
    }
}

impl TvmazeClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent(concat!("Kinema/", env!("CARGO_PKG_VERSION")))
                .timeout(Duration::from_secs(20))
                .build()
                .unwrap_or_default(),
        }
    }

    async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<Option<T>, MetaError> {
        // A scan of a big library can pass the rate limit; TVmaze asks clients to wait and retry.
        for _ in 0..3 {
            let res = self
                .client
                .get(format!("{API}{path}"))
                .query(query)
                .send()
                .await?;
            match res.status() {
                StatusCode::TOO_MANY_REQUESTS => tokio::time::sleep(Duration::from_secs(10)).await,
                StatusCode::NOT_FOUND => return Ok(None),
                s if s.is_success() => return Ok(Some(res.json().await?)),
                s => return Err(MetaError::Tvmaze(format!("{s} {path}"))),
            }
        }
        Err(MetaError::Tvmaze(format!("rate limited {path}")))
    }

    /// Best match by title. With a year, the best hit that premiered that year wins.
    pub async fn search(
        &self,
        title: &str,
        year_hint: Option<i64>,
    ) -> Result<Option<i64>, MetaError> {
        let hits: Vec<SearchHit> = self
            .get("/search/shows", &[("q", title)])
            .await?
            .unwrap_or_default();
        let by_year = year_hint.and_then(|y| {
            hits.iter()
                .find(|h| year(h.show.premiered.as_deref()) == Some(y))
        });
        Ok(by_year.or(hits.first()).map(|h| h.show.id))
    }

    pub async fn show(&self, id: i64) -> Result<Option<Show>, MetaError> {
        let path = format!("/shows/{id}");
        let embeds = [
            ("embed[]", "seasons"),
            ("embed[]", "episodes"),
            ("embed[]", "images"),
            ("embed[]", "cast"),
        ];
        let Some(raw) = self.get::<RawShow>(&path, &embeds).await? else {
            return Ok(None);
        };
        let emb = raw.embedded.unwrap_or_default();

        let backdrop = emb
            .images
            .iter()
            .filter(|i| i.kind.as_deref() == Some("background"))
            .max_by_key(|i| i.main)
            .and_then(|i| image_key(i.resolutions.original.as_ref().map(|u| u.url.as_str())));
        let cast = emb
            .cast
            .into_iter()
            .take(20)
            .map(|c| CastMember {
                id: c.person.id,
                image: image_key(c.person.image.and_then(|i| i.medium).as_deref()),
                name: c.person.name,
                role: c
                    .character
                    .map(|ch| ch.name)
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or_else(|| "Cast".to_string()),
            })
            .collect();

        let mut seasons: HashMap<i64, Season> = emb
            .seasons
            .into_iter()
            .map(|s| {
                let season = Season {
                    poster: image_key(s.image.and_then(|i| i.original).as_deref()),
                    episodes: Vec::new(),
                };
                (s.number, season)
            })
            .collect();
        for e in emb.episodes {
            // Specials have no number and cannot be matched to a file.
            let Some(number) = e.number else { continue };
            seasons.entry(e.season).or_default().episodes.push(Episode {
                number,
                title: e.name.unwrap_or_default(),
                overview: plain_text(e.summary),
                still: image_key(e.image.and_then(|i| i.medium).as_deref()),
                rating: e.rating.average.filter(|r| *r > 0.0),
                runtime_min: e.runtime.filter(|r| *r > 0),
            });
        }

        let image = raw.image.unwrap_or_default();
        Ok(Some(Show {
            id: raw.id,
            details: Details {
                tmdb_id: 0,
                title: raw.name,
                year: year(raw.premiered.as_deref()),
                overview: plain_text(raw.summary),
                rating: raw.rating.average.filter(|r| *r > 0.0),
                runtime_min: raw.average_runtime.or(raw.runtime).filter(|r| *r > 0),
                poster: image_key(image.original.as_deref()),
                backdrop,
                genres: raw.genres,
                cast,
                ..Default::default()
            },
            seasons,
        }))
    }
}

/// Downloads one TVmaze image by the name part of its key. Needs no key.
pub async fn fetch_image(client: &Client, name: &str) -> Result<Vec<u8>, MetaError> {
    let mut parts = name.splitn(3, '-');
    let (Some(variant), Some(dir), Some(file)) = (parts.next(), parts.next(), parts.next()) else {
        return Err(MetaError::NotFound(name.to_string()));
    };
    if !VARIANTS.contains(&variant) || !dir.bytes().all(|b| b.is_ascii_digit()) {
        return Err(MetaError::NotFound(name.to_string()));
    }
    let res = client
        .get(format!("{IMAGES}/{variant}/{dir}/{file}"))
        .send()
        .await?;
    if !res.status().is_success() {
        return Err(MetaError::NotFound(name.to_string()));
    }
    Ok(res.bytes().await?.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_keys() {
        assert_eq!(
            image_key(Some(
                "https://static.tvmaze.com/uploads/images/original_untouched/501/1253519.jpg"
            ))
            .as_deref(),
            Some("tvmaze/original_untouched-501-1253519.jpg")
        );
        assert_eq!(image_key(Some("https://example.com/a/b/c.jpg")), None);
        assert_eq!(
            image_key(Some(
                "https://static.tvmaze.com/uploads/images/evil/1/2.jpg"
            )),
            None
        );
    }

    #[test]
    fn summaries() {
        assert_eq!(
            plain_text(Some(
                "<p><b>Breaking Bad</b> follows Walt &amp; Jesse.</p>".into()
            ))
            .as_deref(),
            Some("Breaking Bad follows Walt & Jesse.")
        );
        assert_eq!(plain_text(Some("<p></p>".into())), None);
    }
}
