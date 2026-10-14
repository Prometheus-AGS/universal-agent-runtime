use crate::uar::domain::knowledge::{ANONYMOUS_KNOWLEDGE_OWNER, KnowledgeChunk};
use crate::uar::persistence::PersistenceLayer;
use crate::uar::rag::chunking::{Chunker, ChunkingStrategy};
use crate::uar::rag::embeddings::EmbeddingBackend;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use uuid::Uuid;
use walkdir::WalkDir;

pub struct IngestService {
    persistence: Arc<dyn PersistenceLayer>,
    embedding_backend: Arc<dyn EmbeddingBackend>,
    chunker: Chunker,
    // Track processed files to avoid re-ingesting identical content (naive check by path/mtime)
    // For MVP, we just ingest everything on startup or change.
    // Ideally store tracking info in DB.
}

impl std::fmt::Debug for IngestService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IngestService")
            .field("persistence", &"<dyn PersistenceLayer>")
            .field("embedding_backend", &self.embedding_backend)
            .field("chunker", &self.chunker)
            .finish()
    }
}

impl IngestService {
    pub fn new(
        persistence: Arc<dyn PersistenceLayer>,
        embedding_backend: Arc<dyn EmbeddingBackend>,
        strategy: ChunkingStrategy,
    ) -> Self {
        let chunker = Chunker::new(strategy, Some(Arc::clone(&embedding_backend)));
        Self {
            persistence,
            embedding_backend,
            chunker,
        }
    }

    /// Record the KB's embedding space on its first ingest, or refuse to mix
    /// a different space into chunks that were already written. Queries check
    /// the same record (see `KnowledgeBase::ensure_embedding_space`).
    async fn bind_embedding_space(&self, owner_id: &str, kb_id: &str) -> Result<()> {
        let Some(mut kb) = self.persistence.get_knowledge_base(owner_id, kb_id).await? else {
            return Ok(());
        };
        let active = self.embedding_backend.fingerprint();
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
    /// did before per-KB strategies were honoured.
    async fn chunk_for_kb(&self, owner_id: &str, kb_id: &str, text: &str) -> Result<Vec<String>> {
        match self.persistence.get_knowledge_base(owner_id, kb_id).await? {
            Some(kb) => {
                Chunker::new(
                    kb.config.chunk_strategy,
                    Some(Arc::clone(&self.embedding_backend)),
                )
                .chunk(text)
                .await
            }
            None => self.chunker.chunk(text).await,
        }
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

        // 1. Chunking
        let chunks = self
            .chunk_for_kb(ANONYMOUS_KNOWLEDGE_OWNER, kb_id, &content)
            .await?;

        if chunks.is_empty() {
            return Ok(());
        }

        // 2. Embedding
        let refs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
        let embeddings = self.embedding_backend.embed(&refs).await?;
        self.bind_embedding_space(ANONYMOUS_KNOWLEDGE_OWNER, kb_id)
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
        // 1. Chunking
        let chunks = self.chunk_for_kb(owner_id, kb_id, content).await?;

        if chunks.is_empty() {
            return Ok(0);
        }

        // 2. Embedding
        let refs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
        let embeddings = self.embedding_backend.embed(&refs).await?;
        self.bind_embedding_space(owner_id, kb_id).await?;

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
        let fingerprint = self.embedding_backend.fingerprint();
        kb.ensure_embedding_space(&fingerprint)?;
        let segments = Chunker::new(kb.config.chunk_strategy, Some(Arc::clone(&self.embedding_backend)))
            .chunk(content).await?;
        let refs: Vec<&str> = segments.iter().map(String::as_str).collect();
        let embeddings = if refs.is_empty() { Vec::new() } else { self.embedding_backend.embed(&refs).await? };
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

    /// Start a watcher loop (polling)
    pub async fn watch(&self, dir: PathBuf, kb_id: String) {
        tracing::info!("Starting file watcher on {:?}", dir);
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));

        // State to track last modified times
        let mut file_state: HashMap<PathBuf, std::time::SystemTime> = HashMap::new();

        loop {
            interval.tick().await;

            for entry in WalkDir::new(&dir)
                .follow_links(true)
                .into_iter()
                .filter_map(Result::ok)
            {
                if entry.file_type().is_file() {
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
        }
    }
}
