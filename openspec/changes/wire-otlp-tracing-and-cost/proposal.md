# wire-otlp-tracing-and-cost

## Why

The OpenTelemetry stack is a dead dependency: `opentelemetry` (0.31), `opentelemetry-otlp` (0.31.1), `opentelemetry_sdk` (0.31, `rt-tokio`), and `tracing-opentelemetry` (0.32) are all declared in `Cargo.toml` but used nowhere. `telemetry::init` (`src/uar/telemetry/mod.rs:15`) builds only `fmt`/`json` layers, the `llm.tracing` flag (`config.rs:1223`) is never read, and there is **not a single `#[instrument]` or span** on the run path — so even with an exporter a run would export as one flat span with no children. Separately, per-request **cost** is computable (a `ModelCost` catalog exists at `catalog.rs:201`, `cost_tracking` flag at `config.rs:1219`) but is never computed — `RunDoneWithUsage` is always emitted with `cost_usd_estimate: None` (`manager.rs:1367`) — and **7 of 10 metric recorders are dead** (defined, never called). This is HP2 of `uar-harness-parity`: turn the half-built observability surface into a usable span tree + cost + complete metrics.

## What Changes

- **OTLP pipeline**: in `telemetry::init`, build an `opentelemetry_otlp` tracer + `opentelemetry_sdk` `TracerProvider` and add a `tracing_opentelemetry::layer()` to the subscriber registry — **gated** on `llm.tracing == true` AND an OTLP endpoint env var (`OTEL_EXPORTER_OTLP_ENDPOINT`) being set, so default/offline runs are unchanged. Shut the provider down cleanly on exit.
- **Run-path span tree**: add `#[instrument]` to `Orchestrator::chat_with_history` (`orchestrator.rs:285`), `LiterLlmDriver::stream` (`liter_driver.rs:95`), and the MCP tool dispatch (`mcp/registry.rs:387`) so a run exports as `run → llm-call(s) → tool-call(s)` with attributes (model, provider, tool name, run_id).
- **Per-LLM-call latency**: wrap the driver call (`liter_driver.rs:121`) with `Instant`/`.elapsed()` and record a new `uar_llm_call_duration_seconds` histogram (provider/model labels).
- **Cost computation**: at `manager.rs:1367`, compute `cost_usd_estimate` from the `ModelCost` catalog (`ModelCatalog::global().model(provider, model).cost`) over accumulated input/output (and cache) tokens when `cost_tracking == true`; emit a `uar_llm_cost_usd` metric. Default `cost_tracking` behavior unchanged when off (estimate stays `None`).
- **Wire dead metric recorders (H8 cheap wins)** where an obvious data source exists: `record_cache_tokens` (Anthropic cache tokens already parsed, fed near `record_llm_tokens` in `liter_driver.rs:214`), `set_active_sessions` (session store lifecycle), and the sandbox recorders (`record_sandbox_created` / `record_sandbox_execution` / `record_sandbox_error` / `set_active_sandboxes`) at sandbox execution sites. `set_mcp_server_status` is wired only if an existing MCP health signal is available; otherwise deferred (documented), to avoid inventing a health loop.

## Capabilities

### New Capabilities
- **`runtime-tracing`** — `specs/runtime-tracing/spec.md`. OTLP exporter lifecycle gated on config+endpoint; the run→LLM→tool span tree with required attributes; per-LLM-call latency; per-request cost computation and emission honoring `cost_tracking`.

### Modified Capabilities
- **`prometheus-metrics`** — delta `specs/prometheus-metrics/spec.md`. Adds the new metric series (`uar_llm_call_duration_seconds`, `uar_llm_cost_usd`, cache-token counter) and requires the previously-dead recorders to be wired (cache tokens, active sessions, sandbox lifecycle). Existing request/token/tool-call requirements unchanged.

## Impact

- **Affected code:** `src/uar/telemetry/mod.rs` (OTLP init + shutdown), `src/uar/telemetry/metrics.rs` (new recorders/series), `src/llm/liter_driver.rs` (latency, cache tokens, span), `src/llm/orchestrator.rs` (span), `src/mcp/registry.rs` (tool span + maybe MCP status), `src/uar/runtime/manager.rs` (cost computation + run span), sandbox runner(s) and session store (recorder wiring), `src/main.rs` (provider shutdown on exit), `src/config.rs` (read the existing `tracing`/`cost_tracking` flags — no new fields).
- **APIs:** no HTTP API changes. New Prometheus metric series exposed at the existing auth-exempt `/metrics`. OTLP traces exported only when configured.
- **Provider compatibility:** unaffected — tracing/latency/cost wrap the generic driver path; cost lookups degrade to `None` for models absent from the catalog.
- **Runtime/UX impact:** operators get a real distributed-trace waterfall and per-request cost; no user-facing UI change in this change.
- **Realtime state:** `RunDoneWithUsage` now carries a real `cost_usd_estimate` when enabled (flows through the existing event/replay path to the Runtime Console).
- **Dependencies:** none added — the OTel crates are already present. No new vendor lock-in (OTLP is an open standard).
- **Security:** spans/metrics must not include secrets, prompts, API keys, or PII — only model ids, token counts, durations, costs, tool names, and run ids.
- **KBD workflow state:** YES — HP2 of `uar-harness-parity`; tracked in `.kbd-orchestrator`.
