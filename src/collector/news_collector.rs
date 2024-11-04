use reqwest::Client;
use std::sync::Arc;
pub struct NewsCollector<M> {
    provider: Arc<M>,
}

impl<M> NewsCollector<M> {
    pub fn new(provider: Arc<M>) -> Self {
        NewsCollector { provider }
    }
}
