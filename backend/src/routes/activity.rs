//! Watch activity: who is playing what right now (kept in memory from progress reports),
//! watch time per profile and recent history. Admins see everyone; a profile sees itself.

use crate::auth::{AdminUser, AuthUser};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use kinema_db::activity::{self, Day, GenreTime, TopTitle, Totals};
use kinema_db::items::{self, ItemCard};
use kinema_db::users;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/activity/now", get(now_playing))
        .route("/activity/users", get(users_summary))
        .route("/activity/users/:id", get(user_stats))
        .route("/profile/stats", get(my_stats))
}

/// A player that reported within this long counts as playing.
const LIVE: Duration = Duration::from_secs(45);
/// Reports further apart than this are separate viewings: the gap is not counted.
const MAX_GAP: Duration = Duration::from_secs(120);

#[derive(Debug, Clone)]
pub struct Playing {
    pub position_s: f64,
    pub device_kind: Option<String>,
    pub device_name: Option<String>,
    pub started_at: i64,
    pub seen: Instant,
}

/// Keyed by (profile, item).
pub type NowPlayingMap = Arc<Mutex<HashMap<(i64, i64), Playing>>>;

/// Records a progress report and returns the seconds actually watched since the last one.
/// Only forward movement counts, capped by the wall time between reports, so a pause adds
/// nothing and a seek forward adds no more than the time that passed.
pub fn report(
    map: &NowPlayingMap,
    user_id: i64,
    item_id: i64,
    position_s: f64,
    finished: bool,
    device: Option<(String, String)>,
) -> i64 {
    let Ok(mut m) = map.lock() else { return 0 };
    m.retain(|_, p| p.seen.elapsed() < MAX_GAP * 5);
    let key = (user_id, item_id);
    let delta = match m.get(&key) {
        Some(p) if p.seen.elapsed() < MAX_GAP => {
            let wall = p.seen.elapsed().as_secs_f64() + 2.0;
            (position_s - p.position_s).clamp(0.0, wall).round() as i64
        }
        _ => 0,
    };
    if finished {
        m.remove(&key);
        return delta;
    }
    let started_at = m
        .get(&key)
        .filter(|p| p.seen.elapsed() < MAX_GAP)
        .map_or_else(kinema_db::now, |p| p.started_at);
    let (device_kind, device_name) = device.map_or((None, None), |(k, n)| (Some(k), Some(n)));
    m.insert(
        key,
        Playing {
            position_s,
            device_kind,
            device_name,
            started_at,
            seen: Instant::now(),
        },
    );
    delta
}

#[derive(Serialize)]
struct NowPlayingView {
    user_id: i64,
    user_name: String,
    item: ItemCard,
    position_s: f64,
    device_kind: Option<String>,
    device_name: Option<String>,
    started_at: i64,
    /// Seconds since the last report; over ~15 the player is likely paused.
    idle_s: u64,
}

async fn now_playing(
    State(state): State<AppState>,
    _: AdminUser,
) -> ApiResult<Json<Vec<NowPlayingView>>> {
    let live: Vec<((i64, i64), Playing)> = state
        .now_playing
        .lock()
        .map(|m| {
            m.iter()
                .filter(|(_, p)| p.seen.elapsed() < LIVE)
                .map(|(k, p)| (*k, p.clone()))
                .collect()
        })
        .unwrap_or_default();
    let mut out = Vec::new();
    for ((user_id, item_id), p) in live {
        let (Some(user), Some(item)) = (
            users::get(&state.db, user_id).await?,
            items::card(&state.db, user_id, item_id).await?,
        ) else {
            continue;
        };
        out.push(NowPlayingView {
            user_id,
            user_name: user.name,
            item,
            position_s: p.position_s,
            device_kind: p.device_kind,
            device_name: p.device_name,
            started_at: p.started_at,
            idle_s: p.seen.elapsed().as_secs(),
        });
    }
    out.sort_by_key(|n| std::cmp::Reverse(n.started_at));
    Ok(Json(out))
}

#[derive(Serialize)]
struct UserSummary {
    user: users::User,
    totals: Totals,
}

async fn users_summary(
    State(state): State<AppState>,
    _: AdminUser,
) -> ApiResult<Json<Vec<UserSummary>>> {
    let mut out = Vec::new();
    for user in users::list(&state.db).await? {
        let totals = activity::totals(&state.db, user.id).await?;
        out.push(UserSummary { user, totals });
    }
    Ok(Json(out))
}

#[derive(Serialize)]
struct HistoryEntry {
    item: ItemCard,
    last_played_at: i64,
}

#[derive(Serialize)]
struct Stats {
    totals: Totals,
    /// Last 30 days; days without play are left out. `day` is unix seconds / 86400.
    daily: Vec<Day>,
    top_titles: Vec<TopTitle>,
    top_genres: Vec<GenreTime>,
    history: Vec<HistoryEntry>,
}

async fn stats(state: &AppState, user_id: i64) -> ApiResult<Stats> {
    let db = &state.db;
    let mut history = Vec::new();
    for (item_id, last_played_at) in activity::history(db, user_id, 30).await? {
        if let Some(item) = items::card(db, user_id, item_id).await? {
            history.push(HistoryEntry {
                item,
                last_played_at,
            });
        }
    }
    Ok(Stats {
        totals: activity::totals(db, user_id).await?,
        daily: activity::daily(db, user_id, 30).await?,
        top_titles: activity::top_titles(db, user_id, 8).await?,
        top_genres: activity::top_genres(db, user_id, 6).await?,
        history,
    })
}

async fn user_stats(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<Stats>> {
    users::get(&state.db, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    Ok(Json(stats(&state, id).await?))
}

async fn my_stats(State(state): State<AppState>, AuthUser(me): AuthUser) -> ApiResult<Json<Stats>> {
    Ok(Json(stats(&state, me.id).await?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_forward_play_only() {
        let map = NowPlayingMap::default();
        assert_eq!(
            report(&map, 1, 2, 100.0, false, None),
            0,
            "first report has no baseline"
        );
        // Pretend the last report was 10 s ago.
        map.lock().unwrap().get_mut(&(1, 2)).unwrap().seen -= Duration::from_secs(10);
        assert_eq!(report(&map, 1, 2, 110.0, false, None), 10);
        // A seek back adds nothing.
        map.lock().unwrap().get_mut(&(1, 2)).unwrap().seen -= Duration::from_secs(10);
        assert_eq!(report(&map, 1, 2, 50.0, false, None), 0);
        // A seek forward adds no more than the wall time (plus slack).
        map.lock().unwrap().get_mut(&(1, 2)).unwrap().seen -= Duration::from_secs(10);
        assert_eq!(report(&map, 1, 2, 3000.0, false, None), 12);
        assert_eq!(report(&map, 1, 2, 3001.0, true, None), 1);
        assert!(map.lock().unwrap().is_empty(), "finished removes the entry");
    }
}
