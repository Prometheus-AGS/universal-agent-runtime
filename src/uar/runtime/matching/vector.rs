use crate::uar::domain::matching::{MatchReason, SkillMatch, SkillMatcher};
use crate::uar::rag::embeddings::{EmbeddingBackend, EmbeddingConfig};
use crate::uar::runtime::matching::cosine_similarity;
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

type EmbeddingCache = Vec<(String, Vec<f32>)>;

/// Real local or hosted embedding inference wrapped behind a unified backend.
/// Replaces the former FastEmbed-only implementation.
#[derive(Debug)]
pub struct VectorMatcher {
    backend: Arc<dyn EmbeddingBackend>,
    embeddings: Arc<Mutex<EmbeddingCache>>,
    threshold: f32,
}

impl VectorMatcher {
    pub fn new(backend: Arc<dyn EmbeddingBackend>, threshold: f32) -> Self {
        Self {
            backend,
            embeddings: Arc::new(Mutex::new(Vec::new())),
            threshold,
        }
    }

    /// Convenience constructor from `EmbeddingConfig`.
    pub fn from_config(config: &EmbeddingConfig, threshold: f32) -> Result<Self> {
        let backend = crate::uar::rag::embeddings::build_backend(config)
            .context("building embedding backend for VectorMatcher")?;
        Ok(Self::new(backend, threshold))
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
        let refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
        self.backend
            .embed(&refs)
            .await
            .context("embedding backend failed")
    }

    pub async fn index_skills(
        &self,
        registry: &crate::uar::runtime::skills::SkillRegistry,
    ) -> Result<()> {
        let skills = registry.list();
        if skills.is_empty() {
            return Ok(());
        }

        let texts: Vec<String> = skills
            .iter()
            .map(|s| format!("{}: {}", s.title, s.description))
            .collect();

        let embeddings = self.embed_batch(texts).await?;

        let mut cache = self.embeddings.lock().await;
        cache.clear();
        for (i, emb) in embeddings.into_iter().enumerate() {
            cache.push((skills[i].skill_id.clone(), emb));
        }
        info!(
            "Skill vector index built ({}).",
            self.backend.backend_name()
        );

        Ok(())
    }
}

#[async_trait]
impl SkillMatcher for VectorMatcher {
    async fn match_skills(
        &self,
        query: &str,
        registry: &crate::uar::runtime::skills::SkillRegistry,
    ) -> Result<Vec<SkillMatch>> {
        let res = self.embed_batch(vec![query.to_string()]).await?;
        let q_emb = res.into_iter().next().context("No embedding")?;

        // Re-index if empty
        {
            let cache = self.embeddings.lock().await;
            if cache.is_empty() {
                drop(cache);
                self.index_skills(registry).await?;
            }
        }

        let cache = self.embeddings.lock().await;
        let mut matches = Vec::new();

        for (skill_id, emb) in cache.iter() {
            let score = cosine_similarity(&q_emb, emb);
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
        Ok(matches)
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
