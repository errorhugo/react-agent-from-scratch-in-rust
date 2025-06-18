pub mod weather {
    use std::env;

    use crate::error::AgentError;
    use crate::prelude::*;
    use async_trait::async_trait;
    use colored::Colorize;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use serde_json::{Value, json};

    use crate::agent::tool::ToolFunction;

    pub struct GetWeatherTool;

    #[derive(Serialize, Deserialize, JsonSchema)]
    pub struct GetWeatherArgs {
        /// the name of the city
        pub city: String,
        /// longitude of the location
        pub longitude: f32,
        /// latitude of the location
        pub latitude: f32,
        /// Unit of measurement - "Celsius" or "Fahrenheit", "Celsius" by default
        pub unit: Option<String>,
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    pub struct GetWeatherResponse {
        /// the name of the city
        pub city: String,
        /// current temperature in Celsius or Fahrenheit
        pub temperature: f32,
        /// Unit of measurement - "Celsius" or "Fahrenheit"
        pub unit: String,
        /// weather condition, e.g., "Sunny"
        pub condition: String,
    }

    impl GetWeatherTool {
        pub async fn get_weather(&self, args: GetWeatherArgs) -> Result<GetWeatherResponse> {
            let client = reqwest::Client::new();

            let url = "https://api.openweathermap.org/data/2.5/weather";

            let params = [
                ("lat", args.latitude.to_string()),
                ("lon", args.longitude.to_string()),
                (
                    "appid",
                    env::var("OPENWEATHERMAP_API_KEY").unwrap_or_default(),
                ),
                (
                    "units",
                    args.unit
                        .clone()
                        .or_else(|| Some("metric".to_string()))
                        .unwrap(),
                ),
            ];

            let response = client.get(url).query(&params).send().await?;
            if !response.status().is_success() {
                return Err(AgentError::Generic(format!(
                    "Failed to get weather: {}",
                    response.status()
                )));
            }

            let body = response.text().await?;
            let json_body = serde_json::from_str::<Value>(&body).map_err(|e| {
                AgentError::Generic(format!("Failed to parse JSON response: {}", e))
            })?;

            println!(
                "3rd party API response (Open Weather Map): {}",
                json_body.to_string().dimmed()
            );

            Ok(GetWeatherResponse {
                city: args.city.clone(),
                temperature: json_body["main"]["temp"].as_f64().unwrap_or(0.0) as f32,
                unit: args.unit.clone().unwrap_or("Celsius".to_string()),
                condition: json_body["weather"][0]["description"]
                    .as_str()
                    .unwrap_or("Unknown")
                    .to_string(),
            })
        }
    }

    #[async_trait]
    impl ToolFunction for GetWeatherTool {
        async fn call(&self, args: Value) -> Result<Value> {
            let city = args
                .get("city")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown City");
            let latitude = args.get("latitude").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let longitude = args
                .get("longitude")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let unit = args
                .get("unit")
                .and_then(|v| v.as_str())
                .unwrap_or("metric");

            let response = self
                .get_weather(GetWeatherArgs {
                    city: city.to_string(),
                    latitude: latitude as f32,
                    longitude: longitude as f32,
                    unit: Some(unit.to_string()),
                })
                .await?;

            let unit_translated = match unit {
                "metric" => "Celsius",
                "imperial" => "Fahrenheit",
                _ => "Kelvin",
            };

            Ok(json!({
                "city": city,
                "latitude": latitude,
                "longitude": longitude,
                "unit": unit_translated.to_string(),
                "temperature": response.temperature,
            }))
        }
    }
}

pub mod geo {
    use std::env;

    use crate::error::AgentError;
    use crate::prelude::*;
    use async_trait::async_trait;
    use colored::Colorize;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use serde_json::{Value, json};

    use crate::agent::tool::ToolFunction;

    pub struct GetGeoLocationTool;

    #[derive(Serialize, Deserialize, JsonSchema)]
    pub struct GetGeoLocationArgs {
        /// the name of the city
        pub city: String,
    }

    pub struct GetGeoLocationResponse {
        /// the name of the city
        pub city: String,
        /// longitude of the location
        pub longitude: f32,
        /// latitude of the location
        pub latitude: f32,
    }

    impl GetGeoLocationTool {
        pub async fn get_geo_location(
            &self,
            args: GetGeoLocationArgs,
        ) -> Result<GetGeoLocationResponse> {
            let client = reqwest::Client::new();

            let url = "https://api.opencagedata.com/geocode/v1/json";

            let params = [
                ("key", env::var("OPENCAGEDATA_API_KEY").unwrap_or_default()),
                ("q", args.city.clone()),
                ("pretty", "1".to_owned()),
            ];

            let response = client.get(url).query(&params).send().await?;
            if !response.status().is_success() {
                return Err(AgentError::Generic(format!(
                    "Failed to get geo location: {}",
                    response.status()
                )));
            }

            let body = response.text().await?;
            let json_body = serde_json::from_str::<Value>(&body).map_err(|e| {
                AgentError::Generic(format!("Failed to parse JSON response: {}", e))
            })?;

            println!(
                "3rd party API response (Open Cage Geo): {}",
                json_body.to_string().dimmed()
            );

            Ok(GetGeoLocationResponse {
                city: args.city.clone(),
                longitude: json_body["results"][0]["geometry"]["lng"]
                    .as_f64()
                    .unwrap_or(0.0) as f32,
                latitude: json_body["results"][0]["geometry"]["lat"]
                    .as_f64()
                    .unwrap_or(0.0) as f32,
            })
        }
    }

    #[async_trait]
    impl ToolFunction for GetGeoLocationTool {
        async fn call(&self, args: Value) -> Result<Value> {
            let city = args
                .get("city")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown City");

            let response = self
                .get_geo_location(GetGeoLocationArgs {
                    city: city.to_string(),
                })
                .await?;

            Ok(json!({
                "city": city,
                "latitude": response.latitude,
                "longitude": response.longitude,
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::geo;
    use super::weather;
    use dotenv::dotenv;

    use tokio_test::block_on;

    #[test]
    fn test_get_weather() {
        dotenv().unwrap();

        let args = weather::GetWeatherArgs {
            city: "Beijing".to_string(),
            longitude: 116.41,
            latitude: 40.190,
            unit: None,
        };

        let result = block_on(weather::GetWeatherTool.get_weather(args));
        if result.is_err() {
            eprintln!("Error: {}", result.as_ref().err().unwrap());
        }

        assert!(result.is_ok());
        let weather = result.unwrap();
        assert_eq!(weather.city, "Beijing");
        assert!(weather.temperature != 0.0);
        assert!(!weather.condition.is_empty());
    }

    #[test]
    fn test_get_geo_location() {
        dotenv().unwrap();

        let args = geo::GetGeoLocationArgs {
            city: "London".to_string(),
        };

        let result = block_on(geo::GetGeoLocationTool.get_geo_location(args));
        if result.is_err() {
            eprintln!("Error: {}", result.as_ref().err().unwrap());
        }

        assert!(result.is_ok());
        let loc = result.unwrap();
        assert_eq!(loc.city, "London");
        assert!(loc.longitude != 0.0);
        assert!(loc.latitude != 0.0);
    }
}
