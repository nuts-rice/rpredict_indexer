use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
enum OutcomeType {
    BINARY,
    MULTIPLE_CHOICE,
    POLL,
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
#[derive(Deserialize, Debug, Serialize)]
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

#[derive(Deserialize, Debug, Serialize)]
pub enum StrategyType {
    ARBITRAGE,
    MARKETMAKING,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct StrategyConfig {
    id: String,
    strategy_type: StrategyType,
    enabled: bool,
    period: i32,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            strategy_type: StrategyType::ARBITRAGE,
            enabled: true,
            period: 60,
        }
    }
}
