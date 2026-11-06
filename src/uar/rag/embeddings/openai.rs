use crate::uar::rag::embeddings::{
    EmbeddingBackend, EmbeddingBatch, EmbeddingConfig, EmbeddingError, EmbeddingTokenUsage,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
const DEFAULT_TIMEOUT_SECS: u64 = 60;

/// OpenAI Embeddings API backend.
#[derive(Debug)]
pub struct OpenAiEmbeddingBackend {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
    vector_dimension: usize,
    send_dimensions: bool,
    /// Maximum inputs per request. Some OpenAI-compatible providers cap it
    /// (DashScope text-embedding-v4 rejects more than 10).
    batch_size: usize,
}

impl OpenAiEmbeddingBackend {
    pub fn new(config: &EmbeddingConfig) -> Result<Self, EmbeddingError> {
        let api_key = resolve_api_key(config, "openai")?;
        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .map_err(|e| EmbeddingError::BackendUnavailable(format!("reqwest client: {e}")))?;

        Ok(Self {
            client,
            base_url,
            api_key,
            model: config.model.clone(),
            vector_dimension: config.vector_dimension,
            send_dimensions: config.send_dimensions,
            batch_size: config.batch_size.max(1),
        })
    }
}

#[async_trait]
impl EmbeddingBackend for OpenAiEmbeddingBackend {
    fn backend_name(&self) -> &str {
        "openai"
    }

    fn vector_dimension(&self) -> usize {
        self.vector_dimension
    }

    fn model_id(&self) -> &str {
        &self.model
    }

    fn endpoint(&self) -> Option<&str> {
        Some(&self.base_url)
    }

    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        Ok(self.embed_with_usage(texts).await?.vectors)
    }

    async fn embed_with_usage(&self, texts: &[&str]) -> Result<EmbeddingBatch, EmbeddingError> {
        if texts.is_empty() {
            return Ok(EmbeddingBatch {
                vectors: Vec::new(),
                usage: EmbeddingTokenUsage::default(),
            });
        }

        let mut embeddings = Vec::with_capacity(texts.len());
        let mut usage = EmbeddingTokenUsage {
            prompt_tokens: Some(0),
            total_tokens: Some(0),
            ..EmbeddingTokenUsage::default()
        };
        for chunk in texts.chunks(self.batch_size) {
            let batch = self.embed_chunk(chunk).await?;
            embeddings.extend(batch.vectors);
            // A missing report or overflow invalidates only that complete-batch
            // counter. A later chunk cannot restore a partial sum as complete.
            usage.prompt_tokens =
                add_reported_counter(usage.prompt_tokens, batch.usage.prompt_tokens);
            usage.total_tokens = add_reported_counter(usage.total_tokens, batch.usage.total_tokens);
        }
        Ok(EmbeddingBatch {
            vectors: embeddings,
            usage,
        })
    }
}

impl OpenAiEmbeddingBackend {
    /// One `/embeddings` request for at most `batch_size` inputs, returned in
    /// input order.
    async fn embed_chunk(&self, texts: &[&str]) -> Result<EmbeddingBatch, EmbeddingError> {
        let body = OpenAiRequest {
            model: self.model.clone(),
            input: texts.iter().map(|s| s.to_string()).collect(),
            dimensions: if self.send_dimensions && self.vector_dimension != 1536 {
                Some(self.vector_dimension)
            } else {
                None
            },
        };

        let resp = self
            .client
            .post(format!("{}/embeddings", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|_| {
                EmbeddingError::RequestFailed("Embedding provider transport failed".into())
            })?;

        let status = resp.status();
        if !status.is_success() {
            // Provider bodies and request URLs may contain credentials or input.
            // Keep only the numeric status across this trust boundary.
            return Err(EmbeddingError::RequestFailed(format!(
                "Embedding provider returned HTTP {}",
                status.as_u16()
            )));
        }

        let resp: OpenAiResponse = resp.json().await.map_err(|_| {
            EmbeddingError::RequestFailed(
                "Embedding provider returned invalid JSON or fields".into(),
            )
        })?;

        let mut data = resp.data;
        if data.len() != texts.len() {
            return Err(EmbeddingError::RequestFailed(
                "Embedding provider returned an incomplete batch".into(),
            ));
        }
        data.sort_by_key(|d| d.index);
        for (expected_index, embedding) in data.iter().enumerate() {
            if embedding.index != expected_index {
                return Err(EmbeddingError::RequestFailed(
                    "Embedding provider returned invalid batch indices".into(),
                ));
            }
            if embedding.embedding.len() != self.vector_dimension {
                return Err(EmbeddingError::DimensionMismatch {
                    expected: self.vector_dimension,
                    actual: embedding.embedding.len(),
                });
            }
            if embedding.embedding.iter().any(|value| !value.is_finite()) {
                return Err(EmbeddingError::RequestFailed(
                    "Embedding provider returned a non-finite vector".into(),
                ));
            }
        }
        Ok(EmbeddingBatch {
            vectors: data.into_iter().map(|d| d.embedding).collect(),
            usage: resp.usage.unwrap_or_default(),
        })
    }
}

fn add_reported_counter(current: Option<u64>, next: Option<u64>) -> Option<u64> {
    current
        .zip(next)
        .and_then(|(current, next)| current.checked_add(next))
}

#[derive(Serialize)]
struct OpenAiRequest {
    model: String,
    input: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dimensions: Option<usize>,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    data: Vec<OpenAiEmbedding>,
    #[serde(default)]
    usage: Option<EmbeddingTokenUsage>,
}

#[derive(Deserialize)]
struct OpenAiEmbedding {
    index: usize,
    embedding: Vec<f32>,
}

/// Resolve the API key from config or the named environment variable.
fn resolve_api_key(config: &EmbeddingConfig, backend: &str) -> Result<String, EmbeddingError> {
    if let Some(key) = config.api_key.clone() {
        return Ok(key);
    }
    if let Some(env_name) = config.api_key_env.as_deref() {
        if let Ok(key) = std::env::var(env_name) {
            return Ok(key);
        }
    }
    Err(EmbeddingError::MissingApiKey {
        backend: backend.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_key_returns_error() {
        let cfg = EmbeddingConfig {
            backend: "openai".to_string(),
            api_key: None,
            api_key_env: None,
            ..EmbeddingConfig::default()
        };
        let err = OpenAiEmbeddingBackend::new(&cfg).expect_err("should fail without key");
        assert!(
            matches!(err, EmbeddingError::MissingApiKey { ref backend } if backend == "openai"),
            "expected MissingApiKey(openai), got {err:?}"
        );
    }

    #[test]
    fn vector_dimension_matches_config() {
        let cfg = EmbeddingConfig {
            backend: "openai".to_string(),
            model: "text-embedding-3-small".to_string(),
            api_key: Some("sk-test".to_string()),
            vector_dimension: 1536,
            ..EmbeddingConfig::default()
        };
        let backend = OpenAiEmbeddingBackend::new(&cfg).expect("build backend");
        assert_eq!(backend.vector_dimension(), 1536);
        assert_eq!(backend.backend_name(), "openai");
    }

    #[tokio::test]
    async fn empty_batch_is_ok() {
        let cfg = EmbeddingConfig {
            backend: "openai".to_string(),
            api_key: Some("sk-test".to_string()),
            ..EmbeddingConfig::default()
        };
        let backend = OpenAiEmbeddingBackend::new(&cfg).expect("build backend");
        let out = backend.embed(&[]).await.expect("embed empty");
        assert!(out.is_empty());
    }
}
