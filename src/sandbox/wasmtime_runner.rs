use async_trait::async_trait;
use std::time::Instant;

use super::runner::*;
use super::types::*;

#[derive(Debug)]
pub struct WasmtimeRunner;

impl WasmtimeRunner {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for WasmtimeRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SandboxRunner for WasmtimeRunner {
    async fn create(&self, _config: SandboxConfig) -> Result<SandboxHandle, SandboxError> {
        Ok(SandboxHandle {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: None,
        })
    }

    async fn execute(
        &self,
        _handle: &SandboxHandle,
        request: ExecutionRequest,
    ) -> Result<ExecutionResult, SandboxError> {
        let start = Instant::now();
        let cmd = request.language.command();
        let args = request.language.exec_args(&request.code);

        let output = tokio::process::Command::new(cmd)
            .args(&args)
            .output()
            .await
            .map_err(|e| SandboxError::ExecutionFailed(e.to_string()))?;

        Ok(ExecutionResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            execution_time_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
        })
    }

    async fn write_file(
        &self,
        _handle: &SandboxHandle,
        path: &str,
        content: &[u8],
    ) -> Result<(), SandboxError> {
        tokio::fs::write(path, content)
            .await
            .map_err(|e| SandboxError::FileError(e.to_string()))
    }

    async fn read_file(
        &self,
        _handle: &SandboxHandle,
        path: &str,
    ) -> Result<Vec<u8>, SandboxError> {
        tokio::fs::read(path)
            .await
            .map_err(|e| SandboxError::FileError(e.to_string()))
    }

    async fn destroy(&self, _handle: SandboxHandle) -> Result<(), SandboxError> {
        Ok(()) // No-op for process-based execution
    }

    fn capabilities(&self) -> RunnerCapabilities {
        RunnerCapabilities {
            supports_long_running: false,
            supports_networking: false,
            max_execution_seconds: Some(300),
            runner_type: RunnerType::Wasmtime,
        }
    }
}
