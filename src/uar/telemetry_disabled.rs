//! Observability facade for builds without the `telemetry` feature.
//!
//! "Telemetry-disabled" means **no OTLP exporters and no metrics**. It does
//! NOT mean no logs: this module still installs a `tracing` subscriber
//! writing to stdout (or `UAR_LOG_FILE`), matching the telemetry build's
//! console behaviour exactly.
//!
//! It used to return `Ok(None)` and install nothing. Because
//! `default = ["minimal"]` does not include `telemetry`, the DEFAULT BUILD
//! therefore emitted no logs at all — every `info!`, `warn!` and `error!` in
//! the runtime went nowhere. The header comment claimed "structured tracing
//! calls remain compiled", which was true and useless: compiled events with
//! no subscriber are discarded. Observed 2026-09-01, when a startup hang and
//! an empty tool list both had to be diagnosed by `eprintln!` and process
//! sampling because the runtime could not say anything about itself.
//!
//! `tracing-subscriber` is an unconditional dependency, so this costs no new
//! feature surface.

use crate::config::LogFormat;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Placeholder matching the exporter shutdown contract.
#[derive(Debug)]
pub struct DisabledTelemetryProvider;

impl DisabledTelemetryProvider {
    /// No exporter buffers exist to flush.
    pub fn shutdown(&self) -> Result<(), std::convert::Infallible> {
        Ok(())
    }
}

/// Install a console subscriber. No exporters, but logs are emitted.
///
/// Mirrors the telemetry build's writer selection (`UAR_LOG_FILE` else
/// stdout) and default filter, so switching the feature on or off changes
/// what is EXPORTED, never whether the process can be observed at all.
pub fn init(log_format: &LogFormat) -> anyhow::Result<Option<DisabledTelemetryProvider>> {
    let filter_layer = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,universal_agent_runtime=debug"));

    let writer = if let Some(path) = std::env::var_os("UAR_LOG_FILE") {
        let path = std::path::PathBuf::from(path);
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|error| {
                anyhow::anyhow!(
                    "opening UAR_LOG_FILE '{}' for append: {error}",
                    path.display()
                )
            })?;
        BoxMakeWriter::new(std::sync::Mutex::new(file))
    } else {
        BoxMakeWriter::new(std::io::stdout)
    };

    // A second `init()` in one process would panic; a server that cannot log
    // is a lesser failure than a server that will not start, so this is
    // tolerant by design (tests and embedders may install their own).
    let installed = match log_format {
        LogFormat::Json => tracing_subscriber::registry()
            .with(filter_layer)
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_target(true)
                    .with_line_number(true)
                    .with_writer(writer),
            )
            .try_init(),
        LogFormat::Compact => tracing_subscriber::registry()
            .with(filter_layer)
            .with(
                tracing_subscriber::fmt::layer()
                    .compact()
                    .with_target(true)
                    .with_line_number(true)
                    .with_writer(writer),
            )
            .try_init(),
        LogFormat::Pretty => tracing_subscriber::registry()
            .with(filter_layer)
            .with(
                tracing_subscriber::fmt::layer()
                    .pretty()
                    .with_target(true)
                    .with_line_number(true)
                    .with_writer(writer),
            )
            .try_init(),
    };
    if let Err(error) = installed {
        eprintln!("tracing subscriber already installed, continuing: {error}");
    }

    Ok(None)
}

/// No-op metrics facade preserving instrumentation call sites.
pub mod metrics {
    use std::time::{Duration, Instant};
    pub fn init() {}
    pub fn record_request(_method: &str, _path: &str, _status: u16, _duration: Duration) {}
    #[must_use]
    pub fn request_timer() -> Instant {
        Instant::now()
    }
    pub fn record_llm_call_latency(_provider: &str, _model: &str, _duration_secs: f64) {}
    pub fn record_llm_cost(_provider: &str, _model: &str, _cost_usd: f64) {}
    pub fn record_provider_health(_provider: &str, _healthy: bool) {}
    pub fn record_llm_tokens(
        _provider: &str,
        _model: &str,
        _input_tokens: u64,
        _output_tokens: u64,
    ) {
    }
    pub fn record_cache_tokens(
        _provider: &str,
        _model: &str,
        _write_tokens: u32,
        _read_tokens: u32,
    ) {
    }
    pub fn record_sycophancy_score(_score: f64) {}
    pub fn record_sycophancy_flagged() {}
    pub fn record_guardrail_flagged(_category: &str) {}
    pub fn record_eval_score(_suite: &str, _scorer: &str, _mean: f64) {}
    pub fn record_eval_regression() {}
    pub fn record_tool_call(_tool_name: &str, _success: bool) {}
    pub fn set_active_sessions(_count: f64) {}
    pub fn record_sandbox_created(_runner_type: &str, _language: &str) {}
    pub fn record_sandbox_execution(_language: &str, _exit_code_class: &str, _duration_secs: f64) {}
    pub fn set_active_sandboxes(_count: f64) {}
    pub fn sandbox_active_inc() {}
    pub fn sandbox_active_dec() {}
    pub fn record_sandbox_error(_error_type: &str) {}
    pub fn set_mcp_server_status(_server_name: &str, _healthy: bool) {}
    pub fn record_skill_activation(_skill_id: &str, _backend: &str, _accepted: bool) {}
    pub fn record_skill_activation_outcome(_skill_id: &str, _success: bool) {}
    pub fn record_skill_shadow_recall(_backend: &str, _hit: bool) {}
    pub fn record_skill_invocation(_skill_id: &str, _invoke_type: &str) {}
    pub fn record_skill_request_usage(_skill_id: &str, _tokens: u64, _cost_usd: Option<f64>) {}
}
