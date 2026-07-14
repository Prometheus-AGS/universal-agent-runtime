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

    /// Embed a batch of texts. The returned outer vector has the same length as
    /// `texts` (unless `texts` is empty, in which case an empty vector is
    /// returned). Each inner vector has `vector_dimension()` elements.
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError>;
}

impl dyn EmbeddingBackend {
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
        }
    }
}

#[cfg(feature = "candle-embeddings")]
pub mod candle;
pub mod cohere;
#[cfg(feature = "local-models")]
pub mod fastembed;
pub mod openai;
pub mod voyage;
