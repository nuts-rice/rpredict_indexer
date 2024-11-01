use crate::context::Context;
use crate::executor::executor::Executor;
use crate::types::{create_match, Market, Settings};
use anyhow::Result;
use api::Platform;
use async_graphql::*;
use axum::{
    extract::Query,
    response::{Html, IntoResponse},
};
use context::StatefulList;
use db::{manifold::ManifoldMarket, metaculus::MetaculusMarket, polymarket::PolymarketResult};
use ratatui::widgets::{Block, List, ListItem};
use ratatui::{
    backend::Backend,
    crossterm::event::{self, KeyCode, KeyEventKind},
    style::{Modifier, Style},
    DefaultTerminal,
};
use serde::Deserialize;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::RwLock;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
pub mod admin;
pub mod api;
pub mod commands;
pub mod context;
pub mod db;
pub mod executor;
pub mod plugins;
pub mod server;
pub mod strategies;
pub mod types;
pub mod ui;
pub use db::*;

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
    let metaculus_markets = api::metaculus::MetaculusPlatform::builder()
        .build()
        .fetch_questions()
        .await
        .expect(">_< error");

    render_metaculus_markets(metaculus_markets, offset, limit).await
}

async fn render_metaculus_markets(
    markets: Vec<MetaculusMarket>,
    offset: usize,
    limit: usize,
) -> Html<String> {
    let mut html = String::new();

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

// async fn markets_index(Option<Query<Pagiation>>, sort: Option<Query<SortType>>) -> impl IntoResponse {
//         let
// }

// pub fn run() -> std::result::Result<(), Box<dyn std::error::Error>> {
//     enable_raw_mode()?;
//     let mut stdout = std::io::stdout();
//     execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
//     let backend = CrosstermBackend::new(stdout);
//     let mut terminal = Terminal::new(backend)?;
//     let context = Context::new();

// }

// fn run_markets<B:

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let strat_config = Arc::new(RwLock::new(create_match()));
    let question_schema = Schema::build(
        model::question::QueryRoot,
        model::question::MutationRoot,
        model::question::SubscriptionRoot,
    )
    .data(model::question::QuestionStorage::default())
    .finish();

    tracing::debug!("connecting to graphql");
    // let executor =
    // ManifoldExecutor::new(Arc::new(api::manifold::ManifoldPlatform::from(PlatformBuilder::default())), Promptor{});

    let config = Arc::new(RwLock::new(Settings::new(create_match()).await));

    let period = {
        let config_guard = config.read().unwrap();
        config_guard.period
    };
    let questions_list_rwlock = Arc::new(RwLock::new(config.read().unwrap().markets.clone()));
    tracing::debug!(
        "questions_list: {:#?}",
        questions_list_rwlock.read().unwrap()
    );
    let mut context: Context<Market> = Context::default();
    questions_list_rwlock
        .read()
        .unwrap()
        .iter()
        .for_each(|q| context.add_question(q.to_string()));
    if let Ok(mut set) = context.run().await {
        while let Some(res) = set.join_next().await {
            tracing::info!("res: {:?}", res);
        }
    }
    //
    //
    // tracing::debug!("context questions: {:#?}", &context.questions);
    // context.questions.

    // let app = Router::new()
    //     .route("/", get(handler))
    //     .route("/manifold_markets", get(manifold_markets_index))
    //     .route("/metaculus_markets", get(metaculus_markets_index))
    //     .route("/polymarket_markets", get(polymarket_markets_index))
    //     .route(
    //         "/graphql",
    //         get(build_graphql).post_service(GraphQL::new(question_schema.clone())),
    //     )
    //     // .route("/questions", get(api::platform::fetch_page(MANIFOLD_ENDPOINT.parse().unwrap())))
    //     .route_service("/ws", GraphQLSubscription::new(question_schema));
    // let listener = tokio::net::TcpListener::bind("127.0.0.1:3010")
    //     .await
    //     .unwrap();
    // tracing::debug!("listening on 127.0.0.1:3010");

    // .unwrap();
    // tracing::debug!("questions: {:#?}", questions[0]);
    // axum::serve(listener, app).await.unwrap();

    // TODO: figure out how to do the terminal
    // run_markets(&mut context);
    Ok(())
}
