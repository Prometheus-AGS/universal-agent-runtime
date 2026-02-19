//! A2A Protocol RC v1.0 — type definitions.
//!
//! These types implement the Google Agent-to-Agent (A2A) protocol specification.
//! Reference: <https://a2a-protocol.org/latest/specification/>
//!
//! Key concepts:
//! - **Task** — the unit of work, maps 1:1 to a [`CompilerSession`]
//! - **Message** — a conversational turn (user or agent)
//! - **Part** — typed content within a message (text, file, data)
//! - **Artifact** — output produced by the agent (compiled descriptor)
//! - **AgentCard** — machine-readable agent capability declaration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────────────────────
// JSON-RPC 2.0 envelope
// ─────────────────────────────────────────────────────────────────────────────

/// A JSON-RPC 2.0 request.
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

/// A JSON-RPC 2.0 response.
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    pub fn ok(id: Option<serde_json::Value>, result: impl Serialize) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: Some(serde_json::to_value(result).unwrap_or(serde_json::Value::Null)),
            error: None,
        }
    }

    pub fn err(id: Option<serde_json::Value>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

/// A JSON-RPC 2.0 error object.
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Standard JSON-RPC error codes.
pub mod rpc_error {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
    /// Task not found (A2A extension).
    pub const TASK_NOT_FOUND: i32 = -32001;
    /// Task cannot be cancelled (A2A extension).
    pub const TASK_NOT_CANCELABLE: i32 = -32002;
}

// ─────────────────────────────────────────────────────────────────────────────
// A2A Task
// ─────────────────────────────────────────────────────────────────────────────

/// Task status per A2A RC v1.0 §4.2.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskState {
    /// Task is queued but not yet started.
    Submitted,
    /// Task is actively being worked on.
    Working,
    /// Task requires additional input from the user.
    InputRequired,
    /// Task completed successfully.
    Completed,
    /// Task was cancelled.
    Canceled,
    /// Task failed.
    Failed,
}

/// A2A Task — the primary unit of work.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique task identifier.
    pub id: String,
    /// Optional context ID for multi-turn sessions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_id: Option<String>,
    /// Current task state.
    pub status: TaskStatus,
    /// Conversation history (messages from both sides).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub history: Vec<Message>,
    /// Artifacts produced by the agent.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<Artifact>,
    /// Arbitrary metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Task status with optional message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatus {
    pub state: TaskState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Message + Part
// ─────────────────────────────────────────────────────────────────────────────

/// Message role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Agent,
}

/// A conversational message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub parts: Vec<Part>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Message {
    pub fn user_text(text: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            parts: vec![Part::text(text)],
            metadata: HashMap::new(),
        }
    }

    pub fn agent_text(text: impl Into<String>) -> Self {
        Self {
            role: Role::Agent,
            parts: vec![Part::text(text)],
            metadata: HashMap::new(),
        }
    }
}

/// A typed content part within a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Part {
    /// Plain text content.
    Text { text: String },
    /// File content (inline or by URI).
    File {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        uri: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        bytes: Option<String>, // base64-encoded
    },
    /// Structured data.
    Data { data: serde_json::Value },
}

impl Part {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Artifact
// ─────────────────────────────────────────────────────────────────────────────

/// An artifact produced by the agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub artifact_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub parts: Vec<Part>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

// ─────────────────────────────────────────────────────────────────────────────
// AgentCard
// ─────────────────────────────────────────────────────────────────────────────

/// AgentCard — machine-readable capability declaration per A2A RC v1.0 §3.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCard {
    pub name: String,
    pub description: String,
    pub url: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_url: Option<String>,
    pub capabilities: AgentCapabilities,
    pub skills: Vec<AgentSkill>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub security_schemes: Vec<SecurityScheme>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub default_input_modes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub default_output_modes: Vec<String>,
}

/// Agent capabilities declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    pub streaming: bool,
    pub push_notifications: bool,
    pub state_transition_history: bool,
}

/// A skill declared in the AgentCard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub input_modes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub output_modes: Vec<String>,
}

/// Security scheme declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScheme {
    #[serde(rename = "type")]
    pub scheme_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "in", skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Method params
// ─────────────────────────────────────────────────────────────────────────────

/// Params for `message/send`.
#[derive(Debug, Deserialize)]
pub struct MessageSendParams {
    pub message: Message,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_id: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Params for `tasks/get`.
#[derive(Debug, Deserialize)]
pub struct TaskGetParams {
    pub id: String,
}

/// Params for `tasks/cancel`.
#[derive(Debug, Deserialize)]
pub struct TaskCancelParams {
    pub id: String,
}
