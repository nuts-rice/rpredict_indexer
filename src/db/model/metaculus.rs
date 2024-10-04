use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub struct MetaculusMarket {
    pub results: Vec<MetaculusResults>,
    // outcomeType: OutcomeType,
    // pool: Option<BetPool>,
}
#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub struct MetaculusResults {
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
        let results: Vec<MetaculusResults> = serde_json::from_value(results).unwrap();
        MetaculusMarket { results }
    }
}

impl fmt::Display for MetaculusMarket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for MetaculusResults {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
