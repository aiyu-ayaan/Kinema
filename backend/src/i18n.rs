use axum::extract::{Path, Request};
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Metadata for a supported UI/API locale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct LocaleInfo {
    pub code: &'static str,
    pub name: &'static str,
    pub rtl: bool,
}

/// Canonical 27 languages (English + 26 translations), matching the other Kinema clients.
pub const SUPPORTED_LOCALES: &[LocaleInfo] = &[
    LocaleInfo {
        code: "en",
        name: "English",
        rtl: false,
    },
    LocaleInfo {
        code: "hi",
        name: "हिन्दी",
        rtl: false,
    },
    LocaleInfo {
        code: "es",
        name: "Español",
        rtl: false,
    },
    LocaleInfo {
        code: "pt-BR",
        name: "Português (Brasil)",
        rtl: false,
    },
    LocaleInfo {
        code: "id",
        name: "Bahasa Indonesia",
        rtl: false,
    },
    LocaleInfo {
        code: "ar",
        name: "العربية",
        rtl: true,
    },
    LocaleInfo {
        code: "fr",
        name: "Français",
        rtl: false,
    },
    LocaleInfo {
        code: "ru",
        name: "Русский",
        rtl: false,
    },
    LocaleInfo {
        code: "de",
        name: "Deutsch",
        rtl: false,
    },
    LocaleInfo {
        code: "tr",
        name: "Türkçe",
        rtl: false,
    },
    LocaleInfo {
        code: "ja",
        name: "日本語",
        rtl: false,
    },
    LocaleInfo {
        code: "ko",
        name: "한국어",
        rtl: false,
    },
    LocaleInfo {
        code: "it",
        name: "Italiano",
        rtl: false,
    },
    LocaleInfo {
        code: "vi",
        name: "Tiếng Việt",
        rtl: false,
    },
    LocaleInfo {
        code: "th",
        name: "ไทย",
        rtl: false,
    },
    LocaleInfo {
        code: "zh-CN",
        name: "中文 (简体)",
        rtl: false,
    },
    LocaleInfo {
        code: "zh-TW",
        name: "中文 (繁體)",
        rtl: false,
    },
    LocaleInfo {
        code: "pl",
        name: "Polski",
        rtl: false,
    },
    LocaleInfo {
        code: "bn",
        name: "বাংলা",
        rtl: false,
    },
    LocaleInfo {
        code: "ta",
        name: "தமிழ்",
        rtl: false,
    },
    LocaleInfo {
        code: "te",
        name: "తెలుగు",
        rtl: false,
    },
    LocaleInfo {
        code: "mr",
        name: "मराठी",
        rtl: false,
    },
    LocaleInfo {
        code: "gu",
        name: "ગુજરાતી",
        rtl: false,
    },
    LocaleInfo {
        code: "kn",
        name: "ಕನ್ನಡ",
        rtl: false,
    },
    LocaleInfo {
        code: "ml",
        name: "മലയാളം",
        rtl: false,
    },
    LocaleInfo {
        code: "pa",
        name: "ਪੰਜਾਬੀ",
        rtl: false,
    },
    LocaleInfo {
        code: "ur",
        name: "اردو",
        rtl: true,
    },
];

const RAW_CATALOGS: &[(&str, &str)] = &[
    ("en", include_str!("../locales/en.json")),
    ("hi", include_str!("../locales/hi.json")),
    ("es", include_str!("../locales/es.json")),
    ("pt-BR", include_str!("../locales/pt-BR.json")),
    ("id", include_str!("../locales/id.json")),
    ("ar", include_str!("../locales/ar.json")),
    ("fr", include_str!("../locales/fr.json")),
    ("ru", include_str!("../locales/ru.json")),
    ("de", include_str!("../locales/de.json")),
    ("tr", include_str!("../locales/tr.json")),
    ("ja", include_str!("../locales/ja.json")),
    ("ko", include_str!("../locales/ko.json")),
    ("it", include_str!("../locales/it.json")),
    ("vi", include_str!("../locales/vi.json")),
    ("th", include_str!("../locales/th.json")),
    ("zh-CN", include_str!("../locales/zh-CN.json")),
    ("zh-TW", include_str!("../locales/zh-TW.json")),
    ("pl", include_str!("../locales/pl.json")),
    ("bn", include_str!("../locales/bn.json")),
    ("ta", include_str!("../locales/ta.json")),
    ("te", include_str!("../locales/te.json")),
    ("mr", include_str!("../locales/mr.json")),
    ("gu", include_str!("../locales/gu.json")),
    ("kn", include_str!("../locales/kn.json")),
    ("ml", include_str!("../locales/ml.json")),
    ("pa", include_str!("../locales/pa.json")),
    ("ur", include_str!("../locales/ur.json")),
];

type CatalogMap = HashMap<&'static str, HashMap<String, String>>;

fn catalogs() -> &'static CatalogMap {
    static CATALOGS: OnceLock<CatalogMap> = OnceLock::new();
    CATALOGS.get_or_init(|| {
        RAW_CATALOGS
            .iter()
            .map(|(code, raw)| (*code, serde_json::from_str(raw).unwrap_or_default()))
            .collect()
    })
}

tokio::task_local! {
    static REQUEST_LOCALE: &'static str;
}

/// Returns the locale resolved for the current HTTP request, or `"en"` outside a request.
pub fn current_locale() -> &'static str {
    REQUEST_LOCALE.try_with(|l| *l).unwrap_or("en")
}

/// Resolves a single BCP-47/ISO language tag to a supported canonical code.
fn resolve_tag(raw: &str) -> Option<&'static str> {
    let tag = raw.trim().split(';').next()?.trim();
    if tag.is_empty() || tag == "*" {
        return None;
    }
    let norm = tag.to_ascii_lowercase().replace('_', "-");

    if let Some(info) = SUPPORTED_LOCALES
        .iter()
        .find(|l| l.code.eq_ignore_ascii_case(&norm))
    {
        return Some(info.code);
    }

    // Legacy Android / script / regional aliases.
    if norm == "in" || norm.starts_with("in-") {
        return Some("id");
    }
    if norm == "zh-hans" || norm.starts_with("zh-hans-") || norm == "zh-sg" || norm == "zh" {
        return Some("zh-CN");
    }
    if norm == "zh-hant" || norm.starts_with("zh-hant-") || norm == "zh-hk" || norm == "zh-mo" {
        return Some("zh-TW");
    }
    if norm == "pt" || norm.starts_with("pt-") {
        return Some("pt-BR");
    }

    let base = norm.split('-').next().unwrap_or("");
    SUPPORTED_LOCALES
        .iter()
        .find(|l| l.code.eq_ignore_ascii_case(base))
        .map(|l| l.code)
}

/// Resolves a single locale tag or an `Accept-Language` header to a supported locale, falling
/// back to `"en"`.
pub fn resolve_locale(input: &str) -> &'static str {
    if !input.contains(',') && !input.contains(';') {
        return resolve_tag(input).unwrap_or("en");
    }

    let mut weighted: Vec<(f32, usize, &str)> = input
        .split(',')
        .enumerate()
        .filter_map(|(idx, part)| {
            let mut segs = part.split(';');
            let tag = segs.next()?.trim();
            if tag.is_empty() {
                return None;
            }
            let mut q = 1.0_f32;
            for param in segs {
                let p = param.trim();
                if let Some(val) = p.strip_prefix("q=").or_else(|| p.strip_prefix("Q="))
                    && let Ok(parsed) = val.trim().parse::<f32>()
                {
                    q = parsed.clamp(0.0, 1.0);
                }
            }
            if q <= 0.0 {
                return None;
            }
            Some((q, idx, tag))
        })
        .collect();

    weighted.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.cmp(&b.1)));

    for (_, _, tag) in weighted {
        if let Some(resolved) = resolve_tag(tag) {
            return resolved;
        }
    }
    "en"
}

/// Translates `key` for the active request locale (or `"en"` outside a request).
pub fn t(key: &str) -> String {
    t_for(current_locale(), key)
}

/// Translates `key` for `lang`, falling back to `"en"` and finally `key` itself.
pub fn t_for(lang: &str, key: &str) -> String {
    let resolved = resolve_locale(lang);
    let all = catalogs();
    if let Some(val) = all.get(resolved).and_then(|c| c.get(key)) {
        return val.clone();
    }
    if resolved != "en"
        && let Some(val) = all.get("en").and_then(|c| c.get(key))
    {
        return val.clone();
    }
    key.to_string()
}

/// Returns metadata for `lang` (resolved with fallback to `"en"`).
pub fn locale_info(lang: &str) -> &'static LocaleInfo {
    let resolved = resolve_locale(lang);
    SUPPORTED_LOCALES
        .iter()
        .find(|l| l.code == resolved)
        .unwrap_or(&SUPPORTED_LOCALES[0])
}

/// Returns the resolved `LocaleInfo` and its full message catalog.
pub fn catalog_for(lang: &str) -> (&'static LocaleInfo, &'static HashMap<String, String>) {
    let info = locale_info(lang);
    let all = catalogs();
    let map = all
        .get(info.code)
        .or_else(|| all.get("en"))
        .expect("en catalog embedded");
    (info, map)
}

fn query_lang(query: Option<&str>) -> Option<&str> {
    let q = query?;
    for pair in q.split('&') {
        if let Some((k, v)) = pair.split_once('=')
            && (k == "lang" || k == "locale")
            && !v.is_empty()
        {
            return Some(v);
        }
    }
    None
}

fn extract_request_locale(req: &Request) -> &'static str {
    if let Some(q) = query_lang(req.uri().query()) {
        return resolve_locale(q);
    }
    for name in ["x-kinema-lang", "x-locale"] {
        if let Some(val) = req.headers().get(name).and_then(|v| v.to_str().ok())
            && !val.trim().is_empty()
        {
            return resolve_locale(val);
        }
    }
    if let Some(accept) = req
        .headers()
        .get(axum::http::header::ACCEPT_LANGUAGE)
        .and_then(|v| v.to_str().ok())
    {
        return resolve_locale(accept);
    }
    "en"
}

/// Resolves the request locale from `?lang=`/`?locale=`, `X-Kinema-Lang`, or `Accept-Language`
/// and binds it for the duration of the request.
pub async fn locale_middleware(req: Request, next: Next) -> Response {
    let lang = extract_request_locale(&req);
    let mut res = REQUEST_LOCALE.scope(lang, next.run(req)).await;
    if let Ok(val) = HeaderValue::from_str(lang) {
        res.headers_mut()
            .insert(axum::http::header::CONTENT_LANGUAGE, val);
    }
    res
}

#[derive(Serialize)]
struct LocalesListResponse {
    default: &'static str,
    locales: &'static [LocaleInfo],
}

#[derive(Serialize)]
struct LocaleCatalogResponse<'a> {
    code: &'static str,
    name: &'static str,
    rtl: bool,
    messages: &'a HashMap<String, String>,
}

async fn list_locales() -> Json<LocalesListResponse> {
    Json(LocalesListResponse {
        default: "en",
        locales: SUPPORTED_LOCALES,
    })
}

async fn get_locale(Path(lang): Path<String>) -> Json<LocaleCatalogResponse<'static>> {
    let (info, messages) = catalog_for(&lang);
    Json(LocaleCatalogResponse {
        code: info.code,
        name: info.name,
        rtl: info.rtl,
        messages,
    })
}

/// Public locale discovery and catalog endpoints mounted under `/api`.
pub fn router<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/locales", get(list_locales))
        .route("/locales/:lang", get(get_locale))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ApiError;
    use axum::body::Body;
    use axum::http::{Request as HttpRequest, StatusCode};
    use std::collections::BTreeSet;
    use tower::ServiceExt;

    #[test]
    fn catalog_parses() {
        assert_eq!(t("common.not_found"), "Not found.");
    }

    #[test]
    fn all_locales_parse_and_match_en_keys() {
        assert_eq!(SUPPORTED_LOCALES.len(), 27);
        assert_eq!(RAW_CATALOGS.len(), 27);

        let en_map: HashMap<String, String> =
            serde_json::from_str(RAW_CATALOGS[0].1).expect("en.json parses");
        let en_keys: BTreeSet<&String> = en_map.keys().collect();
        assert!(!en_keys.is_empty());

        for (code, raw) in RAW_CATALOGS {
            let map: HashMap<String, String> = serde_json::from_str(raw)
                .unwrap_or_else(|e| panic!("{code}.json failed to parse: {e}"));
            let keys: BTreeSet<&String> = map.keys().collect();
            assert_eq!(keys, en_keys, "{code}.json key set differs from en.json");
            for (k, v) in &map {
                assert!(!v.trim().is_empty(), "{code}.json has empty value for {k}");
                if let Some(en_v) = en_map.get(k) {
                    for ph in ["{user}", "{kind}", "{device}", "{title}"] {
                        if en_v.contains(ph) {
                            assert!(
                                v.contains(ph),
                                "{code}.json key {k} missing placeholder {ph}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn resolves_locales_and_accept_language() {
        assert_eq!(resolve_locale("en"), "en");
        assert_eq!(resolve_locale("pt-BR"), "pt-BR");
        assert_eq!(resolve_locale("pt_br"), "pt-BR");
        assert_eq!(resolve_locale("pt-PT"), "pt-BR");
        assert_eq!(resolve_locale("zh-CN"), "zh-CN");
        assert_eq!(resolve_locale("zh_Hans_CN"), "zh-CN");
        assert_eq!(resolve_locale("zh-TW"), "zh-TW");
        assert_eq!(resolve_locale("zh-Hant"), "zh-TW");
        assert_eq!(resolve_locale("zh-HK"), "zh-TW");
        assert_eq!(resolve_locale("in"), "id");
        assert_eq!(resolve_locale("in-ID"), "id");
        assert_eq!(resolve_locale("fr-CA"), "fr");
        assert_eq!(resolve_locale("en-US"), "en");
        assert_eq!(resolve_locale("ar-SA"), "ar");
        assert_eq!(resolve_locale("xx-YY"), "en");
        assert_eq!(resolve_locale(""), "en");
        assert_eq!(resolve_locale("*"), "en");

        assert_eq!(resolve_locale("de;q=0.5, ja;q=0.9, en;q=0.1"), "ja");
        assert_eq!(resolve_locale("xx-YY, fr-FR;q=0.8, es;q=0.7"), "fr");
        assert_eq!(resolve_locale("ko;q=0, en;q=0.5"), "en");
    }

    #[test]
    fn translations_resolve_and_fallback() {
        assert_eq!(t_for("fr", "common.not_found"), "Introuvable.");
        assert_eq!(t_for("ar", "auth.required"), "سجّل الدخول للمتابعة.");
        assert_eq!(t_for("unknown", "common.not_found"), "Not found.");
        assert_eq!(t_for("fr", "nonexistent.key"), "nonexistent.key");
        assert!(locale_info("ar").rtl);
        assert!(locale_info("ur").rtl);
        assert!(!locale_info("en").rtl);
    }

    #[tokio::test]
    async fn http_locale_resolution_and_serving() {
        async fn err_handler() -> Result<(), ApiError> {
            Err(ApiError::not_found())
        }

        let app = Router::new()
            .merge(router::<()>())
            .route("/err", get(err_handler))
            .layer(axum::middleware::from_fn(locale_middleware));

        let res = app
            .clone()
            .oneshot(
                HttpRequest::builder()
                    .uri("/locales")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let list: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(list["default"], "en");
        assert_eq!(list["locales"].as_array().unwrap().len(), 27);

        let res = app
            .clone()
            .oneshot(
                HttpRequest::builder()
                    .uri("/locales/pt-BR")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let cat: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(cat["code"], "pt-BR");
        assert_eq!(cat["messages"]["common.not_found"], "Não encontrado.");

        // Query param takes precedence over header.
        let res = app
            .clone()
            .oneshot(
                HttpRequest::builder()
                    .uri("/err?lang=ja")
                    .header("accept-language", "fr")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.headers().get("content-language").unwrap(), "ja");
        let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let err_json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(err_json["error"]["message"], "見つかりませんでした。");

        // X-Kinema-Lang header.
        let res = app
            .clone()
            .oneshot(
                HttpRequest::builder()
                    .uri("/err")
                    .header("x-kinema-lang", "de")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.headers().get("content-language").unwrap(), "de");
        let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let err_json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(err_json["error"]["message"], "Nicht gefunden.");

        // Accept-Language header.
        let res = app
            .oneshot(
                HttpRequest::builder()
                    .uri("/err")
                    .header("accept-language", "es-MX, en;q=0.5")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.headers().get("content-language").unwrap(), "es");
        let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let err_json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(err_json["error"]["message"], "No encontrado.");
    }
}
