use async_trait::async_trait;
use std::any::Any;
pub use tokio::sync::{broadcast, mpsc, watch};
pub mod index;
pub mod platform;
pub mod questions;

use super::types::{Indicators, Question};

// type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub mod augur;
pub mod manifold;
pub mod metaculus;
pub mod polymarket;
// pub trait PlatformResult {}
// impl<T> PlatformResult for Result<T, Box<dyn std::error::Error + Send + Sync>> {}
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub struct PlatformBuilder<T: Platform> {
    marker: std::marker::PhantomData<T>,
    client: reqwest::Client,
    limit: u32,
    endpoint: String,
}

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
    pub async fn fetch_page(&self, limit: u32) -> Result<Vec<Question>> {
        let url = format!("{}?limit={}", self.endpoint, limit);
        let res = reqwest::get(url.clone()).await?.text().await?;
        tracing::debug!("response: {:?}", res);
        let markets: Vec<Question> = serde_json::from_str(&res)?;
        tracing::debug!("markets: {:?}", markets[0]);
        Ok(markets)
    }
    pub async fn fetch_questions(&self) -> Result<Vec<Question>> {
        unimplemented!()
    }

    pub async fn build(&self) -> Result<()> {
        unimplemented!()
    }
}

#[async_trait]
pub trait Platform: From<PlatformBuilder<Self>> + Any {
    async fn fetch_questions(&self) -> Result<Vec<Self::Market>>;
    fn builder() -> PlatformBuilder<Self> {
        PlatformBuilder::new()
    }
    async fn fetch_question_by_id(&self, id: &str) -> Result<Question>;
    async fn fetch_json(&self) -> Result<serde_json::Value>;
    type Market;
    const ENDPOINT: &'static str;
}

impl<P: Platform + Any> PlatformBuilder<P> {
    pub fn new() -> Self {
        Self {
            marker: std::marker::PhantomData,
            client: reqwest::Client::new(),
            endpoint: P::ENDPOINT.to_string(),
            limit: 3,
            //100
        }
    }
    // fn build() -> Builder<Self> {
    //     unimplemented!()
    // }
    fn indicators<I: Into<Indicators>>(self, indicators: I) -> Self {
        unimplemented!()
    }
    fn limit(self, limit: u32) -> Self {
        unimplemented!()
    }
    pub fn build(self) -> P {
        P::from(self)
    }
}

impl<P: Platform + Any> Default for PlatformBuilder<P> {
    fn default() -> Self {
        Self::new()
    }
}