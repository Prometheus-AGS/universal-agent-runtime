//! Anthropic-compatible embeddings adapter.
//!
//! Anthropic's official embeddings endpoint (`/v1/embeddings`) uses an
//! OpenAI-compatible request shape with the `Authorization: Bearer` header.
//! As of the API pin in this crate, Anthropic ships the `voyage-*` family via
//! this endpoint. Operators set `embedding_base_url` to override the default
//! (`https://api.anthropic.com/v1`) for self-hosted or proxied deployments.
//!
//! This adapter uses the same request shape as `openai.rs` but with an
//! explicit adapter marker so the runtime can route configuration correctly.
//! The runtime's `embedding_api: "anthropic"` discriminator selects this adapter.

use super::openai::OpenAIEmbeddings;
use super::{Embedding, EmbeddingService};
use anyhow::Result;
use async_trait::async_trait;

pub struct AnthropicEmbeddings {
    inner: OpenAIEmbeddings,
}

impl AnthropicEmbeddings {
    pub fn new(api_key: String, model: String) -> Self {
        Self::with_base_url(api_key, model, String::new(), None)
    }

    pub fn with_base_url(
        api_key: String,
        model: String,
        base_url: String,
        dimensions_override: Option<usize>,
    ) -> Self {
        let effective_base_url = if base_url.is_empty() {
            "https://api.anthropic.com/v1".to_string()
        } else {
            base_url
        };
        Self {
            inner: OpenAIEmbeddings::with_base_url(
                api_key,
                model,
                effective_base_url,
                dimensions_override,
            ),
        }
    }
}

#[async_trait]
impl EmbeddingService for AnthropicEmbeddings {
    async fn embed(&self, text: &str) -> Result<Embedding> {
        self.inner.embed(text).await
    }

    async fn embed_batch(&self, texts: Vec<String>) -> Result<Vec<Embedding>> {
        self.inner.embed_batch(texts).await
    }

    fn dimensions(&self) -> usize {
        self.inner.dimensions()
    }
}