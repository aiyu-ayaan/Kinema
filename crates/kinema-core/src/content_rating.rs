//! Content ratings (certifications) as a minimum viewer age, so one per-profile limit works
//! across rating systems: a profile with a limit of 13 sees "PG-13", "TV-14" is too much.
//! Certifications come from TMDB (US first) or an admin override.

/// Ages the profile editor offers as limits. Any age in `0..=MAX_AGE` is accepted.
pub const LEVELS: &[u8] = &[0, 7, 10, 13, 14, 17];

/// Ages are clamped to this; "NC-17" and "18" are the strictest.
pub const MAX_AGE: u8 = 18;

/// Minimum viewer age for a certification, or None when unrated or unknown.
/// Unknown is not the same as "all ages": a profile's `allow_unrated` decides it.
pub fn age(certification: &str) -> Option<u8> {
    let c = certification.trim().to_ascii_uppercase();
    let known = match c.as_str() {
        // US (MPA) and US TV parental guidelines.
        "G" | "TV-Y" | "TV-G" => Some(0),
        "TV-Y7" | "TV-Y7-FV" => Some(7),
        "PG" | "TV-PG" => Some(10),
        "PG-13" => Some(13),
        "TV-14" => Some(14),
        "R" | "TV-MA" => Some(17),
        "NC-17" | "X" => Some(18),
        // UK (BBFC).
        "U" | "UC" => Some(0),
        "12A" => Some(12),
        "R18" => Some(18),
        "NR" | "UNRATED" | "NOT RATED" | "E" | "EXEMPT" | "" => return None,
        _ => None,
    };
    known.or_else(|| {
        // Plain ages used by many systems: "12", "16+", "FSK 16".
        let digits: String = c
            .trim_start_matches("FSK")
            .trim()
            .trim_end_matches('+')
            .to_string();
        digits
            .parse::<u8>()
            .ok()
            .filter(|a| *a <= 21)
            .map(|a| a.min(MAX_AGE))
    })
}

/// Whether a title with `certification` may be shown under a limit.
/// `max_age` None means no limit; unrated titles pass only when `allow_unrated`.
pub fn allowed(certification: Option<&str>, max_age: Option<u8>, allow_unrated: bool) -> bool {
    let Some(max) = max_age else {
        return true;
    };
    match certification.and_then(age) {
        Some(a) => a <= max,
        None => allow_unrated,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_common_systems() {
        assert_eq!(age("G"), Some(0));
        assert_eq!(age("pg-13"), Some(13));
        assert_eq!(age("TV-MA"), Some(17));
        assert_eq!(age("TV-Y7"), Some(7));
        assert_eq!(age("12A"), Some(12));
        assert_eq!(age("16"), Some(16));
        assert_eq!(age("FSK 16"), Some(16));
        assert_eq!(age("18+"), Some(18));
        assert_eq!(age("NC-17"), Some(18));
    }

    #[test]
    fn unknown_is_unrated() {
        assert_eq!(age("NR"), None);
        assert_eq!(age(""), None);
        assert_eq!(age("Banned"), None);
        assert_eq!(age("999"), None);
    }

    #[test]
    fn limits() {
        assert!(allowed(Some("R"), None, false));
        assert!(allowed(Some("PG"), Some(13), false));
        assert!(allowed(Some("PG-13"), Some(13), false));
        assert!(!allowed(Some("TV-14"), Some(13), false));
        assert!(!allowed(Some("R"), Some(13), true));
        assert!(!allowed(None, Some(13), false));
        assert!(allowed(None, Some(13), true));
        assert!(!allowed(Some("NR"), Some(17), false));
    }
}
