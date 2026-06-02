# Tasks — wire-otlp-tracing-and-cost

## 0. Bootstrap

- [x] 0.1 OTel deps present (`opentelemetry` 0.31, `opentelemetry-otlp` 0.31.1 [default `http-proto`], `opentelemetry_sdk` 0.31 rt-tokio, `tracing-opentelemetry` 0.32) — no Cargo change
- [x] 0.2 `cargo check --features postgres-backend` green on branch base

## 1. Metric series + recorders (metrics.rs)

- [x] 1.1 `record_llm_call_latency` → `uar_llm_call_duration_seconds` histogram (provider, model)
- [x] 1.2 `record_llm_cost` → `uar_llm_cost_usd` histogram (provider, model; no run_id label — D6)
- [x] 1.3 `record_cache_tokens` already defined → `uar_cache_read_tokens_total` / `uar_cache_write_tokens_total`
- [ ] 1.4 Sandbox series — DEFERRED (see Notes; removed from spec delta)

## 2. Latency + cache tokens in the driver (liter_driver.rs)

- [x] 2.1 Wrapped the client call with `Instant`/`.elapsed()`; record `uar_llm_call_duration_seconds`
- [x] 2.2 Wired `record_cache_tokens` from `usage.prompt_tokens_details.cached_tokens` (read only; write not exposed by liter — documented), and surface `cached_tokens` on the `Usage` event

## 3. Cost computation (catalog.rs + manager.rs)

- [x] 3.1 `ModelCost::compute(...)` (pure) + `estimate_cost(model, input, output, cache_read) -> Option<f64>` using `ModelCatalog::global()`
- [x] 3.2 Unit tests (5): input/output, cache-read discount, cache fallback-to-input-rate, unknown model → None, malformed id → None — all pass
- [x] 3.3 At the `RunDoneWithUsage` emission, compute `cost_usd_estimate` when `cost_tracking` on; set the event field + record `uar_llm_cost_usd`; `None` when off/unpriced
- [x] 3.4 Captured the existing `llm.cost_tracking` flag for the spawned task (no new config field)

## 4. Wire remaining dead recorders (H8)

- [x] 4.1 `set_active_sessions` — already wired in `session/thread.rs` (create/remove); confirmed live
- [ ] 4.2 Sandbox recorders — DEFERRED: spread across 3 `SandboxRunner` impls + lifecycle hooks; not a cheap win (Rule 2). Documented.
- [ ] 4.3 `set_mcp_server_status` — DEFERRED: no existing MCP health/heartbeat signal to feed it; not inventing a health loop. Documented.

## 5. Run-path spans (#[instrument])

- [x] 5.1 Run-level span via `.instrument(info_span!("run", run_id, agent_id))` on the spawned run task (parents the child spans — chose this over a short-lived span on `chat_with_history`, which only covers setup)
- [x] 5.2 `#[instrument(name="llm.call", skip(self, req), fields(model))]` on `LiterLlmDriver::stream`
- [x] 5.3 `#[instrument(name="tool.call", skip(self, arguments), fields(tool))]` on `McpRegistry::call_namespaced_tool`
- [x] 5.4 No payloads/secrets captured — `skip(...)` covers `self`, request, messages, arguments

## 6. OTLP exporter lifecycle (telemetry/mod.rs + main.rs)

- [x] 6.1 `build_otlp_provider`: OTLP/HTTP exporter (`with_http`, default feature — not tonic) + `SdkTracerProvider` (`with_batch_exporter`) + service-name resource + `tracing_opentelemetry::layer()` added to all registry branches
- [x] 6.2 Gated on env (`OTEL_EXPORTER_OTLP_ENDPOINT` set AND `UAR_LLM__TRACING` != false) — read from env because telemetry inits before config load (env is highest precedence)
- [x] 6.3 `init` returns `Option<SdkTracerProvider>`; `main` calls `provider.shutdown()` after the server stops to flush spans
- [x] 6.4 Gated OFF by default: no endpoint → no exporter, clean start (verified by compile + logic; OFF is the default path in all tests)

## 7. Validation (gate)

- [x] 7.1 `cargo check --features postgres-backend` clean; zero new warnings (`#[expect]` on the cost casts)
- [x] 7.2 `cargo clippy` — no new warnings in touched files (remaining hits are pre-existing)
- [x] 7.3 `cargo test --features postgres-backend --lib` — 223 pass (218 + 5 new `cost_tests`)
- [ ] 7.4 Manual: hit `/metrics` after a run to see new series; with an OTLP collector, confirm a run→llm.call→tool.call span tree — PENDING live env (not runnable headlessly)
- [x] 7.5 `openspec validate wire-otlp-tracing-and-cost --strict`; `.kbd-orchestrator` progress updated in wrap-up

## Notes

- **OTLP transport:** used OTLP/**HTTP** (`with_http`) because `opentelemetry-otlp`'s default feature is `http-proto`, not `grpc-tonic`. Avoids a Cargo feature addition (Rule 27). Endpoint e.g. `http://localhost:4318/v1/traces`.
- **Cost source:** used the in-repo `ModelCost` catalog (self-contained, unit-tested). Note: liter-llm also ships richer cache-aware pricing (`pricing.json`, 1500+ models, `estimated_cost`) — a future option for more complete/cache-accurate cost.
- **Cache tokens:** liter exposes only the cache-**read** portion; cache-write/creation is folded into provider billing and not separately reported. Spec delta scoped accordingly.
- **Deferred (documented, not silently dropped):** sandbox metrics (4.2), MCP server-status gauge (4.3) — both need infrastructure (per-runner hooks / a health loop) beyond a cheap win; sandbox requirement removed from the spec delta to keep the contract honest.
- **Cardinality:** no `run_id` on metric labels (span attributes only).
- **Security:** spans/metrics carry only model/provider/token/duration/cost/tool/run-id — never prompts, keys, or PII.
