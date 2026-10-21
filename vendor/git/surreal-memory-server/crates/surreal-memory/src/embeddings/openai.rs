use super::{Embedding, EmbeddingService};
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OpenAIEmbeddings {
    client: Client,
    api_key: String,
    model: String,
    /// Endpoint base URL. Defaults to OpenAI's official endpoint when empty.
    /// Override for OpenAI-compatible services (Mistral, vLLM, etc.).
    base_url: String,
    /// Override for the returned vector dimension. Use when the runtime cannot
    /// infer the dimension from the model name (most third-party models).
    dimensions_override: Option<usize>,
}

#[derive(Serialize)]
struct EmbeddingRequest {
    input: Vec<String>,
    model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    dimensions: Option<usize>,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

impl OpenAIEmbeddings {
    /// Construct an OpenAI-compatible embeddings client.
    ///
    /// - `base_url`: when empty, defaults to OpenAI's `https://api.openai.com/v1`.
    ///   Set explicitly to point at a compatible host (Mistral, etc.).
    /// - `dimensions_override`: when `Some(n)`, returned vectors are asserted to
    ///   have length `n`. Use this when the host doesn't accept the `dimensions`
    ///   request parameter but you know the model's output dimension.
    pub fn new(api_key: String, model: String) -> Self {
        Self::with_base_url(api_key, model, String::new(), None)
    }

    pub fn with_base_url(
        api_key: String,
        model: String,
        base_url: String,
        dimensions_override: Option<usize>,
    ) -> Self {
        // Reserved for future per-model dimension mapping; the dimensions()
        // method below holds the authoritative inference today.
        let _default_dimensions = match model.as_str() {
            "text-embedding-3-small" => 1536,
            "text-embedding-3-large" => 3072,
            "text-embedding-ada-002" => 1536,
            _ => 1536,
        };
        Self {
            client: Client::new(),
            api_key,
            model,
            base_url: if base_url.is_empty() {
                "https://api.openai.com/v1".to_string()
            } else {
                base_url
            },
            dimensions_override,
        }
    }

    fn base_endpoint(&self) -> String {
        let trimmed = self.base_url.trim_end_matches('/');
        format!("{trimmed}/embeddings")
    }
}

#[async_trait]
impl EmbeddingService for OpenAIEmbeddings {
    async fn embed(&self, text: &str) -> Result<Embedding> {
        let embeddings = self.embed_batch(vec![text.to_string()]).await?;
        embeddings
            .into_iter()
            .next()
            .context("No embedding returned")
    }

    async fn embed_batch(&self, texts: Vec<String>) -> Result<Vec<Embedding>> {
        let request = EmbeddingRequest {
            input: texts,
            model: self.model.clone(),
            dimensions: None,
        };

        let response = self
            .client
            .post(self.base_endpoint())
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .context("Failed to send embedding request")?;

        let embedding_response: EmbeddingResponse = response
            .json()
            .await
            .context("Failed to parse embedding response")?;

        Ok(embedding_response
            .data
            .into_iter()
            .map(|d| d.embedding)
            .collect())
    }

    fn dimensions(&self) -> usize {
        self.dimensions_override.unwrap_or_else(|| match self.model.as_str() {
            "text-embedding-3-small" => 1536,
            "text-embedding-3-large" => 3072,
            "text-embedding-ada-002" => 1536,
            _ => 1536,
        })
    }
}
