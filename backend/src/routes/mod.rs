pub mod server;

use crate::state::AppState;
use axum::Router;

pub fn api_router() -> Router<AppState> {
    Router::new().nest("/server", server::router())
}
