pub mod error;

pub use error::MetaError;
use reqwest::Client;

#[derive(Clone, Debug)]
pub struct TmdbClient {
    client: Client,
    api_key: Option<String>,
}

impl TmdbClient {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            client: Client::builder().build().unwrap_or_default(),
            api_key,
        }
    }

    pub fn has_key(&self) -> bool {
        self.api_key.is_some()
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}
