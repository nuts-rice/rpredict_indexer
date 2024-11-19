use crate::context::Context;
use crate::executor::executor::{
    Executor, ExecutorType, ManifoldExecutor, PolymarketExecutor, Promptor,
};
use crate::types::create_match;
use api::manifold::manifold_api::ManifoldPlatform;
use api::Platform;
use async_graphql::*;
use async_graphql_axum::{GraphQL, GraphQLSubscription};
use axum::{
    extract::Query,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    style::Stylize,
    widgets::Paragraph,
    DefaultTerminal, Terminal,
};
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::RwLock;

use db::{manifold::ManifoldMarket, metaculus::MetaculusMarket, polymarket::PolymarketResult};
use serde::Deserialize;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
pub mod admin;
pub mod api;
pub mod collector;
pub mod commands;
pub mod context;
pub mod db;
pub mod executor;
pub mod plugins;
pub mod server;
pub mod strategies;
pub mod types;
pub mod ui;
pub mod utils;
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
    let manifold = ManifoldPlatform::builder()
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
        <h2>{:?} </h2>
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

fn run_markets(ctx: Context) -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear();
    let app_result = run(terminal, ctx);
    ratatui::restore();
    app_result
}

fn render_market_select(markets: Vec<String>) -> std::io::Result<()> {
    unimplemented!()
}

fn run(mut terminal: DefaultTerminal, ctx: Context) -> std::io::Result<()> {
    loop {
        terminal.draw(|frame| {
            let greeting = Paragraph::new("Hello Ratatui! (press 'q' to quit)")
                .white()
                .on_blue();
            frame.render_widget(greeting, frame.area());
        })?;

        if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(());
            }
        }
    }
}

#[tokio::main]
async fn main() {
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
    let tags: Vec<String> = vec!["us-economics".to_string(), "economics".to_string()];
    let mut context = Context::new();
    let executor =
        ManifoldExecutor::builder(1000, 1000, Promptor {}, ExecutorType::Manifold).build();
    let result = executor
        .init(
            "Will the 10 Year Treasury Yield at closing on 12/31/2024 be 4% or higher?",
            "10 year treasury yield at closing on 12/31/2024 be 4% or higher",
            Some(tags),
            &mut context,
        )
        .await
        .unwrap();
    tracing::debug!("result: {:#?}", result);

    let mut dummy_questions = vec![
        "What is the probability of GPT-5 being availiable by 2025".to_string(),
        "What is the probability of Stalker 2 being released by 2025".to_string(),
        "Will the 10 Year Treasury Yield at closing on 12/31/2024 be 4% or higher?".to_string(),
    ];
    context.questions.push(dummy_questions.into());

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
    // run_markets(context);
}
