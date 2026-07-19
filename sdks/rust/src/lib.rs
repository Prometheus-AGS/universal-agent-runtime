//! Production Rust SDK for Universal Agent Runtime.
//!
//! The default `http-client` feature provides typed asynchronous operations for
//! chat, tools, structured output, embeddings, runs, checkpoints, knowledge
//! bases, documents, search, and ingestion.
//!
//! # Examples
//!
//! ```rust,no_run
//! use universal_agent_runtime_sdk::{ChatCompletionRequest, ChatMessage, Client};
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Client::new("http://localhost:1906")?;
//! let response = client.chat().complete(ChatCompletionRequest {
//!     messages: vec![ChatMessage::text("user", "Hello")],
//!     ..Default::default()
//! }).await?;
//! println!("{}", response.id);
//! # Ok(()) }
//! ```
//!
//! # Errors
//!
//! Network, protocol, decoding, and runtime failures are returned as
//! [`Error`], which implements [`miette::Diagnostic`].
//!
//! # Panics
//!
//! Public SDK operations do not intentionally panic.

pub mod error;
pub mod types;

#[cfg(feature = "http-client")]
pub mod client;
#[cfg(feature = "embedded")]
pub mod runtime;

#[doc(inline)]
pub use error::{Error, UarError};
#[doc(inline)]
pub use types::{
    ArtifactResponseAck, ArtifactResponseRequest, CancelRunResponse, ChatCompletionRequest,
    ChatCompletionResponse, ChatMessage, Checkpoint, CheckpointListResponse,
    CreateKnowledgeBaseRequest, CreateRunRequest, Document, Embedding, EmbeddingRequest,
    EmbeddingResponse, IngestRequest, IngestResponse, KnowledgeBase, KnowledgeBaseConfig,
    ResumeRunRequest, RunResponse, SearchRequest, SearchResponse, SearchResult, StreamEvent,
    ToolCallRequest, ToolCallResponse, UpdateKnowledgeBaseRequest,
};

#[cfg(feature = "http-client")]
#[doc(inline)]
pub use client::{Client, EventStream};
#[cfg(feature = "embedded")]
#[doc(inline)]
pub use runtime::Runtime;
