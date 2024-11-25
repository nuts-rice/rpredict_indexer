use clap::{command, Parser};
mod cli;
mod commands;
use commands::command::{self, *};
#[tokio::main]
async fn main() {
    let cli = cli::Cli::parse();
    match &cli.command {
        cli::Commands::AskLLM { user_input } => {
            let result = commands::command::handle_ask_llm(user_input).await;
        }
        cli::Commands::RunAutonomousTrader => {
            let result = command::handle_run_autonmous_trader().await;
        }
        cli::Commands::AskPolymarketLLM { user_input } => {
            let result = command::handle_ask_polymarket_llm(user_input).await;
        }
        cli::Commands::AskSuperforecaster {
            event_title,
            market_question,
            outcome,
        } => {
            let result =
                command::handle_ask_superforecaster(event_title, market_question, outcome).await;
        }
        cli::Commands::QueryLocalRag {
            query,
            vector_db_path,
        } => {
            let result = command::handle_ask_local_rag(query, vector_db_path).await;
        }
        cli::Commands::CreateLocalRag { local_dir_path } => {
            let result = command::handle_create_rag(local_dir_path).await;
        }
        cli::Commands::GetRelevantNews { tags } => {
            let result = command::handle_get_relevant_news(tags).await;
        }
    }
}
