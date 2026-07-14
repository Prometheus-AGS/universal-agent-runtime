//! List every knowledge base registered on the runtime.
//!
//! Requires a running UAR server (`UAR_BASE_URL`, default
//! `http://localhost:1906`).
use universal_agent_runtime_sdk::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(
        std::env::var("UAR_BASE_URL").unwrap_or_else(|_| "http://localhost:1906".into()),
    )?;
    for kb in client.knowledge().list().await? {
        println!(
            "{}: {} ({} docs config)",
            kb.id, kb.name, kb.config.embedding_model
        );
    }
    Ok(())
}
