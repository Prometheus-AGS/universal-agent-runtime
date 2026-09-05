use std::sync::Arc;
use tracing::{info, warn};

use super::runner::SandboxRunner;

/// Resolve the governed execution backend once in the trusted server host.
/// No configured remote endpoint means isolation is unavailable, not permission
/// to launch a process through the legacy WasmtimeRunner placeholder.
///
/// # Errors
/// Rejects an unsupported explicit backend, missing remote endpoint, non-Unicode
/// configuration, or a base URL that cannot address the sandbox service API.
pub fn configured_isolated_runner() -> anyhow::Result<Option<Arc<dyn SandboxRunner>>> {
    let read = |name: &str| -> anyhow::Result<Option<String>> {
        match std::env::var(name) {
            Ok(value) => Ok(Some(value)),
            Err(std::env::VarError::NotPresent) => Ok(None),
            Err(std::env::VarError::NotUnicode(_)) => anyhow::bail!("{name} must be valid Unicode"),
        }
    };
    let forced = read("UAR_SANDBOX_RUNNER")?;
    anyhow::ensure!(
        forced.as_deref().is_none_or(|runner| runner == "remote"),
        "Governed sandbox execution requires the remote backend; the local runner does not enforce isolation"
    );
    let endpoint = read("UAR_SANDBOX_REMOTE_URL")?;
    let Some(endpoint) = endpoint else {
        anyhow::ensure!(
            forced.is_none(),
            "UAR_SANDBOX_REMOTE_URL is required for the remote backend"
        );
        return Ok(None);
    };
    let url = reqwest::Url::parse(&endpoint)
        .map_err(|_| anyhow::anyhow!("Invalid remote sandbox base URL"))?;
    anyhow::ensure!(
        matches!(url.scheme(), "http" | "https")
            && url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none(),
        "Remote sandbox requires an HTTP(S) base URL without URL credentials, query or fragment"
    );
    let token = read("UAR_SANDBOX_REMOTE_TOKEN")?;
    Ok(Some(Arc::new(super::remote_runner::RemoteRunner::new(
        url.as_str().trim_end_matches('/').to_owned(),
        token,
    ))))
}

/// Build the best available sandbox runner based on environment and platform.
///
/// Selection order:
/// 1. Explicit `UAR_SANDBOX_RUNNER` env var (`wasmtime`, `remote`)
/// 2. Fallback: remote runner when `UAR_SANDBOX_REMOTE_URL` is configured
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
            _ => warn!("Unknown runner '{}', using auto-detection", runner),
        }
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
