use anyhow::Context;
use axum::Router;
use kinema_core::Config;
use kinema_db::{init_pool, run_migrations};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;
use tower_http::services::{ServeDir, ServeFile};
use tracing::{info, warn};

mod auth;
mod error;
mod i18n;
mod routes;
mod scan;
mod state;

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

    // Load configuration
    let config = Config::load().context("Failed to load configuration")?;
    let config = Arc::new(config);

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

    let bins = match kinema_media::MediaBinaries::resolve() {
        Ok(b) => {
            info!("ffmpeg: {}", b.ffmpeg.display());
            Some(Arc::new(b))
        }
        Err(e) => {
            warn!("{e} Files will be listed and direct-played only.");
            None
        }
    };

    let (changes, changes_rx) = tokio::sync::mpsc::unbounded_channel();
    let state = AppState {
        config: config.clone(),
        db: pool,
        bins,
        scans: Arc::new(Mutex::new(HashMap::new())),
        watchers: Arc::new(Mutex::new(HashMap::new())),
        changes,
        probe_slots: Arc::new(Semaphore::new(2)),
        http: reqwest::Client::new(),
    };

    // Watch every library and catch up on changes made while the server was off.
    tokio::spawn(scan::debounce_changes(state.clone(), changes_rx));
    for lib in kinema_db::libraries::list(&state.db).await? {
        scan::watch(&state, lib.id, &lib.roots);
        scan::spawn_scan(state.clone(), lib.id);
    }

    // Build router
    let mut app = Router::new()
        .nest("/api", routes::api_router())
        .with_state(state);

    // Serve the web UI; unknown paths get index.html so client routes survive a reload.
    let web_dist = Path::new("apps/web/dist");
    if web_dist.exists() {
        info!("Serving static files from {}", web_dist.display());
        app = app.fallback_service(
            ServeDir::new(web_dist).fallback(ServeFile::new(web_dist.join("index.html"))),
        );
    }

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .context("Invalid server host or port")?;

    info!("Kinema server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
