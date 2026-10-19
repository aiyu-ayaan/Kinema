use crate::state::AppState;
use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;

#[derive(Serialize)]
pub struct ServerInfoResponse {
    pub name: String,
    pub version: String,
    pub capabilities: Vec<String>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/info", get(server_info))
}

async fn server_info(State(_state): State<AppState>) -> Json<ServerInfoResponse> {
    Json(ServerInfoResponse {
        name: "Kinema Media Server".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        capabilities: vec![
            "direct_play".to_string(),
            "hls_remux".to_string(),
            "metadata_tmdb".to_string(),
        ],
    })
}
