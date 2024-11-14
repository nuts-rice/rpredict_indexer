use crate::types::{MarketStandarizer, Tick};

use super::*;
use core::fmt;
use serde::{de, Deserializer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

pub type BetPool = HashMap<String, f64>;

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

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct ManifoldMarket {
    pub question: String,
    pub id: String,
    pub createdTime: u64,
    pub closeTime: Option<u64>,
    pub isResolved: bool,
    pub mechanism: String,
    pub pool: Option<BetPool>,
    pub marketTier: Option<String>,
    pub slug: String,
    pub outcomeType: String,
    pub volume: f64,
    pub uniqueBettorCount: i32,
    pub creatorId: String,
    pub resolution: Option<String>,
    pub probability: Option<f64>,
    pub bets: Option<Vec<Bet>>,
    pub lastBetTime: Option<f64>,
    // pub extraInfo: ExtraInfo,
}

impl ManifoldMarket {
    pub async fn get_full_data(&mut self) {
        self.bets = Some(get_bets(Some(&self.id), None).await);
    }

    pub async fn get_updates(&mut self) {
        unimplemented!()
    }

    pub fn probability_history(&self) -> Vec<f32> {
        unimplemented!()
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Bet {
    pub id: String,
    userId: String,
    #[serde(with = "ts_milliseconds")]
    createdTime: DateTime<Utc>,
    probAfter: f32,
    probBefore: f32,
    shares: f32,
    outcome: String,
    amount: u32,
}

#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub struct BinaryManifoldMarket {}

impl BinaryManifoldMarket {}

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

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct ExtraInfo {
    groupSlugs: Option<Vec<String>>,
}

fn deserialize_from<'de, D>(deserializer: D) -> std::result::Result<Option<[f64; 2]>, D::Error>
where
    D: Deserializer<'de>,
{
    unimplemented!()
}

impl MarketStandarizer for ManifoldMarket {
    fn platform(&self) -> String {
        unimplemented!()
    }
    fn platform_id(&self) -> String {
        unimplemented!()
    }
    fn num_traders(&self) -> i32 {
        unimplemented!()
    }
    fn close_date(&self) -> anyhow::Result<DateTime<Utc>> {
        unimplemented!()
    }
    fn create_date(&self) -> anyhow::Result<DateTime<Utc>> {
        unimplemented!()
    }
    fn debug(&self) -> String {
        format!("{:?}", self)
    }

    fn ticks(&self) -> Vec<Tick> {
        unimplemented!()
    }

    fn category(&self) -> String {
        unimplemented!()
    }
    //fn category(&self) -> String {
    //    if let Some(categories) = &self.extraInfo.groupSlugs {
    //        for category in categories {
    //            match category.as_str() {
    //                "118th-congress" => return "Politics".to_string(),
    //                "2024-us-presidential-election" => return "Politics".to_string(),
    //                //"africa" => return "Other".to_string(),
    //                "ai" => return "AI".to_string(),
    //                "ai-alignment" => return "AI".to_string(),
    //                "ai-safety" => return "AI".to_string(),
    //                "arabisraeli-conflict" => return "Politics".to_string(),
    //                "apple" => return "Technology".to_string(),
    //                "baseball" => return "Sports".to_string(),
    //                "basketball" => return "Sports".to_string(),
    //                "biotech" => return "Science".to_string(),
    //                "bitcoin" => return "Crypto".to_string(),
    //                "celebrities" => return "Culture".to_string(),
    //                "chatgpt" => return "AI".to_string(),
    //                "chess" => return "Sports".to_string(),
    //                //"china" => return "Other".to_string(),
    //                "climate" => return "Climate".to_string(),
    //                "crypto-speculation" => return "Crypto".to_string(),
    //                "culture-default" => return "Culture".to_string(),
    //                //"daliban-hq" => return "Other".to_string(),
    //                //"destinygg" => return "Other".to_string(),
    //                //"destinygg-stocks" => return "Other".to_string(),
    //                "donald-trump" => return "Politics".to_string(),
    //                "economics-default" => return "Economics".to_string(),
    //                //"effective-altruism" => return "Other".to_string(),
    //                //"elon-musk-14d9d9498c7e" => return "Other".to_string(),
    //                //"europe" => return "Other".to_string(),
    //                "f1" => return "Sports".to_string(),
    //                "finance" => return "Economics".to_string(),
    //                "football" => return "Sports".to_string(),
    //                "formula-1" => return "Sports".to_string(),
    //                //"fun" => return "Other".to_string(),
    //                "gaming" => return "Culture".to_string(),
    //                "gpt4-speculation" => return "AI".to_string(),
    //                //"health" => return "Other".to_string(),
    //                //"india" => return "Other".to_string(),
    //                "internet" => return "Technology".to_string(),
    //                //"israel" => return "Other".to_string(),
    //                "israelhamas-conflict-2023" => return "Politics".to_string(),
    //                "israeli-politics" => return "Politics".to_string(),
    //                //"latin-america" => return "Other".to_string(),
    //                //"lgbtqia" => return "Other".to_string(),
    //                //"mathematics" => return "Other".to_string(),
    //                "medicine" => return "Science".to_string(),
    //                //"middle-east" => return "Other".to_string(),
    //                "movies" => return "Culture".to_string(),
    //                "music-f213cbf1eab5" => return "Culture".to_string(),
    //                "nfl" => return "Sports".to_string(),
    //                "nuclear" => return "Science".to_string(),
    //                "nuclear-risk" => return "Politics".to_string(),
    //                //"one-piece-stocks" => return "Other".to_string(),
    //                "openai" => return "AI".to_string(),
    //                "openai-9e1c42b2bb1e" => return "AI".to_string(),
    //                "openai-crisis" => return "AI".to_string(),
    //                //"personal-goals" => return "Other".to_string(),
    //                "physics" => return "Science".to_string(),
    //                "politics-default" => return "Politics".to_string(),
    //                "programming" => return "Technology".to_string(),
    //                //"russia" => return "Other".to_string(),
    //                //"sam-altman" => return "Other".to_string(),
    //                "science-default" => return "Science".to_string(),
    //                //"sex-and-love" => return "Other".to_string(),
    //                "soccer" => return "Sports".to_string(),
    //                "space" => return "Science".to_string(),
    //                "speaker-of-the-house-election" => return "Politics".to_string(),
    //                "sports-default" => return "Sports".to_string(),
    //                "startups" => return "Economics".to_string(),
    //                "stocks" => return "Economics".to_string(),
    //                "technical-ai-timelines" => return "AI".to_string(),
    //                "technology-default" => return "Technology".to_string(),
    //                "tennis" => return "Sports".to_string(),
    //                //"the-life-of-biden" => return "Other".to_string(),
    //                "time-person-of-the-year" => return "Culture".to_string(),
    //                "tv" => return "Culture".to_string(),
    //                //"twitter" => return "Technology".to_string(),
    //                "uk-politics" => return "Politics".to_string(),
    //                "ukraine" => return "Politics".to_string(),
    //                "ukrainerussia-war" => return "Politics".to_string(),
    //                "us-politics" => return "Politics".to_string(),
    //                "wars" => return "Politics".to_string(),
    //                "world-default" => return "Politics".to_string(),
    //                _ => continue,
    //            }
    //        }
    //    }
    //    "None".to_string()
    //}

    fn resolution(&self) -> anyhow::Result<f32> {
        // match &self.market
        unimplemented!()
    }
}

pub async fn get_bets(market_id: Option<&str>, market_slug: Option<&str>) -> Vec<Bet> {
    let url = format!(
        "https://api.manifoldfinance.com/markets/{}/bets",
        market_id.unwrap_or(""),
    );
    let res = reqwest::get(&url).await;
    let bets: Vec<Bet> = res.unwrap().json().await.unwrap();
    bets
}
