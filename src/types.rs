use clap::{Arg, ArgMatches, Command};
use qdrant_client::{config::QdrantConfig, Qdrant};
use serde::{Deserialize, Serialize};
use std::fs;
use std::{
    collections::HashMap,
    str::FromStr,
    sync::Arc,
};
use toml::Value;
type OutcomeSeries = HashMap<String, Vec<Tick>>;

#[derive(Debug, Clone)]
pub enum Market {
    NewMarket(MarketStandarized),
    MarketPosition(Tick),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum OutcomeType {
    BINARY,
    MULTIPLE_CHOICE,
    POLL,
}

impl FromStr for OutcomeType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "binary" => Ok(Self::BINARY),
            "multiple_choice" => Ok(Self::MULTIPLE_CHOICE),
            "poll" => Ok(Self::POLL),
            _ => Err(()),
        }
    }
}
// impl From<serde_json::Value> for BetPool {
//     fn from(value: serde_json::Value) -> Self {
//         let NO = value["NO"].as_f64().unwrap();
//         let YES = value["YES"].as_f64().unwrap();
//         BetPool { NO, YES }
//     }
// }

impl From<serde_json::Value> for Tick {
    fn from(value: serde_json::Value) -> Self {
        let timestamp = value["timestamp"].as_i64().unwrap();
        let volume = value["volume"].as_f64().unwrap();
        Tick { timestamp, volume }
    }
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct MarketStandarized {
    pub platform: Platform,
    pub question: String,
    pub idx: usize,
    pub created_time: i64,
    pub close_time: i64,
    pub total_liquidity: i32,
    pub outcome_type: Option<OutcomeType>,
    // pub pool: Option<BetPool>,
    // pub indicators: Option<Indicators>,
    //TODO: add this when can handle multiple choice
    // pub outcome_series: Vec<OutcomeSeries>,
}

impl Default for MarketStandarized {
    fn default() -> Self {
        Self {
            platform: Platform::Manifold,
            question: "".to_string(),
            idx: 0,
            created_time: 0,
            close_time: 0,
            total_liquidity: 0,
            outcome_type: None,
            // pool: None,
            // indicators: None,
        }
    }
}

impl MarketStandarized {
    pub fn new(question: &str) -> Self {
        Self {
            question: question.to_string(),
            ..Default::default()
        }
    }
    pub async fn send_request(
        &self,
        tx: serde_json::Value,
    ) -> Result<String, Box<dyn std::error::Error>> {
        unimplemented!()
    }
}

impl From<serde_json::Value> for MarketStandarized {
    fn from(value: serde_json::Value) -> Self {
        let question = value.get("question").unwrap().as_str().unwrap();
        let platform = value.get("platform").unwrap().as_str().unwrap();
        let idx = value.get("idx").unwrap().as_u64().unwrap() as usize;
        let created_time = value.get("created_time").unwrap().as_i64().unwrap();
        let close_time = value.get("close_time").unwrap().as_i64().unwrap();
        let total_liquidity = value.get("total_liquidity").unwrap().as_i64().unwrap() as i32;
        let outcome_type = value["outcomeType"]
            .to_string()
            .parse::<OutcomeType>()
            .unwrap();
        // let pool = value["pool"].to_string().parse::<BetPool>().unwrap();

        Self {
            platform: Platform::from_str(platform).unwrap(),
            question: question.to_string(),
            idx,
            created_time,
            close_time,
            total_liquidity,
            outcome_type: Some(outcome_type),
            // pool: Some(BetPool::from(pool)),
            //            indicators: Some(Indicators::from(indicators)),
        }
    }
}

#[derive(Deserialize, Debug, Serialize, Clone)]
enum Platform {
    Manifold,
    Gamma,
    Polymarket,
    Metaculus,
}

impl FromStr for Platform {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "manifold" => Ok(Self::Manifold),
            "gamma" => Ok(Self::Gamma),
            "polymarket" => Ok(Self::Polymarket),
            "metaculus" => Ok(Self::Metaculus),
            _ => Err(()),
        }
    }
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct MarketOutcome {
    outcome: String,
    idx: usize,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct Indicators {
    num_forecasts: i32,
    num_forecasters: i32,
    spread: f32,
    shares_volume: f32,
    likes: i32,
    votes: i32,
    stars: i32,
}
// #[derive(Deserialize, Debug, Serialize, Clone)]
// pub struct BetPool {
//     pub NO: Vec<Tick>,
//     pub YES: Vec<Tick>,
// }

#[derive(Deserialize, Debug, Serialize, Clone)]

pub enum StrategyType {
    ARBITRAGE,
    MARKETMAKING,
}

#[derive(Deserialize, Debug, Serialize, Clone, Default, Copy)]
pub struct Tick {
    pub timestamp: i64,
    pub volume: f64,
}

#[derive(Clone)]
pub struct StrategyConfig {
    pub id: String,
    pub strategy_type: StrategyType,
    pub enabled: bool,
    pub period: u64,
    pub qdrant: Arc<Qdrant>,
    pub collection_name: String,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            collection_name: "default".to_string(),
            strategy_type: StrategyType::ARBITRAGE,
            qdrant: Arc::new(Qdrant::new(QdrantConfig::default()).unwrap()),
            enabled: true,
            period: 60,
        }
    }
}
pub struct Settings {
    pub markets: Vec<String>,
    pub period: u64,
    pub sled_config: sled::Config,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            markets: vec![],
            period: 60,
            sled_config: sled::Config::default(),
        }
    }
}

impl Settings {
    pub async fn new(matches: Command) -> Settings {
        let matches = matches.get_matches();
        let path = matches.get_one::<String>("config").unwrap();
        let file: Option<String> = match fs::read_to_string(path) {
            Ok(file) => Some(file),
            Err(_) => panic!("\x1b[31mErr:\x1b[0m Error opening config file at {}", path),
        };

        if let Some(file) = file {
            tracing::info!("Using config file at {}", path);
            return Settings::create_from_file(file).await;
        }

        tracing::info!("Using command line arguments for settings...");
        Settings::create_from_matches(matches)
    }

    async fn create_from_file(config_file: String) -> Settings {
        let parsed_toml = config_file.parse::<Value>().expect("Error parsing TOML");
        let table_names: Vec<&String> = parsed_toml.as_table().unwrap().keys().collect::<Vec<_>>();
        let mut question_list: Vec<String> = Vec::new();
        let rpredict_table = parsed_toml.get("rpredict").unwrap().as_table().unwrap();
        let period = rpredict_table.get("period").unwrap().as_integer().unwrap() as u64;
        let sled_table = parsed_toml
            .get("sled")
            .expect("\x1b[31mErr:\x1b[0m Missing sled table!")
            .as_table()
            .expect("\x1b[31mErr:\x1b[0m Could not parse sled_table as table!");
        let db_path = sled_table
            .get("db_path")
            .expect("\x1b[31mErr:\x1b[0m Missing db_path!")
            .as_str()
            .expect("\x1b[31mErr:\x1b[0m Could not parse db_path as str!");
        let sled_config = sled::Config::default().path(db_path);

        for table_name in table_names {
            if table_name == "manifold"
                || table_name == "gamma"
                || table_name == "polymarket"
                || table_name == "metaculus"
            {
                let platform_table = parsed_toml.get(table_name).unwrap().as_table().unwrap();
                let platform = Platform::from_str(table_name).unwrap();
                let questions = platform_table.get("questions").unwrap().as_array().unwrap();
                for question in questions {
                    let question = question.as_str().unwrap();
                    question_list.push(question.to_string());
                }
            }
        }
        Settings {
            markets: question_list,
            period,
            sled_config,
        }
    }
    fn create_from_matches(matches: ArgMatches) -> Settings {
        let questions_list: String = matches
            .get_one::<String>("question_list")
            .expect("No questions provided")
            .to_string();
        let questions_list: Vec<&str> = questions_list.split(",").collect();
        let questions_list: Vec<String> = questions_list.iter().map(|q| q.to_string()).collect();
        let period = matches.get_one::<String>("period").expect("Invalid period");
        let period = period.parse::<u64>().expect("Invalid period");

        Settings {
            markets: questions_list,
            period,
            sled_config: sled::Config::default(),
        }
    }
}

pub fn create_match() -> clap::Command {
    Command::new("rpredict_indexer")
        .arg(
            Arg::new("question_list")
                .num_args(1..)
                .conflicts_with("config"),
        )
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .num_args(1..)
                .default_value("config.toml")
                .conflicts_with("question_list")
                .help("Path to the configuration file"),
        )
        .arg(
            Arg::new("period")
                .long("period")
                .num_args(1..)
                .default_value("15")
                .help("period in seconds"),
        )
        .arg(
            Arg::new("db")
                .long("db")
                .short('d')
                .num_args(1..)
                .default_value("rpredict-cache")
                .help("Database path"),
        )
}
// impl Default for Context {
//     fn default() -> Self {
//         let platform = api::manifold::ManifoldPlatform::from(PlatformBuilder::default());

//         Self {
//             manifold: platform,
//             exit: false,

//             id: "default".to_string(),
//             strategy_config: Arc::new(RwLock::new(StrategyConfig::default())),
//             questions: vec![],
//             indicators: Indicators {
//                 num_forecasts: 0,
//                 num_forecasters: 0,
//                 spread: 0.0,
//                 shares_volume: 0.0,
//                 likes: 0,
//                 votes: 0,
//                 stars: 0,
//             },
//         }
//     }
// }
