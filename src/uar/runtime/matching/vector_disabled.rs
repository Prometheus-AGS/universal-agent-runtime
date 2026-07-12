//! Explicit unavailable implementation for builds without local models.

use anyhow::{Result, bail};
use async_trait::async_trait;

use crate::uar::{
    domain::matching::{SkillMatch, SkillMatcher},
    runtime::skills::SkillRegistry,
};

/// Vector matcher facade for builds without `local-models`.
#[derive(Debug)]
pub struct VectorMatcher {
    _threshold: f32,
    _models_dir: String,
}

impl VectorMatcher {
    /// Create a disabled matcher preserving the runtime construction contract.
    #[must_use]
    pub fn new(threshold: f32, models_dir: String) -> Self {
        Self {
            _threshold: threshold,
            _models_dir: models_dir,
        }
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
