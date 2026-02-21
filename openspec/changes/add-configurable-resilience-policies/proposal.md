# Change: Add Configurable Resilience Policies

## Why

Resilience behavior is currently partially hardcoded and mostly global, which makes it difficult to tune safely for different agent behaviors and runtime environments. We need a consistent global policy model, per-agent overrides, and a clear settings UX so operators can tune rate limits, timeouts, and retry behavior without editing code.

## What Changes

- Add an expanded `resilience` settings model with explicit policy groups for:
  - Rate limiting
  - Timeouts
  - Retries/backoff/jitter
  - Retry eligibility (HTTP status and transport classes)
- Add per-agent resilience overrides in `agent_config` with explicit inheritance (`inherit` vs `override`) and deterministic precedence.
- Apply effective resilience policies at runtime:
  - Global defaults for system-wide behavior
  - Agent-specific effective policy for agent-scoped operations
  - Consistent `Retry-After` handling and idempotency-aware retry gates
- Replace the current minimal resilience form with a purpose-driven settings UX:
  - Basic vs Advanced progressive disclosure
  - Clear helper text for each setting
  - Inline validation and guardrails
  - Effective-value preview for per-agent overrides
- Add policy observability for troubleshooting (policy id/effective source, retry attempts, stop reason).

## Recommended Defaults (Research-Informed)

These defaults are based on current guidance from AWS, Google Cloud, Azure, OpenAI, RFC 6585/9110 semantics, and are tuned for interactive chat latency.

- `rate_limit.enabled`: `true`
- `rate_limit.requests_per_second`: `10`
- `rate_limit.burst_size`: `20`
- `timeouts.request_timeout_ms`: `30000`
- `timeouts.stream_start_timeout_ms`: `15000`
- `retries.enabled`: `true`
- `retries.max_attempts`: `3`
- `retries.base_delay_ms`: `1000`
- `retries.backoff_multiplier`: `2.0`
- `retries.max_delay_ms`: `10000`
- `retries.jitter_mode`: `full`
- `retries.respect_retry_after`: `true`
- `retries.retryable_http_statuses`: `[408, 425, 429, 500, 502, 503, 504]`
- `retries.retryable_transport_errors`: `true`
- `retries.max_retry_budget_ms`: `20000`

Inference note: `max_delay_ms=10000` and `max_retry_budget_ms=20000` are derived from cloud guidance plus interactive UX constraints (faster fail/feedback than background job defaults).

## Impact

- Affected specs:
  - `resilience-policy-config`
  - `resilience-policy-enforcement`
  - `resilience-settings-ui`
- Affected code (expected):
  - `src/config.rs`
  - `src/uar/settings/manager.rs`
  - `src/server.rs`
  - `frontend/src/admin/pages/settings-page.tsx`
  - `frontend/src/features/chat/use-message-stream.ts`
  - related tests in `src/**` and `frontend/src/**`

## Research References

- AWS Builders’ Library, Timeouts/retries/backoff/jitter: [https://aws.amazon.com/builders-library/timeouts-retries-and-backoff-with-jitter/](https://aws.amazon.com/builders-library/timeouts-retries-and-backoff-with-jitter/)
- AWS Well-Architected retry guidance: [https://docs.aws.amazon.com/wellarchitected/latest/reliability-pillar/rel_mitigate_interaction_failure_limit_retries.html](https://docs.aws.amazon.com/wellarchitected/latest/reliability-pillar/rel_mitigate_interaction_failure_limit_retries.html)
- Google Cloud retry strategy (retryable status codes and defaults): [https://cloud.google.com/storage/docs/retry-strategy](https://cloud.google.com/storage/docs/retry-strategy)
- Azure transient fault handling guidance: [https://learn.microsoft.com/azure/well-architected/design-guides/handle-transient-faults](https://learn.microsoft.com/azure/well-architected/design-guides/handle-transient-faults)
- RFC 6585 (429 semantics): [https://httpwg.org/specs/rfc6585.html#status.429](https://httpwg.org/specs/rfc6585.html#status.429)
- MDN Retry-After header behavior: [https://developer.mozilla.org/docs/Web/HTTP/Headers/Retry-After](https://developer.mozilla.org/docs/Web/HTTP/Headers/Retry-After)
- Material 3 text/supporting/error guidance: [https://m3.material.io/components/text-fields/guidelines](https://m3.material.io/components/text-fields/guidelines)
- USWDS form validation guidance: [https://designsystem.digital.gov/components/form/](https://designsystem.digital.gov/components/form/)
- OpenAI 429/backoff guidance: [https://help.openai.com/en/articles/5955604-how-can-i-solve-429-too-many-requests-errors](https://help.openai.com/en/articles/5955604-how-can-i-solve-429-too-many-requests-errors)
