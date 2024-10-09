use core::fmt;
use std::str::FromStr;

use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PolymarketResult {
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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PolymarketToken {
    token_id: String,
    outcome: String,
    // price: f64,
    // winner: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PolymarketRewards {
    min_size: f64,
    max_spread: f64,
    is_50_50_outcome: bool,
    rates: Vec<PolymarketRates>,
    // event_start_date: String,
    // event_end_date: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PolymarketRates {
    asset_address: String,
    rewards_daily_rate: i64,
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
impl FromStr for PolymarketMarket {
    type Err = serde_json::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

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
        let markets: Vec<PolymarketMarket> = serde_json::from_value(data).unwrap();
        PolymarketResult { data: markets }
    }
}
