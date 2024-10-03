use std::collections::HashMap;
use std::str::FromStr;

use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PolymarketMarket {
    active: Option<bool>,
    description: Option<String>,
    question: Option<String>,
    question_id: Option<String>,
    accepting_orders: Option<bool>,
    // tokens: Option<Vec<PolymarketToken>>,
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
