use crate::api::{self, *};
use crate::manifold::ManifoldMarket;
use async_openai::types::realtime::{ConversationItemCreateEvent, Item, ResponseCreateEvent};
use async_openai::types::{CreateMessageRequestArgs, CreateRunRequestArgs};
use async_openai::{
    config::OpenAIConfig,
    types::{
        AssistantStreamEvent, CreateAssistantRequestArgs, CreateMessageRequest, CreateRunRequest,
        CreateThreadRequest, FunctionObject, MessageContent, MessageDeltaContent, MessageRole,
        RunObject, RunStatus, SubmitToolOutputsRunRequest, ToolsOutputs,
    },
    Client,
};
use axum::async_trait;
use futures_util::StreamExt;
use std::any::Any;
use tokio::io::AsyncReadExt;
use tokio_tungstenite::tungstenite::protocol::Message;

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
    async fn prompts_polymarket(
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
    async fn prompts_manifold(
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

        I believe {} has a likelihood (float)% for outcome of {}.", market_data, event_data, market_question, outcome);
        prompt
    }

    async fn read_polymarket_api(&self) -> String {
        let prompt = "You are an AI assistant for analyzing prediction markets.
                You will be provided with json output for api data from Polymarket.
                Polymarket is an online prediction market that lets users Bet on the outcome of future events in a wide range of topics, like sports, politics, and pop culture. 
                Get accurate real-time probabilities of the events that matter most to you";
        prompt.to_string()
    }
    async fn filter_events(&self) -> String {
        let prompt = self.read_polymarket_api().await.to_string()
            + "Filter these events for the ones you will be best at trading on profitably on a prediction market.";
        prompt.to_string()
    }
    async fn filter_markets(&self) -> String {
        let prompt = self.read_polymarket_api().await.to_string()
            + "Filter these markets for the ones you will be best at trading on profitably on a prediction market.";
        prompt.to_string()
    }

    async fn superforecaster(&self, question: &str, outcome: &str) -> String {
        let prompt = self.read_polymarket_api().await.to_string() +  format!(" You are a Superforecaster tasked with correctly predicting the likelihood of events.
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

        The question {}; has a likelihood (float)% for outcome of (str).", question, outcome, outcome, question).as_str();
        prompt.to_string()
    }
}
#[async_trait]
pub trait Executor: From<ExecutorBuilder<Self>> + Any {
    async fn init(&self, question: &str, outcome: &str, tags: Option<Vec<String>>) -> Result<()>;
    async fn execute(&self);
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
    async fn init(&self, question: &str, outcome: &str, tags: Option<Vec<String>>) -> Result<()> {
        let builder = &self.0;
        let platform = api::polymarket::PolymarketPlatform::from(PlatformBuilder::default());
        let news = lookup_news(question, outcome).await?;
        //todo: Pare down news to only the relevant information
        let trimmed_news = news.iter().take(5).collect::<Vec<&String>>();

        // tracing::debug!("News: {:?}", news);
        tracing::debug!("Trimmed News: {:?}", trimmed_news);
        let mut market_data = Vec::new();
        if let Some(tags) = tags {
            for tag in tags {
                let data = platform.fetch_json_by_description(&tag).await.unwrap();
                market_data.push(data);
            }
        }
        ////This is UNHOLY
        //let questions = market_data.iter().map(|q| q[r#"\question\"#].as_str().into()).collect::<Vec<serde_json::Value>>();
        //let probabilities = market_data.iter().map(|q| q[r#"\probability\"#].as_str().into()).collect::<Vec<serde_json::Value>>();
        // let trimmed_market_data = questions.iter().zip(probabilities.iter()).take(5).collect::<Vec<(&serde_json::Value, &serde_json::Value)>>();
        // tracing::debug!("Trimmed Market Data: {:?}", trimmed_market_data);
        // tracing::debug!("Market Data: {:?}", market_data);
        let events = platform.fetch_events(Some(5), 1).await.unwrap();
        // let event_data = events
        //     .iter()
        //     .map(|event| event.to_string())
        //     .collect::<Vec<String>>();

        // let events = platform.fetch_events(Some(5), 1).await.unwrap();
        // let event_data = serde_json::to_string(&events).unwrap();
        let client = async_openai::Client::new();
        let prompt = builder
            .promptor
            .prompts_polymarket(vec!["".into()], trimmed_news, question, outcome)
            .await;
        // let thread_request = async_openai::types::CreateThreadRequestArgs::default().build()?;
        // let thread = client.threads().create(thread_request.clone()).await?;

        let assistant_request = CreateAssistantRequestArgs::default().instructions(builder.promptor.superforecaster(question, outcome).await.to_string()).model("gpt-4o")
            .tools(vec![
                   FunctionObject {
                       name: "lookup_market".into(),
                       description: Some("Lookup a specific market and associated data on Polymarket".into()),
                       parameters: Some(serde_json::json!(
                               {
                                   "type": "object",
                                   "properties": {
                                       "question": {
                                           "type": "string",
                                           "description": "the question on the prediction market being bet on"
                    },
                    // "unit": {
                    //     "type": "string",
                    //     "enum": ["Celsius", "Fahrenheit"],
                    //     "description": "The temperature unit to use. Infer this from the user's location."
                    // }
                               },
                            "required": ["question"]
                               }
                               )),
                               strict: None,
                   }.into(),

            ])
            .build()?;
        let assistant = client.assistants().create(assistant_request).await?;
        let thread = client
            .threads()
            .create(CreateThreadRequest::default())
            .await?;
        let _message = client
            .threads()
            .messages(&thread.id)
            .create(CreateMessageRequest {
                role: MessageRole::Assistant,
                content: prompt.into(),
                ..Default::default()
            })
            .await?;
        let mut event_stream = client
            .threads()
            .runs(&thread.id)
            .create_stream(CreateRunRequest {
                assistant_id: assistant.id.clone(),
                stream: Some(true),
                ..Default::default()
            })
            .await?;

        let mut task_handle = None;

        while let Some(event) = event_stream.next().await {
            match event {
                Ok(event) => match event {
                    AssistantStreamEvent::ThreadRunRequiresAction(run_object) => {
                        println!("thread.run.requires_action: run_id:{}", run_object.id);
                        let client = client.clone();
                        task_handle = Some(tokio::spawn(async move {
                            handle_requires_action(client, run_object).await
                        }));
                    }
                    _ => println!("\nEvent: {event:?}\n"),
                },
                Err(e) => {
                    eprintln!("Error: {e}");
                }
            }
        }

        // wait for task to handle required action and submit tool outputs
        if let Some(task_handle) = task_handle {
            let _ = tokio::join!(task_handle);
        }

        // clean up
        client.threads().delete(&thread.id).await?;
        client.assistants().delete(&assistant.id).await?;

        Ok(())
    }
    async fn execute(&self) {
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
    async fn init(&self, question: &str, outcome: &str, tags: Option<Vec<String>>) -> Result<()> {
        let builder = &self.0;
        let platform = api::manifold::ManifoldPlatform::from(PlatformBuilder::default());
        let news = lookup_news(question, outcome).await.unwrap();
        let trimmed_news = news.iter().take(5).collect::<Vec<&String>>();
        tracing::debug!("Trimmed News: {:?}", trimmed_news);
        // tracing::debug!("News: {:?}", news);
        let mut questions_with_probability: Vec<serde_json::Value> = Vec::new();
        if let Some(tags) = tags {
            for tag in tags {
                let data = platform.fetch_json_by_description(&tag).await.unwrap();
                data.iter().for_each(|d| {
                    let question_with_probability = serde_json::json!({
                        "question": d["question"],
                        "probability": d["probability"]
                    });
                    questions_with_probability.push(question_with_probability);
                });
            }
        }

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
        let prompt = builder
            .promptor
            .prompts_manifold(questions_with_probability, trimmed_news, question, outcome)
            .await;
        // let thread_request = async_openai::types::CreateThreadRequestArgs::default().build()?;
        // let thread = client.threads().create(thread_request.clone()).await?;

        let assistant_request = CreateAssistantRequestArgs::default()
            .instructions(
                builder
                    .promptor
                    .superforecaster(question, outcome)
                    .await
                    .to_string(),
            )
            .model("gpt-4o")
            // .tools(vec![
            //        FunctionObject {
            //            name: "lookup_market".into(),
            //            description: Some("Lookup a specific market and associated data on Polymarket".into()),
            //            parameters: Some(serde_json::json!(
            //                    {
            //                        "type": "object",
            //                        "properties": {
            //                            "question": {
            //                                "type": "string",
            //                                "description": "the question on the prediction market being bet on"
            //         },
            //                    },
            //                 "required": ["question"]
            //                    }
            //                    )),
            //                    strict: None,
            //        }.into(),
            // ])
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

    async fn execute(&self) {
        println!("Manifold Executor executing");
    }
}

//wait for 1 second before checking the status again
//     .threads()
//     .messages(&thread.id)
//     .create(CreateMessageRequest {
//         role: MessageRole::User,
//         content: prompt.into(),
//         ..Default::default()
//     })
// .await?;
// let mut event_stream = client
//     .threads()
//     .runs(&thread.id)
//     .create_stream(CreateRunRequest {
//         assistant_id: assistant.id.clone(),
//         stream: Some(true),
//         ..Default::default()
//     })
//     .await?;

// let mut task_handle = None;

// while let Some(event) = event_stream.next().await {
//     match event {
//         Ok(event) => match event {
//             AssistantStreamEvent::ThreadRunRequiresAction(run_object) => {
//                 println!("thread.run.requires_action: run_id:{}", run_object.id);
//                 let client = client.clone();
//                 task_handle = Some(tokio::spawn(async move {
//                     handle_requires_action(client, run_object).await
//                 }));
//             }
//             _ => tracing::debug!("\nEvent: {event:?}\n"),
//         },
//         Err(e) => {
//             eprintln!("Error: {e}");
//         }
//     }
// }

// // wait for task to handle required action and submit tool outputs
// if let Some(task_handle) = task_handle {
//     let _ = tokio::join!(task_handle);
// }

// clean up

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

// async fn call_fn(name: &str, args: &str) -> Result<serde_json::Value> {

// }

// async fn lookup_market(question: &str, ) -> serde_json::Value {
//     let questions = api::polymarket::PolymarketPlatform::builder().build().

// }

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
            if (r.score.clone() > 0.5) {
                r.content.clone()
            } else {
                "".to_string()
            }
        })
        .collect::<Vec<String>>();

    Ok(results)
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
    async fn test_data_pipeline() {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
        let tags: Vec<String> = vec!["GPT-5".to_string(), "AI".to_string(), "OpenAI".to_string()];

        let executor =
            ManifoldExecutor::builder(1000, 1000, Promptor {}, ExecutorType::Manifold).build();
        let result = executor
            .init(
                "What is the probability of GPT-5 being availiable by 2025",
                "GPT-5 being availiable by 2025",
                Some(tags),
            )
            .await
            .unwrap();
        tracing::debug!("Result: {:?}", result);
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
        let result = executor
            .init(
                "What is the probability of Stalker 2 being released by 2025",
                "Stalker 2 being released by 2025",
                Some(tags),
            )
            .await
            .unwrap();
        tracing::debug!("Result: {:?}", result);
    }
}
