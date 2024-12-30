use crate::{
    api::{manifold::manifold_api::ManifoldPlatform, PlatformBuilder, Platform},
    types::StrategyConfig,
};
use ratatui::widgets::ListState;
use sled::Db;
use std::{str::FromStr, sync::{Arc, RwLock}};

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
    // pub manifold: Arc<RwLock<ManifoldPlatform>>,
    // pub market_platform: Arc<RwLock<Platform>>, 
    //pub questions: //Arc<RwLock<Vec<serde_json::Value>>>,
    pub questions_db: Arc<RwLock<Db>>,
    pub selected_market: Option<SelectedMarket>,
    //TODO: cross platforms
    //pub questions: Vec<MarketStandarized>,
    // pub selecteable_markets: StatefulList<&'a str>,
    pub exit: bool,
}

enum SelectedMarket {
    MANIFOLD,
    METACULUS,
    POLYMARKET,
    AUGUR,
}

impl FromStr for SelectedMarket {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MANIFOLD" => Ok(Self::MANIFOLD),
            "METACULUS" => Ok(Self::METACULUS),
            "POLYMARKET" => Ok(Self::POLYMARKET),
            "AUGUR" => Ok(Self::AUGUR),
            _ => Err(()),
        }
    }
}



impl Default for Context {
    fn default() -> Self {
        let strategy_config = StrategyConfig::default();
        Self {
            // manifold: Arc::new(RwLock::new(manifold)),
            selected_market: None,
            id: "default".to_string(),
            strategy_config: Arc::new(RwLock::new(strategy_config)),
            questions_db: Arc::new(RwLock::new(sled::open("questions_db").unwrap()), 
            exit: false,
        }
    }
}

impl Context
//<'a>
{
    pub fn new() -> Self {
        let manifold = ManifoldPlatform::from(PlatformBuilder::default());
        let strategy_config = StrategyConfig::default();
        Self {
            manifold: Arc::new(RwLock::new(manifold)),
            selected_market: None,
            id: "default".to_string(),
            strategy_config: Arc::new(RwLock::new(strategy_config)),
            questions: vec![],
            exit: false,
        }
    }

    pub fn set_selected_market(&mut self, selected_market: &str) {
        self.selected_market = SelectedMarket::from_str(selected_market).unwrap();
    }

}
