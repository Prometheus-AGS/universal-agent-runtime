//! No-op observability facade for telemetry-disabled builds.

use crate::config::LogFormat;

/// Placeholder matching the exporter shutdown contract.
#[derive(Debug)]
pub struct DisabledTelemetryProvider;

impl DisabledTelemetryProvider {
    /// No exporter buffers exist to flush.
    pub fn shutdown(&self) -> Result<(), std::convert::Infallible> {
        Ok(())
    }
}

/// Initialize no exporters. Structured `tracing` calls remain compiled.
#[must_use]
pub fn init(_log_format: &LogFormat) -> Option<DisabledTelemetryProvider> {
    None
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
}
