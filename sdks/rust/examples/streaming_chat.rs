use futures::StreamExt;
use universal_agent_runtime_sdk::{ChatCompletionRequest, ChatMessage, Client};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(
        std::env::var("UAR_BASE_URL").unwrap_or_else(|_| "http://localhost:1906".into()),
    )?;
    let mut events = client
        .chat()
        .stream(ChatCompletionRequest {
            messages: vec![ChatMessage::text("user", "Tell me a short story")],
            ..Default::default()
        })
        .await?;
    while let Some(event) = events.next().await {
        println!("{:#?}", event?);
    }
    Ok(())
}
