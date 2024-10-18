use chrono::serde::{ts_milliseconds, ts_milliseconds_option, ts_seconds};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use self::metaculus::MetaculusMarket;
use self::polymarket::PolymarketMarket;
use self::question::DBQuestion;
use self::question::QuestionStorage;
pub mod augur;
pub mod gamma;
pub mod index;
pub mod manifold;
pub mod metaculus;
pub mod polymarket;
pub mod question;
pub mod search;
pub mod simplebroker;
pub type Result<T> = std::result::Result<T, Box<MarketError>>;
const DEFAULT_OPENING_PROB: f32 = 0.5;
const SECS_PER_DAY: f32 = (60 * 60 * 24) as f32;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketError {
    details: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct StandardMarket {
    title: String,
    platform: String,
    platform_id: String,
    open_time: DateTime<Utc>,
    close_time: DateTime<Utc>,
    volume_usd: f32,
    num_traders: i32,
    category: String,
    resolution: f32,
    prob_midpoint: f32,
    prob_close: f32,
    prob_tma: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Update {
    time: DateTime<Utc>,
    probability: f32,
}

pub trait Market {
    fn title(&self) -> &str;
    fn platform(&self) -> &str;
    fn platform_id(&self) -> &str;
    fn open_time(&self) -> Result<DateTime<Utc>>;
    fn close_time(&self) -> Result<DateTime<Utc>>;
    fn volume_usd(&self) -> f32;
    fn num_traders(&self) -> i32;
    fn category(&self) -> &str;
    fn resolution(&self) -> f32;
    fn prob_midpoint(&self) -> f32;
    fn prob_close(&self) -> f32;
    fn prob_tma(&self) -> f32;
    fn events(&self) -> Vec<Update>;
    fn prob_at_time(&self, time: DateTime<Utc>) -> Result<f32> {
        if time < self.open_time()? {
            return Err(Box::new(MarketError {
                details: "Time is before market open".to_string(),
            }));
        }
        let mut previous_probability = DEFAULT_OPENING_PROB;
        for event in self.events() {
            if event.time > time {
                return Ok(previous_probability);
            }
            previous_probability = event.probability;
        }
        match self.events().last() {
            None => Ok(DEFAULT_OPENING_PROB),
            Some(event) => Ok(event.probability),
        }
    }
}

async fn store_markets(markets: Vec<StandardMarket>, db: axum::extract::State<QuestionStorage>) {
    let mut db = db.0.lock().await;
    for chunk in markets.chunks(1000) {
        for market in chunk {
            let key = format!("market:{}", market.platform_id);
            let value: DBQuestion = DBQuestion::from(market);
            db.insert(value);
        }
    }
}

impl From<DBQuestion> for StandardMarket {
    fn from(value: DBQuestion) -> Self {
        let open_time: DateTime<Utc> =
            DateTime::parse_from_str(&value.open_time, "%Y %b %d %H:%M:%S.3f %z")
                .unwrap()
                .to_utc();
        let close_time: DateTime<Utc> =
            DateTime::parse_from_str(&value.close_time, "%Y %b %d %H:%M:%S.3f %z")
                .unwrap()
                .to_utc();

        StandardMarket {
            title: value.title,
            platform: value.platform,
            platform_id: value.platform_id,
            open_time,
            close_time,
            volume_usd: value.volume_usd,
            num_traders: value.num_traders,
            category: value.category,
            resolution: value.resolution,
            prob_midpoint: value.prob_midpoint,
            prob_close: value.prob_close,
            prob_tma: value.prob_tma,
        }
    }
}

impl From<PolymarketMarket> for StandardMarket {
    fn from(value: PolymarketMarket) -> Self {
        unimplemented!()
    }
}

impl From<MetaculusMarket> for StandardMarket {
    fn from(value: MetaculusMarket) -> Self {
        unimplemented!()
    }
}
