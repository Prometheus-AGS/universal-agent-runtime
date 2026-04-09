use super::types::*;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct RunnerCapabilities {
    pub supports_long_running: bool,
    pub supports_networking: bool,
    pub max_execution_seconds: Option<u64>,
    pub runner_type: RunnerType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerType {
    MicroVm,
    Wasmtime,
    Remote,
}

#[async_trait]
pub trait SandboxRunner: Send + Sync {
    async fn create(&self, config: SandboxConfig) -> Result<SandboxHandle, SandboxError>;
    async fn execute(
        &self,
        handle: &SandboxHandle,
        request: ExecutionRequest,
    ) -> Result<ExecutionResult, SandboxError>;
    async fn write_file(
        &self,
        handle: &SandboxHandle,
        path: &str,
        content: &[u8],
    ) -> Result<(), SandboxError>;
    async fn read_file(
        &self,
        handle: &SandboxHandle,
        path: &str,
    ) -> Result<Vec<u8>, SandboxError>;
    async fn destroy(&self, handle: SandboxHandle) -> Result<(), SandboxError>;
    fn capabilities(&self) -> RunnerCapabilities;
}
