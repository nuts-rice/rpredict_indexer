use core::fmt;
use serde::Deserializer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub enum OutcomeType {
    BINARY,
    MULTIPLE_CHOICE,
    POLL,
    BOUNTIED_QUESTION,
    PSEUDO_NUMERIC,
    NUMBER,
    STONK,
}
impl FromStr for OutcomeType {
    type Err = ();
    fn from_str(input: &str) -> std::result::Result<OutcomeType, ()> {
        match input {
            "BINARY" => Ok(OutcomeType::BINARY),
            "MULTIPLE_CHOICE" => Ok(OutcomeType::MULTIPLE_CHOICE),
            "POLL" => Ok(OutcomeType::POLL),
            "BOUNTIED_QUESTION" => Ok(OutcomeType::BOUNTIED_QUESTION),
            "PSEUDO_NUMERIC" => Ok(OutcomeType::PSEUDO_NUMERIC),
            "NUMBER" => Ok(OutcomeType::NUMBER),
            "STONK" => Ok(OutcomeType::STONK),
            _ => Err(()),
        }
    }
}

#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub struct ManifoldMarket {
    pub question: String,
    pub id: String,
    //    #[serde(with = "ts_milliseconds")]
    // pub createdTime: Option<u64>,
    // // #[serde(with = "ts_milliseconds_option")]
    // // #[serde(default)]
    // pub closeTime: Option<u64>,
    // // #[serde(with = "ts_milliseconds_option")]
    // // #[serde(default)]
    // pub resolutionTime: Option<u64>,
    // pub totalLiquidity: Option<f64>,
    pub outcomeType: Option<OutcomeType>,
    pub pool: Option<BetPool>,
    pub probability: Option<f64>,
    // pub positions: Option<Vec<Position>>,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct ManifoldEvent {}
// #[derive(Deserialize, Debug, Serialize)]
// pub struct Indicators {
//     num_forecasts: i32,
//     num_forecasters: i32,
//     spread: f32,
//     shares_volume: f32,
//     likes: i32,
//     votes: i32,
//     stars: i32,
// }
#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub struct BetPool {
    pub NO: f64,
    pub YES: f64,
}
pub type PostionFrom = HashMap<String, [u64; 5]>;
pub type PositionShares = HashMap<String, f64>;
#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub struct ManifoldPosition {
    pub id: u64,
    // #[serde(deserialize_with = "deserialize_from")]
    // pub from : PostionFrom,
    pub hasShares: bool,
    pub invested: f64,
    pub loan: f64,
    pub maxSharesOutcome: Option<String>,
    pub payout: f64,
    pub profit: f64,
    pub totalShares: PositionShares,
    pub userId: Option<String>,
    pub userUsername: Option<String>,
    pub lastBetTime: u64,
}
fn deserialize_from<'de, D>(deserializer: D) -> std::result::Result<Option<[f64; 2]>, D::Error>
where
    D: Deserializer<'de>,
{
    unimplemented!()
}

// #[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
// pub struct PositionFrom {
//     period: String,
//     profit: u64,
//     profitPercent: f64,
//     invested: u64,
//     prevValue: u64,
//     value: u64,
// }
impl fmt::Display for ManifoldMarket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if (self.outcomeType) == Some(OutcomeType::BINARY) {
            let question = self.question.to_string();
            let pool = self.pool.as_ref().unwrap();
            let total_pool = pool.NO + pool.YES;
            let yes_share = pool.YES / total_pool;
            let no_share = pool.NO / total_pool;
            if yes_share >= 0.8 {
                write!(f, "{}...YES: {} Pretty Certain", question, yes_share)
            } else if yes_share >= 0.6 {
                write!(f, "{}...YES: {} Likely", question, yes_share)
            } else if yes_share >= 0.4 {
                write!(f, "{}...YES: {} Maybe", question, yes_share)
            } else if yes_share >= 0.2 {
                write!(f, "{}...YES: {} Unlikely", question, yes_share)
            } else {
                write!(f, "{}...YES: {} Very unlikely", question, yes_share)
            }
        } else {
            let question = self.question.to_string();
            write!(f, "{}...", question,)
        }
    }
}
impl From<serde_json::Value> for BetPool {
    fn from(value: serde_json::Value) -> Self {
        let NO = value["NO"].as_f64().unwrap();
        let YES = value["YES"].as_f64().unwrap();
        BetPool { NO, YES }
    }
}
// impl
impl From<serde_json::Value> for ManifoldMarket {
    fn from(value: serde_json::Value) -> Self {
        let id = value["id"].to_string();
        let question = value["question"].to_string();
        // let createdTime = value["createdTime"]
        //     .to_string()
        //     .parse::<u64>()
        //     .expect("Failed to parse created time");
        // let closeTime = value["closeTime"]
        //     .to_string()
        //     .parse::<u64>()
        //     .unwrap();
        // let volume = value["volume"].as_f64().unwrap();
        let outcomeType = value["outcomeType"]
            .to_string()
            .parse::<OutcomeType>()
            .unwrap();
        let pool = BetPool::from(value["pool"].clone());
        let probability = value["probability"].as_f64().unwrap();
        // let positions = value["positions"].as_array().unwrap();
        ManifoldMarket {
            id,
            question,
            // createdTime: Some(createdTime),
            // resolutionTime: Some(closeTime),
            // closeTime: Some(closeTime),
            probability: Some(probability),
            outcomeType: Some(outcomeType),
            pool: Some(pool),
        }
    }
}
