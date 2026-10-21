//! Real local-inference coverage for the embedding lifecycle contract.
//!
//! This fixture never supplies an embedding implementation. The primary,
//! compatible backup, and equal-dimension incompatible members must be actual
//! local model assets named by the operator; see
//! `fixtures/local_embedding_lifecycle/README.md`.

#![cfg(feature = "local-models")]

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use axum::{
    Extension,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use futures::FutureExt as _;
use sha2::Digest as _;
use tower::ServiceExt as _;
use universal_agent_runtime::{
    config::{LocalEmbeddingFailoverConfig, LocalEmbeddingMemberConfig, SecurityConfig},
    uar::{
        api::knowledge::{KnowledgeApiState, build_router},
        domain::knowledge::{DocumentStatus, KbConfig, KnowledgeBase, KnowledgeDocument},
        persistence::{PersistenceLayer, providers::surreal::SurrealDbProvider},
        rag::{
            chunking::ChunkingStrategy,
            embeddings::{
                EmbeddingBackend, EmbeddingConfig, build_local_backend,
                failover::LocalEmbeddingFailover,
                swappable::SwappableEmbeddingBackend,
            },
            ingest::IngestService,
        },
        runtime::{
            lifecycle::{ResourceScope, RuntimeSupervisor},
            matching::VectorMatcher,
        },
        security::{claims::UserContext, middleware::authenticate_bearer_context},
    },
};

const SUBJECT: &str = "embedding-lifecycle-owner";
const PRIMARY_MODEL: &str = "bge-small-en-v1.5";
const DIMENSION: usize = 384;
const MODEL_WEIGHT_FILE: &str = "bg-small-en-v1.5.onnx";
const MODEL_FILES: [&str; 5] = [
    MODEL_WEIGHT_FILE,
    "tokenizer.json",
    "config.json",
    "special_tokens_map.json",
    "tokenizer_config.json",
];

const PRIMARY_TEXT: &str = include_str!("fixtures/local_embedding_lifecycle/primary.txt");
const BACKUP_TEXT: &str = include_str!("fixtures/local_embedding_lifecycle/backup.txt");
const RECOVERY_TEXT: &str = include_str!("fixtures/local_embedding_lifecycle/recovery.txt");
const SECOND_BACKUP_TEXT: &str = include_str!("fixtures/local_embedding_lifecycle/second_backup.txt");
const SECOND_RECOVERY_TEXT: &str =
    include_str!("fixtures/local_embedding_lifecycle/second_recovery.txt");
const QUERY_TEXT: &str = include_str!("fixtures/local_embedding_lifecycle/query.txt");
const INCOMPATIBLE_QUERY: &str = include_str!("fixtures/local_embedding_lifecycle/incompatible-query.txt");

struct Fixture {
    root: Arc<tempfile::TempDir>,
    persistence_layer: Arc<dyn PersistenceLayer>,
    supervisor: RuntimeSupervisor,
    persistence_slot: Arc<std::sync::Mutex<Option<Arc<SurrealDbProvider>>>>,
    failover_scope: ResourceScope,
    failover: Arc<LocalEmbeddingFailover>,
    negative_backend: Arc<std::sync::Mutex<Option<Arc<dyn EmbeddingBackend>>>>,
    handle: Arc<SwappableEmbeddingBackend>,
    incompatible: EmbeddingConfig,
    primary_manifest: String,
    primary_weight_digest: String,
}

impl Fixture {
    async fn new() -> Result<Self, String> {
        let root = Arc::new(tempfile::tempdir().expect("create lifecycle fixture root"));
        let primary_source = required_model_dir("UAR_EMBEDDING_LIFECYCLE_PRIMARY_DIR");
        let backup_source = required_model_dir("UAR_EMBEDDING_LIFECYCLE_BACKUP_DIR");
        let incompatible_source = required_model_dir("UAR_EMBEDDING_LIFECYCLE_INCOMPATIBLE_DIR");
        let incompatible_model = required_environment("UAR_EMBEDDING_LIFECYCLE_INCOMPATIBLE_MODEL");
        let incompatible_dimension = required_environment("UAR_EMBEDDING_LIFECYCLE_INCOMPATIBLE_DIMENSION")
            .parse::<usize>()
            .expect("incompatible fixture dimension must be an integer");
        assert_eq!(
            incompatible_dimension, DIMENSION,
            "incompatible fixture must use the primary dimension; a shape-only mismatch does not satisfy this contract"
        );
        assert_ne!(
            incompatible_model, PRIMARY_MODEL,
            "incompatible fixture must identify a different actual model"
        );

        // The test may retire its primary member but never touches an
        // operator-provided directory. Its private copy is removed only by
        // `TempDir` after the enclosing fixture has drained.
        let primary_copy = root.path().join("primary-assets");
        copy_model_assets(&primary_source, &primary_copy);
        let primary_manifest = asset_manifest(&primary_copy);
        let primary_weight_digest = asset_weight_digest(&primary_copy);

        let primary = local_config(&primary_copy, PRIMARY_MODEL, DIMENSION);
        let backup = LocalEmbeddingMemberConfig {
            backend: "fastembed".to_string(),
            model: PRIMARY_MODEL.to_string(),
            vector_dimension: DIMENSION,
            batch_size: 32,
            models_dir: backup_source.display().to_string(),
        };
        let failover_config = LocalEmbeddingFailoverConfig { backup };
        let supervisor = RuntimeSupervisor::new("embedding-lifecycle-fixture");
        let failover_slot: Arc<std::sync::Mutex<Option<Arc<LocalEmbeddingFailover>>>> =
            Arc::new(std::sync::Mutex::new(None));
        let negative_backend: Arc<std::sync::Mutex<Option<Arc<dyn EmbeddingBackend>>>> =
            Arc::new(std::sync::Mutex::new(None));
        let persistence_slot: Arc<std::sync::Mutex<Option<Arc<SurrealDbProvider>>>> =
            Arc::new(std::sync::Mutex::new(None));
        let construction = async {
            let finalizer_failover = Arc::clone(&failover_slot);
            let finalizer_negative = Arc::clone(&negative_backend);
            let failover_scope = supervisor
                .register_with_finalizer(
                    "embedding-lifecycle-fixture",
                    "embedding-lifecycle-fixture",
                    async move {
                        let failover = finalizer_failover
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .take();
                        let negative = finalizer_negative
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .take();
                        let mut failures = Vec::new();
                        if let Some(failover) = failover {
                            if let Err(error) = failover.shutdown().await {
                                failures.push(format!("primary/backup shutdown: {error}"));
                            }
                        }
                        if let Some(negative) = negative {
                            if let Err(error) = negative.shutdown().await {
                                failures.push(format!("incompatible backend shutdown: {error}"));
                            }
                        }
                        if failures.is_empty() {
                            Ok(())
                        } else {
                            Err(anyhow::anyhow!(failures.join("; ")))
                        }
                    },
                )
                .await
                .map_err(|error| format!("register parent-owned embedding fixture scope: {error}"))?;
            let failover = LocalEmbeddingFailover::build(
                primary,
                &failover_config,
                failover_scope.clone(),
            )
            .await
            .map_err(|error| format!("real compatible local embedding members must initialize: {error}"))?;
            *failover_slot
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::clone(&failover));
            failover_scope
                .mark_ready()
                .map_err(|error| format!("mark embedding fixture scope ready: {error}"))?;
            let backend: Arc<dyn EmbeddingBackend> = failover.clone();
            let handle = Arc::new(SwappableEmbeddingBackend::new(backend));

            let endpoint = format!("surrealkv://{}", root.path().join("knowledge.surrealkv").display());
            let persistence = Arc::new(
                SurrealDbProvider::new(
                    &endpoint,
                    None,
                    None,
                    Some("embedding-lifecycle"),
                    Some("embedding-lifecycle"),
                )
                .await
                .map_err(|error| format!("open fixture-owned SurrealKV persistence: {error}"))?,
            );
            *persistence_slot
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::clone(&persistence));
            let persistence_layer: Arc<dyn PersistenceLayer> = persistence;

            Ok(Self {
                root: Arc::clone(&root),
                persistence_layer,
                supervisor: supervisor.clone(),
                persistence_slot: Arc::clone(&persistence_slot),
                failover_scope,
                failover,
                negative_backend,
                handle,
                incompatible: local_config(&incompatible_source, &incompatible_model, incompatible_dimension),
                primary_manifest,
                primary_weight_digest,
            })
        };

        match std::panic::AssertUnwindSafe(construction).catch_unwind().await {
            Ok(Ok(fixture)) => Ok(fixture),
            Ok(Err(error)) => match finalize_acquired(&supervisor, &persistence_slot, false).await {
                Ok(()) => Err(error),
                Err(cleanup) => Err(format!("{error}; cleanup: {cleanup}")),
            },
            Err(panic) => {
                let _ = finalize_acquired(&supervisor, &persistence_slot, false).await;
                std::panic::resume_unwind(panic);
            }
        }
    }

    async fn close(&self) -> Result<(), String> {
        let _fixture_root = self.root.path();
        let _failover_scope = self.failover_scope.snapshot();
        finalize_acquired(&self.supervisor, &self.persistence_slot, true).await
    }

    async fn exercise(&self) {
        let user = user().await;
        let owner = user
            .session_owner_key()
            .expect("fixture credential must produce a verified principal storage key");
        let kb = create_knowledge_base(self.persistence_layer.as_ref(), &owner).await;
        let ingest = IngestService::new(
            Arc::clone(&self.persistence_layer),
            Arc::clone(&self.handle),
            ChunkingStrategy::Document,
        );
        let matcher = Arc::new(VectorMatcher::new(Arc::clone(&self.handle), 0.0));

        track_document(self.persistence_layer.as_ref(), &owner, &kb.id, "primary-document").await;
        ingest
            .ingest_text(PRIMARY_TEXT, &owner, &kb.id, "primary-document".to_string())
            .await
            .expect("primary must perform actual local inference and durable ingestion");
        assert_searches(
            Arc::clone(&self.persistence_layer),
            Arc::clone(&matcher),
            &kb.id,
            QUERY_TEXT,
            &user,
            StatusCode::OK,
        )
        .await;
        assert_status(&self.failover.status(), "primary", 1);
        assert_eq!(
            stored_binding(self.persistence_layer.as_ref(), &owner, &kb.id).await,
            self.handle.fingerprint(),
            "primary ingestion must persist the same space served to queries"
        );

        let retired = self
            .failover
            .retire_primary(1)
            .await
            .expect("retire primary through its lifecycle scope");
        assert_status(&retired, "primary", 1);

        track_document(self.persistence_layer.as_ref(), &owner, &kb.id, "backup-document").await;
        ingest
            .ingest_text(BACKUP_TEXT, &owner, &kb.id, "backup-document".to_string())
            .await
            .expect("retired primary must promote the compatible backup and retry real ingestion");
        assert_status(&self.failover.status(), "backup", 2);
        assert_searches(
            Arc::clone(&self.persistence_layer),
            Arc::clone(&matcher),
            &kb.id,
            QUERY_TEXT,
            &user,
            StatusCode::OK,
        )
        .await;
        assert_eq!(
            stored_binding(self.persistence_layer.as_ref(), &owner, &kb.id).await,
            self.handle.fingerprint(),
            "compatible backup must preserve the durable embedding space"
        );

        let recovered = self
            .failover
            .recover_primary(2)
            .await
            .expect("recover primary while retaining the compatible backup as standby");
        assert_status(&recovered, "primary", 3);
        track_document(self.persistence_layer.as_ref(), &owner, &kb.id, "recovery-document").await;
        ingest
            .ingest_text(RECOVERY_TEXT, &owner, &kb.id, "recovery-document".to_string())
            .await
            .expect("recovered primary must resume actual local ingestion without a restart");
        assert_searches(
            Arc::clone(&self.persistence_layer),
            Arc::clone(&matcher),
            &kb.id,
            QUERY_TEXT,
            &user,
            StatusCode::OK,
        )
        .await;
        assert_eq!(
            stored_binding(self.persistence_layer.as_ref(), &owner, &kb.id).await,
            self.handle.fingerprint(),
            "first recovery must preserve the durable embedding space"
        );

        let second_retirement = self
            .failover
            .retire_primary(3)
            .await
            .expect("retire recovered primary for a second lifecycle cycle");
        assert_status(&second_retirement, "primary", 3);
        track_document(self.persistence_layer.as_ref(), &owner, &kb.id, "second-backup-document").await;
        ingest
            .ingest_text(
                SECOND_BACKUP_TEXT,
                &owner,
                &kb.id,
                "second-backup-document".to_string(),
            )
            .await
            .expect("second retired primary must promote the retained compatible backup");
        assert_status(&self.failover.status(), "backup", 4);
        assert_searches(
            Arc::clone(&self.persistence_layer),
            Arc::clone(&matcher),
            &kb.id,
            QUERY_TEXT,
            &user,
            StatusCode::OK,
        )
        .await;

        let second_recovery = self
            .failover
            .recover_primary(4)
            .await
            .expect("recover primary a second time with the backup still reusable");
        assert_status(&second_recovery, "primary", 5);
        track_document(self.persistence_layer.as_ref(), &owner, &kb.id, "second-recovery-document").await;
        ingest
            .ingest_text(
                SECOND_RECOVERY_TEXT,
                &owner,
                &kb.id,
                "second-recovery-document".to_string(),
            )
            .await
            .expect("second recovered primary must resume real local ingestion without a restart");
        assert_searches(
            Arc::clone(&self.persistence_layer),
            Arc::clone(&matcher),
            &kb.id,
            QUERY_TEXT,
            &user,
            StatusCode::OK,
        )
        .await;
        assert_eq!(
            stored_binding(self.persistence_layer.as_ref(), &owner, &kb.id).await,
            self.handle.fingerprint(),
            "second recovery must preserve the durable embedding space"
        );

        let binding_before = stored_binding(self.persistence_layer.as_ref(), &owner, &kb.id).await;
        let documents_before = self
            .persistence_layer
            .list_documents(&owner, &kb.id)
            .await
            .expect("list durable documents before incompatible query");
        let incompatible_backend = build_local_backend(&self.incompatible, self.failover_scope.clone())
            .expect("actual equal-dimension incompatible local model must construct");
        *self
            .negative_backend
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::clone(&incompatible_backend));
        let incompatible_manifest = asset_manifest(Path::new(&self.incompatible.models_dir));
        assert_ne!(
            incompatible_manifest,
            self.primary_manifest,
            "incompatible fixture must supply distinct actual model assets, not relabel the primary manifest"
        );
        assert_ne!(
            asset_weight_digest(Path::new(&self.incompatible.models_dir)),
            self.primary_weight_digest,
            "incompatible fixture must supply different actual model weights, not renamed primary BGE assets"
        );
        assert_ne!(
            incompatible_backend.model_id(),
            self.handle.pin().model_id(),
            "incompatible fixture must expose a different actual model identity"
        );
        let incompatible_vectors = incompatible_backend
            .embed(&[INCOMPATIBLE_QUERY])
            .await
            .expect("incompatible local model must perform actual inference before the query refusal probe");
        assert_eq!(incompatible_vectors.len(), 1, "incompatible local inference result count");
        assert_eq!(
            incompatible_vectors[0].len(),
            DIMENSION,
            "incompatible local model must have the same actual output dimension"
        );
        let incompatible_handle = Arc::new(SwappableEmbeddingBackend::new(incompatible_backend));
        let incompatible_matcher = Arc::new(VectorMatcher::new(incompatible_handle, 0.0));
        let response = search(
            Arc::clone(&self.persistence_layer),
            incompatible_matcher,
            &kb.id,
            INCOMPATIBLE_QUERY,
            user.clone(),
        )
        .await;
        assert_eq!(response.0, StatusCode::CONFLICT, "incompatible query body: {}", response.1);
        assert_eq!(
            response.1["error"]["code"],
            "embedding_space_mismatch",
            "equal-dimension different model must be refused explicitly: {}",
            response.1
        );
        assert_eq!(
            stored_binding(self.persistence_layer.as_ref(), &owner, &kb.id).await,
            binding_before,
            "incompatible refusal must not alter the durable KB binding"
        );
        let documents_after = self
            .persistence_layer
            .list_documents(&owner, &kb.id)
            .await
            .expect("list durable documents after incompatible query");
        assert_eq!(
            document_ids(&documents_after),
            document_ids(&documents_before),
            "incompatible refusal must preserve all durable document sources"
        );
        assert_searches(
            Arc::clone(&self.persistence_layer),
            Arc::clone(&matcher),
            &kb.id,
            QUERY_TEXT,
            &user,
            StatusCode::OK,
        )
        .await;
    }
}

async fn finalize_acquired(
    supervisor: &RuntimeSupervisor,
    persistence_slot: &Arc<std::sync::Mutex<Option<Arc<SurrealDbProvider>>>>,
    require_persistence: bool,
) -> Result<(), String> {
    let lifecycle = supervisor.shutdown().await;
    let persistence = persistence_slot
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();
    let persistence_report = match persistence {
        Some(persistence) => Some(persistence.close_scope().await),
        None => None,
    };
    if !lifecycle.released {
        return Err(format!("fixture lifecycle did not release: {lifecycle:?}"));
    }
    match persistence_report {
        Some(report) if report.scope_released && report.engine_close_acknowledged => Ok(()),
        Some(report) => Err(format!("fixture persistence did not close: {report:?}")),
        None if require_persistence => Err("fixture persistence was never acquired".to_string()),
        None => Ok(()),
    }
}

#[tokio::test]
async fn local_embedding_lifecycle_reuses_backup_across_two_cycles_and_refuses_an_equal_dimension_other_model() {
    let fixture = Fixture::new()
        .await
        .expect("fixture construction must clean acquired local resources before reporting failure");
    let outcome = std::panic::AssertUnwindSafe(fixture.exercise())
        .catch_unwind()
        .await;
    let cleanup = fixture.close().await;
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
    cleanup.expect("parent-owned fixture cleanup must finish after the scenario");
}

fn required_environment(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("missing required real local fixture prerequisite: {name}"))
}

fn required_model_dir(name: &str) -> PathBuf {
    let directory = PathBuf::from(required_environment(name));
    assert!(directory.is_dir(), "{name} must name an existing model directory: {}", directory.display());
    for file in MODEL_FILES {
        assert!(
            directory.join(file).is_file(),
            "{name} is missing required actual local model asset {}",
            directory.join(file).display()
        );
    }
    directory
}

fn copy_model_assets(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).expect("create fixture-owned primary model directory");
    for file in MODEL_FILES {
        std::fs::copy(source.join(file), destination.join(file))
            .unwrap_or_else(|error| panic!("copy primary model asset {file}: {error}"));
    }
}

fn asset_manifest(directory: &Path) -> String {
    let mut manifest = sha2::Sha256::new();
    for file in MODEL_FILES {
        let bytes = std::fs::read(directory.join(file)).unwrap_or_else(|error| {
            panic!(
                "cannot read real local model asset {} for manifest prerequisite: {error}",
                directory.join(file).display()
            )
        });
        let digest = hex_encode(&sha2::Sha256::digest(&bytes));
        manifest.update(file.as_bytes());
        manifest.update([0]);
        manifest.update(digest.as_bytes());
        manifest.update([0]);
    }
    hex_encode(&manifest.finalize())
}

fn asset_weight_digest(directory: &Path) -> String {
    let bytes = std::fs::read(directory.join(MODEL_WEIGHT_FILE)).unwrap_or_else(|error| {
        panic!(
            "cannot read real local model weights {} for digest prerequisite: {error}",
            directory.join(MODEL_WEIGHT_FILE).display()
        )
    });
    hex_encode(&sha2::Sha256::digest(&bytes))
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn local_config(models_dir: &Path, model: &str, vector_dimension: usize) -> EmbeddingConfig {
    EmbeddingConfig {
        backend: "fastembed".to_string(),
        model: model.to_string(),
        api_key: None,
        api_key_env: None,
        base_url: None,
        vector_dimension,
        batch_size: 32,
        models_dir: models_dir.display().to_string(),
        strict_models_dir: true,
    }
}

async fn create_knowledge_base(persistence: &dyn PersistenceLayer, owner: &str) -> KnowledgeBase {
    let now = chrono::Utc::now().to_rfc3339();
    let knowledge_base = KnowledgeBase {
        id: uuid::Uuid::new_v4().to_string(),
        owner_id: owner.to_string(),
        name: "local-embedding-lifecycle".to_string(),
        description: Some("real local embedding failover fixture".to_string()),
        config: KbConfig::default(),
        created_at: now.clone(),
        updated_at: now,
    };
    persistence
        .save_knowledge_base(&knowledge_base)
        .await
        .expect("save owner-scoped knowledge base");
    knowledge_base
}

async fn track_document(
    persistence: &dyn PersistenceLayer,
    owner: &str,
    kb_id: &str,
    document_id: &str,
) {
    let now = chrono::Utc::now().to_rfc3339();
    persistence
        .save_document(&KnowledgeDocument {
            id: document_id.to_string(),
            owner_id: owner.to_string(),
            kb_id: kb_id.to_string(),
            filename: format!("{document_id}.txt"),
            file_path: None,
            mime_type: Some("text/plain".to_string()),
            chunk_count: 0,
            status: DocumentStatus::Processing,
            created_at: now.clone(),
            updated_at: now,
        })
        .await
        .expect("record durable document source");
}

async fn stored_binding(
    persistence: &dyn PersistenceLayer,
    owner: &str,
    kb_id: &str,
) -> universal_agent_runtime::uar::domain::knowledge::EmbeddingFingerprint {
    persistence
        .get_knowledge_base(owner, kb_id)
        .await
        .expect("read owner-scoped knowledge base")
        .expect("knowledge base exists")
        .config
        .indexed_embedding
        .expect("ingestion must bind the embedding space")
}

async fn assert_searches(
    persistence: Arc<dyn PersistenceLayer>,
    matcher: Arc<VectorMatcher>,
    kb_id: &str,
    query: &str,
    user: &UserContext,
    expected: StatusCode,
) {
    let response = search(persistence, matcher, kb_id, query, user.clone()).await;
    assert_eq!(response.0, expected, "knowledge search body: {}", response.1);
    assert!(
        response.1["results"].as_array().is_some_and(|results| !results.is_empty()),
        "real local query returned no durable results: {}",
        response.1
    );
}

async fn search(
    persistence: Arc<dyn PersistenceLayer>,
    matcher: Arc<VectorMatcher>,
    kb_id: &str,
    query: &str,
    user: UserContext,
) -> (StatusCode, serde_json::Value) {
    let state = Arc::new(KnowledgeApiState {
        persistence,
        vector_matcher: matcher,
        ingestion_pool: None,
        realtime_bus: None,
    });
    let response = build_router()
        .with_state(state)
        .layer(Extension(user))
        .oneshot(
            Request::post(format!("/{kb_id}/search"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"query": query, "limit": 8, "min_score": 0.0}).to_string(),
                ))
                .expect("build knowledge search request"),
        )
        .await
        .expect("run production knowledge search route");
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read knowledge search response");
    let json = serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);
    (status, json)
}

async fn user() -> UserContext {
    let secret = "embedding-lifecycle-fixture-only-auth-secret";
    let config: SecurityConfig = serde_json::from_value(serde_json::json!({
        "jwt_required": true,
        "jwt_secret": secret,
        "jwt_issuer": "embedding-lifecycle-fixture",
    }))
    .expect("construct fixture credential verifier");
    let _ = authenticate_bearer_context(&config, "invalid").await;
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &serde_json::json!({
            "iss": "embedding-lifecycle-fixture",
            "sub": SUBJECT,
            "roles": ["user"],
            "exp": chrono::Utc::now().timestamp() + 3600,
        }),
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("sign fixture-only verified principal");
    authenticate_bearer_context(&config, &token)
        .await
        .expect("verify fixture principal through the production boundary")
}

fn assert_status(status: &impl serde::Serialize, member: &str, generation: u64) {
    let status = serde_json::to_value(status).expect("serialize sanitized failover status");
    assert_eq!(status["active_member"], member, "failover status: {status}");
    assert_eq!(status["generation"], generation, "failover status: {status}");
}

fn document_ids(documents: &[KnowledgeDocument]) -> Vec<String> {
    let mut ids = documents.iter().map(|document| document.id.clone()).collect::<Vec<_>>();
    ids.sort();
    ids
}
