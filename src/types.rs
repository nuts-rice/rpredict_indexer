use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
enum OutcomeType {
    BINARY,
}

#[derive(Deserialize, Debug)]
pub struct Question {
    question: String,
    id: String,
    createdTime: i64,
    closeTime: i64,
    totalLiquidity: i32,
    outcomeType: OutcomeType,
}
pub struct Indicators {
    num_forecasts: i32,
    num_forecasters: i32,
    spread: f32,
    shares_volume: f32,
    likes: i32,
    votes: i32,
    stars: i32,
}
