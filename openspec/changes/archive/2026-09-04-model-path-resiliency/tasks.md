# Tasks — model-path-resiliency

scope: src/llm/orchestrator.rs (retry, failover, stream loop), src/llm/liter_driver.rs, src/llm/anthropic_driver.rs, src/llm/external_driver.rs, src/uar/settings/resilience_policy.rs, src/uar/settings/manager.rs, src/uar/runtime/manager.rs (model selection, failover build, turn persistence), src/server.rs (chat SSE cursor), Cargo.toml (backon direct), tests/model_path_resiliency.rs

## 0. Precondition

- [x] 0.1 Read the vendored liter-llm 1.18.2 error type and record in this file and the phase decision log whether HTTP status and `Retry-After` are exposed; choose the classification site accordingly. In the same read, record whether the liter client honors a per-request base-URL override
- [x] 0.2 If 0.1 confirms a base-URL override, add `wiremock` 0.6.5 as a dev-dependency and write the `Retry-After` and status tests against a wiremock server; otherwise write them against the driver mock and record the deferral reason here

Task 0.1 source read (2026-09-02): vendored `liter-llm` 1.18.2 exposes `LiterLlmError::status_code()`, `retry_after()`, `is_transient()`, and `error_type()` in `vendor/git/liter-llm/crates/liter-llm/src/error.rs`. The HTTP layer parses `Retry-After` before consuming an error response and retains it on `RateLimited`. `ClientConfigBuilder::base_url` is a client-wide override, not a field on each `ChatCompletionRequest`; it pins all requests from that constructed client to the override. UAR constructs a `LiterLlmDriver` per resolved run configuration, so a wiremock base URL can exercise the real driver boundary. Classification will occur in UAR's concrete drivers before errors enter the `anyhow`-typed `LlmDriver` interface; established liter stream-item errors must be returned as typed errors instead of converted to successful `NormalizedEvent::Error` items. Upstream base-URL context: https://github.com/xberg-io/liter-llm/issues/83. The installed Firecrawl CLI lacks the documented `developer` subcommand; no unsupported upstream behavior was inferred from it.

## 1. Failing tests first

- [x] 1.1 `tests/model_path_resiliency.rs`: with `retry_jitter_mode: full` and a seeded jitter RNG (`with_jitter_seed`), ten retries produce the exact expected delay sequence for that seed, every delay is within `(0, base * factor^n]`, and at least two delays differ; with `retry_jitter_mode: none` the delays equal the unjittered exponential sequence
- [x] 1.2 A 429 with `Retry-After: 7` waits 7 s (mock clock) when `retry_respect_retry_after` is on and the computed backoff otherwise
- [x] 1.3 A driver error typed `kind: InvalidRequest` is not retried; `kind: Overloaded` is; no error-string matching exists
- [x] 1.4 With `fallback_models: [a, b]` and `a` in cooldown, the run selects `b` without attempting `a`
- [x] 1.5 A stream that emits nothing for `stream_idle_timeout_ms` fails as retryable and the retry succeeds
- [x] 1.6 A stream that fails mid-turn persists the partial assistant text with a `TurnInterrupted` marker; the next turn's history contains the marker fragment
- [x] 1.7 A chat SSE reconnect with `Last-Event-ID` replays only events after the cursor

## 2. Retry and classification

- [x] 2.1 Add `backon` as a direct dependency at 1.6.0; build `ExponentialBuilder` from `ResiliencePolicy` (jitter, min/max delay, total delay = `retry_budget_ms`, max times)
- [x] 2.2 Add `ProviderError { status, kind, retry_after }` and classify in each driver; delete `is_retryable_error_string`
- [x] 2.3 Use `Retry::adjust` to apply `retry_after` when the policy allows

## 3. Selection and failover

- [x] 3.1 Route run model selection through `ModelRouter` with health gating; keep policy-resolved model as the preferred candidate
- [x] 3.2 Failover iterates every `fallback_models` entry, each health-gated

## 4. Streams and persistence

- [x] 4.1 Add `stream_idle_timeout_ms` to `ResiliencePolicy`, settings schema, and validation; apply to the established stream
- [x] 4.2 On mid-stream error, persist partial content with a `TurnInterrupted` marker fragment; do not persist as complete
- [x] 4.3 Chat SSE reads `Last-Event-ID` and passes the cursor to `history_since`

## 5. Verification

## Independent phase-end audit corrections

- [x] 6.1 Resume the original authorized run on the primary chat endpoint without starting another model execution; cover mid-run replay and duplicate counts
- [x] 6.2 Retry metadata-only stream stalls before semantic output and retain safe interruption after partial output; cover both boundaries

The current primary route starts a new run before interpreting Last-Event-ID;
its existing replay test covers only the separate runs endpoint. The first-event
retry boundary also treats provider metadata as semantic output. Inspect and
document the existing run-identity contract before adding primary-route resume;
never infer an owner's latest run. Build all audit repairs before test execution.

## Original verification receipts

- [x] 5.1 Tier 1: `cargo test --locked --no-default-features --features server-full --test model_path_resiliency`
- [x] 5.2 Tier 2: fmt check and full test run
- [x] 5.3 `openspec validate model-path-resiliency --strict`
- [ ] 5.4 Deferred live smoke: one real provider 429 observed honoring `Retry-After` (record command and output when run)

## Implementation/evidence distinction — 2026-09-04

Implementation and required local verification are complete (tasks0.1–5.3).
Task5.4 remains unchecked and unverified: no real provider429 was observed.
The proposal explicitly defers this live smoke; the spec requires correct retry
behavior but does not gate implementation completion on encountering a real429.
Independent artifact review confirmed this interpretation against KBD's separate
implementation/evidence dimensions. Do not count this as a21/21 task pass.

The exact live429 reproduction command remains deferred until a real provider
condition and observation method are available; no synthetic response, traffic
flood, or ordinary successful smoke is substituted for the missing receipt.
