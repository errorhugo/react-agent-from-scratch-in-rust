use std::env;

use clap::{Parser, command};
use colored::Colorize;
use reactagent::{
    GetGeoLocationArgs, GetGeoLocationTool, GetWeatherArgs, GetWeatherTool,
    agent::{
        base::BaseAgent,
        prompt::create_system_prompt,
        react::ReactAgent,
        tool::{FunctionSchemaStyle, build_function_schema},
    },
    error::Result,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Region name to check current weather (required)
    location: String,
    /// Load the .env at the specified absolute path, otherwise, it will load the .env
    /// located in the environment's current directory or its parents in sequence.
    #[arg(short, long)]
    dotenv_absolute_path: Option<String>,
}

async fn crate_base_agent() -> Result<BaseAgent> {
    let tools = [
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

    let base_agent = BaseAgent::new(
        "Base Agent",
        "A base agent for LLM interactions",
        &env::var("LLM_MODEL")
            .unwrap_or_else(|_| "meta-llama/llama-3.3-8b-instruct:free".to_string()),
        Some(&system_prompt),
    )
    .await;

    Ok(base_agent)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    match args.dotenv_absolute_path {
        Some(ref abs_path) => {
            dotenv::from_path(abs_path).unwrap();
        }
        None => {
            dotenv::dotenv().unwrap();
        }
    }

    println!(
        "\nOPENAI_API_KEY: {}",
        env::var("OPENAI_API_KEY").unwrap().bold().bright_green()
    );

    println!(
        "OPENAI_BASE_URL: {}",
        env::var("OPENAI_BASE_URL").unwrap().bold().bright_green()
    );

    println!(
        "LLM Model: {}",
        env::var("LLM_MODEL").unwrap().bold().bright_green()
    );

    let query = format!("What is the weather like in {} today?", args.location);

    let base_agent = crate_base_agent().await?;
    let mut react_agent = ReactAgent::new(
        "React Agent".to_string(),
        "An agent that can react to user queries and use tools".to_string(),
        base_agent,
        Some(10), // Set max interactions to 10
    );

    react_agent.add_tool("get_weather", GetWeatherTool);
    react_agent.add_tool("get_geo_location", GetGeoLocationTool);

    match react_agent.react_loop(&query).await {
        Ok(answer) => {
            println!("\n\n");
            println!("Final answer: ");
            println!("{}", "--".repeat(30).bold().bright_green());
            println!("{}", answer.bold().bright_green());
        }
        Err(e) => {
            println!("Error: {}", e.to_string().red());
        }
    }

    Ok(())
}
