use rmcp::{model::JsonObject, tool};
use schemars::JsonSchema;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GetWeatherRequest {
    pub city: String,
    pub date: String,
}
pub struct Server {

}

impl Server {
    /// This tool is used to get the weather of a city.
    #[tool(name = "get-weather", description = "Get the weather of a city.")]
    pub async fn get_weather(
        #[tool(param)] city: String
    ) -> String {
        drop(city);
        "rain".to_string()
    }
}

#[tokio::test]
async fn test_tool_macros() {
    let attr = Server::get_weather_tool_attr();
    println!("{:?}", attr);
    let result = Server::get_weather_call(
        serde_json::json!(
            {"city": "Harbin"}
        )
        .as_object()
        .cloned()
        .unwrap(),
    )
    .await
    .unwrap();
    let weather = &result.content[0].as_text().as_ref().unwrap().text;
    assert_eq!(weather, "rain");
}

impl GetWeatherRequest {

}