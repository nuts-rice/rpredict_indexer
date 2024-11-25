use super::simplebroker::SimpleBroker;
use crate::StandardMarket;
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
    pub title: String,
    pub platform: String,
    pub platform_id: String,
    pub open_time: String,
    pub close_time: String,
    pub volume_usd: f32,
    pub num_traders: i32,
    pub category: String,
    pub resolution: f32,
    pub prob_midpoint: f32,
    pub prob_close: f32,
    pub prob_tma: f32,
}

#[Object]
impl DBQuestion {
    pub async fn id(&self) -> &ID {
        &self.id
    }
    pub async fn title(&self) -> &str {
        &self.title
    }
    pub async fn open_time(&self) -> &str {
        &self.open_time
    }
    pub async fn close_time(&self) -> &str {
        &self.close_time
    }
    pub async fn volume_usd(&self) -> f32 {
        self.volume_usd
    }
    // pub async fn closeTime(&self) -> &str {
    //     &self.close_time
    // }
}

pub type QuestionStorage = Arc<Mutex<Slab<DBQuestion>>>;

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
    async fn add_question(
        &self,
        ctx: &Context<'_>,
        title: String,
        open_time: String,
        close_time: String,
        category: String,
        platform: String,
        volume_usd: f32,
        num_traders: i32,
        platform_id: String,
        prob_close: f32,
        prob_tma: f32,
        prob_midpoint: f32,
        resolution: f32,
    ) -> ID {
        let mut questions = ctx.data_unchecked::<QuestionStorage>().lock().await;
        let entry = questions.vacant_entry();
        let id: ID = entry.key().into();
        let question = DBQuestion {
            id: id.clone(),
            title,
            open_time,
            close_time,
            category,
            platform,
            volume_usd,
            num_traders,
            platform_id,
            prob_close,
            prob_midpoint,
            prob_tma,
            resolution,
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

impl From<&StandardMarket> for DBQuestion {
    fn from(value: &StandardMarket) -> Self {
        DBQuestion {
            //TODO: Change ID to be a unique identifier
            id: async_graphql::ID::from(uuid::Uuid::new_v4().to_string().as_str()),
            title: value.title.clone(),
            platform: value.platform.clone(),
            platform_id: value.platform_id.clone(),
            open_time: value.open_time.to_string(),
            close_time: value.close_time.to_string(),
            volume_usd: value.volume_usd,
            num_traders: value.num_traders,
            category: value.category.clone(),
            resolution: value.resolution,
            prob_midpoint: value.prob_midpoint,
            prob_close: value.prob_close,
            prob_tma: value.prob_tma,
        }
    }
}
