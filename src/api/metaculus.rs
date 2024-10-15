use super::Result;
use super::{Platform, PlatformBuilder};
use crate::metaculus::MetaculusEvent;
use crate::model::metaculus::MetaculusMarket;
use async_trait::async_trait;
use axum::extract::Query;
pub struct MetaculusPlatform(PlatformBuilder<Self>);

impl From<PlatformBuilder<Self>> for MetaculusPlatform {
    fn from(value: PlatformBuilder<Self>) -> Self {
        Self(value)
    }
}

#[async_trait]
impl Platform for MetaculusPlatform {
    const ENDPOINT: &'static str = "https://www.metaculus.com/api2/questions/";
    const SORT: &'static str = "order:";

    type Market = MetaculusMarket;
    type Event = MetaculusEvent;
    async fn fetch_questions(&self) -> Result<Vec<Self::Market>> {
        let builder = &self.0;
        let url = builder.endpoint.as_str();
        let limit = builder.limit;
        let markets: Vec<Self::Market> = Vec::new();
        let response = builder
            .client
            .get(format!("{url}?limit={}", limit.to_string().as_str()))
            // .query(&("limit", builder.limit.to_string().as_str()))
            // .query(&["limit", builder.limit.to])
            .send()
            // .await?
            // .json::<Vec<Self::Market>>()
            .await;
        let market_text = response.unwrap().text().await;
        tracing::debug!("Response : {:?}", market_text);

        // let results = market_text.unwrap().parse::<Vec<Self::Market>>();
        // let markets: Vec<Self::Market> = serde_json::from_str(&text).unwrap();
        Ok(markets)
    }

    async fn fetch_question_by_id(&self, id: &str) -> Result<crate::types::Question> {
        unimplemented!()
    }
    async fn fetch_json(&self) -> Result<serde_json::Value> {
        let builder = &self.0;
        let url = builder.endpoint.as_str();
        let response = builder
            .client
            .get(url)
            .send()
            .await?
            .json()
            .await
            .expect("Failed to parse JSON response");
        Ok(response)
    }
    async fn build_order(&self, token: &str, amount: f64, nonce: &str) {
        unimplemented!()
    }
    async fn fetch_ratelimited(
        request_count: usize,
        interval_ms: Option<u64>,
    ) -> PlatformBuilder<Self> {
        unimplemented!()
    }
    async fn fetch_json_by_description(&self, description: &str) -> Result<serde_json::Value> {
        unimplemented!()
    }

    async fn fetch_events(
        pagiation: Option<Query<crate::api::index::Pagiation>>,
    ) -> Result<Vec<Self::Event>> {
        unimplemented!()
    }
}
