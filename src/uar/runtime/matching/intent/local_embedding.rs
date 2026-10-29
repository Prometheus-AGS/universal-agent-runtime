//! Local embedding intent classifier.
//!
//! Uses one captured FastEmbed asset-backed instance for descriptions and query;
//! a hosted or unavailable embedding resource is rejected before inference.

use super::{ClassificationResult, IntentClassifier, IntentScore};
use crate::uar::domain::skills::Skill;
use crate::uar::runtime::matching::vector::VectorMatcher;
use crate::uar::runtime::skills::SkillRegistry;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Cached embedding for a single skill.
#[derive(Debug, Clone)]
struct SkillEmbedding {
    skill_id: String,
    embedding: Vec<f32>,
}

/// On-device embedding classifier with an immutable serving-instance pin.
#[derive(Debug)]
pub struct LocalEmbeddingClassifier {
    embedding_backend: Arc<dyn crate::uar::rag::embeddings::EmbeddingBackend>,
    cache: RwLock<Vec<SkillEmbedding>>,
    topk: usize,
}

impl LocalEmbeddingClassifier {
    /// Capture the resource once; existing constructor callers remain supported.
    /// Hosted resources fail at classification/indexing rather than sending I/O.
    pub fn new(vector_matcher: Arc<VectorMatcher>, topk: usize) -> Self {
        Self {
            embedding_backend: vector_matcher.pin_embedding(),
            cache: RwLock::new(Vec::new()),
            topk,
        }
    }

    fn require_local_resource(&self) -> anyhow::Result<()> {
        anyhow::ensure!(self.embedding_backend.backend_name() == "fastembed"
            && self.embedding_backend.endpoint().is_none()
            && self.embedding_backend.provenance().is_some(),
            "local_embedding_requires_pinned_fastembed_assets");
        Ok(())
    }

    /// Rank the service's already scoped candidates without reapplying global
    /// enabled flags or allowing other scopes to consume the request's top-k.
    pub(crate) async fn classify_candidates(
        &self,
        query: &str,
        skills: &[Skill],
    ) -> anyhow::Result<ClassificationResult> {
        self.rebuild_skills(skills).await?;
        self.score_skills(query, skills).await
    }

    async fn rebuild_skills(&self, skills: &[Skill]) -> anyhow::Result<()> {
        self.require_local_resource()?;
        if skills.is_empty() {
            *self.cache.write().await = Vec::new();
            return Ok(());
        }
        let texts = skills.iter().map(|skill| format!("{}: {} {}",
            skill.title, skill.description, skill.triggers.keywords.join(" "))).collect::<Vec<_>>();
        let inputs = texts.iter().map(String::as_str).collect::<Vec<_>>();
        let embeddings = self.embedding_backend.embed(&inputs).await?;
        anyhow::ensure!(embeddings.len() == skills.len(), "local_embedding_batch_length_mismatch");
        *self.cache.write().await = skills.iter().zip(embeddings).map(|(skill, embedding)| SkillEmbedding {
            skill_id: skill.skill_id.clone(), embedding,
        }).collect();
        tracing::info!(skills = skills.len(), "Local embedding index rebuilt");
        Ok(())
    }

    async fn score_skills(&self, query: &str, skills: &[Skill]) -> anyhow::Result<ClassificationResult> {
        self.require_local_resource()?;
        let cache = self.cache.read().await;
        if cache.is_empty() {
            anyhow::ensure!(skills.is_empty(), "local_embedding_index_not_built");
            return Ok(ClassificationResult::new(Vec::new()));
        }
        let query_embeddings = self.embedding_backend.embed(&[query]).await?;
        let query_vec = query_embeddings.first()
            .ok_or_else(|| anyhow::anyhow!("Failed to embed query"))?;
        let skill_map = skills.iter().map(|skill| (skill.skill_id.as_str(), skill))
            .collect::<std::collections::HashMap<_, _>>();
        let mut scores = cache.iter().filter_map(|entry| {
            skill_map.get(entry.skill_id.as_str()).map(|skill| IntentScore::with_skill(
                entry.skill_id.clone(), cosine_similarity(&entry.embedding, query_vec), (*skill).clone()))
        }).collect::<Vec<_>>();
        scores.sort_by(|a, b| b.score.total_cmp(&a.score));
        scores.truncate(self.topk);
        Ok(ClassificationResult::new(scores))
    }
}

#[async_trait]
impl IntentClassifier for LocalEmbeddingClassifier {
    async fn classify(
        &self,
        query: &str,
        _context: &[String],
        registry: &SkillRegistry,
    ) -> anyhow::Result<ClassificationResult> {
        let raw = self.score_skills(query, &registry.list_enabled()).await?;
        Ok(ClassificationResult::new(raw.scores.into_iter()
            .map(|score| IntentScore { score: calibrated_confidence(score.score), ..score })
            .collect()))
    }

    async fn rebuild_index(&self, registry: &SkillRegistry) -> anyhow::Result<()> {
        self.rebuild_skills(&registry.list_enabled()).await
    }
}

/// Raw cosine similarity between unrelated texts sits near this level with the
/// pinned FastEmbed model (about 0.57 observed on Mac, 2026-10-28).
const UNRELATED_COSINE_FLOOR: f32 = 0.5;

/// The legacy classifier shares one accept threshold (default 0.5) with tfidf and
/// rules, so its raw cosine is rescaled from the unrelated floor: 0.5 then means
/// raw cosine 0.75, UAR's semantic threshold elsewhere (`rag.semantic_threshold`).
/// The skill service's `local_embedding` path keeps raw scores and its own threshold.
fn calibrated_confidence(raw: f32) -> f32 {
    ((raw - UNRELATED_COSINE_FLOOR) / (1.0 - UNRELATED_COSINE_FLOOR)).clamp(0.0, 1.0)
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}
