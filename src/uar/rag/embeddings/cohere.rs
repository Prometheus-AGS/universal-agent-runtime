use crate::uar::rag::embeddings::{EmbeddingBackend, EmbeddingConfig, EmbeddingError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://api.cohere.com/v2";
const DEFAULT_TIMEOUT_SECS: u64 = 60;

/// Cohere Embed API backend.
#[derive(Debug)]
pub struct CohereEmbeddingBackend {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
    vector_dimension: usize,
}

impl CohereEmbeddingBackend {
    pub fn new(config: &EmbeddingConfig) -> Result<Self, EmbeddingError> {
        let api_key = resolve_api_key(config, "cohere")?;
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
impl EmbeddingBackend for CohereEmbeddingBackend {
    fn backend_name(&self) -> &str {
        "cohere"
    }

    fn vector_dimension(&self) -> usize {
        self.vector_dimension
    }

    async fn embed(
        &self,
        texts: &[&str],
    ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let body = CohereRequest {
            model: self.model.clone(),
            texts: texts.iter().map(|s| s.to_string()).collect(),
            input_type: "search_document".to_string(),
            embedding_types: vec!["float".to_string()],
        };

        let resp = self
            .client
            .post(format!("{}/embed", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| EmbeddingError::RequestFailed(format!("HTTP request failed: {e}")))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp
                .text()
                .await
                .unwrap_or_else(|_| "<unreadable>".to_string());
            return Err(EmbeddingError::RequestFailed(format!(
                "Cohere returned {status}: {text}"
            )));
        }

        let resp: CohereResponse = resp
            .json()
            .await
            .map_err(|e| EmbeddingError::RequestFailed(format!("failed to parse response: {e}")))?;

        Ok(resp.embeddings.float.unwrap_or_default())
    }
}

#[derive(Serialize)]
struct CohereRequest {
    model: String,
    texts: Vec<String>,
    #[serde(rename = "input_type")]
    input_type: String,
    #[serde(rename = "embedding_types")]
    embedding_types: Vec<String>,
}

#[derive(Deserialize)]
struct CohereResponse {
    embeddings: CohereEmbeddings,
}

#[derive(Deserialize)]
struct CohereEmbeddings {
    float: Option<Vec<Vec<f32>>>,
}

fn resolve_api_key(
    config: &EmbeddingConfig,
    backend: &str,
) -> Result<String, EmbeddingError> {
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
            backend: "cohere".to_string(),
            api_key: None,
            api_key_env: None,
            ..EmbeddingConfig::default()
        };
        let err = CohereEmbeddingBackend::new(&cfg).expect_err("should fail without key");
        assert!(
            matches!(err, EmbeddingError::MissingApiKey { ref backend } if backend == "cohere"),
            "expected MissingApiKey(cohere), got {err:?}"
        );
    }

    #[test]
    fn vector_dimension_matches_config() {
        let cfg = EmbeddingConfig {
            backend: "cohere".to_string(),
            model: "embed-english-v3".to_string(),
            api_key: Some("test-key".to_string()),
            vector_dimension: 1024,
            ..EmbeddingConfig::default()
        };
        let backend = CohereEmbeddingBackend::new(&cfg).expect("build backend");
        assert_eq!(backend.vector_dimension(), 1024);
        assert_eq!(backend.backend_name(), "cohere");
    }

    #[tokio::test]
    async fn empty_batch_is_ok() {
        let cfg = EmbeddingConfig {
            backend: "cohere".to_string(),
            api_key: Some("test-key".to_string()),
            ..EmbeddingConfig::default()
        };
        let backend = CohereEmbeddingBackend::new(&cfg).expect("build backend");
        let out = backend.embed(&[]).await.expect("embed empty");
        assert!(out.is_empty());
    }
}
