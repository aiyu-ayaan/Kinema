//! Device approval, watch activity, title requests and admin notifications over the real
//! router and an in-memory database. Same harness as `access_tests`.

use crate::access_tests::{LAN, WAN, call, movie, session, state, user};
use crate::auth::hash_password;
use crate::notify::{self, MASK};
use crate::routes::api_router;
use crate::state::AppState;
use axum::Router;
use axum::http::{Method, StatusCode};
use kinema_db::users::{self, NewUser};
use kinema_db::{devices, libraries};
use serde_json::{Value, json};
use std::time::Duration;

struct Env {
    state: AppState,
    app: Router,
    admin: String,
    admin_id: i64,
}

async fn env() -> Env {
    let state = state().await;
    let admin_id = user(&state, "Admin", true, None).await;
    let admin = session(&state, admin_id, false).await;
    Env {
        app: api_router().with_state(state.clone()),
        state,
        admin,
        admin_id,
    }
}

/// A non-admin profile with a password, so it can sign in from outside the LAN.
async fn member(e: &Env, name: &str) -> i64 {
    let hash = hash_password("secret12").unwrap();
    users::create(
        &e.state.db,
        &NewUser {
            name,
            is_admin: false,
            avatar: "ember",
            password_hash: Some(&hash),
            pin_hash: None,
        },
    )
    .await
    .unwrap()
    .unwrap()
}

async fn sign_in(
    e: &Env,
    user_id: i64,
    password: &str,
    client: &str,
    kind: &str,
) -> (StatusCode, Value) {
    let body = json!({
        "user_id": user_id,
        "password": password,
        "device": {"id": client, "kind": kind, "name": format!("{kind} {client}")},
    });
    call(&e.app, Method::POST, "/auth/login", None, WAN, Some(body)).await
}

async fn member_in(e: &Env, id: i64, client: &str, kind: &str) -> (StatusCode, Value) {
    sign_in(e, id, "secret12", client, kind).await
}

fn token(v: &Value) -> String {
    v["token"].as_str().expect("a session token").to_string()
}

fn poll_token(v: &Value) -> String {
    v["approval_required"]["poll_token"]
        .as_str()
        .expect("approval_required with a poll token")
        .to_string()
}

async fn admin_call(
    e: &Env,
    method: Method,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    call(&e.app, method, uri, Some(&e.admin), LAN, body).await
}

async fn poll(e: &Env, poll_token: &str) -> (StatusCode, Value) {
    let body = json!({"poll_token": poll_token});
    call(
        &e.app,
        Method::POST,
        "/auth/device/poll",
        None,
        WAN,
        Some(body),
    )
    .await
}

async fn me(e: &Env, token: &str) -> StatusCode {
    call(&e.app, Method::GET, "/auth/me", Some(token), WAN, None)
        .await
        .0
}

async fn device_id(e: &Env, user_id: i64, client: &str) -> i64 {
    devices::find(&e.state.db, user_id, client)
        .await
        .unwrap()
        .expect("device row")
        .id
}

// ---- device approval ----

#[tokio::test]
async fn second_web_device_waits_and_approval_yields_one_session() {
    let e = env().await;
    let mia = member(&e, "Mia").await;

    let (s, first) = member_in(&e, mia, "laptop-0001", "web").await;
    assert_eq!(s, StatusCode::OK, "first web login is free");
    assert_eq!(me(&e, &token(&first)).await, StatusCode::OK);

    let (s, second) = member_in(&e, mia, "tablet-0002", "web").await;
    assert_eq!(s, StatusCode::OK);
    assert!(second.get("token").is_none(), "no session while pending");
    let pt = poll_token(&second);

    assert_eq!(
        poll(&e, &pt).await,
        (StatusCode::OK, json!({"pending": true}))
    );

    // Waiting requests are listed first, with the profile's name.
    let (_, list) = admin_call(&e, Method::GET, "/devices", None).await;
    assert_eq!(list[0]["status"], "pending");
    assert_eq!(list[0]["user_name"], "Mia");
    let id = list[0]["id"].as_i64().unwrap();

    let (s, approved) = admin_call(&e, Method::POST, &format!("/devices/{id}/approve"), None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(approved["status"], "approved");

    let (s, session) = poll(&e, &pt).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(me(&e, &token(&session)).await, StatusCode::OK);
    // The poll secret is spent.
    assert_eq!(poll(&e, &pt).await.0, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn mobile_is_free_and_desktop_shares_the_web_slot() {
    let e = env().await;
    let mia = member(&e, "Mia").await;
    assert_eq!(
        member_in(&e, mia, "laptop-0001", "web").await.0,
        StatusCode::OK
    );
    let (_, phone) = member_in(&e, mia, "phone-0001", "mobile").await;
    assert!(phone.get("token").is_some(), "mobile has its own slot");
    let (_, desktop) = member_in(&e, mia, "desk-0001", "desktop").await;
    assert!(desktop.get("approval_required").is_some());
    let (_, phone2) = member_in(&e, mia, "phone-0002", "mobile").await;
    assert!(phone2.get("approval_required").is_some());
}

#[tokio::test]
async fn same_client_can_sign_in_again_and_admins_are_never_limited() {
    let e = env().await;
    let mia = member(&e, "Mia").await;
    assert!(
        member_in(&e, mia, "laptop-0001", "web")
            .await
            .1
            .get("token")
            .is_some()
    );
    assert!(
        member_in(&e, mia, "tablet-0002", "web")
            .await
            .1
            .get("approval_required")
            .is_some()
    );
    let (_, again) = member_in(&e, mia, "laptop-0001", "web").await;
    assert!(
        again.get("token").is_some(),
        "approved device signs in again"
    );

    for client in ["a-device-01", "a-device-02", "a-device-03"] {
        let (s, v) = sign_in(&e, e.admin_id, "password1", client, "web").await;
        assert_eq!(s, StatusCode::OK);
        assert!(v.get("token").is_some(), "admin on {client}");
    }
}

#[tokio::test]
async fn lan_one_tap_session_is_not_limited() {
    let e = env().await;
    let kid = user(&e.state, "Kid", false, None).await;
    for client in ["kid-dev-0001", "kid-dev-0002", "kid-dev-0003"] {
        let body = json!({"user_id": kid, "device": {"id": client, "kind": "web", "name": "tv"}});
        let (s, v) = call(&e.app, Method::POST, "/auth/login", None, LAN, Some(body)).await;
        assert_eq!(s, StatusCode::OK);
        assert!(v.get("token").is_some());
    }
    assert!(
        devices::list(&e.state.db).await.unwrap().is_empty(),
        "no device rows"
    );
}

#[tokio::test]
async fn removing_or_declining_a_device_ends_it() {
    let e = env().await;
    let mia = member(&e, "Mia").await;
    let (_, first) = member_in(&e, mia, "laptop-0001", "web").await;
    let tok = token(&first);
    let id = device_id(&e, mia, "laptop-0001").await;
    let (s, _) = admin_call(&e, Method::DELETE, &format!("/devices/{id}"), None).await;
    assert_eq!(s, StatusCode::NO_CONTENT);
    assert_eq!(
        me(&e, &tok).await,
        StatusCode::UNAUTHORIZED,
        "session ended with the device"
    );

    // The freed slot lets the next web device in; one more waits and is declined.
    assert!(
        member_in(&e, mia, "laptop-0002", "web")
            .await
            .1
            .get("token")
            .is_some()
    );
    let (_, waiting) = member_in(&e, mia, "tablet-0003", "web").await;
    let pt = poll_token(&waiting);
    let did = device_id(&e, mia, "tablet-0003").await;
    admin_call(&e, Method::DELETE, &format!("/devices/{did}"), None).await;
    assert_eq!(poll(&e, &pt).await.0, StatusCode::NOT_FOUND);

    // An admin cannot remove their own last device (it would end this very session).
    let (_, adm) = sign_in(&e, e.admin_id, "password1", "admin-dev-01", "web").await;
    let aid = device_id(&e, e.admin_id, "admin-dev-01").await;
    let (s, _) = call(
        &e.app,
        Method::DELETE,
        &format!("/devices/{aid}"),
        Some(&token(&adm)),
        WAN,
        None,
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn device_endpoints_are_admin_only() {
    let e = env().await;
    let mia = member(&e, "Mia").await;
    let tok = session(&e.state, mia, false).await;
    for (m, uri) in [
        (Method::GET, "/devices"),
        (Method::POST, "/devices/1/approve"),
        (Method::DELETE, "/devices/1"),
    ] {
        let (s, _) = call(&e.app, m, uri, Some(&tok), LAN, None).await;
        assert_eq!(s, StatusCode::FORBIDDEN, "{uri}");
    }
}

#[tokio::test]
async fn pending_requests_per_profile_are_capped() {
    let e = env().await;
    let mia = member(&e, "Mia").await;
    member_in(&e, mia, "laptop-0001", "web").await;
    for n in 0..5 {
        let (_, v) = member_in(&e, mia, &format!("extra-dev-{n}0"), "web").await;
        assert!(v.get("approval_required").is_some(), "request {n}");
    }
    let (s, _) = member_in(&e, mia, "extra-dev-60", "web").await;
    assert_eq!(s, StatusCode::TOO_MANY_REQUESTS);
    // A device already waiting may retry (new poll secret) without counting again.
    let (s, _) = member_in(&e, mia, "extra-dev-00", "web").await;
    assert_eq!(s, StatusCode::OK);
}

// ---- activity ----

#[tokio::test]
async fn progress_reports_become_watch_time_and_now_playing() {
    let e = env().await;
    let mia = member(&e, "Mia").await;
    let lib = libraries::create(&e.state.db, "Films", "movie", &["/media/films".into()])
        .await
        .unwrap();
    let film = movie(&e.state, lib, "Heat", None).await;
    let (_, signed) = member_in(&e, mia, "laptop-0001", "web").await;
    let tok = token(&signed);
    let report = |pos: f64| {
        let app = e.app.clone();
        let tok = tok.clone();
        async move {
            call(
                &app,
                Method::PUT,
                &format!("/items/{film}/progress"),
                Some(&tok),
                WAN,
                Some(json!({"position_s": pos})),
            )
            .await
            .0
        }
    };
    assert_eq!(report(0.0).await, StatusCode::NO_CONTENT);
    // Pretend the last report was 10 s ago.
    let back = |secs: u64| {
        e.state
            .now_playing
            .lock()
            .unwrap()
            .get_mut(&(mia, film))
            .unwrap()
            .seen -= Duration::from_secs(secs);
    };
    back(10);
    report(10.0).await;
    back(10);
    report(20.0).await;
    // A seek back adds nothing; a long gap starts a new viewing without counting the gap.
    back(10);
    report(5.0).await;
    back(300);
    report(500.0).await;

    let (s, stats) = call(&e.app, Method::GET, "/profile/stats", Some(&tok), WAN, None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(stats["totals"]["total_s"], 20);
    assert_eq!(stats["totals"]["titles"], 1);
    assert_eq!(stats["top_titles"][0]["title"], "Heat");

    let (s, now) = admin_call(&e, Method::GET, "/activity/now", None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(now.as_array().unwrap().len(), 1);
    assert_eq!(now[0]["user_name"], "Mia");
    assert_eq!(now[0]["device_kind"], "web");
    assert_eq!(now[0]["position_s"], 500.0);

    // Admin sees any profile's numbers; a profile does not see others or the live list.
    let (_, summary) = admin_call(&e, Method::GET, "/activity/users", None).await;
    assert!(
        summary
            .as_array()
            .unwrap()
            .iter()
            .any(|u| u["totals"]["total_s"] == 20)
    );
    assert_eq!(
        call(&e.app, Method::GET, "/activity/now", Some(&tok), WAN, None)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &e.app,
            Method::GET,
            &format!("/activity/users/{mia}"),
            Some(&tok),
            WAN,
            None
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );

    // Finishing removes the live entry.
    back(5);
    call(
        &e.app,
        Method::PUT,
        &format!("/items/{film}/progress"),
        Some(&tok),
        WAN,
        Some(json!({"position_s": 505.0, "watched": true})),
    )
    .await;
    let (_, now) = admin_call(&e, Method::GET, "/activity/now", None).await;
    assert!(now.as_array().unwrap().is_empty());
}

// ---- title requests ----

async fn request(e: &Env, tok: &str, body: Value) -> (StatusCode, Value) {
    call(
        &e.app,
        Method::POST,
        "/requests",
        Some(tok),
        LAN,
        Some(body),
    )
    .await
}

#[tokio::test]
async fn title_requests_rules() {
    let e = env().await;
    let mia = member(&e, "Mia").await;
    let bo = member(&e, "Bo").await;
    let (mt, bt) = (
        session(&e.state, mia, false).await,
        session(&e.state, bo, false).await,
    );

    let (s, r) = request(
        &e,
        &mt,
        json!({"kind": "movie", "title": "Heat", "tmdb_id": 949}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED);
    let rid = r["id"].as_i64().unwrap();
    assert_eq!(r["status"], "pending");

    let (s, _) = request(
        &e,
        &bt,
        json!({"kind": "movie", "title": "Heat again", "tmdb_id": 949}),
    )
    .await;
    assert_eq!(s, StatusCode::CONFLICT, "open duplicate");
    let (s, _) = request(&e, &bt, json!({"kind": "tv", "title": "x"})).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let (s, _) = request(&e, &bt, json!({"kind": "movie", "title": "  "})).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);

    // A title the library already has.
    let lib = libraries::create(&e.state.db, "Films", "movie", &["/media/films".into()])
        .await
        .unwrap();
    let have = movie(&e.state, lib, "Paddington", None).await;
    sqlx::query("UPDATE items SET tmdb_id = 116 WHERE id = ?1")
        .bind(have)
        .execute(&e.state.db)
        .await
        .unwrap();
    let (s, _) = request(
        &e,
        &bt,
        json!({"kind": "movie", "title": "Paddington", "tmdb_id": 116}),
    )
    .await;
    assert_eq!(s, StatusCode::CONFLICT, "in library");

    let (s, own) = request(&e, &bt, json!({"kind": "show", "title": "Bluey"})).await;
    assert_eq!(s, StatusCode::CREATED);
    let own_id = own["id"].as_i64().unwrap();

    // Each profile sees its own list, the admin everything.
    let list = |tok: String| {
        let app = e.app.clone();
        async move {
            call(&app, Method::GET, "/requests", Some(&tok), LAN, None)
                .await
                .1
        }
    };
    assert_eq!(list(mt.clone()).await.as_array().unwrap().len(), 1);
    assert_eq!(list(bt.clone()).await.as_array().unwrap().len(), 1);
    assert_eq!(list(e.admin.clone()).await.as_array().unwrap().len(), 2);

    // Only the admin answers.
    let answer = json!({"status": "approved", "admin_note": "on it"});
    let (s, _) = call(
        &e.app,
        Method::PATCH,
        &format!("/requests/{rid}"),
        Some(&mt),
        LAN,
        Some(answer.clone()),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN);
    let (s, _) = admin_call(
        &e,
        Method::PATCH,
        &format!("/requests/{rid}"),
        Some(json!({"status": "maybe"})),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let (s, done) = admin_call(&e, Method::PATCH, &format!("/requests/{rid}"), Some(answer)).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(done["status"], "approved");

    // The owner withdraws a pending request, not an approved one; nobody withdraws another's.
    let del = |tok: String, id: i64| {
        let app = e.app.clone();
        async move {
            call(
                &app,
                Method::DELETE,
                &format!("/requests/{id}"),
                Some(&tok),
                LAN,
                None,
            )
            .await
            .0
        }
    };
    assert_eq!(del(mt.clone(), rid).await, StatusCode::FORBIDDEN);
    assert_eq!(del(mt.clone(), own_id).await, StatusCode::FORBIDDEN);
    assert_eq!(del(bt.clone(), own_id).await, StatusCode::NO_CONTENT);
}

// ---- notifications ----

#[tokio::test]
async fn notification_secrets_are_masked_and_kept() {
    let e = env().await;
    let put = |v: Value| {
        let app = e.app.clone();
        let tok = e.admin.clone();
        async move {
            call(
                &app,
                Method::PUT,
                "/settings/notifications",
                Some(&tok),
                LAN,
                Some(v),
            )
            .await
        }
    };
    let (s, saved) = put(json!({"channels": [
        {"kind": "ntfy", "label": "Phone", "enabled": true, "url": "https://ntfy.example", "topic": "k", "token": "tk_secret"},
        {"kind": "smtp", "enabled": true, "smtp_host": "mail.example", "smtp_password": "pw_secret", "to": "a@example.com"},
        {"kind": "carrier-pigeon"},
    ]}))
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(
        saved["channels"].as_array().unwrap().len(),
        2,
        "unknown kind dropped"
    );
    assert_eq!(saved["channels"][0]["token"], MASK);
    assert_eq!(saved["channels"][1]["smtp_password"], MASK);

    let (_, read) = admin_call(&e, Method::GET, "/settings/notifications", None).await;
    let text = read.to_string();
    assert!(!text.contains("tk_secret") && !text.contains("pw_secret"));

    // Saving what the page read keeps the stored secrets.
    let mut edited = read.clone();
    edited["channels"][0]["label"] = json!("Renamed");
    let (s, _) = put(edited).await;
    assert_eq!(s, StatusCode::OK);
    let stored = notify::load(&e.state.db).await.unwrap();
    assert_eq!(stored.channels[0].label, "Renamed");
    assert_eq!(stored.channels[0].token, "tk_secret");
    assert_eq!(stored.channels[1].smtp_password, "pw_secret");

    // Test send on a channel that does not exist.
    let (s, _) = admin_call(&e, Method::POST, "/settings/notifications/test/9", None).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);

    let mia = member(&e, "Mia").await;
    let tok = session(&e.state, mia, false).await;
    assert_eq!(
        call(
            &e.app,
            Method::GET,
            "/settings/notifications",
            Some(&tok),
            LAN,
            None
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn site_owner_cannot_be_demoted_deleted_or_edited_by_others() {
    let e = env().await;
    // The test harness creates the admin directly; mark it owner like setup or the migration does.
    sqlx::query("UPDATE users SET is_owner = 1 WHERE id = ?1")
        .bind(e.admin_id)
        .execute(&e.state.db)
        .await
        .unwrap();
    let other = user(&e.state, "Other admin", true, None).await;
    let other_token = session(&e.state, other, false).await;
    let as_other = |method: Method, uri: String, body: Option<Value>| {
        let (app, token) = (e.app.clone(), other_token.clone());
        async move { call(&app, method, &uri, Some(&token), LAN, body).await }
    };
    let uri = format!("/users/{}", e.admin_id);
    let body = json!({"name": "Admin", "is_admin": false, "password": "newpassword1"});
    assert_eq!(
        as_other(Method::PATCH, uri.clone(), Some(body)).await.0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        as_other(Method::DELETE, uri.clone(), None).await.0,
        StatusCode::BAD_REQUEST
    );

    // The owner cannot step down either.
    let (s, v) = admin_call(
        &e,
        Method::PATCH,
        &uri,
        Some(json!({"name": "Admin", "is_admin": false})),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    assert_eq!(v["error"]["code"], "users.owner_stays_admin");

    let (_, list) = admin_call(&e, Method::GET, "/users", None).await;
    assert_eq!(list[0]["is_owner"], true, "owner listed first");
    assert_eq!(list[0]["is_admin"], true);
}
