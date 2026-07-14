//! Ingest raw text content via the generic ingestion endpoint.
//!
//! Requires a running UAR server (`UAR_BASE_URL`, default
//! `http://localhost:1906`).
use serde_json::json;
use universal_agent_runtime_sdk::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(
        std::env::var("UAR_BASE_URL").unwrap_or_else(|_| "http://localhost:1906".into()),
    )?;
    let response = client
        .ingest()
        .ingest(
            "Universal Agent Runtime ships a Rust SDK cookbook.",
            Some(json!({"source": "document_ingest example"})),
        )
        .await?;
    println!("{response:#?}");
    Ok(())
}
