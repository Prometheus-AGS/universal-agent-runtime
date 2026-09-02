<!-- mirror of openspec/changes/model-path-resiliency/proposal.md and specs/*/spec.md -->
# model-path-resiliency

Rank 5 of the codex-harness-comparative-analysis change set. Source: gap G5 in the phase `analysis.md`.

## Why

`ResiliencePolicy` declares `retry_jitter_mode` and `retry_respect_retry_after`, validates and persists them (`src/uar/settings/resilience_policy.rs:39`, `:97-100`), and the retry loop reads neither (`src/llm/orchestrator.rs:640-643`): concurrent runs failing against one provider retry in lockstep, and a provider's own `Retry-After` is ignored. Retryability is a substring match on the error string (`orchestrator.rs:150-175`), the pattern Gemini CLI's own tracker calls fragile. Failover is one hop from a `Vec` (`src/uar/runtime/manager.rs:1665-1690`). `ModelRouter` and `ProviderHealthMonitor` exist but the run path never consults them (`manager.rs:1584-1607`; `src/llm/router.rs:54`), so a provider in cooldown is still selected. There is no idle timeout on an established stream (`orchestrator.rs:628`). A half-streamed turn is persisted as a complete assistant message (`manager.rs:2397-2399`). The main chat SSE path ignores `Last-Event-ID` (`src/server.rs:5019`) although `src/uar/api/routes.rs:94-99` implements it.

`backon` 1.6.0 is already in the dependency graph (via `redis`) and provides jitter, max delay, total-delay budget, and an `adjust` hook for `Retry-After` (registry `backon-1.6.0/src/retry.rs:329`), matching Codex's precedence at `core/src/responses_retry.rs:105`. A circuit breaker is deferred: the health monitor already implements consecutive-failure cooldown; the defect is that it is unwired. Codex paths are outside this repository; verbatim excerpts for the cited lines are in the phase `analysis.md` appendix "verified Codex excerpts".

## What changes

- Retry mechanics through `backon::ExponentialBuilder` configured from `ResiliencePolicy`: jitter modes, max delay, total delay budget, max attempts; `adjust` applies `Retry-After` when `retry_respect_retry_after` is on.
- Retryability from a typed `ProviderError { status, kind, retry_after }` classified at the driver boundary; the substring matcher is deleted.
- Failover walks every entry in `fallback_models`, each gated by provider health.
- Run model selection consults `ModelRouter` and skips providers in cooldown.
- `stream_idle_timeout_ms` on `ResiliencePolicy`; an established stream that emits nothing for that long fails as retryable.
- A partially streamed turn is persisted with a `TurnInterrupted` marker fragment the model can see on the next turn; it is not persisted as complete.
- The chat SSE path reads `Last-Event-ID` and replays from the cursor.

## Scope

- `src/llm/orchestrator.rs` (`:150-175`, `:625-765`, `:858-864`)
- `src/llm/{liter_driver.rs,anthropic_driver.rs,external_driver.rs}` (error classification)
- `src/uar/settings/resilience_policy.rs`, `src/uar/settings/manager.rs`
- `src/uar/runtime/manager.rs` (`:1584-1607`, `:1665-1690`, `:2386-2399`)
- `src/server.rs` (`:5019`)
- `Cargo.toml` (promote `backon` to a direct dependency, same version)
- tests: `tests/model_path_resiliency.rs`

Out of scope: `/readyz` dependency probes (separate change against `deep-health-probes`), circuit breaker, MCP reconnect.

## Dependencies

deterministic-prompt-assembly for the `TurnInterrupted` fragment; otherwise none. Open question 1 from the analysis (liter-llm error typing) must be answered in task 0.1 before task 2.2.

## Verification

Tier 0 per edit; Tier 1 the new tests with a mock driver that returns typed errors and `Retry-After`; Tier 2 at the boundary. Live smoke against a real provider is recorded as deferred with the exact command.

## The uncomfortable thing

If liter-llm 1.18.2 does not expose status or `Retry-After` on its error type, classification has to happen by intercepting the HTTP layer inside the driver, which is more code than this proposal assumes. Task 0.1 settles that before the retry work starts.


## Spec delta: model-path-resiliency

## ADDED Requirements

### Requirement: Retry mechanics honor the configured resilience policy
The runtime SHALL apply the configured jitter mode, maximum delay, total delay budget, and maximum attempts to every model-call retry, and SHALL honor a provider's `Retry-After` value in preference to the computed backoff when the policy enables it.

#### Scenario: Jittered backoff
- **WHEN** `retry_jitter_mode` is not `none` and a retryable failure repeats
- **THEN** successive delays vary within the jitter bounds and never exceed the maximum delay or the total budget

#### Scenario: Provider supplies Retry-After
- **WHEN** a provider responds with a retryable status and a `Retry-After` value and `retry_respect_retry_after` is enabled
- **THEN** the next attempt waits the provider's value

### Requirement: Retryability is decided from a typed provider error
The runtime SHALL classify provider failures into a typed error carrying status, kind, and optional retry delay at the driver boundary, and SHALL NOT decide retryability by matching substrings of an error message.

#### Scenario: Non-retryable request error
- **WHEN** the provider rejects the request as invalid
- **THEN** the run fails without retrying regardless of the message text

### Requirement: Model selection and failover respect provider health
The runtime SHALL consult provider health when selecting the run model and each failover candidate, SHALL skip providers in cooldown, and SHALL try every configured fallback in order.

#### Scenario: Primary in cooldown
- **WHEN** the policy-resolved provider is in cooldown and a healthy fallback is configured
- **THEN** the run uses the fallback without attempting the primary

### Requirement: Established streams have an idle timeout
The runtime SHALL fail an established model stream that emits no data for the configured idle timeout as a retryable error.

#### Scenario: Stalled stream
- **WHEN** a provider opens a stream and stops emitting
- **THEN** the run does not hang; the stream fails after the idle timeout and retry policy applies

### Requirement: Interrupted turns are persisted as interrupted
When a model stream fails or is cancelled after partial output, the runtime SHALL persist the partial assistant content with an interrupted-turn marker visible to the model on the next turn, and SHALL NOT persist it as a complete assistant message.

#### Scenario: Mid-stream provider failure
- **WHEN** a stream fails after emitting some text
- **THEN** session history contains the partial text followed by an interrupted-turn marker fragment

### Requirement: Chat streams resume from a cursor
The primary chat stream endpoint SHALL accept `Last-Event-ID` and SHALL replay only events after that cursor.

#### Scenario: Client reconnects mid-run
- **WHEN** a chat client reconnects with the id of the last event it rendered
- **THEN** it receives subsequent events once and no earlier event is repeated
