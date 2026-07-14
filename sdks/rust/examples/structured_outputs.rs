use serde_json::json;
use universal_agent_runtime_sdk::{ChatCompletionRequest, ChatMessage, Client};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(
        std::env::var("UAR_BASE_URL").unwrap_or_else(|_| "http://localhost:1906".into()),
    )?;
    let request = ChatCompletionRequest {
        messages: vec![ChatMessage::text("user", "Return a color")],
        response_format: Some(
            json!({"type":"json_schema","json_schema":{"name":"color","schema":{"type":"object","properties":{"color":{"type":"string"}},"required":["color"]}}}),
        ),
        ..Default::default()
    };
    println!("{:#?}", client.chat().complete(request).await?);
    Ok(())
}
