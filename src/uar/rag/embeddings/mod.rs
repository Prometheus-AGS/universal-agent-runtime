use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;
use thiserror::Error;

/// Unified embedding backend for RAG and skill matching.
///
/// Implementations may be local (FastEmbed, Candle) or hosted (OpenAI, Voyage,
/// Cohere). Callers operate against the trait and do not depend on a specific
/// provider.
#[async_trait]
pub trait EmbeddingBackend: Send + Sync + Debug {
    /// Return the provider name for logging and diagnostics.
    fn backend_name(&self) -> &str;

    /// Return the expected output vector dimension.
    fn vector_dimension(&self) -> usize;

    /// Return the model identifier, used to fingerprint the embedding space a
    /// knowledge base was indexed in. Implementations with a configurable
    /// model must override this; two models of equal dimension are not
    /// interchangeable.
    fn model_id(&self) -> &str {
        "unknown"
    }

    /// Actual remote endpoint, if this backend sends embedding requests over
    /// HTTP. Host settings inspection exposes only its host, never credentials.
    fn endpoint(&self) -> Option<&str> {
        None
    }

    /// Operator-visible provenance of the serving INSTANCE (where its assets
    /// were resolved from, plus their digest). This is instance identity, not
    /// space identity: two byte-identical asset copies are one embedding
    /// space served by two instances, and a failover between them must stay
    /// visible in status surfaces instead of silent. Defaults to `None` for
    /// backends without local assets.
    fn provenance(&self) -> Option<BackendProvenance> {
        None
    }

    /// Embed a batch of texts. The returned outer vector has the same length as
    /// `texts` (unless `texts` is empty, in which case an empty vector is
    /// returned). Each inner vector has `vector_dimension()` elements.
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError>;

    /// Release local inference resources after the trusted host has closed
    /// admission and joined all accepted work. Backends without owned local
    /// resources retain the default no-op implementation.
    async fn shutdown(&self) -> Result<(), EmbeddingError> {
        Ok(())
    }
}

/// Instance provenance for status surfaces. Deliberately NOT part of
/// `EmbeddingFingerprint`: an instance change is not a space change, so the
/// KB space guard must never compare these fields.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackendProvenance {
    /// Directory the serving assets were resolved from.
    pub source_dir: String,
    /// SHA-256 over the sorted asset filenames and their per-file SHA-256
    /// digests. Byte-identical mirrors produce an identical manifest.
    pub asset_manifest_sha256: String,
}

/// Non-fatal placeholder used when an optional embedding implementation is
/// not compiled into the current UAR profile. Chat, tools, AG-UI and A2UI must
/// remain available; an operation that actually requires embeddings receives
/// the original, actionable backend error at that boundary.
#[derive(Debug)]
pub struct UnavailableEmbeddingBackend {
    dimension: usize,
    reason: String,
}

impl UnavailableEmbeddingBackend {
    #[must_use]
    pub fn new(dimension: usize, reason: impl Into<String>) -> Self {
        Self {
            dimension,
            reason: reason.into(),
        }
    }
}

#[async_trait]
impl EmbeddingBackend for UnavailableEmbeddingBackend {
    fn backend_name(&self) -> &str {
        "unavailable"
    }

    fn vector_dimension(&self) -> usize {
        self.dimension
    }

    async fn embed(&self, _texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        Err(EmbeddingError::BackendUnavailable(self.reason.clone()))
    }
}

impl dyn EmbeddingBackend {
    /// The embedding space this backend produces vectors in.
    pub fn fingerprint(&self) -> crate::uar::domain::knowledge::EmbeddingFingerprint {
        crate::uar::domain::knowledge::EmbeddingFingerprint {
            backend: self.backend_name().to_string(),
            model: self.model_id().to_string(),
            dimension: self.vector_dimension(),
        }
    }

    /// Embed a single text and return the first (only) embedding.
    pub async fn embed_one(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let mut batch = self.embed(&[text]).await?;
        batch.pop().ok_or(EmbeddingError::EmptyResponse)
    }
}

/// Errors that can occur when embedding texts.
#[derive(Debug, Error)]
pub enum EmbeddingError {
    #[error("embedding backend is not available: {0}")]
    BackendUnavailable(String),
    #[error("embedding request failed: {0}")]
    RequestFailed(String),
    #[error("dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("provider returned an empty response")]
    EmptyResponse,
    #[error("API key is missing for backend {backend}")]
    MissingApiKey { backend: String },
}

/// Factory that builds the configured backend from `EmbeddingConfig`.
pub fn build_backend(
    config: &EmbeddingConfig,
) -> Result<Arc<dyn EmbeddingBackend>, EmbeddingError> {
    match config.backend.as_str() {
        #[cfg(feature = "local-models")]
        "fastembed" => Ok(Arc::new(fastembed::FastEmbedBackend::new(config)?)),
        #[cfg(feature = "candle-embeddings")]
        "candle" => Ok(Arc::new(candle::CandleEmbeddingBackend::new(config)?)),
        "openai" => Ok(Arc::new(openai::OpenAiEmbeddingBackend::new(config)?)),
        "voyage" => Ok(Arc::new(voyage::VoyageEmbeddingBackend::new(config)?)),
        "cohere" => Ok(Arc::new(cohere::CohereEmbeddingBackend::new(config)?)),
        other => Err(EmbeddingError::BackendUnavailable(format!(
            "unknown or disabled embedding backend: {other}"
        ))),
    }
}

/// Build the local backend for a failover member under the supplied trusted
/// host scope. This deliberately has no hosted-provider branch.
pub fn build_local_backend(
    config: &EmbeddingConfig,
    scope: crate::uar::runtime::lifecycle::ResourceScope,
) -> Result<Arc<dyn EmbeddingBackend>, EmbeddingError> {
    if config.backend != "fastembed" {
        return Err(EmbeddingError::BackendUnavailable(format!(
            "local embedding failover supports only fastembed, not {}",
            config.backend
        )));
    }
    #[cfg(feature = "local-models")]
    {
        Ok(Arc::new(fastembed::FastEmbedBackend::new_with_scope(config, scope)?))
    }
    #[cfg(not(feature = "local-models"))]
    {
        let _ = scope;
        Err(EmbeddingError::BackendUnavailable(
            "local embedding failover requires the local-models build feature".to_string(),
        ))
    }
}

/// Configuration for the embedding backend.
#[derive(Clone, Debug)]
pub struct EmbeddingConfig {
    pub backend: String,
    pub model: String,
    pub api_key: Option<String>,
    pub api_key_env: Option<String>,
    pub base_url: Option<String>,
    pub vector_dimension: usize,
    pub batch_size: usize,
    pub models_dir: String,
    /// Failover members resolve exactly this directory. Legacy single-backend
    /// construction retains its established discovery fallbacks.
    pub strict_models_dir: bool,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        #[cfg(feature = "local-models")]
        let backend = "fastembed".to_string();
        #[cfg(all(not(feature = "local-models"), feature = "candle-embeddings"))]
        let backend = "candle".to_string();
        #[cfg(all(not(feature = "local-models"), not(feature = "candle-embeddings")))]
        let backend = "openai".to_string();

        Self {
            backend,
            model: "bge-small-en-v1.5".to_string(),
            api_key: None,
            api_key_env: None,
            base_url: None,
            vector_dimension: 384,
            batch_size: 32,
            models_dir: "src/uar/runtime/matching/models".to_string(),
            strict_models_dir: false,
        }
    }
}

impl From<&crate::config::EmbeddingBackendConfig> for EmbeddingConfig {
    fn from(c: &crate::config::EmbeddingBackendConfig) -> Self {
        Self {
            backend: c.backend.clone(),
            model: c.model.clone(),
            api_key: c.api_key.clone(),
            api_key_env: c.api_key_env.clone(),
            base_url: c.base_url.clone(),
            vector_dimension: c.vector_dimension,
            batch_size: c.batch_size,
            models_dir: c.models_dir.clone(),
            strict_models_dir: false,
        }
    }
}

impl EmbeddingConfig {
    /// Construct a credential-free, explicitly located local failover member.
    #[must_use]
    pub fn from_local_member(member: &crate::config::LocalEmbeddingMemberConfig) -> Self {
        Self {
            backend: member.backend.clone(),
            model: member.model.clone(),
            api_key: None,
            api_key_env: None,
            base_url: None,
            vector_dimension: member.vector_dimension,
            batch_size: member.batch_size,
            models_dir: member.models_dir.clone(),
            strict_models_dir: true,
        }
    }

    /// Restrict an existing primary configuration to its declared asset path.
    #[must_use]
    pub fn strict_local_member(mut self) -> Self {
        self.strict_models_dir = true;
        self
    }
}

#[cfg(feature = "candle-embeddings")]
pub mod candle;
pub mod cohere;
pub mod failover;
#[cfg(feature = "local-models")]
pub mod fastembed;
pub mod openai;
pub mod swappable;
pub mod voyage;
