use super::*;
use axum::Json;
use core::fmt;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PolymarketResult {
    next_cursor: String,
    pub data: Vec<PolymarketMarket>,
    // tokens: Option<Vec<PolymarketToken>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PolymarketMarket {
    active: bool,
    description: Option<String>,
    question: Option<String>,
    question_id: Option<String>,
    accepting_orders: bool,
    // category: String,
    // is_50_50_outcome: bool,
    tokens: Option<Vec<PolymarketToken>>,
    rewards: Option<PolymarketRewards>,
    events: Option<Vec<PolymarketEvent>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PolymarketToken {
    token_id: String,
    outcome: String,
    // price: f64,
    winner: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PolymarketRewards {
    min_size: f64,
    max_spread: f64,
    // is_50_50_outcome: bool,
    rates: Vec<PolymarketRates>,
    // event_start_date: String,
    // event_end_date: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PolymarketRates {
    asset_address: String,
    rewards_daily_rate: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PolymarketEvent {
    id: String,
    ticker: Option<String>,
    title: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    liquidity: Option<f64>,
    volume: Option<f64>,
    markets: Option<Vec<PolymarketMarket>>,
    active: Option<bool>,
    closed: Option<bool>,
    restricted: Option<bool>,
    order_min_size: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PricesHistoryPoint {
    #[serde(with = "ts_seconds")]
    timestamp: DateTime<Utc>,
    price: f64,
}

// type PolymarketToken = HashMap<>

// #[derive(Debug, Serialize, Deserialize, Clone)]
// struct PolymarketToken {
//     token_id: u64,
//     outcome: String,
//     price: f64,
//     winner: bool,
// }
//
// impl FromStr for PolymarketMarket {
//     type Err = serde_json::Error;
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         serde_json::from_str(s)
//     }
// }

pub fn parse_polymarket_text(text: &str) -> Json<Vec<PolymarketMarket>> {
    let markets = serde_json::from_str(text).unwrap();

    Json(markets)
}

impl fmt::Display for PolymarketMarket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl From<serde_json::Value> for PolymarketResult {
    fn from(value: serde_json::Value) -> Self {
        let data = value["data"].clone();
        let next_cursor = value["next_cursor"].to_string().clone();

        let markets: Vec<PolymarketMarket> = serde_json::from_value(data).unwrap();
        PolymarketResult {
            next_cursor,
            data: markets,
        }
    }
}
