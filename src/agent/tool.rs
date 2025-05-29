use crate::error::Result;
use async_trait::async_trait;
use schemars::{JsonSchema, schema_for};
use serde_json::{Value, json};

#[async_trait]
pub trait ToolFunction: Send + Sync {
    async fn call(&self, args: Value) -> Result<Value>;
}

/// API Category for tool functions
#[derive(Debug, Clone, Copy)]
pub enum FunctionSchemaStyle {
    Legacy, // for chat/completions API（functions field）
    Tool,   // for assistants/responses API（tools filed）
}

/// Generic Function Schema Generator
pub fn build_function_schema<T: JsonSchema>(
    name: &str,
    description: &str,
    style: FunctionSchemaStyle,
) -> Value {
    let schema = schema_for!(T);
    let mut parameters = serde_json::to_value(schema.schema).unwrap();

    if let Some(obj) = parameters.as_object_mut() {
        obj.remove("title");
        obj.entry("type").or_insert(json!("object"));
    }

    match style {
        FunctionSchemaStyle::Legacy => {
            json!({
                "name": name,
                "description": description,
                "parameters": parameters
            })
        }
        FunctionSchemaStyle::Tool => {
            json!({
                "type": "function",
                "function": {
                    "name": name,
                    "description": description,
                    "parameters": parameters
                }
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemars::JsonSchema;

    #[derive(JsonSchema)]
    #[allow(dead_code)]
    struct GetWeatherArgs {
        /// The name of the city to get the weather for
        city: String,
        /// Latitude of the location
        latitude: f64,
        /// Longitude of the location
        longitude: f64,
        /// Unit of measurement, "Celsius" by default
        unit: Option<String>,
    }

    #[test]
    fn test_legacy_schema() {
        let schema = build_function_schema::<GetWeatherArgs>(
            "get_weather",
            "Get current weather",
            FunctionSchemaStyle::Legacy,
        );
        println!(
            "Legacy format:\n{}",
            serde_json::to_string_pretty(&schema).unwrap()
        );
    }

    #[test]
    fn test_tool_schema() {
        let schema = build_function_schema::<GetWeatherArgs>(
            "get_weather",
            "Get current weather",
            FunctionSchemaStyle::Tool,
        );
        println!(
            "Tool format:\n{}",
            serde_json::to_string_pretty(&schema).unwrap()
        );
    }
}
