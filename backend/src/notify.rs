//! Admin notifications (new device waiting for approval, new title request) sent to any
//! number of channels: ntfy, Gotify, Discord, Slack, Telegram, a generic JSON webhook,
//! or e-mail over SMTP. Stored as one JSON row in `settings`; secrets never leave the server.

use crate::i18n::t;
use crate::state::AppState;
use kinema_db::{DbError, SqlitePool, settings};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const KEY: &str = "notifications";
/// Stands in for a stored secret in what the admin page reads; sent back, it keeps the secret.
pub const MASK: &str = "••••••••";
pub const KINDS: &[&str] = &[
    "ntfy", "gotify", "discord", "slack", "telegram", "webhook", "smtp",
];

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Channel {
    pub kind: String,
    pub label: String,
    pub enabled: bool,
    /// ntfy / Gotify server, or the Discord, Slack or generic webhook address.
    pub url: String,
    /// ntfy topic.
    pub topic: String,
    /// ntfy access token, Gotify app token, Telegram bot token. Secret.
    pub token: String,
    /// Telegram chat id, or the e-mail recipient(s), comma separated.
    pub to: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    /// tls (implicit, 465) | starttls (587) | none
    pub smtp_security: String,
    pub smtp_username: String,
    /// Secret.
    pub smtp_password: String,
    pub from: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Notifications {
    pub channels: Vec<Channel>,
    pub on_device_request: bool,
    pub on_media_request: bool,
}

impl Default for Notifications {
    fn default() -> Self {
        Self {
            channels: Vec::new(),
            on_device_request: true,
            on_media_request: true,
        }
    }
}

fn secret_url(kind: &str) -> bool {
    matches!(kind, "discord" | "slack" | "webhook")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    DeviceRequest,
    MediaRequest,
}

pub async fn load(pool: &SqlitePool) -> Result<Notifications, DbError> {
    Ok(settings::get(pool, KEY)
        .await?
        .and_then(|v| serde_json::from_str(&v).ok())
        .unwrap_or_default())
}

pub async fn save(pool: &SqlitePool, n: &Notifications) -> Result<(), DbError> {
    settings::set(pool, KEY, &serde_json::to_string(n).unwrap_or_default()).await
}

impl Notifications {
    /// What the admin page sees: secrets replaced by `MASK`.
    pub fn masked(&self) -> Self {
        let hide = |s: &str| {
            if s.is_empty() {
                String::new()
            } else {
                MASK.to_string()
            }
        };
        let mut out = self.clone();
        for c in &mut out.channels {
            c.token = hide(&c.token);
            c.smtp_password = hide(&c.smtp_password);
            // A Discord, Slack or webhook address is itself the secret.
            if secret_url(&c.kind) {
                c.url = hide(&c.url);
            }
        }
        out
    }

    /// Takes an edit from the admin page: unknown kinds dropped, `MASK` swapped back for
    /// the stored secret of the channel at the same position and kind.
    pub fn merged(mut self, old: &Notifications) -> Self {
        self.channels.retain(|c| KINDS.contains(&c.kind.as_str()));
        self.channels.truncate(20);
        for (i, c) in self.channels.iter_mut().enumerate() {
            let prev = old.channels.get(i).filter(|p| p.kind == c.kind);
            if c.token == MASK {
                c.token = prev.map(|p| p.token.clone()).unwrap_or_default();
            }
            if c.url == MASK && secret_url(&c.kind) {
                c.url = prev.map(|p| p.url.clone()).unwrap_or_default();
            }
            if c.smtp_password == MASK {
                c.smtp_password = prev.map(|p| p.smtp_password.clone()).unwrap_or_default();
            }
            c.label = c.label.trim().chars().take(60).collect();
            if !["tls", "starttls", "none"].contains(&c.smtp_security.as_str()) {
                c.smtp_security = "starttls".into();
            }
        }
        self
    }

    fn wants(&self, event: Event) -> bool {
        match event {
            Event::DeviceRequest => self.on_device_request,
            Event::MediaRequest => self.on_media_request,
        }
    }
}

/// `{name}` style placeholders in a locale string.
fn fill(key: &str, args: &[(&str, &str)]) -> String {
    args.iter()
        .fold(t(key), |s, (k, v)| s.replace(&format!("{{{k}}}"), v))
}

pub struct Message {
    pub title: String,
    pub body: String,
}

pub fn device_request(user: &str, device: &str, kind: &str) -> Message {
    Message {
        title: t("notify.device.title"),
        body: fill(
            "notify.device.body",
            &[("user", user), ("device", device), ("kind", kind)],
        ),
    }
}

pub fn media_request(user: &str, title: &str, kind: &str) -> Message {
    Message {
        title: t("notify.request.title"),
        body: fill(
            "notify.request.body",
            &[("user", user), ("title", title), ("kind", kind)],
        ),
    }
}

/// Fire and forget: a slow or broken channel never holds up the request that caused it.
pub fn spawn(state: &AppState, event: Event, msg: Message) {
    let state = state.clone();
    tokio::spawn(async move {
        let Ok(n) = load(&state.db).await else { return };
        if !n.wants(event) {
            return;
        }
        let msg = with_link(&state, msg);
        for c in n.channels.iter().filter(|c| c.enabled) {
            if let Err(e) = send(&state.http, c, &msg).await {
                tracing::warn!("notification via {} ({}): {e}", c.kind, c.label);
            }
        }
    });
}

/// Adds the admin page address when a public URL is set.
fn with_link(state: &AppState, mut msg: Message) -> Message {
    let base = state.settings().public_url;
    if !base.is_empty() {
        msg.body = format!("{}\n{}/settings", msg.body, base.trim_end_matches('/'));
    }
    msg
}

/// Sends one message on one channel; also used by the admin page's "Send test".
pub async fn send(http: &reqwest::Client, c: &Channel, msg: &Message) -> Result<(), String> {
    let timeout = Duration::from_secs(15);
    let url = c.url.trim().trim_end_matches('/');
    let req = match c.kind.as_str() {
        "ntfy" => {
            let base = if url.is_empty() {
                "https://ntfy.sh"
            } else {
                url
            };
            if c.topic.trim().is_empty() {
                return Err("no topic".into());
            }
            let mut r = http
                .post(format!("{base}/{}", c.topic.trim()))
                // ntfy reads the title from a header, which must be ASCII-safe.
                .header("Title", msg.title.replace(|ch: char| !ch.is_ascii(), ""))
                .body(msg.body.clone());
            if !c.token.is_empty() {
                r = r.bearer_auth(&c.token);
            }
            r
        }
        "gotify" => http
            .post(format!("{url}/message"))
            .header("X-Gotify-Key", &c.token)
            .json(&serde_json::json!({ "title": msg.title, "message": msg.body, "priority": 5 })),
        "discord" => http
            .post(url)
            .json(&serde_json::json!({ "content": format!("**{}**\n{}", msg.title, msg.body) })),
        "slack" => http
            .post(url)
            .json(&serde_json::json!({ "text": format!("*{}*\n{}", msg.title, msg.body) })),
        "telegram" => http
            .post(format!(
                "https://api.telegram.org/bot{}/sendMessage",
                c.token.trim()
            ))
            .json(&serde_json::json!({
                "chat_id": c.to.trim(),
                "text": format!("{}\n{}", msg.title, msg.body),
            })),
        "webhook" => http.post(url).json(&serde_json::json!({
            "source": "kinema",
            "title": msg.title,
            "message": msg.body,
        })),
        "smtp" => return send_mail(c, msg).await,
        other => return Err(format!("unknown channel {other}")),
    };
    let res = req
        .timeout(timeout)
        .send()
        .await
        // The error text carries the URL, which holds the Telegram bot token or a webhook secret.
        .map_err(|e| e.without_url().to_string())?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(format!("HTTP {}", res.status()))
    }
}

async fn send_mail(c: &Channel, msg: &Message) -> Result<(), String> {
    use lettre::message::Mailbox;
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{AsyncSmtpTransport, AsyncTransport, Message as Mail, Tokio1Executor};

    let host = c.smtp_host.trim();
    if host.is_empty() {
        return Err("no SMTP host".into());
    }
    let from: Mailbox = c.from.trim().parse().map_err(|e| format!("from: {e}"))?;
    let mut builder = Mail::builder().from(from).subject(&msg.title);
    for to in c.to.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        builder = builder.to(to.parse().map_err(|e| format!("to: {e}"))?);
    }
    let mail = builder.body(msg.body.clone()).map_err(|e| e.to_string())?;
    let mut transport = match c.smtp_security.as_str() {
        "tls" => AsyncSmtpTransport::<Tokio1Executor>::relay(host),
        "none" => Ok(AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(
            host,
        )),
        _ => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(host),
    }
    .map_err(|e| e.to_string())?
    .timeout(Some(Duration::from_secs(20)));
    if c.smtp_port != 0 {
        transport = transport.port(c.smtp_port);
    }
    if !c.smtp_username.is_empty() {
        transport = transport.credentials(Credentials::new(
            c.smtp_username.clone(),
            c.smtp_password.clone(),
        ));
    }
    transport
        .build()
        .send(mail)
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secrets_are_masked_and_kept() {
        let stored = Notifications {
            channels: vec![Channel {
                kind: "ntfy".into(),
                token: "secret".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let seen = stored.masked();
        assert_eq!(seen.channels[0].token, MASK);
        let back = seen.merged(&stored);
        assert_eq!(back.channels[0].token, "secret");
        let hook = Notifications {
            channels: vec![Channel {
                kind: "discord".into(),
                url: "https://discord.com/api/webhooks/1/abc".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert_eq!(hook.masked().channels[0].url, MASK);
        assert_eq!(
            hook.masked().merged(&hook).channels[0].url,
            hook.channels[0].url
        );
        // A channel whose kind changed does not inherit the old secret.
        let mut changed = stored.masked();
        changed.channels[0].kind = "gotify".into();
        assert_eq!(changed.merged(&stored).channels[0].token, "");
    }

    #[test]
    fn unknown_kinds_are_dropped() {
        let n = Notifications {
            channels: vec![Channel {
                kind: "carrier-pigeon".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(n.merged(&Notifications::default()).channels.is_empty());
    }

    #[test]
    fn placeholders_fill() {
        let m = media_request("Ana", "Dune", "movie");
        assert!(m.body.contains("Ana") && m.body.contains("Dune"));
    }
}
