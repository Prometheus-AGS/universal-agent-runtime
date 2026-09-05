use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Instant;

static METRICS_HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

/// Build the Prometheus recorder, install it as the process-global recorder,
/// and return its handle.
///
/// Falls back to a detached recorder only when the global slot is already
/// owned by a *foreign* recorder (one this module did not install) — in that
/// case the `counter!` / `gauge!` macros write elsewhere, so the returned
/// handle renders an empty payload. That is the best available outcome: the
/// alternative is aborting a `/metrics` scrape, and this module cannot
/// dislodge a recorder another component installed deliberately.
fn install() -> PrometheusHandle {
    match PrometheusBuilder::new().install_recorder() {
        Ok(handle) => handle,
        Err(err) => {
            tracing::warn!(
                error = %err,
                "could not install the Prometheus recorder; /metrics will render an \
                 empty payload because another global recorder owns metric writes"
            );
            PrometheusBuilder::new().build_recorder().handle()
        }
    }
}

/// Initialize the Prometheus metrics exporter.
///
/// Idempotent: the recorder is installed at most once per process, so callers
/// need not coordinate. `server::start_server` calls this at boot, and the
/// binaries call it earlier still to fix ordering relative to other telemetry
/// setup.
///
/// Call this before recording any metric. The `metrics` macros resolve the
/// global recorder on every call and silently discard writes to a no-op when
/// none is installed, so metrics recorded before this point are lost — which
/// is why installation cannot be left to [`metrics_handle`]'s lazy path
/// alone.
pub fn init() {
    let _ = metrics_handle();
}

/// Get the Prometheus handle for rendering metrics.
///
/// Initializes on first use as a backstop rather than requiring a prior
/// [`init`] call. `/metrics` is registered unconditionally by
/// `server::start_server`, which embedded hosts (including SDK consumers)
/// reach without going through a binary's startup path — so requiring
/// explicit initialization here made that route abort its request thread for
/// every embedder.
///
/// This guarantees `/metrics` never panics, but it is not the intended
/// install point: see [`init`] for why the recorder must be installed at boot
/// rather than on the first scrape.
pub fn metrics_handle() -> &'static PrometheusHandle {
    METRICS_HANDLE.get_or_init(install)
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
    histogram!("uar_request_duration_seconds", &labels).record(duration.as_secs_f64());
}

/// Create a request timer. Call `.finish()` on the returned value when done.
pub fn request_timer() -> Instant {
    Instant::now()
}

// ─────────────────────────────────────────────────────────────────────────────
// LLM Token Metrics
// ─────────────────────────────────────────────────────────────────────────────

/// Record the wall-clock duration of a single LLM driver call.
pub fn record_llm_call_latency(provider: &str, model: &str, duration_secs: f64) {
    let labels = [
        ("provider", provider.to_string()),
        ("model", model.to_string()),
    ];
    histogram!("uar_llm_call_duration_seconds", &labels).record(duration_secs);
}

/// Record an estimated per-request LLM cost in USD. Recorded as a histogram so
/// the `_sum` series gives cumulative spend while preserving per-run distribution.
pub fn record_llm_cost(provider: &str, model: &str, cost_usd: f64) {
    let labels = [
        ("provider", provider.to_string()),
        ("model", model.to_string()),
    ];
    histogram!("uar_llm_cost_usd", &labels).record(cost_usd);
}

/// Record a provider's current health (CH-03): `1.0` when available, `0.0`
/// when in a failover cooldown window.
pub fn record_provider_health(provider: &str, healthy: bool) {
    let labels = [("provider", provider.to_string())];
    gauge!("uar_provider_health", &labels).set(if healthy { 1.0 } else { 0.0 });
}

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
// Response Quality (Sycophancy) Metrics
// ─────────────────────────────────────────────────────────────────────────────

/// Record the sycophancy score (0.0 clean – 1.0 fully sycophantic) of a response.
pub fn record_sycophancy_score(score: f64) {
    histogram!("uar_sycophancy_score").record(score);
}

/// Increment the count of responses flagged as sycophantic.
pub fn record_sycophancy_flagged() {
    counter!("uar_sycophancy_flagged_total").increment(1);
}

/// Increment the count of chat inputs flagged by an input guardrail, by category
/// (`injection` | `pii`).
pub fn record_guardrail_flagged(category: &str) {
    let labels = [("category", category.to_string())];
    counter!("uar_guardrail_flagged_total", &labels).increment(1);
}

// ─────────────────────────────────────────────────────────────────────────────
// Eval Harness Metrics
// ─────────────────────────────────────────────────────────────────────────────

/// Set the mean score for an eval suite + scorer (0.0–1.0).
pub fn record_eval_score(suite: &str, scorer: &str, mean: f64) {
    let labels = [("suite", suite.to_string()), ("scorer", scorer.to_string())];
    gauge!("uar_eval_score", &labels).set(mean);
}

/// Increment the count of eval regressions detected against a baseline.
pub fn record_eval_regression() {
    counter!("uar_eval_regressions_total").increment(1);
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

/// Process-global count of in-flight sandbox executions.
static ACTIVE_SANDBOXES: AtomicI64 = AtomicI64::new(0);

#[expect(
    clippy::cast_precision_loss,
    reason = "concurrent sandbox count is small, far within f64's exact-integer range"
)]
fn report_active_sandboxes(n: i64) {
    set_active_sandboxes(n.max(0) as f64);
}

/// A sandbox execution started — increment the in-flight gauge.
pub fn sandbox_active_inc() {
    report_active_sandboxes(ACTIVE_SANDBOXES.fetch_add(1, Ordering::Relaxed) + 1);
}

/// A sandbox execution finished — decrement the in-flight gauge.
pub fn sandbox_active_dec() {
    report_active_sandboxes(ACTIVE_SANDBOXES.fetch_sub(1, Ordering::Relaxed) - 1);
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

// ── Skill activation metrics (CH-08) ─────────────────────────────────────────

/// Record a skill-activation decision: which skill was selected for an intent,
/// by which classifier backend, and whether it was accepted (vs. an override /
/// fallback). Enables per-skill / per-backend precision-recall accounting —
/// the prerequisite for measuring and improving activation accuracy
/// (fable §8, plan CH-08).
pub fn record_skill_activation(skill_id: &str, backend: &str, accepted: bool) {
    let labels = [
        ("skill_id", skill_id.to_string()),
        ("backend", backend.to_string()),
        ("accepted", accepted.to_string()),
    ];
    counter!("uar_skill_activation_total", &labels).increment(1);
}

/// Record that an activated skill's execution succeeded or failed — pairs with
/// `record_skill_activation` to distinguish "chosen" from "chosen and worked".
pub fn record_skill_activation_outcome(skill_id: &str, success: bool) {
    let labels = [
        ("skill_id", skill_id.to_string()),
        ("success", success.to_string()),
    ];
    counter!("uar_skill_activation_outcome_total", &labels).increment(1);
}

/// One explicit activation is the ground-truth sample for Recall@10.
/// The histogram's sum / count is recall; this does not filter any catalog.
pub fn record_skill_shadow_recall(backend: &str, hit: bool) {
    histogram!("uar_skill_shadow_recall", "backend" => backend.to_string(), "k" => "10")
        .record(if hit { 1.0 } else { 0.0 });
}

/// Exact host activation, distinct from matcher acceptance.
pub fn record_skill_invocation(skill_id: &str, invoke_type: &str) {
    counter!("uar_skill_invocations_total",
        "skill" => skill_id.to_string(), "invoke_type" => invoke_type.to_string())
    .increment(1);
}

/// Attribution only. Never updates the ordinary token or cost totals.
pub fn record_skill_request_usage(skill_id: &str, tokens: u64, cost_usd: Option<f64>) {
    counter!("uar_skill_request_tokens_total", "skill" => skill_id.to_string()).increment(tokens);
    if let Some(cost) = cost_usd {
        histogram!("uar_skill_request_cost_usd", "skill" => skill_id.to_string()).record(cost);
    }
}

#[cfg(test)]
mod tests {
    /// Recording through the `metrics` macros must be visible in the handle's
    /// render — i.e. `metrics_handle()` returns the GLOBAL recorder's handle,
    /// not a detached one.
    ///
    /// Order matters and is the point of this test. `metrics::with_recorder`
    /// resolves the global recorder on every macro call and falls back to a
    /// no-op when none is installed, discarding the write. So `init()` must
    /// run before the first `record_*` call — which is why `start_server`
    /// installs eagerly at boot rather than relying on the first `/metrics`
    /// scrape to initialise lazily.
    #[test]
    fn handle_renders_series_recorded_through_macros() {
        super::init();
        super::record_request("GET", "/health", 200, std::time::Duration::from_millis(1));
        let body = super::metrics_handle().render();
        assert!(
            body.contains("uar_requests_total"),
            "handle is detached from the global recorder; body was:\n{body}"
        );
    }

    #[test]
    fn cache_usage_records_provider_reported_write_and_read_tokens() {
        super::init();
        super::record_cache_tokens("anthropic-stub", "claude-cache-test", 13, 29);
        let body = super::metrics_handle().render();
        let write = body.lines().find(|line| {
            line.starts_with("uar_cache_write_tokens_total")
                && line.contains("provider=\"anthropic-stub\"")
                && line.contains("model=\"claude-cache-test\"")
        });
        let read = body.lines().find(|line| {
            line.starts_with("uar_cache_read_tokens_total")
                && line.contains("provider=\"anthropic-stub\"")
                && line.contains("model=\"claude-cache-test\"")
        });

        assert!(write.is_some_and(|line| line.ends_with(" 13")), "{body}");
        assert!(read.is_some_and(|line| line.ends_with(" 29")), "{body}");
    }
}
