//! The web UI's page. Its HTML carries a placeholder for the server name, filled in on every
//! request, so the browser tab, bookmarks, home-screen shortcuts and link previews all show
//! the name the admin chose, before any script runs and on every device.

use crate::state::AppState;
use axum::http::{StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use std::path::Path;

/// Kept in sync with `apps/web/index.html` and the dev plugin in `apps/web/vite.config.ts`.
pub const NAME_PLACEHOLDER: &str = "__KINEMA_SERVER_NAME__";

pub async fn index(state: AppState, path: &Path) -> Response {
    match tokio::fs::read_to_string(path).await {
        Ok(html) => (
            // The name can change at any time; never let a cache keep the old one.
            [(header::CACHE_CONTROL, "no-cache")],
            Html(fill(&html, &state.settings().server_name)),
        )
            .into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

fn fill(html: &str, name: &str) -> String {
    html.replace(NAME_PLACEHOLDER, &escape(name))
}

/// The name lands in text and in attribute values, so both kinds of quote are escaped too.
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_every_placeholder_escaped() {
        let html = r#"<title>__KINEMA_SERVER_NAME__</title><meta content="__KINEMA_SERVER_NAME__">"#;
        assert_eq!(
            fill(html, r#"Tom's "<Media>" & more"#),
            "<title>Tom&#39;s &quot;&lt;Media&gt;&quot; &amp; more</title>\
             <meta content=\"Tom&#39;s &quot;&lt;Media&gt;&quot; &amp; more\">"
        );
        assert_eq!(fill(html, "Home"), r#"<title>Home</title><meta content="Home">"#);
    }
}
