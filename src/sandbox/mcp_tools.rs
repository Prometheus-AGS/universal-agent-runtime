//! MCP tool definitions for sandbox code execution.
//!
//! Exposes sandbox execution capabilities as [`NativeTool`] implementations
//! that agents can invoke for running code in isolated environments.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;

use crate::mcp::registry::NativeTool;
use super::session_manager::SessionManager;
use super::types::{ExecutionMode, ExecutionRequest, Language, SandboxConfig};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn default_session_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn parse_language(s: &str) -> anyhow::Result<Language> {
    match s.to_lowercase().as_str() {
        "bash" | "sh" => Ok(Language::Bash),
        "python" | "py" => Ok(Language::Python),
        "rust" | "rs" => Ok(Language::Rust),
        "node" | "nodejs" | "javascript" | "js" => Ok(Language::Node),
        other => Err(anyhow::anyhow!("unsupported language: {other}")),
    }
}

// ---------------------------------------------------------------------------
// SandboxCodeExecTool
// ---------------------------------------------------------------------------

/// Execute code in a sandboxed environment.
#[derive(Debug)]
pub struct SandboxCodeExecTool {
    manager: Arc<SessionManager>,
    default_config: SandboxConfig,
}

impl SandboxCodeExecTool {
    #[must_use]
    pub fn new(manager: Arc<SessionManager>, default_config: SandboxConfig) -> Self {
        Self { manager, default_config }
    }
}

#[async_trait]
impl NativeTool for SandboxCodeExecTool {
    fn name(&self) -> &'static str {
        "code_exec"
    }

    fn description(&self) -> &'static str {
        "Execute code in a sandboxed environment"
    }

    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "language": {
                    "type": "string",
                    "enum": ["bash", "python", "rust", "node"],
                    "description": "Programming language to execute"
                },
                "code": {
                    "type": "string",
                    "description": "Source code to execute"
                },
                "session_id": {
                    "type": "string",
                    "description": "Optional session ID for persistent sandbox state"
                },
                "timeout_seconds": {
                    "type": "number",
                    "description": "Execution timeout in seconds (default: sandbox default)"
                }
            },
            "required": ["language", "code"]
        })
    }

    async fn call(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let language_str = args
            .get("language")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("missing required parameter: language"))?;
        let code = args
            .get("code")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("missing required parameter: code"))?;
        let session_id = args
            .get("session_id")
            .and_then(serde_json::Value::as_str)
            .map(ToString::to_string)
            .unwrap_or_else(default_session_id);
        let timeout_seconds = args
            .get("timeout_seconds")
            .and_then(serde_json::Value::as_u64);

        let language = parse_language(language_str)?;

        let mut config = self.default_config.clone();
        if let Some(t) = timeout_seconds {
            config.timeout_secs = t;
        }

        let handle = self.manager.get_or_create(&session_id, config).await
            .map_err(|e| anyhow::anyhow!("sandbox creation failed: {e}"))?;

        let request = ExecutionRequest {
            language,
            code: code.to_string(),
            stdin: None,
            env: HashMap::new(),
            cwd: None,
            timeout_seconds,
            mode: ExecutionMode::Session { session_id: session_id.clone() },
        };

        let result = self.manager.runner().execute(&handle, request).await
            .map_err(|e| anyhow::anyhow!("sandbox execution failed: {e}"))?;

        Ok(json!({
            "exit_code": result.exit_code,
            "stdout": result.stdout,
            "stderr": result.stderr,
            "execution_time_ms": result.execution_time_ms,
            "session_id": session_id
        }))
    }
}

// ---------------------------------------------------------------------------
// SandboxShellExecTool
// ---------------------------------------------------------------------------

/// Execute a shell command in a sandboxed environment.
#[derive(Debug)]
pub struct SandboxShellExecTool {
    manager: Arc<SessionManager>,
    default_config: SandboxConfig,
}

impl SandboxShellExecTool {
    #[must_use]
    pub fn new(manager: Arc<SessionManager>, default_config: SandboxConfig) -> Self {
        Self { manager, default_config }
    }
}

#[async_trait]
impl NativeTool for SandboxShellExecTool {
    fn name(&self) -> &'static str {
        "shell_exec"
    }

    fn description(&self) -> &'static str {
        "Execute a shell command in a sandboxed environment"
    }

    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Shell command to execute"
                },
                "session_id": {
                    "type": "string",
                    "description": "Optional session ID for persistent sandbox state"
                },
                "timeout_seconds": {
                    "type": "number",
                    "description": "Execution timeout in seconds (default: sandbox default)"
                }
            },
            "required": ["command"]
        })
    }

    async fn call(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let command = args
            .get("command")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("missing required parameter: command"))?;
        let session_id = args
            .get("session_id")
            .and_then(serde_json::Value::as_str)
            .map(ToString::to_string)
            .unwrap_or_else(default_session_id);
        let timeout_seconds = args
            .get("timeout_seconds")
            .and_then(serde_json::Value::as_u64);

        let mut config = self.default_config.clone();
        if let Some(t) = timeout_seconds {
            config.timeout_secs = t;
        }

        let handle = self.manager.get_or_create(&session_id, config).await
            .map_err(|e| anyhow::anyhow!("sandbox creation failed: {e}"))?;

        let request = ExecutionRequest {
            language: Language::Bash,
            code: command.to_string(),
            stdin: None,
            env: HashMap::new(),
            cwd: None,
            timeout_seconds,
            mode: ExecutionMode::Session { session_id: session_id.clone() },
        };

        let result = self.manager.runner().execute(&handle, request).await
            .map_err(|e| anyhow::anyhow!("sandbox execution failed: {e}"))?;

        Ok(json!({
            "exit_code": result.exit_code,
            "stdout": result.stdout,
            "stderr": result.stderr,
            "execution_time_ms": result.execution_time_ms,
            "session_id": session_id
        }))
    }
}

// ---------------------------------------------------------------------------
// SandboxFileReadTool
// ---------------------------------------------------------------------------

/// Read a file from a sandbox workspace.
#[derive(Debug)]
pub struct SandboxFileReadTool {
    manager: Arc<SessionManager>,
    default_config: SandboxConfig,
}

impl SandboxFileReadTool {
    #[must_use]
    pub fn new(manager: Arc<SessionManager>, default_config: SandboxConfig) -> Self {
        Self { manager, default_config }
    }
}

#[async_trait]
impl NativeTool for SandboxFileReadTool {
    fn name(&self) -> &'static str {
        "file_read"
    }

    fn description(&self) -> &'static str {
        "Read a file from a sandbox workspace"
    }

    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "session_id": {
                    "type": "string",
                    "description": "Session ID of the sandbox to read from"
                },
                "path": {
                    "type": "string",
                    "description": "Path of the file to read within the sandbox"
                }
            },
            "required": ["session_id", "path"]
        })
    }

    async fn call(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let session_id = args
            .get("session_id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("missing required parameter: session_id"))?;
        let path = args
            .get("path")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("missing required parameter: path"))?;

        let handle = self.manager.get_or_create(session_id, self.default_config.clone()).await
            .map_err(|e| anyhow::anyhow!("sandbox creation failed: {e}"))?;

        let content = self.manager.runner().read_file(&handle, path).await
            .map_err(|e| anyhow::anyhow!("file read failed: {e}"))?;

        let text = String::from_utf8_lossy(&content).to_string();

        Ok(json!({
            "content": text,
            "path": path,
            "session_id": session_id,
            "size_bytes": content.len()
        }))
    }
}

// ---------------------------------------------------------------------------
// SandboxFileWriteTool
// ---------------------------------------------------------------------------

/// Write a file to a sandbox workspace.
#[derive(Debug)]
pub struct SandboxFileWriteTool {
    manager: Arc<SessionManager>,
    default_config: SandboxConfig,
}

impl SandboxFileWriteTool {
    #[must_use]
    pub fn new(manager: Arc<SessionManager>, default_config: SandboxConfig) -> Self {
        Self { manager, default_config }
    }
}

#[async_trait]
impl NativeTool for SandboxFileWriteTool {
    fn name(&self) -> &'static str {
        "file_write"
    }

    fn description(&self) -> &'static str {
        "Write a file to a sandbox workspace"
    }

    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "session_id": {
                    "type": "string",
                    "description": "Session ID of the sandbox to write to"
                },
                "path": {
                    "type": "string",
                    "description": "Path of the file to write within the sandbox"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write to the file"
                }
            },
            "required": ["session_id", "path", "content"]
        })
    }

    async fn call(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let session_id = args
            .get("session_id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("missing required parameter: session_id"))?;
        let path = args
            .get("path")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("missing required parameter: path"))?;
        let content = args
            .get("content")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("missing required parameter: content"))?;

        let handle = self.manager.get_or_create(session_id, self.default_config.clone()).await
            .map_err(|e| anyhow::anyhow!("sandbox creation failed: {e}"))?;

        self.manager.runner().write_file(&handle, path, content.as_bytes()).await
            .map_err(|e| anyhow::anyhow!("file write failed: {e}"))?;

        Ok(json!({
            "success": true,
            "path": path,
            "session_id": session_id,
            "bytes_written": content.len()
        }))
    }
}
