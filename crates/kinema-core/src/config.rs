use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::CoreError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    3000
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}

/// HTTPS. On by default with a self-signed certificate made in `data_dir/tls`; apps pin
/// its fingerprint. Set `cert_path` and `key_path` to use a real certificate, or turn it
/// off only behind a proxy that terminates TLS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub cert_path: Option<PathBuf>,
    #[serde(default)]
    pub key_path: Option<PathBuf>,
}

fn default_true() -> bool {
    true
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cert_path: None,
            key_path: None,
        }
    }
}

/// Who the server believes about a client's address. Profiles that open without a PIN
/// only do so from a private (LAN) address, so a forwarded address is only trusted when
/// the request comes straight from one of `trusted_proxies` (IPs or CIDRs, e.g.
/// `["127.0.0.1", "10.0.0.0/8"]`). Empty: the socket peer is the client, and any
/// request carrying forwarding headers counts as remote.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkConfig {
    #[serde(default)]
    pub trusted_proxies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub tls: TlsConfig,
    #[serde(default)]
    pub network: NetworkConfig,
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,
    #[serde(default)]
    pub tmdb_api_key: Option<String>,
}

fn default_data_dir() -> PathBuf {
    PathBuf::from("data")
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            tls: TlsConfig::default(),
            network: NetworkConfig::default(),
            data_dir: default_data_dir(),
            tmdb_api_key: None,
        }
    }
}

impl Config {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, CoreError> {
        let content = std::fs::read_to_string(path.as_ref())?;
        toml::from_str(&content).map_err(|e| CoreError::Config(e.to_string()))
    }

    pub fn load() -> Result<Self, CoreError> {
        let path = Path::new("kinema.toml");
        if path.exists() {
            Self::load_from_file(path)
        } else {
            Ok(Self::default())
        }
    }
}
