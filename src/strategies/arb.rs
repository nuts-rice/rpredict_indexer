use super::Platform;
use super::Strategy;
use super::StrategyBuilder;
use crate::api::PlatformBuilder;
use crate::strategies::Result;
use axum::async_trait;
// pub struct ArbitrageStrategy(Strategy<Self>);

pub struct ArbitrageStrategy(StrategyBuilder);

impl From<StrategyBuilder> for ArbitrageStrategy {
    fn from(builder: StrategyBuilder) -> Self {
        Self(builder)
    }
}

#[async_trait]
impl Strategy for ArbitrageStrategy {
    const INTERVAL: i32 = 60;
    fn set_apis<T: Platform>(&mut self, api: PlatformBuilder<T>) {
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
}

async fn find_arb(buy_balance: f32, sell_balance: f32, is_source_buying: bool) -> Result<f32> {
    unimplemented!()
}
