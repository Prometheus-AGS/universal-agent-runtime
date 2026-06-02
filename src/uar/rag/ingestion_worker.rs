//! Document Ingestion Worker Pool
//!
//! Uses `prometheus_parking_lot` `WorkerPool` for scalable document ingestion.
//! This ensures CPU-bound document processing doesn't block the async HTTP server.

use crate::uar::{
    domain::knowledge::{DocumentStatus, KnowledgeDocument},
    file_processing::{FileProcessor, FileProcessorFactory, KreuzbergProvider, process_bytes},
    persistence::PersistenceLayer,
    rag::ingest::IngestService,
};
use crate::{AppConfig, config::FileProcessingConfig};
use anyhow::Result;
use async_trait::async_trait;
use prometheus_parking_lot::{
    config::WorkerPoolConfig,
    core::{PoolError, TaskMetadata, WorkerExecutor, WorkerPool},
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
        }
    }
}

#[async_trait]
impl WorkerExecutor<DocumentIngestionJob, IngestionResult> for DocumentIngestionExecutor {
    async fn execute(&self, job: DocumentIngestionJob, meta: TaskMetadata) -> IngestionResult {
        let doc_id = job.document.id.clone();

        // Honour deadline from task metadata as a secondary cancellation guard.
        let deadline_exceeded = meta.deadline_ms.map_or(false, |dl_ms| {
            u128::try_from(chrono::Utc::now().timestamp_millis()).unwrap_or(0) > dl_ms
        });
        if self.cancellation.is_cancelled() || deadline_exceeded {
            warn!(
                document_id = %doc_id,
                cancelled = self.cancellation.is_cancelled(),
                deadline_exceeded,
                "Ingestion task cancelled before start — leaving status unchanged"
            );
            return IngestionResult {
                document_id: doc_id,
                chunk_count: 0,
                status: DocumentStatus::Failed {
                    error: "cancelled before start".to_string(),
                },
            };
        }

        info!(document_id = %doc_id, "Starting document ingestion");

        // Update status to Processing
        if let Err(e) = self
            .persistence
            .update_document_status(&doc_id, &DocumentStatus::Processing)
            .await
        {
            warn!(document_id = %doc_id, error = %e, "Failed to update status to processing");
        }

        // Attempt to ingest the document; check cancellation token between stages
        match self.process_document(&job).await {
            Ok(chunk_count) => {
                // Update status to Indexed
                let status = DocumentStatus::Indexed;
                if let Err(e) = self
                    .persistence
                    .update_document_status(&doc_id, &status)
                    .await
                {
                    error!(document_id = %doc_id, error = %e, "Failed to update status to indexed");
                }

                info!(document_id = %doc_id, chunk_count, "Document ingestion completed");
                IngestionResult {
                    document_id: doc_id,
                    chunk_count,
                    status,
                }
            }
            Err(e) => {
                // Update status to Failed
                let status = DocumentStatus::Failed {
                    error: e.to_string(),
                };
                if let Err(update_err) = self
                    .persistence
                    .update_document_status(&doc_id, &status)
                    .await
                {
                    error!(document_id = %doc_id, error = %update_err, "Failed to update status to failed");
                }

                error!(document_id = %doc_id, error = %e, "Document ingestion failed");
                IngestionResult {
                    document_id: doc_id,
                    chunk_count: 0,
                    status,
                }
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
            return Err(anyhow::anyhow!("cancelled between extraction and embedding"));
        }

        let provider_name = result.provider_name.clone();
        let metadata = build_chunk_metadata(&job.document, &result, provider_name);

        // Stage 2 — chunk, embed, and store
        let chunks = self
            .ingest_service
            .ingest_text_with_metadata(
                &result.content,
                &job.kb_id,
                job.document.id.clone(),
                metadata,
            )
            .await?;

        Ok(chunks)
    }

    async fn resolve_file_processing_config(
        &self,
        job: &DocumentIngestionJob,
    ) -> Result<FileProcessingConfig> {
        let mut file_config = self.config.file_processing.clone();
        if let Some(kb) = self.persistence.get_knowledge_base(&job.kb_id).await?
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

        let temp_path = write_temp_document(&job.document.filename, &job.file_content).await?;
        let processor = FileProcessorFactory::create_for_file(
            &temp_path,
            file_config,
            self.config.unstructured.as_ref(),
            self.config.mistral_ocr.as_ref(),
            self.config.kreuzberg.as_ref(),
        )?;
        let provider_name = processor.provider_name().to_string();
        let result = processor.process(&temp_path).await;
        let cleanup_result = tokio::fs::remove_file(&temp_path).await;
        if let Err(err) = cleanup_result {
            warn!(
                path = %temp_path.display(),
                error = %err,
                "Failed to remove temporary ingestion file"
            );
        }

        let result = result?;
        Ok(ProcessedDocument {
            provider_name,
            content: result.content,
            mime_type: result.mime_type,
            metadata: result.metadata,
        })
    }
}

#[derive(Debug)]
struct ProcessedDocument {
    provider_name: String,
    content: String,
    mime_type: String,
    metadata: Option<serde_json::Value>,
}

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

async fn write_temp_document(filename: &str, data: &[u8]) -> Result<std::path::PathBuf> {
    let safe_name = Path::new(filename)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("document.bin");
    let path =
        std::env::temp_dir().join(format!("uar_ingest_{}_{}", uuid::Uuid::new_v4(), safe_name));
    tokio::fs::write(&path, data).await?;
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

/// High-level wrapper around the `prometheus_parking_lot` `WorkerPool`
/// for document ingestion.
pub struct IngestionWorkerPool {
    /// The underlying worker pool.
    pool: WorkerPool<DocumentIngestionJob, IngestionResult, DocumentIngestionExecutor>,
    /// Cancellation token — signalled on graceful shutdown so in-flight
    /// executors can abort between processing stages.
    cancellation: CancellationToken,
    /// Per-task deadline (ms) — `None` means unbounded.
    task_deadline_ms: Option<u128>,
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
    /// * `config` - Application config (used for file-processing provider selection
    ///   and, optionally, per-task deadline from `server.shutdown_timeout_secs`)
    pub fn new(
        worker_count: usize,
        max_queue_depth: usize,
        ingest_service: Arc<IngestService>,
        persistence: Arc<dyn PersistenceLayer>,
        config: Arc<AppConfig>,
    ) -> Result<Self, PoolError> {
        let worker_count = if worker_count == 0 {
            num_cpus::get()
        } else {
            worker_count
        };

        let cancellation = CancellationToken::new();

        let pool_config = WorkerPoolConfig::new()
            .with_worker_count(worker_count)
            .with_max_units(1000) // Resource capacity
            .with_max_queue_depth(max_queue_depth);

        let executor = DocumentIngestionExecutor::new(
            ingest_service,
            persistence,
            Arc::clone(&config),
            cancellation.clone(),
        );
        let pool = WorkerPool::new(pool_config, executor)?;

        info!(
            worker_count,
            max_queue_depth, "Ingestion worker pool initialized"
        );

        Ok(Self {
            pool,
            cancellation,
            task_deadline_ms: None,
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

    /// Submit a document for ingestion.
    ///
    /// Returns a job key that can be used to retrieve the result.
    pub async fn submit(
        &self,
        document: KnowledgeDocument,
        file_content: Vec<u8>,
    ) -> Result<String, PoolError> {
        let job = DocumentIngestionJob {
            kb_id: document.kb_id.clone(),
            document,
            file_content,
        };

        let now_ms = u128::try_from(chrono::Utc::now().timestamp_millis()).unwrap_or(0);
        let deadline_ms = self.task_deadline_ms.map(|d| now_ms + d);

        let meta = TaskMetadata {
            id: uuid::Uuid::new_v4().as_u128() as u64,
            priority: Priority::Normal,
            cost: ResourceCost {
                kind: ResourceKind::Cpu, // CPU-bound work
                units: 10,               // Each document uses 10 resource units
            },
            created_at_ms: now_ms,
            deadline_ms,
            mailbox: None,
            idempotency_key: None,
        };

        let key = self.pool.submit_async(job, meta).await?;
        Ok(format!("{key:?}"))
    }

    /// Retrieve the result of an ingestion job.
    ///
    /// Blocks until the job completes or the timeout expires.
    pub async fn retrieve(
        &self,
        key: &MailboxKey,
        timeout: std::time::Duration,
    ) -> Result<IngestionResult, PoolError> {
        self.pool.retrieve_async(key, timeout).await
    }

    /// Shut down the worker pool gracefully.
    ///
    /// Signals the cancellation token so in-flight executors can abort between
    /// processing stages, then calls the fixed `WorkerPool::shutdown(&self)`
    /// which honours a join timeout and detaches wedged workers.
    ///
    /// This method takes `&self` so it is callable through an `Arc`.
    pub fn shutdown(&self) {
        info!("Signalling ingestion worker pool shutdown");
        self.cancellation.cancel();
        self.pool.shutdown();
        info!("Ingestion worker pool shut down");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::KreuzbergConfig;

    fn test_document() -> KnowledgeDocument {
        KnowledgeDocument {
            id: "doc-1".to_string(),
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
    fn ingestion_worker_prefers_kreuzberg_bytes_for_default_documents() {
        let file_config = FileProcessingConfig::default();
        assert!(should_use_kreuzberg_bytes(
            &file_config,
            Some(&KreuzbergConfig::default()),
            "application/pdf"
        ));
    }

    #[test]
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
