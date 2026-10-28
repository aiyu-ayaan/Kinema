//! Locked fields, playlists, collections and shelves against a real (temporary) SQLite file.

use crate::items::{self, Edit, Meta};
use crate::{SqlitePool, collections, init_pool, libraries, playlists, run_migrations, shelves};

struct TestDb {
    pool: SqlitePool,
    path: std::path::PathBuf,
}

impl Drop for TestDb {
    fn drop(&mut self) {
        for ext in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{ext}", self.path.display()));
        }
    }
}

async fn db() -> TestDb {
    let path = std::env::temp_dir().join(format!("kinema-test-{}.db", uuid::Uuid::new_v4()));
    let pool = init_pool(&format!("sqlite://{}", path.display()))
        .await
        .expect("pool");
    run_migrations(&pool).await.expect("migrations");
    TestDb { pool, path }
}

async fn user(pool: &SqlitePool, name: &str) -> i64 {
    sqlx::query("INSERT INTO users (name, is_admin, created_at) VALUES (?1, 0, 0)")
        .bind(name)
        .execute(pool)
        .await
        .expect("user")
        .last_insert_rowid()
}

async fn library(pool: &SqlitePool) -> i64 {
    libraries::create(pool, "Movies", "movie", &["/media/movies".to_string()])
        .await
        .expect("library")
}

async fn movie(pool: &SqlitePool, lib: i64, title: &str) -> i64 {
    items::find_or_create(
        pool,
        lib,
        items::NewItem::Movie {
            title,
            year: Some(2000),
        },
    )
    .await
    .expect("movie")
}

async fn row(pool: &SqlitePool, id: i64) -> (String, Option<String>, Option<i64>, Option<String>) {
    sqlx::query_as("SELECT title, summary, year, poster FROM items WHERE id = ?1")
        .bind(id)
        .fetch_one(pool)
        .await
        .expect("row")
}

fn tmdb_meta() -> Meta {
    Meta {
        title: Some("TMDB Title".into()),
        summary: Some("TMDB summary".into()),
        year: Some(1999),
        poster: Some("w500/new.jpg".into()),
        genres: vec!["Drama".into()],
        ..Default::default()
    }
}

#[tokio::test]
async fn locked_fields_survive_a_tmdb_refresh() {
    let t = db().await;
    let lib = library(&t.pool).await;
    let id = movie(&t.pool, lib, "Raw Name").await;
    items::edit_meta(
        &t.pool,
        id,
        &Edit {
            title: Some("My Title".into()),
            summary: Some("My story".into()),
            ..Default::default()
        },
    )
    .await
    .expect("edit");
    assert_eq!(
        items::locked_fields(&t.pool, id).await.expect("locks"),
        ["title", "summary"]
    );

    items::save_meta(&t.pool, id, &tmdb_meta())
        .await
        .expect("save");
    let (title, summary, year, poster) = row(&t.pool, id).await;
    assert_eq!(title, "My Title");
    assert_eq!(summary.as_deref(), Some("My story"));
    // Unlocked fields still update.
    assert_eq!(year, Some(1999));
    assert_eq!(poster.as_deref(), Some("w500/new.jpg"));
}

#[tokio::test]
async fn fix_match_keeps_locked_fields_and_clears_the_rest() {
    let t = db().await;
    let lib = library(&t.pool).await;
    let id = movie(&t.pool, lib, "Raw Name").await;
    items::save_meta(&t.pool, id, &tmdb_meta())
        .await
        .expect("save");
    items::set_locked_fields(&t.pool, id, &["poster", "genres", "bogus"])
        .await
        .expect("lock");
    assert_eq!(
        items::locked_fields(&t.pool, id).await.expect("locks"),
        ["poster", "genres"]
    );

    items::clear_meta(&t.pool, id).await.expect("clear");
    let (_, summary, year, poster) = row(&t.pool, id).await;
    assert_eq!(poster.as_deref(), Some("w500/new.jpg"));
    assert_eq!(summary, None);
    assert_eq!(year, None);
    assert_eq!(items::genres(&t.pool, id).await.expect("genres"), ["Drama"]);

    // Unlock: the next refresh may overwrite it again.
    items::set_locked_fields::<&str>(&t.pool, id, &[])
        .await
        .expect("unlock");
    items::save_meta(
        &t.pool,
        id,
        &Meta {
            poster: Some("w500/other.jpg".into()),
            ..Default::default()
        },
    )
    .await
    .expect("save");
    assert_eq!(row(&t.pool, id).await.3.as_deref(), Some("w500/other.jpg"));
}

#[tokio::test]
async fn root_options_are_clamped_and_unknown_paths_ignored() {
    let t = db().await;
    let lib = library(&t.pool).await;
    libraries::set_root_options(
        &t.pool,
        lib,
        &[
            libraries::RootOption {
                path: "/media/movies".into(),
                poll: true,
                poll_interval_min: 0,
            },
            libraries::RootOption {
                path: "/elsewhere".into(),
                poll: true,
                poll_interval_min: 5,
            },
        ],
    )
    .await
    .expect("options");
    let l = libraries::get(&t.pool, lib)
        .await
        .expect("get")
        .expect("lib");
    assert_eq!(l.root_options.len(), 1);
    assert!(l.root_options[0].poll);
    assert_eq!(l.root_options[0].poll_interval_min, libraries::POLL_MIN);
}

#[tokio::test]
async fn playlist_order_append_and_permissions() {
    let t = db().await;
    let (alice, bob) = (user(&t.pool, "alice").await, user(&t.pool, "bob").await);
    let lib = library(&t.pool).await;
    let (a, b, c) = (
        movie(&t.pool, lib, "A").await,
        movie(&t.pool, lib, "B").await,
        movie(&t.pool, lib, "C").await,
    );
    let pl = playlists::create(&t.pool, alice, "Friday")
        .await
        .expect("create");
    playlists::append(&t.pool, pl, &[a, b])
        .await
        .expect("append");
    playlists::append(&t.pool, pl, &[b, c])
        .await
        .expect("append again");
    assert_eq!(
        playlists::item_ids(&t.pool, pl).await.expect("ids"),
        [a, b, c]
    );

    playlists::set_items(&t.pool, pl, &[c, a])
        .await
        .expect("reorder");
    let cards = playlists::items(&t.pool, alice, pl).await.expect("items");
    assert_eq!(cards.iter().map(|i| i.id).collect::<Vec<_>>(), [c, a]);

    // Private until shared; never editable by anyone but the owner.
    let p = playlists::get(&t.pool, pl)
        .await
        .expect("get")
        .expect("some");
    assert!(p.editable_by(alice) && !p.editable_by(bob));
    assert!(p.visible_to(alice) && !p.visible_to(bob));
    assert!(
        playlists::list_visible(&t.pool, bob)
            .await
            .expect("list")
            .is_empty()
    );
    playlists::update(&t.pool, pl, None, Some(true))
        .await
        .expect("share");
    let seen = playlists::list_visible(&t.pool, bob).await.expect("list");
    assert_eq!(seen.len(), 1);
    let p = playlists::get(&t.pool, pl)
        .await
        .expect("get")
        .expect("some");
    assert!(p.visible_to(bob) && !p.editable_by(bob));

    playlists::delete(&t.pool, pl).await.expect("delete");
    assert!(playlists::get(&t.pool, pl).await.expect("get").is_none());
}

#[tokio::test]
async fn collections_and_shelves() {
    let t = db().await;
    let alice = user(&t.pool, "alice").await;
    let lib = library(&t.pool).await;
    let (a, b) = (
        movie(&t.pool, lib, "Part 1").await,
        movie(&t.pool, lib, "Part 2").await,
    );
    collections::assign_tmdb(&t.pool, a, 10, "Saga")
        .await
        .expect("assign");
    collections::assign_tmdb(&t.pool, b, 10, "Saga renamed upstream")
        .await
        .expect("assign");
    let all = collections::list(&t.pool).await.expect("list");
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].name, "Saga");
    assert_eq!(
        collections::items(&t.pool, alice, all[0].id)
            .await
            .expect("items")
            .len(),
        2
    );

    items::save_meta(
        &t.pool,
        a,
        &Meta {
            genres: vec!["Horror".into()],
            ..Default::default()
        },
    )
    .await
    .expect("genre");
    // The seeded shelf finds it; a profile can hide it; a library scope can exclude it.
    let shelf = shelves::list(&t.pool).await.expect("shelves").remove(0);
    assert_eq!(shelf.genre, "Horror");
    let hits = shelves::genre_items(&t.pool, alice, &shelf.genre, None, 24)
        .await
        .expect("hits");
    assert_eq!(hits.iter().map(|i| i.id).collect::<Vec<_>>(), [a]);
    let none = shelves::genre_items(&t.pool, alice, &shelf.genre, Some(&[lib + 1]), 24)
        .await
        .expect("scoped");
    assert!(none.is_empty());
    shelves::set_hidden(&t.pool, alice, shelf.id, true)
        .await
        .expect("hide");
    assert!(
        shelves::shown_to(&t.pool, alice)
            .await
            .expect("shown")
            .is_empty()
    );
}

#[tokio::test]
async fn library_pages_cover_every_title_once() {
    let db = db().await;
    let u = user(&db.pool, "viewer").await;
    let lib = library(&db.pool).await;
    for n in 0..25 {
        movie(&db.pool, lib, &format!("Film {n:02}")).await;
    }
    let (any, title) = (items::KindFilter::Any, items::LibrarySort::Title);
    assert_eq!(
        items::library_count(&db.pool, lib, any)
            .await
            .expect("count"),
        25
    );
    let mut seen = Vec::new();
    for offset in (0..25).step_by(10) {
        let page = items::library_page(&db.pool, u, lib, any, title, offset, 10)
            .await
            .expect("page");
        assert!(page.len() <= 10);
        seen.extend(page.into_iter().map(|c| c.title));
    }
    let mut sorted = seen.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(seen.len(), 25);
    assert_eq!(seen, sorted, "pages come back in title order, no repeats");
    let shows = items::library_count(&db.pool, lib, items::KindFilter::Shows)
        .await
        .expect("shows");
    assert_eq!(shows, 0);
}

#[tokio::test]
async fn web_and_desktop_share_a_device_slot_mobile_has_its_own() {
    use crate::devices::{self, NewDevice};
    let db = db().await;
    let u = user(&db.pool, "Mia").await;
    let add = |client_id: &'static str, kind: &'static str, approved: bool| {
        let pool = db.pool.clone();
        async move {
            devices::upsert(
                &pool,
                &NewDevice {
                    user_id: u,
                    client_id,
                    kind,
                    name: kind,
                    approved,
                    poll_hash: None,
                },
            )
            .await
            .expect("device")
        }
    };
    assert_eq!(
        devices::approved_in_slot(&db.pool, u, "web").await.unwrap(),
        0
    );
    add("web-client-1", "web", true).await;
    assert_eq!(
        devices::approved_in_slot(&db.pool, u, "web").await.unwrap(),
        1
    );
    assert_eq!(
        devices::approved_in_slot(&db.pool, u, "desktop")
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        devices::approved_in_slot(&db.pool, u, "mobile")
            .await
            .unwrap(),
        0
    );
    // A pending device uses no slot; an approved mobile one fills only the mobile slot.
    add("desk-client-1", "desktop", false).await;
    assert_eq!(
        devices::approved_in_slot(&db.pool, u, "web").await.unwrap(),
        1
    );
    add("phone-client-1", "mobile", true).await;
    assert_eq!(
        devices::approved_in_slot(&db.pool, u, "mobile")
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        devices::approved_in_slot(&db.pool, u, "desktop")
            .await
            .unwrap(),
        1
    );
    // The same client stays approved when it signs in again as pending.
    let id = add("web-client-1", "web", false).await;
    assert_eq!(
        devices::get(&db.pool, id).await.unwrap().unwrap().status,
        "approved"
    );
}

#[tokio::test]
async fn top_titles_roll_episodes_up_to_their_show() {
    use crate::{activity, items::NewItem};
    let db = db().await;
    let u = user(&db.pool, "Mia").await;
    let lib = library(&db.pool).await;
    let show = items::find_or_create(
        &db.pool,
        lib,
        NewItem::Show {
            title: "Bluey",
            year: None,
        },
    )
    .await
    .unwrap();
    let s1 = items::find_or_create(
        &db.pool,
        lib,
        NewItem::Season {
            show_id: show,
            season: 1,
        },
    )
    .await
    .unwrap();
    let mut eps = Vec::new();
    for n in 1..=2 {
        eps.push(
            items::find_or_create(
                &db.pool,
                lib,
                NewItem::Episode {
                    season_id: s1,
                    episode: n,
                    title: "Ep",
                },
            )
            .await
            .unwrap(),
        );
    }
    let film = movie(&db.pool, lib, "Heat").await;
    activity::add_watch_time(&db.pool, u, eps[0], 600)
        .await
        .unwrap();
    activity::add_watch_time(&db.pool, u, eps[1], 500)
        .await
        .unwrap();
    activity::add_watch_time(&db.pool, u, film, 900)
        .await
        .unwrap();
    activity::add_watch_time(&db.pool, u, film, 0)
        .await
        .unwrap();
    let top = activity::top_titles(&db.pool, u, 5).await.unwrap();
    assert_eq!(top.len(), 2, "two episodes are one show");
    assert_eq!((top[0].item_id, top[0].seconds), (show, 1100));
    assert_eq!((top[1].item_id, top[1].seconds), (film, 900));
    let totals = activity::totals(&db.pool, u).await.unwrap();
    assert_eq!((totals.total_s, totals.titles), (2000, 3));
}

#[tokio::test]
async fn open_duplicate_ignores_declined_and_other_kinds() {
    use crate::requests::{self, NewRequest};
    let db = db().await;
    let u = user(&db.pool, "Mia").await;
    let id = requests::create(
        &db.pool,
        &NewRequest {
            user_id: u,
            kind: "movie",
            title: "Heat",
            year: None,
            tmdb_id: Some(949),
            poster: None,
            overview: None,
            note: None,
        },
    )
    .await
    .unwrap();
    assert!(
        requests::open_duplicate(&db.pool, "movie", 949)
            .await
            .unwrap()
    );
    assert!(
        !requests::open_duplicate(&db.pool, "show", 949)
            .await
            .unwrap()
    );
    assert!(
        !requests::open_duplicate(&db.pool, "movie", 1)
            .await
            .unwrap()
    );
    requests::set_status(&db.pool, id, "approved", None)
        .await
        .unwrap();
    assert!(
        requests::open_duplicate(&db.pool, "movie", 949)
            .await
            .unwrap()
    );
    requests::set_status(&db.pool, id, "declined", None)
        .await
        .unwrap();
    assert!(
        !requests::open_duplicate(&db.pool, "movie", 949)
            .await
            .unwrap()
    );
}
