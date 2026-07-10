use crate::uar::domain::matching::{MatchReason, SkillMatch, SkillMatcher};
use crate::uar::runtime::skills::SkillRegistry;
use anyhow::{Context, Result};
use async_trait::async_trait;
use fastembed::{
    InitOptionsUserDefined, Pooling, TextEmbedding, TokenizerFiles, UserDefinedEmbeddingModel,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

type EmbeddingCache = Vec<(String, Vec<f32>)>;

/// Real local embedding inference (BGE-small-en-v1.5, 384-dim, CLS-pooled,
/// L2-normalized) via fastembed/ONNX Runtime, loaded entirely from on-disk
/// assets — no network access at runtime. Replaces the former burn-based
/// placeholder that returned zero vectors (`fix-embeddings-fastembed`).
pub struct VectorMatcher {
    engine: Arc<Mutex<Option<Arc<TextEmbedding>>>>,
    embeddings: Arc<Mutex<EmbeddingCache>>,
    threshold: f32,
    models_dir: String,
}

impl std::fmt::Debug for VectorMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VectorMatcher")
            .field("backend", &"fastembed-onnx (bge-small-en-v1.5)")
            .field("threshold", &self.threshold)
            .finish()
    }
}

/// The five on-disk assets a user-defined fastembed model needs.
const MODEL_FILES: [&str; 5] = [
    "bg-small-en-v1.5.onnx",
    "tokenizer.json",
    "config.json",
    "special_tokens_map.json",
    "tokenizer_config.json",
];

impl VectorMatcher {
    pub fn new(threshold: f32, models_dir: String) -> Self {
        Self {
            engine: Arc::new(Mutex::new(None)),
            embeddings: Arc::new(Mutex::new(Vec::new())),
            threshold,
            models_dir,
        }
    }

    /// Locate a directory containing all required model assets, trying the
    /// same candidate order the previous implementation used: env override,
    /// configured dir, container path, then development paths.
    fn resolve_models_dir(&self) -> Result<PathBuf> {
        let candidates = [
            std::env::var("UAR_MODELS_DIR").ok().map(PathBuf::from),
            Some(PathBuf::from(&self.models_dir)),
            Some(PathBuf::from("/app/models")),
            Some(PathBuf::from("src/uar/runtime/matching/models")),
            Some(PathBuf::from("./src/uar/runtime/matching/models")),
        ];

        let candidates: Vec<PathBuf> = candidates.into_iter().flatten().collect();
        for dir in &candidates {
            if MODEL_FILES.iter().all(|f| dir.join(f).exists()) {
                return Ok(dir.clone());
            }
        }
        anyhow::bail!(
            "embedding model assets ({}) not found in any candidate dir: {}",
            MODEL_FILES.join(", "),
            candidates
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    pub async fn initialize(&self) -> Result<()> {
        let mut engine_guard = self.engine.lock().await;
        if engine_guard.is_some() {
            return Ok(());
        }

        let dir = self.resolve_models_dir()?;
        info!(dir = %dir.display(), "Initializing fastembed engine (bge-small-en-v1.5)…");

        // Session construction is synchronous CPU/IO work — keep it off the
        // async executor.
        let engine = tokio::task::spawn_blocking(move || -> Result<TextEmbedding> {
            let read = |name: &str| -> Result<Vec<u8>> {
                std::fs::read(dir.join(name))
                    .with_context(|| format!("reading embedding asset {name}"))
            };
            let mut model = UserDefinedEmbeddingModel::new(
                read("bg-small-en-v1.5.onnx")?,
                TokenizerFiles {
                    tokenizer_file: read("tokenizer.json")?,
                    config_file: read("config.json")?,
                    special_tokens_map_file: read("special_tokens_map.json")?,
                    tokenizer_config_file: read("tokenizer_config.json")?,
                },
            );
            // BGE-family models are CLS-pooled (matches fastembed's own
            // built-in BGESmallENV15 definition).
            model.pooling = Some(Pooling::Cls);

            TextEmbedding::try_new_from_user_defined(
                model,
                InitOptionsUserDefined::new().with_max_length(512),
            )
            .map_err(|e| anyhow::anyhow!("initializing fastembed engine: {e}"))
        })
        .await
        .context("embedding engine init task panicked")??;

        *engine_guard = Some(Arc::new(engine));
        info!("Embedding engine ready (384-dim, CLS pooling, normalized).");
        Ok(())
    }

    pub async fn embed_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let engine = {
            let guard = self.engine.lock().await;
            guard
                .as_ref()
                .cloned()
                .context("Embedding engine not initialized")?
        };

        // fastembed inference is synchronous CPU work.
        tokio::task::spawn_blocking(move || {
            engine
                .embed(texts, None)
                .map_err(|e| anyhow::anyhow!("embedding inference failed: {e}"))
        })
        .await
        .context("embedding task panicked")?
    }

    pub async fn index_skills(&self, registry: &SkillRegistry) -> Result<()> {
        let skills = registry.list();
        if skills.is_empty() {
            return Ok(());
        }

        let texts: Vec<String> = skills
            .iter()
            .map(|s| format!("{}: {}", s.title, s.description))
            .collect();

        self.initialize().await?;

        let embeddings = self.embed_batch(texts).await?;

        let mut cache = self.embeddings.lock().await;
        cache.clear();
        for (i, emb) in embeddings.into_iter().enumerate() {
            cache.push((skills[i].skill_id.clone(), emb));
        }
        info!("Skill vector index built (fastembed).");

        Ok(())
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
        let norm_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }
}

#[async_trait]
impl SkillMatcher for VectorMatcher {
    async fn match_skills(&self, query: &str, registry: &SkillRegistry) -> Result<Vec<SkillMatch>> {
        self.initialize().await?;

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
            let score = Self::cosine_similarity(&q_emb, emb);
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
        matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        Ok(matches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matcher() -> VectorMatcher {
        VectorMatcher::new(0.75, "src/uar/runtime/matching/models".to_string())
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

        let near = VectorMatcher::cosine_similarity(&out[0], &out[1]);
        let far = VectorMatcher::cosine_similarity(&out[0], &out[2]);
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
