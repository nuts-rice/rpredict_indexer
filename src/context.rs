use crate::{executor::executor::Executor, types::StrategyConfig};
use ratatui::widgets::ListState;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::task::JoinSet;

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

pub struct MarketCollector<M> {
    provider: M,
}
impl<M> MarketCollector<M> {}

pub struct OrderCollector<M> {
    provider: M,
}
impl<M> OrderCollector<M> {}

pub struct Context<M> {
    pub id: String,
    pub strategy_config: Arc<RwLock<StrategyConfig>>,
    pub questions: Vec<String>,
    pub executors: Vec<Box<dyn Executor<M>>>,
    pub exit: bool,
    question_channel_capacity: usize,
}

impl<M: Sync + Send + Clone + 'static> Context<M> {
    pub fn new() -> Self {
        let strategy_config = StrategyConfig::default();
        Context {
            executors: vec![],
            questions: vec![],
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
        self.questions.push(question);
    }
    //Core run loop. Spawn thread for each question/executo.
    pub async fn run(self) -> Result<JoinSet<()>, Box<dyn std::error::Error>> {
        let (tx, rx): (Sender<M>, Receiver<M>) = tokio::sync::broadcast::channel(100);
        let set = JoinSet::new();
        for question in self.questions {}
        Ok(set)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_match;
    use crate::Settings;
    use tracing_subscriber::prelude::*;
}
