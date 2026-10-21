//! Document Ingestion Worker Pool
//!
//! Uses scope-owned blocking jobs with acknowledged completion for document ingestion.
//! This ensures CPU-bound document processing doesn't block the async HTTP server.

use crate::config::{AppConfig, FileProcessingConfig};
#[cfg(feature = "document-intelligence")]
use crate::uar::file_processing::{FileProcessor, KreuzbergProvider, process_bytes};
use crate::uar::{
    domain::knowledge::{DocumentStatus, KnowledgeDocument},
    file_processing::FileProcessorFactory,
    persistence::PersistenceLayer,
    rag::ingest::IngestService,
};
use anyhow::Result;
use futures::FutureExt as _;
use crate::uar::persistence::ingestion_records::{IngestionJob, IngestionState, IngestionStoreError};
use tokio::sync::{Notify, mpsc};
use async_trait::async_trait;
use prometheus_parking_lot::{
    core::{PoolError, TaskMetadata, WorkerExecutor},
    util::serde::{MailboxKey, Priority, ResourceCost, ResourceKind},
};
use std::{collections::HashMap, path::Path, sync::Arc};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

// =============================================================================
// Job and Result Types
// =============================================================================

/// A document ingestion job to be processed by the worker pool.
#[derive(Debug, Clone)]
pub struct DocumentIngestionJob {
    /// Durable attempt whose exact state fences every write.
    pub durable: IngestionJob,
    /// The document to process
    pub document: KnowledgeDocument,
    /// Raw file content (bytes)
    pub file_content: Vec<u8>,
    /// Knowledge base ID for the document
    pub kb_id: String,
}

/// Result from processing a document.
#[derive(Debug, Clone)]
pub struct IngestionResult {
    /// Document ID that was processed
    pub document_id: String,
    /// Number of chunks created
    pub chunk_count: usize,
    /// Final status
    pub status: DocumentStatus,
}

// =============================================================================
// Worker Executor Implementation
// =============================================================================

/// Executor for document ingestion jobs.
#[derive(Clone, Debug)]
pub struct DocumentIngestionExecutor {
    /// Shared ingest service for chunking and embedding
    ingest_service: Arc<IngestService>,
    /// Persistence layer for status updates
    persistence: Arc<dyn PersistenceLayer>,
    /// Application configuration for file-processing provider selection
    config: Arc<AppConfig>,
    /// Cancellation token — checked between processing stages to honour
    /// graceful shutdown without leaving documents in `Processing` state.
    cancellation: CancellationToken,
    lifecycle_scope: Option<crate::uar::runtime::lifecycle::ResourceScope>,
}

impl DocumentIngestionExecutor {
    /// Create a new document ingestion executor.
    pub fn new(
        ingest_service: Arc<IngestService>,
        persistence: Arc<dyn PersistenceLayer>,
        config: Arc<AppConfig>,
        cancellation: CancellationToken,
    ) -> Self {
        Self {
            ingest_service,
            persistence,
            config,
            cancellation,
            lifecycle_scope: None,
        }
    }
}

#[async_trait]
impl WorkerExecutor<DocumentIngestionJob, IngestionResult> for DocumentIngestionExecutor {
    async fn execute(&self, job: DocumentIngestionJob, meta: TaskMetadata) -> IngestionResult {
        let doc_id = job.document.id.clone();
        let deadline_exceeded = meta.deadline_ms.is_some_and(|deadline| {
            u128::try_from(chrono::Utc::now().timestamp_millis()).unwrap_or(0) > deadline
        });
        let result = if self.cancellation.is_cancelled() || deadline_exceeded {
            Err(anyhow::anyhow!("ingestion interrupted before preparation"))
        } else {
            self.process_document(&job).await
        };
        match result {
            Ok(chunk_count) => IngestionResult { document_id: doc_id, chunk_count, status: DocumentStatus::Indexed },
            Err(error) => {
                // CAS cannot overwrite a concurrent publication, retry, or deletion.
                let failed = job.durable.failed_attempt(error.to_string());
                if let Err(update_error) = self.persistence.transition_ingestion(&job.durable, &failed).await {
                    error!(document_id = %doc_id, error = %update_error, "Could not persist ingestion failure; committed source remains discoverable");
                }
                warn!(document_id = %doc_id, error = %error, "Ingestion attempt did not publish");
                IngestionResult { document_id: doc_id, chunk_count: 0, status: DocumentStatus::Failed { error: error.to_string() } }
            }
        }
    }
}

impl DocumentIngestionExecutor {
    /// Process a document and return chunk count.
    async fn process_document(&self, job: &DocumentIngestionJob) -> Result<usize> {
        let file_config = self.resolve_file_processing_config(job).await?;
        let mime_type = job
            .document
            .mime_type
            .clone()
            .unwrap_or_else(|| guess_mime_type(&job.document.filename));

        // Stage 1 — extraction (CPU/IO bound; may be long)
        let result = self.extract_document(job, &file_config, &mime_type).await?;

        // Check cancellation between stages so a shutdown during a slow
        // extraction is caught before we spend time on embedding.
        if self.cancellation.is_cancelled() {
            return Err(anyhow::anyhow!(
                "cancelled between extraction and embedding"
            ));
        }

        let provider_name = result.provider_name.clone();
        let metadata = build_chunk_metadata(&job.document, &result, provider_name);

        // Preparation is replayable; only the storage transaction publishes a generation.
        let publication = self.ingest_service.prepare_ingestion(&result.content, &job.durable, metadata).await?;
        if self.cancellation.is_cancelled() { anyhow::bail!("ingestion interrupted before publication"); }
        let count = publication.chunks.len();
        self.persistence.publish_ingestion(&job.durable, &publication).await?;
        Ok(count)
    }

    async fn resolve_file_processing_config(
        &self,
        job: &DocumentIngestionJob,
    ) -> Result<FileProcessingConfig> {
        let mut file_config = self.config.file_processing.clone();
        if let Some(kb) = self
            .persistence
            .get_knowledge_base(&job.document.owner_id, &job.kb_id)
            .await?
            && !kb.config.file_processor.trim().is_empty()
        {
            file_config.provider = kb.config.file_processor;
        }
        Ok(file_config)
    }

    async fn extract_document(
        &self,
        job: &DocumentIngestionJob,
        file_config: &FileProcessingConfig,
        mime_type: &str,
    ) -> Result<ProcessedDocument> {
        #[cfg(not(feature = "document-intelligence"))]
        let _ = mime_type;
        #[cfg(feature = "document-intelligence")]
        if should_use_kreuzberg_bytes(file_config, self.config.kreuzberg.as_ref(), mime_type) {
            let kreuzberg_config = self.config.kreuzberg.clone().unwrap_or_default();
            let result = process_bytes(&job.file_content, mime_type, &kreuzberg_config).await?;
            return Ok(ProcessedDocument {
                provider_name: "Kreuzberg".to_string(),
                content: result.content,
                mime_type: result.mime_type,
                metadata: result.metadata,
            });
        }

        let temp_path = write_temp_document(&job.document.filename, &job.file_content, self.lifecycle_scope.as_ref()).await?;
        let result: Result<ProcessedDocument> = match std::panic::AssertUnwindSafe(async {
            let processor = FileProcessorFactory::create_for_file(
                &temp_path, file_config, self.config.unstructured.as_ref(),
                self.config.mistral_ocr.as_ref(), self.config.kreuzberg.as_ref(),
            )?;
            let provider_name = processor.provider_name().to_string();
            let result = processor.process(&temp_path).await?;
            Ok(ProcessedDocument {
                provider_name, content: result.content, mime_type: result.mime_type,
                metadata: result.metadata,
            })
        }).catch_unwind().await {
            Ok(result) => result,
            Err(_) => Err(anyhow::anyhow!("document extraction panicked")),
        };
        let cleanup_result = tokio::fs::remove_file(&temp_path).await;
        if let Err(error) = cleanup_result {
            if let Some(scope) = &self.lifecycle_scope {
                scope.record_failure(format!("temporary ingestion file release failed: {}", temp_path.display()));
            }
            return match result {
                Ok(_) => Err(error.into()),
                Err(original) => Err(original.context(format!("temporary ingestion file cleanup also failed: {error}"))),
            };
        }
        result
    }
}



#[derive(Debug)]
struct ProcessedDocument {
    provider_name: String,
    content: String,
    mime_type: String,
    metadata: Option<serde_json::Value>,
}

#[cfg(feature = "document-intelligence")]
fn should_use_kreuzberg_bytes(
    file_config: &FileProcessingConfig,
    kreuzberg_config: Option<&crate::config::KreuzbergConfig>,
    mime_type: &str,
) -> bool {
    let provider = file_config.provider.as_str();
    if provider != "kreuzberg" && provider != "auto" {
        return false;
    }

    let provider = KreuzbergProvider::new(kreuzberg_config.cloned().unwrap_or_default());
    provider.supports_mime_type(mime_type)
}

async fn write_temp_document(filename: &str, data: &[u8], scope: Option<&crate::uar::runtime::lifecycle::ResourceScope>) -> Result<std::path::PathBuf> {
    let safe_name = Path::new(filename)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("document.bin");
    let path =
        std::env::temp_dir().join(format!("uar_ingest_{}_{}", uuid::Uuid::new_v4(), safe_name));
    if let Err(error) = tokio::fs::write(&path, data).await {
        match tokio::fs::remove_file(&path).await {
            Ok(()) => {}
            Err(cleanup) if cleanup.kind() == std::io::ErrorKind::NotFound => {}
            Err(cleanup) => {
                if let Some(scope) = scope { scope.record_failure(format!("partial ingestion file release failed: {}", path.display())); }
                return Err(anyhow::Error::new(error).context(format!("partial ingestion file cleanup failed: {cleanup}")));
            }
        }
        return Err(error.into());
    }
    Ok(path)
}

fn guess_mime_type(filename: &str) -> String {
    mime_guess::from_path(filename)
        .first_or_octet_stream()
        .to_string()
}

fn copy_metadata_field(
    metadata: &mut HashMap<String, serde_json::Value>,
    extraction_metadata: &serde_json::Value,
    field: &str,
) {
    if let Some(value) = extraction_metadata.get(field)
        && !value.is_null()
    {
        metadata.insert(field.to_string(), value.clone());
    }
}

fn build_chunk_metadata(
    document: &KnowledgeDocument,
    result: &ProcessedDocument,
    provider_name: String,
) -> HashMap<String, serde_json::Value> {
    let mut metadata = HashMap::new();
    metadata.insert(
        "filename".to_string(),
        serde_json::Value::String(document.filename.clone()),
    );
    metadata.insert(
        "mime_type".to_string(),
        serde_json::Value::String(result.mime_type.clone()),
    );
    metadata.insert(
        "processor".to_string(),
        serde_json::Value::String(provider_name),
    );
    metadata.insert(
        "source".to_string(),
        serde_json::Value::String("knowledge_upload".to_string()),
    );

    if let Some(extraction_metadata) = &result.metadata {
        copy_metadata_field(&mut metadata, extraction_metadata, "title");
        copy_metadata_field(&mut metadata, extraction_metadata, "language");
        copy_metadata_field(&mut metadata, extraction_metadata, "pages");
        metadata.insert("extraction".to_string(), extraction_metadata.clone());
    }

    metadata
}

// =============================================================================
// Worker Pool Wrapper
// =============================================================================

/// Bounded document preparation owned by a trusted-host resource scope.
pub struct IngestionWorkerPool {
    executor: DocumentIngestionExecutor,
    scope: crate::uar::runtime::lifecycle::ResourceScope,
    permits: Arc<tokio::sync::Semaphore>,
    results: std::sync::Mutex<HashMap<MailboxKey, tokio::sync::watch::Sender<Option<std::result::Result<IngestionResult, String>>>>>,
    completed_tx: mpsc::UnboundedSender<String>,
    /// Cancellation token — signalled on graceful shutdown so in-flight
    /// executors can abort between processing stages.
    cancellation: CancellationToken,
    /// Per-task deadline (ms) — `None` means unbounded.
    task_deadline_ms: Option<u128>,
    persistence: Arc<dyn PersistenceLayer>,
    wake: Notify,
    completed: std::sync::Mutex<Option<mpsc::UnboundedReceiver<String>>>,
    dispatch_capacity: usize,
}

impl std::fmt::Debug for IngestionWorkerPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IngestionWorkerPool").finish()
    }
}

impl IngestionWorkerPool {
    /// Create a new ingestion worker pool.
    ///
    /// # Arguments
    /// * `worker_count` - Number of worker threads (0 = auto-detect based on CPU)
    /// * `max_queue_depth` - Maximum pending jobs before backpressure
    /// * `ingest_service` - Shared ingest service
    /// * `persistence` - Persistence layer for status updates
    /// * `config` - Application config used for file-processing provider selection
    /// * `scope` - Host ownership of accepted dispatch and blocking work
    pub fn new(
        worker_count: usize,
        _max_queue_depth: usize,
        ingest_service: Arc<IngestService>,
        persistence: Arc<dyn PersistenceLayer>,
        config: Arc<AppConfig>,
        scope: crate::uar::runtime::lifecycle::ResourceScope,
    ) -> Result<Self, PoolError> {
        let worker_count = if worker_count == 0 {
            num_cpus::get()
        } else {
            worker_count
        };

        let cancellation = scope.cancellation_token();
        let (completed_tx, completed_rx) = mpsc::unbounded_channel();
        let mut executor = DocumentIngestionExecutor::new(
            ingest_service, Arc::clone(&persistence), config, cancellation.clone(),
        );
        executor.lifecycle_scope = Some(scope.clone());
        info!(worker_count, "Scope-owned ingestion workers initialized");
        Ok(Self {
            executor,
            scope,
            permits: Arc::new(tokio::sync::Semaphore::new(worker_count)),
            results: std::sync::Mutex::new(HashMap::new()),
            completed_tx,
            cancellation,
            task_deadline_ms: None,
            persistence,
            wake: Notify::new(),
            completed: std::sync::Mutex::new(Some(completed_rx)),
            dispatch_capacity: worker_count,
        })
    }

    /// Set the per-task deadline in milliseconds from now.
    ///
    /// When set, each submitted task receives an absolute deadline. Tasks still
    /// running when the deadline passes are abandoned after the cancellation
    /// check between stages.
    pub fn with_task_deadline_ms(mut self, deadline_ms: Option<u128>) -> Self {
        self.task_deadline_ms = deadline_ms;
        self
    }

    /// Compatibility entry point: commit acceptance before requesting dispatch.
    pub async fn submit(&self, document: KnowledgeDocument, file_content: Vec<u8>) -> Result<String> {
        let job = self.persistence.accept_ingestion(&document, &file_content).await?;
        self.notify_committed();
        Ok(job.document_id)
    }

    /// Queue notifications accelerate discovery; losing one cannot lose a job.
    pub fn notify_committed(&self) { self.wake.notify_one(); }

    /// Called once by the trusted server after wrapping this pool in an Arc.
    pub fn start_durable_dispatch(self: &Arc<Self>) {
        let eligibility = self.persistence.durable_store_eligibility();
        if !eligibility.permits_durable_work() || !self.persistence.supports_durable_ingestion() {
            warn!(durable_store_eligibility = eligibility.as_str(), "Durable ingestion dispatcher disabled: storage does not support durable uploads");
            return;
        }
        let receiver = match self.completed.lock() {
            Ok(mut receiver) => receiver.take(),
            Err(error) => { error!(%error, "Could not start durable ingestion dispatcher"); return; }
        };
        let Some(mut completed) = receiver else { return; };
        let shared = Arc::clone(self);
        self.notify_committed();
        let scope = self.scope.clone();
        if let Err(error) = scope.spawn("ingestion-dispatch", async move {
            let mut active: HashMap<String, MailboxKey> = HashMap::new();
            let mut reconciled = false;
            loop {
                let next_due = if active.len() < shared.dispatch_capacity {
                    match shared.persistence.next_ingestion_due_at().await {
                        Ok(due) => due,
                        Err(error) => { warn!(%error, "Ingestion due-date source unavailable; awaiting an authoritative change"); None }
                    }
                } else { None };
                let scheduled = async {
                    if let Some(due) = next_due {
                        let remaining = due.saturating_sub(chrono::Utc::now().timestamp());
                        tokio::time::sleep(std::time::Duration::from_secs(remaining.max(0) as u64)).await;
                    } else {
                        std::future::pending::<()>().await;
                    }
                };
                tokio::select! {
                    _ = shared.cancellation.cancelled() => break,
                    key = completed.recv() => {
                        if let Some(key) = key {
                            if let Some(mailbox) = active.remove(&key) {
                                // Completion is published only after the blocking job is joined
                                // and its result has been installed in retained state.
                                if let Err(error) = shared.retrieve(&mailbox, std::time::Duration::ZERO).await {
                                    warn!(%error, "Ingestion mailbox result unavailable; database remains authoritative");
                                }
                            }
                        }
                    }
                    _ = shared.wake.notified() => {}
                    () = scheduled => {}
                }
                if shared.cancellation.is_cancelled() { break; }
                if !reconciled {
                    match shared.persistence.reconcile_ingestion().await {
                        Ok(()) => reconciled = true,
                        Err(error) => { error!(%error, "Could not reconcile legacy uploads"); continue; }
                    }
                }
                if active.len() >= shared.dispatch_capacity { continue; }
                let jobs = match shared.persistence.discover_ingestion(shared.dispatch_capacity + active.len()).await {
                    Ok(jobs) => jobs,
                    Err(error) => { error!(%error, "Durable ingestion discovery failed"); continue; }
                };
                for job in jobs {
                    if shared.cancellation.is_cancelled() || active.len() >= shared.dispatch_capacity { break; }
                    let key = job.storage_key();
                    if active.contains_key(&key) { continue; }
                    match shared.dispatch(job).await {
                        Ok(Some(mailbox)) => { active.insert(key, mailbox); }
                        Ok(None) => {}
                        Err(error) => { warn!(%error, "Ingestion dispatch failed; job remains durable"); }
                    }
                }
            }
            Ok(())
        }) {
            error!(%error, "Ingestion dispatcher admission closed");
        }
    }

    async fn dispatch(&self, job: IngestionJob) -> Result<Option<MailboxKey>> {
        job.validate_namespace()?;
        let job = if job.state == IngestionState::Processing {
            // Crash recovery can repeat pure preparation. Publication alone owns the fence.
            job
        } else {
            let processing = job.processing();
            if !self.persistence.transition_ingestion(&job, &processing).await? { return Ok(None); }
            processing
        };
        let Some(file_content) = self.persistence.get_ingestion_source(&job).await? else { return Ok(None); };
        let document = self.persistence.get_document(&job.owner_id, &job.document_id).await?
            .ok_or(IngestionStoreError::Revoked)?;
        let payload = DocumentIngestionJob { kb_id: job.kb_id.clone(), durable: job, document, file_content };
        let now_ms = u128::try_from(chrono::Utc::now().timestamp_millis()).unwrap_or(0);
        let meta = TaskMetadata {
            id: uuid::Uuid::new_v4().as_u128() as u64,
            priority: Priority::Normal,
            cost: ResourceCost { kind: ResourceKind::Cpu, units: 10 },
            created_at_ms: now_ms,
            deadline_ms: self.task_deadline_ms.map(|duration| now_ms + duration),
            mailbox: None, idempotency_key: None,
        };
        let mailbox = MailboxKey {
            tenant: "uar-ingestion".to_owned(), user_id: Some(uuid::Uuid::new_v4().to_string()),
            session_id: None,
        };
        let (result_tx, _) = tokio::sync::watch::channel(None);
        self.results.lock().unwrap_or_else(|error| error.into_inner())
            .insert(mailbox.clone(), result_tx.clone());
        let executor = self.executor.clone();
        let permits = Arc::clone(&self.permits);
        let completed = self.completed_tx.clone();
        let key = payload.durable.storage_key();
        let cancellation = self.cancellation.clone();
        let task_scope = self.scope.clone();
        let handle = tokio::runtime::Handle::current();
        if let Err(error) = self.scope.spawn("ingestion-prepare", async move {
            let permit = tokio::select! {
                () = cancellation.cancelled() => {
                    result_tx.send_replace(Some(Err("ingestion canceled before execution".to_owned())));
                    let _ = completed.send(key);
                    return Ok(());
                }
                permit = permits.acquire_owned() => permit?,
            };
            let result = tokio::task::spawn_blocking(move || {
                let _permit = permit;
                // Tokio task locals do not cross the blocking runtime entry.
                // Keep the accepted worker under the pool's scope until the
                // actual blocking operation returns and this parent task joins
                // it.
                handle.block_on(task_scope.run_in_scope(executor.execute(payload, meta)))
            }).await.map_err(|_| "ingestion worker panicked".to_owned());
            result_tx.send_replace(Some(result.clone()));
            let _ = completed.send(key);
            result.map(|_| ()).map_err(anyhow::Error::msg)
        }) {
            self.results.lock().unwrap_or_else(|error| error.into_inner()).remove(&mailbox);
            return Err(anyhow::Error::new(error));
        }
        Ok(Some(mailbox))
    }

    /// Await a joined job result. The legacy timeout is not completion evidence.
    pub async fn retrieve(&self, key: &MailboxKey, _legacy_timeout: std::time::Duration)
        -> Result<IngestionResult, PoolError>
    {
        let mut state = self.results.lock().unwrap_or_else(|error| error.into_inner())
            .get(key).ok_or(PoolError::ResultNotFound)?.subscribe();
        loop {
            let result = state.borrow_and_update().clone();
            if let Some(result) = result {
                self.results.lock().unwrap_or_else(|error| error.into_inner()).remove(key);
                return result.map_err(PoolError::Internal);
            }
            state.changed().await.map_err(|_| PoolError::PoolShutdown)?;
        }
    }

    /// Request cancellation and await all actual job and dispatcher completion.
    pub async fn shutdown(&self) -> Result<()> {
        let report = self.scope.close().await;
        anyhow::ensure!(report.released, "ingestion closure failed: {:?}; unresolved: {:?}",
            report.failures, report.unresolved_effects);
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "document-intelligence")]
    use crate::config::KreuzbergConfig;

    fn test_document() -> KnowledgeDocument {
        KnowledgeDocument {
            id: "doc-1".to_string(),
            owner_id: "anonymous".to_string(),
            kb_id: "kb-1".to_string(),
            filename: "report.pdf".to_string(),
            file_path: None,
            mime_type: Some("application/pdf".to_string()),
            chunk_count: 0,
            status: DocumentStatus::Pending,
            created_at: "2026-04-26T00:00:00Z".to_string(),
            updated_at: "2026-04-26T00:00:00Z".to_string(),
        }
    }

    #[test]
    #[cfg(feature = "document-intelligence")]
    fn ingestion_worker_prefers_kreuzberg_bytes_for_default_documents() {
        let file_config = FileProcessingConfig::default();
        assert!(should_use_kreuzberg_bytes(
            &file_config,
            Some(&KreuzbergConfig::default()),
            "application/pdf"
        ));
    }

    #[test]
    #[cfg(feature = "document-intelligence")]
    fn ingestion_worker_does_not_route_images_to_kreuzberg_when_ocr_is_disabled() {
        let file_config = FileProcessingConfig::default();
        assert!(!should_use_kreuzberg_bytes(
            &file_config,
            Some(&KreuzbergConfig::default()),
            "image/png"
        ));
    }

    #[test]
    fn ingestion_worker_chunk_metadata_includes_extraction_citation_fields() {
        let result = ProcessedDocument {
            provider_name: "Kreuzberg".to_string(),
            content: "extracted text".to_string(),
            mime_type: "application/pdf".to_string(),
            metadata: Some(serde_json::json!({
                "title": "Quarterly Report",
                "language": "en",
                "pages": 12
            })),
        };

        let metadata =
            build_chunk_metadata(&test_document(), &result, result.provider_name.clone());
        assert_eq!(metadata["filename"], "report.pdf");
        assert_eq!(metadata["mime_type"], "application/pdf");
        assert_eq!(metadata["processor"], "Kreuzberg");
        assert_eq!(metadata["title"], "Quarterly Report");
        assert_eq!(metadata["language"], "en");
        assert_eq!(metadata["pages"], 12);
        assert!(metadata.get("extraction").is_some());
    }
}
