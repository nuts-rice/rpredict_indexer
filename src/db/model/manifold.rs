use super::*;
use core::fmt;
use std::str::FromStr;
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
enum OutcomeType {
    BINARY,
    MULTIPLE_CHOICE,
    POLL,
}
impl FromStr for OutcomeType {
    type Err = ();
    fn from_str(input: &str) -> std::result::Result<OutcomeType, ()> {
        match input {
            "BINARY" => Ok(OutcomeType::BINARY),
            "MULTIPLE_CHOICE" => Ok(OutcomeType::MULTIPLE_CHOICE),
            "POLL" => Ok(OutcomeType::POLL),
            _ => Err(()),
        }
    }
}

#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub struct ManifoldMarket {
    pub question: String,
    pub id: String,
    #[serde(with = "ts_milliseconds")]
    pub createdTime: DateTime<Utc>,
    #[serde(with = "ts_milliseconds_option")]
    #[serde(default)]
    pub closeTime: Option<DateTime<Utc>>,
    #[serde(with = "ts_milliseconds_option")]
    #[serde(default)]
    pub resolutionTime: Option<DateTime<Utc>>,
    pub totalLiquidity: f64,
    pub outcomeType: OutcomeType,
    pub pool: Option<BetPool>,
    pub probability: f64,
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
    NO: f64,
    YES: f64,
}

impl fmt::Display for ManifoldMarket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.outcomeType == OutcomeType::BINARY {
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
        let createdTime = value["createdTime"]
            .to_string()
            .parse::<DateTime<Utc>>()
            .expect("Failed to parse created time");
        let closeTime = value["closeTime"]
            .to_string()
            .parse::<DateTime<Utc>>()
            .unwrap();
        let volume = value["volume"].as_f64().unwrap();
        let outcomeType = value["outcomeType"]
            .to_string()
            .parse::<OutcomeType>()
            .unwrap();
        let pool = BetPool::from(value["pool"].clone());
        let probability = value["probability"].as_f64().unwrap();
        ManifoldMarket {
            id,
            question,
            createdTime,
            resolutionTime: Some(closeTime),
            closeTime: Some(closeTime),
            probability,
            totalLiquidity: volume,
            outcomeType,
            pool: Some(pool),
        }
    }
}
