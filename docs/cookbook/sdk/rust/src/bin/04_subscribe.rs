//! Cookbook SDK (Rust): subscribe to a streaming chat completion.

use futures::StreamExt as _;
use universal_agent_runtime_sdk::{ChatCompletionRequest, ChatMessage, Client};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = std::env::var("UAR_BASE_URL").unwrap_or_else(|_| "http://localhost:1906".into());
    let client = Client::new(&base_url)?;

    let request = ChatCompletionRequest {
        messages: vec![ChatMessage::text("user", "Count to three")],
        ..Default::default()
    };

    let mut events = client.chat().stream(request).await?;
    println!("Subscribed to chat stream");

    while let Some(event) = events.next().await {
        match event {
            Ok(event) => println!("event: {event:?}"),
            Err(error) => {
                eprintln!("stream error: {error}");
                break;
            }
        }
    }

    Ok(())
}
