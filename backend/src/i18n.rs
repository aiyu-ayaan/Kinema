use std::collections::HashMap;
use std::sync::OnceLock;

/// User-facing server messages live in `backend/locales/<lang>.json`; English only for now.
static EN: &str = include_str!("../locales/en.json");

pub fn t(key: &str) -> String {
    static CATALOG: OnceLock<HashMap<String, String>> = OnceLock::new();
    CATALOG
        .get_or_init(|| serde_json::from_str(EN).unwrap_or_default())
        .get(key)
        .cloned()
        .unwrap_or_else(|| key.to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn catalog_parses() {
        assert_eq!(super::t("common.not_found"), "Not found.");
    }
}
