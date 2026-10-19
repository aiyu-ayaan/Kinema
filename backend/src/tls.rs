//! HTTPS for the server. Uses the configured certificate, or makes a self-signed one
//! covering localhost, the hostname and the LAN address. Apps pin its fingerprint the
//! first time they connect, so no authority is needed on a home network.

use anyhow::Context;
use axum_server::tls_rustls::RustlsConfig;
use kinema_core::Config;
use kinema_core::platform;
use std::path::PathBuf;
use tracing::info;

/// The TLS setup and the SHA-256 fingerprint of its certificate.
pub async fn load(config: &Config) -> anyhow::Result<(RustlsConfig, Option<String>)> {
    let (cert, key) = match (&config.tls.cert_path, &config.tls.key_path) {
        (Some(c), Some(k)) => (c.clone(), k.clone()),
        _ => ensure_self_signed(config)?,
    };
    info!("TLS certificate: {}", cert.display());
    // Apps show this when they first meet a self-signed server; the owner compares the two.
    let fp = std::fs::read_to_string(&cert)
        .ok()
        .and_then(|pem| fingerprint(&pem));
    if let Some(fp) = &fp {
        info!("TLS certificate SHA-256 fingerprint: {fp}");
    }
    let rustls = RustlsConfig::from_pem_file(&cert, &key)
        .await
        .with_context(|| format!("Cannot load TLS certificate {}", cert.display()))?;
    Ok((rustls, fp))
}

/// SHA-256 of the first certificate in `pem`, as `AA:BB:..`.
fn fingerprint(pem: &str) -> Option<String> {
    use base64::Engine;
    use sha2::Digest;
    let body: String = pem
        .lines()
        .skip_while(|l| !l.starts_with("-----BEGIN CERTIFICATE"))
        .skip(1)
        .take_while(|l| !l.starts_with("-----END"))
        .collect();
    let der = base64::engine::general_purpose::STANDARD
        .decode(body.trim())
        .ok()
        .filter(|d| !d.is_empty())?;
    let hex: Vec<String> = sha2::Sha256::digest(der)
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect();
    Some(hex.join(":"))
}

fn ensure_self_signed(config: &Config) -> anyhow::Result<(PathBuf, PathBuf)> {
    let dir = config.data_dir.join("tls");
    let cert_path = dir.join("cert.pem");
    let key_path = dir.join("key.pem");
    if cert_path.exists() && key_path.exists() {
        return Ok((cert_path, key_path));
    }
    std::fs::create_dir_all(&dir)?;
    let mut names = vec!["localhost".to_string(), "127.0.0.1".to_string()];
    if let Some(host) = platform::hostname() {
        names.push(format!("{host}.local"));
        names.push(host);
    }
    if let Some(ip) = lan_ip() {
        names.push(ip.to_string());
    }
    let cert = rcgen::generate_simple_self_signed(names)?;
    std::fs::write(&cert_path, cert.cert.pem())?;
    platform::write_private(&key_path, cert.key_pair.serialize_pem().as_bytes())?;
    info!("Generated self-signed TLS certificate in {}", dir.display());
    Ok((cert_path, key_path))
}

/// The LAN address other devices reach this server on. Connecting a UDP socket
/// sends no packet; it only asks the OS which interface would route outwards.
pub fn lan_ip() -> Option<std::net::IpAddr> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("192.0.2.1:9").ok()?;
    socket.local_addr().ok().map(|a| a.ip())
}

#[cfg(test)]
mod tests {
    #[test]
    fn fingerprint_is_sha256_of_the_der_bytes() {
        // "hello" in base64; sha256("hello") starts 2C:F2:4D.
        let pem = "-----BEGIN CERTIFICATE-----\naGVsbG8=\n-----END CERTIFICATE-----\n";
        let fp = super::fingerprint(pem).unwrap();
        assert!(fp.starts_with("2C:F2:4D:BA"));
        assert_eq!(fp.len(), 32 * 3 - 1);
        assert!(super::fingerprint("not a certificate").is_none());
    }
}
