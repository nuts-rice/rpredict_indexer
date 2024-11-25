use crate::admin::listener::MarketUpdateRcv;
use crate::api::manifold::manifold_api::ManifoldPlatform;
use crate::api::manifold::utils::order::BetParams;
use crate::api::{self, Platform, *};
use crate::context::Context;
use crate::manifold::{ManifoldMarket, MarketOutcome};
use crate::model::manifold::User;
use crate::polymarket::PolymarketEvent;
use async_openai::types::realtime::{ConversationItemCreateEvent, Item, ResponseCreateEvent};
use async_openai::types::{CreateMessageRequestArgs, CreateRunRequestArgs};
use async_openai::{
    config::OpenAIConfig,
    types::{
        AssistantStreamEvent, CreateAssistantRequestArgs, CreateThreadRequest, MessageContent, MessageDeltaContent,
        MessageRole, RunObject, RunStatus, SubmitToolOutputsRunRequest, ToolsOutputs,
    },
    Client,
};
use axum::async_trait;
use futures_util::StreamExt;
use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, PointStruct,
    VectorParamsBuilder,
};
use qdrant_client::Qdrant;
use std::any::Any;
use std::sync::{Arc, RwLock};
use tokio::io::AsyncReadExt;
use tokio_tungstenite::tungstenite::protocol::Message;
// use warp::{http::StatusCode, Filter};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Clone)]
pub enum ExecutorType {
    Polymarket,
    Augur,
    Metaculus,
    Manifold,
}
#[derive(Debug, Clone)]
pub enum PromptorType {
    Polymarket,
    Augur,
    Metaculus,
    Manifold,
}

#[derive(Clone)]
pub struct ExecutorBuilder<E: Executor + Any> {
    marker: std::marker::PhantomData<E>,
    max_tokens: u32,
    token_limit: u32,
    promptor: Promptor,
    executor_type: ExecutorType,
    // platform: api::PlatformBuilder<>
    // assitant_args: CreateAssistantRequestArgs,
}

impl<E: Executor + Any> ExecutorBuilder<E> {
    pub fn new(
        max_tokens: u32,
        token_limit: u32,
        promptor: Promptor,
        executor_type: ExecutorType,
    ) -> Self {
        Self {
            max_tokens,
            token_limit,
            promptor,
            marker: std::marker::PhantomData,
            executor_type,
        }
    }

    pub fn build(self) -> E {
        E::from(self)
    }
}
#[derive(Clone)]
pub struct Promptor {}

impl Promptor {
    pub fn prompts_polymarket(
        &self,
        market_data: Vec<serde_json::Value>,
        event_data: Vec<&String>,
        market_question: &str,
        outcome: &str,
    ) -> String {
        let prompt = format!("You are an AI assistant for users of a prediction market called Polymarket.
        Users want to place bets based on their beliefs of market outcomes such as political or sports events.
        
        Here is data for current Polymarket markets {:?} and 
        current events {:?}.

        Help users identify markets to trade based on their interests or queries.
        Provide specific information for markets including probabilities of outcomes.
        Give your response in the following format:

        I believe {} has a likelihood (float)% for outcome of {}.", market_data, event_data, market_question, outcome);
        prompt
    }
    pub fn prompts_polymarket_filter(
        &self,
        market_data: Vec<&serde_json::Value>,
        event_data: Vec<&String>,
        market_question: &str,
        outcome: &str,
    ) -> String {
        let prompt = format!("You are an AI assistant for users of a prediction market called Polymarket.
        Users want to place bets based on their beliefs of market outcomes such as political or sports events.
        
        Here is data for current Polymarket markets {:?} and 
        current events {:?}.

        Help users identify markets to trade based on their interests or queries.
        Provide specific information for markets including probabilities of outcomes.
        Give your response in the following format:

        I believe {} has a likelihood (float)% for outcome of {}.", market_data, event_data, market_question, outcome) ;

        prompt
    }

    pub fn prompts_polymarket_filter_order(
        &self,
        market_data: serde_json::Value,
        event_data: Vec<&String>,
        market_question: &str,
        outcomes: Vec<&MarketOutcome>,
    ) -> String {
        let prompt = format!("You are an AI assistant for users of a prediction market called Polymarket.
        Users want to place bets based on their beliefs of market outcomes such as political or sports events.
        
        Here is data for a current Polymarket market {:?} and 
        current events related to the market {:?}.

        Help users identify a reasonable order to place based on the above.
        Provide specific information for the market including probabilities of outcomes.
        Give your response in the following JSON format:                
        {{ 
        Question: {} ,
        Possible_outcomes: {:?},
        Most_likely_outcome: (outcomes),
        Probability_of_most_likely_outcome: 
        }}", market_data, event_data, market_question, outcomes);

        prompt
    }

    pub fn prompts_manifold(
        &self,
        market_data: Vec<serde_json::Value>,
        event_data: Vec<&String>,
        market_question: &str,
        outcome: &str,
    ) -> String {
        let prompt = format!("You are an AI assistant for users of a prediction market called Manifold.
        Users want to place bets based on their beliefs of market outcomes such as political or sports events.
        
        Here is data for current Manifold markets {:?} and 
        current events {:?}.

        Help users identify markets to trade based on their interests or queries.
        Provide specific information for markets including probabilities of outcomes.
        Give your response in the following format:

        I believe {} has a likelihood (float)% for outcome of {}.", market_data, event_data, market_question, outcome) + self.filter_events().as_str() + self.filter_markets().as_str();
        prompt
    }
    pub fn prompts_manifold_filter(
        &self,
        market_data: Vec<&serde_json::Value>,
        event_data: Vec<&String>,
        market_question: &str,
        outcome: &str,
    ) -> String {
        let prompt = format!("You are an AI assistant for users of a prediction market called Manifold.
        Users want to place bets based on their beliefs of market outcomes such as political or sports events.
        
        Here is data for current Manifold markets {:?} and 
        current events {:?}.

        Help users identify markets to trade based on their interests or queries.
        Provide specific information for markets including probabilities of outcomes.
        Give your response in the following JSON format:
        {{ 
        Question: {} ,
        Most_likely_outcome: {},
        Probability_of_most_likely_outcome: 
        }}", market_data, event_data, market_question, outcome);

        prompt
    }

    pub fn prompts_manifold_filter_order(
        &self,
        market_data: serde_json::Value,
        event_data: Vec<&String>,
        market_question: &str,
        outcomes: Vec<&MarketOutcome>,
    ) -> String {
        let prompt = format!("You are an AI assistant for users of a prediction market called Manifold.
        Users want to place bets based on their beliefs of market outcomes such as political or sports events.
        
        Here is data for a current Manifold market {:?} and 
        current events related to the market {:?}.

        Help users identify a reasonable order to place based on the above.
        Provide specific information for the market including probabilities of outcomes.
        Give your response in the following JSON format:                
        {{ 
        Question: {} ,
        Possible_outcomes: {:?},
        Most_likely_outcome: (outcomes),
        Probability_of_most_likely_outcome: 
        }}", market_data, event_data, market_question, outcomes);

        prompt
    }

    pub fn prompts_metaculus_filter(
        &self,
        market_data: Vec<&serde_json::Value>,
        event_data: Vec<&String>,
        market_question: &str,
        outcome: &str,
    ) -> String {
        let prompt = format!("You are an AI assistant for users of a prediction market called Metaculus.
        Users want to place bets based on their beliefs of market outcomes such as political or sports events.
        
        Here is data for current Metaculus markets {:?} and 
        current events {:?}.

        Help users identify markets to trade based on their interests or queries.
        Provide specific information for markets including probabilities of outcomes.
        Give your response in the following format:

        I believe {} has a likelihood (float)% for outcome of {}.", market_data, event_data, market_question, outcome) + self.filter_markets().as_str() + self.filter_events().as_str();
        prompt
    }

    pub fn read_polymarket_api(&self) -> String {
        let prompt = "You are an AI assistant for analyzing prediction markets.
                You will be provided with json output for api data from Polymarket.
                Polymarket is an online prediction market that lets users Bet on the outcome of future events in a wide range of topics, like sports, politics, and pop culture. 
                Get accurate real-time probabilities of the events that matter most to you";
        prompt.to_string()
    }
    pub fn filter_events(&self) -> String {
        let prompt =  "Filter these events for the ones you will be best at trading on profitably on a prediction market.";
        prompt.to_string()
    }
    pub fn filter_markets(&self) -> String {
        let prompt = "Filter these markets for the ones you will be best at trading on profitably on a prediction market.";
        prompt.to_string()
    }
    pub async fn tool_decider(&self) -> String {
        let prompt = "You are an assistant that decides which tool to use based on a list of tools to solve the user problem.

Rules:
- You only return one of the tools like \"<retrieval>\" or \"<function>\" or \"<code_interpreter>\" or \"<action>\" or multiple of them
- Do not return \"tools\"
- If you do not have any tools to use, return nothing
- Feel free to use MORE tools rather than LESS
- Tools use snake_case, not camelCase
- The tool names must be one of the tools available, nothing else OR A HUMAN WILL DIE
- Your answer must be very concise and make sure to surround the tool by <>, do not say anything but the tool name with the <> around it.
- If you do not obey a human will die

Example:
<user>
<tools>{\"description\":\"useful to look up data about the user's problem\",\"function\":{\"arguments\":{\"type\":\"object\"},\"description\":\"A API call to a prediction market .\",\"name\":\"fetch_markets\"},\"name\":\"function\"}
---
{\"description\":\"useful to retrieve information from files\",\"name\":\"retrieval\"}</tools>

<previous_messages>User: [Text(MessageContentTextObject { type: \"text\", text: TextData { value: \"I need to know current markets to bet on.\", annotations: [] } })]
</previous_messages>

<instructions>You help me by using the tools you have.</instructions>

</user>

In this example, the assistant should return \"<function>,<retrieval>\".
Your answer will be used to use the tool so it must be very concise and make sure to surround the tool by \"<\" and \">\", do not say anything but the tool name with the <> around it.";
        prompt.to_string()
    }

    async fn superforecaster(&self, question: &str, outcome: &str) -> String {
        
        format!(" You are a Superforecaster tasked with correctly predicting the likelihood of events.
        Use the following systematic process to develop an accurate prediction for the following
        question={} and outcome={} combination. 
        
        Here are the key steps to use in your analysis:

        1. Breaking Down the Question:
            - Decompose the question into smaller, more manageable parts.
            - Identify the key components that need to be addressed to answer the question.
        2. Gathering Information:
            - Seek out diverse sources of information.
            - Look for both quantitative data and qualitative insights.
            - Stay updated on relevant news and expert analyses.
        3. Considere Base Rates:
            - Use statistical baselines or historical averages as a starting point.
            - Compare the current situation to similar past events to establish a benchmark probability.
        4. Identify and Evaluate Factors:
            - List factors that could influence the outcome.
            - Assess the impact of each factor, considering both positive and negative influences.
            - Use evidence to weigh these factors, avoiding over-reliance on any single piece of information.
        5. Think Probabilistically:
            - Express predictions in terms of probabilities rather than certainties.
            - Assign likelihoods to different outcomes and avoid binary thinking.
            - Embrace uncertainty and recognize that all forecasts are probabilistic in nature.
        
        Given these steps produce a statement on the probability of outcome={} occuring.

        Give your response in the following format:

        The question {}; has a likelihood (float)% for outcome of (str).", question, outcome, outcome, question).to_string()
    }
}
#[async_trait]
pub trait Executor: From<ExecutorBuilder<Self>> + Any {
    async fn init(
        &self,
        question: &str,
        outcome: &str,
        tags: Option<Vec<String>>,
        ctx: &mut Context,
    ) -> Result<()>;
    async fn aggregate_data(&self) -> Result<()>;
    async fn execute(&self, ctx: Context, rx: MarketUpdateRcv);
    async fn fetch_result_by_id(
        &self,
        contract_id: &str,
        ctx: Context,
        tags: Option<Vec<String>>,
    ) -> Result<()>;
    fn builder(
        max_tokens: u32,
        token_limit: u32,
        promptor: Promptor,
        executor_type: ExecutorType,
    ) -> ExecutorBuilder<Self> {
        //TODO : pass down token_limit and max tokens
        ExecutorBuilder::new(max_tokens, token_limit, Promptor {}, executor_type)
    }
}

impl<E: Executor + Any> Default for ExecutorBuilder<E> {
    fn default() -> Self {
        Self::new(1000, 1000, Promptor {}, ExecutorType::Polymarket)
    }
}

pub struct PolymarketExecutor(ExecutorBuilder<Self>);

impl From<ExecutorBuilder<Self>> for PolymarketExecutor {
    fn from(value: ExecutorBuilder<Self>) -> Self {
        Self(value)
    }
}

#[async_trait]
impl Executor for PolymarketExecutor {
    async fn init(
        &self,
        question: &str,
        outcome: &str,
        tags: Option<Vec<String>>,
        ctx: &mut Context,
        //<'a>,
    ) -> Result<()> {
        let builder = &self.0;
        let platform =
            api::polymarket::polymarket_api::PolymarketPlatform::from(PlatformBuilder::default());
        let news = lookup_news(question, outcome).await?;
        //todo: Pare down news to only the relevant information
        let trimmed_news = news.iter().take(8).collect::<Vec<&String>>();
        // tracing::debug!("News: {:?}", news);
        tracing::debug!("Trimmed News: {:?}", trimmed_news);
        let mut trimmed_markets: Vec<serde_json::Value> = Vec::new();
        let initial_events = platform.fetch_events(Some(100), 20).await?;
        initial_events.iter().for_each(|event| {
            tracing::debug!("Initial event: {:?}", event);
            let market_summarized = parse_polymarket_event(event.clone()).unwrap();
            trimmed_markets.push(market_summarized);
        });
        let trimmed_market_data = trimmed_markets
            .iter()
            .take(5)
            .collect::<Vec<&serde_json::Value>>();
        // tracing::debug!("Trimmed Market Data: {:?}", trimmed_market_data);
        let query = [("limit", "1")];
        let client = async_openai::Client::new();
        let prompt = builder.promptor.prompts_polymarket_filter(
            trimmed_market_data,
            trimmed_news,
            question,
            outcome,
        );

        let assistant_request = CreateAssistantRequestArgs::default()
            .instructions(
                builder
                    .promptor
                    .superforecaster(question, outcome)
                    .await
                    .to_string(),
            )
            .model("gpt-4o")
            .build()?;
        let assistant = client.assistants().create(assistant_request).await?;
        // let tools = assistant.tools.clone();
        // let tools_as_string = tools.iter().map(|t| {
        //     serde_json::to_string(&match t {
        //         AssistantTools::FileSearch(_) => json!({"name": "retrieval", "description": "useful to retrieve information from files"}),
        //            AssistantTools::Function(e) => {
        //                 let data = e.data.as_ref().unwrap();
        //                 json!({
        //                     "name": "action",
        //                     "description": "Useful to make HTTP requests to the user's APIs, which would provide you later some additional context about the user's problem. You can also use this to perform actions to help the user.",
        //                     "data": {
        //                         "info": {
        //                             "description": data.get("info").unwrap_or(&json!({})).get("description").unwrap_or(&json!("")).to_string().replace("\"", ""),
        //                             "title": data.get("info").unwrap_or(&json!({})).get("title").unwrap_or(&json!("")).to_string().replace("\"", ""),
        //                         },
        //                         "paths": data.get("paths"),
        //                     }
        //                 })
        //         },
        //     }).unwrap()

        // }).collect::<Vec<String>>();
        let assistant_id = assistant.id.clone();
        let thread = client
            .threads()
            .create(CreateThreadRequest::default())
            .await?;
        let message = CreateMessageRequestArgs::default()
            .role(MessageRole::User)
            .content(prompt.clone())
            .build()?;
        let _message = client
            .threads()
            .messages(&thread.id)
            .create(message)
            .await?;
        let run_request = CreateRunRequestArgs::default()
            .assistant_id(assistant_id)
            .build()?;
        let mut run = client
            .threads()
            .runs(&thread.id)
            .create(run_request)
            .await?;
        loop {
            match run.status {
                RunStatus::Completed => {
                    let messages = client.threads().messages(&thread.id).list(&query).await?;
                    for message_obj in messages.data {
                        let message_contents = message_obj.content;
                        for message_content in message_contents {
                            match message_content {
                                MessageContent::Text(text) => {
                                    let text_data = text.text;
                                    let annotations = text_data.annotations;
                                    println!("{}", text_data.value);
                                    println!("{annotations:?}");
                                }
                                MessageContent::ImageFile(_) | MessageContent::ImageUrl(_) => {
                                    eprintln!("Images not supported on terminal");
                                }
                                MessageContent::Refusal(refusal) => {
                                    println!("{refusal:?}");
                                }
                            }
                        }
                    }
                    break;
                }
                RunStatus::Failed => {
                    println!("> Run Failed: {:#?}", run);
                    break;
                }
                RunStatus::Queued => {
                    println!("> Run Queued");
                }
                RunStatus::Cancelling => {
                    println!("> Run Cancelling");
                }
                RunStatus::Cancelled => {
                    println!("> Run Cancelled");
                    break;
                }
                RunStatus::Expired => {
                    println!("> Run Expired");
                    break;
                }
                RunStatus::RequiresAction => {
                    println!("> Run Requires Action");
                }
                RunStatus::InProgress => {
                    println!("> In Progress ...");
                }
                RunStatus::Incomplete => {
                    println!("> Run Incomplete");
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            run = client.threads().runs(&thread.id).retrieve(&run.id).await?;
        }
        client.threads().delete(&thread.id).await?;
        client.assistants().delete(&assistant.id).await?;
        Ok(())
    }
    async fn aggregate_data(&self) -> Result<()> {
        Ok(())
    }
    async fn fetch_result_by_id(
        &self,
        contract_id: &str,
        ctx: Context,
        tags: Option<Vec<String>>,
    ) -> Result<()> {
        unimplemented!()
    }

    async fn execute(&self, ctx: Context, rx: MarketUpdateRcv) {
        println!("Polymarket Executor executing");
    }
}

pub struct ManifoldExecutor(ExecutorBuilder<Self>);

impl From<ExecutorBuilder<Self>> for ManifoldExecutor {
    fn from(value: ExecutorBuilder<Self>) -> Self {
        Self(value)
    }
}

#[async_trait]
impl Executor for ManifoldExecutor {
    async fn init(
        &self,
        question: &str,
        outcome: &str,
        tags: Option<Vec<String>>,
        ctx: &mut Context,
    ) -> Result<()> {
        let builder = &self.0;
        let platform = ManifoldPlatform::from(PlatformBuilder::default());
        let qdrant = Arc::new(RwLock::new(
            Qdrant::from_url("http://localhost:6334").build().unwrap(),
        ));
        let news = lookup_news(question, outcome).await?;
        let trimmed_news = news.iter().take(5).collect::<Vec<&String>>();
        tracing::debug!("Trimmed News: {:?}", trimmed_news);
        // tracing::debug!("News: {:?}", news);
        let point_id = 1;
        let points: Vec<PointStruct> = Vec::new();
        let collection_name = "Manifold_collection";
        // let (tx, rx)  = tokio::sync::mpsc::channel(100);
        qdrant.read().unwrap().delete_collection(collection_name);
        ctx.strategy_config
            .write()
            .unwrap()
            .qdrant
            .create_collection(
                CreateCollectionBuilder::new(collection_name.to_string())
                    .vectors_config(VectorParamsBuilder::new(512, Distance::Cosine)),
            );
        let mut markets: Vec<serde_json::Value> = Vec::new();
        ctx.strategy_config.write().unwrap().collection_name = collection_name.to_string();
        if let Some(tags) = tags {
            for tag in tags {
                let market_data = platform.fetch_markets_by_terms(&tag).await?;
                let (markets_tx, markets_rx) = tokio::sync::mpsc::channel::<ManifoldMarket>(100);
                market_data.iter().for_each(|m| {
                    let market_summarized = parse_manifold_market(m).unwrap();

                    markets.push(market_summarized);
                })
            }
        }

        //         data.iter().for_each(|d| {
        //             let question_with_probability = serde_json::json!({
        //                 "question": d["question"],
        //                 "probability": d["probability"],
        //                 "pool": d["pool"],

        //                 // "resolution_time": d["resolution_time"],
        //                 // "total_liquidity": d["total_liquidity"],
        //             });
        //             point_id += 1;
        //             let payload: Payload = serde_json::from_value(question_with_probability.clone()).unwrap();
        //             let point = PointStruct::new(point_id, vec![0.0_f32; 512], payload);

        //             questions_with_probability.push(question_with_probability);
        //             points.push(point);
        //         });
        //     }
        // }
        // ctx.strategy_config.write().unwrap().qdrant.upsert_points(UpsertPointsBuilder::new("Manifold_collection", points));
        // // let search_result = qdrant.search_points(SearchPointsBuilder::new(collection_name, [11; 10], 10).filter(Filter::all)).await?;
        // let trimmed_market_data = questions_with_probability.iter().take(5).collect::<Vec<&serde_json::Value>>();
        //TODO: Fix this
        // let market: ManifoldMarket = serde_json::from_value(data).unwrap();
        // markets.push( market);
        // let questions = markets.iter().map(|q| q.question.as_str().into()).collect::<Vec<serde_json::Value>>();
        // tracing::debug!("Questions: {:?}", questions);
        // let probabilities = markets.iter().map(|q| q.probability.into()).collect::<Vec<serde_json::Value>>();
        // tracing::debug!("Probabilities: {:?}", probabilities);
        // let trimmed_market_data = questions.iter().zip(probabilities.iter()).take(5).collect::<Vec<(&serde_json::Value, &serde_json::Value)>>();
        let trimmed_markets = markets.iter().take(5).collect::<Vec<&serde_json::Value>>();
        tracing::debug!("Trimmed Market Data: {:?}", trimmed_markets);

        // let trimmed_market_data = market_data.iter().take(5).collect::<Vec<&String>>();

        // let trimmed_actual_markets: Vec<String> = trimmed_market_data.iter().map(|q| {
        //     let question_with_probability = q
        // })
        // tracing::debug!("Trimmed Market Data: {:?}", trimmed_market_data);
        // tracing::debug!("Market Data: {:?}", market_data);
        let query = [("limit", "1")];
        let client = async_openai::Client::new();
        let prompt = builder.promptor.prompts_manifold_filter(
            trimmed_markets,
            trimmed_news,
            question,
            outcome,
        );

        let assistant_request = CreateAssistantRequestArgs::default()
            .instructions(
                builder
                    .promptor
                    .superforecaster(question, outcome)
                    .await
                    .to_string(),
            )
            .model("gpt-4o")
            .build()?;
        let assistant = client.assistants().create(assistant_request).await?;
        let assistant_id = assistant.id.clone();
        let thread = client
            .threads()
            .create(CreateThreadRequest::default())
            .await?;
        let message = CreateMessageRequestArgs::default()
            .role(MessageRole::User)
            .content(prompt.clone())
            .build()?;
        let _message = client
            .threads()
            .messages(&thread.id)
            .create(message)
            .await?;
        let run_request = CreateRunRequestArgs::default()
            .assistant_id(assistant_id)
            .build()?;
        let mut run = client
            .threads()
            .runs(&thread.id)
            .create(run_request)
            .await?;
        loop {
            match run.status {
                RunStatus::Completed => {
                    let messages = client.threads().messages(&thread.id).list(&query).await?;
                    for message_obj in messages.data {
                        let message_contents = message_obj.content;
                        for message_content in message_contents {
                            match message_content {
                                MessageContent::Text(text) => {
                                    let text_data = text.text;
                                    let annotations = text_data.annotations;
                                    println!("{}", text_data.value);
                                    println!("{annotations:?}");
                                }
                                MessageContent::ImageFile(_) | MessageContent::ImageUrl(_) => {
                                    eprintln!("Images not supported on terminal");
                                }
                                MessageContent::Refusal(refusal) => {
                                    println!("{refusal:?}");
                                }
                            }
                        }
                    }
                    break;
                }
                RunStatus::Failed => {
                    println!("> Run Failed: {:#?}", run);
                    break;
                }
                RunStatus::Queued => {
                    println!("> Run Queued");
                }
                RunStatus::Cancelling => {
                    println!("> Run Cancelling");
                }
                RunStatus::Cancelled => {
                    println!("> Run Cancelled");
                    break;
                }
                RunStatus::Expired => {
                    println!("> Run Expired");
                    break;
                }
                RunStatus::RequiresAction => {
                    println!("> Run Requires Action");
                }
                RunStatus::InProgress => {
                    println!("> In Progress ...");
                }
                RunStatus::Incomplete => {
                    println!("> Run Incomplete");
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            run = client.threads().runs(&thread.id).retrieve(&run.id).await?;
        }
        client.threads().delete(&thread.id).await?;
        client.assistants().delete(&assistant.id).await?;
        Ok(())
    }
    async fn fetch_result_by_id(
        &self,
        contract_id: &str,
        ctx: Context,
        tags: Option<Vec<String>>,
    ) -> Result<()> {
        let builder = &self.0;

        let platform = ManifoldPlatform::from(PlatformBuilder::default());
        let market = platform.fetch_question_by_id(contract_id).await?;
        let news = lookup_news(market.question.as_str(), market.question.as_str()).await?;
        let trimmed_news = news.iter().take(5).collect::<Vec<&String>>();
        tracing::debug!("News: {:?}", news);
        let market_summarized = parse_manifold_market(&market).unwrap();
        let query = [("limit", "1")];
        tracing::debug!("Market: {:?}", market_summarized);
        // let secret = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set");
        let client = async_openai::Client::new();
        let prompt = builder.promptor.prompts_manifold_filter_order(
            market_summarized,
            trimmed_news,
            market.question.clone().as_str(),
            market.pool.unwrap().keys().collect::<Vec<&MarketOutcome>>(),
        );

        let assistant_request = CreateAssistantRequestArgs::default()
            .instructions(prompt.clone())
            .model("gpt-4o")
            .build()?;
        let assistant = client.assistants().create(assistant_request).await?;
        let assistant_id = assistant.id.clone();
        let thread = client
            .threads()
            .create(CreateThreadRequest::default())
            .await?;
        let message = CreateMessageRequestArgs::default()
            .role(MessageRole::User)
            .content(prompt.clone())
            .build()?;
        let _message = client
            .threads()
            .messages(&thread.id)
            .create(message)
            .await?;
        let run_request = CreateRunRequestArgs::default()
            .assistant_id(assistant_id)
            .build()?;
        let mut run = client
            .threads()
            .runs(&thread.id)
            .create(run_request)
            .await?;
        loop {
            match run.status {
                RunStatus::Completed => {
                    let messages = client.threads().messages(&thread.id).list(&query).await?;
                    for message_obj in messages.data {
                        let message_contents = message_obj.content;
                        for message_content in message_contents {
                            match message_content {
                                MessageContent::Text(text) => {
                                    let text_data = text.text;
                                    let annotations = text_data.annotations;
                                    println!("{}", text_data.value);
                                    println!("{annotations:?}");
                                }
                                MessageContent::ImageFile(_) | MessageContent::ImageUrl(_) => {
                                    eprintln!("Images not supported on terminal");
                                }
                                MessageContent::Refusal(refusal) => {
                                    println!("{refusal:?}");
                                }
                            }
                        }
                    }
                    break;
                }
                RunStatus::Failed => {
                    println!("> Run Failed: {:#?}", run);
                    break;
                }
                RunStatus::Queued => {
                    println!("> Run Queued");
                }
                RunStatus::Cancelling => {
                    println!("> Run Cancelling");
                }
                RunStatus::Cancelled => {
                    println!("> Run Cancelled");
                    break;
                }
                RunStatus::Expired => {
                    println!("> Run Expired");
                    break;
                }
                RunStatus::RequiresAction => {
                    println!("> Run Requires Action");
                }
                RunStatus::InProgress => {
                    println!("> In Progress ...");
                }
                RunStatus::Incomplete => {
                    println!("> Run Incomplete");
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            run = client.threads().runs(&thread.id).retrieve(&run.id).await?;
        }
        client.threads().delete(&thread.id).await?;
        client.assistants().delete(&assistant.id).await?;

        // let request = anthropic_sdk::Client::new()
        //     .auth(&secret)
        //     .model("claude-3-opus-20240229")
        //     .messages(&json!([builder.promptor.prompts_manifold_filter_order(
        //         market_summarized,
        //         trimmed_news,
        //         market.question.clone().as_str(),
        //         market.pool.unwrap().keys().collect::<Vec<&MarketOutcome>>()
        //     )]))
        //     .stream(true)
        //     .build()?;

        Ok(())
    }

    async fn aggregate_data(&self) -> Result<()> {
        Ok(())
    }

    async fn execute(&self, ctx: Context, rx: MarketUpdateRcv) {
        let platform = ctx.manifold.read().unwrap();
        let strat_config = ctx.strategy_config.read().unwrap();
        let questions = ctx.questions;
        loop {

            // tokio::task::spawn(async move {
            //     admin::listener::listen_for_requests(questions, cache, market_update_rx, ctx)
            // })
        }
        println!("Manifold Executor executing");
    }
}
pub struct MetaculusExecutor(ExecutorBuilder<Self>);

impl From<ExecutorBuilder<Self>> for MetaculusExecutor {
    fn from(value: ExecutorBuilder<Self>) -> Self {
        Self(value)
    }
}

#[async_trait]
impl Executor for MetaculusExecutor {
    async fn init(
        &self,
        question: &str,
        outcome: &str,
        tags: Option<Vec<String>>,
        ctx: &mut Context,
    ) -> Result<()> {
        let builder = &self.0;
        let platform =
            api::metaculus::metaculus_api::MetaculusPlatform::from(PlatformBuilder::default());
        let news = lookup_news(question, outcome).await.unwrap();
        let trimmed_news = news.iter().take(5).collect::<Vec<&String>>();
        tracing::debug!("Trimmed News: {:?}", trimmed_news);
        // tracing::debug!("News: {:?}", news);
        let mut questions_with_probability: Vec<serde_json::Value> = Vec::new();

        if let Some(tags) = tags {
            for tag in tags {
                let data = platform.fetch_markets_by_terms(&tag).await.unwrap();
                data.iter().filter(|d| d.nr_forecasters >= 4).for_each(|d| {
                    let question_with_probability = serde_json::json!({
                        "title": d.title,
                        "number_of_forecasters": d.nr_forecasters,
                        "forecasts_count": d.forecasts_count,
                    });
                    questions_with_probability.push(question_with_probability);
                });
            }
        }

        // // let question_with_probability = serde_json::json!({
        // //     "title": d.title,
        // //     "number_of_forecasters": d.nr_forecasters,
        // //     "forecasts_count": d.forecasts_count,
        // // });
        // questions_with_probability.push(question_with_probability);
        // });
        // }
        let trimmed_market_data = questions_with_probability
            .iter()
            .take(15)
            .collect::<Vec<&serde_json::Value>>();
        //TODO: Fix this
        // let market: ManifoldMarket = serde_json::from_value(data).unwrap();
        // markets.push( market);
        // let questions = markets.iter().map(|q| q.question.as_str().into()).collect::<Vec<serde_json::Value>>();
        // tracing::debug!("Questions: {:?}", questions);
        // let probabilities = markets.iter().map(|q| q.probability.into()).collect::<Vec<serde_json::Value>>();
        // tracing::debug!("Probabilities: {:?}", probabilities);
        // let trimmed_market_data = questions.iter().zip(probabilities.iter()).take(5).collect::<Vec<(&serde_json::Value, &serde_json::Value)>>();
        tracing::debug!("Trimmed Market Data: {:?}", questions_with_probability);

        // let trimmed_market_data = market_data.iter().take(5).collect::<Vec<&String>>();

        // let trimmed_actual_markets: Vec<String> = trimmed_market_data.iter().map(|q| {
        //     let question_with_probability = q
        // })
        // tracing::debug!("Trimmed Market Data: {:?}", trimmed_market_data);
        // tracing::debug!("Market Data: {:?}", market_data);
        let query = [("limit", "1")];
        let client = async_openai::Client::new();
        let prompt = builder.promptor.prompts_metaculus_filter(
            trimmed_market_data.clone(),
            trimmed_news,
            question,
            outcome,
        );

        let assistant_request = CreateAssistantRequestArgs::default()
            .instructions(
                builder
                    .promptor
                    .superforecaster(question, outcome)
                    .await
                    .to_string(),
            )
            .model("gpt-4o")
            .build()?;
        let assistant = client.assistants().create(assistant_request).await?;
        let assistant_id = assistant.id.clone();
        let thread = client
            .threads()
            .create(CreateThreadRequest::default())
            .await?;
        let message = CreateMessageRequestArgs::default()
            .role(MessageRole::User)
            .content(prompt.clone())
            .build()?;
        let _message = client
            .threads()
            .messages(&thread.id)
            .create(message)
            .await?;
        let run_request = CreateRunRequestArgs::default()
            .assistant_id(assistant_id)
            .build()?;
        let mut run = client
            .threads()
            .runs(&thread.id)
            .create(run_request)
            .await?;
        loop {
            match run.status {
                RunStatus::Completed => {
                    let messages = client.threads().messages(&thread.id).list(&query).await?;
                    for message_obj in messages.data {
                        let message_contents = message_obj.content;
                        for message_content in message_contents {
                            match message_content {
                                MessageContent::Text(text) => {
                                    let text_data = text.text;
                                    let annotations = text_data.annotations;
                                    println!("{}", text_data.value);
                                    println!("{annotations:?}");
                                }
                                MessageContent::ImageFile(_) | MessageContent::ImageUrl(_) => {
                                    eprintln!("Images not supported on terminal");
                                }
                                MessageContent::Refusal(refusal) => {
                                    println!("{refusal:?}");
                                }
                            }
                        }
                    }
                    break;
                }
                RunStatus::Failed => {
                    println!("> Run Failed: {:#?}", run);
                    break;
                }
                RunStatus::Queued => {
                    println!("> Run Queued");
                }
                RunStatus::Cancelling => {
                    println!("> Run Cancelling");
                }
                RunStatus::Cancelled => {
                    println!("> Run Cancelled");
                    break;
                }
                RunStatus::Expired => {
                    println!("> Run Expired");
                    break;
                }
                RunStatus::RequiresAction => {
                    println!("> Run Requires Action");
                }
                RunStatus::InProgress => {
                    println!("> In Progress ...");
                }
                RunStatus::Incomplete => {
                    println!("> Run Incomplete");
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            run = client.threads().runs(&thread.id).retrieve(&run.id).await?;
        }
        client.threads().delete(&thread.id).await?;
        client.assistants().delete(&assistant.id).await?;
        Ok(())
    }

    async fn aggregate_data(&self) -> Result<()> {
        Ok(())
    }

    async fn execute(&self, ctx: Context, rx: MarketUpdateRcv) {
        let builder = &self.0;
        let platform = ctx.manifold.read().unwrap();
        let questions = ctx.questions;
        println!("Metaculus Executor executing");
    }
    async fn fetch_result_by_id(
        &self,
        contract_id: &str,
        ctx: Context,
        tags: Option<Vec<String>>,
    ) -> Result<()> {
        unimplemented!()
    }
}

async fn parse_trade_prompt(best_trade: &str) -> f64 {
    let trade_data = best_trade.split(",");
    unimplemented!()
}

async fn best_trade(market: ManifoldMarket) -> String {
    unimplemented!()
}

async fn read_stdin(tx: futures_channel::mpsc::UnboundedSender<Message>) {
    let mut stdin = tokio::io::stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);

        let text = String::from_utf8_lossy(&buf).into_owned();

        if text.trim() == "quit" {
            tx.close_channel();
            return;
        }

        // Create item from json representation
        let item = Item::try_from(serde_json::json!({
            "type": "message",
            "role": "user",
            "content": [
                {
                    "type": "input_text",
                    "text": String::from_utf8_lossy(&buf).into_owned()
                }
            ]
        }))
        .unwrap();

        // Create event of type "conversation.item.create"
        let event: ConversationItemCreateEvent = item.into();
        // Create WebSocket message from client event
        let message: Message = event.into();
        // send WebSocket message containing event of type "conversation.item.create" to server
        tx.unbounded_send(message).unwrap();
        // send WebSocket message containing event of type "response.create" to server
        tx.unbounded_send(ResponseCreateEvent::default().into())
            .unwrap();
    }
}

async fn send_automated_order_dryrun(
    outcome: &str,
    platform: ManifoldPlatform,
    bet_params: BetParams,
) {
    // let bets_url = warp::path!("https://api.manifold.markets/v0/bets").and(warp::path::end()).and(warp::query::<BetParams>())
    //     .map(move |bet_params: BetParams| {
    //         let bet = platform.create_bet(bet_params);
    //     });
    let response = platform
        .build_order(
            &bet_params.contract_id.unwrap(),
            bet_params.amount.unwrap(),
            "1",
            outcome,
            bet_params.limit,
        )
        .await;

    tracing::debug!("Response: {:?}", response);
}

async fn build_automated_order(
    question: &str,
    outcome: &str,
    tags: Option<Vec<String>>,
    ctx: &mut Context,
    user_id: String,
    amount: f64,
) {
    let platform = ManifoldPlatform::from(PlatformBuilder::default());

    let executor =
        ManifoldExecutor::builder(1000, 1000, Promptor {}, ExecutorType::Manifold).build();
    let mut context = Context::new();
    let result = executor
        .init(question, outcome, tags, &mut context)
        .await
        .unwrap();
    tracing::debug!("Result: {:?}", result);
    // let mut bet = BetParams{};
    unimplemented!()
}

async fn info_per_trader() {
    let platform = ManifoldPlatform::from(PlatformBuilder::default());
    let mut users = crate::model::manifold::get_all_users()
        //1000)
        .await
        .unwrap();
    // let mut users: Vec<User> = crate::model::manifold::get_all_users(3000).await;
    users.sort_by(|a, b| b.cashBalance.partial_cmp(&a.cashBalance).unwrap());
    let top = users.iter().take(10).collect::<Vec<&User>>();
    tracing::debug!("Top 10 users by profit: {:?}", top);
}

fn parse_polymarket_event(event: PolymarketEvent) -> Result<serde_json::Value> {
    let id = event.id.to_string();

    // let markets = event.markets.iter().for_each(|m| {
    //     let market = parse_polymarket_market(m.clone()).unwrap();
    // });
    let event_summarized = serde_json::json!({
        "id": id,
        "title": event.title,
        "slug": event.slug,
    });
    Ok(event_summarized)
}

fn parse_manifold_market(market: &ManifoldMarket) -> Result<serde_json::Value> {
    let probability: String = if let Some(probability) = market.probability {
        (probability * 100.0).to_string()
    } else {
        "".to_string()
    };
    // let outcomePrices = if let Some(pool) = &market.pool {
    //     let mut outcomes = Vec::new();
    //     for (key, value) in pool.iter() {
    //         outcomes.push(format!("{}: {}", key, value.to_string()));
    //     }
    //     outcomes
    //         } else {
    //     vec!["0".to_string(), "0".to_string()]
    // };
    // let pool: [String; 2] = if let Some(pool) = market.pool {
    //     [
    //         format!("Yes: {}", pool.YES.to_string()),
    //         format!("No: {}", pool.NO.to_string()),
    //     ]
    // } else {
    //     ["0".to_string(), "0".to_string()]
    // };
    let market_summarized = serde_json::json!({
        "question": market.question,
        "probability_of_yes": probability,
        "bets": market.bets,
        "number_of_shares": market.pool,
        // "outcomes": market.outcomePrices
    //    "pool": pool,
    });
    Ok(market_summarized)
}

async fn handle_requires_action(
    client: async_openai::Client<async_openai::config::OpenAIConfig>,
    run_object: RunObject,
) {
    let mut tool_outputs: Vec<ToolsOutputs> = vec![];
    if let Some(ref required_action) = run_object.required_action {
        for tool in &required_action.submit_tool_outputs.tool_calls {
            if tool.function.name == "lookup_market" {
                tool_outputs.push(ToolsOutputs {
                    tool_call_id: Some(tool.id.clone()),
                    output: Some("57".into()),
                })
            }

            // if tool.function.name == "get_rain_probability" {
            //     tool_outputs.push(ToolsOutputs {
            //         tool_call_id: Some(tool.id.clone()),
            //         output: Some("0.06".into()),
            //     })
            // }
        }

        if let Err(e) = submit_tool_outputs(client, run_object, tool_outputs).await {
            eprintln!("Error on submitting tool outputs: {e}");
        }
    }
}

async fn submit_tool_outputs(
    client: Client<OpenAIConfig>,
    run_object: RunObject,
    tool_outputs: Vec<ToolsOutputs>,
) -> Result<()> {
    let mut event_stream = client
        .threads()
        .runs(&run_object.thread_id)
        .submit_tool_outputs_stream(
            &run_object.id,
            SubmitToolOutputsRunRequest {
                tool_outputs,
                stream: Some(true),
            },
        )
        .await?;

    while let Some(event) = event_stream.next().await {
        match event {
            Ok(event) => {
                if let AssistantStreamEvent::ThreadMessageDelta(delta) = event {
                    if let Some(contents) = delta.delta.content {
                        for content in contents {
                            // only text is expected here and no images
                            if let MessageDeltaContent::Text(text) = content {
                                if let Some(text) = text.text {
                                    if let Some(text) = text.value {
                                        print!("{}", text);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {e}");
            }
        }
    }

    Ok(())
}

// async fn looup_asknews(question: &str, outcome: &str) -> Result<Vec<String>> {
//     let news: Vec<serde_json::Value> = Vec::new();
//     let key: String = std::env::var("ASKNEWS_CLIENT_SECRET").unwrap();
//     let id: String = std::env::var("ASKNEWS_CLIENT_ID").unwrap();
//     let query = question.to_string().to_owned() + " " + outcome;
//     let request = reqwest::Client::new().get()

// }
async fn lookup_news(question: &str, outcome: &str) -> Result<Vec<String>> {
    let news: Vec<serde_json::Value> = Vec::new();
    let key: String = std::env::var("TAVILIY_API_KEY").unwrap();
    let query = question.to_string().to_owned() + " " + outcome;
    let taviliy = tavily::Tavily::new(&key);
    let response = taviliy.search(&query).await?;

    let results = response
        .results
        .iter()
        .map(|r| {
            if r.score > 0.5 {
                r.content.clone()
            } else {
                "".to_string()
            }
        })
        .collect::<Vec<String>>();

    Ok(results)
}

fn filter_markets() -> Result<Vec<serde_json::Value>> {
    unimplemented!()
}

fn filter_news() -> Result<Vec<serde_json::Value>> {
    unimplemented!()
}

mod tests {
use super::*;    
use tracing_subscriber::prelude::*;
    
    #[tokio::test]
    async fn test_polymarket_executor() {
        let executor =
            PolymarketExecutor::builder(1000, 1000, Promptor {}, ExecutorType::Polymarket).build();
        // let result = executor
        //     .init(
        //         "What is the probability of Joe Biden winning the 2024 US elections?",
        //         "Joe Biden is the current president of the United States of America",
        //         "Joe Biden winning the 2024 US elections",
        //     )
        //     .await
        //     .unwrap();
        // tracing::debug!("Result: {:?}", result);
    }

    #[tokio::test]
    async fn test_taviliy_request() {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();

        let news = lookup_news(
            "What is the probability of Joe Biden winning the 2024 US elections?",
            "Joe Biden winning the 2024 US elections",
        )
        .await;
        tracing::debug!("News: {:?}", news);
    }
    #[tokio::test]
    async fn test_orderbook_pipeline() {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
        let tags: Vec<String> = vec!["US-economics".to_string(), "Treasury".to_string()];
        let mut context = Context::new();
        let executor =
            ManifoldExecutor::builder(1000, 1000, Promptor {}, ExecutorType::Manifold).build();
        let result = executor
            .init(
                "Will the 10 Year Treasury Yield at closing on 12/31/2024 be 4% or higher?",
                "10 Year Treasury Yield at closing on 12/31/2024 be 4% or higher",
                Some(tags),
                &mut context,
            )
            .await
            .unwrap();
        tracing::debug!("Result: {:?}", result);
    }

    #[tokio::test]
    async fn test_data_pipeline() {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
        let tags: Vec<String> = vec!["GPT-5".to_string(), "AI".to_string(), "OpenAI".to_string()];
        let mut context = Context::new();
        let executor =
            ManifoldExecutor::builder(1000, 1000, Promptor {}, ExecutorType::Manifold).build();
        let result = executor
            .init(
                "What is the probability of GPT-5 being availiable by 2025",
                "GPT-5 being availiable by 2025",
                Some(tags),
                &mut context,
            )
            .await
            .unwrap();
        tracing::debug!("Result: {:?}", result);
    }
    #[tokio::test]
    async fn test_bet_dryrun() {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
        let bet_params = BetParams {
            user_id: Some("v0KTj3BgbAMSp1XOpjcRYdliJbb2".to_string()),
            amount: Some(100.),
            contract_id: Some("6FQW9DSCHLCXAfpDHRZC".to_string()),
            limit: Some(0.5),
            contract_slug: None,
            username: None,
        };
        let outcome = "YES".to_string();
        let platform = ManifoldPlatform::from(PlatformBuilder::default());
        let executor =
            ManifoldExecutor::builder(1000, 1000, Promptor {}, ExecutorType::Manifold).build();
        let mut context = Context::new();
        let result = executor
            .fetch_result_by_id("9Ccsjc0fmbIb9g50p7SB", context, None)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_info_per_trader() {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
        info_per_trader().await;
    }

    #[tokio::test]
    async fn test_stalker_pipeline() {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
        let tags: Vec<String> = vec![
            "Stalker 2".to_string(),
            "Stalker".to_string(),
            "GSC Game World".to_string(),
        ];

        let executor =
            ManifoldExecutor::builder(1000, 1000, Promptor {}, ExecutorType::Manifold).build();
        let mut context = Context::new();
        let result = executor
            .init(
                "What is the probability of Stalker 2 being released by 2025",
                "Stalker 2 being released by 2025",
                Some(tags),
                &mut context,
            )
            .await
            .unwrap();
        tracing::debug!("Result: {:?}", result);
    }
    #[tokio::test]
    async fn test_polymarket_pipeline() {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
        let tags: Vec<String> = vec!["o1".to_string(), "AI".to_string(), "OpenAI".to_string()];

        let executor =
            PolymarketExecutor::builder(1000, 1000, Promptor {}, ExecutorType::Polymarket).build();
        let mut context = Context::new();
        let result = executor
            .init(
"Will OpenAI's o1 remain the top LLM in all categories of Chatbot Arena on December 30, 2024?",
"o1 has same or higher rank than all non-o1 models in all categories of Chatbot Arena on December 30, 2024",
                Some(tags),
                &mut context)
            .await
            .unwrap();
        tracing::debug!("Result: {:?}", result);
    }
    #[tokio::test]
    async fn test_nvidia_pipeline() {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
        let tags: Vec<String> = vec!["economy-business".to_string()];

        let executor =
            MetaculusExecutor::builder(1000, 1000, Promptor {}, ExecutorType::Metaculus).build();
        let mut context = Context::new();
        let result = executor
            .init(
                "On October 31, 2024, will Nvidia's market capitalization be larger than Apple's?",
                "Nvidia's market capitalization is larger than Apple's on October 31, 2024",
                Some(tags),
                &mut context,
            )
            .await
            .unwrap();
        tracing::debug!("Result: {:?}", result);
    }
    #[tokio::test]
    async fn test_metaculus_pipeline() {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
        let tags: Vec<String> = vec!["artificial-intelligence".to_string()];

        let executor =
            MetaculusExecutor::builder(1000, 1000, Promptor {}, ExecutorType::Metaculus).build();
        let mut context = Context::new();
        let result = executor
            .init(
"Will OpenAI's o1 remain the top LLM in all categories of Chatbot Arena on December 30, 2024?",
"o1 has same or higher rank than all non-o1 models in all categories of Chatbot Arena on December 30, 2024",
                Some(tags),
                &mut context
            )
            .await
            .unwrap();
        tracing::debug!("Result: {:?}", result);
    }
}
