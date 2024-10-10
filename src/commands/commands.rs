use super::Result;
use clap::Arg;

pub struct Args {
    pub event_title: String,
    pub outcome: String,
}

pub async fn ask_superforecaster(event_title: &str, outcome: &str) -> Result<()> {
    unimplemented!()
}
