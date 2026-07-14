use futures::StreamExt;
use serde_json::json;
use universal_agent_runtime_sdk::{Client, CreateRunRequest};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(
        std::env::var("UAR_BASE_URL").unwrap_or_else(|_| "http://localhost:1906".into()),
    )?;
    let run = client
        .runs()
        .create(CreateRunRequest {
            artifact: json!({"name":"example","version":"1"}),
            input: "Hello".into(),
            session_id: None,
        })
        .await?;
    let mut events = client.runs().stream(&run.run_id, None).await?;
    while let Some(event) = events.next().await {
        println!("{:#?}", event?);
    }
    Ok(())
}
