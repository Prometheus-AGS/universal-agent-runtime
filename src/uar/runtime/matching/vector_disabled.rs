//! Explicit unavailable implementation for builds without local models.

use anyhow::{Result, bail};
use async_trait::async_trait;
use std::sync::Arc;

use crate::uar::{
    domain::matching::{SkillMatch, SkillMatcher},
    rag::embeddings::EmbeddingBackend,
    runtime::skills::SkillRegistry,
};

/// Vector matcher facade for builds without `local-models`.
#[derive(Debug)]
pub struct VectorMatcher {
    backend: Arc<dyn EmbeddingBackend>,
    _threshold: f32,
}

impl VectorMatcher {
    /// Create a disabled matcher preserving the runtime construction contract.
    #[must_use]
    pub fn new(backend: Arc<dyn EmbeddingBackend>, threshold: f32) -> Self {
        Self {
            backend,
            _threshold: threshold,
        }
    }

    /// The backend built from `llm.embedding`, shared with ingestion. Only
    /// skill matching is disabled in this build; knowledge-base retrieval
    /// embeds through this backend directly, so hosted backends still work.
    pub fn embedding_backend(&self) -> &Arc<dyn EmbeddingBackend> {
        &self.backend
    }

    /// The embedding space queries are produced in.
    pub fn embedding_fingerprint(&self) -> crate::uar::domain::knowledge::EmbeddingFingerprint {
        self.backend.fingerprint()
    }

    /// Report that local model initialization is unavailable.
    pub async fn initialize(&self) -> Result<()> {
        bail!("local embeddings are unavailable: rebuild with `local-models`")
    }

    /// Embedding is not silently approximated when the capability is absent.
    pub async fn embed_batch(&self, _texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        bail!("local embeddings are unavailable: rebuild with `local-models`")
    }

    /// No local index is produced in a capability-disabled build.
    pub async fn index_skills(&self, _registry: &SkillRegistry) -> Result<()> {
        bail!("local embeddings are unavailable: rebuild with `local-models`")
    }
}

#[async_trait]
impl SkillMatcher for VectorMatcher {
    async fn match_skills(
        &self,
        _query: &str,
        _registry: &SkillRegistry,
    ) -> Result<Vec<SkillMatch>> {
        Ok(Vec::new())
    }
}
