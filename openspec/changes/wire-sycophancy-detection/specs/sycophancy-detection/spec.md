## ADDED Requirements

### Requirement: Sycophancy detection runs on completed responses when enabled

When `sycophancy.enabled` is true, the server SHALL run the local rule-based
sycophancy detector on the completed assistant response of a chat run, using the
configured strictness. Detection SHALL NOT make an LLM or network call and SHALL
NOT add latency to the user's response stream (it runs after the terminal event).

#### Scenario: Detection runs after a response completes
- **WHEN** `sycophancy.enabled` is true and a chat run produces a non-empty assistant response
- **THEN** the detector evaluates the response text and produces a sycophancy score without any additional model call

#### Scenario: Disabled is a no-op
- **WHEN** `sycophancy.enabled` is false
- **THEN** no detection runs, no quality event is emitted, and no error occurs

#### Scenario: Empty response is skipped
- **WHEN** the completed assistant response is empty or whitespace-only
- **THEN** detection is skipped and no event is emitted

### Requirement: Strictness maps from configuration

The server SHALL map the `sycophancy.strictness` string to the detector's
strictness level: `"permissive"` → Permissive, `"strict"` → Strict, and any
other value (including `"standard"`) → Standard.

#### Scenario: Strict configuration
- **WHEN** `sycophancy.strictness` is `"strict"`
- **THEN** detection runs at Strict strictness

#### Scenario: Unknown strictness defaults to standard
- **WHEN** `sycophancy.strictness` is an unrecognized value
- **THEN** detection runs at Standard strictness (no error)

### Requirement: Flagging is threshold-based and surfaces a quality event

The server SHALL emit a `SycophancyFlagged` quality event and log a warning when
the detection score is at or above `auto_correct_threshold`, or when the result
is marked critical. The event SHALL carry the run id, the numeric score, the
critical/mandatory flags, and a compact list of pattern classifications (pattern
id, severity, rationale), and SHALL NOT include the full response text.

#### Scenario: High score is flagged
- **WHEN** detection yields a score ≥ `auto_correct_threshold` (or a critical classification)
- **THEN** a `SycophancyFlagged` event is emitted for the run (mapped to `agui.quality.sycophancy` on the SSE stream) and a warning is logged

#### Scenario: Low score is not flagged
- **WHEN** detection yields a score below `auto_correct_threshold` with no critical classification
- **THEN** no `SycophancyFlagged` event is emitted (the score may still be recorded as a metric)

### Requirement: Detection records metrics

The server SHALL record the sycophancy score as a histogram and increment a
counter when a response is flagged.

#### Scenario: Score and flag metrics
- **WHEN** detection runs on a response
- **THEN** `uar_sycophancy_score` records the score, and `uar_sycophancy_flagged_total` is incremented if the response was flagged

### Requirement: Detection failures never break the response

Detection is best-effort and SHALL NOT affect the delivery of the response to the
user. Detection runs after the response has streamed, so any detector panic or
error SHALL be contained (logged) and SHALL NOT fail the request.

#### Scenario: Detector error is contained
- **WHEN** detection encounters an internal error
- **THEN** the user's already-streamed response is unaffected and the error is logged without failing the request
