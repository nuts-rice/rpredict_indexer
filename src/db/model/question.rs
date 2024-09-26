use crate::QuestionStorage;

use super::simplebroker::SimpleBroker;
use async_graphql::*;
use futures_util::{lock::Mutex, Stream, StreamExt};
use slab::Slab;
use std::sync::Arc;
// use crate::db::Db;
pub struct Indicators {
    num_forecasts: i32,
    num_forecasters: i32,
    spread: f32,
    shares_volume: f32,
    likes: i32,
    votes: i32,
    stars: i32,
}

pub type QuestionsSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

#[derive(Debug, Clone)]
pub struct DBQuestion {
    id: async_graphql::ID,
    text: String,
    timestamp: i64,
}

#[Object]
impl DBQuestion {
    async fn query(&self) -> String {
        self.text.to_string()
    }
    async fn text(&self) -> &str {
        &self.text
    }
    async fn timestamp(&self) -> i64 {
        self.timestamp
    }
}

pub type Storage = Arc<Mutex<Slab<DBQuestion>>>;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn questions(&self, ctx: &Context<'_>) -> Vec<DBQuestion> {
        let storage = ctx.data_unchecked::<QuestionStorage>().lock().await;
        let questions = storage.iter().map(|(_, q)| q).cloned().collect();
        questions
    }
}

pub struct MutationRoot;

#[derive(Enum, Eq, PartialEq, Copy, Clone)]
enum MutationType {
    Created,
    Deleted,
}

#[derive(Clone)]
struct QuestionChanged {
    mutation_type: MutationType,
    id: ID,
}
#[Object]
impl QuestionChanged {
    async fn mutation_type(&self) -> MutationType {
        self.mutation_type
    }
    async fn id(&self) -> &ID {
        &self.id
    }
    async fn question(&self, ctx: &Context<'_>) -> Result<Option<DBQuestion>> {
        let questions = ctx.data_unchecked::<QuestionStorage>().lock().await;
        let id = self.id.parse::<usize>()?;

        Ok(questions.get(id).cloned())
    }
}
#[Object]
impl MutationRoot {
    async fn add_question(&self, ctx: &Context<'_>, text: String, timestamp: i64) -> ID {
        let mut questions = ctx.data_unchecked::<QuestionStorage>().lock().await;
        let entry = questions.vacant_entry();
        let id: ID = entry.key().into();
        let question = DBQuestion {
            id: id.clone(),
            text,
            timestamp,
        };
        questions.insert(question);
        SimpleBroker::publish(QuestionChanged {
            mutation_type: MutationType::Created,
            id: id.clone(),
        });
        id
    }
}

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn interval(&self, #[graphql(default = 1)] interval: i32) -> impl Stream<Item = i32> {
        let mut value = 0;
        async_stream::stream! {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(interval as u64)).await;
                value += interval;
                yield value;
            }
        }
    }

    async fn questions(
        &self,
        mutation_type: Option<MutationType>,
    ) -> impl Stream<Item = QuestionChanged> {
        SimpleBroker::<QuestionChanged>::subscribe().filter(move |event| {
            let res = if let Some(mutation_type) = mutation_type {
                event.mutation_type == mutation_type
            } else {
                true
            };
            async move { res }
        })
    }
}
