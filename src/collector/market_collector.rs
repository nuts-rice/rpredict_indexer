use std::sync::Arc;

pub struct MarketCollector<M> {
    provider: Arc<M>,
}
