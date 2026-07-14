use universal_agent_runtime_sdk::{ChatCompletionRequest, ChatMessage, Client};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(
        std::env::var("UAR_BASE_URL").unwrap_or_else(|_| "http://localhost:1906".into()),
    )?;
    let response = client
        .chat()
        .complete(ChatCompletionRequest {
            messages: vec![ChatMessage::text("user", "Hello from Rust")],
            ..Default::default()
        })
        .await?;
    println!("{response:#?}");
    Ok(())
}
