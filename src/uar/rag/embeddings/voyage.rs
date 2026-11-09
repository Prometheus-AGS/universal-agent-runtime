use crate::uar::rag::embeddings::{
    EmbeddingBackend, EmbeddingBatch, EmbeddingConfig, EmbeddingError, EmbeddingTokenUsage,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://api.voyageai.com/v1";
const DEFAULT_TIMEOUT_SECS: u64 = 60;

/// Voyage AI Embeddings API backend.
#[derive(Debug)]
pub struct VoyageEmbeddingBackend {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
    vector_dimension: usize,
}

impl VoyageEmbeddingBackend {
    pub fn new(config: &EmbeddingConfig) -> Result<Self, EmbeddingError> {
        let api_key = resolve_api_key(config, "voyage")?;
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
        })
    }
}

#[async_trait]
impl EmbeddingBackend for VoyageEmbeddingBackend {
    fn backend_name(&self) -> &str {
        "voyage"
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

        let body = VoyageRequest {
            model: self.model.clone(),
            input: texts.iter().map(|s| s.to_string()).collect(),
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
            return Err(EmbeddingError::RequestFailed(format!(
                "Embedding provider returned HTTP {}",
                status.as_u16()
            )));
        }

        let resp: VoyageResponse = resp.json().await.map_err(|_| {
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
        data.sort_by_key(|item| item.index);
        for (index, item) in data.iter().enumerate() {
            if item.index != index {
                return Err(EmbeddingError::RequestFailed(
                    "Embedding provider returned invalid batch indices".into(),
                ));
            }
            if item.embedding.len() != self.vector_dimension {
                return Err(EmbeddingError::DimensionMismatch {
                    expected: self.vector_dimension,
                    actual: item.embedding.len(),
                });
            }
            if item.embedding.iter().any(|value| !value.is_finite()) {
                return Err(EmbeddingError::RequestFailed(
                    "Embedding provider returned a non-finite vector".into(),
                ));
            }
        }
        Ok(EmbeddingBatch {
            vectors: data.into_iter().map(|item| item.embedding).collect(),
            usage: EmbeddingTokenUsage {
                total_tokens: resp.usage.and_then(|usage| usage.total_tokens),
                ..EmbeddingTokenUsage::default()
            },
        })
    }
}

#[derive(Serialize)]
struct VoyageRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Deserialize)]
struct VoyageResponse {
    data: Vec<VoyageEmbedding>,
    #[serde(default)]
    usage: Option<VoyageUsage>,
}

#[derive(Deserialize)]
struct VoyageUsage {
    total_tokens: Option<u64>,
}

#[derive(Deserialize)]
struct VoyageEmbedding {
    index: usize,
    embedding: Vec<f32>,
}

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
            backend: "voyage".to_string(),
            api_key: None,
            api_key_env: None,
            ..EmbeddingConfig::default()
        };
        let err = VoyageEmbeddingBackend::new(&cfg).expect_err("should fail without key");
        assert!(
            matches!(err, EmbeddingError::MissingApiKey { ref backend } if backend == "voyage"),
            "expected MissingApiKey(voyage), got {err:?}"
        );
    }

    #[test]
    fn vector_dimension_matches_config() {
        let cfg = EmbeddingConfig {
            backend: "voyage".to_string(),
            model: "voyage-3-lite".to_string(),
            api_key: Some("pk-test".to_string()),
            vector_dimension: 512,
            ..EmbeddingConfig::default()
        };
        let backend = VoyageEmbeddingBackend::new(&cfg).expect("build backend");
        assert_eq!(backend.vector_dimension(), 512);
        assert_eq!(backend.backend_name(), "voyage");
    }

    #[tokio::test]
    async fn empty_batch_is_ok() {
        let cfg = EmbeddingConfig {
            backend: "voyage".to_string(),
            api_key: Some("pk-test".to_string()),
            ..EmbeddingConfig::default()
        };
        let backend = VoyageEmbeddingBackend::new(&cfg).expect("build backend");
        let out = backend.embed(&[]).await.expect("embed empty");
        assert!(out.is_empty());
    }
}
