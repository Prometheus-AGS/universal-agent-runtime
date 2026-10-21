use crate::uar::rag::embeddings::{
    BackendProvenance, EmbeddingBackend, EmbeddingConfig, EmbeddingError,
};
use async_trait::async_trait;
use fastembed::{
    InitOptionsUserDefined, Pooling, TextEmbedding, TokenizerFiles, UserDefinedEmbeddingModel,
};
use sha2::Digest;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex;
use tracing::info;

/// The five on-disk assets a user-defined fastembed model needs.
const MODEL_FILES: [&str; 5] = [
    "bg-small-en-v1.5.onnx",
    "tokenizer.json",
    "config.json",
    "special_tokens_map.json",
    "tokenizer_config.json",
];

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Local FastEmbed/ONNX backend (BGE-small-en-v1.5, 384-dim, CLS-pooled, normalized).
pub struct FastEmbedBackend {
    models_dir: String,
    model: String,
    vector_dimension: usize,
    /// Directory the assets were resolved from at construction. Resolved
    /// eagerly so a missing-assets configuration fails at build/activation
    /// time (truthful status) instead of the first embed call.
    resolved_dir: PathBuf,
    /// Per-file SHA-256 digests (MODEL_FILES order) captured at construction.
    /// `ensure_initialized` re-verifies the bytes it loads against these, so
    /// an asset change between build and load is refused outright — the
    /// reported manifest can never disagree with the vectors being served.
    asset_digests: Vec<(&'static str, String)>,
    /// SHA-256 manifest of the five assets in `resolved_dir`.
    asset_manifest_sha256: String,
    /// Trusted host scope that owns every accepted blocking initialization and
    /// inference operation until its actual join completes.
    lifecycle_scope: crate::uar::runtime::lifecycle::ResourceScope,
    // fastembed 5.x's `TextEmbedding::embed` takes `&mut self`, so the shared
    // engine is wrapped in a std Mutex for exclusive per-call access from the
    // blocking inference thread. The outer async Mutex guards lazy init only.
    engine: Mutex<Option<Arc<StdMutex<TextEmbedding>>>>,
}

impl std::fmt::Debug for FastEmbedBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FastEmbedBackend")
            .field("models_dir", &self.models_dir)
            .field("model", &self.model)
            .field("vector_dimension", &self.vector_dimension)
            .finish_non_exhaustive()
    }
}

impl FastEmbedBackend {
    pub fn new(config: &EmbeddingConfig) -> Result<Self, EmbeddingError> {
        let scope = crate::uar::runtime::lifecycle::current_scope().unwrap_or_else(|| {
            crate::uar::runtime::lifecycle::ResourceScope::standalone(
                "fastembed-compatibility",
                "embedding-backend",
            )
        });
        Self::new_with_scope(config, scope)
    }

    pub fn new_with_scope(
        config: &EmbeddingConfig,
        lifecycle_scope: crate::uar::runtime::lifecycle::ResourceScope,
    ) -> Result<Self, EmbeddingError> {
        let resolved_dir = Self::resolve_models_dir(&config.models_dir, config.strict_models_dir)?;
        let (asset_digests, asset_manifest_sha256) = Self::hash_assets(&resolved_dir)?;
        Ok(Self {
            models_dir: config.models_dir.clone(),
            // Legacy single-backend callers retain the packaged BGE identity.
            // Opt-in failover members use strict directories and bind the
            // explicit model identifier to the recorded asset manifest.
            model: if config.strict_models_dir {
                config.model.clone()
            } else {
                "bge-small-en-v1.5".to_string()
            },
            vector_dimension: config.vector_dimension,
            resolved_dir,
            asset_digests,
            asset_manifest_sha256,
            lifecycle_scope,
            engine: Mutex::new(None),
        })
    }

    fn resolve_models_dir(models_dir: &str, strict: bool) -> Result<PathBuf, EmbeddingError> {
        if strict {
            let dir = PathBuf::from(models_dir);
            return MODEL_FILES
                .iter()
                .all(|file| dir.join(file).is_file())
                .then_some(dir)
                .ok_or_else(|| {
                    EmbeddingError::BackendUnavailable(format!(
                        "strict local embedding assets ({}) are not all present in configured directory {}",
                        MODEL_FILES.join(", "),
                        models_dir
                    ))
                });
        }
        let candidates = [
            std::env::var("UAR_MODELS_DIR").ok().map(PathBuf::from),
            Some(PathBuf::from(models_dir)),
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
        Err(EmbeddingError::BackendUnavailable(format!(
            "embedding model assets ({}) not found in any candidate dir: {}",
            MODEL_FILES.join(", "),
            candidates
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )))
    }

    /// SHA-256 over the asset names (MODEL_FILES declaration order) and their
    /// per-file SHA-256 digests, so byte-identical mirrors produce an
    /// identical manifest and any single-asset drift produces a different
    /// one. Returns the per-file digests alongside the manifest.
    fn hash_assets(dir: &Path) -> Result<(Vec<(&'static str, String)>, String), EmbeddingError> {
        let mut digests = Vec::with_capacity(MODEL_FILES.len());
        let mut manifest = sha2::Sha256::new();
        for name in MODEL_FILES {
            let bytes = std::fs::read(dir.join(name))
                .map_err(|e| EmbeddingError::BackendUnavailable(format!("reading {name}: {e}")))?;
            let hex = hex_encode(&sha2::Sha256::digest(&bytes));
            manifest.update(name.as_bytes());
            manifest.update([0u8]);
            manifest.update(hex.as_bytes());
            manifest.update([0u8]);
            digests.push((name, hex));
        }
        Ok((digests, hex_encode(&manifest.finalize())))
    }

    async fn ensure_initialized(&self) -> Result<Arc<StdMutex<TextEmbedding>>, EmbeddingError> {
        let mut guard = self.engine.lock().await;
        if let Some(engine) = guard.as_ref() {
            return Ok(Arc::clone(engine));
        }

        let dir = self.resolved_dir.clone();
        let digests = self.asset_digests.clone();
        let model_id = self.model.clone();
        info!(dir = %dir.display(), model = %model_id, "Initializing fastembed engine");

        let engine = self
            .lifecycle_scope
            .spawn_blocking("fastembed-initialize", move || {
            let read = |name: &str| -> Result<Vec<u8>, EmbeddingError> {
                let bytes = std::fs::read(dir.join(name))
                    .map_err(|e| EmbeddingError::BackendUnavailable(format!("reading {name}: {e}")))?;
                let actual = hex_encode(&sha2::Sha256::digest(&bytes));
                let expected = digests.iter().find(|(n, _)| *n == name).map(|(_, h)| h);
                if expected != Some(&actual) {
                    return Err(EmbeddingError::BackendUnavailable(format!(
                        "asset {name} changed since backend construction; the reported \
                         provenance manifest no longer describes the bytes being served — \
                         refusing to load (rebuild the backend to re-resolve and re-hash)"
                    )));
                }
                Ok(bytes)
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
            model.pooling = Some(Pooling::Cls);

            TextEmbedding::try_new_from_user_defined(
                model,
                InitOptionsUserDefined::new().with_max_length(512),
            )
            .map_err(|e| EmbeddingError::BackendUnavailable(format!("initializing: {e}")))
            })
            .map_err(|error| EmbeddingError::BackendUnavailable(format!("initialization admission closed: {error}")))?
            .await
            .map_err(|error| EmbeddingError::BackendUnavailable(format!("init task panicked: {error}")))?;

        let engine = engine?;
        let engine = Arc::new(StdMutex::new(engine));
        *guard = Some(Arc::clone(&engine));
        Ok(engine)
    }
}

#[async_trait]
impl EmbeddingBackend for FastEmbedBackend {
    fn backend_name(&self) -> &str {
        "fastembed"
    }

    fn vector_dimension(&self) -> usize {
        self.vector_dimension
    }

    fn model_id(&self) -> &str {
        &self.model
    }

    fn provenance(&self) -> Option<BackendProvenance> {
        Some(BackendProvenance {
            source_dir: self.resolved_dir.display().to_string(),
            asset_manifest_sha256: self.asset_manifest_sha256.clone(),
        })
    }

    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(vec![]);
        }
        let engine = self.ensure_initialized().await?;
        let texts: Vec<String> = texts.iter().map(|s| s.to_string()).collect();
        let expected_dimension = self.vector_dimension;
        self.lifecycle_scope
            .spawn_blocking("fastembed-inference", move || {
            // fastembed 5.x `embed` needs `&mut self`; lock the std Mutex for
            // exclusive access on this blocking thread.
            let mut engine = engine
                .lock()
                .map_err(|e| EmbeddingError::RequestFailed(format!("engine lock poisoned: {e}")))?;
            let vectors = engine
                .embed(texts, None)
                .map_err(|e| EmbeddingError::RequestFailed(format!("inference failed: {e}")))?;
            for vector in &vectors {
                if vector.len() != expected_dimension {
                    return Err(EmbeddingError::DimensionMismatch {
                        expected: expected_dimension,
                        actual: vector.len(),
                    });
                }
            }
            Ok(vectors)
            })
            .map_err(|error| EmbeddingError::BackendUnavailable(format!("inference admission closed: {error}")))?
            .await
            .map_err(|error| EmbeddingError::RequestFailed(format!("embedding task panicked: {error}")))?
    }

    async fn shutdown(&self) -> Result<(), EmbeddingError> {
        let mut engine = self.engine.lock().await;
        let _ = engine.take();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fastembed_embeddings_are_nonzero_normalized_and_discriminative() {
        let cfg = EmbeddingConfig::default();
        let backend = FastEmbedBackend::new(&cfg).expect("backend should build");
        let out = backend
            .embed(&[
                "The quarterly financial report shows revenue growth.",
                "Quarterly finances: the report indicates revenues grew.",
                "My cat enjoys sleeping in cardboard boxes.",
            ])
            .await
            .expect("embed");

        assert_eq!(out.len(), 3);
        for v in &out {
            assert_eq!(v.len(), 384);
            let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!((norm - 1.0).abs() < 1e-3, "expected ~unit norm, got {norm}");
        }

        let near = crate::uar::runtime::matching::cosine_similarity(&out[0], &out[1]);
        let far = crate::uar::runtime::matching::cosine_similarity(&out[0], &out[2]);
        assert!(
            near > far,
            "near-duplicate pair ({near}) must beat unrelated pair ({far})"
        );
        assert!(near > 0.8, "near-duplicate similarity too low: {near}");
    }

    #[tokio::test]
    async fn empty_batch_is_ok() {
        let cfg = EmbeddingConfig::default();
        let backend = FastEmbedBackend::new(&cfg).expect("backend should build");
        let out = backend.embed(&[]).await.expect("embed empty");
        assert!(out.is_empty());
    }
}
