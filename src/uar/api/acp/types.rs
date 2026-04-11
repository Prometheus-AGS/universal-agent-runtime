//! ACP (Agent Communication Protocol) type definitions.
//!
//! JSON-RPC 2.0 envelope + BeeAI ACP domain types for agent introspection,
//! session management, and streaming run execution.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

// =============================================================================
// JSON-RPC 2.0 envelope
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcResponse {
    pub fn ok(id: Option<Value>, result: Value) -> Self {
        Self { jsonrpc: "2.0".into(), id, result: Some(result), error: None }
    }
    pub fn err(id: Option<Value>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(JsonRpcError { code, message: message.into(), data: None }),
        }
    }
}

// Standard JSON-RPC error codes
pub const RPC_PARSE_ERROR: i32 = -32700;
pub const RPC_INVALID_REQUEST: i32 = -32600;
pub const RPC_METHOD_NOT_FOUND: i32 = -32601;
pub const RPC_INVALID_PARAMS: i32 = -32602;
pub const RPC_INTERNAL_ERROR: i32 = -32603;
// ACP application error codes (-32000 .. -32099)
pub const ACP_SESSION_NOT_FOUND: i32 = -32001;
pub const ACP_RUN_NOT_FOUND: i32 = -32002;
pub const ACP_AGENT_NOT_FOUND: i32 = -32003;

// =============================================================================
// ACP domain types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpAgentCard {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub capabilities: AcpCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AcpCapabilities {
    pub streaming: bool,
    pub tool_use: bool,
    pub memory: bool,
    pub multi_turn: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpSession {
    pub session_id: String,
    pub agent_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_active: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub metadata: Value,
}

impl AcpSession {
    pub fn new(agent_id: impl Into<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            session_id: Uuid::new_v4().to_string(),
            agent_id: agent_id.into(),
            created_at: now,
            last_active: now,
            metadata: Value::Object(Default::default()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpRunRequest {
    pub session_id: String,
    pub input: String,
    #[serde(default)]
    pub stream: bool,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case", tag = "event")]
pub enum AcpRunEvent {
    RunStart { run_id: String },
    Delta { run_id: String, text: String },
    ToolStart { run_id: String, tool: String, input: Value },
    ToolEnd { run_id: String, tool: String, output: Value, ok: bool },
    RunDone { run_id: String, output: String },
    Error { run_id: String, message: String },
}
