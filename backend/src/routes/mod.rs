pub mod activity;
pub mod auth;
pub mod collections;
pub mod devices;
pub mod items;
pub mod libraries;
pub mod matching;
pub mod media;
pub mod metadata_edit;
pub mod playlists;
pub mod profile;
pub mod requests;
pub mod scrub;
pub mod server;
pub mod settings;
pub mod shares;
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
        .merge(metadata_edit::router())
        .merge(playlists::router())
        .merge(collections::router())
        .merge(media::router())
        .merge(scrub::router())
        .merge(settings::router())
        .merge(users::router())
        .merge(profile::router())
        .merge(shares::router())
        .merge(devices::router())
        .merge(activity::router())
        .merge(requests::router())
}
