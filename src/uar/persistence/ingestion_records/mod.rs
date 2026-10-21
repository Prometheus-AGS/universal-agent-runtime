//! Durable upload admission and fenced publication contracts owned by the host.

use crate::uar::domain::knowledge::{EmbeddingFingerprint, KnowledgeChunk, KnowledgeDocument};
use serde::{Deserialize, Serialize};
use super::knowledge_namespace::KnowledgeNamespace;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IngestionState {
    Pending,
    Processing,
    Retryable,
    Failed,
    Indexed,
}

/// Public job metadata. Immutable source bytes are deliberately a separate read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngestionJob {
    #[serde(default, skip_serializing_if = "KnowledgeNamespace::is_legacy")]
    pub namespace: KnowledgeNamespace,
    pub owner_id: String,
    pub document_id: String,
    pub kb_id: String,
    /// Replaced for each retry; stale workers cannot publish into a new attempt.
    pub generation: String,
    pub state: IngestionState,
    pub attempts: u32,
    pub next_attempt_at: i64,
    pub last_error: Option<String>,
}

impl IngestionJob {
    pub fn new(document: &KnowledgeDocument) -> anyhow::Result<Self> {
        Ok(Self {
            namespace: KnowledgeNamespace::for_owner(&document.owner_id)?,
            owner_id: document.owner_id.clone(),
            document_id: document.id.clone(),
            kb_id: document.kb_id.clone(),
            generation: uuid::Uuid::new_v4().to_string(),
            state: IngestionState::Pending,
            attempts: 0,
            next_attempt_at: chrono::Utc::now().timestamp(),
            last_error: None,
        })
    }

    /// Legacy canonical-looking subjects cannot enter the authenticated dispatcher.
    pub fn validate_namespace(&self) -> anyhow::Result<()> {
        anyhow::ensure!(self.namespace == KnowledgeNamespace::for_owner(&self.owner_id)?, "ingestion namespace does not match trusted owner boundary");
        Ok(())
    }

    pub fn storage_key(&self) -> String {
        super::tenant_storage_key(&self.owner_id, &self.document_id)
    }

    pub fn processing(&self) -> Self {
        let mut next = self.clone();
        next.state = IngestionState::Processing;
        next.attempts += 1;
        next.generation = uuid::Uuid::new_v4().to_string();
        next.last_error = None;
        next
    }

    pub fn failed_attempt(&self, error: String) -> Self {
        let mut next = self.clone();
        next.state = if self.attempts < 5 { IngestionState::Retryable } else { IngestionState::Failed };
        next.next_attempt_at = chrono::Utc::now().timestamp() + 30 * i64::from(self.attempts.max(1));
        next.last_error = Some(error);
        next
    }

    pub fn retry(&self) -> Self {
        let mut next = self.clone();
        next.state = IngestionState::Pending;
        next.attempts = 0;
        next.generation = uuid::Uuid::new_v4().to_string();
        next.next_attempt_at = chrono::Utc::now().timestamp();
        next.last_error = None;
        next
    }

    pub fn validate_transition(&self, after: &Self) -> anyhow::Result<()> {
        self.validate_namespace()?;
        after.validate_namespace()?;
        anyhow::ensure!(self.namespace == after.namespace && self.owner_id == after.owner_id && self.document_id == after.document_id && self.kb_id == after.kb_id,
            "ingestion transition cannot change owner or document");
        let allowed = matches!((self.state, after.state),
            (IngestionState::Pending | IngestionState::Retryable, IngestionState::Processing)
            | (IngestionState::Processing, IngestionState::Retryable | IngestionState::Failed)
            | (IngestionState::Failed | IngestionState::Retryable, IngestionState::Pending));
        anyhow::ensure!(allowed, "invalid ingestion transition");
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum IngestionStoreError {
    #[error("durable ingestion storage unavailable")]
    Unsupported,
    #[error("ingestion generation was revoked or already settled")]
    Revoked,
    #[error("knowledge base is unavailable to the document owner")]
    MissingKnowledgeBase,
}

/// Entire generation is prepared without storage writes, then published once.
#[derive(Debug, Clone)]
pub struct IngestionPublication {
    pub chunks: Vec<KnowledgeChunk>,
    pub embedding: EmbeddingFingerprint,
}

impl IngestionPublication {
    pub fn validate(&self, job: &IngestionJob) -> anyhow::Result<()> {
        job.validate_namespace()?;
        anyhow::ensure!(job.state == IngestionState::Processing, "only processing jobs publish");
        for chunk in &self.chunks {
            anyhow::ensure!(chunk.owner_id == job.owner_id && chunk.kb_id == job.kb_id
                && chunk.document_id.as_deref() == Some(job.document_id.as_str()), "chunk scope differs from job");
            anyhow::ensure!(chunk.embedding.len() == self.embedding.dimension, "embedding dimension differs from publication");
        }
        Ok(())
    }
}
