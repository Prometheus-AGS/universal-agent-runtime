//! Mistral embeddings adapter.
//!
//! Mistral's `https://api.mistral.ai/v1/embeddings` endpoint uses the OpenAI
//! request shape. Default base URL is Mistral's standard endpoint; the operator
//! can override via `embedding_base_url`.
//!
//! The runtime's `embedding_api: "mistral"` discriminator selects this adapter.
//! Mistral's `mistral-embed` returns 1024-dim vectors.

use super::openai::OpenAIEmbeddings;
use super::{Embedding, EmbeddingService};
use anyhow::Result;
use async_trait::async_trait;

pub struct MistralEmbeddings {
    inner: OpenAIEmbeddings,
}

impl MistralEmbeddings {
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
            "https://api.mistral.ai/v1".to_string()
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
impl EmbeddingService for MistralEmbeddings {
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