use crate::api::{manifold::manifold_api::ManifoldPlatform, PlatformBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Outcome {
    #[serde(rename = "YES")]
    Yes,
    #[serde(rename = "NO")]
    No,
    FreeResponse(String),
    Numeric(String, f64),
}

#[derive(Deserialize)]
pub struct BetParams {
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    pub username: Option<String>,
    #[serde(rename = "contractId")]
    pub contract_id: Option<String>,
    #[serde(rename = "contractSlug")]
    pub contract_slug: Option<String>,
    pub limit: Option<f64>,
    pub amount: Option<f64>,
}

pub async fn prep_order(platform: ManifoldPlatform) {
    // let builder = &platform
    unimplemented!()
}
