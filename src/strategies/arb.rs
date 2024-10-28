use super::Platform;
use super::Strategy;
use super::StrategyBuilder;
use crate::api::PlatformBuilder;
use crate::strategies::Result;
use axum::async_trait;
use std::any::Any;
// pub struct ArbitrageStrategy(Strategy<Self>);

pub struct ArbitrageStrategy(StrategyBuilder);

impl From<StrategyBuilder> for ArbitrageStrategy {
    fn from(builder: StrategyBuilder) -> Self {
        Self(builder)
    }
}

#[async_trait]
impl<P: Platform + Any + std::marker::Sync> Strategy<P> for ArbitrageStrategy {
    const INTERVAL: i32 = 60;
    fn set_apis(&mut self, builder: PlatformBuilder<P>) {
        unimplemented!()
    }
    fn register_markets() {
        unimplemented!()
    }
    async fn run(&self) {
        unimplemented!()
    }
    fn builder() -> StrategyBuilder {
        unimplemented!()
    }
    async fn one_best(&self) -> Result<()> {
        unimplemented!()
    }
}

async fn find_arb(buy_balance: f32, sell_balance: f32, is_source_buying: bool) -> Result<f32> {
    unimplemented!()
}

#[cfg(test)]
mod tests {}
