pub use tokio::sync::{broadcast, mpsc, watch};
pub mod platform;
pub mod questions;
use super::types::Question;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
trait Builder {}

pub struct ConnectionParams {}

pub struct RequestChannels {}

pub struct APIClient {
    client: reqwest::Client,
    endpoint: String,
}

impl APIClient {
    pub fn new(endpoint: &str) -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::new(),
            endpoint: endpoint.to_string(),
        })
    }
    pub async fn fetch_page(&self) -> Result<String> {
        let res = reqwest::get(self.endpoint.clone()).await?.text().await?;
        tracing::debug!("response: {:?}", res);
        let markets: Vec<Question> = serde_json::from_str(&res)?;
        tracing::debug!("markets: {:?}", markets[0]);
        Ok(res)
    }
    pub async fn fetch_questions(&self) -> Result<Vec<Question>> {
        unimplemented!()
    }
}
