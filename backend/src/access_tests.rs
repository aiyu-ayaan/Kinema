//! Access-control tests over the real router and an in-memory database: content limits,
//! LAN-only profiles and sessions, share-link scope, profile export / import and
//! per-profile languages. Requests carry a socket peer like the real server sees.

use crate::auth::new_token;
use crate::jobs::Jobs;
use crate::routes::api_router;
use crate::server_settings::ServerSettings;
use crate::state::AppState;
use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::ConnectInfo;
use axum::http::{Method, Request, StatusCode};
use kinema_core::Config;
use kinema_db::items::{self, NewItem};
use kinema_db::profiles::{self, Prefs};
use kinema_db::users::{self, NewUser};
use kinema_db::{libraries, shares};
use serde_json::{Value, json};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::Semaphore;
use tower::ServiceExt;

pub(crate) const LAN: &str = "192.168.1.20:50000";
pub(crate) const WAN: &str = "203.0.113.7:50000";

struct World {
    state: AppState,
    app: Router,
    admin: String,
    kid: String,
    kid_id: i64,
    lib: i64,
    pg_movie: i64,
    r_movie: i64,
    unrated_movie: i64,
    show: i64,
    season1: i64,
    season2: i64,
    ep1: i64,
    ep2: i64,
    other_show_ep: i64,
}

pub(crate) async fn state() -> AppState {
    let opts = SqliteConnectOptions::from_str("sqlite::memory:")
        .unwrap()
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(opts)
        .await
        .unwrap();
    kinema_db::run_migrations(&pool).await.unwrap();
    let (changes, _) = tokio::sync::mpsc::unbounded_channel();
    let config = Config {
        data_dir: std::env::temp_dir().join(format!("kinema-test-{}", new_token().0)),
        ..Default::default()
    };
    AppState {
        config: Arc::new(config),
        db: pool,
        bins: None,
        settings: Arc::new(RwLock::new(ServerSettings {
            preferred_audio_language: "eng".into(),
            ..Default::default()
        })),
        scans: Jobs::new("scan"),
        thumb_jobs: Jobs::new("thumbnails"),
        marker_jobs: Jobs::new("skip markers"),
        subtitle_jobs: Jobs::new("subtitles"),
        ocr: None,
        keyframe_jobs: Jobs::new("keyframe index"),
        hls: crate::hls::HlsSessions::default(),
        watchers: Arc::new(Mutex::new(HashMap::new())),
        changes,
        probe_slots: Arc::new(Semaphore::new(1)),
        thumb_slots: Arc::new(Semaphore::new(1)),
        active_streams: Arc::default(),
        active_transcodes: Arc::default(),
        started: std::time::Instant::now(),
        login_failures: Arc::default(),
        share_hits: Arc::default(),
        share_transcodes: Arc::default(),
        now_playing: Arc::default(),
        http: reqwest::Client::new(),
        tls_fingerprint: None,
    }
}

pub(crate) async fn user(state: &AppState, name: &str, admin: bool, pin: Option<&str>) -> i64 {
    let pw = admin.then(|| crate::auth::hash_password("password1").unwrap());
    let pin = pin.map(|p| crate::auth::hash_password(p).unwrap());
    users::create(
        &state.db,
        &NewUser {
            name,
            is_admin: admin,
            avatar: "ember",
            password_hash: pw.as_deref(),
            pin_hash: pin.as_deref(),
        },
    )
    .await
    .unwrap()
    .unwrap()
}

pub(crate) async fn session(state: &AppState, user_id: i64, lan_only: bool) -> String {
    let (token, hash) = new_token();
    users::create_session_scoped(&state.db, &hash, user_id, 3600, lan_only)
        .await
        .unwrap();
    token
}

pub(crate) async fn movie(state: &AppState, lib: i64, title: &str, rating: Option<&str>) -> i64 {
    let id = items::find_or_create(
        &state.db,
        lib,
        NewItem::Movie {
            title,
            year: Some(2020),
        },
    )
    .await
    .unwrap();
    items::set_manual_content_rating(&state.db, id, rating)
        .await
        .unwrap();
    id
}

/// A file with an English and a Japanese audio track, plus one English subtitle.
async fn file(state: &AppState, item: i64, path: &str) -> i64 {
    let id = sqlx::query(
        "INSERT INTO files (item_id, path, size, mtime, fingerprint, container, probe_status)
         VALUES (?1, ?2, 1, 1, 'fp', 'matroska', 'ok')",
    )
    .bind(item)
    .bind(path)
    .execute(&state.db)
    .await
    .unwrap()
    .last_insert_rowid();
    for (idx, kind, codec, lang) in [
        (0, "video", "h264", None),
        (1, "audio", "aac", Some("eng")),
        (2, "audio", "aac", Some("jpn")),
        (3, "subtitle", "subrip", Some("eng")),
    ] {
        sqlx::query(
            "INSERT INTO streams (file_id, idx, type, codec, language) VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .bind(id)
        .bind(idx)
        .bind(kind)
        .bind(codec)
        .bind(lang)
        .execute(&state.db)
        .await
        .unwrap();
    }
    id
}

async fn world() -> World {
    let state = state().await;
    let db = &state.db;
    let admin_id = user(&state, "Admin", true, None).await;
    let kid_id = user(&state, "Kid", false, None).await;
    let lib = libraries::create(db, "Films", "movie", &["/media/films".into()])
        .await
        .unwrap();
    let pg_movie = movie(&state, lib, "Paddington", Some("PG")).await;
    let r_movie = movie(&state, lib, "Heat", Some("R")).await;
    let unrated_movie = movie(&state, lib, "Home Video", None).await;
    let show = items::find_or_create(
        db,
        lib,
        NewItem::Show {
            title: "Bluey",
            year: None,
        },
    )
    .await
    .unwrap();
    items::set_manual_content_rating(db, show, Some("TV-Y"))
        .await
        .unwrap();
    let season1 = items::find_or_create(
        db,
        lib,
        NewItem::Season {
            show_id: show,
            season: 1,
        },
    )
    .await
    .unwrap();
    let season2 = items::find_or_create(
        db,
        lib,
        NewItem::Season {
            show_id: show,
            season: 2,
        },
    )
    .await
    .unwrap();
    let ep1 = items::find_or_create(
        db,
        lib,
        NewItem::Episode {
            season_id: season1,
            episode: 1,
            title: "Magic Xylophone",
        },
    )
    .await
    .unwrap();
    let ep2 = items::find_or_create(
        db,
        lib,
        NewItem::Episode {
            season_id: season2,
            episode: 1,
            title: "Dance Mode",
        },
    )
    .await
    .unwrap();
    let mature = items::find_or_create(
        db,
        lib,
        NewItem::Show {
            title: "The Wire",
            year: None,
        },
    )
    .await
    .unwrap();
    items::set_manual_content_rating(db, mature, Some("TV-MA"))
        .await
        .unwrap();
    let mature_s1 = items::find_or_create(
        db,
        lib,
        NewItem::Season {
            show_id: mature,
            season: 1,
        },
    )
    .await
    .unwrap();
    let other_show_ep = items::find_or_create(
        db,
        lib,
        NewItem::Episode {
            season_id: mature_s1,
            episode: 1,
            title: "The Target",
        },
    )
    .await
    .unwrap();
    profiles::set(
        db,
        kid_id,
        &Prefs {
            max_rating: Some(10),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let admin = session(&state, admin_id, false).await;
    let kid = session(&state, kid_id, false).await;
    World {
        app: api_router().with_state(state.clone()),
        state,
        admin,
        kid,
        kid_id,
        lib,
        pg_movie,
        r_movie,
        unrated_movie,
        show,
        season1,
        season2,
        ep1,
        ep2,
        other_show_ep,
    }
}

pub(crate) async fn call(
    app: &Router,
    method: Method,
    uri: &str,
    token: Option<&str>,
    peer: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut req = Request::builder().method(method).uri(uri);
    if let Some(t) = token {
        req = req.header("authorization", format!("Bearer {t}"));
    }
    if body.is_some() {
        req = req.header("content-type", "application/json");
    }
    let mut req = req
        .body(body.map_or(Body::empty(), |b| Body::from(b.to_string())))
        .unwrap();
    req.extensions_mut()
        .insert(ConnectInfo(peer.parse::<SocketAddr>().unwrap()));
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

async fn get(w: &World, uri: &str, token: &str) -> (StatusCode, Value) {
    call(&w.app, Method::GET, uri, Some(token), LAN, None).await
}

fn ids(v: &Value) -> Vec<i64> {
    v.as_array()
        .map(|a| a.iter().filter_map(|c| c["id"].as_i64()).collect())
        .unwrap_or_default()
}

// ---- content limits ----

#[tokio::test]
async fn content_limit_filters_every_listing() {
    let w = world().await;
    let (s, list) = get(&w, &format!("/libraries/{}/items", w.lib), &w.kid).await;
    assert_eq!(s, StatusCode::OK);
    let seen = ids(&list);
    assert!(seen.contains(&w.pg_movie));
    assert!(seen.contains(&w.show));
    assert!(!seen.contains(&w.r_movie), "R over a limit of 10");
    assert!(
        !seen.contains(&w.unrated_movie),
        "unrated hidden by default"
    );

    let (_, admin_list) = get(&w, &format!("/libraries/{}/items", w.lib), &w.admin).await;
    assert!(
        ids(&admin_list).contains(&w.r_movie),
        "admins are never limited"
    );

    let (_, hits) = get(&w, "/search?q=e", &w.kid).await;
    let hits = ids(&hits);
    assert!(!hits.contains(&w.r_movie) && !hits.contains(&w.other_show_ep));

    let (_, home) = get(&w, "/home", &w.kid).await;
    let shelved: Vec<i64> = home["shelves"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|s| ids(&s["items"]))
        .collect();
    assert!(shelved.contains(&w.pg_movie));
    assert!(!shelved.contains(&w.r_movie) && !shelved.contains(&w.other_show_ep));
}

#[tokio::test]
async fn content_limit_blocks_direct_paths() {
    let w = world().await;
    let r_file = file(&w.state, w.r_movie, "/media/films/heat.mkv").await;
    let ep_file = file(&w.state, w.other_show_ep, "/media/wire/s1e1.mkv").await;
    let sub: i64 =
        sqlx::query_scalar("SELECT id FROM streams WHERE file_id = ?1 AND type = 'subtitle'")
            .bind(r_file)
            .fetch_one(&w.state.db)
            .await
            .unwrap();
    for uri in [
        format!("/items/{}", w.r_movie),
        format!("/items/{}", w.other_show_ep),
        format!("/items/{}/thumb", w.r_movie),
        format!("/files/{r_file}/plan"),
        format!("/stream/{r_file}"),
        format!("/stream/{ep_file}"),
        format!("/subtitles/{sub}"),
    ] {
        assert_eq!(
            get(&w, &uri, &w.kid).await.0,
            StatusCode::NOT_FOUND,
            "{uri}"
        );
    }
    for (uri, body) in [
        (
            format!("/items/{}/progress", w.r_movie),
            json!({"position_s": 5}),
        ),
        (
            format!("/items/{}/watched", w.r_movie),
            json!({"watched": true}),
        ),
        (
            format!("/items/{}/track-choice", w.r_movie),
            json!({"audio": 1}),
        ),
    ] {
        let (s, _) = call(&w.app, Method::PUT, &uri, Some(&w.kid), LAN, Some(body)).await;
        assert_eq!(s, StatusCode::NOT_FOUND, "{uri}");
    }
    assert_eq!(
        get(&w, &format!("/files/{r_file}/plan"), &w.admin).await.0,
        StatusCode::OK
    );
    // Episodes inherit their show's rating; allowed shows stay reachable.
    assert_eq!(
        get(&w, &format!("/items/{}", w.ep1), &w.kid).await.0,
        StatusCode::OK
    );
}

#[tokio::test]
async fn content_limit_covers_cached_artwork() {
    let w = world().await;
    for (item, name) in [(w.pg_movie, "ok.jpg"), (w.r_movie, "restricted.jpg")] {
        sqlx::query("UPDATE items SET poster = ?2 WHERE id = ?1")
            .bind(item)
            .bind(format!("w300/{name}"))
            .execute(&w.state.db)
            .await
            .unwrap();
    }
    // Cached already, so an allowed request never reaches TMDB.
    for size in ["w300", "w1280"] {
        let dir = w.state.cache_dir("images").join(size);
        std::fs::create_dir_all(&dir).unwrap();
        for name in ["ok.jpg", "restricted.jpg", "cast.jpg"] {
            std::fs::write(dir.join(name), b"jpg").unwrap();
        }
    }
    for (uri, kid) in [
        ("/images/w300/ok.jpg", StatusCode::OK),
        // The same art at another size is the same title's art.
        ("/images/w1280/restricted.jpg", StatusCode::NOT_FOUND),
        ("/images/w300/restricted.jpg", StatusCode::NOT_FOUND),
        // Used by no title (cast photo, request poster): not tied to a limit.
        ("/images/w300/cast.jpg", StatusCode::OK),
    ] {
        assert_eq!(get(&w, uri, &w.kid).await.0, kid, "kid {uri}");
    }
    assert_eq!(
        get(&w, "/images/w300/restricted.jpg", &w.admin).await.0,
        StatusCode::OK
    );
}

#[tokio::test]
async fn unrated_follows_the_profile_setting() {
    let w = world().await;
    let uri = format!("/items/{}", w.unrated_movie);
    assert_eq!(get(&w, &uri, &w.kid).await.0, StatusCode::NOT_FOUND);
    profiles::set(
        &w.state.db,
        w.kid_id,
        &Prefs {
            max_rating: Some(10),
            allow_unrated: true,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(get(&w, &uri, &w.kid).await.0, StatusCode::OK);
}

#[tokio::test]
async fn only_admins_set_limits_and_ratings() {
    let w = world().await;
    let (s, _) = call(
        &w.app,
        Method::PUT,
        &format!("/items/{}/content-rating", w.r_movie),
        Some(&w.kid),
        LAN,
        Some(json!({"content_rating": "G"})),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN);
    // A profile's own preferences endpoint changes languages only, never its limit.
    let (s, prefs) = call(
        &w.app,
        Method::PUT,
        "/profile/preferences",
        Some(&w.kid),
        LAN,
        Some(json!({"audio_language": "jpn", "max_rating": null, "lan_unlock": true})),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(prefs["max_rating"], 10);
    assert_eq!(prefs["lan_unlock"], false);
    let (s, _) = call(
        &w.app,
        Method::PUT,
        "/profile/preferences",
        Some(&w.kid),
        LAN,
        Some(json!({"audio_language": "klingon"})),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
}

// ---- LAN-only profiles ----

async fn login(w: &World, user_id: i64, pin: Option<&str>, peer: &str) -> (StatusCode, Value) {
    let body = match pin {
        Some(p) => json!({"user_id": user_id, "pin": p}),
        None => json!({"user_id": user_id}),
    };
    call(&w.app, Method::POST, "/auth/login", None, peer, Some(body)).await
}

#[tokio::test]
async fn unlocked_profile_only_from_lan() {
    let w = world().await;
    assert_eq!(login(&w, w.kid_id, None, LAN).await.0, StatusCode::OK);
    assert_eq!(
        login(&w, w.kid_id, None, "[fd00::5]:1").await.0,
        StatusCode::OK
    );
    assert_eq!(
        login(&w, w.kid_id, None, WAN).await.0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        login(&w, w.kid_id, Some("1234"), WAN).await.0,
        StatusCode::FORBIDDEN
    );
    // A same-host reverse proxy that is not configured as trusted: forwarded = remote.
    let mut req = Request::builder()
        .method(Method::POST)
        .uri("/auth/login")
        .header("content-type", "application/json")
        .header("x-forwarded-for", "192.168.1.9")
        .body(Body::from(json!({"user_id": w.kid_id}).to_string()))
        .unwrap();
    req.extensions_mut()
        .insert(ConnectInfo("127.0.0.1:9".parse::<SocketAddr>().unwrap()));
    let res = w.app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    let (_, picker) = call(&w.app, Method::GET, "/auth/profiles", None, WAN, None).await;
    let kid = picker
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == w.kid_id)
        .unwrap();
    assert_eq!(kid["home_only"], true);
    assert_eq!(kid["opens_without_lock"], false);
}

#[tokio::test]
async fn lan_unlock_skips_pin_only_at_home() {
    let w = world().await;
    let teen = user(&w.state, "Teen", false, Some("2468")).await;
    // Without lan_unlock the PIN is always needed.
    assert_eq!(login(&w, teen, None, LAN).await.0, StatusCode::UNAUTHORIZED);
    profiles::set(
        &w.state.db,
        teen,
        &Prefs {
            lan_unlock: true,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let (s, home) = login(&w, teen, None, LAN).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(login(&w, teen, None, WAN).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(
        login(&w, teen, Some("0000"), LAN).await.0,
        StatusCode::UNAUTHORIZED,
        "a typed wrong PIN never signs in"
    );
    let (s, away) = login(&w, teen, Some("2468"), WAN).await;
    assert_eq!(s, StatusCode::OK);

    // The session opened without the PIN only works from the LAN.
    let home_token = home["token"].as_str().unwrap();
    let me = |peer: &'static str, token: String| {
        let app = w.app.clone();
        async move {
            call(&app, Method::GET, "/auth/me", Some(&token), peer, None)
                .await
                .0
        }
    };
    assert_eq!(me(LAN, home_token.into()).await, StatusCode::OK);
    assert_eq!(me(WAN, home_token.into()).await, StatusCode::UNAUTHORIZED);
    let away_token = away["token"].as_str().unwrap();
    assert_eq!(me(WAN, away_token.into()).await, StatusCode::OK);

    // Admins never skip their password, whatever their preferences say.
    let admin_id = users::find_by_name(&w.state.db, "Admin")
        .await
        .unwrap()
        .unwrap()
        .id;
    profiles::set(
        &w.state.db,
        admin_id,
        &Prefs {
            lan_unlock: true,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(
        login(&w, admin_id, None, LAN).await.0,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn lan_session_rejected_without_peer() {
    let w = world().await;
    let token = session(&w.state, w.kid_id, true).await;
    let req = Request::builder()
        .uri("/auth/me")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let res = w.app.clone().oneshot(req).await.unwrap();
    assert_eq!(
        res.status(),
        StatusCode::UNAUTHORIZED,
        "unknown origin is remote"
    );
}

// ---- share links ----

async fn share(w: &World, item: i64, hours: i64) -> (StatusCode, Value) {
    call(
        &w.app,
        Method::POST,
        "/shares",
        Some(&w.admin),
        LAN,
        Some(json!({"item_id": item, "expires_in_hours": hours, "label": "Grandma"})),
    )
    .await
}

#[tokio::test]
async fn share_link_is_scoped() {
    let w = world().await;
    let ep1_file = file(&w.state, w.ep1, "/media/bluey/s1e1.mp4").await;
    let ep2_file = file(&w.state, w.ep2, "/media/bluey/s2e1.mp4").await;
    let pg_file = file(&w.state, w.pg_movie, "/media/films/paddington.mp4").await;

    let (s, _) = call(
        &w.app,
        Method::POST,
        "/shares",
        Some(&w.kid),
        LAN,
        Some(json!({"item_id": w.season1, "expires_in_hours": 24})),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "only admins share");
    assert_eq!(share(&w, w.season1, 0).await.0, StatusCode::BAD_REQUEST);

    let (s, created) = share(&w, w.season1, 24).await;
    assert_eq!(s, StatusCode::CREATED);
    let token = created["token"].as_str().unwrap().to_string();
    assert_eq!(token.len(), 80);
    let stored: String = sqlx::query_scalar("SELECT token_hash FROM share_links")
        .fetch_one(&w.state.db)
        .await
        .unwrap();
    assert!(!stored.contains(&token[16..]), "only a hash is stored");

    let open = |uri: String| {
        let app = w.app.clone();
        async move { call(&app, Method::GET, &uri, None, WAN, None).await }
    };
    let (s, landing) = open(format!("/share/{token}")).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(landing["item"]["id"], w.season1);
    assert_eq!(ids(&landing["children"]), vec![w.ep1]);
    assert!(
        landing.to_string().find("/media/").is_none(),
        "no server paths"
    );

    assert_eq!(
        open(format!("/share/{token}/items/{}", w.ep1)).await.0,
        StatusCode::OK
    );
    for out in [w.ep2, w.season2, w.show, w.pg_movie, w.other_show_ep] {
        assert_eq!(
            open(format!("/share/{token}/items/{out}")).await.0,
            StatusCode::NOT_FOUND,
            "item {out} is outside the link"
        );
    }
    assert_eq!(
        open(format!("/share/{token}/files/{ep1_file}/plan"))
            .await
            .0,
        StatusCode::OK
    );
    for f in [ep2_file, pg_file] {
        assert_eq!(
            open(format!("/share/{token}/files/{f}/plan")).await.0,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            open(format!("/share/{token}/stream/{f}")).await.0,
            StatusCode::NOT_FOUND
        );
    }

    // The link is no session: the normal API refuses it.
    for t in [token.as_str(), &token[16..]] {
        let (s, _) = call(&w.app, Method::GET, "/home", Some(t), LAN, None).await;
        assert_eq!(s, StatusCode::UNAUTHORIZED);
    }
    // A right selector with a wrong secret is the same 404.
    let forged = format!("{}{}", &token[..16], "0".repeat(64));
    assert_eq!(
        open(format!("/share/{forged}")).await.0,
        StatusCode::NOT_FOUND
    );

    // Revocation ends it at once; admins list it as revoked.
    let id = created["share"]["id"].as_i64().unwrap();
    let (s, _) = call(
        &w.app,
        Method::DELETE,
        &format!("/shares/{id}"),
        Some(&w.kid),
        LAN,
        None,
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN);
    let (s, _) = call(
        &w.app,
        Method::DELETE,
        &format!("/shares/{id}"),
        Some(&w.admin),
        LAN,
        None,
    )
    .await;
    assert_eq!(s, StatusCode::NO_CONTENT);
    assert_eq!(
        open(format!("/share/{token}")).await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        open(format!("/share/{token}/files/{ep1_file}/plan"))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    let (_, list) = get(&w, "/shares", &w.admin).await;
    assert!(list[0]["revoked_at"].is_i64());
    assert!(list[0].get("token_hash").is_none());
}

#[tokio::test]
async fn share_link_expires() {
    let w = world().await;
    let (_, created) = share(&w, w.pg_movie, 1).await;
    let token = created["token"].as_str().unwrap();
    sqlx::query("UPDATE share_links SET expires_at = 1")
        .execute(&w.state.db)
        .await
        .unwrap();
    let (s, _) = call(
        &w.app,
        Method::GET,
        &format!("/share/{token}"),
        None,
        WAN,
        None,
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND);
    assert!(
        !shares::is_active(&w.state.db, created["share"]["id"].as_i64().unwrap())
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn share_link_admits_a_limited_number_of_viewers() {
    let w = world().await;
    let make = |body: Value| {
        call(
            &w.app,
            Method::POST,
            "/shares",
            Some(&w.admin),
            LAN,
            Some(body),
        )
    };
    let (s, created) =
        make(json!({"item_id": w.pg_movie, "expires_in_hours": 1, "max_uses": 1})).await;
    assert_eq!(s, StatusCode::CREATED);
    assert_eq!(created["share"]["max_uses"], 1);
    let uri = format!("/share/{}", created["token"].as_str().unwrap());
    let open = |peer: &'static str| call(&w.app, Method::GET, &uri, None, peer, None);
    assert_eq!(open(WAN).await.0, StatusCode::OK);
    // The same viewer reloading costs nothing; a second address finds nothing.
    assert_eq!(open(WAN).await.0, StatusCode::OK);
    assert_eq!(open("198.51.100.9:1").await.0, StatusCode::NOT_FOUND);
    let (_, list) = call(&w.app, Method::GET, "/shares", Some(&w.admin), LAN, None).await;
    assert_eq!(list[0]["viewers"], 1);

    for bad in [0, 101, -3] {
        let (s, _) =
            make(json!({"item_id": w.pg_movie, "expires_in_hours": 1, "max_uses": bad})).await;
        assert_eq!(s, StatusCode::BAD_REQUEST, "{bad}");
    }
    // No limit by default.
    let (_, open_link) = share(&w, w.pg_movie, 1).await;
    assert_eq!(open_link["share"]["max_uses"], Value::Null);
}

#[tokio::test]
async fn share_links_rate_limit_each_address() {
    let w = world().await;
    let (_, created) = share(&w, w.pg_movie, 1).await;
    let uri = format!("/share/{}", created["token"].as_str().unwrap());
    let peer = "198.51.100.20:1";
    for _ in 0..300 {
        assert_eq!(
            call(&w.app, Method::GET, &uri, None, peer, None).await.0,
            StatusCode::OK
        );
    }
    assert_eq!(
        call(&w.app, Method::GET, &uri, None, peer, None).await.0,
        StatusCode::TOO_MANY_REQUESTS
    );
    // Another address is unaffected.
    assert_eq!(
        call(&w.app, Method::GET, &uri, None, WAN, None).await.0,
        StatusCode::OK
    );
}

// ---- export / import ----

#[tokio::test]
async fn export_has_no_secrets_unless_asked() {
    let w = world().await;
    let teen = user(&w.state, "Teen", false, Some("2468")).await;
    let teen_token = session(&w.state, teen, false).await;
    items::set_progress(&w.state.db, teen, w.ep1, 42.0, false)
        .await
        .unwrap();
    let (s, export) = get(&w, "/profile/export", &teen_token).await;
    assert_eq!(s, StatusCode::OK);
    assert!(export.get("lock").is_none());
    assert!(!export.to_string().contains("argon2"));
    assert_eq!(export["watch"][0]["kind"], "episode");
    assert_eq!(export["watch"][0]["title"], "Bluey");

    let (s, _) = get(
        &w,
        &format!("/users/{teen}/export?include_lock=true"),
        &teen_token,
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN);
    let (_, with_lock) = get(
        &w,
        &format!("/users/{teen}/export?include_lock=true"),
        &w.admin,
    )
    .await;
    assert!(
        with_lock["lock"]["pin_hash"]
            .as_str()
            .unwrap()
            .starts_with("$argon2")
    );
}

#[tokio::test]
async fn import_is_validated_and_cannot_lift_limits() {
    let w = world().await;
    let export = json!({
        "format": "kinema.profile", "version": 1, "exported_at": 1,
        "profile": {"name": "Elsewhere", "avatar": null},
        "preferences": {"audio_language": "jpn", "subtitle_language": null, "subtitle_mode": "always",
                        "max_rating": null, "allow_unrated": true, "lan_unlock": true},
        "watch": [
            {"kind": "movie", "tmdb_id": null, "tvmaze_id": null, "title": "Paddington", "year": 2020,
             "season_no": null, "episode_no": null, "position_s": 300.0, "watched": false,
             "play_count": 0, "last_played_at": 100},
            {"kind": "episode", "tmdb_id": null, "tvmaze_id": null, "title": "Bluey", "year": null,
             "season_no": 2, "episode_no": 1, "position_s": 0.0, "watched": true,
             "play_count": 1, "last_played_at": 100},
            {"kind": "movie", "tmdb_id": 999, "tvmaze_id": null, "title": "Not Here", "year": 1999,
             "season_no": null, "episode_no": null, "position_s": 1.0, "watched": false,
             "play_count": 0, "last_played_at": 100}
        ],
        "lock": {"password_hash": null, "pin_hash": "$argon2id$v=19$m=19456,t=2,p=1$c2FsdHNhbHQ$aGFzaGhhc2hoYXNoaGFzaA"}
    });
    let (s, r) = call(
        &w.app,
        Method::POST,
        "/profile/import",
        Some(&w.kid),
        LAN,
        Some(export.clone()),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(r["imported"], 2);
    assert_eq!(r["unmatched"], 1);
    assert_eq!(r["lock_applied"], false);
    let prefs = profiles::get(&w.state.db, w.kid_id).await.unwrap();
    assert_eq!(prefs.audio_language.as_deref(), Some("jpn"));
    assert_eq!(
        prefs.max_rating,
        Some(10),
        "own import never lifts the limit"
    );
    assert!(!prefs.allow_unrated && !prefs.lan_unlock);
    let kid = users::get(&w.state.db, w.kid_id).await.unwrap().unwrap();
    assert!(!kid.has_pin, "own import never sets a lock");
    let card = items::card(&w.state.db, w.kid_id, w.pg_movie)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(card.position_s, Some(300.0));

    for bad in [
        json!({"format": "other", "version": 1, "exported_at": 1, "profile": {"name": "x"}, "preferences": {}, "watch": []}),
        json!({"format": "kinema.profile", "version": 2, "exported_at": 1, "profile": {"name": "x"}, "preferences": {}, "watch": []}),
        json!({"format": "kinema.profile", "version": 1, "exported_at": 1, "profile": {"name": "x"},
               "preferences": {"audio_language": "zzz"}, "watch": []}),
        json!({"format": "kinema.profile", "version": 1, "exported_at": 1, "profile": {"name": "x"}, "preferences": {},
               "watch": [{"kind": "movie", "title": "Heat", "position_s": -5.0, "watched": false, "play_count": 0}]}),
        json!({"format": "kinema.profile", "version": 1, "exported_at": 1, "profile": {"name": "x"}, "preferences": {},
               "watch": [{"kind": "episode", "title": "Bluey", "position_s": 1.0, "watched": false, "play_count": 0}]}),
    ] {
        let (s, _) = call(
            &w.app,
            Method::POST,
            "/profile/import",
            Some(&w.kid),
            LAN,
            Some(bad),
        )
        .await;
        assert!(s.is_client_error(), "{s}");
    }

    // Admin import into the kid applies the household rules; the lock only on request,
    // and only a well-formed argon2 hash.
    let (s, _) = call(
        &w.app,
        Method::POST,
        &format!("/users/{}/import", w.kid_id),
        Some(&w.kid),
        LAN,
        Some(export.clone()),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN);
    let (s, r) = call(
        &w.app,
        Method::POST,
        &format!("/users/{}/import", w.kid_id),
        Some(&w.admin),
        LAN,
        Some(export.clone()),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(r["lock_applied"], false);
    assert!(
        profiles::get(&w.state.db, w.kid_id)
            .await
            .unwrap()
            .lan_unlock
    );
    let mut garbage = export.clone();
    garbage["lock"]["pin_hash"] = json!("not-a-hash");
    let (s, _) = call(
        &w.app,
        Method::POST,
        &format!("/users/{}/import?apply_lock=true", w.kid_id),
        Some(&w.admin),
        LAN,
        Some(garbage),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let (s, r) = call(
        &w.app,
        Method::POST,
        &format!("/users/{}/import?apply_lock=true", w.kid_id),
        Some(&w.admin),
        LAN,
        Some(export),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(r["lock_applied"], true);
    assert!(
        users::get(&w.state.db, w.kid_id)
            .await
            .unwrap()
            .unwrap()
            .has_pin
    );
}

// ---- languages ----

#[tokio::test]
async fn profile_languages_drive_default_tracks() {
    let w = world().await;
    let f = file(&w.state, w.pg_movie, "/media/films/paddington.mkv").await;
    let uri = format!("/files/{f}/plan");
    let (s, plan) = get(&w, &uri, &w.kid).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(plan["audio"], 1, "server default: English");
    assert!(plan["subtitle"].is_null(), "server subtitle mode is off");

    let (s, _) = call(
        &w.app,
        Method::PUT,
        "/profile/preferences",
        Some(&w.kid),
        LAN,
        Some(json!({"audio_language": "jpn", "subtitle_language": "eng", "subtitle_mode": "foreign"})),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let (_, plan) = get(&w, &uri, &w.kid).await;
    assert_eq!(plan["audio"], 2, "profile: Japanese");
    assert!(
        plan["subtitle"].is_null(),
        "audio is the preferred language"
    );

    // English audio for a Japanese-preferring profile with "foreign" subtitles: subtitles on.
    let (_, plan) = get(&w, &format!("{uri}?audio=1"), &w.kid).await;
    assert!(plan["subtitle"].is_i64());

    // Another profile still follows the server.
    let (_, plan) = get(&w, &uri, &w.admin).await;
    assert_eq!(plan["audio"], 1);
}
