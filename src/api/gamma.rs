use super::Result;
use super::{Platform, PlatformBuilder};
use crate::gamma::{GammaMarket, GammaPosition};
use async_trait::async_trait;
use std::collections::HashMap;

//https://github.com/Polymarket/py-clob-client
pub struct GammaPlatform(PlatformBuilder<Self>);

impl From<PlatformBuilder<Self>> for GammaPlatform {
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
impl Platform for GammaPlatform {
    // const ENDPOINT: &'static str = "https://clob.polymarket.com/markets";
    const ENDPOINT: &'static str = "https://gamma-api.polymarket.com/";
    const SORT: &'static str = "order:";

    type Market = GammaMarket;
    type Event = crate::db::gamma::GammaEvent;
    type Position = GammaPosition;
    async fn fetch_questions(&self) -> Result<Vec<Self::Market>> {
        let builder = &self.0;
        let url = builder.endpoint.to_owned() + "/markets";

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

    async fn fetch_question_by_id(&self, token_id: &str) -> Result<Self::Market> {
        let mut args = HashMap::new();
        args.insert("token_id", token_id);
        let builder = &self.0;
        let url = builder.endpoint.as_str().to_owned() + "/" + token_id;
        let market: Self::Market = GammaMarket {};

        let response = builder
            .client
            .get(url)
            // .query(&args)
            .send()
            .await?;
        // .json::<Self::Market>()
        // .await?;
        let market_text = response.text().await.unwrap();
        tracing::info!("Market: {:?}", market_text);

        Ok(market)
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

    async fn build_order(
        &self,
        token: &str,
        amount: f64,
        nonce: &str,
        outcome: &str,
        limit: Option<f64>,
    ) -> Result<()> {
        unimplemented!()
    }

    async fn fetch_markets_by_terms(&self, term: &str) -> Result<Vec<Self::Market>> {
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
        unimplemented!()
    }
    async fn fetch_orderbook(&self, id: &str) -> Result<Vec<Self::Position>> {
        unimplemented!()
    }
    async fn subscribe_to(&self) -> Result<()> {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    // #[tokio::test]
    // async fn test_fetch_questions() {
    //     let platform = GammaPlatform::from(PlatformBuilder::default());
    //     let questions = platform.fetch_json().await.unwrap();
    //     println!("Questions: {:?}", questions);

    //     // assert!(questions.len() > 0);
    // }
    #[tokio::test]
    async fn test_fetch_question_by_id() {
        let platform = GammaPlatform::from(PlatformBuilder::default());
        let question = platform.fetch_question_by_id("506962").await.unwrap();
        println!("Question: {:?}", question);
        // assert!(question.len() > 0);
    }
    #[tokio::test]
    async fn test_fetch_events_by_tag() {
        unimplemented!()
    }
}
