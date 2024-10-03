use serde::{Deserialize, Serialize};
use std::str::FromStr;
#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub struct MetaculusMarket {
    next: Option<String>,
    previous: Option<String>,
    results: Vec<Results>,
    // outcomeType: OutcomeType,
    // pool: Option<BetPool>,
}
#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub struct Results {
    title: String,
    title_short: String,
    created_time: String,
    close_time: String,
    // _type: Type,
}
#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub enum Active_State {}

#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub enum Type {
    forecast,
    notebook,
    discussion,
    claim,
    group,
    conditional_group,
    multiple_choice,
}

impl FromStr for MetaculusMarket {
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        unimplemented!()
    }
}

impl From<serde_json::Value> for MetaculusMarket {
    fn from(value: serde_json::Value) -> Self {
        let results = value["results"].clone();
        let next = value["next"].clone();
        let previous = value["previous"].clone();
        let results = serde_json::from_value(results).unwrap();
        MetaculusMarket {
            next: next.as_str().map(|s| s.to_string()),
            previous: previous.as_str().map(|s| s.to_string()),
            results,
        }
    }
}
