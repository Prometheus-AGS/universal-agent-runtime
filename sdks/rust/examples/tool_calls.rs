use serde_json::json;
use universal_agent_runtime_sdk::Client;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(
        std::env::var("UAR_BASE_URL").unwrap_or_else(|_| "http://localhost:1906".into()),
    )?;
    println!(
        "{:#?}",
        client.tools().execute("memory_list", json!({})).await?
    );
    Ok(())
}
