use super::Result;
use crate::Context;
use clap::{Parser, Subcommand};
use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{
        block::Title,
        Block, Paragraph, Widget,
    },
    DefaultTerminal, Frame,
};
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

//TODO: Standarized market
pub async fn parse_markets(markets: Vec<serde_json::Value>) -> Result<Vec<String>> {
    let selectable_markets = markets
        .iter()
        .map(|market| {
            let question = market["question"].as_str().unwrap();
            question.to_string()
        })
        .collect::<Vec<String>>();
    Ok(selectable_markets)
}

pub fn draw_market_select(buf: &mut Buffer, markets: Vec<String>, area: Rect) {
    let title = Title::from("Markets".bold());
    let selectable_markets = markets
        .iter()
        .map(|market| Line::from(market.as_str()))
        .collect::<Vec<Line>>();

    let block = Block::bordered()
        .title(title.alignment(Alignment::Center))
        .border_set(border::THICK);
    let text = Text::from(selectable_markets.clone());
    Paragraph::new(text).centered().render(area, buf);
}

pub fn handle_events(ctx: Context) -> std::io::Result<()> {
    match event::read()? {
        Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
            handle_key_event(ctx, key_event);
        }
        _ => {}
    };
    Ok(())
}

pub fn handle_key_event(ctx: Context, key_event: KeyEvent) {}

pub fn run(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    unimplemented!()
}

pub fn draw(ctx: Context, frame: &mut Frame) {
    unimplemented!()
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

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Style;
}
