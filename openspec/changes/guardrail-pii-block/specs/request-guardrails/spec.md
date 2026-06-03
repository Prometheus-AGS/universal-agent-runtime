## ADDED Requirements

### Requirement: PII blocking is opt-in

The server SHALL reject a chat input flagged as PII before the LLM call when
`block_on_pii` is enabled; when it is disabled (the default), PII findings are
flag-only (detect, emit event/metric, but do not block). This mirrors the
existing injection block and preserves detect-only behavior by default.

#### Scenario: PII blocked when enabled
- **WHEN** `block_on_pii` is enabled and the input is flagged as PII
- **THEN** the request is rejected before the LLM call with a guardrail error and no run is started

#### Scenario: PII flag-only by default
- **WHEN** `block_on_pii` is disabled and the input is flagged as PII
- **THEN** the request proceeds to the LLM (the finding is still flagged via the guardrail event/metric)

#### Scenario: Injection block unaffected
- **WHEN** `block_on_injection` is enabled and an input is flagged as injection
- **THEN** the request is rejected exactly as before, independent of the PII setting
