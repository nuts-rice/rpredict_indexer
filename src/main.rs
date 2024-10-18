use std::fmt::Debug;

use crate::executor::executor::{Executor, ExecutorType, PolymarketExecutor, Promptor};
use api::Platform;
use async_graphql::*;
use async_graphql_axum::{GraphQL, GraphQLSubscription};
use axum::{
    extract::Query,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use db::{manifold::ManifoldMarket, polymarket::PolymarketResult};
use serde::Deserialize;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
pub mod api;
pub mod commands;
pub mod db;
pub mod executor;
pub mod server;
pub mod strategies;
pub mod types;
pub mod web;
pub use db::*;

use crate::metaculus::MetaculusMarket;
const MANIFOLD_ENDPOINT: &str = "https://api.manifold.markets/v0/markets";
#[derive(Debug, Deserialize, Default)]
pub struct Pagiation {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

async fn handler() -> Html<&'static str> {
    Html(
        "
         <h1>Prediction market indexer</h1>
        <span> Markets </span>
        <section>
        Latest Markets

        </section>
        <section>
        </section>
         ",
    )
}

async fn manifold_markets_index(pagiation: Option<Query<Pagiation>>) -> impl IntoResponse {
    let pagiation = pagiation.unwrap_or_default();
    let offset = pagiation.offset.unwrap_or(0);
    let limit = pagiation.limit.unwrap_or(10);
    let manifold = api::manifold::ManifoldPlatform::builder()
        .build()
        .fetch_questions()
        .await
        .expect(">_< error");
    tracing::debug!("manifold: {:#?}", &manifold);
    // match manifold.first() {
    //     Some(question) => {
    //         tracing::info!("question: {:#?}", question);
    //     }
    //     None => {
    //         tracing::info!("no questions found");
    //     }
    // }

    render_markets(manifold, offset, limit).await

    // tracing::debug!("page: {:#?}", page);
    //    page

    // let client = api::APIClient::new(MANIFOLD_ENDPOINT).unwrap();
    // let questions = client.fetch_page(limit as u32).await.unwrap();
    // let html: Html<String> = format!(
    //     r#"
    //         <h1>Manifold Markets</h1>
    //         <section>
    //         <h2>Latest Markets</h2>
    //         {:?}
    //     "#,
    //     &json
    // )
    // .into();
}

async fn render_markets(markets: Vec<ManifoldMarket>, offset: usize, limit: usize) -> Html<String> {
    let page_length = markets.len() / limit;
    let page = 0;
    let mut html = String::new();
    for i in 0..page_length {
        let question_card: String = format!(
            r#"
            <div>
        <h2>{} </h2>
        </div>
        "#,
            &markets[i]
        );
        html.push_str(&question_card);
    }
    Html(html)
}

async fn polymarket_markets_index(pagiation: Option<Query<Pagiation>>) -> impl IntoResponse {
    let pagiation = pagiation.unwrap_or_default();
    let questions = api::polymarket::PolymarketPlatform::builder()
        .build()
        .fetch_json()
        .await
        .expect(">_< error");
    let result = PolymarketResult::from(questions);
    render_polymarket(result, 0, 10).await
}

async fn render_polymarket(results: PolymarketResult, offset: usize, limit: usize) -> Html<String> {
    let mut html = String::new();
    let markets = results.data;
    let page_length = markets.len() / limit;
    let page = 0;
    for i in 0..page_length {
        let question_card: String = format!(
            r#"
            <div>
        <h2>{} </h2>
        </div>
        "#,
            &markets[i]
        );
        html.push_str(&question_card);
    }
    Html(html)
}

async fn metaculus_markets_index(pagiation: Option<Query<Pagiation>>) -> impl IntoResponse {
    let pagiation = pagiation.unwrap_or_default();
    let offset = pagiation.offset.unwrap_or(0);
    let limit = pagiation.limit.unwrap_or(10);
    let metaculus_json = api::metaculus::MetaculusPlatform::builder()
        .build()
        .fetch_json()
        .await
        .expect(">_< error");
    let markets: MetaculusMarket = MetaculusMarket::from(metaculus_json);

    render_metaculus_markets(markets, offset, limit).await
}

async fn render_metaculus_markets(
    markets: MetaculusMarket,
    offset: usize,
    limit: usize,
) -> Html<String> {
    let mut html = String::new();
    let results = markets.results;
    let page_length = results.len() / limit;
    let page = 0;
    for i in 0..page_length {
        let question_card: String = format!(
            r#"
            <div>
        <h2>{} </h2>
        </div>
        "#,
            &results[i]
        );
        html.push_str(&question_card);
    }
    Html(html)
}

// async fn markets_index(Option<Query<Pagiation>>, sort: Option<Query<SortType>>) -> impl IntoResponse {
//         let
// }

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let question_schema = Schema::build(
        model::question::QueryRoot,
        model::question::MutationRoot,
        model::question::SubscriptionRoot,
    )
    .data(model::question::QuestionStorage::default())
    .finish();

    tracing::debug!("connecting to graphql");
    let executor =
        PolymarketExecutor::builder(1000, 1000, Promptor {}, ExecutorType::Polymarket).build();
    // let event_data =
    // let result = executor
    //     .init(
    //         "What is the probability of Joe Biden winning the 2024 US elections?",
    //         "Joe Biden is the current president of the United States of America",
    //         "Joe Biden winning the 2024 US elections",
    //     )
    //     .await
    //     .unwrap();

    // let cli = commands::commands::Cli::parse();
    //     match &cli.command {
    //         commands::commands::Commands::AskSuperforecaster { question_title, description, outcome } => {
    //             let _ = executor::PolymarketExecutor::builder(1000, 1000, executor::Promptor {  }, executor::ExecutorType::Polymarket).build().init(question_title.clone().unwrap().as_str(), description.clone().unwrap().as_str(), outcome.clone().unwrap().as_str());
    //         },
    //         _ => {}
    //     }

    let app = Router::new()
        .route("/", get(handler))
        .route("/manifold_markets", get(manifold_markets_index))
        .route("/metaculus_markets", get(metaculus_markets_index))
        .route("/polymarket_markets", get(polymarket_markets_index))
        .route(
            "/graphql",
            get(build_graphql).post_service(GraphQL::new(question_schema.clone())),
        )
        // .route("/questions", get(api::platform::fetch_page(MANIFOLD_ENDPOINT.parse().unwrap())))
        .route_service("/ws", GraphQLSubscription::new(question_schema));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3010")
        .await
        .unwrap();
    tracing::debug!("listening on 127.0.0.1:3010");

    // .unwrap();
    // tracing::debug!("questions: {:#?}", questions[0]);
    axum::serve(listener, app).await.unwrap();
    // let cli = commands::commands::Cli::parse();
    // match &cli.command {
    //     commands::commands::Commands::AskSuperforecaster { question_title, description, outcome } => {
    //         let _ = executor::PolymarketExecutor::builder(1000, 1000, executor::Promptor {  }, executor::ExecutorType::Polymarket).build().init(question_title.clone().unwrap().as_str(), description.clone().unwrap().as_str(), outcome.clone().unwrap().as_str());
    //     },
    //     _ => {}
    // }
}
