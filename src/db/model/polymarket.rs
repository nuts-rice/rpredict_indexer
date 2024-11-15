use super::*;
use crate::utils::auth::{self, AmpCookie};
use axum::Json;
use core::fmt;
use reqwest::Proxy;
use serde::{de, Deserializer};
use serde::{Deserialize, Serialize};
const CLOB_URL: &str = "https://clob.polymarket.com/";
const CLOB_WS_URL: &str = "wss://ws-subscriptions-clob.polymarket.com/ws/";
const CLOB_TRADES_URL: &str = "https://clob.polymarket.com/data/trades";
const CLOB_PRICE_HISTORY_URL: &str = "https://clob.polymarket.com/prices-history";
const POLYMARKET_RATELIMIT: u32 = 50;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PolymarketResult {
    next_cursor: String,
    pub data: Vec<PolymarketMarket>,
    // tokens: Option<Vec<PolymarketToken>>,
}

pub struct PolymarketTimeSeries {
    history: Vec<TimeseriesPoint>,
}

pub struct TimeseriesPoint {
    t: u64,
    p: f64,
}

impl PolymarketTimeSeries {
    pub fn get_time_series(&self, market: u64, start: u64, end: u64) -> Vec<TimeseriesPoint> {
        unimplemented!()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PolymarketMarket {
    pub active: bool,
    pub question: Option<String>,
    #[serde(rename = "questionId")]
    pub question_id: Option<String>,
    // #[serde(deserialize_with = "deserialize_into_string_array")]
    // pub outcomes: [String; 2],
    pub accepting_orders: Option<bool>,
    #[serde(deserialize_with = "deserialize_outcome_prices")]
    pub outcome_prices: Option<[f64; 2]>,
    pub category: Option<String>,
    // is_50_50_outcome: bool,
    // #[serde(deserialize_with = "deserialize_into_string_array")]
    // pub clob_token_ids: [String; 2],
    pub spread: f64,
    pub order_price_min_tick_size: f64,
    pub tokens: Option<Vec<PolymarketToken>>,
    pub rewards: Option<PolymarketRewards>,
    // events: Option<Vec<PolymarketEvent>>,
}

pub struct PolymarketUser {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MakerOrder {
    order_id: String,
    maker_address: String,
    owner: String,
    matched_amount: String,
    price: String,
    asset_id: String,
    outcome: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PolymarketTrade {
    id: String,
    taker_order_id: String,
    maker_orders: Vec<MakerOrder>,
    owner: String,
    size: String,
    side: String,
    fee_rate_bps: String,
    price: String,
}

pub struct PolymarketPosition {
    position_id: String,
    owner: String,
    size: String,
    price: String,
    outcome: String,
    asset_id: String,
    status: String,
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
#[serde(rename_all = "camelCase")]
pub struct PolymarketEvent {
    pub id: String,
    pub title: String,
    pub markets: Vec<PolymarketMarket>,
    pub slug: String,
    pub neg_risk: Option<bool>,
}

fn deserialize_outcome_prices<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<[f64; 2]>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt_s: Option<String> = Option::deserialize(deserializer)?;

    match opt_s {
        Some(s) => {
            let vec_str: Vec<String> =
                serde_json::from_str(&s).map_err(|err| de::Error::custom(err.to_string()))?;

            if vec_str.len() != 2 {
                return Err(de::Error::invalid_length(
                    vec_str.len(),
                    &"expected an array of length 2",
                ));
            }

            let mut vec_f64 = [0.0; 2];
            for (i, val_str) in vec_str.iter().enumerate() {
                vec_f64[i] = val_str
                    .parse::<f64>()
                    .map_err(|e| de::Error::custom(e.to_string()))?;
            }

            Ok(Some(vec_f64))
        }
        None => Ok(None),
    }
}

impl PolymarketTrade {
    pub fn get_trade(&self, id: &str) -> Result<PolymarketTrade> {
        let url = format!("{}/{}", CLOB_TRADES_URL, id);
        unimplemented!()
    }
}

pub fn parse_polymarket_text(text: &str) -> Json<Vec<PolymarketMarket>> {
    let markets = serde_json::from_str(text).unwrap();

    Json(markets)
}

impl PolymarketEvent {
    pub fn get_url(&self) -> String {
        format!("https://polymarket.com/event/{}", self.slug)
    }
}

impl std::fmt::Display for PolymarketEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.title, self.get_url(),)
    }
}
impl fmt::Display for PolymarketMarket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl From<Vec<serde_json::Value>> for PolymarketResult {
    fn from(value: Vec<serde_json::Value>) -> Self {
        let data = value[0]["data"].clone();
        let next_cursor = value[0]["next_cursor"].to_string().clone();

        let markets: Vec<PolymarketMarket> = serde_json::from_value(data).unwrap();
        PolymarketResult {
            next_cursor,
            data: markets,
        }
    }
}

pub async fn get_user(
    amp_cookie: &mut AmpCookie,
    polymarket_nonce: &str,
    polymarket_session: &str,
    proxy: Option<&Proxy>,
) -> Result<PolymarketUser> {
    unimplemented!()
}

pub async fn get_all_users(limit: u32) -> Result<Vec<PolymarketUser>> {
    unimplemented!()
}

fn deserialize_into_string_array<'de, D>(
    deserializer: D,
) -> std::result::Result<[String; 2], D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;

    let vec: Vec<String> =
        serde_json::from_str(&s).map_err(|err| de::Error::custom(err.to_string()))?;

    if vec.len() != 2 {
        return Err(de::Error::invalid_length(
            vec.len(),
            &"expected an array of length 2",
        ));
    }

    Ok([vec[0].clone(), vec[1].clone()])
}
