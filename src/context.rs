use crate::{executor::executor::Executor, types::StrategyConfig};
use ratatui::widgets::ListState;
use ratatui::widgets::{Block, List, ListItem};
use ratatui::{
    backend::Backend,
    crossterm::event::{self, KeyCode, KeyEventKind},
    style::{Modifier, Style},
    DefaultTerminal,
};
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::task::JoinSet;

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<M> StatefulList<M> {
    pub fn with_items(items: Vec<M>) -> Self {
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

pub struct MarketCollector<M> {
    provider: M,
}
impl<M> MarketCollector<M> {}

pub struct OrderCollector<M> {
    provider: M,
}
impl<M> OrderCollector<M> {}

//TODO: map to strat config, map outcome  + tags to respective question
pub struct Context<M> {
    pub id: String,
    pub strategy_config: Arc<RwLock<StrategyConfig>>,
    pub questions: StatefulList<String>,
    pub outcome: String,
    pub tags: Vec<String>,
    pub executors: Vec<Box<dyn Executor<M>>>,
    pub exit: bool,
    question_channel_capacity: usize,
}

impl<M: Sync + Send + Clone + 'static> Context<M> {
    pub fn new() -> Self {
        let strategy_config = StrategyConfig::default();
        Context {
            executors: vec![],
            tags: vec![],
            outcome: "".to_string(),
            questions: StatefulList::with_items(vec![]),
            id: "default".to_string(),
            question_channel_capacity: 512,
            strategy_config: Arc::new(RwLock::new(strategy_config)),
            exit: false,
        }
    }

    pub fn with_question_channel_capacity(mut self, capacity: usize) -> Self {
        self.question_channel_capacity = capacity;
        self
    }
}

impl<M: Send + Sync + Clone + 'static> Default for Context<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M> Context<M>
where
    M: Send + Sync + Clone + 'static,
{
    pub fn add_question(&mut self, question: String) {
        self.questions.items.push(question);
    }
    //Core run loop. Spawn thread for each question/executo.
    pub async fn run(self) -> Result<JoinSet<()>, Box<dyn std::error::Error>> {
        let (tx, rx): (Sender<M>, Receiver<M>) =
            tokio::sync::broadcast::channel(self.question_channel_capacity);
        let mut set = JoinSet::new();
        for executor in self.executors {
            let mut rx = tx.subscribe();
            set.spawn(async move {
                tracing::info!("starting executor... ");
                loop {

                    match rx.recv().await {
                        //TODO: un-dummy this
                        Ok(question) => match executor.init("Will the 10 Year Treasury Yield at closing on 12/31/2024 be 4% or higher?", "The 10 year treasury yield closing at 4% or higher on 12/31/2024", vec!["us-economics".to_string()]).await {
                            Ok(_) => {}
                            Err(e) => tracing::error!("error executing action: {}", e),
                        },
                        Err(e) => tracing::error!("error receiving action: {}", e),
                    }
                }
            });
        }

        Ok(set)
    }
    fn run_terminal(&mut self, mut terminal: DefaultTerminal) -> std::io::Result<()> {
        let markets: Vec<ListItem> = self
            .questions
            .items
            .iter()
            .map(|q| ListItem::from(q.to_string()))
            .collect();
        let markets = List::new(markets)
            .block(Block::bordered().title("Markets"))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");
        loop {
            terminal.draw(|frame| {
                frame.render_stateful_widget(
                    markets.clone(),
                    frame.area(),
                    &mut self.questions.state,
                );
            })?;

            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    return Ok(());
                }
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Up {
                    self.questions.previous();
                }
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Down {
                    self.questions.next();
                }
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Enter {
                    // let selected = self.questions;
                    // println!("selected: {:?}", selected);
                }
                // if key.kind == KeyEventKind::Press && key.code == KeyCode::Enter {
                //     let selected = ctx.questions.selected();
                //     println!("selected: {}", selected);
            }
        }
    }

    fn run_markets(&mut self) -> std::io::Result<()> {
        let mut terminal = ratatui::init();
        terminal.clear();
        let app_result = self.run_terminal(terminal);
        ratatui::restore();
        app_result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_match;
    use crate::Settings;
    use tracing_subscriber::prelude::*;
}
