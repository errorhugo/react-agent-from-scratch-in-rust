use serde_json::Value;

const SYSTEM_PROMPT: &str = r#"
You are an intelligent assistant that operates strictly in a loop of Thought → Action → PAUSE → Observation.

You must follow this exact step-by-step interaction loop:

**Thought**: Think step by step about the current task or available information.
**Action**: Call a tool using the available tools list, providing input parameters.
**PAUSE**: Immediately pause and wait for the result. Do not generate any output beyond this point.
**Observation**: You will be given an observation after the action. Reflect and continue the loop.

Repeat the loop until you have enough information to produce a final Answer.

***Response Format Rule***:
You must only produce output in this strict JSON format:
{
  "state": "pause" | "answer",
  "thought": "<step-by-step reasoning>",
  "action": {
    "tool": "<tool_name>",
    "input": {
      // tool-specific parameters
    }
  }
}

When calling a tool, always set "state": "pause" and stop. Do not generate Observation or Answer yet.

When ready to answer, use "state": "answer".

This loop is strictly enforced. Any deviation will be considered invalid output.




Your available tools are:
-------------------------
{available_tools}

-------------------------



Example session:
-------------------------
{example}
-------------------------

Now it's your turn to use the tools effectively. Return only the concise final answer in a single sentence. No additional text. 
"#;

const DEFAULT_EXAMPLE: &str = r##"
{
  "state": "pause",
  "thought": "I need to find out the current weather conditions in London. To do this, I'll first need to get the latitude and longitude of London as these are required for the get_weather tool. Once I have the coordinates, I can use the get_weather tool to find out the current weather in London.",
  "action": {
    "tool": "get_geo_location",
    "input": {
      "city": "London"
    }
  }
}


**Observation**: The latitude of London is 51.5074 and the longitude is -0.1278.

{
  "state": "pause",
  "thought": "Now that I have the coordinates, I can use the get_weather tool to find out the current weather in London. I'll also specify the unit of measurement, which in this case, I'll leave as null to default to the most common unit.",
  "action": {
    "tool": "get_weather",
    "input": { 
        "city": "London", 
        "latitude": 51.5074, 
        "longitude": -0.1278, 
        "unit": null 
    }
  }
}

**Observation**: The current weather in London is described as overcast with a temperature of 12°C.

{
    "state": "answer",
    "thought": "The weather in London today is overcast with a temperature of 12°C.",
    "action": {
        "tool": "none",
        "input": {}
    }
}
"##;

pub fn create_system_prompt(tool_schema_list: Vec<String>, example: Option<String>) -> String {
    let mut avaiable_tools = String::new();

    for tool_json in tool_schema_list {
        let parsed: Value = serde_json::from_str(&tool_json).expect("Invalid JSON");
        if let Some(name) = parsed.get("name").and_then(|v| v.as_str()) {
            if let Some(desc) = parsed.get("description").and_then(|v| v.as_str()) {
                avaiable_tools.push_str(&format!("- {}: {}\n", name, desc));
                avaiable_tools.push_str(&tool_json);
                avaiable_tools.push_str("\n\n");
            }
        }
    }

    let example_prompt = example.unwrap_or(DEFAULT_EXAMPLE.to_string());

    SYSTEM_PROMPT
        .replace("{available_tools}", &avaiable_tools)
        .replace("{example}", &example_prompt)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCHEMA_STR: &str = r###"
        {
            "description": "Get current weather",
            "name": "get_weather",
            "parameters": {
                "properties": {
                "city": {
                    "description": "the name of the city",
                    "type": "string"
                },
                "latitude": {
                    "description": "latitude of the location",
                    "format": "float",
                    "type": "number"
                },
                "longitude": {
                    "description": "longitude of the location",
                    "format": "float",
                    "type": "number"
                },
                "unit": {
                    "description": "Unit of measurement, e.g., \"Celsius\"",
                    "type": [
                      "string",
                      "null"
                    ]
                }
                },
                "required": [
                  "city",
                  "latitude",
                  "longitude"
                ],
                "type": "object"
            }
        }
    "###;

    #[test]
    fn should_work() {
        let prompt = create_system_prompt(vec![SCHEMA_STR.to_owned()], None);
        println!("{prompt}");
    }
}
