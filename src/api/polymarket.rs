use super::Result;
use super::{Platform, PlatformBuilder};
use crate::polymarket::PolymarketMarket;
use async_trait::async_trait;

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
    const ENDPOINT: &'static str = "https://clob.polymarket.com/sampling-simplified-markets";

    type Market = PolymarketMarket;

    async fn fetch_questions(&self) -> Result<Vec<Self::Market>> {
        let builder = &self.0;
        let url = builder.endpoint.as_str();
        let limit = builder.limit;
        let markets: Vec<Self::Market> = vec![];
        let response = builder
            .client
            .get(url.to_string())
            .headers(get_headers())
            // .query(&("limit", builder.limit.to_string().as_str()))
            // .query(&["limit", builder.limit.to])
            .send()
            // .await?
            // .json::<Vec<Self::Market>>()
            .await?;

        let markets_text = response.text().await.unwrap();
        // let markets = parse_polymarket_text(&markets_text);
        tracing::info!("Market: {:?}", markets_text);

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
}
