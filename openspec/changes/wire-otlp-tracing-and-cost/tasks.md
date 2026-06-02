# Tasks — wire-otlp-tracing-and-cost

## 0. Bootstrap

- [ ] 0.1 Confirm OTel deps present (`opentelemetry` 0.31, `opentelemetry-otlp` 0.31.1, `opentelemetry_sdk` 0.31 rt-tokio, `tracing-opentelemetry` 0.32) — no Cargo change expected
- [ ] 0.2 Confirm `cargo check --features postgres-backend` green on branch base

## 1. Metric series + recorders (metrics.rs)

- [ ] 1.1 Add `uar_llm_call_duration_seconds` histogram recorder (provider, model)
- [ ] 1.2 Add `uar_llm_cost_usd` recorder (no run_id label — D6)
- [ ] 1.3 Confirm/adjust `record_cache_tokens` series → `uar_llm_cache_tokens_total{kind=write|read}`
- [ ] 1.4 Confirm sandbox recorders' series names (`uar_sandbox_executions_total`, `uar_active_sandboxes`, etc.)

## 2. Latency + cache tokens in the driver (liter_driver.rs)

- [ ] 2.1 Wrap the client call (`liter_driver.rs:121`) with `Instant`/`.elapsed()`; record `uar_llm_call_duration_seconds`
- [ ] 2.2 Wire `record_cache_tokens` next to `record_llm_tokens` (`:214`) using the parsed Anthropic cache token fields

## 3. Cost computation (catalog.rs + manager.rs)

- [ ] 3.1 Add pure `estimate_cost(model, input, output, cache_read, cache_write) -> Option<f64>` in `catalog.rs` using `ModelCatalog::global().model(provider, model).cost`
- [ ] 3.2 Unit tests for `estimate_cost`: priced model, unpriced model (None), cache-priced model
- [ ] 3.3 At `manager.rs:1367`, compute `cost_usd_estimate` when `cost_tracking` is on; set the event field + record `uar_llm_cost_usd`. `None` when off/unpriced
- [ ] 3.4 Read the existing `llm.cost_tracking` flag (`config.rs:1219`) at the cost site (no new config field)

## 4. Wire remaining dead recorders (H8)

- [ ] 4.1 `set_active_sessions` on session create/remove (fulfills existing `prometheus-metrics` requirement)
- [ ] 4.2 Sandbox recorders at the runner execution sites (`record_sandbox_created`/`_execution`/`_error`, `set_active_sandboxes`)
- [ ] 4.3 `set_mcp_server_status`: wire IF an existing MCP health signal exists; else defer + document (no invented health loop)

## 5. Run-path spans (#[instrument])

- [ ] 5.1 `#[instrument(skip(...), fields(...))]` on `Orchestrator::chat_with_history` (run/turn span; run_id, model)
- [ ] 5.2 `#[instrument]` on `LiterLlmDriver::stream` (provider, model) — skip request/messages/self
- [ ] 5.3 Span on MCP tool dispatch (`mcp/registry.rs:387`) (tool_name, success)
- [ ] 5.4 Verify no payloads/secrets captured (all sensitive args in `skip(...)`)

## 6. OTLP exporter lifecycle (telemetry/mod.rs + main.rs)

- [ ] 6.1 In `telemetry::init`, when `llm.tracing` AND `OTEL_EXPORTER_OTLP_ENDPOINT` set, build OTLP tonic exporter + `TracerProvider` (`install_batch(rt-tokio)`) + `tracing_opentelemetry::layer()`; add to registry
- [ ] 6.2 Read the existing `llm.tracing` flag (`config.rs:1223`) and thread it into `init`
- [ ] 6.3 Return the `TracerProvider`/guard to `main.rs`; call `shutdown()` on exit to flush
- [ ] 6.4 Verify gated OFF by default (no endpoint → starts clean, no exporter, no error)

## 7. Validation (gate)

- [ ] 7.1 `cargo check --features postgres-backend` clean; zero new warnings
- [ ] 7.2 `cargo clippy` — no new warnings in touched files
- [ ] 7.3 `cargo test --features postgres-backend --lib` — existing pass + new `estimate_cost` tests pass
- [ ] 7.4 Manual: hit `/metrics`, confirm new series register after a run; with an OTLP collector, confirm a run→LLM→tool span tree (pending live env — document if not runnable here)
- [ ] 7.5 `openspec validate wire-otlp-tracing-and-cost --strict`; update `.kbd-orchestrator` progress

## Notes

- Cardinality: NO `run_id` on metric labels (span attributes only) — D6.
- No new dependencies; OTel crates already present.
- Security: spans/metrics carry only model/provider/token/duration/cost/tool/run-id — never prompts, keys, or PII.
- `set_mcp_server_status` and any recorder needing new infrastructure are deferred + documented rather than inventing health loops (Rule 2 simplicity).
