//! Full CRUD lifecycle for a knowledge base: create, get, update, delete.
//!
//! Requires a running UAR server (`UAR_BASE_URL`, default
//! `http://localhost:1906`).
use universal_agent_runtime_sdk::{Client, CreateKnowledgeBaseRequest, UpdateKnowledgeBaseRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(
        std::env::var("UAR_BASE_URL").unwrap_or_else(|_| "http://localhost:1906".into()),
    )?;

    let created = client
        .knowledge()
        .create(CreateKnowledgeBaseRequest {
            name: "cookbook-example-kb".into(),
            description: Some("Created by the knowledge_base_crud example".into()),
            config: None,
        })
        .await?;
    println!("created: {created:#?}");

    let fetched = client.knowledge().get(&created.id).await?;
    println!("fetched: {fetched:#?}");

    let updated = client
        .knowledge()
        .update(
            &created.id,
            UpdateKnowledgeBaseRequest {
                name: None,
                description: Some("Updated by the knowledge_base_crud example".into()),
                config: None,
            },
        )
        .await?;
    println!("updated: {updated:#?}");

    client.knowledge().delete(&created.id).await?;
    println!("deleted {}", created.id);
    Ok(())
}
