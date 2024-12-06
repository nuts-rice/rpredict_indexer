use super::*;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
// struct

pub async fn handle_get_polymarket_markets(limit: u16, sort_by: &str) -> Result<()> {
    unimplemented!()
}

pub async fn handle_get_relevant_news(tags: &str) -> Result<()> {
    unimplemented!()
}

pub async fn handle_ask_superforecaster(
    event_title: &str,
    market_question: &str,
    outcome: &str,
) -> Result<()> {
    unimplemented!()
}

pub async fn handle_run_autonmous_trader() -> Result<()> {
    unimplemented!()
}

pub async fn handle_ask_llm(user_input: &str) -> Result<()> {
    unimplemented!()
}

pub async fn handle_ask_polymarket_llm(user_input: &str) -> Result<()> {
    // let executor = crate::
    unimplemented!()
}

pub async fn handle_ask_local_rag(vector_db_dir: &str, query: &str) -> Result<()> {
    unimplemented!()
}

pub async fn handle_create_rag(local_dir_path: &str) -> Result<()> {
    unimplemented!()
}
