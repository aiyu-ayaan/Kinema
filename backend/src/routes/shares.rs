//! Share links (users-and-sharing.md § Sharing). An admin makes a link to one movie, show,
//! season or episode; whoever holds it can see that title and play what is under it,
//! nothing else, until the link expires or is revoked.
//!
//! The link never opens a session. Its routes live under `/api/share/{token}/...` and
//! check the token, the target's scope and the creator's own access on every request,
//! so revoking a link, removing the title, or narrowing the creator's access ends it.

use crate::access::{self, Policy};
use crate::auth::{AdminUser, ct_eq, random_hex, token_hash};
use crate::error::{ApiError, ApiResult};
use crate::routes::media::{self, Langs, PlayQuery};
use crate::state::AppState;
use axum::body::Body;
use axum::extract::{Path, Query, Request, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::{delete, get};
use axum::{Json, Router};
use futures_util::StreamExt;
use kinema_db::items::{self, ItemCard, Stream};
use kinema_db::shares::{self, NewShare, Share};
use kinema_db::users;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Public selector: 64 bits, finds the row. Secret verifier: 256 bits, only its hash is stored.
const SELECTOR_BYTES: usize = 8;
const VERIFIER_BYTES: usize = 32;
const TOKEN_LEN: usize = (SELECTOR_BYTES + VERIFIER_BYTES) * 2;
const MAX_HOURS: i64 = 90 * 24;
/// How often a running share stream checks that its link still works.
const RECHECK: Duration = Duration::from_secs(10);

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/shares", get(list).post(create))
        .route("/shares/:id", delete(revoke))
        .route("/share/:token", get(landing))
        .route("/share/:token/items/:id", get(item))
        .route("/share/:token/files/:id/plan", get(plan))
        .route("/share/:token/stream/:id", get(stream))
        .route("/share/:token/subtitles/:stream_id", get(subtitle))
        .route("/share/:token/images/:size/:name", get(image))
}

#[derive(Deserialize)]
struct NewShareBody {
    item_id: i64,
    /// 1 hour to 90 days. Every link expires.
    expires_in_hours: i64,
    #[serde(default)]
    label: Option<String>,
}

#[derive(Serialize)]
struct Created {
    share: Share,
    /// Shown once; the server keeps only a hash.
    token: String,
    /// Client route of the share page.
    path: String,
}

async fn create(
    State(state): State<AppState>,
    AdminUser(me): AdminUser,
    Json(b): Json<NewShareBody>,
) -> ApiResult<(StatusCode, Json<Created>)> {
    if !(1..=MAX_HOURS).contains(&b.expires_in_hours) {
        return Err(ApiError::bad_request("shares.expiry_invalid"));
    }
    let label = b
        .label
        .as_deref()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(|l| l.chars().take(60).collect::<String>());
    let target = access::visible_item(&state, &me, b.item_id).await?;
    if !matches!(
        target.kind.as_str(),
        "movie" | "show" | "season" | "episode" | "video"
    ) {
        return Err(ApiError::bad_request("shares.target_invalid"));
    }
    let selector = random_hex(SELECTOR_BYTES);
    let verifier = random_hex(VERIFIER_BYTES);
    let id = shares::create(
        &state.db,
        &NewShare {
            selector: &selector,
            token_hash: &token_hash(&verifier),
            created_by: me.id,
            item_id: target.id,
            label: label.as_deref(),
            expires_at: kinema_db::now() + b.expires_in_hours * 3600,
        },
    )
    .await?;
    let share = shares::get(&state.db, id)
        .await?
        .ok_or_else(ApiError::internal)?;
    let token = format!("{selector}{verifier}");
    Ok((
        StatusCode::CREATED,
        Json(Created {
            path: format!("/s/{token}"),
            token,
            share,
        }),
    ))
}

async fn list(State(state): State<AppState>, _: AdminUser) -> ApiResult<Json<Vec<Share>>> {
    Ok(Json(shares::list(&state.db).await?))
}

async fn revoke(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    if shares::revoke(&state.db, id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::not_found())
    }
}

/// A checked link: the share, the creator's visibility rules and the target title.
struct Scope {
    share: Share,
    policy: Policy,
    target: ItemCard,
}

impl Scope {
    /// The target itself, or a season or episode under it, that the creator may still see.
    fn contains(&self, card: &ItemCard) -> bool {
        let t = self.target.id;
        (card.id == t || card.parent_id == Some(t) || card.show_id == Some(t))
            && self.policy.allows(card)
    }
}

/// Splits `selector || verifier`; anything else is not a token.
fn split_token(token: &str) -> Option<(&str, &str)> {
    let ok = token.len() == TOKEN_LEN
        && token
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
    ok.then(|| token.split_at(SELECTOR_BYTES * 2))
}

/// Every failure is the same 404, so a probe learns nothing about which part was wrong.
async fn resolve(state: &AppState, token: &str) -> ApiResult<Scope> {
    let (selector, verifier) = split_token(token).ok_or_else(ApiError::not_found)?;
    let share = shares::active_by_selector(&state.db, selector)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if !ct_eq(token_hash(verifier).as_bytes(), share.token_hash.as_bytes()) {
        return Err(ApiError::not_found());
    }
    let creator = users::get(&state.db, share.created_by)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let policy = access::policy(state, &creator).await?;
    let target = items::card(&state.db, 0, share.item_id)
        .await?
        .filter(|c| policy.allows(c))
        .ok_or_else(ApiError::not_found)?;
    Ok(Scope {
        share,
        policy,
        target,
    })
}

async fn scoped_item(state: &AppState, scope: &Scope, id: i64) -> ApiResult<ItemCard> {
    items::card(&state.db, 0, id)
        .await?
        .filter(|c| scope.contains(c))
        .ok_or_else(ApiError::not_found)
}

async fn scoped_file(state: &AppState, scope: &Scope, file_id: i64) -> ApiResult<()> {
    let item_id = items::file(&state.db, file_id)
        .await?
        .and_then(|f| f.item_id)
        .ok_or_else(ApiError::not_found)?;
    scoped_item(state, scope, item_id).await.map(|_| ())
}

/// What a share viewer may see of a link. No paths, no other titles, no watch state.
#[derive(Serialize)]
struct ShareInfo {
    label: Option<String>,
    expires_at: i64,
    /// The title the link was made for, so the page can link back to it.
    target_id: i64,
}

#[derive(Serialize)]
struct ShareFile {
    id: i64,
    container: Option<String>,
    duration_s: Option<f64>,
    streams: Vec<Stream>,
}

#[derive(Serialize)]
struct ShareView {
    share: ShareInfo,
    item: ItemCard,
    children: Vec<ItemCard>,
    files: Vec<ShareFile>,
}

async fn view(state: &AppState, scope: &Scope, card: ItemCard) -> ApiResult<ShareView> {
    let mut children = items::children(&state.db, 0, card.id).await?;
    children.retain(|c| scope.contains(c));
    let mut files = Vec::new();
    for f in items::item_files(&state.db, card.id).await? {
        files.push(ShareFile {
            streams: items::streams(&state.db, f.id).await?,
            id: f.id,
            container: f.container,
            duration_s: f.duration_s,
        });
    }
    Ok(ShareView {
        share: ShareInfo {
            label: scope.share.label.clone(),
            expires_at: scope.share.expires_at,
            target_id: scope.target.id,
        },
        item: card,
        children,
        files,
    })
}

async fn landing(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> ApiResult<Json<ShareView>> {
    let scope = resolve(&state, &token).await?;
    let card = scope.target.clone();
    Ok(Json(view(&state, &scope, card).await?))
}

async fn item(
    State(state): State<AppState>,
    Path((token, id)): Path<(String, i64)>,
) -> ApiResult<Json<ShareView>> {
    let scope = resolve(&state, &token).await?;
    let card = scoped_item(&state, &scope, id).await?;
    Ok(Json(view(&state, &scope, card).await?))
}

async fn plan(
    State(state): State<AppState>,
    Path((token, id)): Path<(String, i64)>,
    Query(q): Query<PlayQuery>,
) -> ApiResult<Json<media::PlanResponse>> {
    let scope = resolve(&state, &token).await?;
    scoped_file(&state, &scope, id).await?;
    // No profile: the server's languages, no per-title choices.
    media::plan_for(&state, 0, &Langs::server(&state), id, &q)
        .await
        .map(Json)
}

async fn stream(
    State(state): State<AppState>,
    Path((token, id)): Path<(String, i64)>,
    Query(q): Query<PlayQuery>,
    req: Request,
) -> ApiResult<Response> {
    let scope = resolve(&state, &token).await?;
    scoped_file(&state, &scope, id).await?;
    let res = media::stream_for(&state, 0, &Langs::server(&state), id, &q, req).await?;
    Ok(stop_when_revoked(&state, scope.share.id, res))
}

/// A long response (one ranged read of a whole film, or an ffmpeg pipe) ends within
/// `RECHECK` of the link being revoked or expiring; dropping the body stops ffmpeg.
fn stop_when_revoked(state: &AppState, share_id: i64, res: Response) -> Response {
    let (parts, body) = res.into_parts();
    let db = state.db.clone();
    let mut next = Instant::now() + RECHECK;
    let data = body.into_data_stream().take_while(move |_| {
        let due = Instant::now() >= next;
        if due {
            next = Instant::now() + RECHECK;
        }
        let db = db.clone();
        async move { !due || shares::is_active(&db, share_id).await.unwrap_or(false) }
    });
    Response::from_parts(parts, Body::from_stream(data))
}

async fn subtitle(
    State(state): State<AppState>,
    Path((token, stream_id)): Path<(String, i64)>,
) -> ApiResult<Response> {
    let scope = resolve(&state, &token).await?;
    let s = items::stream(&state.db, stream_id)
        .await?
        .filter(|s| s.kind == "subtitle")
        .ok_or_else(ApiError::not_found)?;
    scoped_file(&state, &scope, s.file_id).await?;
    media::subtitle_for(&state, s).await
}

/// Artwork of the shared title and what is under it; any other image key is 404.
async fn image(
    State(state): State<AppState>,
    Path((token, size, name)): Path<(String, String, String)>,
    req: Request,
) -> ApiResult<Response> {
    let scope = resolve(&state, &token).await?;
    let key = format!("{size}/{name}");
    if !shares::image_in_scope(&state.db, scope.target.id, &key).await? {
        return Err(ApiError::not_found());
    }
    media::image_for(&state, &size, &name, req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_shape() {
        let t = format!(
            "{}{}",
            random_hex(SELECTOR_BYTES),
            random_hex(VERIFIER_BYTES)
        );
        let (sel, ver) = split_token(&t).unwrap();
        assert_eq!(sel.len(), 16);
        assert_eq!(ver.len(), 64);
        assert!(split_token(&t[1..]).is_none());
        assert!(split_token(&t.to_uppercase()).is_none());
        assert!(split_token(&format!("{}g", &t[1..])).is_none());
        assert!(split_token("").is_none());
    }
}
