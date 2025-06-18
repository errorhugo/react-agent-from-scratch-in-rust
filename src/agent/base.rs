use crate::{error::AgentError, prelude::*};
use std::env;

use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
};
use async_trait::async_trait;

#[async_trait]
pub trait Agent {
    fn name(&self) -> &str;
    fn description(&self) -> &str;

    async fn step<'a>(&self, message: &'a str) -> Result<String>;
}

pub struct BaseAgent {
    name: String,
    description: String,
    messages: tokio::sync::Mutex<Vec<ChatCompletionRequestMessage>>,
    client: Client<OpenAIConfig>,
    model_name: String,
}

#[async_trait]
impl Agent for BaseAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn step<'a>(&self, message: &'a str) -> Result<String> {
        {
            let mut lock = self.messages.lock().await;

            lock.push(ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessageArgs::default()
                    .content(message)
                    .build()
                    .unwrap(),
            ));
        }

        let result = self.execute().await?;

        {
            let mut lock = self.messages.lock().await;

            lock.push(ChatCompletionRequestMessage::Assistant(
                ChatCompletionRequestAssistantMessageArgs::default()
                    .content(result.clone())
                    .build()
                    .unwrap(),
            ));
        }

        Ok(result)
    }
}

impl BaseAgent {
    pub async fn new(
        name: &str,
        description: &str,
        model_name: &str,
        system_prompt: Option<&str>,
    ) -> Self {
        let api_key = env::var("OPENAI_API_KEY").unwrap();
        let base_url = env::var("OPENAI_BASE_URL").unwrap();

        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(base_url);
        let client = Client::with_config(config);

        let instance = BaseAgent {
            name: name.to_string(),
            description: description.to_string(),
            messages: tokio::sync::Mutex::new(Vec::new()),
            client,
            model_name: model_name.to_string(),
        };

        if let Some(system_prompt) = system_prompt {
            let mut lock = instance.messages.lock().await;

            lock.push(ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(system_prompt)
                    .build()
                    .unwrap(),
            ));
        }

        instance
    }

    async fn execute(&self) -> Result<String> {
        let completion = {
            let lock = self.messages.lock().await;

            self.client
                .chat()
                .create(
                    CreateChatCompletionRequestArgs::default()
                        .model(&self.model_name)
                        .messages(lock.clone())
                        .build()?,
                )
                .await?
        };

        let choice = completion
            .choices
            .first()
            .ok_or_else(|| AgentError::Generic("No choices returned from OpenAI API".to_owned()))?;

        choice
            .message
            .content
            .clone()
            .ok_or_else(|| AgentError::Generic("No content in the response message".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        GetGeoLocationArgs, GetWeatherArgs,
        agent::prompt::create_system_prompt,
        agent::tool::{FunctionSchemaStyle, build_function_schema},
    };

    use super::*;

    #[tokio::test]
    async fn test_generate_response() {
        dotenv::dotenv().unwrap();

        let tools = vec![
            build_function_schema::<GetWeatherArgs>(
                "get_weather",
                "Get current weather of the location",
                FunctionSchemaStyle::Legacy,
            ),
            build_function_schema::<GetGeoLocationArgs>(
                "get_geo_location",
                "Get the latitude and longitude of a city",
                FunctionSchemaStyle::Legacy,
            ),
        ];

        let system_prompt = create_system_prompt(
            tools
                .iter()
                .map(|t| serde_json::to_string_pretty(t).unwrap())
                .collect::<Vec<String>>(),
            None,
        );

        let agent = BaseAgent::new(
            "TestAgent",
            "A test agent for generating responses",
            &env::var("LLM_MODEL")
                .unwrap_or_else(|_| "meta-llama/llama-3.3-8b-instruct:free".to_string()),
            Some(&system_prompt),
        )
        .await;

        let reply = agent
            .step("What is the weather like in London today?")
            .await;

        assert!(reply.is_ok());
        println!("{:?}", reply.unwrap());
    }
}
