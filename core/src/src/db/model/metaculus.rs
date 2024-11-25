use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub struct MetaculusResponse {
    pub results: Vec<MetaculusMarket>,
    pub next: Option<String>,
    pub previous: Option<String>,
    // outcomeType: OutcomeType,
    // pool: Option<BetPool>,
}
#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub struct MetaculusMarket {
    pub id: u32,
    pub title: Option<String>,
    pub created_at: Option<String>,
    pub scheduled_close_time: Option<String>,
    pub scheduled_resolve_time: Option<String>,

    // question: Question,
    pub nr_forecasters: u32,
    pub forecasts_count: u32,
    pub status: Option<Status>,
    pub forecast_type: Option<Type>,
    // type: Type,
    //
}

pub struct MetaculusPosition {}

#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub enum Status {
    approved,
    upcoming,
    closed,
    resolved,
    open,
    pending,
}

impl FromStr for Status {
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "approved" => Ok(Status::approved),
            "upcoming" => Ok(Status::upcoming),
            "closed" => Ok(Status::closed),
            "resolved" => Ok(Status::resolved),
            "open" => Ok(Status::open),
            "pending" => Ok(Status::pending),
            _ => Err("Invalid Status".into()),
        }
    }
}

#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub struct Question {
    aggregations: Aggregations,
}
#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub struct Aggregations {
    recently_weighted: u32,
    metaculus_prediction: u32,
}

#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub struct Possibilities {
    // type: Type,
}

#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub struct MetaculusEvent {}

#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub enum Active_State {}

#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub enum Type {
    binary,
    numeric,
    date,
    multiple_choice,
    conditional,
    group_of_questions,
    notebook,
}

impl FromStr for Type {
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "binary" => Ok(Type::binary),
            "numeric" => Ok(Type::numeric),
            "date" => Ok(Type::date),
            "multiple_choice" => Ok(Type::multiple_choice),
            "conditional" => Ok(Type::conditional),
            "group_of_questions" => Ok(Type::group_of_questions),
            "notebook" => Ok(Type::notebook),
            _ => Err("Invalid Type".into()),
        }
    }
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
        let id = results["id"].as_u64().unwrap() as u32;
        let title = results["title"].as_str().unwrap().to_string();
        let created_at = results["created_at"].as_str().unwrap().to_string();
        let scheduled_close_time = results["scheduled_close_time"]
            .as_str()
            .unwrap()
            .to_string();
        let scheduled_resolve_time = results["scheduled_resolve_time"]
            .as_str()
            .unwrap()
            .to_string();
        let nr_forecasters = results["nr_forecasters"].as_u64().unwrap() as u32;
        let forecasts_count = results["forecasts_count"].as_u64().unwrap() as u32;
        let status = Status::from_str(results["status"].as_str().unwrap()).unwrap();
        let forecast_type = Type::from_str(results["forecast_type"].as_str().unwrap()).unwrap();

        MetaculusMarket {
            id,
            title: Some(title),
            created_at: Some(created_at),
            scheduled_close_time: Some(scheduled_close_time),
            scheduled_resolve_time: Some(scheduled_resolve_time),
            nr_forecasters,
            forecasts_count,
            status: Some(status),
            forecast_type: Some(forecast_type),
        }
    }
}

impl fmt::Display for MetaculusMarket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
