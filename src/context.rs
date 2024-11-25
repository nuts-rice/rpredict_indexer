use crate::{
    api::{self, manifold::manifold_api::ManifoldPlatform, PlatformBuilder},
    manifold::ManifoldMarket,
    types::StrategyConfig,
};
use chrono::{DateTime, Utc};
use qdrant_client::{config::QdrantConfig, Qdrant};
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> Self {
        Self {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

pub struct Context
//<'a>
{
    pub id: String,
    pub strategy_config: Arc<RwLock<StrategyConfig>>,
    pub manifold: Arc<RwLock<ManifoldPlatform>>,
    //pub questions: //Arc<RwLock<Vec<serde_json::Value>>>,
    pub questions: Vec<serde_json::Value>,
    //TODO: cross platforms
    //pub questions: Vec<MarketStandarized>,
    // pub selecteable_markets: StatefulList<&'a str>,
    pub exit: bool,
}

impl Context
//<'a>
{
    pub fn new() -> Self {
        let manifold = ManifoldPlatform::from(PlatformBuilder::default());
        let strategy_config = StrategyConfig::default();
        let selecteable_markets =
            StatefulList::with_items(vec!["Market 1", "Market 2", "Market 3"]);
        Self {
            manifold: Arc::new(RwLock::new(manifold)),
            id: "default".to_string(),
            strategy_config: Arc::new(RwLock::new(strategy_config)),
            questions: vec![],
            exit: false,
        }
    }
}
