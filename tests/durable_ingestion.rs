//! Dot08 database and HTTP admission boundaries. Authored during Execute; run only
//! after the expanded Dot production inventory is complete. Each crash stage owns
//! the local database in a separate, joined process. No same-handle restart claim.
#![cfg(all(feature = "server", feature = "surreal-backend", feature = "a2a-transport"))]

#[path = "support/api_key_process.rs"]
mod process;

use std::{process::Command, sync::Arc};
use axum::{Extension, body::{Body, to_bytes}, http::{Request, StatusCode}};
use tower::ServiceExt;
use universal_agent_runtime::uar::{
    api::knowledge::{KnowledgeApiState, build_router},
    domain::knowledge::{DocumentStatus, EmbeddingFingerprint, KbConfig, KnowledgeBase, KnowledgeChunk, KnowledgeDocument},
    persistence::{PersistenceLayer, ingestion_records::{IngestionPublication, IngestionState}, providers::surreal::SurrealDbProvider},
    rag::embeddings::{EmbeddingBackend, EmbeddingError},
    runtime::matching::VectorMatcher,
    security::{claims::UserContext, middleware::authenticate_bearer_context},
};

const SOURCE: &[u8] = b"durable upload survives loss of dispatch and process";
const OWNER: &str = "v1:k:issuer:i:5:dot08:local:s:11:dot08-owner";

async fn identity() -> UserContext {
    let secret = "dot08-fixture-only-auth-secret";
    let config = serde_json::from_value(serde_json::json!({"jwt_required":true,"jwt_secret":secret,"jwt_issuer":"dot08"})).unwrap();
    let _ = authenticate_bearer_context(&config, "invalid").await;
    let token = jsonwebtoken::encode(&jsonwebtoken::Header::default(),
        &serde_json::json!({"iss":"dot08","sub":"dot08-owner","roles":["user"],"exp":chrono::Utc::now().timestamp()+3600}),
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes())).unwrap();
    let context = authenticate_bearer_context(&config, &token).await.unwrap();
    assert_eq!(context.session_owner_key().unwrap(), OWNER);
    context
}
const KB: &str = "dot08-kb";

#[derive(Debug)]
struct UnusedEmbedding;
#[async_trait::async_trait]
impl EmbeddingBackend for UnusedEmbedding {
    fn backend_name(&self) -> &str { "openai" }
    fn model_id(&self) -> &str { "dot08-fixture" }
    fn vector_dimension(&self) -> usize { 384 }
    async fn embed(&self, _: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        panic!("admission without a worker must not invoke embeddings")
    }
}

async fn provider() -> Arc<dyn PersistenceLayer> {
    let url = std::env::var("UAR_DOT08_URL").unwrap();
    #[cfg(feature = "postgres-backend")]
    if std::env::var("UAR_DOT08_PROVIDER").unwrap() == "postgres" {
        return Arc::new(universal_agent_runtime::uar::persistence::providers::postgres::PostgresProvider::new(&url).await.unwrap());
    }
    Arc::new(SurrealDbProvider::new(&url,
        std::env::var("UAR_DOT08_USER").ok().as_deref(),
        std::env::var("UAR_DOT08_PASS").ok().as_deref(),
        Some(&std::env::var("UAR_DOT08_NS").unwrap()), Some("ingestion")).await.unwrap())
}

async fn app(store: Arc<dyn PersistenceLayer>) -> axum::Router {
    build_router().with_state(Arc::new(KnowledgeApiState {
        persistence: store,
        vector_matcher: Arc::new(VectorMatcher::new(Arc::new(UnusedEmbedding), 0.75)),
        ingestion_pool: None,
    })).layer(Extension(identity().await))
}

fn document(id: &str) -> KnowledgeDocument {
    let now = chrono::Utc::now().to_rfc3339();
    KnowledgeDocument { id: id.to_owned(), owner_id: OWNER.into(), kb_id: KB.into(), filename: "source.txt".into(),
        file_path: None, mime_type: Some("text/plain".into()), chunk_count: 0, status: DocumentStatus::Pending,
        created_at: now.clone(), updated_at: now }
}

fn publication(document: &str) -> IngestionPublication {
    IngestionPublication {
        embedding: EmbeddingFingerprint { backend: "openai".into(), model: "dot08-fixture".into(), dimension: 384 },
        chunks: vec![KnowledgeChunk { id: uuid::Uuid::new_v4(), owner_id: OWNER.into(), kb_id: KB.into(),
            document_id: Some(document.into()), content: String::from_utf8(SOURCE.to_vec()).unwrap(), metadata: None,
            embedding: vec![1.0;384], created_at: chrono::Utc::now().to_rfc3339() }],
    }
}

async fn child(stage: &str) {
    let store = provider().await;
    let manifest = std::env::var("UAR_DOT08_MANIFEST").unwrap();
    if stage == "unattested" {
        let now = chrono::Utc::now().to_rfc3339();
        store.save_knowledge_base(&KnowledgeBase { id: KB.into(), owner_id: OWNER.into(), name: KB.into(),
            description: None, config: KbConfig::default(), created_at: now.clone(), updated_at: now }).await.unwrap();
        let body = format!("--dot08\r\nContent-Disposition: form-data; name=\"file\"; filename=\"source.txt\"\r\nContent-Type: text/plain\r\n\r\n{}\r\n--dot08--\r\n", String::from_utf8_lossy(SOURCE));
        let response = app(Arc::clone(&store)).await.oneshot(Request::post(format!("/{KB}/documents"))
            .header("content-type", "multipart/form-data; boundary=dot08").body(Body::from(body)).unwrap()).await.unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert!(store.list_documents(OWNER, KB).await.unwrap().is_empty());
        assert!(store.get_ingestion_job(OWNER, "unattested").await.unwrap().is_none());
        return;
    }
    if stage == "accept" {
        let now = chrono::Utc::now().to_rfc3339();
        store.save_knowledge_base(&KnowledgeBase { id: KB.into(), owner_id: OWNER.into(), name: KB.into(),
            description: None, config: KbConfig::default(), created_at: now.clone(), updated_at: now }).await.unwrap();
        let body = format!("--dot08\r\nContent-Disposition: form-data; name=\"file\"; filename=\"source.txt\"\r\nContent-Type: text/plain\r\n\r\n{}\r\n--dot08--\r\n", String::from_utf8_lossy(SOURCE));
        let response = app(Arc::clone(&store)).await.oneshot(Request::post(format!("/{KB}/documents"))
            .header("content-type", "multipart/form-data; boundary=dot08").body(Body::from(body)).unwrap()).await.unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        let response: serde_json::Value = serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap()).unwrap();
        let id = response["id"].as_str().unwrap();
        std::fs::write(manifest, id).unwrap();
        // Exit without running worker dispatch or destructors after an actual HTTP202.
        std::process::exit(0);
    }
    let id = std::fs::read_to_string(manifest).unwrap();
    let job = store.get_ingestion_job(OWNER, &id).await.unwrap().unwrap();
    assert_eq!(store.get_ingestion_source(&job).await.unwrap().unwrap(), SOURCE);
    assert!(store.get_ingestion_job("v1:k:issuer:i:5:dot08:local:s:7:foreign", &id).await.unwrap().is_none());
    if stage == "claim" {
        assert!(store.discover_ingestion(100).await.unwrap().iter().any(|record| record.document_id == id));
        assert_eq!(job.state, IngestionState::Pending);
        let processing = job.processing();
        assert!(store.transition_ingestion(&job, &processing).await.unwrap());
        assert!(store.search_knowledge_scoped(OWNER, &[KB], &vec![1.0;384], 10, 0.0).await.unwrap().is_empty());
        // Crash during an attempt, before any generation is published.
        std::process::exit(0);
    }
    assert_eq!(job.state, IngestionState::Processing);
    let output = publication(&id);
    store.publish_ingestion(&job, &output).await.unwrap();
    assert!(store.publish_ingestion(&job, &output).await.is_err(), "duplicate worker overwrote terminal generation");
    let matches = store.search_knowledge_scoped(OWNER, &[KB], &vec![1.0;384], 10, 0.0).await.unwrap();
    assert_eq!(matches.len(), 1);
    let indexed = store.get_document(OWNER, &id).await.unwrap().unwrap();
    assert_eq!(indexed.status, DocumentStatus::Indexed);
    assert_eq!(indexed.chunk_count, 1);

    let pending = store.accept_ingestion(&document("deleted"), SOURCE).await.unwrap();
    let processing = pending.processing();
    assert!(store.transition_ingestion(&pending, &processing).await.unwrap());
    store.delete_document(OWNER, "deleted").await.unwrap();
    assert!(store.publish_ingestion(&processing, &publication("deleted")).await.is_err());
    assert!(store.get_ingestion_source(&processing).await.unwrap().is_none());
    assert!(!store.transition_ingestion(&processing, &processing.failed_attempt("late error".into())).await.unwrap());

    let pending = store.accept_ingestion(&document("retry"), SOURCE).await.unwrap();
    let first = pending.processing();
    assert!(store.transition_ingestion(&pending, &first).await.unwrap());
    let failed = first.failed_attempt("processing provider unavailable".into());
    assert!(store.transition_ingestion(&first, &failed).await.unwrap());
    let retry = failed.retry();
    assert!(store.transition_ingestion(&failed, &retry).await.unwrap());
    let second = retry.processing();
    assert!(store.transition_ingestion(&retry, &second).await.unwrap());
    assert!(store.publish_ingestion(&first, &publication("retry")).await.is_err());
    store.publish_ingestion(&second, &publication("retry")).await.unwrap();

    let mut rejected = document("rejected");
    rejected.kb_id = "missing-kb".into();
    assert!(store.accept_ingestion(&rejected, SOURCE).await.is_err());
    assert!(store.get_document(OWNER, "rejected").await.unwrap().is_none());
    assert!(store.get_ingestion_job(OWNER, "rejected").await.unwrap().is_none());
    store.delete_knowledge_base(OWNER, KB).await.unwrap();
    assert!(store.get_ingestion_job(OWNER, &id).await.unwrap().is_none());
    assert!(store.publish_ingestion(&job, &output).await.is_err());
}

#[tokio::test]
async fn durable_ingestion_contract() {
    if let Ok(stage) = std::env::var("UAR_DOT08_CHILD") { child(&stage).await; return; }
    let mut databases = vec![process::Database::local(), process::Database::shared()];
    #[cfg(feature = "postgres-backend")]
    databases.push(process::Database::postgres());
    for database in databases {
        let manifest = tempfile::NamedTempFile::new().unwrap();
        let remote_surreal = database.provider == "surreal"
            && ["ws://", "wss://", "http://", "https://"]
                .iter()
                .any(|prefix| database.url.starts_with(prefix));
        if remote_surreal {
            let mut command = Command::new(std::env::current_exe().unwrap());
            command.args(["--nocapture", "--test-threads=1"])
                .env_clear().env("PATH", std::env::var_os("PATH").unwrap_or_default())
                .env("UAR_DOT08_CHILD", "unattested").env("UAR_DOT08_URL", &database.url)
                .env("UAR_DOT08_PROVIDER", database.provider).env("UAR_DOT08_NS", &database.namespace)
                .env("UAR_DOT08_MANIFEST", manifest.path());
            for name in ["DYLD_LIBRARY_PATH", "LD_LIBRARY_PATH"] {
                if let Some(value) = std::env::var_os(name) { command.env(name, value); }
            }
            if let Some(user) = &database.user { command.env("UAR_DOT08_USER", user); }
            if let Some(password) = &database.password { command.env("UAR_DOT08_PASS", password); }
            assert!(command.status().unwrap().success(), "unattested remote ingestion child failed");
        }
        for stage in ["accept", "claim", "publish"] {
            let mut command = Command::new(std::env::current_exe().unwrap());
            command.args(["--nocapture", "--test-threads=1"])
                .env_clear().env("PATH", std::env::var_os("PATH").unwrap_or_default())
                .env("UAR_REMOTE_SURREAL_DURABILITY_ATTESTED", "1")
                .env("UAR_DOT08_CHILD", stage).env("UAR_DOT08_URL", &database.url)
                .env("UAR_DOT08_PROVIDER", database.provider).env("UAR_DOT08_NS", &database.namespace)
                .env("UAR_DOT08_MANIFEST", manifest.path());
            for name in ["DYLD_LIBRARY_PATH", "LD_LIBRARY_PATH"] {
                if let Some(value) = std::env::var_os(name) { command.env(name, value); }
            }
            if let Some(user) = &database.user { command.env("UAR_DOT08_USER", user); }
            if let Some(password) = &database.password { command.env("UAR_DOT08_PASS", password); }
            assert!(command.status().unwrap().success(), "ingestion child stage {stage} failed");
        }
    }
}
