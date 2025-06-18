use std::{collections::HashMap, sync::Arc};

use crate::error::AgentError;
use crate::prelude::*;
use colored::Colorize;
use serde::Deserialize;
use serde_json::Value;

use super::{base::Agent, tool::ToolFunction};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReactState {
    PAUSE,
    ANSWER,
}

#[derive(Debug, Deserialize)]
pub struct ActionCall {
    pub state: ReactState,
    pub thought: String,
    pub action: Action,
}

#[derive(Debug, Deserialize)]
pub struct Action {
    pub tool: String,
    pub input: serde_json::Value,
}

pub struct ReactAgent<T: Agent> {
    pub name: String,
    pub description: String,
    max_interactions: u8,
    agent: T,
    tools: HashMap<String, Arc<dyn ToolFunction>>,
}

impl<T: Agent> ReactAgent<T> {
    pub fn new(name: String, description: String, agent: T, max_interactions: Option<u8>) -> Self {
        ReactAgent {
            name,
            description,
            agent,
            max_interactions: max_interactions.unwrap_or(10), // Default to 10 if not specified
            tools: HashMap::new(),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn add_tool<F: ToolFunction + 'static>(&mut self, name: &str, tool: F) {
        self.tools.insert(name.to_string(), Arc::new(tool));
    }

    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn ToolFunction>> {
        self.tools.get(name).cloned()
    }

    pub async fn react_loop(&mut self, user_input: &str) -> Result<String> {
        let mut interactions = 0_u8;

        let mut next_prompt = user_input.to_string();
        loop {
            println!("\nprompt: {}\n", next_prompt.blue());

            interactions += 1;
            if interactions > self.max_interactions {
                return Err(AgentError::Generic(format!(
                    "Maximum interactions {} reached",
                    self.max_interactions
                )));
            }

            let response = self.agent.step(&next_prompt).await;
            let json_resp = match response {
                Ok(res) => res,
                Err(e) => {
                    println!(
                        "{}",
                        format!("Failed to get response from agent: {}", e).red()
                    );
                    continue;
                }
            };

            if let Ok(parsed_resp) = serde_json::from_str::<ActionCall>(&json_resp) {
                match parsed_resp.state {
                    ReactState::PAUSE => {
                        println!("Paused: {}", json_resp.cyan());
                    }
                    ReactState::ANSWER => {
                        println!("Answer: {}", json_resp.cyan());
                        println!("Interaction {} times.", interactions.to_string().yellow());
                        return Ok(parsed_resp.thought.clone());
                    }
                }

                let observation: String;

                // Process the action call
                if parsed_resp.action.tool != "none" {
                    // Execute the tool with the provided arguments
                    let tool_name = &parsed_resp.action.tool;
                    let tool_args = &parsed_resp.action.input;

                    // call the actual tool with its name and arguments
                    let tool_result = self.execute_tool(tool_name, tool_args).await?;
                    observation = tool_result;
                } else {
                    observation = parsed_resp.thought;
                }

                println!("Observation: {}", observation.cyan());
                next_prompt = format!("**Observation**: {}", observation);
            } else {
                println!(
                    "Failed to parse action call JSON: <BEGIN>\n{}\n<END>",
                    json_resp
                );
                continue;
            }
        }
    }

    pub async fn execute_tool(&mut self, tool_name: &str, tool_args: &Value) -> Result<String> {
        println!(
            "\nExecuted tool: {} with args: {}",
            tool_name.italic(),
            tool_args.to_string().italic()
        );

        if let Some(tool) = self.get_tool(tool_name) {
            return Ok(tool.call(tool_args.clone()).await?.to_string());
        }

        Err(AgentError::Generic(format!(
            "Not found tool: {} with args: {}",
            tool_name, tool_args,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_action_call() {
        let raw_resp = r#"
        {
            "state": "pause",
            "thought": "I need the latitude and longitude of Beijing to use with the get_weather tool.",
            "action": {
                "tool": "get_geo_location",
                "input": {
                    "city": "Beijing"
                }
            }
        }
        "#;

        if let Ok(action_call) = serde_json::from_str::<ActionCall>(raw_resp) {
            assert!(matches!(action_call.state, ReactState::PAUSE));
            assert_eq!(action_call.action.tool, "get_geo_location".to_string());
        }
    }
}
