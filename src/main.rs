use async_graphql::*;
use async_graphql_axum::{GraphQL, GraphQLSubscription};
use axum::{response::Html, routing::get, Router};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
pub mod api;
pub mod db;
pub mod server;
pub mod types;
pub use db::*;
const MANIFOLD_ENDPOINT: &str = "https://api.manifold.markets/v0/markets?limit=1";

async fn handler() -> Html<&'static str> {
    Html(
        "
         <h1>Prediction market indexer</h1>
        <span> Markets </span>
        <section>
        Latest Markets

        </section>
         ",
    )
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
    let question_schema = Schema::build(
        model::question::QueryRoot,
        model::question::MutationRoot,
        model::question::SubscriptionRoot,
    )
    .data(model::question::Storage::default())
    .finish();

    tracing::debug!("connecting to graphql");

    let app = Router::new()
        .route("/", get(handler))
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
    let client = api::APIClient::new(MANIFOLD_ENDPOINT).unwrap();

    let questions = client.fetch_page().await.unwrap();
    // .unwrap();
    // tracing::debug!("questions: {:#?}", questions[0]);
    axum::serve(listener, app).await.unwrap();
}
