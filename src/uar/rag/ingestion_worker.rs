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
    core::{SchedulerError, TaskError, TaskMetadata, WorkerExecutor, ScheduledTask},
    util::serde::{MailboxKey, Priority, ResourceCost, ResourceKind},
};
use std::{collections::HashMap, path::Path, sync::Arc};
use tokio_util::sync::CancellationToken;
use prometheus_parking_lot_core::{dispatch::{Dispatched, Grant}, semaphore::Permit};
use super::ingestion_admission::IngestionAdmission;
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
    async fn execute(&self, job: DocumentIngestionJob, meta: TaskMetadata) -> Result<IngestionResult, TaskError> {
        let doc_id = job.document.id.clone();
        let result = std::panic::AssertUnwindSafe(async {
            if self.cancellation.is_cancelled() || deadline_passed(meta.deadline_ms) {
                anyhow::bail!("ingestion interrupted before preparation");
            }
            self.process_document(&job, meta.deadline_ms).await
        }).catch_unwind().await;
        let result = match result {
            Ok(result) => result.map_err(TaskError::executor),
            Err(_) => Err(TaskError::ExecutorPanicked { message: "document ingestion panicked".to_owned() }),
        };
        match result {
            Ok(chunk_count) => Ok(IngestionResult { document_id: doc_id, chunk_count, status: DocumentStatus::Indexed }),
            Err(error) => {
                self.persist_failure(&job.durable, &error).await;
                Err(error)
            }
        }
    }

}

impl DocumentIngestionExecutor {
    async fn persist_failure(&self, job: &IngestionJob, error: &TaskError) {
        let failed = job.failed_attempt(error.to_string());
        let settlement = std::panic::AssertUnwindSafe(
            self.persistence.transition_ingestion(job, &failed)
        ).catch_unwind().await;
        match settlement {
            // False is a settled stale attempt: retry, publication or deletion
            // already revoked this exact CAS identity. Do not claim a new one.
            Ok(Ok(_)) => {}
            Ok(Err(update_error)) => {
                error!(document_id = %job.document_id, error = %update_error, "Could not persist ingestion failure; committed source remains discoverable");
                if let Some(scope) = &self.lifecycle_scope {
                    scope.record_failure(format!("ingestion failure settlement: {update_error}"));
                }
            }
            Err(_) => {
                error!(document_id = %job.document_id, "Ingestion failure settlement panicked; durable outcome is unknown");
                if let Some(scope) = &self.lifecycle_scope {
                    scope.record_failure("ingestion failure settlement panicked");
                    scope.record_unresolved_effect(format!("ingestion failure settlement outcome unknown: {}", job.storage_key()));
                }
            }
        }
    }

    /// Process a document and return chunk count.
    async fn process_document(&self, job: &DocumentIngestionJob, deadline: Option<u128>) -> Result<usize> {
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
        if self.cancellation.is_cancelled() || deadline_passed(deadline) {
            return Err(anyhow::anyhow!(
                "cancelled between extraction and embedding"
            ));
        }

        let provider_name = result.provider_name.clone();
        let metadata = build_chunk_metadata(&job.document, &result, provider_name);

        // Preparation is replayable; only the storage transaction publishes a generation.
        let publication = self.ingest_service.prepare_ingestion(&result.content, &job.durable, metadata).await?;
        if self.cancellation.is_cancelled() || deadline_passed(deadline) { anyhow::bail!("ingestion interrupted before publication"); }
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

/// Inspectable local admission state; the durable store remains authoritative.
#[derive(Clone, Debug, Default)]
pub struct IngestionAdmissionSnapshot {
    /// Grants retained through worker join and durable outcome settlement.
    pub active_jobs: usize,
    /// Descriptors parked locally without claiming or loading source bytes.
    pub waiting_jobs: usize,
    /// Active durable storage keys mapped to their result mailbox receipts.
    pub active_mailboxes: HashMap<String, MailboxKey>,
}

type Descriptor = ScheduledTask<IngestionJob>;
type WorkerResult = std::result::Result<IngestionResult, TaskError>;

struct Completion {
    key: String,
    mailbox: MailboxKey,
    permit: Permit,
}

/// Bounded document preparation owned by a trusted-host resource scope.
pub struct IngestionWorkerPool {
    executor: DocumentIngestionExecutor,
    scope: crate::uar::runtime::lifecycle::ResourceScope,
    admission: IngestionAdmission<Descriptor>,
    results: std::sync::Mutex<HashMap<MailboxKey, tokio::sync::watch::Sender<Option<WorkerResult>>>>,
    completed_tx: mpsc::UnboundedSender<Completion>,
    cancellation: CancellationToken,
    task_deadline_ms: Option<u128>,
    persistence: Arc<dyn PersistenceLayer>,
    wake: Arc<Notify>,
    completed: std::sync::Mutex<Option<mpsc::UnboundedReceiver<Completion>>>,
    dispatch_capacity: usize,
    max_queue_depth: usize,
    admission_state: tokio::sync::watch::Sender<IngestionAdmissionSnapshot>,
}

impl std::fmt::Debug for IngestionWorkerPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IngestionWorkerPool").finish()
    }
}

impl IngestionWorkerPool {
    /// Construct a pool with `worker_count` slots (zero selects CPU count) and
    /// at most `max_queue_depth` waiting descriptors, separate from durable work.
    pub fn new(
        worker_count: usize,
        max_queue_depth: usize,
        ingest_service: Arc<IngestService>,
        persistence: Arc<dyn PersistenceLayer>,
        config: Arc<AppConfig>,
        scope: crate::uar::runtime::lifecycle::ResourceScope,
    ) -> Result<Self, SchedulerError> {
        let worker_count = if worker_count == 0 { num_cpus::get() } else { worker_count };
        let admission = IngestionAdmission::new(worker_count, max_queue_depth)?;
        worker_count.checked_add(max_queue_depth)
            .ok_or_else(|| SchedulerError::invalid_config("ingestion descriptor capacity overflows usize"))?;
        let wake = Arc::new(Notify::new());
        admission.install_release_wake(Arc::clone(&wake));
        let cancellation = scope.cancellation_token();
        let (completed_tx, completed_rx) = mpsc::unbounded_channel();
        let mut executor = DocumentIngestionExecutor::new(
            ingest_service, Arc::clone(&persistence), config, cancellation.clone(),
        );
        executor.lifecycle_scope = Some(scope.clone());
        let (admission_state, _) = tokio::sync::watch::channel(IngestionAdmissionSnapshot::default());
        info!(worker_count, max_queue_depth, "Scope-owned ingestion admission initialized");
        Ok(Self {
            executor, scope, admission, results: std::sync::Mutex::new(HashMap::new()),
            completed_tx, cancellation, task_deadline_ms: None, persistence, wake,
            completed: std::sync::Mutex::new(Some(completed_rx)),
            dispatch_capacity: worker_count, max_queue_depth, admission_state,
        })
    }

    /// Set an explicit cooperative deadline checked between processing stages.
    /// The deadline never detaches work or releases a held grant early.
    pub fn with_task_deadline_ms(mut self, deadline_ms: Option<u128>) -> Self {
        self.task_deadline_ms = deadline_ms;
        self
    }

    /// Inspect admission and legitimate active-attempt mailbox receipts.
    pub fn admission_snapshot(&self) -> IngestionAdmissionSnapshot {
        self.admission_state.borrow().clone()
    }

    /// Observe local admission changes without polling the durable store.
    pub fn subscribe_admission(&self) -> tokio::sync::watch::Receiver<IngestionAdmissionSnapshot> {
        self.admission_state.subscribe()
    }

    /// Commit acceptance before requesting dispatch; local saturation is durable backlog.
    pub async fn submit(&self, document: KnowledgeDocument, file_content: Vec<u8>) -> Result<String> {
        let job = self.persistence.accept_ingestion(&document, &file_content).await?;
        self.notify_committed();
        Ok(job.document_id)
    }

    /// Queue notifications accelerate discovery; losing one cannot lose a job.
    pub fn notify_committed(&self) { self.wake.notify_one(); }

    fn publish_admission(&self, active: &HashMap<String, MailboxKey>) {
        self.admission_state.send_replace(IngestionAdmissionSnapshot {
            active_jobs: self.admission.active_jobs(),
            waiting_jobs: self.admission.waiting_jobs(),
            active_mailboxes: active.clone(),
        });
    }

    /// Called once by the trusted host after wrapping this pool in an Arc.
    pub fn start_durable_dispatch(self: &Arc<Self>) {
        let eligibility = self.persistence.durable_store_eligibility();
        if !eligibility.permits_durable_work() || !self.persistence.supports_durable_ingestion() {
            warn!(durable_store_eligibility = eligibility.as_str(), "Durable ingestion dispatcher disabled: storage does not support durable uploads");
            return;
        }
        let receiver = self.completed.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take();
        let Some(mut completed) = receiver else { return; };
        let shared = Arc::clone(self);
        self.notify_committed();
        let scope = self.scope.clone();
        if let Err(error) = scope.spawn("ingestion-dispatch", async move {
            let mut active = HashMap::new();
            let mut admitted = std::collections::HashSet::new();
            let mut reconciled = false;
            let mut closing = false;
            loop {
                if shared.cancellation.is_cancelled() && !closing {
                    closing = true;
                    for descriptor in shared.admission.close() {
                        admitted.remove(&descriptor.payload.storage_key());
                    }
                    shared.publish_admission(&active);
                }
                if closing && active.is_empty() { break; }
                let has_room = !closing && admitted.len() < shared.dispatch_capacity + shared.max_queue_depth;
                // The durable schedule reports future due dates, excluding active rows.
                let next_due = if has_room {
                    match shared.persistence.next_ingestion_due_at().await {
                        Ok(due) => due,
                        Err(error) => { warn!(%error, "Ingestion due-date source unavailable"); None }
                    }
                } else { None };
                let scheduled = async {
                    if let Some(due) = next_due {
                        let remaining = due.saturating_sub(chrono::Utc::now().timestamp());
                        tokio::time::sleep(std::time::Duration::from_secs(remaining.max(0) as u64)).await;
                    } else { std::future::pending::<()>().await; }
                };
                let mut dispatched = tokio::select! {
                    () = shared.cancellation.cancelled(), if !closing => continue,
                    completion = completed.recv() => {
                        let Some(completion) = completion else { break; };
                        admitted.remove(&completion.key);
                        active.remove(&completion.key);
                        // The worker sends only after its actual blocking join and durable settlement.
                        shared.results.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
                            .remove(&completion.mailbox);
                        shared.admission.complete(completion.permit, now_ms())
                    }
                    () = shared.wake.notified(), if !closing => shared.admission.poke(now_ms()),
                    () = scheduled, if !closing => shared.admission.poke(now_ms()),
                };
                shared.start_grants(&mut dispatched, &mut active, &mut admitted).await;
                if !closing && !shared.cancellation.is_cancelled() {
                    if !reconciled {
                        match shared.persistence.reconcile_ingestion().await {
                            Ok(()) => reconciled = true,
                            Err(error) => { error!(%error, "Could not reconcile legacy uploads"); continue; }
                        }
                    }
                    let capacity = shared.dispatch_capacity + shared.max_queue_depth;
                    if admitted.len() < capacity {
                        let jobs = match shared.persistence.discover_ingestion(capacity).await {
                            Ok(jobs) => jobs,
                            Err(error) => { error!(%error, "Durable ingestion discovery failed"); shared.publish_admission(&active); continue; }
                        };
                        for job in jobs {
                            if shared.cancellation.is_cancelled() || admitted.len() >= capacity { break; }
                            let key = job.storage_key();
                            if admitted.contains(&key) { continue; }
                            let meta = TaskMetadata {
                                id: uuid::Uuid::new_v4().as_u128() as u64, priority: Priority::Normal,
                                cost: ResourceCost::new(ResourceKind::Cpu, 1), created_at_ms: now_ms(),
                                deadline_ms: shared.task_deadline_ms.map(|duration| now_ms().saturating_add(duration)),
                                mailbox: None, idempotency_key: None, sequence: 0,
                            };
                            match shared.admission.submit(ScheduledTask { meta, payload: job.clone() }, now_ms()) {
                                Ok(admission) => {
                                    admitted.insert(key);
                                    let mut dispatched = admission.dispatched;
                                    shared.start_grants(&mut dispatched, &mut active, &mut admitted).await;
                                }
                                Err(SchedulerError::QueueFull { .. }) => break,
                                Err(error) => {
                                    shared.executor.persist_failure(&job, &TaskError::executor(error)).await;
                                }
                            }
                        }
                    }
                }
                shared.publish_admission(&active);
            }
            shared.publish_admission(&active);
            Ok(())
        }) { error!(%error, "Ingestion dispatcher admission closed"); }
    }

    async fn start_grants(
        self: &Arc<Self>, dispatched: &mut Dispatched<Descriptor>,
        active: &mut HashMap<String, MailboxKey>, admitted: &mut std::collections::HashSet<String>,
    ) {
        for descriptor in dispatched.expired.drain(..) {
            admitted.remove(&descriptor.payload.storage_key());
            self.executor.persist_failure(&descriptor.payload, &TaskError::DeadlineExpired).await;
        }
        for Grant { item, permit } in dispatched.started.drain(..) {
            let key = item.payload.storage_key();
            if self.cancellation.is_cancelled() {
                admitted.remove(&key);
                drop(permit);
                continue;
            }
            let mailbox = MailboxKey { tenant: "uar-ingestion".to_owned(),
                user_id: Some(uuid::Uuid::new_v4().to_string()), session_id: None };
            let (result_tx, _) = tokio::sync::watch::channel(None);
            self.results.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert(mailbox.clone(), result_tx.clone());
            let shared = Arc::clone(self);
            let task_key = key.clone();
            let task_mailbox = mailbox.clone();
            let spawned = self.scope.spawn("ingestion-prepare", async move {
                // Permit remains in this parent through actual join and failure CAS.
                let initial = item.payload.clone();
                let mut claimed_attempt = None;
                let result = match std::panic::AssertUnwindSafe(shared.run_descriptor(item, &mut claimed_attempt)).catch_unwind().await {
                    Ok(result) => result,
                    Err(_) => {
                        let error = TaskError::ExecutorPanicked { message: "ingestion admission or claim panicked".to_owned() };
                        // Retain the actual generated claim identity, including an
                        // ambiguous store panic after commit. Never fabricate a generation.
                        if let Some(claimed) = &claimed_attempt {
                            shared.executor.persist_failure(claimed, &error).await;
                        }
                        shared.executor.persist_failure(&initial, &error).await;
                        Err(error)
                    }
                };
                result_tx.send_replace(Some(result));
                let _ = shared.completed_tx.send(Completion { key: task_key, mailbox: task_mailbox, permit });
                Ok(())
            });
            match spawned {
                Ok(()) => { active.insert(key, mailbox); }
                Err(error) => {
                    admitted.remove(&key);
                    self.results.lock().unwrap_or_else(std::sync::PoisonError::into_inner).remove(&mailbox);
                    warn!(%error, "Ingestion spawn refused; unclaimed descriptor remains durable");
                }
            }
        }
    }

    async fn run_descriptor(&self, descriptor: Descriptor, claimed_attempt: &mut Option<IngestionJob>) -> WorkerResult {
        let initial = descriptor.payload;
        if self.cancellation.is_cancelled() {
            return Err(TaskError::Cancelled { reason: "ingestion shutdown before claim".to_owned() });
        }
        if deadline_passed(descriptor.meta.deadline_ms) {
            self.executor.persist_failure(&initial, &TaskError::DeadlineExpired).await;
            return Err(TaskError::DeadlineExpired);
        }
        if let Err(error) = initial.validate_namespace() {
            let error = TaskError::executor(error);
            self.executor.persist_failure(&initial, &error).await;
            return Err(error);
        }
        let job = if initial.state == IngestionState::Processing {
            *claimed_attempt = Some(initial.clone());
            initial
        } else {
            let processing = initial.processing();
            *claimed_attempt = Some(processing.clone());
            match self.persistence.transition_ingestion(&initial, &processing).await {
                Ok(true) => processing,
                Ok(false) => return Err(TaskError::executor(IngestionStoreError::Revoked)),
                Err(error) => {
                    let error = TaskError::executor(error);
                    self.executor.persist_failure(&processing, &error).await;
                    self.executor.persist_failure(&initial, &error).await;
                    return Err(error);
                }
            }
        };
        let execution = std::panic::AssertUnwindSafe(async {
            let file_content = self.persistence.get_ingestion_source(&job).await?
                .ok_or(IngestionStoreError::Revoked)?;
            let document = self.persistence.get_document(&job.owner_id, &job.document_id).await?
                .ok_or(IngestionStoreError::Revoked)?;
            let payload = DocumentIngestionJob { kb_id: job.kb_id.clone(), durable: job.clone(), document, file_content };
            let executor = self.executor.clone();
            let scope = self.scope.clone();
            let handle = tokio::runtime::Handle::current();
            let result = tokio::task::spawn_blocking(move || {
                handle.block_on(scope.run_in_scope(executor.execute(payload, descriptor.meta)))
            }).await;
            Ok::<WorkerResult, anyhow::Error>(match result {
                Ok(result) => result,
                Err(error) => Err(TaskError::ExecutorPanicked { message: error.to_string() }),
            })
        }).catch_unwind().await;
        match execution {
            Ok(Ok(result)) => {
                // Executor failures already settled against the exact processing attempt.
                if let Err(error @ TaskError::ExecutorPanicked { .. }) = &result {
                    self.executor.persist_failure(&job, error).await;
                }
                result
            }
            Ok(Err(error)) => {
                let error = TaskError::executor(error);
                self.executor.persist_failure(&job, &error).await;
                Err(error)
            }
            Err(_) => {
                let error = TaskError::ExecutorPanicked { message: "ingestion preparation panicked".to_owned() };
                self.executor.persist_failure(&job, &error).await;
                Err(error)
            }
        }
    }

    /// Await a joined result. Dropping this waiter never cancels real execution.
    pub async fn retrieve(&self, key: &MailboxKey, _legacy_timeout: std::time::Duration)
        -> Result<IngestionResult, SchedulerError>
    {
        let mut state = self.results.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(key).ok_or_else(|| SchedulerError::ResultNotFound { key: None })?.subscribe();
        loop {
            let result = state.borrow_and_update().clone();
            if let Some(result) = result { return result.map_err(SchedulerError::from); }
            state.changed().await.map_err(|_| SchedulerError::PoolShutdown { pool: "uar-ingestion".to_owned() })?;
        }
    }

    /// Cancel cooperatively and join actual dispatch/worker completion.
    pub async fn shutdown(&self) -> Result<()> {
        let report = self.scope.close().await;
        anyhow::ensure!(report.released, "ingestion closure failed: {:?}; unresolved: {:?}",
            report.failures, report.unresolved_effects);
        Ok(())
    }
}

fn now_ms() -> u128 {
    u128::try_from(chrono::Utc::now().timestamp_millis()).unwrap_or(0)
}

fn deadline_passed(deadline: Option<u128>) -> bool {
    deadline.is_some_and(|deadline| now_ms() > deadline)
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
