//! Knowledge-base retrieval embeds queries with the ingestion backend
//! (`llm.embedding`) and refuses to search a KB that was indexed in a
//! different embedding space, instead of silently returning nothing.
//!
//! Offline: a deterministic hashing backend stands in for a hosted model, and
//! persistence is an embedded SurrealKV database in a temp directory. The
//! search goes through the real `/api/uar/knowledge-bases/{id}/search` router.

use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    Extension,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use universal_agent_runtime::uar::{
    api::knowledge::{KnowledgeApiState, build_router},
    domain::knowledge::{DocumentStatus, KbConfig, KnowledgeBase, KnowledgeDocument},
    persistence::{PersistenceLayer, providers::surreal::SurrealDbProvider},
    rag::{
        chunking::ChunkingStrategy,
        embeddings::{EmbeddingBackend, EmbeddingError},
        ingest::IngestService,
    },
    runtime::matching::VectorMatcher,
    security::claims::{UserClaims, UserContext},
};

const OWNER: &str = "kb-embedding-space-owner";
const RUST_DOC: &str = "Rust ownership and borrowing rules keep memory safe without a collector";
const BREAD_DOC: &str = "Sourdough bread needs flour water salt and a lively starter";

/// Bag-of-words hashing embedder: identical text maps to identical vectors,
/// shared words raise cosine similarity. Reports itself as a hosted backend so
/// the test exercises the non-fastembed path the defect was about.
#[derive(Debug)]
struct HashingBackend {
    model: &'static str,
    dimension: usize,
}

impl HashingBackend {
    fn shared(model: &'static str, dimension: usize) -> Arc<dyn EmbeddingBackend> {
        Arc::new(Self { model, dimension })
    }

    fn vectorize(&self, text: &str) -> Vec<f32> {
        let mut vector = vec![0.0_f32; self.dimension];
        for word in text
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
        {
            let hash = word
                .to_lowercase()
                .bytes()
                .fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
                    (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
                });
            let slot = usize::try_from(hash % self.dimension as u64).unwrap_or(0);
            vector[slot] += 1.0;
        }
        vector
    }
}

#[async_trait]
impl EmbeddingBackend for HashingBackend {
    fn backend_name(&self) -> &str {
        "openai"
    }

    fn vector_dimension(&self) -> usize {
        self.dimension
    }

    fn model_id(&self) -> &str {
        self.model
    }

    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        Ok(texts.iter().map(|t| self.vectorize(t)).collect())
    }
}

fn user() -> UserContext {
    UserContext {
        host_authority: None, authority: None,
        user_id: OWNER.to_string(),
        tenant_id: None,
        claims: UserClaims {
            sub: OWNER.to_string(),
            name: None,
            roles: Some(vec!["user".to_string()]),
            tenant_id: None,
            uar_instance_id: None,
            exp: usize::MAX,
        },
    }
}

async fn open_persistence(dir: &tempfile::TempDir) -> Arc<dyn PersistenceLayer> {
    let endpoint = format!("surrealkv://{}", dir.path().join("kb.db").display());
    Arc::new(
        SurrealDbProvider::new(&endpoint, None, None, Some("kb-space"), Some("kb-space"))
            .await
            .expect("open embedded SurrealKV database"),
    )
}

async fn create_kb(persistence: &dyn PersistenceLayer) -> KnowledgeBase {
    let now = chrono::Utc::now().to_rfc3339();
    let kb = KnowledgeBase {
        id: uuid::Uuid::new_v4().to_string(),
        owner_id: OWNER.to_string(),
        name: format!("kb-space-{}", uuid::Uuid::new_v4()),
        description: None,
        config: KbConfig::default(),
        created_at: now.clone(),
        updated_at: now,
    };
    persistence
        .save_knowledge_base(&kb)
        .await
        .expect("save knowledge base");
    kb
}

/// Chunks may only reference a tracked document; the worker pool saves the
/// document record before ingesting, so the fixture does the same.
async fn track_document(persistence: &dyn PersistenceLayer, kb_id: &str, doc_id: &str) {
    let now = chrono::Utc::now().to_rfc3339();
    persistence
        .save_document(&KnowledgeDocument {
            id: doc_id.to_string(),
            owner_id: OWNER.to_string(),
            kb_id: kb_id.to_string(),
            filename: format!("{doc_id}.txt"),
            file_path: None,
            mime_type: Some("text/plain".to_string()),
            chunk_count: 0,
            status: DocumentStatus::Processing,
            created_at: now.clone(),
            updated_at: now,
        })
        .await
        .expect("save document record");
}

/// Ingest both documents through the production `IngestService`.
async fn ingest_docs(
    persistence: &Arc<dyn PersistenceLayer>,
    backend: Arc<dyn EmbeddingBackend>,
    kb_id: &str,
) {
    let ingest = IngestService::new(Arc::clone(persistence), backend, ChunkingStrategy::Document);
    for (doc_id, text) in [("doc-rust", RUST_DOC), ("doc-bread", BREAD_DOC)] {
        track_document(persistence.as_ref(), kb_id, doc_id).await;
        ingest
            .ingest_text(text, OWNER, kb_id, doc_id.to_string())
            .await
            .expect("ingest document");
    }
}

/// POST the search route with the given query backend; returns status + JSON.
async fn search(
    persistence: &Arc<dyn PersistenceLayer>,
    query_backend: Arc<dyn EmbeddingBackend>,
    kb_id: &str,
    query: &str,
) -> (StatusCode, serde_json::Value) {
    let state = Arc::new(KnowledgeApiState {
        persistence: Arc::clone(persistence),
        vector_matcher: Arc::new(VectorMatcher::new(query_backend, 0.75)),
        ingestion_pool: None,
    });
    let app = build_router().with_state(state).layer(Extension(user()));
    let body = serde_json::json!({ "query": query, "limit": 5, "min_score": 0.3 });
    let response = app
        .oneshot(
            Request::post(format!("/{kb_id}/search"))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("build request"),
        )
        .await
        .expect("route request");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read body");
    let json = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

#[tokio::test]
async fn kb_search_with_ingestion_backend_returns_the_matching_chunk() {
    let dir = tempfile::tempdir().expect("tempdir");
    let persistence = open_persistence(&dir).await;
    let kb = create_kb(persistence.as_ref()).await;
    let backend = HashingBackend::shared("text-embedding-v4", 64);

    ingest_docs(&persistence, Arc::clone(&backend), &kb.id).await;
    let (status, json) = search(&persistence, backend, &kb.id, "rust ownership borrowing").await;

    assert_eq!(status, StatusCode::OK, "body: {json}");
    let top = json["results"][0]["content"].as_str().unwrap_or_default();
    assert_eq!(top, RUST_DOC, "results: {json}");
}

#[tokio::test]
async fn ingest_records_the_embedding_space_on_the_kb() {
    let dir = tempfile::tempdir().expect("tempdir");
    let persistence = open_persistence(&dir).await;
    let kb = create_kb(persistence.as_ref()).await;
    let backend = HashingBackend::shared("text-embedding-v4", 64);

    ingest_docs(&persistence, Arc::clone(&backend), &kb.id).await;
    let stored = persistence
        .get_knowledge_base(OWNER, &kb.id)
        .await
        .expect("read knowledge base")
        .expect("knowledge base exists");

    assert_eq!(stored.config.indexed_embedding, Some(backend.fingerprint()));
}

#[tokio::test]
async fn kb_search_from_another_embedding_space_is_an_explicit_conflict() {
    let dir = tempfile::tempdir().expect("tempdir");
    let persistence = open_persistence(&dir).await;
    let kb = create_kb(persistence.as_ref()).await;
    ingest_docs(
        &persistence,
        HashingBackend::shared("text-embedding-v4", 64),
        &kb.id,
    )
    .await;

    let other = HashingBackend::shared("bge-small-en-v1.5", 32);
    let (status, json) = search(&persistence, other, &kb.id, "rust ownership borrowing").await;

    assert_eq!(status, StatusCode::CONFLICT, "body: {json}");
    assert_eq!(
        json["error"]["code"], "embedding_space_mismatch",
        "body: {json}"
    );
}

#[tokio::test]
async fn ingest_from_another_embedding_space_is_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    let persistence = open_persistence(&dir).await;
    let kb = create_kb(persistence.as_ref()).await;
    ingest_docs(
        &persistence,
        HashingBackend::shared("text-embedding-v4", 64),
        &kb.id,
    )
    .await;

    let other = IngestService::new(
        Arc::clone(&persistence),
        HashingBackend::shared("bge-small-en-v1.5", 32),
        ChunkingStrategy::Document,
    );
    track_document(persistence.as_ref(), &kb.id, "doc-late").await;
    let result = other
        .ingest_text(RUST_DOC, OWNER, &kb.id, "doc-late".to_string())
        .await;

    let err = result.expect_err("mixing embedding spaces must fail");
    assert!(err.to_string().contains("was indexed with"), "error: {err}");
}
