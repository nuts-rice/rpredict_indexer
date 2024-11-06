use crate::api::manifold::ManifoldPlatform;
use crate::api::{Platform, PlatformBuilder};
use crate::db::model::manifold::ManifoldMarket;
use crate::manifold::ManifoldPosition;
use crate::types::{Collector, CollectorStream, MarketStandarized, Tick};
use anyhow::Result;
use async_trait::async_trait;
use tokio_stream::wrappers::BroadcastStream;

enum ManifoldRequest {
    Market,
    Position,
}

enum ManifoldResponse {
    Market(ManifoldMarket),
    Position(ManifoldPosition),
}
pub struct ManifoldCollector {
    pub key: String,
}

impl ManifoldCollector {
    pub fn new(key: String) -> ManifoldCollector {
        Self { key }
    }
}

#[async_trait]
impl Collector<ManifoldMarket> for ManifoldCollector {
    async fn collect_all(&self) -> Result<Vec<MarketStandarized>> {
        let mut platform = ManifoldPlatform::from(PlatformBuilder::default());
        let markets = platform.fetch_questions().await;
        // let stream = BroadcastStream::new(Box::new(markets));
        unimplemented!()
    }

    async fn collect_by_id(&self, id: &str) -> Result<MarketStandarized> {
        let client = ManifoldPlatform::from(PlatformBuilder::default());
        let market: ManifoldMarket = client.fetch_question_by_id(id).await.unwrap();
        unimplemented!()
    }
}

// impl Collector<Vec<Tick>> for ManifoldCollector {
//     async fn collect_stream(&self) -> Result<Vec<Tick>> {
//         unimplemented!()
//     }
// }
