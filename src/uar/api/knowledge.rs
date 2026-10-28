//! REST API routes for Knowledge Base and Document management.
//!
//! Provides CRUD operations for knowledge bases and document ingestion.

use axum::{
    Json, Router,
    extract::{Extension, Multipart, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response, sse::{Event, Sse}},
    routing::{get, post},
};
use futures::Stream;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::uar::persistence::ingestion_records::{IngestionJob, IngestionState, IngestionStoreError};
use std::{convert::Infallible, sync::Arc};

use crate::uar::{
    domain::knowledge::{
        DocumentStatus, EmbeddingSpaceMismatch, KbConfig, KnowledgeBase, KnowledgeDocument,
        KnowledgeMatch, KnowledgeRegeneration, MAX_RETRIEVAL_TOP_K,
    },
    persistence::PersistenceLayer,
    rag::{
        chunking::ChunkingStrategy,
        ingestion_worker::IngestionWorkerPool,
        pipeline::{RagRetrievalPipeline, RetrievalBackend},
    },
    realtime::{EntityTopic, LiveAction, RealtimeBus},
    runtime::matching::VectorMatcher,
    security::claims::UserContext,
};

// =============================================================================
// State Wrapper (shares persistence layer)
// =============================================================================

/// Shared state for knowledge base API handlers.
#[derive(Clone, Debug)]
pub struct KnowledgeApiState {
    pub persistence: Arc<dyn PersistenceLayer>,
    pub vector_matcher: Arc<VectorMatcher>,
    pub ingestion_pool: Option<Arc<IngestionWorkerPool>>,
    /// Private ingestion wakeups. The API treats these as invalidations only
    /// and always re-reads through the caller's persistence authorization.
    pub realtime_bus: Option<Arc<dyn RealtimeBus>>,
}

// =============================================================================
// Request/Response DTOs
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateKnowledgeBaseRequest {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub config: Option<KbConfigRequest>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateKnowledgeBaseRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub config: Option<KbConfigRequest>,
}

#[derive(Debug, Deserialize)]
pub struct KbConfigRequest {
    pub embedding_provider: Option<String>,
    pub embedding_model: Option<String>,
    pub vector_dimensions: Option<usize>,
    pub file_processor: Option<String>,
    pub chunk_strategy: Option<String>,
    pub chunk_size: Option<usize>,
    /// Minimum similarity for a chunk to enter an agent's prompt (0.0 to 1.0).
    pub retrieval_min_score: Option<f32>,
    /// Chunks retrieved per agent turn (1 to `MAX_RETRIEVAL_TOP_K`).
    pub retrieval_top_k: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct KnowledgeBaseResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub config: KbConfigResponse,
    pub created_at: String,
    pub updated_at: String,
    /// Number of documents currently stored in this KB (regardless of indexing status).
    pub document_count: usize,
}

#[derive(Debug, Serialize)]
pub struct KbConfigResponse {
    pub embedding_provider: String,
    pub embedding_model: String,
    pub vector_dimensions: Option<usize>,
    pub file_processor: String,
    pub chunk_strategy: String,
    pub retrieval_min_score: Option<f32>,
    pub retrieval_top_k: Option<usize>,
    pub indexed_embedding: Option<crate::uar::domain::knowledge::EmbeddingFingerprint>,
    pub regeneration: Option<KnowledgeRegeneration>,
}

#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub id: String,
    pub kb_id: String,
    pub filename: String,
    pub file_path: Option<String>,
    pub mime_type: Option<String>,
    pub chunk_count: usize,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default = "default_min_score")]
    pub min_score: f32,
}

fn default_limit() -> usize {
    5
}
fn default_min_score() -> f32 {
    0.7
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub content: String,
    pub score: f32,
    pub metadata: serde_json::Value,
    pub document_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub offset: usize,
    #[serde(default = "default_page_limit")]
    pub limit: usize,
}

fn default_page_limit() -> usize {
    50
}

// =============================================================================
// Router Builder
// =============================================================================

/// Build the knowledge base API router.
pub fn build_router() -> Router<Arc<KnowledgeApiState>> {
    Router::new()
        // Knowledge Base CRUD
        .route("/", get(list_knowledge_bases).post(create_knowledge_base))
        .route(
            "/{id}",
            get(get_knowledge_base)
                .put(update_knowledge_base)
                .delete(delete_knowledge_base),
        )
        // Documents
        .route("/{id}/documents", get(list_documents).post(upload_document))
        .route(
            "/{id}/documents/{doc_id}",
            get(get_document).delete(delete_document),
        )
        .route("/{id}/documents/{doc_id}/ingestion", get(get_ingestion_job))
        .route(
            "/{id}/documents/{doc_id}/ingestion/events",
            get(stream_ingestion_job),
        )
        .route("/{id}/documents/{doc_id}/retry", post(retry_ingestion))
        .route("/{id}/regeneration", get(get_regeneration).post(begin_regeneration))
        // Search
        .route("/{id}/search", post(search_knowledge_base))
}

// =============================================================================
// Knowledge Base Handlers
// =============================================================================

/// GET / - List all knowledge bases
async fn list_knowledge_bases(
    State(state): State<Arc<KnowledgeApiState>>,
    Extension(user): Extension<UserContext>,
) -> Result<Json<Vec<KnowledgeBaseResponse>>, (StatusCode, String)> {
    let owner_key = user.session_owner_key().map_err(|error| (StatusCode::UNAUTHORIZED, error.to_string()))?;
    let kbs = state
        .persistence
        .list_knowledge_bases(&owner_key)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut responses: Vec<KnowledgeBaseResponse> = Vec::with_capacity(kbs.len());
    for kb in kbs {
        let count = state
            .persistence
            .count_documents(&owner_key, &kb.id)
            .await
            .unwrap_or_else(|err| {
                tracing::warn!(kb_id = %kb.id, error = %err, "count_documents failed; reporting 0");
                0
            });
        responses.push(kb_to_response(kb, count));
    }
    Ok(Json(responses))
}

/// POST / - Create a new knowledge base
async fn create_knowledge_base(
    State(state): State<Arc<KnowledgeApiState>>,
    Extension(user): Extension<UserContext>,
    Json(req): Json<CreateKnowledgeBaseRequest>,
) -> Result<(StatusCode, Json<KnowledgeBaseResponse>), (StatusCode, String)> {
    let owner_key = user.session_owner_key().map_err(|error| (StatusCode::UNAUTHORIZED, error.to_string()))?;
    // Check if name already exists
    if let Ok(Some(_)) = state
        .persistence
        .get_knowledge_base_by_name(&owner_key, &req.name)
        .await
    {
        return Err((
            StatusCode::CONFLICT,
            format!(
                "Knowledge base with name '{name}' already exists",
                name = req.name
            ),
        ));
    }

    if let Some(cfg) = &req.config {
        validate_retrieval_settings(cfg)?;
    }
    let now = chrono::Utc::now().to_rfc3339();
    let config = build_kb_config(req.config);

    let kb = KnowledgeBase {
        id: uuid::Uuid::new_v4().to_string(),
        owner_id: owner_key,
        name: req.name,
        description: req.description,
        config,
        created_at: now.clone(),
        updated_at: now,
    };

    state
        .persistence
        .save_knowledge_base(&kb)
        .await
        .map_err(ingestion_api_error)?;

    tracing::info!("Created knowledge base: {} ({})", kb.name, kb.id);
    Ok((StatusCode::CREATED, Json(kb_to_response(kb, 0))))
}

/// GET /{id} - Get a knowledge base by ID
async fn get_knowledge_base(
    State(state): State<Arc<KnowledgeApiState>>,
    Extension(user): Extension<UserContext>,
    Path(id): Path<String>,
) -> Result<Json<KnowledgeBaseResponse>, (StatusCode, String)> {
    let owner_key = user.session_owner_key().map_err(|error| (StatusCode::UNAUTHORIZED, error.to_string()))?;
    let kb = state
        .persistence
        .get_knowledge_base(&owner_key, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::NOT_FOUND,
            format!("Knowledge base '{id}' not found"),
        ))?;

    let count = state
        .persistence
        .count_documents(&owner_key, &kb.id)
        .await
        .unwrap_or(0);
    Ok(Json(kb_to_response(kb, count)))
}

/// PUT /{id} - Update a knowledge base
async fn update_knowledge_base(
    State(state): State<Arc<KnowledgeApiState>>,
    Extension(user): Extension<UserContext>,
    Path(id): Path<String>,
    Json(req): Json<UpdateKnowledgeBaseRequest>,
) -> Result<Json<KnowledgeBaseResponse>, (StatusCode, String)> {
    let owner_key = user.session_owner_key().map_err(|error| (StatusCode::UNAUTHORIZED, error.to_string()))?;
    let mut kb = state
        .persistence
        .get_knowledge_base(&owner_key, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::NOT_FOUND,
            format!("Knowledge base '{id}' not found"),
        ))?;

    if kb.regeneration_unavailable() {
        return Err((StatusCode::CONFLICT, "Knowledge regeneration is incomplete".to_string()));
    }
    // Apply updates
    if let Some(name) = req.name {
        // Check uniqueness
        if let Ok(Some(existing)) = state
            .persistence
            .get_knowledge_base_by_name(&owner_key, &name)
            .await
            && existing.id != kb.id
        {
            return Err((
                StatusCode::CONFLICT,
                format!("Knowledge base with name '{name}' already exists"),
            ));
        }
        kb.name = name;
    }
    if let Some(desc) = req.description {
        kb.description = Some(desc);
    }
    if let Some(cfg_req) = req.config {
        validate_retrieval_settings(&cfg_req)?;
        kb.config = merge_kb_config(kb.config, cfg_req);
    }
    kb.updated_at = chrono::Utc::now().to_rfc3339();

    state
        .persistence
        .save_knowledge_base(&kb)
        .await
        .map_err(ingestion_api_error)?;

    let count = state
        .persistence
        .count_documents(&owner_key, &kb.id)
        .await
        .unwrap_or(0);
    Ok(Json(kb_to_response(kb, count)))
}

/// DELETE /{id} - Delete a knowledge base
async fn delete_knowledge_base(
    State(state): State<Arc<KnowledgeApiState>>,
    Extension(user): Extension<UserContext>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let owner_key = user.session_owner_key().map_err(|error| (StatusCode::UNAUTHORIZED, error.to_string()))?;
    // Check exists
    let _ = state
        .persistence
        .get_knowledge_base(&owner_key, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::NOT_FOUND,
            format!("Knowledge base '{id}' not found"),
        ))?;

    state
        .persistence
        .delete_knowledge_base(&owner_key, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tracing::info!("Deleted knowledge base: {id}");
    Ok(StatusCode::NO_CONTENT)
}

// =============================================================================
// Document Handlers
// =============================================================================

/// GET /{id}/documents - List documents in a knowledge base
async fn list_documents(
    State(state): State<Arc<KnowledgeApiState>>,
    Extension(user): Extension<UserContext>,
    Path(kb_id): Path<String>,
    Query(_query): Query<ListQuery>,
) -> Result<Json<Vec<DocumentResponse>>, (StatusCode, String)> {
    let owner_key = user.session_owner_key().map_err(|error| (StatusCode::UNAUTHORIZED, error.to_string()))?;
    // Verify KB exists
    let _ = state
        .persistence
        .get_knowledge_base(&owner_key, &kb_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::NOT_FOUND,
            format!("Knowledge base '{kb_id}' not found"),
        ))?;

    let docs = state
        .persistence
        .list_documents(&owner_key, &kb_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let responses: Vec<DocumentResponse> = docs.into_iter().map(doc_to_response).collect();
    Ok(Json(responses))
}

/// POST /{id}/documents - Upload a document (multipart form)
async fn upload_document(
    State(state): State<Arc<KnowledgeApiState>>,
    Extension(user): Extension<UserContext>,
    Path(kb_id): Path<String>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<DocumentResponse>), (StatusCode, String)> {
    let owner_key = user.session_owner_key().map_err(|error| (StatusCode::UNAUTHORIZED, error.to_string()))?;
    let eligibility = state.persistence.durable_store_eligibility();
    if !eligibility.permits_durable_work() || !state.persistence.supports_durable_ingestion() {
        tracing::warn!(durable_store_eligibility = eligibility.as_str(), "Durable ingestion upload refused");
        return Err((StatusCode::SERVICE_UNAVAILABLE, "Durable ingestion storage is unavailable".to_string()));
    }
    // Verify KB exists
    let _ = state
        .persistence
        .get_knowledge_base(&owner_key, &kb_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::NOT_FOUND,
            format!("Knowledge base '{kb_id}' not found"),
        ))?;

    // Read multipart file
    let mut filename = String::new();
    let mut mime_type = None;
    let mut file_data = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        let field_name = field.name().unwrap_or_default().to_string();
        if field_name == "file" {
            filename = field.file_name().unwrap_or("uploaded_file").to_string();
            mime_type = field.content_type().map(ToString::to_string);
            file_data = field
                .bytes()
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
                .to_vec();
        }
    }

    if filename.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "No file field in multipart form".to_string(),
        ));
    }

    let now = chrono::Utc::now().to_rfc3339();
    let doc = KnowledgeDocument {
        id: uuid::Uuid::new_v4().to_string(),
        owner_id: owner_key,
        kb_id: kb_id.clone(),
        filename: filename.clone(),
        file_path: None, // Immutable source bytes live in the database.
        mime_type,
        chunk_count: 0,
        status: DocumentStatus::Pending,
        created_at: now.clone(),
        updated_at: now,
    };

    // A 202 means all three records committed: immutable source, document and job.
    state.persistence.accept_ingestion(&doc, &file_data).await.map_err(ingestion_api_error)?;
    if let Some(pool) = &state.ingestion_pool {
        pool.notify_committed();
    }

    tracing::info!("Document uploaded: {} -> KB {}", doc.id, kb_id);
    Ok((StatusCode::ACCEPTED, Json(doc_to_response(doc))))
}

/// GET `/{id}/documents/{doc_id}` - Get document status
async fn get_document(
    State(state): State<Arc<KnowledgeApiState>>,
    Extension(user): Extension<UserContext>,
    Path((kb_id, doc_id)): Path<(String, String)>,
) -> Result<Json<DocumentResponse>, (StatusCode, String)> {
    let owner_key = user.session_owner_key().map_err(|error| (StatusCode::UNAUTHORIZED, error.to_string()))?;
    let doc = state
        .persistence
        .get_document(&owner_key, &doc_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::NOT_FOUND,
            format!("Document '{doc_id}' not found"),
        ))?;

    // Verify document belongs to the KB
    if doc.kb_id != kb_id {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Document '{doc_id}' not found in KB '{kb_id}'"),
        ));
    }

    Ok(Json(doc_to_response(doc)))
}

fn ingestion_api_error(error: anyhow::Error) -> (StatusCode, String) {
    if let Some(cause) = error.downcast_ref::<crate::uar::rag::regeneration::RegenerationError>() {
        return (match cause {
            crate::uar::rag::regeneration::RegenerationError::Conflict => StatusCode::CONFLICT,
            crate::uar::rag::regeneration::RegenerationError::SourceMissing => StatusCode::UNPROCESSABLE_ENTITY,
        }, cause.to_string());
    }
    let code = match error.downcast_ref::<IngestionStoreError>() {
        Some(IngestionStoreError::Unsupported) => StatusCode::SERVICE_UNAVAILABLE,
        Some(IngestionStoreError::MissingKnowledgeBase) => StatusCode::NOT_FOUND,
        Some(IngestionStoreError::Revoked) => StatusCode::CONFLICT,
        None => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (code, error.to_string())
}

/// Owner-scoped durable job state; immutable source bytes are never returned here.
async fn get_ingestion_job(
    State(state): State<Arc<KnowledgeApiState>>,
    Extension(user): Extension<UserContext>,
    Path((kb_id, doc_id)): Path<(String, String)>,
) -> Result<Json<IngestionJob>, (StatusCode, String)> {
    let owner_key = user.session_owner_key().map_err(|error| (StatusCode::UNAUTHORIZED, error.to_string()))?;
    let job = authorized_ingestion_job(&state, &owner_key, &kb_id, &doc_id).await?;
    Ok(Json(job))
}

/// Owner-authorized ingestion status stream. The private lifecycle topic is a
/// wakeup only: its event data can describe another principal's row, so every
/// emitted status comes from the existing owner-scoped durable read instead.
async fn stream_ingestion_job(
    State(state): State<Arc<KnowledgeApiState>>,
    Extension(user): Extension<UserContext>,
    Path((kb_id, doc_id)): Path<(String, String)>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>> + Send>, (StatusCode, String)> {
    let owner_key = user
        .session_owner_key()
        .map_err(|error| (StatusCode::UNAUTHORIZED, error.to_string()))?;
    let bus = state.realtime_bus.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "private ingestion status notifications are unavailable".to_string(),
    ))?;
    let topic = EntityTopic::PrincipalIngestionJobs;
    // Subscribe before the durable snapshot. The broadcast receiver buffers a
    // commit that races that snapshot, and the stream reconciles it below.
    let receiver = bus.subscribe(topic).ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "private ingestion status topic is unavailable".to_string(),
    ))?;
    let initial_topic_state = bus.topic_state(topic);
    let initial = authorized_ingestion_job(&state, &owner_key, &kb_id, &doc_id).await?;
    let bus = Arc::clone(bus);

    let stream = async_stream::stream! {
        let mut receiver = receiver;
        let mut initial = Some(initial);
        let mut initial_gap = initial_topic_state.filter(|topic_state| !topic_state.ready);
        loop {
            if let Some(job) = initial.take() {
                yield Ok::<Event, Infallible>(ingestion_status_event(
                    &job,
                    "initial",
                    bus.topic_state(topic),
                ));
                if let Some(topic_state) = initial_gap.take() {
                    yield Ok(ingestion_gap_event(
                        "subscription_unavailable",
                        Some(topic_state),
                    ));
                }
                continue;
            }

            let mut gap_reason = match receiver.recv().await {
                Ok(event) if matches!(event.action, LiveAction::Reset) => {
                    Some("lifecycle_transition")
                }
                Ok(_) => None,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                    Some("consumer_lagged")
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    yield Ok(ingestion_gap_event(
                        "subscription_closed",
                        bus.topic_state(topic),
                    ));
                    break;
                }
            };
            let topic_state = bus.topic_state(topic);
            if gap_reason.is_none() && topic_state.as_ref().is_some_and(|state| !state.ready) {
                gap_reason = Some("subscription_unavailable");
            }
            if let Some(reason) = gap_reason {
                yield Ok(ingestion_gap_event(reason, topic_state.clone()));
            }

            // Never serialize the bus event. PostgreSQL intentionally emits
            // an opaque wildcard ID for lifecycle work, and Surreal live rows
            // may belong to another principal. This durable read reapplies the
            // exact owner and KB checks for this request.
            match authorized_ingestion_job(&state, &owner_key, &kb_id, &doc_id).await {
                Ok(job) => yield Ok(ingestion_status_event(
                    &job,
                    if gap_reason.is_some() { "reconciled" } else { "committed_change" },
                    topic_state,
                )),
                Err((StatusCode::NOT_FOUND, _)) => {
                    yield Ok(ingestion_gap_event(
                        "ingestion_job_unavailable",
                        topic_state,
                    ));
                    break;
                }
                Err(_) => {
                    yield Ok(ingestion_gap_event(
                        "authoritative_status_unavailable",
                        topic_state,
                    ));
                    break;
                }
            }
        }
    };

    Ok(Sse::new(stream))
}

async fn authorized_ingestion_job(
    state: &KnowledgeApiState,
    owner_key: &str,
    kb_id: &str,
    doc_id: &str,
) -> Result<IngestionJob, (StatusCode, String)> {
    state
        .persistence
        .get_ingestion_job(owner_key, doc_id)
        .await
        .map_err(ingestion_api_error)?
        .filter(|job| job.kb_id == kb_id)
        .ok_or((
            StatusCode::NOT_FOUND,
            "Durable ingestion job not found; legacy documents may require reupload".to_string(),
        ))
}

fn ingestion_status_event(
    job: &IngestionJob,
    delivery: &str,
    topic_state: Option<crate::uar::realtime::lifecycle::TopicState>,
) -> Event {
    Event::default().event("ingestion.status").data(json!({
        "version": 1,
        "topic": EntityTopic::PrincipalIngestionJobs.as_str(),
        "delivery": delivery,
        "generation": topic_state.as_ref().map(|state| state.generation),
        "ready": topic_state.as_ref().map(|state| state.ready),
        "job": job,
    }).to_string())
}

fn ingestion_gap_event(
    reason: &str,
    topic_state: Option<crate::uar::realtime::lifecycle::TopicState>,
) -> Event {
    Event::default().event("ingestion.gap").data(json!({
        "version": 1,
        "topic": EntityTopic::PrincipalIngestionJobs.as_str(),
        "reason": reason,
        "requires_snapshot": true,
        "generation": topic_state.as_ref().map(|state| state.generation),
        "ready": topic_state.as_ref().map(|state| state.ready),
    }).to_string())
}

/// Retry committed bytes after an operator has corrected the processing configuration.
async fn retry_ingestion(
    State(state): State<Arc<KnowledgeApiState>>,
    Extension(user): Extension<UserContext>,
    Path((kb_id, doc_id)): Path<(String, String)>,
) -> Result<(StatusCode, Json<IngestionJob>), (StatusCode, String)> {
    let owner_key = user.session_owner_key().map_err(|error| (StatusCode::UNAUTHORIZED, error.to_string()))?;
    let before = state.persistence.get_ingestion_job(&owner_key, &doc_id).await
        .map_err(ingestion_api_error)?.filter(|job| job.kb_id == kb_id)
        .ok_or((StatusCode::NOT_FOUND, "Durable ingestion job not found; reupload required".to_string()))?;
    if before.regeneration.is_some() {
        return Err((StatusCode::CONFLICT, "Retry the knowledge base regeneration action".to_string()));
    }
    if !matches!(before.state, IngestionState::Failed | IngestionState::Retryable) {
        return Err((StatusCode::CONFLICT, "Only failed or retryable ingestion can be retried".to_string()));
    }
    let after = before.retry();
    if !state.persistence.transition_ingestion(&before, &after).await.map_err(ingestion_api_error)? {
        return Err((StatusCode::CONFLICT, "Ingestion changed concurrently".to_string()));
    }
    if let Some(pool) = &state.ingestion_pool { pool.notify_committed(); }
    Ok((StatusCode::ACCEPTED, Json(after)))
}

/// DELETE `/{id}/documents/{doc_id}` - Delete a document
async fn delete_document(
    State(state): State<Arc<KnowledgeApiState>>,
    Extension(user): Extension<UserContext>,
    Path((kb_id, doc_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, String)> {
    let owner_key = user.session_owner_key().map_err(|error| (StatusCode::UNAUTHORIZED, error.to_string()))?;
    // Verify document exists and belongs to KB
    let doc = state
        .persistence
        .get_document(&owner_key, &doc_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::NOT_FOUND,
            format!("Document '{doc_id}' not found"),
        ))?;

    if doc.kb_id != kb_id {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Document '{doc_id}' not found in KB '{kb_id}'"),
        ));
    }

    // Delete document and its chunks
    state
        .persistence
        .delete_document(&owner_key, &doc_id)
        .await
        .map_err(ingestion_api_error)?;

    tracing::info!("Deleted document: {doc_id} from KB {kb_id}");
    Ok(StatusCode::NO_CONTENT)
}

// =============================================================================
// Regeneration and Search Handlers
// =============================================================================

async fn get_regeneration(
    State(state): State<Arc<KnowledgeApiState>>,
    Extension(user): Extension<UserContext>, Path(id): Path<String>,
) -> Result<Json<Option<KnowledgeRegeneration>>, (StatusCode, String)> {
    let owner = user.session_owner_key().map_err(|_| (StatusCode::UNAUTHORIZED, "Owner authority unavailable".to_string()))?;
    let kb = state.persistence.get_knowledge_base(&owner, &id).await.map_err(ingestion_api_error)?
        .ok_or((StatusCode::NOT_FOUND, "Knowledge base not found".to_string()))?;
    Ok(Json(kb.config.regeneration))
}

async fn begin_regeneration(
    State(state): State<Arc<KnowledgeApiState>>,
    Extension(user): Extension<UserContext>, Path(id): Path<String>,
) -> Result<(StatusCode, Json<KnowledgeRegeneration>), (StatusCode, String)> {
    let owner = user.session_owner_key().map_err(|_| (StatusCode::UNAUTHORIZED, "Owner authority unavailable".to_string()))?;
    let pool = state.ingestion_pool.as_ref().ok_or((StatusCode::SERVICE_UNAVAILABLE,
        "Durable ingestion worker is unavailable".to_string()))?;
    if !state.persistence.durable_store_eligibility().permits_durable_work()
        || !state.persistence.supports_durable_ingestion() {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "Durable ingestion storage is unavailable".to_string()));
    }
    let target = state.vector_matcher.embedding_fingerprint();
    let cohort = state.persistence.begin_knowledge_regeneration(&owner, &id, &target).await.map_err(|error| {
        if let Some(cause) = error.downcast_ref::<crate::uar::rag::regeneration::RegenerationError>() {
            (match cause { crate::uar::rag::regeneration::RegenerationError::Conflict => StatusCode::CONFLICT,
                crate::uar::rag::regeneration::RegenerationError::SourceMissing => StatusCode::UNPROCESSABLE_ENTITY }, cause.to_string())
        } else { ingestion_api_error(error) }
    })?;
    pool.notify_committed();
    Ok((StatusCode::ACCEPTED, Json(cohort)))
}

/// Adapts `(PersistenceLayer, VectorMatcher)` to [`RetrievalBackend`] so
/// [`RagRetrievalPipeline`] (CH-11 rag-hardening) can run one search per
/// decomposed sub-query without depending on either concrete type.
struct KbSearchBackend<'a> {
    persistence: &'a dyn PersistenceLayer,
    owner_id: &'a str,
    kb_id: &'a str,
    embedding: Arc<dyn crate::uar::rag::embeddings::EmbeddingBackend>,
}

#[async_trait::async_trait]
impl RetrievalBackend for KbSearchBackend<'_> {
    async fn search_one(
        &self,
        sub_query: &str,
        limit: usize,
        min_score: f32,
    ) -> anyhow::Result<Vec<KnowledgeMatch>> {
        // Embed with the same backend ingestion used (`llm.embedding`), not
        // the skill-matching facade, which is disabled without `local-models`.
        // The pin is one coherent snapshot; a mid-request backend swap cannot
        // interleave into this sub-query.
        let kb = self.persistence.get_knowledge_base(self.owner_id, self.kb_id).await?
            .ok_or_else(|| anyhow::anyhow!("knowledge base is unavailable"))?;
        anyhow::ensure!(!kb.regeneration_unavailable(), "knowledge regeneration is incomplete");
        kb.ensure_embedding_space(&self.embedding.fingerprint())?;
        let query_vec = self.embedding.embed_one(sub_query).await?;
        self.persistence
            .search_knowledge_scoped_in_space(self.owner_id, &[self.kb_id], &query_vec, limit, min_score,
                &self.embedding.fingerprint())
            .await
    }
}

/// POST /{id}/search - Vector search within a knowledge base
///
/// Runs through [`RagRetrievalPipeline`] (CH-11 rag-hardening): the query is
/// decomposed into sub-queries, each searched independently, results are
/// deduped/fused, verification-annotated, and the decision is audit-logged
/// (`rag.retrieval.decision`, Rule 34). Verification only annotates by
/// default (`RagRetrievalConfig::default()`) — it does not drop results, so
/// existing callers see the same result set as before this pipeline existed.
async fn search_knowledge_base(
    State(state): State<Arc<KnowledgeApiState>>,
    Extension(user): Extension<UserContext>,
    Path(kb_id): Path<String>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, Response> {
    let owner_key = user.session_owner_key().map_err(|error| (StatusCode::UNAUTHORIZED, error.to_string()).into_response())?;
    // Verify KB exists
    let kb = state
        .persistence
        .get_knowledge_base(&owner_key, &kb_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("Knowledge base '{kb_id}' not found"),
            )
                .into_response()
        })?;

    // Comparing vectors from different embedding spaces yields meaningless
    // scores that fall under `min_score`, i.e. a silent empty result.
    if kb.regeneration_unavailable() {
        return Err((StatusCode::CONFLICT, Json(json!({"error": {"code": "knowledge_regeneration_incomplete",
            "kb_id": kb_id, "regeneration": kb.config.regeneration}}))).into_response());
    }
    let embedding = state.vector_matcher.pin_embedding();
    kb.ensure_embedding_space(&embedding.fingerprint())
        .map_err(embedding_mismatch_response)?;

    tracing::debug!(
        "Search in KB '{}': query='{}', limit={}",
        kb.name,
        req.query,
        req.limit
    );

    let backend = KbSearchBackend {
        persistence: state.persistence.as_ref(),
        owner_id: owner_key.as_str(),
        kb_id: kb_id.as_str(),
        embedding,
    };
    let matches = RagRetrievalPipeline::new()
        .retrieve(&backend, &kb_id, &req.query, req.limit, req.min_score)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())?;

    // Transform to response
    let results = matches
        .into_iter()
        .map(|m| SearchResult {
            content: m.chunk.content,
            score: m.score,
            metadata: m.chunk.metadata.unwrap_or_else(|| serde_json::json!({})),
            document_id: m.chunk.document_id,
        })
        .collect();

    Ok(Json(SearchResponse { results }))
}

// =============================================================================
// Helper Functions
// =============================================================================

/// 409: the KB exists but cannot be searched with the active embedding space.
fn embedding_mismatch_response(err: EmbeddingSpaceMismatch) -> Response {
    tracing::warn!(kb_id = %err.kb_id, indexed = %err.indexed, active = %err.active,
        "knowledge base search refused: embedding space mismatch");
    (
        StatusCode::CONFLICT,
        Json(serde_json::json!({
            "error": {
                "code": "embedding_space_mismatch",
                "message": err.to_string(),
                "kb_id": err.kb_id,
                "indexed_embedding": err.indexed,
                "active_embedding": err.active,
            }
        })),
    )
        .into_response()
}

fn kb_to_response(kb: KnowledgeBase, document_count: usize) -> KnowledgeBaseResponse {
    KnowledgeBaseResponse {
        id: kb.id,
        name: kb.name,
        description: kb.description,
        config: KbConfigResponse {
            embedding_provider: kb.config.embedding_provider,
            embedding_model: kb.config.embedding_model,
            vector_dimensions: kb.config.vector_dimensions,
            file_processor: kb.config.file_processor,
            chunk_strategy: format!("{:?}", kb.config.chunk_strategy),
            retrieval_min_score: kb.config.retrieval_min_score,
            retrieval_top_k: kb.config.retrieval_top_k,
            indexed_embedding: kb.config.indexed_embedding,
            regeneration: kb.config.regeneration,
        },
        created_at: kb.created_at,
        updated_at: kb.updated_at,
        document_count,
    }
}

fn doc_to_response(doc: KnowledgeDocument) -> DocumentResponse {
    let (status_str, error_msg) = match &doc.status {
        DocumentStatus::Pending => ("pending".to_string(), None),
        DocumentStatus::Processing => ("processing".to_string(), None),
        DocumentStatus::Indexed => ("indexed".to_string(), None),
        DocumentStatus::Failed { error } => ("failed".to_string(), Some(error.clone())),
    };

    DocumentResponse {
        id: doc.id,
        kb_id: doc.kb_id,
        filename: doc.filename,
        file_path: doc.file_path,
        mime_type: doc.mime_type,
        chunk_count: doc.chunk_count,
        status: status_str,
        error_message: error_msg,
        created_at: doc.created_at,
        updated_at: doc.updated_at,
    }
}

/// A per-KB `embedding_provider` is descriptive only. Ingestion and every
/// query path embed with the single global `llm.embedding` backend, and the
/// space actually used is recorded in `KbConfig::indexed_embedding`. Anything
/// other than the default is accepted for backward compatibility but warned
/// about, so the field cannot silently pose as a per-KB selector.
fn validate_embedding_provider(provider: &str) {
    if provider != "fastembed" {
        tracing::warn!(
            requested = %provider,
            "per-KB embedding_provider is not a selector — ingestion and retrieval \
             both use the global llm.embedding backend; configure llm.embedding \
             to change the embedding model"
        );
    }
}

fn build_kb_config(req: Option<KbConfigRequest>) -> KbConfig {
    match req {
        Some(cfg) => KbConfig {
            embedding_provider: cfg
                .embedding_provider
                .inspect(|p| validate_embedding_provider(p))
                .unwrap_or_else(KbConfig::default_embedding_provider),
            embedding_model: cfg
                .embedding_model
                .unwrap_or_else(KbConfig::default_embedding_model),
            vector_dimensions: cfg.vector_dimensions,
            file_processor: cfg
                .file_processor
                .unwrap_or_else(KbConfig::default_file_processor),
            chunk_strategy: parse_chunk_strategy(cfg.chunk_strategy.as_deref(), cfg.chunk_size),
            retrieval_min_score: cfg.retrieval_min_score,
            retrieval_top_k: cfg.retrieval_top_k,
            indexed_embedding: None,
            regeneration: None,
        },
        None => KbConfig::default(),
    }
}

/// Reject retrieval settings that would silently break retrieval: a NaN or
/// out-of-range `min_score` filters out every chunk (or none), and a zero or
/// huge `top_k` returns nothing or floods the prompt.
fn validate_retrieval_settings(cfg: &KbConfigRequest) -> Result<(), (StatusCode, String)> {
    if let Some(min_score) = cfg.retrieval_min_score
        && !(0.0..=1.0).contains(&min_score)
    {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("retrieval_min_score must be between 0.0 and 1.0, got {min_score}"),
        ));
    }
    if let Some(top_k) = cfg.retrieval_top_k
        && !(1..=MAX_RETRIEVAL_TOP_K).contains(&top_k)
    {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("retrieval_top_k must be between 1 and {MAX_RETRIEVAL_TOP_K}, got {top_k}"),
        ));
    }
    Ok(())
}

fn merge_kb_config(mut existing: KbConfig, req: KbConfigRequest) -> KbConfig {
    if let Some(provider) = req.embedding_provider {
        validate_embedding_provider(&provider);
        existing.embedding_provider = provider;
    }
    if let Some(model) = req.embedding_model {
        existing.embedding_model = model;
    }
    if req.vector_dimensions.is_some() {
        existing.vector_dimensions = req.vector_dimensions;
    }
    if let Some(processor) = req.file_processor {
        existing.file_processor = processor;
    }
    if req.chunk_strategy.is_some() || req.chunk_size.is_some() {
        existing.chunk_strategy =
            parse_chunk_strategy(req.chunk_strategy.as_deref(), req.chunk_size);
    }
    if req.retrieval_min_score.is_some() {
        existing.retrieval_min_score = req.retrieval_min_score;
    }
    if req.retrieval_top_k.is_some() {
        existing.retrieval_top_k = req.retrieval_top_k;
    }
    existing
}

pub(crate) fn parse_chunk_strategy(strategy: Option<&str>, size: Option<usize>) -> ChunkingStrategy {
    let size = size.unwrap_or(1024);
    match strategy {
        Some("fixed") => ChunkingStrategy::FixedSize { size },
        Some("recursive") => ChunkingStrategy::Recursive { size },
        Some("structured") => ChunkingStrategy::Structured { size },
        Some("token") => ChunkingStrategy::Token { tokens: size },
        Some("sentence") => ChunkingStrategy::Sentence,
        Some("document") => ChunkingStrategy::Document,
        Some("semantic") => ChunkingStrategy::Semantic { threshold: 0.7 },
        _ => ChunkingStrategy::Structured { size },
    }
}
