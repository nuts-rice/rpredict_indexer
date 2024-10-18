use super::*;
use axum::Json;
use core::fmt;
use serde::{de, Deserializer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PolymarketResult {
    next_cursor: String,
    pub data: Vec<PolymarketMarket>,
    // tokens: Option<Vec<PolymarketToken>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PolymarketMarket {
    pub active: bool,
    pub question: String,
    #[serde(rename = "questionId")]
    pub question_id: Option<String>,
    #[serde(deserialize_with = "deserialize_into_string_array")]
    pub outcomes: [String; 2],
    accepting_orders: bool,
    #[serde(deserialize_with = "deserialize_outcome_prices")]
    pub outcome_prices: Option<[f64; 2]>,
    // category: String,
    // is_50_50_outcome: bool,
    #[serde(deserialize_with = "deserialize_into_string_array")]
    pub clob_token_ids: [String; 2],
    pub spread: f64,
    pub order_price_min_tick_size: f64,
    // tokens: Option<Vec<PolymarketToken>>,
    // rewards: Option<PolymarketRewards>,
    // events: Option<Vec<PolymarketEvent>>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PricesHistoryPoint {
    #[serde(with = "ts_seconds")]
    timestamp: DateTime<Utc>,
    price: f64,
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
