use super::Result;
use super::{Platform, PlatformBuilder};
use crate::manifold::ManifoldEvent;
use crate::model::manifold::ManifoldMarket;
use crate::model::manifold::ManifoldPosition;

use async_trait::async_trait;
pub struct ManifoldPlatform(PlatformBuilder<Self>);

//TODO: use this to grab tags
const GROUP_URL: &str = "https://api.manifold.markets/v0/groups";
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
    async fn incoming_market_to_value(&self, market: Self::Market) -> Result<serde_json::Value> {
        let probability: String = if let Some(probability) = market.probability {
            probability.to_string()
        } else {
            "".to_string()
        };
        let pool: [String; 2] = if let Some(pool) = market.pool {
            [format!("Yes: {}", pool.YES), format!("No: {}", pool.NO)]
        } else {
            ["0".to_string(), "0".to_string()]
        };
        let market_summarized = serde_json::json!({
            "question": market.question,
            "probability": probability,
            "pool": pool,
        });
        Ok(market_summarized)
    }

    async fn incoming_position_to_value(
        &self,
        position: Self::Position,
    ) -> Result<serde_json::Value> {
        unimplemented!()
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
        contract_id: &str,
        amount: f64,
        nonce: &str,
        outcome: &str,
    ) -> Result<()> {
        let builder = &self.0;
        let url = "https://api.manifold.markets/v0/bet".to_string();
        let key: String = std::env::var("MANIFOLD_API_KEY").unwrap();
        let prepped_order = serde_json::json!(
            {
            "contractId": contract_id,
            "amount": amount,
            "outcome": outcome
            }
        );

        let response = builder
            .client
            .post(url)
            .header("Authorization: Key {}", key)
            .json(&prepped_order)
            .send()
            .await?;
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
}

mod tests {

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
        let bet = manifold.build_order("9Ccsjc0fmbIb9g50p7SB", 10., "", "YES");
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
