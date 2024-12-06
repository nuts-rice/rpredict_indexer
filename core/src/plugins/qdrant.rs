use crate::context::Context;
use crate::types::Tick;
use std::sync::{Arc, RwLock};
pub type MarketUpdateRecv = tokio::sync::mpsc::Receiver<MarketUpdateResult>;
pub type MarketUpdateSend = tokio::sync::mpsc::Sender<MarketUpdateResult>;
#[derive(Debug, Default)]
pub struct MarketUpdateResult {
    pub market_idx: usize,
    pub tick: Tick,
}
pub async fn aggregate_data(
    ctx: Arc<RwLock<Context>>,
    tx: tokio::sync::watch::Sender<u64>,
    market_tx: MarketUpdateSend,
    chunk_size: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // loop {
    // let ctx = ctx.read().unwrap();
    // let mut qdrant = ctx.strategy_config.read().unwrap().qdrant.to_owned();
    // tokio::time::sleep(Duration::from_millis(ctx.strategy_config.read().unwrap().period)) ;
    // let collection_name = ctx.strategy_config.read().unwrap().collection_name.clone();
    // let len = ctx.questions.len();
    // let mut futures = Vec::new();
    // if len == 0 {
    //     return Ok(());
    // }
    // for i 0..len {
    //     let question = ctx.questions[i].clone();
    //     let tick : Tick =
    //     let qdrant = qdrant.clone();
    //     let market_tx = market_tx.clone();
    //     let future = async move {
    //         let _ = update_market(question, tick, outcome, &market_tx).await;
    //     };
    //     futures.push(future);
    // }
    // qdrant.upsert_points_chunked(, chunk_size)
    // }
    Ok(())
}

// pub async fn update_markets(
//     markets: &Arc<RwLock<Vec<MarketStandarized>>>, ctx: &Arc<RwLock<Context>>)
//      -> Result<Vec<MarketUpdateResult>, Box<dyn std::error::Error>> {
//     let len = markets.read().unwrap().len();
//     let mut market_futures = Vec::new();
//     let mut updates = Vec::<MarketUpdateResult>::new();
//     if len == 0 {
//         return Ok(updates);
//     }
//     let (tx, mut rx) = tokio::sync::mpsc::channel(len);
//     for i in 0..len {
//         let market_clone = markets.read().unwrap()[i].clone();
//         let tx = tx.clone();
//         let market_future = async move {
//             let market = market_clone.;
//             let result = tokio::time::timeout(Duration::from_millis(), future)
//             let tick = Tick {
//             timestamp: DateTime::<Utc>::from(Utc::now()),
//             price: 0.0,
//             };
//             let _ = update_market(market, tick, "YES", &tx).await;
//         };
//     }

// }

// pub async fn update_market(
//     market: MarketStandarized,
//     tick: Tick,
//     outcome: &str,
//     market_tx: &MarketUpdateSend,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     match outcome {
//         "YES" => {
//             market.unwrap().YES.push(tick.clone());
//         }
//         "NO" => {
//             market.pool.unwrap().NO.push(tick.clone());
//         }
//         _ => {}
//     }
//     let _ = market_tx
//         .send(MarketUpdateResult {
//             market_idx: market.idx,
//             tick,
//         })
//         .await;
//     Ok(())
// }
