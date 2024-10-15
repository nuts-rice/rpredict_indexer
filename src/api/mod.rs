use async_trait::async_trait;
use axum::extract::Query;
use clap::{Arg, Parser};
use std::any::Any;
pub use tokio::sync::{broadcast, mpsc, watch};
pub mod index;
pub mod platform;
pub mod questions;

use self::index::Pagiation;

use super::types::{Indicators, Question};

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

#[async_trait]
pub trait Platform: From<PlatformBuilder<Self>> + Any {
    async fn fetch_questions(&self) -> Result<Vec<Self::Market>>;
    fn builder() -> PlatformBuilder<Self> {
        PlatformBuilder::new()
    }
    async fn fetch_json_by_description(&self, description: &str) -> Result<serde_json::Value>;
    async fn fetch_question_by_id(&self, id: &str) -> Result<Question>;
    async fn fetch_json(&self) -> Result<serde_json::Value>;
    async fn build_order(&self, token: &str, amount: f64, nonce: &str);
    async fn fetch_ratelimited(
        request_count: usize,
        interval_ms: Option<u64>,
    ) -> PlatformBuilder<Self>;
    async fn fetch_events(pagiation: Option<Query<Pagiation>>) -> Result<Vec<Self::Event>>;
    type Market;
    type Event;
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

#[derive(Parser, Debug)]
pub struct Args {}

pub async fn senf_request_retries() {}
