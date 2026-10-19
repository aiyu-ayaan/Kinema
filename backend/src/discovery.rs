//! LAN discovery beacon. Apps broadcast `KINEMA_DISCOVER` to UDP [`PORT`]; every server on
//! the network answers the sender with a small JSON reply naming itself, its port and scheme.
//! A blocked `recv` costs no CPU while nobody asks, so an idle server stays idle.

use crate::state::AppState;
use serde::Serialize;
use tokio::net::UdpSocket;
use tracing::{info, warn};

pub const PORT: u16 = 48555;
const PROBE: &[u8] = b"KINEMA_DISCOVER";

#[derive(Serialize)]
struct Reply<'a> {
    service: &'static str,
    name: &'a str,
    port: u16,
    tls: bool,
    version: &'static str,
}

fn reply(state: &AppState) -> Vec<u8> {
    let name = state.settings().server_name;
    serde_json::to_vec(&Reply {
        service: "kinema",
        name: &name,
        port: state.config.server.port,
        tls: state.config.tls.enabled,
        version: env!("CARGO_PKG_VERSION"),
    })
    .unwrap_or_default()
}

/// Answers discovery probes until the process ends. A server bound to loopback is not
/// reachable from other devices, so it stays silent.
pub async fn run(state: AppState) {
    if matches!(
        state.config.server.host.as_str(),
        "127.0.0.1" | "localhost" | "::1"
    ) {
        return;
    }
    let socket = match UdpSocket::bind(("0.0.0.0", PORT)).await {
        Ok(s) => s,
        Err(e) => {
            warn!("LAN discovery off: cannot bind UDP {PORT}: {e}");
            return;
        }
    };
    info!("LAN discovery listening on UDP {PORT}");
    let mut buf = [0u8; 64];
    loop {
        let Ok((len, from)) = socket.recv_from(&mut buf).await else {
            continue;
        };
        if &buf[..len] == PROBE {
            let _ = socket.send_to(&reply(&state), from).await;
        }
    }
}
