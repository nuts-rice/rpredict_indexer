use serde::Deserialize;
#[derive(Debug, Deserialize, Default)]
pub struct Pagiation {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

// async fn markets_index(pagiation: Option<Query<Pagiation>>, State(db): State<Db>) -> impl IntoResponse {
//     let pagiation = pagiation.unwrap_or_default();
//     let offset = pagiation.offset.unwrap_or(0).to_be_bytes();
//     let limit = pagiation.limit.unwrap_or(20).to_be_bytes();
//     let markets = db.range(offset..limit);
//     markets.collect::<Vec<_>>()

// }
