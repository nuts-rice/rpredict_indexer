use std::sync::Arc;

use axum::{
    extract::Query,
    response::{Html, IntoResponse, Json},
    Extension,
};

use crate::{MarketsDb, Pagiation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Component {
    pub html: String,
}

async fn index(
    Pagiation: Option<Query<Pagiation>>,
    sort: Option<Query<String>>,
    state: Extension<Arc<MarketsDb>>,
) -> impl IntoResponse {
    let pagiation = Pagiation.unwrap_or_default();
    let sort = sort.unwrap_or_default();
    let page: Component = Component {
        html: String::new(),
    };

    Html(page.html)
}
