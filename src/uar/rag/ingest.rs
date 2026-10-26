use crate::uar::domain::knowledge::{ANONYMOUS_KNOWLEDGE_OWNER, KnowledgeChunk};
use crate::uar::persistence::PersistenceLayer;
use crate::uar::rag::chunking::{Chunker, ChunkingStrategy};
use crate::uar::rag::embeddings::EmbeddingBackend;
use crate::uar::rag::embeddings::swappable::SwappableEmbeddingBackend;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use uuid::Uuid;
use walkdir::WalkDir;

pub struct IngestService {
    persistence: Arc<dyn PersistenceLayer>,
    embedding_backend: Arc<SwappableEmbeddingBackend>,
    strategy: ChunkingStrategy,
    // Track processed files to avoid re-ingesting identical content (naive check by path/mtime)
    // For MVP, we just ingest everything on startup or change.
    // Ideally store tracking info in DB.
}

impl std::fmt::Debug for IngestService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IngestService")
            .field("persistence", &"<dyn PersistenceLayer>")
            .field("embedding_backend", &self.embedding_backend)
            .field("strategy", &self.strategy)
            .finish()
    }
}

impl IngestService {
    pub fn new(
        persistence: Arc<dyn PersistenceLayer>,
        embedding_backend: Arc<SwappableEmbeddingBackend>,
        strategy: ChunkingStrategy,
    ) -> Self {
        Self {
            persistence,
            embedding_backend,
            strategy,
        }
    }

    /// Record the KB's embedding space on its first ingest, or refuse to mix
    /// a different space into chunks that were already written. Queries check
    /// the same record (see `KnowledgeBase::ensure_embedding_space`). The
    /// caller passes the job's pinned backend so the recorded space always
    /// matches the vectors the same job writes.
    async fn bind_embedding_space(
        &self,
        owner_id: &str,
        kb_id: &str,
        backend: &Arc<dyn EmbeddingBackend>,
    ) -> Result<()> {
        let Some(mut kb) = self.persistence.get_knowledge_base(owner_id, kb_id).await? else {
            return Ok(());
        };
        let active = backend.fingerprint();
        anyhow::ensure!(!kb.regeneration_unavailable(), "knowledge regeneration is incomplete");
        kb.ensure_embedding_space(&active)?;
        if kb.config.indexed_embedding.is_none() {
            kb.config.indexed_embedding = Some(active);
            self.persistence.save_knowledge_base(&kb).await?;
        }
        Ok(())
    }

    /// Chunk `text` with the strategy stored on the knowledge base, so the
    /// KB's `config.chunk_strategy` decides how its documents are split. When
    /// the KB cannot be found the service-wide strategy still applies, as it
    /// did before per-KB strategies were honoured. The chunker is built from
    /// the job's pinned backend, so semantic chunking embeds in the same
    /// space as the document embeddings that follow.
    async fn chunk_for_kb(
        &self,
        owner_id: &str,
        kb_id: &str,
        text: &str,
        backend: &Arc<dyn EmbeddingBackend>,
    ) -> Result<Vec<String>> {
        let strategy = match self.persistence.get_knowledge_base(owner_id, kb_id).await? {
            Some(kb) => kb.config.chunk_strategy,
            None => self.strategy.clone(),
        };
        Chunker::new(strategy, Some(Arc::clone(backend)))
            .chunk(text)
            .await
    }

    /// Process a single file
    pub async fn ingest_file(&self, path: &Path, kb_id: &str) -> Result<()> {
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        // Only support text/md for now
        if !matches!(extension, "txt" | "md" | "markdown") {
            return Ok(()); // Skip unsupported
        }

        let content = tokio::fs::read_to_string(path).await?;
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        tracing::info!("Ingesting processed file: {}", filename);

        // ONE pinned backend for the whole job: chunking, document embeds,
        // and the KB space binding all observe the same embedding space even
        // if a live `llm.embedding.*` swap lands mid-job.
        let backend = self.embedding_backend.pin();

        // 1. Chunking
        let chunks = self
            .chunk_for_kb(ANONYMOUS_KNOWLEDGE_OWNER, kb_id, &content, &backend)
            .await?;

        if chunks.is_empty() {
            return Ok(());
        }

        // 2. Embedding
        let refs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
        let embeddings = backend.embed(&refs).await?;
        self.bind_embedding_space(ANONYMOUS_KNOWLEDGE_OWNER, kb_id, &backend)
            .await?;

        // 3. Storage
        for (i, segment) in chunks.into_iter().enumerate() {
            let embedding = embeddings
                .get(i)
                .ok_or_else(|| anyhow!("Missing embedding for chunk {i}"))?;

            let mut metadata = HashMap::new();
            metadata.insert(
                "filename".to_string(),
                serde_json::Value::String(filename.to_string()),
            );
            metadata.insert(
                "path".to_string(),
                serde_json::Value::String(path.to_string_lossy().to_string()),
            );
            metadata.insert("index".to_string(), serde_json::json!(i));
            metadata.insert("embedding_space".to_string(), serde_json::to_value(backend.fingerprint())?);

            let chunk_id = Uuid::new_v4(); // Or deterministic based on content?

            let k_chunk = KnowledgeChunk {
                id: chunk_id,
                owner_id: ANONYMOUS_KNOWLEDGE_OWNER.to_string(),
                kb_id: kb_id.to_string(),
                document_id: None, // No document tracking in basic ingest
                content: segment,
                metadata: Some(serde_json::to_value(metadata)?),
                embedding: embedding.clone(),
                created_at: chrono::Utc::now().to_rfc3339(),
            };

            self.persistence.save_chunk(&k_chunk).await?;
        }

        Ok(())
    }

    /// Ingest text content directly (for worker pool use).
    /// Returns the number of chunks created.
    pub async fn ingest_text(
        &self,
        content: &str,
        owner_id: &str,
        kb_id: &str,
        document_id: String,
    ) -> Result<usize> {
        self.ingest_text_with_metadata(content, owner_id, kb_id, document_id, HashMap::new())
            .await
    }

    /// Ingest text content directly with additional metadata copied to each chunk.
    /// Returns the number of chunks created.
    pub async fn ingest_text_with_metadata(
        &self,
        content: &str,
        owner_id: &str,
        kb_id: &str,
        document_id: String,
        extra_metadata: HashMap<String, serde_json::Value>,
    ) -> Result<usize> {
        // ONE pinned backend for the whole job (see ingest_file).
        let backend = self.embedding_backend.pin();

        // 1. Chunking
        let chunks = self
            .chunk_for_kb(owner_id, kb_id, content, &backend)
            .await?;

        if chunks.is_empty() {
            return Ok(0);
        }

        // 2. Embedding
        let refs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
        let embeddings = backend.embed(&refs).await?;
        self.bind_embedding_space(owner_id, kb_id, &backend).await?;

        // 3. Storage
        for (i, segment) in chunks.iter().enumerate() {
            let embedding = embeddings
                .get(i)
                .ok_or_else(|| anyhow!("Missing embedding for chunk {i}"))?;

            let mut metadata = HashMap::new();
            metadata.insert(
                "document_id".to_string(),
                serde_json::Value::String(document_id.clone()),
            );
            metadata.insert("index".to_string(), serde_json::json!(i));
            metadata.extend(extra_metadata.clone());
            metadata.insert("embedding_space".to_string(), serde_json::to_value(backend.fingerprint())?);

            let chunk_id = Uuid::new_v4();

            let k_chunk = KnowledgeChunk {
                id: chunk_id,
                owner_id: owner_id.to_string(),
                kb_id: kb_id.to_string(),
                document_id: Some(document_id.clone()),
                content: segment.clone(),
                metadata: Some(serde_json::to_value(&metadata)?),
                embedding: embedding.clone(),
                created_at: chrono::Utc::now().to_rfc3339(),
            };

            self.persistence.save_chunk(&k_chunk).await?;
        }

        Ok(chunks.len())
    }

    /// Prepare a replay-safe generation without exposing partial chunks or changing a KB.
    pub async fn prepare_ingestion(
        &self,
        content: &str,
        job: &crate::uar::persistence::ingestion_records::IngestionJob,
        extra_metadata: HashMap<String, serde_json::Value>,
    ) -> Result<crate::uar::persistence::ingestion_records::IngestionPublication> {
        use sha2::{Digest, Sha256};
        use crate::uar::persistence::ingestion_records::{IngestionPublication, IngestionStoreError};
        job.validate_namespace()?;
        let kb = self.persistence.get_knowledge_base(&job.owner_id, &job.kb_id).await?
            .ok_or(IngestionStoreError::Revoked)?;
        // ONE pinned backend for fingerprint check, chunking, and embeds.
        let backend = self.embedding_backend.pin();
        // Publication binds the initial space in the same transaction as its
        // chunks. An authorized cohort may prepare the new target while the
        // old indexed identity stays unavailable until every member settles.
        let fingerprint = backend.fingerprint();
        crate::uar::rag::regeneration::validate_preparation(&kb, job, &fingerprint)?;
        let segments = Chunker::new(kb.config.chunk_strategy, Some(Arc::clone(&backend)))
            .chunk(content).await?;
        let refs: Vec<&str> = segments.iter().map(String::as_str).collect();
        let embeddings = if refs.is_empty() { Vec::new() } else { backend.embed(&refs).await? };
        anyhow::ensure!(segments.len() == embeddings.len(), "embedding result count differs from chunk count");
        let created_at = chrono::Utc::now().to_rfc3339();
        let chunks = segments.into_iter().zip(embeddings).enumerate().map(|(index, (content, embedding))| {
            // A logical position in an immutable attempt has one stable chunk identity.
            let mut hash = Sha256::new();
            hash.update(job.storage_key().as_bytes());
            hash.update(job.generation.as_bytes());
            hash.update((index as u64).to_be_bytes());
            let digest = hash.finalize();
            let mut bytes = [0u8; 16];
            bytes.copy_from_slice(&digest[..16]);
            let mut metadata = extra_metadata.clone();
            metadata.insert("document_id".to_string(), serde_json::json!(job.document_id));
            metadata.insert("index".to_string(), serde_json::json!(index));
            metadata.insert("ingestion_generation".to_string(), serde_json::json!(job.generation));
            metadata.insert("embedding_space".to_string(), serde_json::json!(fingerprint));
            if let Some(generation) = &job.regeneration {
                metadata.insert("knowledge_regeneration".to_string(), serde_json::json!(generation));
            }
            KnowledgeChunk {
                id: Uuid::from_bytes(bytes), owner_id: job.owner_id.clone(), kb_id: job.kb_id.clone(),
                document_id: Some(job.document_id.clone()), content,
                metadata: Some(serde_json::json!(metadata)), embedding, created_at: created_at.clone(),
            }
        }).collect();
        Ok(IngestionPublication { chunks, embedding: fingerprint })
    }

    /// Recursively scan and ingest a directory
    pub async fn ingest_directory(&self, dir: &Path, kb_id: &str) -> Result<()> {
        for entry in WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(Result::ok)
        {
            if entry.file_type().is_file()
                && let Err(e) = self.ingest_file(entry.path(), kb_id).await
            {
                tracing::error!("Failed to ingest {:?}: {:?}", entry.path(), e);
            }
        }
        Ok(())
    }

    /// Subscribe before the initial directory snapshot. Cancellation stops new
    /// ingestion; a file already accepted finishes before watcher release.
    pub async fn watch(&self, dir: PathBuf, kb_id: String,
        stop: tokio_util::sync::CancellationToken) -> Result<()> {
        self.watch_with_readiness(dir, kb_id, stop, None).await
    }

    /// Register the native watch before reporting readiness, then subscribe
    /// before the first authoritative directory snapshot. The readiness
    /// receiver must be created by the caller before this task starts so
    /// startup cannot mistake task admission for OS watcher registration.
    pub async fn watch_with_readiness(
        &self,
        dir: PathBuf,
        kb_id: String,
        stop: tokio_util::sync::CancellationToken,
        readiness: Option<tokio::sync::oneshot::Sender<std::result::Result<(), String>>>,
    ) -> Result<()> {
        use notify::Watcher as _;
        tracing::info!("Starting file watcher on {:?}", dir);
        let (events, mut changes) = tokio::sync::mpsc::unbounded_channel();
        let mut readiness = readiness;
        let mut watcher = match notify::recommended_watcher(move |event| {
            let _ = events.send(event);
        }) {
            Ok(watcher) => watcher,
            Err(error) => {
                if let Some(readiness) = readiness.take() {
                    let _ = readiness.send(Err(error.to_string()));
                }
                return Err(error.into());
            }
        };
        if let Err(error) = watcher.watch(&dir, notify::RecursiveMode::Recursive) {
            if let Some(readiness) = readiness.take() {
                let _ = readiness.send(Err(error.to_string()));
            }
            return Err(error.into());
        }
        if let Some(readiness) = readiness.take() {
            let _ = readiness.send(Ok(()));
        }

        // State to track last modified times
        let mut file_state: HashMap<PathBuf, std::time::SystemTime> = HashMap::new();

        loop {
            for entry in WalkDir::new(&dir)
                .follow_links(true)
                .into_iter()
                .filter_map(Result::ok)
            {
                if entry.file_type().is_file() {
                    if stop.is_cancelled() { break; }
                    let path = entry.path().to_path_buf();
                    // Check modified time
                    if let Ok(metadata) = std::fs::metadata(&path)
                        && let Ok(modified) = metadata.modified()
                    {
                        let should_process = match file_state.get(&path) {
                            Some(last_mod) => modified > *last_mod,
                            None => true,
                        };

                        if should_process {
                            if let Err(e) = self.ingest_file(&path, &kb_id).await {
                                tracing::error!("Watch ingest failed for {:?}: {:?}", path, e);
                            } else {
                                file_state.insert(path, modified);
                            }
                        }
                    }
                }
            }
            tokio::select! {
                biased;
                () = stop.cancelled() => break,
                event = changes.recv() => match event {
                    Some(Ok(_)) => {}
                    Some(Err(error)) => return Err(error.into()),
                    None => return Err(anyhow!("filesystem event source ended")),
                }
            }
        }
        watcher.unwatch(&dir)?;
        drop(watcher);
        Ok(())
    }
}
