use crate::types::{MarketStandarizer, Tick};

use super::*;
use core::fmt;
use serde::{de, Deserializer};
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::str::FromStr;

pub type BetPool = HashMap<MarketOutcome, f64>;

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

    // #[serde(deserialize_with = "deserialize_into_string_array")]
    // pub pool: [String; 2],
    // #[serde(default, deserialize_with = "deserialize_outcome_prices")]
    // pub outcomePrices: Option<[f64; 2]>,
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

#[derive(Deserialize, Debug, Clone, PartialEq, Hash, Eq, Serialize)]
pub enum MarketOutcome {
    #[serde(rename = "YES")]
    Yes,
    #[serde(rename = "NO")]
    No,
    #[serde(untagged)]
    Other(String),
}

impl std::fmt::Display for MarketOutcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MarketOutcome::Yes => write!(f, "YES"),
            MarketOutcome::No => write!(f, "NO"),
            MarketOutcome::Other(s) => write!(f, "{}", s),
        }
    }
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

#[derive(Deserialize, Debug, Clone, PartialEq, Serialize)]
pub struct Bet {
    pub id: String,
    pub userId: String,
    #[serde(with = "ts_milliseconds")]
    pub createdTime: DateTime<Utc>,
    pub probAfter: f32,
    pub probBefore: f32,
    pub shares: f32,
    pub outcome: String,
    pub amount: u32,
}

pub type ProfitMap = HashMap<String, Option<i64>>;
pub type VolumeMap = HashMap<String, i64>;

#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub struct User {
    pub id: String,
    pub createdTime: u64,
    pub username: String,
    pub name: String,
    pub url: String,
    pub cashBalance: f64,
    pub balance: f64,
    pub currentBettingStreak: Option<i32>,

    pub totalDeposits: Option<f64>,
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
        "https://api.manifold.markets/v0/markets/{}/bets",
        market_id.unwrap_or(""),
    );
    let res = reqwest::get(&url).await;
    let bets: Vec<Bet> = res.unwrap().json().await.unwrap();
    bets
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
struct UserResponse {
    #[serde(deserialize_with = "unwrap_user")]
    users: Vec<User>,
}

fn unwrap_user<'de, D>(deserializer: D) -> std::result::Result<Vec<User>, D::Error>
where
    D: Deserializer<'de>,
{
    let users: Vec<User> = Deserialize::deserialize(deserializer)?;
    Ok(users)
}

pub async fn get_all_users(//limit: u32
) -> Result<Vec<User>> {
    let url = "https://api.manifold.markets/v0/users".to_string();
    //?limit={}", limit);
    let client = reqwest::Client::new();
    let res = client.get(&url).send().await.unwrap();
    let res_text = &res.text().await.unwrap();
    // tracing::debug!("{:?}", res_text);
    // let res_text = &res.text().await.unwrap();
    let user_response: Vec<User> = serde_json::from_str(res_text).unwrap();
    //tracing::debug!("{:?}", user_response);
    // Ok(user_response.users)
    // tracing::debug!()
    // let users: Vec<User> = res.json().await.unwrap();
    Ok(user_response)
}
