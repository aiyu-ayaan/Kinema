pub mod auth;
pub mod items;
pub mod libraries;
pub mod matching;
pub mod media;
pub mod server;
pub mod settings;
pub mod users;

use crate::state::AppState;
use axum::Router;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .nest("/server", server::router())
        .nest("/auth", auth::router())
        .merge(libraries::router())
        .merge(items::router())
        .merge(matching::router())
        .merge(media::router())
        .merge(settings::router())
        .merge(users::router())
}
