use axum::{
    extract::{Path, Query, State},
};
use crate::db::model::question;
use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Default)]
pub struct Pagiation {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}


async fn markets_index(pagiation: Option<Query<Pagiation>>,  )

