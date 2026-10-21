use crate::uar::domain::knowledge::EmbeddingFingerprint;
use crate::uar::domain::matching::{MatchReason, SkillMatch, SkillMatcher};
use crate::uar::rag::embeddings::swappable::SwappableEmbeddingBackend;
use crate::uar::rag::embeddings::{EmbeddingBackend, EmbeddingConfig};
use crate::uar::runtime::matching::cosine_similarity;
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

type EmbeddingCache = Vec<(String, Vec<f32>)>;

/// Derived in-memory skill index plus the embedding space it was built in.
/// Space and vectors are guarded by ONE lock, and every job pins its backend
/// once, so a query can never be compared against vectors from another space
/// even when a live `llm.embedding.*` swap lands mid-flight.
#[derive(Debug, Default)]
struct SkillIndex {
    space: Option<EmbeddingFingerprint>,
    vectors: EmbeddingCache,
}

/// Real local or hosted embedding inference wrapped behind a unified backend.
/// Replaces the former FastEmbed-only implementation.
///
/// The backend handle is swappable (`llm.embedding.*` settings are live). The
/// in-memory skill index is derived data: whenever the active embedding space
/// no longer matches the space the index was built in, the index is rebuilt —
/// a query is never compared against vectors from another space.
#[derive(Debug)]
pub struct VectorMatcher {
    backend: Arc<SwappableEmbeddingBackend>,
    index: Arc<RwLock<SkillIndex>>,
    threshold: f32,
}

impl VectorMatcher {
    pub fn new(backend: Arc<SwappableEmbeddingBackend>, threshold: f32) -> Self {
        Self {
            backend,
            index: Arc::new(RwLock::new(SkillIndex::default())),
            threshold,
        }
    }

    /// Pin the current backend as one coherent snapshot for a job (a query or
    /// an index rebuild). The pin is what makes a concurrent backend swap
    /// safe: the job finishes in the space it started in.
    pub fn pin_embedding(&self) -> Arc<dyn EmbeddingBackend> {
        self.backend.pin()
    }

    /// The embedding space the next pinned job would produce vectors in.
    pub fn embedding_fingerprint(&self) -> EmbeddingFingerprint {
        self.backend.fingerprint()
    }

    /// Convenience constructor from `EmbeddingConfig`.
    pub fn from_config(config: &EmbeddingConfig, threshold: f32) -> Result<Self> {
        let backend = crate::uar::rag::embeddings::build_backend(config)
            .context("building embedding backend for VectorMatcher")?;
        Ok(Self::new(
            Arc::new(SwappableEmbeddingBackend::new(backend)),
            threshold,
        ))
    }

    pub async fn initialize(&self) -> Result<()> {
        // Most backends lazy-initialize on first embed call; the public trait
        // intentionally hides init details. We probe the backend once to surface
        // any configuration errors early.
        let _ = self.embed_batch(vec!["probe".to_string()]).await?;
        Ok(())
    }

    pub async fn embed_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }
        let backend = self.backend.pin();
        let refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
        backend
            .embed(&refs)
            .await
            .context("embedding backend failed")
    }

    /// Embed `texts` with one pinned backend and return the vectors plus the
    /// space they were produced in — the pair is published or discarded
    /// together, never mixed.
    async fn embed_with_space(
        backend: &Arc<dyn EmbeddingBackend>,
        texts: Vec<String>,
    ) -> Result<(EmbeddingFingerprint, Vec<Vec<f32>>)> {
        if texts.is_empty() {
            return Ok((backend.fingerprint(), Vec::new()));
        }
        let refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
        let vectors = backend
            .embed(&refs)
            .await
            .context("embedding backend failed")?;
        Ok((backend.fingerprint(), vectors))
    }

    fn skill_texts(
        registry: &crate::uar::runtime::skills::SkillRegistry,
    ) -> (Vec<String>, Vec<String>) {
        let skills = registry.list();
        let ids = skills.iter().map(|s| s.skill_id.clone()).collect();
        let texts = skills
            .iter()
            .map(|s| format!("{}: {}", s.title, s.description))
            .collect();
        (ids, texts)
    }

    pub async fn index_skills(
        &self,
        registry: &crate::uar::runtime::skills::SkillRegistry,
    ) -> Result<()> {
        // ONE pin for the whole rebuild: the recorded space always describes
        // the vectors it is stored with, even if a swap lands mid-embed.
        let backend = self.backend.pin();
        let (ids, texts) = Self::skill_texts(registry);
        let (space, vectors) = Self::embed_with_space(&backend, texts).await?;
        let mut index = self.index.write().await;
        index.vectors.clear();
        index
            .vectors
            .extend(ids.into_iter().zip(vectors.into_iter()));
        index.space = Some(space);
        info!("Skill vector index built ({}).", backend.backend_name());
        Ok(())
    }

    fn score_index(
        &self,
        index: &SkillIndex,
        q_emb: &[f32],
        registry: &crate::uar::runtime::skills::SkillRegistry,
    ) -> Vec<SkillMatch> {
        let mut matches = Vec::new();
        for (skill_id, emb) in index.vectors.iter() {
            let score = cosine_similarity(q_emb, emb);
            if score >= self.threshold
                && let Some(skill) = registry.get(skill_id)
            {
                matches.push(SkillMatch {
                    skill_id: skill_id.clone(),
                    score,
                    reason: MatchReason::VectorSimilarity(score),
                    skill: skill.clone(),
                });
            }
        }
        matches.sort_by(|a, b| b.score.total_cmp(&a.score));
        matches
    }
}

#[async_trait]
impl SkillMatcher for VectorMatcher {
    async fn match_skills(
        &self,
        query: &str,
        registry: &crate::uar::runtime::skills::SkillRegistry,
    ) -> Result<Vec<SkillMatch>> {
        // Each pass pins the backend once and only compares when the index
        // space equals the pin's space, so a swap can never interleave into a
        // cross-space comparison. A rebuild publishes vectors+space under one
        // write guard. The loop converges because swaps are operator-rare.
        loop {
            let backend = self.backend.pin();
            let space = backend.fingerprint();
            {
                let index = self.index.read().await;
                if index.space.as_ref() == Some(&space) {
                    // Compare under the read guard: a rebuild (writer) cannot
                    // interleave, and the query vector comes from the same pin.
                    let res = backend
                        .embed(&[query])
                        .await
                        .context("embedding backend failed")?;
                    let q_emb = res.into_iter().next().context("No embedding")?;
                    return Ok(self.score_index(&index, &q_emb, registry));
                }
            }
            // Slow path: (re)build the index for our pinned space under the
            // write guard. Re-check after acquiring: another task may have
            // already rebuilt for this space.
            let mut index = self.index.write().await;
            if index.space.as_ref() == Some(&space) {
                continue;
            }
            if let Some(previous) = index.space.as_ref() {
                warn!(
                    indexed = ?previous,
                    active = ?space,
                    "Embedding space changed; rebuilding in-memory skill vector index"
                );
            }
            let (ids, texts) = Self::skill_texts(registry);
            let (built_space, vectors) = Self::embed_with_space(&backend, texts).await?;
            index.vectors.clear();
            index
                .vectors
                .extend(ids.into_iter().zip(vectors.into_iter()));
            index.space = Some(built_space);
            drop(index);
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    fn matcher() -> VectorMatcher {
        let cfg = EmbeddingConfig::default();
        VectorMatcher::from_config(&cfg, 0.75)
            .expect("VectorMatcher should build from default config")
    }

    #[tokio::test]
    async fn embeddings_are_nonzero_normalized_and_discriminative() {
        let m = matcher();
        m.initialize().await.expect("engine init");

        let out = m
            .embed_batch(vec![
                "The quarterly financial report shows revenue growth.".to_string(),
                "Quarterly finances: the report indicates revenues grew.".to_string(),
                "My cat enjoys sleeping in cardboard boxes.".to_string(),
            ])
            .await
            .expect("embed");

        assert_eq!(out.len(), 3);
        for v in &out {
            assert_eq!(v.len(), 384);
            let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!((norm - 1.0).abs() < 1e-3, "expected ~unit norm, got {norm}");
        }

        let near = cosine_similarity(&out[0], &out[1]);
        let far = cosine_similarity(&out[0], &out[2]);
        assert!(
            near > far,
            "near-duplicate pair ({near}) must beat unrelated pair ({far})"
        );
        assert!(near > 0.8, "near-duplicate similarity too low: {near}");
    }

    #[tokio::test]
    async fn empty_batch_is_ok() {
        let m = matcher();
        m.initialize().await.expect("engine init");
        let out = m.embed_batch(vec![]).await.expect("embed empty");
        assert!(out.is_empty());
    }
}
