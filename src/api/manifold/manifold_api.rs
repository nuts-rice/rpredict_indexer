use crate::api::base_request;
use crate::api::{Platform, PlatformBuilder, Result};
use crate::db;
use crate::manifold::{ExtraInfo, ManifoldEvent};
use crate::model::manifold::ManifoldPosition;
use crate::model::manifold::{Bet, ManifoldMarket};
use crate::types::Tick;

use async_trait::async_trait;
use reqwest_middleware::ClientWithMiddleware;
pub struct ManifoldPlatform(PlatformBuilder<Self>);

//TODO: use this to grab tags
const GROUP_URL: &'static str = "https://api.manifold.markets/v0/groups";
impl From<PlatformBuilder<Self>> for ManifoldPlatform {
    fn from(value: PlatformBuilder<Self>) -> Self {
        Self(value)
    }
}

#[async_trait]
impl Platform for ManifoldPlatform {
    const ENDPOINT: &'static str = "https://api.manifold.markets/v0/markets";
    const SORT: &'static str = "order:";
    type Market = ManifoldMarket;
    type Event = ManifoldEvent;
    type Position = ManifoldPosition;
    async fn fetch_questions(&self) -> Result<Vec<Self::Market>> {
        let builder = &self.0;
        let url = builder.endpoint.as_str();
        let limit = builder.limit;
        let response = builder
            .client
            .get(format!("{url}?limit={}", limit.to_string().as_str()))
            // .query(&("limit", builder.limit.to_string().as_str()))
            // .query(&["limit", builder.limit.to])
            .send()
            .await?;

        let text = response.text().await?;
        let markets: Vec<Self::Market> = serde_json::from_str(&text).unwrap();
        Ok(markets)
    }
    async fn fetch_markets_by_terms(&self, terms: &str) -> Result<Vec<Self::Market>> {
        let builder = &self.0;
        let url = "https://api.manifold.markets/v0/search-markets";
        let response = builder
            .client
            .get(format!("{url}?term={terms}", terms = terms))
            .send()
            // .await?
            // .json::<Vec<Self::Market>>()
            .await?;
        let text = response.text().await?;
        let markets: Vec<Self::Market> = serde_json::from_str(&text).unwrap();

        Ok(markets)
    }

    async fn fetch_question_by_id(&self, id: &str) -> Result<Self::Market> {
        let builder = &self.0;
        let url = format!("https://api.manifold.markets/v0/market/{}", id);
        tracing::debug!("URL: {:?}", url);
        let response = builder
            .client
            .get(url)
            .send()
            .await?
            .json::<Self::Market>()
            .await?;
        Ok(response)
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

    async fn get_user_id(&self) -> Result<String> {
        unimplemented!()
    }
    async fn build_order(
        &self,
        contract_id: &str,
        amount: f64,
        nonce: &str,
        outcome: &str,
        limit: Option<f64>,
    ) -> Result<()> {
        let builder = &self.0;
        let url = "https://api.manifold.markets/v0/bet";
        let key: String = std::env::var("MANIFOLD_API_KEY").unwrap();
        let mut prepped_order = serde_json::json!(
            {
            "amount": amount,
            "contractId": contract_id,
            "outcome": outcome,
            "dryRun": true,
            }

        );
        let body = prepped_order.as_object_mut().unwrap();
        if limit.is_some() {
            body.insert("limitProb".to_string(), serde_json::json!(limit.unwrap()));
        }
        tracing::debug!("Prepped Order: {:?}", body.clone());
        //TMI: it took me a whole hour to figure this when Postman solved this in seconds
        let mut headers = reqwest::header::HeaderMap::new();
        let auth_header = format!("Key {}", key);
        headers.insert("Content-Type", "application/json".parse()?);
        headers.insert("Authorization", auth_header.parse()?);
        let response = builder
            .client
            .post(url)
            .headers(headers)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        tracing::debug!("Bet: {:?}", response);
        Ok(())
    }
    async fn fetch_ratelimited(
        request_count: usize,
        interval_ms: Option<u64>,
    ) -> PlatformBuilder<Self> {
        unimplemented!()
    }
    async fn fetch_json_by_description(&self, term: &str) -> Result<Vec<serde_json::Value>> {
        let builder = &self.0;
        let url = format!(
            "https://api.manifold.markets/v0/search-markets?term={}&filter=open",
            term
        );
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
    async fn fetch_events(&self, limit: Option<u64>, offset: u64) -> Result<Vec<Self::Event>> {
        unimplemented!()
    }
    async fn fetch_orderbook(&self, id: &str) -> Result<Vec<Self::Position>> {
        let builder = &self.0;
        let url = format!("https://api.manifold.markets/v0/market/{}/positions", id);
        tracing::debug!("URL: {:?}", url);
        let response = builder
            .client
            .get(url)
            .send()
            .await?
            .json::<Vec<Self::Position>>()
            .await?;
        // .json::<Vec<serde_json::Value>>()
        // .await?;

        // .await
        // .expect("Failed to parse JSON response");
        // let positions_text = response.text().await?;

        Ok(response)
    }
    async fn subscribe_to(&self) -> Result<()> {
        unimplemented!()
    }
}

async fn probability_series(id: &str) -> Vec<(u64, f64)> {
    use crate::api::manifold;
    let times: Vec<u64> = vec![];
    let probabilities: Vec<f64> = vec![];
    let platform = ManifoldPlatform::from(PlatformBuilder::default());
    let mut trimmed_markets: Vec<serde_json::Value> = Vec::new();
    let bets = platform.fetch_orderbook(id).await.unwrap();

    if bets.len() == 0 {
        return vec![];
    }
    let market = platform.fetch_question_by_id(id).await.unwrap();
    let trimmed_market = parse_manifold_market(market).unwrap();

    unimplemented!()
}
async fn get_extra_data(
    client: &ClientWithMiddleware,
    market: ManifoldMarket,
) -> Result<ManifoldMarket> {
    unimplemented!()
    // let mut before: Option<String> = None;
    // let mut full_bets: Vec<Bet> = Vec::new();
    // let url = "https://api.manifold.markets/v0/bets";
    // loop {
    //     let bets: Vec<Bet> = base_request(client.get(format!(
    //         "{url}?contractId={}&limit=100&before={:?}",
    //         market.id, &before
    //     )))
    //     .await
    //     .unwrap();
    //     if bets.len() == 100 {
    //         full_bets.extend(bets);
    //         before = Some(full_bets.last().unwrap().id.to_string());
    //     } else {
    //         full_bets.extend(bets);
    //         break;
    //     }
    // }
    // let extra_info: ExtraInfo = base_request(client.get(format!(
    //     "https://api.manifold.markets/v0/market/{}",
    //     market.id
    // )))
    // .await
    // .unwrap();
    // Ok(ManifoldMarket {

    //     market: market.clone(),
    //     bets: full_bets.clone(),
    //     ticks: get_ticks(full_bets.clone()).unwrap(),
    //     extraInfo: extra_info,
    // })
}
fn is_valid(market: ManifoldMarket) -> bool {
    market.isResolved
        && market.mechanism == "cpmm-1"
        && market.outcomeType == "BINARY"
        && market.volume > 0.0
        && market.resolution != Some("CANCEL".to_string())
}

fn get_ticks(mut bets: Vec<Bet>) -> Result<Vec<Tick>> {
    unimplemented!()
}

fn parse_manifold_market(market: ManifoldMarket) -> Result<serde_json::Value> {
    let probability: String = if let Some(probability) = market.probability {
        probability.to_string()
    } else {
        "".to_string()
    };
    // let pool: [String; 2] = if let Some(pool) = market.pool {
    //     [
    //         format!("Yes: {}", pool.YES.to_string()),
    //         format!("No: {}", pool.NO.to_string()),
    //     ]
    // } else {
    //     ["0".to_string(), "0".to_string()]
    // };
    let market_summarized = serde_json::json!({
        "question": market.question,
        "probability": probability,
        // "pool": pool,
    });
    Ok(market_summarized)
}

mod tests {
    use super::*;
    use tracing_subscriber::prelude::*;
    #[tokio::test]
    async fn test_manifold_markets() {
        // tracing_subscriber::registry()
        //     .with(
        //         tracing_subscriber::EnvFilter::try_from_default_env()
        //             .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        //     )
        //     .with(tracing_subscriber::fmt::layer())
        //     .init();

        let mut manifold = ManifoldPlatform::from(PlatformBuilder::new());
        manifold.0.limit(15);
        let questions = manifold.fetch_questions().await.unwrap();
        tracing::debug!("Questions: {:?}", questions);
    }
    #[tokio::test]
    async fn test_manifold_search() {
        // tracing_subscriber::registry()
        //     .with(
        //         tracing_subscriber::EnvFilter::try_from_default_env()
        //             .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        //     )
        //     .with(tracing_subscriber::fmt::layer())
        //     .init();
        let mut manifold = ManifoldPlatform::from(PlatformBuilder::new());
        manifold.0.limit(15);
        let questions = manifold.fetch_json_by_description("crispr").await.unwrap();
        tracing::debug!("Questions: {:?}", questions);
    }
    #[tokio::test]
    async fn test_manifold_search_markets() {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
        let mut manifold = ManifoldPlatform::from(PlatformBuilder::new());
        manifold.0.limit(15);
        let questions = manifold.fetch_markets_by_terms("crispr").await.unwrap();
        tracing::debug!("Questions: {:?}", questions);
    }
    #[tokio::test]
    async fn test_build_order() {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
        let mut manifold = ManifoldPlatform::from(PlatformBuilder::new());
        let bet = manifold
            .build_order("9Ccsjc0fmbIb9g50p7SB", 1., "", "YES", None)
            .await;
        tracing::debug!("Bet: {:?}", bet);
        // let bets = manifold
        //     .fetch_orderbook("9Ccsjc0fmbIb9g50p7SB")
        //     .await
        //     .unwrap();
        // let trimmed_bets: Vec<_> = bets.iter().take(5).collect();
        // tracing::debug!("Orderbook for 'Will the 10 Year Treasury Yield at closing on 12/31/2024 be 4% or higher?': {:?}", trimmed_bets);
    }

    #[tokio::test]
    async fn test_manifold_search_bets() {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
        let mut manifold = ManifoldPlatform::from(PlatformBuilder::new());
        manifold.0.limit(15);
        let bets = manifold
            .fetch_orderbook("9Ccsjc0fmbIb9g50p7SB")
            .await
            .unwrap();
        let trimmed_bets: Vec<_> = bets.iter().take(5).collect();
        tracing::debug!("Orderbook for 'Will the 10 Year Treasury Yield at closing on 12/31/2024 be 4% or higher?': {:?}", trimmed_bets);
    }
}
