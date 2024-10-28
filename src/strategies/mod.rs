use crate::context::Context;
use crate::{
    api::{Platform, PlatformBuilder},
    types::StrategyConfig,
};
use axum::async_trait;
use std::any::Any;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub mod arb;
pub struct StrategyBuilder {
    pub ctx: Context,
    // api: PlatformBuilder<T>,
    // marker: std::marker::PhantomData<T>,
}

#[async_trait]
pub trait Strategy<P: Platform + std::marker::Sync>: From<StrategyBuilder> + Any {
    const INTERVAL: i32;
    fn builder() -> StrategyBuilder {
        StrategyBuilder::new()
    }
    fn set_apis(&mut self, api: PlatformBuilder<P>);
    fn register_markets();
    async fn run(&self);
    async fn one_best(&self) -> Result<()>;
}

impl Default for StrategyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl StrategyBuilder {
    pub fn new() -> Self {
        let context = Context::new();
        Self { ctx: context }
    }
    pub fn strat_config(mut self, config: StrategyConfig) -> Result<()> {
        let mut strat_config = self.ctx.strategy_config.write().unwrap();
        *strat_config = config;
        Ok(())
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
