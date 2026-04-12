use std::sync::Arc;
use tracing::{info, warn};

use super::runner::SandboxRunner;

/// Build the best available sandbox runner based on environment and platform.
///
/// Selection order:
/// 1. Explicit `UAR_SANDBOX_RUNNER` env var (`wasmtime`, `remote`, `microsandbox`)
/// 2. Auto-detect: KVM on Linux or HVF on macOS (if `sandbox-microsandbox` feature enabled)
/// 3. Remote runner if `UAR_SANDBOX_REMOTE_URL` is set
/// 4. Wasmtime fallback (process-based, limited isolation)
#[allow(unused_variables, unreachable_code)]
pub fn build_runner(config: &crate::config::AppConfig) -> Arc<dyn SandboxRunner> {
    let forced = std::env::var("UAR_SANDBOX_RUNNER").ok();

    if let Some(ref runner) = forced {
        match runner.as_str() {
            "wasmtime" => {
                info!("Forced wasmtime runner via UAR_SANDBOX_RUNNER");
                return Arc::new(super::wasmtime_runner::WasmtimeRunner::new());
            }
            "remote" => {
                let url = std::env::var("UAR_SANDBOX_REMOTE_URL")
                    .expect("UAR_SANDBOX_REMOTE_URL required for remote runner");
                let token = std::env::var("UAR_SANDBOX_REMOTE_TOKEN").ok();
                info!("Forced remote runner via UAR_SANDBOX_RUNNER");
                return Arc::new(super::remote_runner::RemoteRunner::new(url, token));
            }
            #[cfg(feature = "sandbox-microsandbox")]
            "microsandbox" => {
                info!("Forced microsandbox runner via UAR_SANDBOX_RUNNER");
                return Arc::new(super::microsandbox_runner::MicrosandboxRunner::new(config));
            }
            _ => warn!("Unknown runner '{}', using auto-detection", runner),
        }
    }

    // Auto-detection: KVM on Linux
    #[cfg(all(target_os = "linux", feature = "sandbox-microsandbox"))]
    if std::path::Path::new("/dev/kvm").exists() {
        info!("KVM available - using microsandbox runner");
        return Arc::new(super::microsandbox_runner::MicrosandboxRunner::new(config));
    }

    // Auto-detection: HVF on macOS
    #[cfg(all(target_os = "macos", feature = "sandbox-microsandbox"))]
    {
        info!("macOS detected - using microsandbox runner (HVF)");
        return Arc::new(super::microsandbox_runner::MicrosandboxRunner::new(config));
    }

    // Fallback: remote if URL is configured
    if let Ok(url) = std::env::var("UAR_SANDBOX_REMOTE_URL") {
        let token = std::env::var("UAR_SANDBOX_REMOTE_TOKEN").ok();
        info!("Remote sandbox URL configured - using remote runner");
        return Arc::new(super::remote_runner::RemoteRunner::new(url, token));
    }

    warn!("No microVM support available - using Wasmtime fallback (limited)");
    Arc::new(super::wasmtime_runner::WasmtimeRunner::new())
}
