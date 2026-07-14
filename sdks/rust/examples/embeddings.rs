//! Create embeddings for a batch of strings.
//!
//! Requires a running UAR server (`UAR_BASE_URL`, default
//! `http://localhost:1906`) with an embedding-capable model configured.
use universal_agent_runtime_sdk::{Client, EmbeddingRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(
        std::env::var("UAR_BASE_URL").unwrap_or_else(|_| "http://localhost:1906".into()),
    )?;
    let response = client
        .embeddings()
        .create(EmbeddingRequest {
            input: vec!["Universal Agent Runtime".into(), "Rust SDK".into()],
            model: None,
        })
        .await?;
    for embedding in response.data {
        println!(
            "index={} dims={}",
            embedding.index,
            embedding.embedding.len()
        );
    }
    Ok(())
}
