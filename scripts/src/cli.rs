use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(name = "rpredict")]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Parser, Debug)]
pub enum Commands {
    #[clap(name = "superforecaster")]
    AskSuperforecaster {
        #[clap(long, short)]
        event_title: String,
        #[clap(long, short)]
        market_question: String,
        #[clap(long, short)]
        outcome: String,
    },
    #[clap(name = "ask_llm")]
    AskLLM {
        #[clap(long, short)]
        user_input: String,
    },
    #[clap(name = "ask_polymarket_llm")]
    AskPolymarketLLM {
        #[clap(long, short)]
        user_input: String,
    },
    #[clap(name = "run_autonomous_trader")]
    RunAutonomousTrader,
    #[clap(name = "query_local_rag")]
    QueryLocalRag {
        #[clap(long, short)]
        query: String,
        #[clap(long, short)]
        vector_db_path: String,
    },
    #[clap(name = "get_relevant_news")]
    GetRelevantNews { tags: String },
    #[clap(name = "create_local_rag")]
    CreateLocalRag { local_dir_path: String },
}
