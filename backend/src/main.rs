use anyhow::Context;
use axum::Router;
use kinema_core::Config;
use kinema_db::{init_pool, run_migrations};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing::info;

mod routes;
mod state;

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "kinema=debug,tower_http=debug,axum=trace".into()),
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

    let state = AppState {
        config: config.clone(),
        db: pool,
    };

    // Build router
    let mut app = Router::new()
        .nest("/api", routes::api_router())
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Serve static files if apps/web/dist exists
    let web_dist = Path::new("apps/web/dist");
    if web_dist.exists() {
        info!("Serving static files from {}", web_dist.display());
        app = app.fallback_service(ServeDir::new(web_dist));
    }

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .context("Invalid server host or port")?;

    info!("Kinema server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
