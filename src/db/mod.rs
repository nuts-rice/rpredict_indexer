pub mod model;
pub use model::*;
use slab::Slab;
pub mod error;
use self::question::DBQuestion;
use futures_util::lock::Mutex;
use std::sync::Arc;
pub mod graphql;
use async_graphql::{http::GraphiQLSource, Schema};
use axum::response::{self, IntoResponse};
use question::{MutationRoot, QueryRoot, Storage, SubscriptionRoot};
pub enum DbType {
    GraphQL,
}
pub type Db = sled::Db;
pub type QuestionStorage = Arc<Mutex<Slab<DBQuestion>>>;

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
        .data(Storage::default())
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
