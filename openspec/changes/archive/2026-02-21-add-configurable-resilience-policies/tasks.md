## 1. Schema & Configuration

- [x] 1.1 Expand `resilience` schema in `src/uar/settings/manager.rs` to include timeout and retry/backoff/jitter parameters.
- [x] 1.2 Add new resilience fields and defaults in `src/config.rs` with backward-compatible mapping from legacy keys.
- [x] 1.3 Extend `agent_config` schema to include `resilience.mode` and optional override fields.
- [x] 1.4 Add strict validation for ranges, enum values, and retryable status list shape.

## 2. Runtime Enforcement

- [x] 2.1 Implement effective policy resolver (`global + agent override`) with deterministic precedence and fallback behavior.
- [x] 2.2 Apply effective timeout/retry settings to agent-scoped LLM/runtime operations.
- [x] 2.3 Keep global ingress rate limit as baseline and add agent-scoped resilience hooks where agent context exists.
- [x] 2.4 Ensure retry engine honors `Retry-After` and skips non-retryable classes.
- [x] 2.5 Add structured logging/telemetry fields for retries and effective policy source.

## 3. Settings UI/UX

- [x] 3.1 Redesign global Resilience panel with grouped sections, helper text, and progressive disclosure for advanced settings.
- [x] 3.2 Add inline validation/error messaging and disable save while invalid.
- [x] 3.3 Add “Reset to recommended defaults” action in global resilience settings.
- [x] 3.4 Add per-agent `Inherit Global` vs `Override` control in Agent Configuration.
- [x] 3.5 Add effective policy preview for each agent when overrides are enabled.

## 4. Frontend Runtime Retry Wiring

- [x] 4.1 Replace hardcoded stream retry constants in `frontend/src/features/chat/use-message-stream.ts` with configured values.
- [x] 4.2 Preserve current retryable status semantics as defaults and keep `Retry-After` support.
- [x] 4.3 Ensure loading/retry UI status reflects configured retry behavior and stop conditions.

## 5. Verification

- [x] 5.1 Add/update unit tests for policy validation and precedence resolution.
- [x] 5.2 Add/update frontend tests for retry delay calculation and policy-driven behavior.
- [x] 5.3 Run `cargo test` and targeted frontend tests (`bun test frontend/src/features/chat/use-message-stream.test.ts`).
- [x] 5.4 Run `openspec validate add-configurable-resilience-policies --strict`.
