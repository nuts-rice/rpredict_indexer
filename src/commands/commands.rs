use super::Result;
use clap::{Arg, Command, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[clap(about = "Ask a superforecaster")]
    AskSuperforecaster {
        #[clap(short, long)]
        question_title: Option<String>,
        #[clap(short, long)]
        description: Option<String>,

        #[clap(short, long)]
        outcome: Option<String>,
    },
}

// pub fn matches_command() -> clap::Command {
//     Command::new("rpredict-indexer")
//         .
// }

// pub async fn ask_superforecaster(event_title: &str, outcome: &str) -> Result<()> {
// println!("Asking superforecaster: event_title: {}, outcome (usually yes or no): {}", event_title, outcome);
// }

pub async fn get_all_markets(limit: u32, sort_by: &str) -> Result<()> {
    unimplemented!()
}
