use core::fmt;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
enum OutcomeType {
    BINARY,
    MULTIPLE_CHOICE,
    POLL,
}

#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub struct ManifoldMarket {
    question: String,
    id: String,
    createdTime: i64,
    closeTime: i64,
    totalLiquidity: Option<f32>,
    outcomeType: OutcomeType,
    pool: Option<BetPool>,
}

// #[derive(Deserialize, Debug, Serialize)]
// pub struct Indicators {
//     num_forecasts: i32,
//     num_forecasters: i32,
//     spread: f32,
//     shares_volume: f32,
//     likes: i32,
//     votes: i32,
//     stars: i32,
// }
#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub struct BetPool {
    NO: f32,
    YES: f32,
}

impl fmt::Display for ManifoldMarket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.outcomeType == OutcomeType::BINARY {
            let question = self.question.to_string();
            let pool = self.pool.as_ref().unwrap();
            let total_pool = pool.NO + pool.YES;
            let yes_share = pool.YES / total_pool;
            let no_share = pool.NO / total_pool;
            if yes_share >= 0.8 {
                write!(f, "{}...YES: {} Pretty Certain", question, yes_share)
            } else if yes_share >= 0.6 {
                write!(f, "{}...YES: {} Likely", question, yes_share)
            } else if yes_share >= 0.4 {
                write!(f, "{}...YES: {} Maybe", question, yes_share)
            } else if yes_share >= 0.2 {
                write!(f, "{}...YES: {} Unlikely", question, yes_share)
            } else {
                write!(f, "{}...YES: {} Very unlikely", question, yes_share)
            }
        } else {
            let question = self.question.to_string();
            write!(f, "{}...", question,)
        }
    }
}
