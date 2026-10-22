//! The signed-in profile's own settings (track languages) and profile export / import.
//! Content limits and the LAN unlock are household rules: only an admin sets them
//! (`routes/users.rs`), and a profile importing a file can never lift its own limits.

use crate::auth::{AdminUser, AuthUser};
use crate::error::{ApiError, ApiResult};
use crate::server_settings::{SUBTITLE_MODES, TRACK_LANGUAGES};
use crate::state::AppState;
use argon2::password_hash::PasswordHash;
use axum::extract::{DefaultBodyLimit, Path, Query, State};
use axum::http::StatusCode;
use axum::http::header::CONTENT_DISPOSITION;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use kinema_core::content_rating;
use kinema_db::profiles::{self, Prefs, WatchRow};
use kinema_db::users::{self, User};
use serde::{Deserialize, Serialize};

/// Large enough for years of history (about 200 bytes per row), small enough to parse quickly.
const IMPORT_LIMIT: usize = 8 * 1024 * 1024;
const MAX_ROWS: usize = 50_000;
pub const FORMAT: &str = "kinema.profile";
pub const VERSION: u32 = 1;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/profile", get(me))
        .route("/profile/preferences", put(set_languages))
        .route("/profile/export", get(export_me))
        .route(
            "/profile/import",
            post(import_me).layer(DefaultBodyLimit::max(IMPORT_LIMIT)),
        )
        .route("/users/:id/export", get(export_user))
        .route(
            "/users/:id/import",
            post(import_user).layer(DefaultBodyLimit::max(IMPORT_LIMIT)),
        )
}

/// Track language preferences a profile edits for itself.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Languages {
    /// ISO 639-2, "" for the file's default, null to follow the server.
    pub audio_language: Option<String>,
    pub subtitle_language: Option<String>,
    /// off | always | foreign, null to follow the server.
    pub subtitle_mode: Option<String>,
}

/// Validates languages; unknown values are refused rather than silently dropped.
pub fn clean_languages(l: Languages) -> ApiResult<Languages> {
    let lang = |v: Option<String>| -> ApiResult<Option<String>> {
        match v.map(|s| s.trim().to_ascii_lowercase()) {
            None => Ok(None),
            Some(s) if s.is_empty() || TRACK_LANGUAGES.iter().any(|(c, _)| *c == s) => Ok(Some(s)),
            Some(_) => Err(ApiError::bad_request("profile.language_invalid")),
        }
    };
    let mode = match l.subtitle_mode {
        None => None,
        Some(m) if SUBTITLE_MODES.contains(&m.as_str()) => Some(m),
        Some(_) => return Err(ApiError::bad_request("profile.language_invalid")),
    };
    Ok(Languages {
        audio_language: lang(l.audio_language)?,
        subtitle_language: lang(l.subtitle_language)?,
        subtitle_mode: mode,
    })
}

pub fn clean_max_rating(r: Option<i64>) -> ApiResult<Option<i64>> {
    match r {
        Some(r) if !(0..=i64::from(content_rating::MAX_AGE)).contains(&r) => {
            Err(ApiError::bad_request("profile.rating_invalid"))
        }
        r => Ok(r),
    }
}

#[derive(Serialize)]
struct Me {
    user: User,
    preferences: Prefs,
    options: Options,
}

/// Choices the profile and admin editors offer.
#[derive(Serialize)]
struct Options {
    track_languages: Vec<&'static str>,
    subtitle_modes: &'static [&'static str],
    rating_levels: &'static [u8],
}

async fn me(State(state): State<AppState>, AuthUser(user): AuthUser) -> ApiResult<Json<Me>> {
    Ok(Json(Me {
        preferences: profiles::get(&state.db, user.id).await?,
        user,
        options: Options {
            track_languages: TRACK_LANGUAGES.iter().map(|(c, _)| *c).collect(),
            subtitle_modes: SUBTITLE_MODES,
            rating_levels: content_rating::LEVELS,
        },
    }))
}

/// Languages only: limits and the LAN unlock stay as the admin set them.
async fn set_languages(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(l): Json<Languages>,
) -> ApiResult<Json<Prefs>> {
    let l = clean_languages(l)?;
    let mut prefs = profiles::get(&state.db, user.id).await?;
    prefs.audio_language = l.audio_language;
    prefs.subtitle_language = l.subtitle_language;
    prefs.subtitle_mode = l.subtitle_mode;
    profiles::set(&state.db, user.id, &prefs).await?;
    Ok(Json(prefs))
}

/// Lock hashes, only in an admin export that asked for them.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LockExport {
    pub password_hash: Option<String>,
    pub pin_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportProfile {
    pub name: String,
    pub avatar: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Export {
    pub format: String,
    pub version: u32,
    pub exported_at: i64,
    pub profile: ExportProfile,
    pub preferences: Prefs,
    #[serde(default)]
    pub watch: Vec<WatchRow>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lock: Option<LockExport>,
}

async fn build_export(state: &AppState, user: &User, with_lock: bool) -> ApiResult<Export> {
    let lock = if with_lock {
        let (password_hash, pin_hash) = profiles::lock_hashes(&state.db, user.id).await?;
        Some(LockExport {
            password_hash,
            pin_hash,
        })
    } else {
        None
    };
    Ok(Export {
        format: FORMAT.into(),
        version: VERSION,
        exported_at: kinema_db::now(),
        profile: ExportProfile {
            name: user.name.clone(),
            avatar: user.avatar.clone(),
        },
        preferences: profiles::get(&state.db, user.id).await?,
        watch: profiles::watch_rows(&state.db, user.id).await?,
        lock,
    })
}

fn download(export: &Export) -> Response {
    let safe: String = export
        .profile
        .name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(40)
        .collect();
    let disposition = format!("attachment; filename=\"kinema-profile-{safe}.json\"");
    let mut res = Json(export).into_response();
    if let Ok(v) = disposition.parse() {
        res.headers_mut().insert(CONTENT_DISPOSITION, v);
    }
    res
}

/// A profile exports its own history and preferences; never its lock.
async fn export_me(State(state): State<AppState>, AuthUser(user): AuthUser) -> ApiResult<Response> {
    Ok(download(&build_export(&state, &user, false).await?))
}

#[derive(Deserialize)]
struct ExportQuery {
    /// Admin only, explicit: include the password and PIN hashes.
    #[serde(default)]
    include_lock: bool,
}

async fn export_user(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<i64>,
    Query(q): Query<ExportQuery>,
) -> ApiResult<Response> {
    let user = users::get(&state.db, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    Ok(download(
        &build_export(&state, &user, q.include_lock).await?,
    ))
}

#[derive(Debug, Default, Serialize)]
pub struct ImportResult {
    /// Rows written.
    pub imported: usize,
    /// Rows whose title is not on this server.
    pub unmatched: usize,
    /// Rows where this server already had newer progress.
    pub kept: usize,
    pub lock_applied: bool,
}

/// Structural checks before anything is written: a bad file changes nothing.
pub fn validate(export: &Export) -> ApiResult<()> {
    let bad = || ApiError::bad_request("profile.import_invalid");
    if export.format != FORMAT || export.version != VERSION || export.watch.len() > MAX_ROWS {
        return Err(bad());
    }
    let soon = kinema_db::now() + 86_400;
    for w in &export.watch {
        let k = &w.key;
        let kind_ok = matches!(k.kind.as_str(), "movie" | "show" | "episode" | "video");
        let numbers_ok = [k.season_no, k.episode_no]
            .iter()
            .flatten()
            .all(|n| (0..=10_000).contains(n));
        let ok = kind_ok
            && !k.title.trim().is_empty()
            && k.title.chars().count() <= 500
            && numbers_ok
            && (k.kind != "episode" || (k.season_no.is_some() && k.episode_no.is_some()))
            && w.position_s.is_finite()
            && (0.0..=1_000_000.0).contains(&w.position_s)
            && (0..=1_000_000).contains(&w.play_count)
            && w.last_played_at.is_none_or(|t| (0..=soon).contains(&t));
        if !ok {
            return Err(bad());
        }
    }
    Ok(())
}

/// An argon2 hash string, or the import is refused (a garbage hash would lock the profile out).
fn valid_hash(h: &Option<String>) -> bool {
    h.as_deref().is_none_or(|h| {
        h.len() <= 512
            && PasswordHash::new(h).is_ok_and(|p| {
                matches!(p.algorithm.as_str(), "argon2id" | "argon2i" | "argon2d")
                    && p.hash.is_some()
                    && p.salt.is_some()
            })
    })
}

async fn apply_watch(state: &AppState, user_id: i64, rows: &[WatchRow]) -> ApiResult<ImportResult> {
    let mut r = ImportResult::default();
    for w in rows {
        match profiles::find_item(&state.db, &w.key).await? {
            None => r.unmatched += 1,
            Some(item) if profiles::import_watch(&state.db, user_id, item, w).await? => {
                r.imported += 1
            }
            Some(_) => r.kept += 1,
        }
    }
    Ok(r)
}

/// Into the signed-in profile: watch history and languages. Limits, LAN unlock and the
/// lock in the file are ignored, so a restricted profile cannot loosen itself.
async fn import_me(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(export): Json<Export>,
) -> ApiResult<Json<ImportResult>> {
    validate(&export)?;
    let l = clean_languages(Languages {
        audio_language: export.preferences.audio_language.clone(),
        subtitle_language: export.preferences.subtitle_language.clone(),
        subtitle_mode: export.preferences.subtitle_mode.clone(),
    })?;
    let mut prefs = profiles::get(&state.db, user.id).await?;
    prefs.audio_language = l.audio_language;
    prefs.subtitle_language = l.subtitle_language;
    prefs.subtitle_mode = l.subtitle_mode;
    let result = apply_watch(&state, user.id, &export.watch).await?;
    profiles::set(&state.db, user.id, &prefs).await?;
    Ok(Json(result))
}

#[derive(Deserialize)]
struct ImportQuery {
    /// Explicit: also replace the profile's password / PIN with the hashes in the file.
    #[serde(default)]
    apply_lock: bool,
}

/// Admin: into any profile, including limits and LAN unlock; the lock only on request.
async fn import_user(
    State(state): State<AppState>,
    AdminUser(me): AdminUser,
    Path(id): Path<i64>,
    Query(q): Query<ImportQuery>,
    Json(export): Json<Export>,
) -> ApiResult<(StatusCode, Json<ImportResult>)> {
    let target = users::get(&state.db, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    validate(&export)?;
    let l = clean_languages(Languages {
        audio_language: export.preferences.audio_language.clone(),
        subtitle_language: export.preferences.subtitle_language.clone(),
        subtitle_mode: export.preferences.subtitle_mode.clone(),
    })?;
    let prefs = Prefs {
        audio_language: l.audio_language,
        subtitle_language: l.subtitle_language,
        subtitle_mode: l.subtitle_mode,
        max_rating: clean_max_rating(export.preferences.max_rating)?,
        allow_unrated: export.preferences.allow_unrated,
        lan_unlock: export.preferences.lan_unlock,
    };
    let lock = match (&export.lock, q.apply_lock) {
        (Some(lock), true) => {
            let ok = valid_hash(&lock.password_hash)
                && valid_hash(&lock.pin_hash)
                // Admins keep a password; never lock yourself out mid-session.
                && (!target.is_admin || lock.password_hash.is_some())
                && id != me.id;
            if !ok {
                return Err(ApiError::bad_request("profile.lock_invalid"));
            }
            Some(lock)
        }
        (None, true) => return Err(ApiError::bad_request("profile.lock_invalid")),
        _ => None,
    };
    let mut result = apply_watch(&state, id, &export.watch).await?;
    profiles::set(&state.db, id, &prefs).await?;
    if let Some(lock) = lock {
        profiles::set_lock_hashes(
            &state.db,
            id,
            lock.password_hash.as_deref(),
            lock.pin_hash.as_deref(),
        )
        .await?;
        result.lock_applied = true;
    }
    Ok((StatusCode::OK, Json(result)))
}
