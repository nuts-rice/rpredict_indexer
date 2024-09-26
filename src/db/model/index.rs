use crate::Db;
use axum::extract::{Query, State};
use serde::Deserialize;
#[derive(Debug, Deserialize, Default)]
pub struct Pagiation {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

async fn markets_index(pagiation: Option<Query<Pagiation>>, State(db): State<Db>) {}
