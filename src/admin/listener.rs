use crate::api::manifold;
use crate::context::Context;
use crate::types::StrategyConfig;
use crate::types::Tick;
use sled::Db;
use std::sync::{Arc, RwLock};
use tracing::info;
pub type MarketRequestSnd = tokio::sync::mpsc::Sender<serde_json::Value>;
pub type MarketRequestRcv = tokio::sync::mpsc::Receiver<serde_json::Value>;
pub type MarketUpdateSnd = tokio::sync::mpsc::Sender<serde_json::Value>;
pub type MarketUpdateRcv = tokio::sync::mpsc::Receiver<serde_json::Value>;
pub type TickSnd = tokio::sync::mpsc::Sender<Tick>;
pub type TickRcv = tokio::sync::mpsc::Receiver<Tick>;

pub async fn listen_for_requests(
    markets_list: Arc<RwLock<Vec<serde_json::Value>>>,
    cache: Db,
    market_update_rx: MarketUpdateRcv,
    platform: manifold::ManifoldPlatform,
) -> Result<(), Box<dyn std::error::Error>> {
    unimplemented!()
}

pub async fn listen_for_ticks(
    markets_list: Arc<RwLock<Vec<serde_json::Value>>>,
    cache: Db,
    tick_rx: TickRcv,
) -> Result<(), Box<dyn std::error::Error>> {
    unimplemented!()
}

pub async fn admin_api_server() {
    unimplemented!()
}
