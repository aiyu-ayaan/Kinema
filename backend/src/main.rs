use anyhow::Context;
use axum::Router;
use axum::routing::get;
use kinema_core::Config;
use kinema_db::{init_pool, run_migrations};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::Semaphore;
use tower_http::services::ServeDir;
use tracing::{info, warn};

mod access;
#[cfg(test)]
mod access_tests;
#[cfg(test)]
mod accounts_tests;
mod auth;
mod boot;
mod discovery;
mod error;
mod hls;
mod i18n;
mod jobs;
mod keyframes;
mod markers;
mod net;
mod notify;
mod routes;
mod scan;
mod server_settings;
mod sprites;
mod state;
mod subtitles;
mod thumbs;
mod tls;
mod web;

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "kinema=info,backend=info,tower_http=warn".into()),
        )
        .init();

    info!("Starting Kinema Media Server...");
    let _ = rustls::crypto::ring::default_provider().install_default();

    // Load configuration
    let mut config = Config::load().context("Failed to load configuration")?;
    // Bound first, so discovery and the dashboard report the port actually in use.
    let listener = bind(&config.server.host, config.server.port)?;
    config.server.port = listener.local_addr()?.port();
    let config = Arc::new(config);
    for proxy in &config.network.trusted_proxies {
        if net::Cidr::parse(proxy).is_none() {
            warn!("Ignoring network.trusted_proxies entry {proxy:?}: not an IP address or CIDR");
        }
    }

    // Ensure data directory exists
    std::fs::create_dir_all(&config.data_dir).context("Failed to create data directory")?;

    // Initialize database pool
    let db_path = config.data_dir.join("kinema.db");
    let db_url = format!("sqlite://{}", db_path.to_string_lossy());
    let pool = init_pool(&db_url)
        .await
        .context("Failed to initialize SQLite pool")?;

    // Run migrations
    run_migrations(&pool)
        .await
        .context("Failed to run database migrations")?;

    let boot::Report { bins, ocr } = boot::run();

    let rustls = if config.tls.enabled {
        Some(tls::load(&config).await?)
    } else {
        None
    };

    let (changes, changes_rx) = tokio::sync::mpsc::unbounded_channel();
    let state = AppState {
        config: config.clone(),
        db: pool.clone(),
        bins,
        settings: Arc::new(RwLock::new(server_settings::load(&pool).await?)),
        scans: jobs::Jobs::new("scan"),
        thumb_jobs: jobs::Jobs::new("thumbnails"),
        marker_jobs: jobs::Jobs::new("skip markers"),
        subtitle_jobs: jobs::Jobs::new("subtitles"),
        ocr,
        keyframe_jobs: jobs::Jobs::new("keyframe index"),
        hls: hls::HlsSessions::default(),
        watchers: Arc::new(Mutex::new(HashMap::new())),
        changes,
        probe_slots: Arc::new(Semaphore::new(2)),
        thumb_slots: Arc::new(Semaphore::new(1)),
        active_streams: Arc::default(),
        active_transcodes: Arc::default(),
        started: std::time::Instant::now(),
        login_failures: Arc::default(),
        now_playing: Arc::default(),
        http: reqwest::Client::new(),
        tls_fingerprint: rustls
            .as_ref()
            .and_then(|(_, fp)| fp.as_deref().map(Arc::from)),
    };

    hls::purge(&state);
    // Watch every library and catch up on changes made while the server was off.
    tokio::spawn(scan::debounce_changes(state.clone(), changes_rx));
    tokio::spawn(scan::scheduled_rescans(state.clone()));
    tokio::spawn(discovery::run(state.clone()));
    // Polled folders: the first snapshot is the baseline, taken as the startup scans below begin.
    tokio::spawn(scan::poll_roots(state.clone()));
    for lib in kinema_db::libraries::list(&state.db).await? {
        scan::watch_lib(&state, &lib);
        scan::spawn_scan(state.clone(), lib.id);
    }

    let page_state = state.clone();

    // Build router
    let mut app = Router::new()
        .nest("/api", routes::api_router())
        .with_state(state);

    // Serve the web UI; unknown paths get index.html so client routes survive a reload.
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        std::env::var("KINEMA_WEB_DIST").ok().map(PathBuf::from),
        Some(PathBuf::from("apps/web/dist")),
        Some(
            manifest_dir
                .parent()
                .unwrap_or(manifest_dir)
                .join("apps/web/dist"),
        ),
        Some(PathBuf::from("../../apps/web/dist")),
        Some(PathBuf::from("../apps/web/dist")),
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("web"))),
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("../../apps/web/dist"))),
    ];
    let web_dist = candidates
        .into_iter()
        .flatten()
        .find(|p| p.join("index.html").exists());

    if let Some(dist) = web_dist {
        info!("Serving static files from {}", dist.display());
        let index = dist.join("index.html");
        let page = get(move || {
            let (state, index) = (page_state.clone(), index.clone());
            async move { web::index(state, &index).await }
        });
        app = app
            .route("/", page.clone())
            .route("/index.html", page.clone())
            .fallback_service(
                tower::ServiceBuilder::new()
                    .layer(axum::middleware::from_fn(web::cache_assets))
                    // Static files only: media streams are never compressed.
                    .layer(tower_http::compression::CompressionLayer::new())
                    .service(ServeDir::new(&dist).fallback(page)),
            );
    } else {
        warn!("No web UI build found (apps/web/dist). Web routes will return 404.");
    }

    let addr = listener.local_addr()?;
    // The peer address decides LAN-only profiles (net.rs).
    let service = app.into_make_service_with_connect_info::<std::net::SocketAddr>();
    if let Some((rustls, _)) = rustls {
        info!("Kinema server listening on https://{}", addr);
        axum_server::from_tcp_rustls(listener, rustls)
            .serve(service)
            .await?;
    } else {
        warn!("TLS is off: traffic is unencrypted. Only do this behind a TLS-terminating proxy.");
        info!("Kinema server listening on http://{}", addr);
        axum::serve(tokio::net::TcpListener::from_std(listener)?, service).await?;
    }

    Ok(())
}

/// How many ports after the configured one are tried when it is taken.
const PORT_TRIES: u16 = 20;

/// The configured port, or the next free one: another program on the port must not keep the
/// server down. Apps find the real port through LAN discovery; the log and dashboard show it.
fn bind(host: &str, port: u16) -> anyhow::Result<std::net::TcpListener> {
    for p in port..=port.saturating_add(PORT_TRIES) {
        match std::net::TcpListener::bind((host, p)) {
            Ok(listener) => {
                if p != port {
                    warn!(
                        "Port {port} is in use; using {p} instead. Set server.port in kinema.toml to choose another."
                    );
                }
                listener.set_nonblocking(true)?;
                return Ok(listener);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => continue,
            Err(e) => return Err(e).with_context(|| format!("Cannot listen on {host}:{p}")),
        }
    }
    anyhow::bail!(
        "Ports {port} to {} are all in use",
        port.saturating_add(PORT_TRIES)
    )
}

#[cfg(test)]
mod tests {
    use super::bind;

    #[test]
    fn taken_port_falls_back_to_the_next_free_one() {
        let taken = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = taken.local_addr().unwrap().port();
        let got = bind("127.0.0.1", port)
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        assert_ne!(got, port);
        assert!(got > port && got <= port + super::PORT_TRIES);
    }
}
