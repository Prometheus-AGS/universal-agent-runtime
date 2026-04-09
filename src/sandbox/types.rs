use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub image: String,
    pub memory_mib: u32,
    pub cpus: u32,
    pub network_enabled: bool,
    pub volumes: HashMap<String, String>,
    pub env_vars: HashMap<String, String>,
    pub timeout_secs: u64,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            image: "ubuntu:22.04".to_string(),
            memory_mib: 512,
            cpus: 1,
            network_enabled: false,
            volumes: HashMap::new(),
            env_vars: HashMap::new(),
            timeout_secs: 300,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SandboxHandle {
    pub id: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub language: Language,
    pub code: String,
    pub stdin: Option<String>,
    pub env: HashMap<String, String>,
    pub cwd: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub mode: ExecutionMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Bash,
    Python,
    Rust,
    Node,
}

impl Language {
    #[must_use]
    pub fn command(&self) -> &str {
        match self {
            Self::Bash => "bash",
            Self::Python => "python3",
            Self::Rust => "rustc",
            Self::Node => "node",
        }
    }

    #[must_use]
    pub fn exec_args(&self, code: &str) -> Vec<String> {
        match self {
            Self::Bash => vec!["-c".to_string(), code.to_string()],
            Self::Python => vec!["-c".to_string(), code.to_string()],
            Self::Node => vec!["-e".to_string(), code.to_string()],
            Self::Rust => vec!["--edition=2021".to_string(), "-".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ExecutionMode {
    Ephemeral,
    Session { session_id: String },
    Project { session_id: String, repo_url: Option<String> },
}

#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("Sandbox creation failed: {0}")]
    CreationFailed(String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("File I/O error: {0}")]
    FileError(String),
    #[error("Sandbox not found: {0}")]
    NotFound(String),
    #[error("Max concurrent sandboxes reached ({0})")]
    CapacityExceeded(usize),
    #[error("Execution timed out after {0}s")]
    Timeout(u64),
    #[error("Runner not available: {0}")]
    RunnerUnavailable(String),
}
