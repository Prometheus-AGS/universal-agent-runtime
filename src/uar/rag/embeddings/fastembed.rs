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
    // fastembed 5.x's `TextEmbedding::embed` takes `&mut self`, so the shared
    // engine is wrapped in a std Mutex for exclusive per-call access from the
    // blocking inference thread. The outer async Mutex guards lazy init only.
    engine: Mutex<Option<Arc<StdMutex<TextEmbedding>>>>,
}

impl std::fmt::Debug for FastEmbedBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FastEmbedBackend")
            .field("models_dir", &self.models_dir)
            .field("vector_dimension", &self.vector_dimension)
            .finish_non_exhaustive()
    }
}

impl FastEmbedBackend {
    pub fn new(config: &EmbeddingConfig) -> Result<Self, EmbeddingError> {
        let resolved_dir = Self::resolve_models_dir(&config.models_dir)?;
        let (asset_digests, asset_manifest_sha256) = Self::hash_assets(&resolved_dir)?;
        Ok(Self {
            models_dir: config.models_dir.clone(),
            vector_dimension: config.vector_dimension,
            resolved_dir,
            asset_digests,
            asset_manifest_sha256,
            engine: Mutex::new(None),
        })
    }

    fn resolve_models_dir(models_dir: &str) -> Result<PathBuf, EmbeddingError> {
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
        info!(dir = %dir.display(), "Initializing fastembed engine (bge-small-en-v1.5)…");

        let engine = tokio::task::spawn_blocking(move || {
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
        .await
        .map_err(|e| EmbeddingError::BackendUnavailable(format!("init task panicked: {e}")))?;

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

    // The packaged ONNX assets are always BGE small; `llm.embedding.model` is
    // not consulted by this backend.
    fn model_id(&self) -> &str {
        "bge-small-en-v1.5"
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
        tokio::task::spawn_blocking(move || {
            // fastembed 5.x `embed` needs `&mut self`; lock the std Mutex for
            // exclusive access on this blocking thread.
            let mut engine = engine
                .lock()
                .map_err(|e| EmbeddingError::RequestFailed(format!("engine lock poisoned: {e}")))?;
            engine
                .embed(texts, None)
                .map_err(|e| EmbeddingError::RequestFailed(format!("inference failed: {e}")))
        })
        .await
        .map_err(|e| EmbeddingError::RequestFailed(format!("embedding task panicked: {e}")))?
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
