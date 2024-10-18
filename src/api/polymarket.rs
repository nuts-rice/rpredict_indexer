use super::Result;
use super::{Platform, PlatformBuilder};
use crate::polymarket::{PolymarketEvent, PolymarketMarket};
use async_trait::async_trait;
use serde_json::json;
//https://github.com/Polymarket/py-clob-client
pub struct PolymarketPlatform(PlatformBuilder<Self>);

impl From<PlatformBuilder<Self>> for PolymarketPlatform {
    fn from(value: PlatformBuilder<Self>) -> Self {
        Self(value)
    }
}

pub fn get_headers() -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        "application/json".parse().unwrap(),
    );
    headers
}

#[async_trait]
impl Platform for PolymarketPlatform {
    // const ENDPOINT: &'static str = "https://clob.polymarket.com/markets";
    const ENDPOINT: &'static str = "https://clob.polymarket.com";
    const SORT: &'static str = "order:";

    type Market = PolymarketMarket;
    type Event = PolymarketEvent;
    async fn fetch_questions(&self) -> Result<Vec<Self::Market>> {
        let builder = &self.0;
        let url = builder.endpoint.as_str().to_owned() + "/markets";
        let limit = builder.limit;
        let markets: Vec<Self::Market> = vec![];
        let response = builder
            .client
            .get(url.to_string())
            .headers(get_headers())
            // .query(&("limit", builder.limit.to_string().as_str()))
            // .query(&["limit", builder.limit.to])
            .send()
            .await?;
        // .json::<Vec<Self::Market>>()
        // .await?;
        let response_headers = response.headers().clone();
        let content_type = response_headers.get("content-type").unwrap();
        let markets_text = response.text().await.unwrap();
        println!("Markets text: {:?}", markets_text);
        let response_body = if markets_text.trim().is_empty() {
            None
        } else {
            let deserialized = if content_type == "application/json" {
                serde_json::from_str::<Vec<PolymarketMarket>>(&markets_text)
            } else {
                let json = json!(markets_text);
                serde_json::from_value::<Vec<PolymarketMarket>>(json)
            }
            .expect("Failed to parse JSON response");
            Some(deserialized)
        };
        println!("Response: {:?}", response_body.unwrap());

        println!("Markets: {:?}", markets);
        // println!("Markets: {:?}", markets_text);

        // // let markets = serde_json::from_str::<Vec<PolymarketMarket>>(&markets_text).unwrap();
        // // // let markets = parse_polymarket_text(&markets_text);
        // tracing::debug!("Markets: {:?}", markets_text);

        Ok(markets)
    }

    async fn fetch_question_by_id(&self, id: &str) -> Result<Self::Market> {
        unimplemented!()
    }
    async fn fetch_json(&self) -> Result<Vec<serde_json::Value>> {
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
    async fn fetch_markets_by_terms(&self, terms: &str) -> Result<Vec<Self::Market>> {
        unimplemented!()
    }

    async fn fetch_ratelimited(
        request_count: usize,
        interval_ms: Option<u64>,
    ) -> PlatformBuilder<Self> {
        unimplemented!()
    }

    async fn fetch_json_by_description(&self, description: &str) -> Result<Vec<serde_json::Value>> {
        unimplemented!()
    }
    async fn fetch_events(&self, limit: Option<u64>, offset: u64) -> Result<Vec<Self::Event>> {
        let offset = offset.to_string();
        let limit = limit.unwrap_or(30).to_string();
        let args: Vec<_> = [
            ("limit", limit.as_str()),
            ("active", "true"),
            ("archived", "false"),
            ("closed", "false"),
            ("order", "volume24hr"),
            ("ascending", "false"),
            ("offset", offset.as_str()),
        ]
        .iter()
        .map(|(arg, value)| (*arg, *value))
        .collect();
        let url = "https://gamma-api.polymarket.com/events";
        let builder = &self.0;
        let response = builder
            .client
            .get(url)
            .query(&args)
            .send()
            .await?
            .json::<Vec<Self::Event>>()
            .await?;
        Ok(response)
    }
    async fn fetch_orderbook(&self, id: &str) -> Result<Vec<serde_json::Value>> {
        unimplemented!()
    }
}

mod tests {
    use super::*;

    #[tokio::test]
    async fn test_polymarket_markets() {
        let platform = PolymarketPlatform::from(PlatformBuilder::default());
        let markets = platform.fetch_questions().await.unwrap();
        tracing::info!("Markets: {:?}", markets);
    }
    #[tokio::test]
    async fn test_polymarket_events() {
        let platform = PolymarketPlatform::from(PlatformBuilder::default());
        let events = platform.fetch_events(Some(5), 1).await.unwrap();
        println!("Events: {:?}", events);
    }
}
