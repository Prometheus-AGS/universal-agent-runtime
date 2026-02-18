//! Local embedding intent classifier.
//!
//! Uses the existing `VectorMatcher` to embed skill descriptions and queries,
//! then ranks skills by cosine similarity — all on-device, no API calls.

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

/// On-device embedding classifier using `VectorMatcher`.
#[derive(Debug)]
pub struct LocalEmbeddingClassifier {
    vector_matcher: Arc<VectorMatcher>,
    cache: RwLock<Vec<SkillEmbedding>>,
    topk: usize,
}

impl LocalEmbeddingClassifier {
    /// Creates a new local embedding classifier.
    pub fn new(vector_matcher: Arc<VectorMatcher>, topk: usize) -> Self {
        Self {
            vector_matcher,
            cache: RwLock::new(Vec::new()),
            topk,
        }
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
        // Embed the query
        let query_embeddings = self
            .vector_matcher
            .embed_batch(vec![query.to_string()])
            .await?;
        let query_vec = query_embeddings
            .first()
            .ok_or_else(|| anyhow::anyhow!("Failed to embed query"))?;

        // Read cached skill embeddings
        let cache = self.cache.read().await;

        if cache.is_empty() {
            // Cache not built yet — fallback to keyword
            tracing::warn!("Local embedding cache empty, rebuild_index not called");
            return Ok(ClassificationResult::new(Vec::new()));
        }

        // Compute cosine similarity for each cached skill
        let all_skills = registry.list_enabled();
        let skill_map: std::collections::HashMap<&str, &Skill> = all_skills
            .iter()
            .map(|s| (s.skill_id.as_str(), s))
            .collect();

        let mut scores: Vec<IntentScore> = cache
            .iter()
            .filter_map(|se| {
                let score = cosine_similarity(&se.embedding, query_vec);
                skill_map.get(se.skill_id.as_str()).map(|skill| {
                    IntentScore::with_skill(se.skill_id.clone(), score, (*skill).clone())
                })
            })
            .collect();

        // Sort descending
        scores.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scores.truncate(self.topk);

        Ok(ClassificationResult::new(scores))
    }

    async fn rebuild_index(&self, registry: &SkillRegistry) -> anyhow::Result<()> {
        let skills = registry.list_enabled();
        if skills.is_empty() {
            *self.cache.write().await = Vec::new();
            return Ok(());
        }

        // Build text representations
        let texts: Vec<String> = skills
            .iter()
            .map(|s| {
                format!(
                    "{}: {} {}",
                    s.title,
                    s.description,
                    s.triggers.keywords.join(" ")
                )
            })
            .collect();

        // Embed all at once
        let embeddings = self.vector_matcher.embed_batch(texts).await?;

        let mut entries = Vec::with_capacity(skills.len());
        for (skill, emb) in skills.iter().zip(embeddings.into_iter()) {
            entries.push(SkillEmbedding {
                skill_id: skill.skill_id.clone(),
                embedding: emb,
            });
        }

        *self.cache.write().await = entries;
        tracing::info!("Local embedding index rebuilt with {} skills", skills.len());
        Ok(())
    }
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
