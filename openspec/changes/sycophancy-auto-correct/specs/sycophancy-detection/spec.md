## ADDED Requirements

### Requirement: Sycophancy auto-correction is opt-in and non-blocking

The server SHALL, when `auto_correct` is enabled and a completed response is
flagged (and `log_only` is not set), run one corrective LLM pass and emit the
corrected text as a follow-up event. Correction SHALL run after the original
response has streamed (never blocking or delaying it) and SHALL be skipped when
`auto_correct` is disabled (the default) or `log_only` is set.

#### Scenario: Correction emitted when enabled
- **WHEN** `auto_correct` is enabled, `log_only` is false, and a response is flagged at/above `auto_correct_threshold` (or critical)
- **THEN** one corrective LLM pass runs after the terminal event and a `SycophancyCorrected` event (mapped to `agui.quality.sycophancy_corrected`) carrying the rewritten text is emitted on the run stream

#### Scenario: Detection-only by default
- **WHEN** `auto_correct` is disabled (default)
- **THEN** a flagged response emits only `SycophancyFlagged` (no correction pass, no follow-up)

#### Scenario: log_only suppresses correction
- **WHEN** `auto_correct` is enabled but `log_only` is true
- **THEN** the response is flagged but no corrective pass runs

#### Scenario: Correction failure is contained
- **WHEN** the corrective LLM pass errors
- **THEN** the already-delivered original response is unaffected and the error is logged without failing the request
