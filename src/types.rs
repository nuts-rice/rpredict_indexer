use crate::{
    api::{self, *},
    manifold::{ManifoldMarket, ManifoldPosition},
};
use anyhow::Result;
use axum::async_trait;
use chrono::{DateTime, Utc};
use clap::{Arg, Command};
use qdrant_client::{config::QdrantConfig, Qdrant};
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};
use tokio_stream::Stream;
use tokio_stream::StreamExt;

pub type CollectorStream<'a, M> = Box<dyn Stream<Item = M> + Send + 'a>;

#[async_trait]
pub trait Collector<M>: Send + Sync {
    fn collect(&self) -> Result<CollectorStream<'_, M>>;
}
pub trait Executor<M>: Send + Sync {
    fn execute(&self, m: M);
}

#[async_trait]
pub trait Strategy<M>: Send + Sync {
    fn process_market(&self, m: M);
    fn process_tick(&self, m: M);
    fn process_position(&self, m: M);
    fn process_news(&self, m: M);
}

pub enum MarketEvent {
    Market(MarketStandarized),
    Tick(Tick),
    Position(ManifoldPosition),
    // News(News),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum OutcomeType {
    BINARY,
    MULTIPLE_CHOICE,
    POLL,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct MarketStandarized {
    pub question: String,
    pub idx: usize,
    pub created_time: i64,
    pub close_time: i64,
    pub total_liquidity: i32,
    pub outcome_type: Option<OutcomeType>,
    pub pool: Option<BetPool>,
    pub indicators: Option<Indicators>,
    //TODO: add this when can handle multiple choice
    // pub outcome_series: Vec<OutcomeSeries>,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct MarketOutcome {
    outcome: String,
    idx: usize,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct Indicators {
    num_forecasts: i32,
    num_forecasters: i32,
    spread: f32,
    shares_volume: f32,
    likes: i32,
    votes: i32,
    stars: i32,
}
#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct BetPool {
    pub NO: Vec<Tick>,
    pub YES: Vec<Tick>,
}

#[derive(Deserialize, Debug, Serialize, Clone)]

pub enum StrategyType {
    ARBITRAGE,
    MARKETMAKING,
}

#[derive(Deserialize, Debug, Serialize, Clone, Default)]
pub struct Tick {
    pub timestamp: DateTime<Utc>,
    pub volume: f32,
}

#[derive(Clone)]
pub struct StrategyConfig {
    pub id: String,
    pub strategy_type: StrategyType,
    pub enabled: bool,
    pub ticks: Vec<Tick>,
    pub period: u64,
    pub qdrant: Arc<Qdrant>,
    pub collection_name: String,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            collection_name: "default".to_string(),
            strategy_type: StrategyType::ARBITRAGE,
            qdrant: Arc::new(Qdrant::new(QdrantConfig::default()).unwrap()),
            enabled: true,
            ticks: vec![],
            period: 60,
        }
    }
}

pub fn create_match() -> clap::Command {
    Command::new("rpredict_indexer")
        .arg(
            Arg::new("question_list")
                .num_args(1..)
                .conflicts_with("config"),
        )
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .num_args(1..)
                .default_value("config.toml")
                .conflicts_with("question_list")
                .help("Path to the configuration file"),
        )
        .arg(
            Arg::new("period")
                .long("period")
                .num_args(1..)
                .default_value("15")
                .help("period in seconds"),
        )
        .arg(
            Arg::new("db")
                .long("db")
                .short('d')
                .num_args(1..)
                .default_value("rpredict-cache")
                .help("Database path"),
        )
}
// impl Default for Context {
//     fn default() -> Self {
//         let platform = api::manifold::ManifoldPlatform::from(PlatformBuilder::default());

//         Self {
//             manifold: platform,
//             exit: false,

//             id: "default".to_string(),
//             strategy_config: Arc::new(RwLock::new(StrategyConfig::default())),
//             questions: vec![],
//             indicators: Indicators {
//                 num_forecasts: 0,
//                 num_forecasters: 0,
//                 spread: 0.0,
//                 shares_volume: 0.0,
//                 likes: 0,
//                 votes: 0,
//                 stars: 0,
//             },
//         }
//     }
// }
