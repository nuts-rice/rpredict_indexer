use jsonrpsee::http_client::HttpClient;
use reqwest::Client;
use std::sync::Arc;

use crate::types::Collector;

pub struct MarketCollector<M> {
    provider: Arc<M>,
}
