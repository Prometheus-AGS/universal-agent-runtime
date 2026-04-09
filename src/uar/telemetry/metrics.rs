use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::sync::OnceLock;
use std::time::Instant;

static METRICS_HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

/// Initialize the Prometheus metrics exporter and return the handle.
/// Must be called once at startup.
pub fn init() {
    let handle = PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install Prometheus metrics recorder");
    METRICS_HANDLE
        .set(handle)
        .expect("metrics already initialized");
}

/// Get the Prometheus handle for rendering metrics.
pub fn metrics_handle() -> &'static PrometheusHandle {
    METRICS_HANDLE
        .get()
        .expect("metrics not initialized — call metrics::init() first")
}

// ─────────────────────────────────────────────────────────────────────────────
// HTTP Request Metrics
// ─────────────────────────────────────────────────────────────────────────────

/// Record an HTTP request completion.
pub fn record_request(method: &str, path: &str, status: u16, duration: std::time::Duration) {
    let labels = [
        ("method", method.to_string()),
        ("path", path.to_string()),
        ("status", status.to_string()),
    ];
    counter!("uar_requests_total", &labels).increment(1);
    histogram!("uar_request_duration_seconds", &labels)
        .record(duration.as_secs_f64());
}

/// Create a request timer. Call `.finish()` on the returned value when done.
pub fn request_timer() -> Instant {
    Instant::now()
}

// ─────────────────────────────────────────────────────────────────────────────
// LLM Token Metrics
// ─────────────────────────────────────────────────────────────────────────────

/// Record LLM token usage.
pub fn record_llm_tokens(provider: &str, model: &str, input_tokens: u64, output_tokens: u64) {
    let input_labels = [
        ("provider", provider.to_string()),
        ("model", model.to_string()),
        ("direction", "input".to_string()),
    ];
    let output_labels = [
        ("provider", provider.to_string()),
        ("model", model.to_string()),
        ("direction", "output".to_string()),
    ];
    counter!("uar_llm_tokens_total", &input_labels).increment(input_tokens);
    counter!("uar_llm_tokens_total", &output_labels).increment(output_tokens);
}

// ─────────────────────────────────────────────────────────────────────────────
// Cache Token Metrics
// ─────────────────────────────────────────────────────────────────────────────

/// Record prompt-cache token usage (write = cache miss tokens, read = cache hit tokens).
pub fn record_cache_tokens(provider: &str, model: &str, write_tokens: u32, read_tokens: u32) {
    let labels = [
        ("provider", provider.to_string()),
        ("model", model.to_string()),
    ];
    counter!("uar_cache_write_tokens_total", &labels).increment(u64::from(write_tokens));
    counter!("uar_cache_read_tokens_total", &labels).increment(u64::from(read_tokens));
}

// ─────────────────────────────────────────────────────────────────────────────
// Tool Call Metrics
// ─────────────────────────────────────────────────────────────────────────────

/// Record a tool call result.
pub fn record_tool_call(tool_name: &str, success: bool) {
    let status = if success { "success" } else { "failure" };
    let labels = [
        ("tool_name", tool_name.to_string()),
        ("status", status.to_string()),
    ];
    counter!("uar_tool_calls_total", &labels).increment(1);
}

// ─────────────────────────────────────────────────────────────────────────────
// Session Metrics
// ─────────────────────────────────────────────────────────────────────────────

/// Set the active session count gauge.
pub fn set_active_sessions(count: f64) {
    gauge!("uar_active_sessions").set(count);
}

// ─────────────────────────────────────────────────────────────────────────────
// Sandbox Metrics
// ─────────────────────────────────────────────────────────────────────────────

/// Record sandbox creation.
pub fn record_sandbox_created(runner_type: &str, language: &str) {
    let labels = [
        ("runner_type", runner_type.to_string()),
        ("language", language.to_string()),
    ];
    counter!("uar_sandbox_created_total", &labels).increment(1);
}

/// Record sandbox execution duration.
pub fn record_sandbox_execution(language: &str, exit_code_class: &str, duration_secs: f64) {
    let labels = [
        ("language", language.to_string()),
        ("exit_code_class", exit_code_class.to_string()),
    ];
    histogram!("uar_sandbox_execution_duration_seconds", &labels).record(duration_secs);
}

/// Set active sandbox count gauge.
pub fn set_active_sandboxes(count: f64) {
    gauge!("uar_sandbox_active").set(count);
}

/// Record sandbox error.
pub fn record_sandbox_error(error_type: &str) {
    let labels = [("error_type", error_type.to_string())];
    counter!("uar_sandbox_errors_total", &labels).increment(1);
}

/// Record MCP server status.
pub fn set_mcp_server_status(server_name: &str, healthy: bool) {
    let labels = [("server_name", server_name.to_string())];
    let value = if healthy { 1.0 } else { 0.0 };
    gauge!("uar_mcp_server_status", &labels).set(value);
}
