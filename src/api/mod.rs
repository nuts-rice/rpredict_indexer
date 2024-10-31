use async_trait::async_trait;
use clap::Parser;
use std::any::Any;
pub use tokio::sync::{broadcast, mpsc, watch};
pub mod index;
pub mod platform;
pub mod questions;

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
    async fn incoming_position_to_value(
        &self,
        position: Self::Position,
    ) -> Result<serde_json::Value>;
    async fn incoming_market_to_value(&self, market: Self::Market) -> Result<serde_json::Value>;

    type Market;
    type Event;
    type Position;
    const ENDPOINT: &'static str;
    const SORT: &'static str;
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
