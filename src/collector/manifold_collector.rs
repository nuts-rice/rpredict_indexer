use crate::db::model::manifold::ManifoldMarket;
use crate::manifold::ManifoldPosition;
use crate::types::{Collector, CollectorStream};
use anyhow::Result;
use async_trait::async_trait;
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
    fn collect(&self) -> Result<CollectorStream<'_, ManifoldMarket>> {
        unimplemented!()
    }
}

impl Collector<ManifoldPosition> for ManifoldCollector {
    fn collect(&self) -> Result<CollectorStream<'_, ManifoldPosition>> {
        unimplemented!()
    }
}
