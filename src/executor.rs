use crate::api::{self, *};
use async_openai::types::realtime::{
    ConversationItemCreateEvent, Item, ResponseCreateEvent, ServerEvent,
};
use async_openai::{
    config::OpenAIConfig,
    types::{
        AssistantStreamEvent, CreateAssistantRequestArgs, CreateMessageRequest, CreateRunRequest,
        CreateThreadRequest, FunctionObject, MessageDeltaContent, MessageRole, RunObject,
        SubmitToolOutputsRunRequest, ToolsOutputs,
    },
    Client,
};
use axum::async_trait;
use futures_util::{future, pin_mut, StreamExt};
use std::any::Any;
use tokio::io::AsyncReadExt;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, protocol::Message},
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Clone)]
pub enum ExecutorType {
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
        market_data: &str,
        event_data: &str,
        market_question: &str,
        outcome: &str,
    ) -> String {
        let prompt = format!("You are an AI assistant for users of a prediction market called Polymarket.
        Users want to place bets based on their beliefs of market outcomes such as political or sports events.
        
        Here is data for current Polymarket markets {} and 
        current Polymarket events {}.

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
            + "Filter these events for the ones you will be best at trading on profitably.";
        prompt.to_string()
    }
    async fn filter_markets(&self) -> String {
        let prompt = self.read_polymarket_api().await.to_string()
            + "Filter these markets for the ones you will be best at trading on profitably.";
        prompt.to_string()
    }

    async fn superforecaster(&self, question: &str, description: &str, outcome: &str) -> String {
        let prompt = self.read_polymarket_api().await.to_string() +  format!(" You are a Superforecaster tasked with correctly predicting the likelihood of events.
        Use the following systematic process to develop an accurate prediction for the following
        question={} and description={} combination. 
        
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

        The question {}; has a likelihood (float)% for outcome of (str).", question, description, outcome, question).as_str();
        prompt.to_string()
    }
}
#[async_trait]
pub trait Executor: From<ExecutorBuilder<Self>> + Any {
    async fn init(&self, question: &str, description: &str, outcome: &str) -> Result<()>;
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
    async fn init(&self, question: &str, description: &str, outcome: &str) -> Result<()> {
        let query = [("limit", "1")];
        let builder = &self.0;
        let client = async_openai::Client::new();
        let prompt = builder
            .promptor
            .superforecaster(question, description, outcome)
            .await
            .to_string();
        // let thread_request = async_openai::types::CreateThreadRequestArgs::default().build()?;
        // let thread = client.threads().create(thread_request.clone()).await?;

        let assistant_request = CreateAssistantRequestArgs::default().instructions(builder.promptor.superforecaster(question, description, outcome).await.to_string()).model("gpt-4o")
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

        // let chat_request = async_openai::CreateChatCompletionRequestArgs::default().(builder.promptor.superforecaster(question, description, outcome).await.to_string()).model("gpt-4o-2024-08-06").build()?;
        // let chat_request = async_openai::types::CreateChatCompletionRequestArgs::default().max_tokens(512u32).model("gpt-4o-2024-08-06").messages([
        //         async_openai::types::ChatCompletionRequestSystemMessage::from(builder.promptor.superforecaster(question, description, outcome).await).into(),]).build()?;
        // let chat = client.chat().create(chat_request).await?;
        // println!("Chat: {:#?}", chat);
        //let assistant = client.assistants().create(assistant_request).await?;
        //let assistant_id = assistant.id;
        //loop {
        //      let mut input = String::new();
        //    std::io::stdin().read_line(&mut input).unwrap();

        //    //break out of the loop if the user enters exit()
        //    if input.trim() == "exit()" {
        //        break;
        //    }
        //let run_request = CreateRunRequestArgs::default().assistant_id(&assistant_id).build()?;
        //let mut awaiting_response = true;
        // let run = client.threads().runs(&thread.id).create(run_request).await?;
        //while awaiting_response {
        //   let run = client.threads().runs(&thread.id).retrieve(&run.id).await?;
        //    match run.status {
        //            RunStatus::Completed => {
        //                awaiting_response = false;
        //                // once the run is completed we
        //                // get the response from the run
        //                // which will be the first message
        //                // in the thread

        //                //retrieve the response from the run
        //                let response = client.threads().messages(&thread.id).list(&query).await?;
        //                //get the message id from the response
        //                let message_id = response.data.first().unwrap().id.clone();
        //                //get the message from the response
        //                let message = client
        //                    .threads()
        //                    .messages(&thread.id)
        //                    .retrieve(&message_id)
        //                    .await?;
        //                //get the content from the message
        //                let content = message.content.first().unwrap();
        //                //get the text from the content
        //                let text = match content {
        //                    MessageContent::Text(text) => text.text.value.clone(),
        //                    MessageContent::ImageFile(_) | MessageContent::ImageUrl(_) => {
        //                        panic!("imaged are not expected in this example");
        //                    }
        //                    MessageContent::Refusal(refusal) => refusal.refusal.clone(),
        //                };
        //                //print the text
        //                println!("--- Response: {}\n", text);
        //            }
        //            RunStatus::Failed => {
        //                awaiting_response = false;
        //                println!("--- Run Failed: {:#?}", run);
        //            }
        //            RunStatus::Queued => {
        //                println!("--- Run Queued");
        //            }
        //            RunStatus::Cancelling => {
        //                println!("--- Run Cancelling");
        //            }
        //            RunStatus::Cancelled => {
        //                println!("--- Run Cancelled");
        //            }
        //            RunStatus::Expired => {
        //                println!("--- Run Expired");
        //            }
        //            RunStatus::RequiresAction => {
        //                println!("--- Run Requires Action");
        //            }
        //            RunStatus::InProgress => {
        //                println!("--- In Progress ...");
        //            }
        //            RunStatus::Incomplete => {
        //                println!("--- Run Incomplete");
        //            }
        //        }
        //    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        //}
        //}
        //client.assistants().delete(&assistant_id).await?;
        //client.threads().delete(&thread.id).await?;
        Ok(())
        // let message = CreateMessageRequestArgs::default()
        //     .role(MessageRole::User)
        //     .content(input.clone())
        //     .build()?;

        // }
    }
    async fn execute(&self) {
        println!("Polymarket Executor executing");
    }
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

mod tests {
    use super::*;

    #[tokio::test]
    async fn test_polymarket_executor() {
        let executor =
            PolymarketExecutor::builder(1000, 1000, Promptor {}, ExecutorType::Polymarket).build();
        let result = executor
            .init(
                "What is the probability of Joe Biden winning the 2024 US elections?",
                "Joe Biden is the current president of the United States of America",
                "Joe Biden winning the 2024 US elections",
            )
            .await
            .unwrap();
        tracing::debug!("Result: {:?}", result);
    }
}
