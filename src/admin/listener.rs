use crate::api::manifold;
use crate::types::{Market, Tick};
use serde::{Serialize, Serializer};
use sled::Db;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
pub type MarketRequestSnd = tokio::sync::mpsc::Sender<serde_json::Value>;
pub type MarketRequestRcv = tokio::sync::mpsc::Receiver<serde_json::Value>;
pub type MarketUpdateSnd = tokio::sync::mpsc::Sender<serde_json::Value>;
pub type MarketUpdateRcv = tokio::sync::mpsc::Receiver<serde_json::Value>;
pub type TickRequestSnd = tokio::sync::mpsc::Sender<TickSnd>;
pub type TickRequestRcv = tokio::sync::mpsc::Receiver<TickSnd>;
pub type TickSnd = tokio::sync::mpsc::Sender<Tick>;
pub type TickRcv = tokio::sync::mpsc::Receiver<Tick>;

#[derive(Clone, Debug, Default, Serialize)]
pub struct MarketRequest {
    #[serde(rename = "markets")]
    #[serde(serialize_with = "serialize_markets")]
    markets: Vec<Market>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_uuid_as_string")]
    uuid: Option<Uuid>,
}

impl From<Market> for MarketRequest {
    fn from(market: Market) -> Self {
        Self {
            markets: vec![market],
            uuid: None,
        }
    }
}

impl MarketRequest {
    // async fn process_market_request(self, mut market_request_rx: MarketRequestRcv, markets_list: Arc<RwLock<Vec<serde_json::Value>>) {
    //     loop {
    //         while let Some(incoming) = market_request_rx.recv().await {
    //             let markets = markets_list.read().unwrap();
    //             let _ = incoming.send(MarketRequest { markets: markets.clone(), uuid: None });
    //         }
    //     }
    // }
    pub fn new() -> Self {
        Default::default()
    }

    pub fn push_market<S: Into<Market>>(mut self, market: S) -> Self {
        self.markets.push(market.into());
        self
    }

    pub fn add_market<S: Into<Market>>(&mut self, market: S) {
        self.markets.push(market.into());
    }

    pub fn markets(&self) -> &Vec<Market> {
        &self.markets
    }

    pub fn uuid(&self) -> &Option<Uuid> {
        &self.uuid
    }

    async fn process_tick_request(
        self,
        mut tick_request_rx: TickRequestRcv,
        tick: Arc<RwLock<Tick>>,
    ) {
        loop {
            while let Some(incoming) = tick_request_rx.recv().await {
                let current_tick = *tick.read().unwrap();
                let _ = incoming.send(current_tick);
            }
        }
    }
}

fn serialize_uuid_as_string<S>(x: &Option<Uuid>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Don't need to handle None option here as handled by
    // #[serde(skip_serializing_if = "Option::is_none")]
    s.serialize_str(&x.unwrap().to_string())
}

pub fn serialize_markets<S: Serializer>(markets: &[Market], s: S) -> Result<S::Ok, S::Error> {
    let raw_markets: Vec<serde_json::Value> = markets
        .iter()
        .map(|market| match market {
            Market::NewMarket(inner) => serde_json::to_value(inner).unwrap(),
            Market::MarketPosition(inner) => serde_json::to_value(inner).unwrap(),
        })
        .collect();
    raw_markets.serialize(s)
}

pub async fn listen_for_requests(
    markets_list: Arc<RwLock<Vec<serde_json::Value>>>,
    cache: Db,
    market_update_rx: MarketUpdateRcv,
    platform: manifold::ManifoldPlatform,
) -> Result<(), Box<dyn std::error::Error>> {
    unimplemented!()
}

// pub async fn listen_for_ticks(
//     markets_list: Arc<RwLock<Vec<serde_json::Value>>>,
//     tick_rx: TickRcv,
//     tick_status: Arc<RwLock<Tick>>,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     while let Some(tick) = tick_rx.recv().await {
//         let mut tick = tick_status.write().unwrap();
//         let tick_value = serde_json::to_string(&tick)?;
//     }
// }

pub async fn admin_api_server() {
    unimplemented!()
}

async fn process_tick_request(mut tick_request_rx: TickRequestRcv, tick: Arc<RwLock<Tick>>) {
    loop {
        while let Some(incoming) = tick_request_rx.recv().await {
            let current_tick = *tick.read().unwrap();
            let _ = incoming.send(current_tick);
        }
    }
}

// async fn accept_tick_request(
//     tx: serde_json::Value,
//     tick_request_snd: TickRequestSnd,
//     tick: Arc<RwLock<Tick>>,
// ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
//     let mut tx = parse
//     tick_request_snd.send(tx).await?;
//     process_tick_request(rx, tick).await;
//     Ok(())
// }
