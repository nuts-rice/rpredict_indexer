use super::Result;
use super::{Platform, PlatformBuilder};
use crate::model::manifold::ManifoldMarket;
use async_trait::async_trait;

pub struct ManifoldPlatform(PlatformBuilder<Self>);

impl From<PlatformBuilder<Self>> for ManifoldPlatform {
    fn from(value: PlatformBuilder<Self>) -> Self {
        Self(value)
    }
}

#[async_trait]
impl Platform for ManifoldPlatform {
    const ENDPOINT: &'static str = "https://api.manifold.markets/v0/markets";
    type Market = ManifoldMarket;

    async fn fetch_questions(&self) -> Result<Vec<crate::types::Question>> {
        let builder = &self.0;
        let url = builder.endpoint.as_str();
        let response = builder
            .client
            .get(url)
            .query(&["limit", builder.limit.to_string().as_str()])
            .send()
            .await?;
        unimplemented!()
    }
    async fn fetch_question_by_id(&self, id: &str) -> Result<crate::types::Question> {
        unimplemented!()
    }
}
