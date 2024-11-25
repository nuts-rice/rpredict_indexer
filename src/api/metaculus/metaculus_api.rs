use crate::api::{Platform, PlatformBuilder, Result};

use crate::model::metaculus::{
    MetaculusEvent, MetaculusMarket, MetaculusPosition, MetaculusResponse,
};
use serde::{Deserialize, Serialize};

use async_trait::async_trait;
pub struct MetaculusPlatform(PlatformBuilder<Self>);

impl From<PlatformBuilder<Self>> for MetaculusPlatform {
    fn from(value: PlatformBuilder<Self>) -> Self {
        Self(value)
    }
}

#[async_trait]
impl Platform for MetaculusPlatform {
    const ENDPOINT: &'static str = "https://www.metaculus.com/api/posts/";
    const SORT: &'static str = "order:";

    type Market = MetaculusMarket;
    type Event = MetaculusEvent;
    type Position = MetaculusPosition;
    async fn fetch_questions(&self) -> Result<Vec<Self::Market>> {
        let token: String = std::env::var("METACULUS_TOKEN").expect("METACULUS_TOKEN not set");
        let web_token = format!("Token {}", token);
        let builder = &self.0;
        let url = builder.endpoint.as_str();
        let limit = builder.limit;
        let response = builder
            .client
            .get(url)
            // .header("Authorization", web_token)
            // .query(&("limit", limit.to_string().as_str()))
            .send()
            .await?
            .json::<MetaculusResponse>()
            .await?;
        tracing::debug!("response: {:?}", response);
        let mut questions = Vec::new();
        questions.extend(response.results.into_iter());
        tracing::debug!("questions: {:?}", questions);
        // let mut next = response.next;
        //  while let Some(next_url) = next {
        //     tracing::debug!("next_url: {:?}", next_url);
        //     let response: MetaculusResponse = builder
        //         .client
        //         .get(next_url.as_str())
        //         // .header("Authorization", web_token.clone())
        //         .send()
        //         .await?
        //         .json()
        //         .await?;
        //     questions.extend(response.results.into_iter());
        //     next = response.next;
        //  }
        Ok(questions)
    }

    async fn fetch_json_by_description(&self, description: &str) -> Result<Vec<serde_json::Value>> {
        let builder = &self.0;
        let url = format!("{}?categories={}", builder.endpoint.as_str(), description);
        let limit = builder.limit;
        let response = builder
            .client
            .get(url)
            // .header("Authorization", web_token)
            // .query(&("limit", limit.to_string().as_str()))
            .send()
            .await?
            .json()
            .await
            .expect("Failed to parse JSON response");
        Ok(response)

        // tracing::debug!("response: {:?}", response);
        // let mut questions: Vec<serde_json::Value> = Vec::new();
        // questions.extend(response.results.into_iter());
        // tracing::debug!("questions: {:?}", questions);
        // Ok(questions)
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

    async fn get_user_id(&self, ) -> Result<String> {
        unimplemented!()
    }

    async fn fetch_ratelimited(
        request_count: usize,
        interval_ms: Option<u64>,
    ) -> PlatformBuilder<Self> {
        unimplemented!()
    }
    async fn fetch_events(&self, limit: Option<u64>, offset: u64) -> Result<Vec<Self::Event>> {
        unimplemented!()
    }

    async fn fetch_orderbook(&self, id: &str) -> Result<Vec<Self::Position>> {
        unimplemented!()
    }
    async fn fetch_markets_by_terms(&self, term: &str) -> Result<Vec<Self::Market>> {
        let builder = &self.0;
        let url = format!("{}?categories={}", builder.endpoint.as_str(), term);
        let response = builder
            .client
            .get(url)
            // .header("Authorization", web_token)
            // // .query(&("limit", limit.to_string().as_str()))
            .send()
            .await?
            .json::<MetaculusResponse>()
            .await?;
        tracing::debug!("response: {:?}", response);
        let mut questions = Vec::new();
        questions.extend(response.results.into_iter());
        tracing::debug!("questions: {:?}", questions);
        Ok(questions)
    }
    async fn subscribe_to(&self) -> Result<()> {
        unimplemented!()
    }
}


async fn post_forecast(question_id: u32, probability_yes: f64, probability_yes_per_catagory: Option<Vec<f64>>) -> Result<()> {
    let client = reqwest::Client::builder()
        .build()?;
    let url = format!("https://www.metaculus.com/api/questions/forecast");
    let token: String = std::env::var("METACULUS_API_KEY").unwrap();
    let mut headers = reqwest::header::HeaderMap::new();
    let auth_header = format!("Token {}", token);
    headers.insert("Authorization", auth_header.parse()?);
    // headers.insert("Content-Type", "application/json".parse()?);

    let mut prepped_body = serde_json::json!({
        "question": question_id,               
        "prediction": probability_yes
    });
    let body = prepped_body.as_object_mut().unwrap();
    tracing::debug!("Prepped Order: {:?}", body.clone());
    let request = client
        .post(url)
        .headers(headers)
        .json(&body)
        .send()
        .await?
        .error_for_status()?;
    tracing::debug!("Response: {:?}", request);
    Ok(())

}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber::prelude::*;
    #[tokio::test]
    async fn test_metaculus_fetch_questions() {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();

        let platform = MetaculusPlatform::from(PlatformBuilder::new());
        let questions = platform.fetch_questions().await.unwrap();

        assert!(!questions.is_empty());
    }

    #[tokio::test]
    async fn test_metaculus_post_forecast() {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
        post_forecast(489, 0.6, None).await.unwrap();
    }

}
