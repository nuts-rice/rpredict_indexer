use chrono::serde::{ts_milliseconds, ts_milliseconds_option, ts_seconds};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

pub mod augur;
pub mod index;
pub mod manifold;
pub mod metaculus;
pub mod polymarket;
pub mod question;
pub mod search;
pub mod simplebroker;

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

pub trait Market {

} 
