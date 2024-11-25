pub mod model;
pub use model::*;
pub mod error;
pub mod graphql;
use async_graphql::{http::GraphiQLSource, Schema};
use axum::response::{self, IntoResponse};
use question::{MutationRoot, QueryRoot, QuestionStorage, SubscriptionRoot};
pub enum DbType {
    GraphQL,
}
pub type MarketsDb = sled::Db;

pub async fn build_graphql() -> impl IntoResponse {
    let response = response::Html(
        GraphiQLSource::build()
            .endpoint("/graphql")
            .subscription_endpoint("/ws")
            .finish(),
    );
    response
}

pub async fn build_schema() {
    let schema = Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
        .data(QuestionStorage::default())
        .finish();
}

// pub fn build(db_type: DbType, db_path: &str) -> Result<Box<dyn Db>, Error> {
//     let db = match db_type {
//         DbType::GraphQL =>
//             Builder::<GraphQL>::build(db_path)
//         }?;

//     }

// pub struct Builder<T>
// where
//     T: Db,
// {
//     phantom: std::marker::PhantomData<T>,
// }

// impl<T> Builder<T>
// where
//     T: Db + 'static,
// {
//     pub(self) fn build(db_path: &str) -> Result<Box<dyn Db>, Error> {
//         todo!()
//     }
// }

// pub trait Db: Sync + Send {
//     fn new() -> Result<Self, Error>
//     where
//         Self: Sized;
//     fn create(&self, db_path: &str) -> Result<(), Error>;
// }
