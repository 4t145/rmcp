use rmcp::tool;
use schemars::JsonSchema;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GetWeatherRequest {
    pub city: String,
    pub date: String,
}

#[tool(GET_WEATHER)]
pub async fn get_weather(request: GetWeatherRequest) -> String {
    "rain".to_string()
}

#[test]
fn test_tool_macros()  {

}