//! Turns release-style file and folder names into titles, years and episode numbers.

use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovieName {
    pub title: String,
    pub year: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpisodeName {
    pub show: String,
    pub year: Option<i64>,
    pub season: i64,
    pub episode: i64,
}

/// Words that mark the end of a title in release names.
const STOP_WORDS: &[&str] = &[
    "2160p",
    "1080p",
    "1080i",
    "720p",
    "576p",
    "480p",
    "4k",
    "uhd",
    "bluray",
    "blu-ray",
    "bdrip",
    "brrip",
    "webrip",
    "web-dl",
    "webdl",
    "web",
    "hdtv",
    "dvdrip",
    "dvd",
    "remux",
    "x264",
    "x265",
    "h264",
    "h265",
    "hevc",
    "avc",
    "xvid",
    "hdr",
    "hdr10",
    "dv",
    "10bit",
    "proper",
    "repack",
    "extended",
    "unrated",
    "remastered",
    "imax",
    "aac",
    "ac3",
    "dts",
    "atmos",
    "multi",
];

/// Splits on separators; dots and underscores count as spaces in release names.
fn words(s: &str) -> Vec<String> {
    s.split(['.', '_', ' '])
        .filter(|w| !w.is_empty())
        .map(str::to_string)
        .collect()
}

fn year_of(word: &str) -> Option<i64> {
    let w = word.trim_matches(|c: char| "()[]{}".contains(c));
    if w.len() == 4 && w.chars().all(|c| c.is_ascii_digit()) {
        let y: i64 = w.parse().ok()?;
        if (1900..=2100).contains(&y) {
            return Some(y);
        }
    }
    None
}

fn is_stop(word: &str) -> bool {
    let w = word
        .trim_matches(|c: char| "()[]{}".contains(c))
        .to_lowercase();
    STOP_WORDS.contains(&w.as_str())
}

/// Drops leading "[group]" tags and tidies separators left at the ends.
fn tidy(words: &[String]) -> String {
    let joined = words.join(" ");
    let mut s = joined.trim();
    while s.starts_with('[') {
        match s.find(']') {
            Some(end) => s = s[end + 1..].trim_start(),
            None => break,
        }
    }
    s.trim_matches(|c: char| c == '-' || c == ' ' || c == '(' || c == '[')
        .to_string()
}

/// "Inception.2010.1080p.BluRay" and "Inception (2010)" both give ("Inception", 2010).
/// The last year wins so "2001 A Space Odyssey (1968)" keeps 2001 in the title.
pub fn parse_movie(name: &str) -> MovieName {
    let w = words(name);
    let year_pos = w
        .iter()
        .enumerate()
        .skip(1)
        .filter(|(_, x)| year_of(x).is_some())
        .map(|(i, _)| i)
        .next_back();
    let stop_pos = w.iter().skip(1).position(|x| is_stop(x)).map(|i| i + 1);
    let end = match (year_pos, stop_pos) {
        (Some(y), Some(s)) => y.min(s),
        (Some(y), None) => y,
        (None, Some(s)) => s,
        (None, None) => w.len(),
    };
    let title = tidy(&w[..end]);
    MovieName {
        title: if title.is_empty() {
            name.to_string()
        } else {
            title
        },
        year: year_pos.and_then(|i| year_of(&w[i])),
    }
}

/// Finds "S01E02", "s1e2" or "1x02" in a word; returns (season, episode).
fn episode_marker(word: &str) -> Option<(i64, i64)> {
    let lower = word.to_lowercase();
    let b = lower.as_bytes();
    let digits = |from: usize| -> (usize, Option<i64>) {
        let end = b[from..]
            .iter()
            .position(|c| !c.is_ascii_digit())
            .map_or(b.len(), |p| from + p);
        (end, lower[from..end].parse().ok())
    };
    for start in 0..b.len() {
        let at_word_start = start == 0 || !b[start - 1].is_ascii_alphanumeric();
        if b[start] == b's' && at_word_start {
            let (end, season) = digits(start + 1);
            if end < b.len()
                && b[end] == b'e'
                && (1..=2).contains(&(end - start - 1))
                && let (Some(s), (_, Some(e))) = (season, digits(end + 1))
            {
                return Some((s, e));
            }
        }
        if b[start].is_ascii_digit() && at_word_start {
            let (end, season) = digits(start);
            if end < b.len() && b[end] == b'x' && (1..=2).contains(&(end - start)) {
                let (eend, ep) = digits(end + 1);
                if eend - end > 2
                    && let (Some(s), Some(e)) = (season, ep)
                {
                    return Some((s, e));
                }
            }
        }
    }
    None
}

/// "Season 2", "S02", "Specials" (season 0).
pub fn season_folder(name: &str) -> Option<i64> {
    let lower = name.trim().to_lowercase();
    if lower == "specials" {
        return Some(0);
    }
    let rest = lower
        .strip_prefix("season")
        .or_else(|| lower.strip_prefix('s'))?;
    rest.trim_start_matches([' ', '.', '_']).parse().ok()
}

/// Episode from a file name. `show_folder` is the top folder under the library root
/// and wins over the name in the file, which is often abbreviated.
pub fn parse_episode(file_stem: &str, show_folder: Option<&str>) -> Option<EpisodeName> {
    let w = words(file_stem);
    let pos = w.iter().position(|x| episode_marker(x).is_some())?;
    let (season, episode) = episode_marker(&w[pos])?;
    let from_file = parse_movie(&tidy(&w[..pos]));
    let (show, year) = match show_folder.map(parse_movie) {
        Some(m) if !m.title.is_empty() => (m.title, m.year),
        _ => (from_file.title, from_file.year),
    };
    if show.is_empty() {
        return None;
    }
    Some(EpisodeName {
        show,
        year,
        season,
        episode,
    })
}

/// Title for a loose video: the cleaned file name.
pub fn video_title(path: &Path) -> String {
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    parse_movie(&stem).title
}

#[cfg(test)]
mod tests {
    use super::*;

    fn movie(t: &str, y: Option<i64>) -> MovieName {
        MovieName {
            title: t.into(),
            year: y,
        }
    }

    #[test]
    fn movies() {
        assert_eq!(
            parse_movie("Inception.2010.1080p.BluRay.x264-GRP"),
            movie("Inception", Some(2010))
        );
        assert_eq!(
            parse_movie("Inception (2010)"),
            movie("Inception", Some(2010))
        );
        assert_eq!(
            parse_movie("2001 A Space Odyssey (1968)"),
            movie("2001 A Space Odyssey", Some(1968))
        );
        assert_eq!(
            parse_movie("[YTS.MX] The Matrix 1999 720p"),
            movie("The Matrix", Some(1999))
        );
        assert_eq!(
            parse_movie("Some_Home_Video"),
            movie("Some Home Video", None)
        );
        assert_eq!(
            parse_movie("Blade Runner 2049 (2017) 2160p"),
            movie("Blade Runner 2049", Some(2017))
        );
        assert_eq!(parse_movie("Heat.WEBRip"), movie("Heat", None));
    }

    #[test]
    fn episodes() {
        let e = parse_episode("Breaking.Bad.S01E02.720p", None).unwrap();
        assert_eq!(
            (e.show.as_str(), e.season, e.episode),
            ("Breaking Bad", 1, 2)
        );
        let e = parse_episode("bb s3e10", Some("Breaking Bad (2008)")).unwrap();
        assert_eq!(
            (e.show.as_str(), e.year, e.season, e.episode),
            ("Breaking Bad", Some(2008), 3, 10)
        );
        let e = parse_episode("The Office 2x05 Halloween", None).unwrap();
        assert_eq!((e.show.as_str(), e.season, e.episode), ("The Office", 2, 5));
        assert!(parse_episode("Inception 2010", None).is_none());
        assert!(parse_episode("Movie 1920x1080", None).is_none());
    }

    #[test]
    fn seasons() {
        assert_eq!(season_folder("Season 2"), Some(2));
        assert_eq!(season_folder("S03"), Some(3));
        assert_eq!(season_folder("Specials"), Some(0));
        assert_eq!(season_folder("Extras"), None);
    }
}
