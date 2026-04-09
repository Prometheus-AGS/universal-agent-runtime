//! Anthropic Messages API wire protocol types.
//!
//! Serde types for the Anthropic Messages API request/response format,
//! including streaming SSE event types and cache control annotations.

use serde::{Deserialize, Serialize};

// === Request Types ===

/// Top-level request body for the Anthropic Messages API.
#[derive(Debug, Clone, Serialize)]
pub struct MessagesRequest {
    /// Model identifier (e.g. `claude-sonnet-4-20250514`).
    pub model: String,
    /// Maximum number of tokens to generate.
    pub max_tokens: u32,
    /// Optional system prompt blocks (supports cache control).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<Vec<SystemBlock>>,
    /// Conversation messages.
    pub messages: Vec<AnthropicMessage>,
    /// Tool definitions available to the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDef>>,
    /// How the model should choose tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    /// Whether to stream the response via SSE.
    pub stream: bool,
    /// Extended thinking configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,
}

/// A system prompt block with optional cache control.
#[derive(Debug, Clone, Serialize)]
pub struct SystemBlock {
    /// Block type — always `"text"`.
    #[serde(rename = "type")]
    pub block_type: String,
    /// The system prompt text.
    pub text: String,
    /// Optional cache control annotation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// Cache control annotation for prompt caching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheControl {
    /// Control type — currently only `"ephemeral"` is supported.
    #[serde(rename = "type")]
    pub control_type: String,
}

impl CacheControl {
    /// Create an ephemeral cache control annotation.
    #[must_use]
    pub fn ephemeral() -> Self {
        Self {
            control_type: "ephemeral".to_string(),
        }
    }
}

/// A message in the Anthropic conversation format.
#[derive(Debug, Clone, Serialize)]
pub struct AnthropicMessage {
    /// Role — `"user"` or `"assistant"`.
    pub role: String,
    /// Content — either a plain string or an array of content blocks.
    pub content: serde_json::Value,
}

/// A tool definition for the Anthropic Messages API.
#[derive(Debug, Clone, Serialize)]
pub struct ToolDef {
    /// Tool name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// JSON Schema for tool input.
    pub input_schema: serde_json::Value,
    /// Optional cache control annotation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// Tool choice strategy.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ToolChoice {
    /// Let the model decide whether to use tools.
    Auto,
    /// Force the model to use at least one tool.
    Any,
    /// Force the model to use a specific tool.
    Tool {
        /// Name of the required tool.
        name: String,
    },
}

/// Configuration for extended thinking mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingConfig {
    /// Thinking type — `"enabled"` to turn on extended thinking.
    #[serde(rename = "type")]
    pub thinking_type: String,
    /// Token budget for thinking.
    pub budget_tokens: u32,
}

// === Response Types ===

/// Non-streaming response from the Anthropic Messages API.
#[derive(Debug, Clone, Deserialize)]
pub struct MessagesResponse {
    /// Unique message identifier.
    pub id: String,
    /// Object type — always `"message"`.
    #[serde(rename = "type")]
    pub response_type: String,
    /// Role — always `"assistant"`.
    pub role: String,
    /// Model that generated the response.
    pub model: String,
    /// Content blocks in the response.
    pub content: Vec<ContentBlockResponse>,
    /// Reason the model stopped generating.
    pub stop_reason: Option<String>,
    /// Stop sequence that triggered completion, if any.
    pub stop_sequence: Option<String>,
    /// Token usage information.
    pub usage: UsageResponse,
}

/// A content block in a non-streaming response.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlockResponse {
    /// Text content block.
    #[serde(rename = "text")]
    Text {
        /// The generated text.
        text: String,
    },
    /// Tool use content block.
    #[serde(rename = "tool_use")]
    ToolUse {
        /// Tool call identifier.
        id: String,
        /// Tool name.
        name: String,
        /// Tool input as JSON.
        input: serde_json::Value,
    },
    /// Thinking content block (extended thinking).
    #[serde(rename = "thinking")]
    Thinking {
        /// The thinking text.
        thinking: String,
        /// Cryptographic signature for the thinking block.
        signature: String,
    },
}

/// Token usage information.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UsageResponse {
    /// Number of input tokens.
    pub input_tokens: u32,
    /// Number of output tokens.
    pub output_tokens: u32,
    /// Tokens written into the prompt cache.
    #[serde(default)]
    pub cache_creation_input_tokens: Option<u32>,
    /// Tokens read from the prompt cache.
    #[serde(default)]
    pub cache_read_input_tokens: Option<u32>,
}

// === Streaming SSE Event Types ===

/// A streaming SSE event from the Anthropic Messages API.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum StreamEvent {
    /// Stream has started; contains initial message metadata.
    #[serde(rename = "message_start")]
    MessageStart {
        /// Initial message data.
        message: MessageStartData,
    },
    /// A new content block has started.
    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        /// Zero-based index of this content block.
        index: usize,
        /// Initial content block data.
        content_block: ContentBlockStartData,
    },
    /// Incremental delta for a content block.
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta {
        /// Zero-based index of the content block being updated.
        index: usize,
        /// The delta payload.
        delta: ContentBlockDeltaData,
    },
    /// A content block has finished.
    #[serde(rename = "content_block_stop")]
    ContentBlockStop {
        /// Zero-based index of the completed content block.
        index: usize,
    },
    /// Message-level delta (stop reason, usage updates).
    #[serde(rename = "message_delta")]
    MessageDelta {
        /// Delta data.
        delta: MessageDeltaData,
        /// Updated usage information.
        usage: Option<UsageResponse>,
    },
    /// The message has completed.
    #[serde(rename = "message_stop")]
    MessageStop,
    /// An error occurred during streaming.
    #[serde(rename = "error")]
    Error {
        /// Error details.
        error: ErrorData,
    },
    /// Keep-alive ping.
    #[serde(rename = "ping")]
    Ping,
}

/// Initial message metadata from `message_start`.
#[derive(Debug, Clone, Deserialize)]
pub struct MessageStartData {
    /// Unique message identifier.
    pub id: String,
    /// Model that is generating the response.
    pub model: String,
    /// Role — always `"assistant"`.
    pub role: String,
    /// Initial usage information (input tokens).
    pub usage: Option<UsageResponse>,
}

/// Content block start data, tagged by type.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlockStartData {
    /// Text block starting.
    #[serde(rename = "text")]
    Text {
        /// Initial text (usually empty).
        text: String,
    },
    /// Tool use block starting.
    #[serde(rename = "tool_use")]
    ToolUse {
        /// Tool call identifier.
        id: String,
        /// Tool name.
        name: String,
    },
    /// Thinking block starting.
    #[serde(rename = "thinking")]
    Thinking {
        /// Initial thinking text (usually empty).
        thinking: String,
    },
}

/// Content block delta data, tagged by type.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlockDeltaData {
    /// Incremental text delta.
    #[serde(rename = "text_delta")]
    TextDelta {
        /// Text fragment to append.
        text: String,
    },
    /// Incremental JSON delta for tool input.
    #[serde(rename = "input_json_delta")]
    InputJsonDelta {
        /// Partial JSON string to append.
        partial_json: String,
    },
    /// Incremental thinking delta.
    #[serde(rename = "thinking_delta")]
    ThinkingDelta {
        /// Thinking text fragment to append.
        thinking: String,
    },
}

/// Message-level delta data.
#[derive(Debug, Clone, Deserialize)]
pub struct MessageDeltaData {
    /// Reason the model stopped generating.
    pub stop_reason: Option<String>,
    /// Stop sequence that triggered completion, if any.
    pub stop_sequence: Option<String>,
}

/// Error data from the Anthropic API.
#[derive(Debug, Clone, Deserialize)]
pub struct ErrorData {
    /// Error type (e.g. `"overloaded_error"`).
    #[serde(rename = "type")]
    pub error_type: String,
    /// Human-readable error message.
    pub message: String,
}
