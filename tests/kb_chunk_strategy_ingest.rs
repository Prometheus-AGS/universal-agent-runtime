//! A knowledge base's `config.chunk_strategy` decides how its documents are
//! chunked at ingestion. Before this was honoured, `IngestService` chunked
//! every KB with one service-wide strategy, which cut text at "v0." and
//! "Obsidian 1." in version numbers.
//!
//! Run with `cargo test --features in-memory-backend --test kb_chunk_strategy_ingest`.
#![cfg(feature = "in-memory-backend")]

use async_trait::async_trait;
use std::sync::Arc;
use universal_agent_runtime::uar::domain::knowledge::{KbConfig, KnowledgeBase};
use universal_agent_runtime::uar::persistence::PersistenceLayer;
use universal_agent_runtime::uar::persistence::providers::memory::InMemoryProvider;
use universal_agent_runtime::uar::rag::chunking::ChunkingStrategy;
use universal_agent_runtime::uar::rag::embeddings::{EmbeddingBackend, EmbeddingError};
use universal_agent_runtime::uar::rag::ingest::IngestService;

#[derive(Debug)]
struct StubBackend;

#[async_trait]
impl EmbeddingBackend for StubBackend {
    fn backend_name(&self) -> &str {
        "stub"
    }

    fn vector_dimension(&self) -> usize {
        3
    }

    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        Ok(texts.iter().map(|_| vec![0.1, 0.2, 0.3]).collect())
    }
}

// The sentences the original bug split at "v0." and "Obsidian 1.".
const TEXT: &str = "IPFS Sync for Obsidian is at v0.2.0. It needs Obsidian 1.12.3 or later.";
const KB_ID: &str = "kb-1";
// `OWNER`, which is crate-private.
const OWNER: &str = "anonymous";

/// A service whose service-wide strategy is `Sentence`, over an in-memory store
/// that holds one KB configured with `kb_strategy` when it is given.
async fn service_with_kb(kb_strategy: Option<ChunkingStrategy>) -> IngestService {
    let persistence: Arc<dyn PersistenceLayer> = Arc::new(InMemoryProvider::new());
    if let Some(chunk_strategy) = kb_strategy {
        let kb = KnowledgeBase {
            id: KB_ID.to_string(),
            owner_id: OWNER.to_string(),
            name: "site".to_string(),
            description: None,
            config: KbConfig {
                chunk_strategy,
                ..KbConfig::default()
            },
            created_at: "2026-10-04T00:00:00Z".to_string(),
            updated_at: "2026-10-04T00:00:00Z".to_string(),
        };
        persistence.save_knowledge_base(&kb).await.unwrap();
    }
    IngestService::new(
        persistence,
        Arc::new(StubBackend),
        ChunkingStrategy::Sentence,
    )
}

async fn ingested_chunk_count(service: &IngestService) -> usize {
    service
        .ingest_text(TEXT, OWNER, KB_ID, "doc-1".to_string())
        .await
        .unwrap()
}

#[tokio::test]
async fn kb_document_strategy_keeps_version_numbers_in_one_chunk() {
    let service = service_with_kb(Some(ChunkingStrategy::Document)).await;

    assert_eq!(ingested_chunk_count(&service).await, 1);
}

#[tokio::test]
async fn kb_strategy_overrides_the_service_strategy() {
    let service = service_with_kb(Some(ChunkingStrategy::FixedSize { size: 10 })).await;

    // 71 characters in blocks of 10.
    assert_eq!(ingested_chunk_count(&service).await, 8);
}

#[tokio::test]
async fn missing_kb_falls_back_to_the_service_strategy() {
    let service = service_with_kb(None).await;

    // `Sentence` splits after every '.', including the ones inside the versions.
    assert!(ingested_chunk_count(&service).await > 2);
}
