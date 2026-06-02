## Context

The OTel crates (`opentelemetry` 0.31, `opentelemetry-otlp` 0.31.1, `opentelemetry_sdk` 0.31 `rt-tokio`, `tracing-opentelemetry` 0.32) are present but unused. `telemetry::init` (`src/uar/telemetry/mod.rs:15`) builds an `EnvFilter` + a fmt/json layer and calls `.init()`. The run path has zero spans. Metrics live in `src/uar/telemetry/metrics.rs` behind a `PrometheusBuilder` recorder; 3 recorders are wired (`record_request`, `record_llm_tokens`, `record_tool_call`) and 7 are dead. Cost data exists: `ModelCost { input, output, cache_read, cache_write }` (`catalog.rs:201`) looked up via `ModelCatalog::global().model(provider, model).cost`; token totals accumulate in `manager.rs` and `RunDoneWithUsage` is emitted at `manager.rs:1367` with `cost_usd_estimate: None`. Config flags `llm.tracing` (`config.rs:1223`) and `llm.cost_tracking` (`config.rs:1219`) exist but are never read.

## Goals / Non-Goals

**Goals:** a real OTLP span tree (run→LLM→tool) opt-in via config+endpoint; per-LLM-call latency; per-request cost honoring `cost_tracking`; wire the dead recorders that have an obvious data source.

**Non-Goals:** custom dashboards/Grafana; a metrics UI; tracing of non-run subsystems (ingestion already has its own path); replacing the Prometheus pull model; adding new dependencies; emitting cost for models without catalog pricing (graceful `None`).

## Decisions

### D1 — OTLP install is opt-in, gated, with explicit shutdown
Extend `telemetry::init` to also accept the resolved `llm.tracing` flag (or read config). When `tracing == true` AND `OTEL_EXPORTER_OTLP_ENDPOINT` is set, build an OTLP tonic exporter + `TracerProvider` via `install_batch(runtime::Tokio)`, wrap in `tracing_opentelemetry::layer()`, and add it to the registry. Return the `TracerProvider` (or a guard) to `main.rs` so it can call `shutdown()` on exit to flush buffered spans.
- **Why gated on endpoint, not just the flag:** `llm.tracing` defaults to `true`, so gating on the endpoint env keeps default/offline/CI runs from requiring a collector. *Alternative:* always install with a no-op exporter — rejected (still spins a batch worker and can log connection errors).

### D2 — Spans via `#[instrument]`, attributes via fields, no payloads
Add `#[instrument(skip(...), fields(...))]` to `Orchestrator::chat_with_history` (run/turn span), `LiterLlmDriver::stream` (LLM-call span: provider, model), and the tool dispatch in `mcp/registry.rs:387` (tool span: tool_name, success). `skip` the large/ sensitive args (messages, request bodies, `self`); attach only model/provider/tool/run-id/token/duration fields.
- **Why `#[instrument]` over manual `span!`:** less code, correct enter/exit across `.await`, and `skip` makes the no-secrets rule easy to enforce. *Alternative:* manual spans — more error-prone around await points.

### D3 — Latency as a histogram in the driver
In `LiterLlmDriver::stream` wrap the client call (`liter_driver.rs:121`) with `Instant::now()` / `.elapsed()` and record a new `uar_llm_call_duration_seconds` histogram (provider/model labels) next to the existing `record_llm_tokens` site.

### D4 — Cost computed by a pure helper, honoring `cost_tracking`
Add a pure function `estimate_cost(model: &str, input, output, cache_read, cache_write) -> Option<f64>` (in `catalog.rs` next to `ModelCost`) that splits `provider/model`, looks up `ModelCost`, and sums priced components. At `manager.rs:1367`, call it only when `cost_tracking` is true; set `cost_usd_estimate` and also `record` a `uar_llm_cost_usd` metric. Returns `None` (→ event `null`) when disabled or unpriced.
- **Why a pure helper:** unit-testable without a `RunManager` (addresses the HP1 testing gap for the cost math).

### D5 — Wire dead recorders only where a clean data source exists
- `record_cache_tokens`: wire at the `record_llm_tokens` site using the already-parsed Anthropic cache fields. **(definite)**
- `set_active_sessions`: update on session create/remove in the session store. **(definite — fulfills the existing, currently-unmet `prometheus-metrics` requirement)**
- sandbox recorders: wire `record_sandbox_created` / `record_sandbox_execution` / `record_sandbox_error` / `set_active_sandboxes` at the sandbox runner execution sites. **(if sites are clean)**
- `set_mcp_server_status`: wire only if an existing MCP health signal exists; otherwise **defer and document** (do not invent a health loop — Rule 2/scope).

### D6 — Metric cardinality discipline
Metric labels are bounded: provider, model, language, exit-code-class, tool name. **No `run_id` on metric labels** (unbounded) — run id lives only on span attributes. Cost is a counter/gauge without per-run labels.

## Risks / Trade-offs

- **[OTel 0.31 API churn]** the batch-exporter/tracer builder API differs across 0.x versions → Mitigation: code against the exact installed versions; compile-verify; keep the init isolated so breakage is contained.
- **[Exporter overhead/instability]** a misconfigured endpoint could log errors or add latency → Mitigation: opt-in gating (D1); batch (non-blocking) export; clean shutdown.
- **[Cost accuracy]** catalog pricing may be missing/stale for some models → Mitigation: `None` when unpriced; never error; document that cost is an estimate.
- **[Cardinality blowup]** high-cardinality labels would bloat Prometheus → Mitigation: D6 (no run_id labels).
- **[Secret leakage in spans]** `#[instrument]` auto-captures args by default → Mitigation: `skip(...)` all payload/secret args; the no-sensitive-data spec requirement + review.
- **[Sandbox/session wiring surface]** wiring recorders touches several modules → Mitigation: keep each to a one-line recorder call at an existing site; defer anything needing new infrastructure (e.g. MCP health) and document it.

## Migration Plan

1. Add the new metric series + recorders in `metrics.rs` (no behavior change until called).
2. Wire latency + cache tokens in the driver; wire session/sandbox recorders at their sites.
3. Add the `estimate_cost` helper + unit tests; wire cost at `manager.rs:1367` + metric.
4. Add `#[instrument]` spans (no exporter needed to benefit local logs).
5. Wire the OTLP exporter + shutdown last; verify gated on/off.
- **Rollback:** all additive (new metrics, opt-in exporter, spans, a cost field that was already `Option`). Revert restores prior behavior; no data migration.

## Open Questions

- **MCP health source:** is there an existing health/heartbeat for MCP servers to feed `set_mcp_server_status`, or is it deferred? Decide during implementation (D5); document the outcome.
- **Cache-token cost:** include cache read/write in the cost sum when `ModelCost.cache_read/write` are present (yes, when available); confirm token fields are exposed at the cost site.
