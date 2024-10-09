use crate::{
    api::{Platform, PlatformBuilder},
    types::StrategyConfig,
};
use axum::async_trait;
use std::any::Any;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub mod arb;
pub struct StrategyBuilder {
    config: StrategyConfig,
    // api: PlatformBuilder<T>,
    // marker: std::marker::PhantomData<T>,
}

#[async_trait]
pub trait Strategy: From<StrategyBuilder> + Any {
    const INTERVAL: i32;
    fn builder() -> StrategyBuilder {
        StrategyBuilder::new()
    }
    fn set_apis<T: Platform>(&mut self, api: PlatformBuilder<T>);
    fn register_markets();
    async fn run(&self);
}

impl StrategyBuilder {
    pub fn new() -> Self {
        StrategyBuilder {
            config: StrategyConfig::default(),
            // api: PlatformBuilder::new(),
            // marker: std::marker::PhantomData,
        }
    }
    pub fn config(mut self, config: StrategyConfig) -> Self {
        self.config = config;
        self
    }
    // pub fn api<P>(mut self, api: PlatformBuilder<P>) -> Self {
    //     self.api = api;
    //     self
    // }
    // pub fn build(self) -> Box<dyn Strategy> {
    //     let mut strategy = Strategy::from(self);
    //     strategy.set_apis(self.api);
    //     strategy.register_markets();
    //     Box::new(strategy)
    // }
}
