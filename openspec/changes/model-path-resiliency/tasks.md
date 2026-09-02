# Tasks — model-path-resiliency

scope: src/llm/orchestrator.rs (retry, failover, stream loop), src/llm/liter_driver.rs, src/llm/anthropic_driver.rs, src/llm/external_driver.rs, src/uar/settings/resilience_policy.rs, src/uar/settings/manager.rs, src/uar/runtime/manager.rs (model selection, failover build, turn persistence), src/server.rs (chat SSE cursor), Cargo.toml (backon direct), tests/model_path_resiliency.rs

## 0. Precondition

- [ ] 0.1 Read the vendored liter-llm 1.18.2 error type and record in this file and the phase decision log whether HTTP status and `Retry-After` are exposed; choose the classification site accordingly. In the same read, record whether the liter client honors a per-request base-URL override
- [ ] 0.2 If 0.1 confirms a base-URL override, add `wiremock` 0.6.5 as a dev-dependency and write the `Retry-After` and status tests against a wiremock server; otherwise write them against the driver mock and record the deferral reason here

## 1. Failing tests first

- [ ] 1.1 `tests/model_path_resiliency.rs`: with `retry_jitter_mode: full` and a seeded jitter RNG (`with_jitter_seed`), ten retries produce the exact expected delay sequence for that seed, every delay is within `(0, base * factor^n]`, and at least two delays differ; with `retry_jitter_mode: none` the delays equal the unjittered exponential sequence
- [ ] 1.2 A 429 with `Retry-After: 7` waits 7 s (mock clock) when `retry_respect_retry_after` is on and the computed backoff otherwise
- [ ] 1.3 A driver error typed `kind: InvalidRequest` is not retried; `kind: Overloaded` is; no error-string matching exists
- [ ] 1.4 With `fallback_models: [a, b]` and `a` in cooldown, the run selects `b` without attempting `a`
- [ ] 1.5 A stream that emits nothing for `stream_idle_timeout_ms` fails as retryable and the retry succeeds
- [ ] 1.6 A stream that fails mid-turn persists the partial assistant text with a `TurnInterrupted` marker; the next turn's history contains the marker fragment
- [ ] 1.7 A chat SSE reconnect with `Last-Event-ID` replays only events after the cursor

## 2. Retry and classification

- [ ] 2.1 Add `backon` as a direct dependency at 1.6.0; build `ExponentialBuilder` from `ResiliencePolicy` (jitter, min/max delay, total delay = `retry_budget_ms`, max times)
- [ ] 2.2 Add `ProviderError { status, kind, retry_after }` and classify in each driver; delete `is_retryable_error_string`
- [ ] 2.3 Use `Retry::adjust` to apply `retry_after` when the policy allows

## 3. Selection and failover

- [ ] 3.1 Route run model selection through `ModelRouter` with health gating; keep policy-resolved model as the preferred candidate
- [ ] 3.2 Failover iterates every `fallback_models` entry, each health-gated

## 4. Streams and persistence

- [ ] 4.1 Add `stream_idle_timeout_ms` to `ResiliencePolicy`, settings schema, and validation; apply to the established stream
- [ ] 4.2 On mid-stream error, persist partial content with a `TurnInterrupted` marker fragment; do not persist as complete
- [ ] 4.3 Chat SSE reads `Last-Event-ID` and passes the cursor to `history_since`

## 5. Verification

- [ ] 5.1 Tier 1: `cargo test --locked --no-default-features --features server-full --test model_path_resiliency`
- [ ] 5.2 Tier 2: fmt check and full test run
- [ ] 5.3 `openspec validate model-path-resiliency --strict`
- [ ] 5.4 Deferred live smoke: one real provider 429 observed honoring `Retry-After` (record command and output when run)
