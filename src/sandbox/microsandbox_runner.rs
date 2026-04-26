//! Microsandbox runner — hardware VM isolation via libkrun.
//!
//! Uses the `microsandbox` crate to create OCI-compatible microVMs with
//! sub-100ms boot times, named volumes for workspace persistence, and
//! network policy controls.
//!
//! Requires the `sandbox-microsandbox` feature flag and KVM (Linux) or
//! HVF (macOS) hypervisor support.

use async_trait::async_trait;
use microsandbox::Sandbox;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use super::runner::{RunnerCapabilities, RunnerType, SandboxRunner};
use super::types::*;

/// Registry of running microsandbox VMs, keyed by the UAR sandbox handle ID.
type SandboxMap = Arc<Mutex<HashMap<String, Arc<Sandbox>>>>;

/// Microsandbox runner backed by libkrun hardware VMs.
pub struct MicrosandboxRunner {
    default_memory_mib: u32,
    network_enabled: bool,
    /// Live sandbox registry: handle_id → Sandbox.
    sandboxes: SandboxMap,
}

impl MicrosandboxRunner {
    pub fn new(config: &crate::config::AppConfig) -> Self {
        Self {
            default_memory_mib: config.sandbox.default_memory_mib,
            network_enabled: config.sandbox.network_enabled,
            sandboxes: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl std::fmt::Debug for MicrosandboxRunner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MicrosandboxRunner")
            .field("default_memory_mib", &self.default_memory_mib)
            .field("network_enabled", &self.network_enabled)
            .finish()
    }
}

#[async_trait]
impl SandboxRunner for MicrosandboxRunner {
    async fn create(&self, config: SandboxConfig) -> Result<SandboxHandle, SandboxError> {
        let id = uuid::Uuid::new_v4().to_string();

        info!(
            sandbox_id = %id,
            image = %config.image,
            memory_mib = config.memory_mib,
            "Creating microsandbox VM"
        );

        // Microsandbox names must be filesystem-safe; use the UUID directly.
        let msb_config = Sandbox::builder(&id)
            .image(config.image.clone())
            .memory(config.memory_mib)
            .cpus(config.cpus as u8)
            .build()
            .map_err(|e| {
                SandboxError::CreationFailed(format!("sandbox config build failed: {e}"))
            })?;

        let sandbox = Sandbox::create(msb_config).await.map_err(|e| {
            SandboxError::CreationFailed(format!("microsandbox create failed: {e}"))
        })?;

        debug!(sandbox_id = %id, "Microsandbox VM started");

        self.sandboxes
            .lock()
            .await
            .insert(id.clone(), Arc::new(sandbox));

        Ok(SandboxHandle {
            id,
            session_id: None,
        })
    }

    async fn execute(
        &self,
        handle: &SandboxHandle,
        request: ExecutionRequest,
    ) -> Result<ExecutionResult, SandboxError> {
        let start = Instant::now();

        let sandbox = {
            let map = self.sandboxes.lock().await;
            map.get(&handle.id)
                .cloned()
                .ok_or_else(|| SandboxError::NotFound(handle.id.clone()))?
        };

        let cmd = request.language.command().to_string();
        let args = request.language.exec_args(&request.code);

        debug!(
            sandbox_id = %handle.id,
            language = ?request.language,
            "Executing in microsandbox"
        );

        let timeout = request.timeout_seconds.map(Duration::from_secs);
        let env_pairs: Vec<(String, String)> = request.env.into_iter().collect();

        let output = sandbox
            .exec_with(&cmd, |mut e| {
                e = e.args(args);
                for (k, v) in &env_pairs {
                    e = e.env(k, v);
                }
                if let Some(t) = timeout {
                    e = e.timeout(t);
                }
                e
            })
            .await
            .map_err(|e| SandboxError::ExecutionFailed(e.to_string()))?;

        let elapsed = start.elapsed();

        Ok(ExecutionResult {
            exit_code: output.status().code,
            stdout: output
                .stdout()
                .unwrap_or_else(|_| String::from("[invalid utf-8]")),
            stderr: output
                .stderr()
                .unwrap_or_else(|_| String::from("[invalid utf-8]")),
            execution_time_ms: elapsed.as_millis() as u64,
        })
    }

    async fn write_file(
        &self,
        handle: &SandboxHandle,
        path: &str,
        content: &[u8],
    ) -> Result<(), SandboxError> {
        debug!(sandbox_id = %handle.id, path = %path, "Writing file to microsandbox");

        let sandbox = {
            let map = self.sandboxes.lock().await;
            map.get(&handle.id)
                .cloned()
                .ok_or_else(|| SandboxError::NotFound(handle.id.clone()))?
        };

        // Write via stdin pipe into a `cat > path` shell command.
        let cmd = format!("cat > {path}");
        sandbox
            .exec_with("bash", |e| {
                e.args(["-c", &cmd]).stdin_bytes(content.to_vec())
            })
            .await
            .map_err(|e| SandboxError::FileError(e.to_string()))?;

        Ok(())
    }

    async fn read_file(&self, handle: &SandboxHandle, path: &str) -> Result<Vec<u8>, SandboxError> {
        debug!(sandbox_id = %handle.id, path = %path, "Reading file from microsandbox");

        let sandbox = {
            let map = self.sandboxes.lock().await;
            map.get(&handle.id)
                .cloned()
                .ok_or_else(|| SandboxError::NotFound(handle.id.clone()))?
        };

        let output = sandbox
            .exec("cat", [path])
            .await
            .map_err(|e| SandboxError::FileError(e.to_string()))?;

        Ok(output.stdout_bytes().to_vec())
    }

    async fn destroy(&self, handle: SandboxHandle) -> Result<(), SandboxError> {
        info!(sandbox_id = %handle.id, "Destroying microsandbox VM");

        let sandbox = self
            .sandboxes
            .lock()
            .await
            .remove(&handle.id)
            .ok_or_else(|| SandboxError::NotFound(handle.id.clone()))?;

        sandbox
            .stop_and_wait()
            .await
            .map_err(|e| SandboxError::ExecutionFailed(format!("stop failed: {e}")))?;

        // Remove persisted state from disk.
        if let Err(e) = Sandbox::remove(&handle.id).await {
            warn!(sandbox_id = %handle.id, error = %e, "Failed to remove sandbox artifacts");
        }

        Ok(())
    }

    fn capabilities(&self) -> RunnerCapabilities {
        RunnerCapabilities {
            supports_long_running: true,
            supports_networking: self.network_enabled,
            max_execution_seconds: Some(86400), // 24 hours
            runner_type: RunnerType::MicroVm,
        }
    }
}
