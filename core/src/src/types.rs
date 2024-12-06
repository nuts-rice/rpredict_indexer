use crate::manifold::ManifoldPosition;
use anyhow::Result;
use axum::async_trait;
use chrono::{DateTime, Utc};
use clap::ValueEnum;
use clap::{Arg, Command};
use diesel::{prelude::*, Insertable};
use qdrant_client::{config::QdrantConfig, Qdrant};
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::Arc};
use tokio_stream::Stream;

pub type CollectorStream<'a, M> = Box<dyn Stream<Item = M> + Send + 'a>;
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize)]
pub enum Platform {
    Manifold,
    Metaculus,
    Polymarket,
}

#[async_trait]
pub trait Collector<M>: Send + Sync {
    async fn collect_all(&self) -> Result<Vec<MarketStandarized>>;
    async fn collect_by_id(&self, id: &str) -> Result<MarketStandarized>;
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

#[derive(Debug, Clone)]
enum OutcomeType {
    BINARY,
    MULTIPLE_CHOICE,
    POLL,
}

impl FromStr for OutcomeType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "BINARY" => Ok(OutcomeType::BINARY),
            "MULTIPLE_CHOICE" => Ok(OutcomeType::MULTIPLE_CHOICE),
            "POLL" => Ok(OutcomeType::POLL),
            _ => Err(anyhow::anyhow!("Invalid OutcomeType")),
        }
    }
}

#[derive(Debug, Serialize, Insertable, AsChangeset)]
#[diesel(table_name = article)]
pub struct Article {
    source: String,
    title: String,
    content: String,
    published_date: DateTime<Utc>,
    url: String,
    description: String,
}

#[derive(Debug, Serialize, Insertable, AsChangeset)]
#[diesel(table_name = market)]
pub struct MarketStandarized {
    title: String,
    platform: String,
    platform_id: String,
    created_date: DateTime<Utc>,
    close_time: DateTime<Utc>,
    category: String,
    //pub outcome_type: Option<OutcomeType>,
    prob_at_midpoint: f32,
    prob_at_close: f32,
    prob_each_pct: Vec<f32>,
    prob_each_date: serde_json::Value,
    prob_time_avg: f32,
    resolution: f32,
    number_traders: i32,
    // pub indicators: Option<Indicators>,
    //TODO: add this when can handle multiple choice
    // pub outcome_series: Vec<OutcomeSeries>,
}

// #[derive(Deserialize, Debug, Serialize, Clone)]
// pub struct MarketOutcome {
//     outcome: String,
//     idx: usize,
// }

#[derive(Debug, Clone)]
pub struct Indicators {
    num_forecasts: i32,
    num_forecasters: i32,
    spread: f32,
    shares_volume: f32,
    likes: i32,
    votes: i32,
    stars: i32,
}
// #[derive(Deserialize, Debug, Serialize, Clone)]
// pub struct BetPool {
//     pub NO: Vec<Tick>,
//     pub YES: Vec<Tick>,
// }

#[derive(Deserialize, Debug, Serialize, Clone)]
pub enum StrategyType {
    ARBITRAGE,
    MARKETMAKING,
}

#[derive(Debug, Clone, Default)]
pub struct Tick {
    pub timestamp: DateTime<Utc>,
    pub probability: f32,
}

pub trait MarketStandarizer {
    fn debug(&self) -> String;
    fn num_traders(&self) -> i32;
    fn platform(&self) -> String;
    fn platform_id(&self) -> String;
    fn close_date(&self) -> anyhow::Result<DateTime<Utc>>;
    fn create_date(&self) -> anyhow::Result<DateTime<Utc>>;
    fn category(&self) -> String;
    fn resolution(&self) -> anyhow::Result<f32>;
    fn ticks(&self) -> Vec<Tick>;
    fn prob_at_time(&self, time: DateTime<Utc>) -> anyhow::Result<f32> {
        if time < self.create_date()? {
            return Err(anyhow::anyhow!("Time is before market creation"));
        }
        let mut prev_prob = 0.5;
        for tick in self.ticks() {
            if tick.probability < 0.0 || 1.0 < tick.probability {
                return Err(anyhow::anyhow!("Probability out of bounds"));
            }
            if tick.timestamp > time {
                return Ok(prev_prob);
            }

            prev_prob = tick.probability;
        }
        match self.ticks().last() {
            Some(tick) => Ok(tick.probability),
            None => Ok(0.5),
        }
    }
    fn prob_duration_before_close(&self, dt: chrono::Duration) -> Result<f32> {
        unimplemented!()
    }
    fn prob_at_percent(&self, percent: f32) -> anyhow::Result<f32> {
        unimplemented!()
    }
    fn prob_time_avg_between(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<f32> {
        unimplemented!()
    }
    fn prob_each_date(&self) -> anyhow::Result<serde_json::Value> {
        unimplemented!()
    }
}

table! {
    article (id) {
        id -> Int4,
        source -> Varchar,
        title -> Varchar,
        content -> Text,
        published_date -> Timestamptz,
        url -> Varchar,
        description -> Varchar,
    }
}

table! {
    market (id) {
    id -> Int4,
    title -> Varchar,
    platform -> Varchar,
    platform_id -> Varchar,
    idx -> Int4,
    category -> Varchar,
    created_date -> Timestamptz,
    close_time -> Timestamptz,
    total_liquidity -> Int4,
    outcome_type -> Varchar,
    prob_at_midpoint -> Float4,
    prob_at_close -> Float4,
    prob_each_pct -> Array<Float4>,
    prob_each_date -> Jsonb,
    prob_time_avg -> Float4,
    resolution -> Float4,
    number_traders -> Int4,
    // indicators -> Jsonb,
    }
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
