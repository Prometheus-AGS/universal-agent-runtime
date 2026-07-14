use universal_agent_runtime_sdk::Client;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(
        std::env::var("UAR_BASE_URL").unwrap_or_else(|_| "http://localhost:1906".into()),
    )?;
    let kb_id = std::env::var("UAR_KB_ID").unwrap_or_else(|_| "replace-me".into());
    for hit in client
        .knowledge()
        .search(&kb_id, "What does this knowledge base contain?")
        .await?
        .results
    {
        println!("{:.3}: {}", hit.score, hit.content);
    }
    Ok(())
}
