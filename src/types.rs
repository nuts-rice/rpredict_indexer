use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
enum OutcomeType {
    BINARY,
    MULTIPLE_CHOICE,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Question {
    question: String,
    id: String,
    createdTime: i64,
    closeTime: i64,
    totalLiquidity: i32,
    outcomeType: OutcomeType,
    pool: Option<BetPool>,
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
#[derive(Deserialize, Debug, Serialize)]
pub struct BetPool {
    NO: f32,
    YES: f32,
}
