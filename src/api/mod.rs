use async_trait::async_trait;
use clap::Parser;
use std::any::Any;
pub use tokio::sync::{broadcast, mpsc, watch};
pub mod index;
pub mod platform;
pub mod questions;
pub mod schema;
use super::types::Indicators;

// type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub mod augur;
pub mod gamma;
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

pub enum SortType {
    Oldest,
    Youngest,
    MostVotes,
    Popular,
}

pub enum PlatformType {
    Polymarket,
    Metaculus,
    Manifold,
}

pub enum BinaryOutcome {
    YES,
    NO,
}

#[async_trait]
pub trait Platform: From<PlatformBuilder<Self>> + Any {
    async fn fetch_questions(&self) -> Result<Vec<Self::Market>>;
    fn builder() -> PlatformBuilder<Self> {
        PlatformBuilder::new()
    }
    async fn fetch_json_by_description(&self, description: &str) -> Result<Vec<serde_json::Value>>;
    async fn fetch_question_by_id(&self, id: &str) -> Result<Self::Market>;
    async fn fetch_json(&self) -> Result<Vec<serde_json::Value>>;
    async fn build_order(
        &self,
        contract_id: &str,
        amount: f64,
        nonce: &str,
        outcome: &str,
    ) -> Result<()>;
    async fn fetch_ratelimited(
        request_count: usize,
        interval_ms: Option<u64>,
    ) -> PlatformBuilder<Self>;
    async fn fetch_events(&self, limit: Option<u64>, offset: u64) -> Result<Vec<Self::Event>>;
    async fn fetch_orderbook(&self, id: &str) -> Result<Vec<Self::Position>>;
    async fn fetch_markets_by_terms(&self, terms: &str) -> Result<Vec<Self::Market>>;
    async fn subscribe_to(&self) -> Result<()>;
    type Market;
    type Event;
    type Position;
    const ENDPOINT: &'static str;
    const SORT: &'static str;
}
#[derive(Debug, Clone)]
pub struct MarketConvertError {
    data: String,
    message: String,
    level: u8,
}
impl std::fmt::Display for MarketConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.message, self.data)
    }
}

async fn base_request<T: for<'de> serde::Deserialize<'de>>(
    req: reqwest_middleware::RequestBuilder,
) -> std::result::Result<T, MarketConvertError> {
    let req_clone = req.try_clone().unwrap().build().unwrap();
    let final_url = req_clone.url();
    let response = match req.send().await {
        Ok(response) => Ok(response),
        Err(e) => Err(MarketConvertError {
            data: e.to_string(),
            message: "Error sending request.".to_string(),
            level: 5,
        }),
    }
    .unwrap();
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| MarketConvertError {
            data: final_url.to_string(),
            message: e.to_string(),
            level: 4,
        })
        .unwrap();
    if !status.is_success() {
        return Err(MarketConvertError {
            data: text.to_owned(),
            message: format!("Query to {} returned status code {}.", final_url, status),
            level: 4,
        });
    }
    serde_json::from_str(&text).map_err(|e| MarketConvertError {
        data: text,
        message: e.to_string(),
        level: 3,
    })
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
    pub fn limit(&mut self, new_limit: u32) {
        self.limit = new_limit;
    }
    pub fn endpoint(&mut self, new_endpoint: &str) {
        self.endpoint = new_endpoint.to_string();
    }

    // fn build() -> Builder<Self> {
    //     unimplemented!()
    // }
    fn indicators<I: Into<Indicators>>(self, indicators: I) -> Self {
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

#[derive(Parser, Debug)]
pub struct Args {}

pub async fn senf_request_retries() {}
