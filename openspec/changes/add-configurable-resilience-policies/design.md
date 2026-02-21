## Context

The current resilience surface is limited to four global values (`rate_limit_enabled`, `timeout_disabled`, `requests_per_second`, `burst_size`) and a separate hardcoded streaming retry policy in the frontend. There is no first-class inheritance model for per-agent resilience behavior, and the current settings UI does not explain operational tradeoffs well enough for safe tuning.

## Goals / Non-Goals

- Goals:
  - Define one resilience policy model that supports global defaults and per-agent overrides.
  - Apply deterministic policy precedence at runtime.
  - Expose a high-clarity settings UX with helper text, progressive disclosure, and validation.
  - Remove hardcoded retry constants from chat streaming code.
- Non-Goals:
  - Introduce new transport protocols.
  - Redesign unrelated settings panels.
  - Build provider-specific auto-tuning in this change.

## Decisions

- Decision: Use a hierarchical policy model with explicit inheritance.
  - Global resilience policy is authoritative default.
  - Each agent may choose `inherit` or `override` for resilience.
  - In `override`, unspecified fields fall back to global values.

- Decision: Keep rate limiting, timeouts, and retries in one policy namespace.
  - This avoids fragmented tuning and helps operators reason about tradeoffs.

- Decision: Use idempotency-aware retries.
  - Retry only transient failures and only when operation safety permits retry.
  - Respect `Retry-After` when provided.

- Decision: Use progressive disclosure in settings UI.
  - Show essential controls first.
  - Keep advanced controls discoverable but collapsed by default.

### Policy Model

Global `resilience` keys (representative):

- `rate_limit_enabled` (boolean)
- `requests_per_second` (number, `>=0.1`)
- `burst_size` (number, `>=1`)
- `request_timeout_ms` (integer, `>=1000`)
- `stream_start_timeout_ms` (integer, `>=1000`)
- `retries_enabled` (boolean)
- `retry_max_attempts` (integer, `0..10`)
- `retry_base_delay_ms` (integer, `>=100`)
- `retry_backoff_multiplier` (number, `1.1..5.0`)
- `retry_max_delay_ms` (integer, `>=100`)
- `retry_jitter_mode` (enum: `none|full|equal|decorrelated`)
- `retry_respect_retry_after` (boolean)
- `retryable_http_statuses` (array of integers)
- `retryable_transport_errors` (boolean)
- `retry_budget_ms` (integer, `>=0`)

Per-agent extension (`agent_config.<agent>.resilience` object):

- `mode` (enum: `inherit|override`, default `inherit`)
- Optional override fields mirroring global keys

### Effective Policy Resolution

1. Load global resilience policy.
2. If agent has `mode=inherit`, effective policy = global policy.
3. If agent has `mode=override`, overlay non-null per-agent fields onto global policy.
4. Validate effective policy constraints.
5. Cache effective policy with invalidation on settings save.

### Runtime Application

- Inbound request controls:
  - Global rate limit and global request timeout remain baseline.
- Agent-executed operations:
  - Use effective agent policy for outbound/model retry behavior and operation timeout budget.
- Frontend stream start retries:
  - Use server-provided effective retry policy (or global fallback fetched from settings endpoint).
  - Stop retries after first stream chunk.
  - Respect `Retry-After` if present.

### UX Structure

Global Resilience panel:

- Section A: Essential controls
  - Rate limiting toggle + two primary knobs
  - Retry toggle + max attempts
  - Timeout values
- Section B: Advanced controls (collapsed by default)
  - Jitter mode, retry budget, status code list, transport retry toggle
- Each field includes concise helper text and recommended range.
- Validation is inline and blocks save on invalid input.

Agent Resilience inside Agent Configuration:

- `Inherit Global` / `Override` selector.
- When `Override` selected:
  - Show editable override fields (same grouping as global).
  - Show read-only “Effective policy preview” merged with global defaults.

## Risks / Trade-offs

- More configuration increases misuse risk.
  - Mitigation: guardrail ranges, helper copy, safe defaults, “Reset to recommended”.
- Per-agent policy resolution adds runtime complexity.
  - Mitigation: immutable resolved policy cache and deterministic precedence tests.
- UI density can overwhelm users.
  - Mitigation: progressive disclosure and section-level summaries.

## Migration Plan

1. Add new settings schema fields with defaults while preserving existing keys.
2. Migrate legacy timeout key (`timeout_disabled`) to positive timeout semantics.
3. Update runtime code paths to read effective policy object.
4. Wire frontend streaming retry logic to configured policy.
5. Add compatibility fallback when old keys are present.

## Open Questions

- Should per-agent rate limiting apply only to agent runtime operations or also to incoming API traffic routed to an agent identity?
- Should status code defaults remain fixed globally, or be customizable only in advanced mode?
