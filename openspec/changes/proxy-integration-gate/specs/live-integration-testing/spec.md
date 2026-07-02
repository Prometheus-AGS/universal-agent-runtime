## ADDED Requirements

### Requirement: Dual-backend live integration tier
The system SHALL provide a `live` integration test tier whose test cases run
against a pluggable `LiveBackend`, with exactly two implementations: `live`
(a real HTTP client bound to `UAR_LLM__BASE_URL`) and `recorded` (the
existing eval-harness `CompletionProvider` fixture). Each test case SHALL be
written once and executed through either backend via a runtime/environment
selection, never duplicated per backend.

#### Scenario: Default backend is recorded
- **WHEN** the live integration tier runs with no
  `UAR_LIVE_INTEGRATION_BACKEND` environment variable set
- **THEN** the tier executes every case against the `recorded` backend

#### Scenario: Explicit live backend targets the local proxy
- **WHEN** `UAR_LIVE_INTEGRATION_BACKEND=live` is set together with
  `UAR_LLM__BASE_URL=http://127.0.0.1:8181/v1`
- **THEN** the tier executes every case against the real HTTP endpoint at
  that base URL, using model `openai/gpt-5.4-mini`

### Requirement: Proxy health check with actionable remediation
The system SHALL provide `scripts/live-integration.sh`, which performs a
health check against the configured live proxy before running any live-backend
test case. On health-check failure, the script SHALL print the specific
remediation steps (re-authenticate the Codex-backed proxy; restart the
`ai.prometheus.openai-proxy` launchd service) and exit non-zero without
running any test case.

#### Scenario: Healthy proxy allows the live run to proceed
- **WHEN** `scripts/live-integration.sh` is invoked and the configured proxy
  responds successfully to the health check
- **THEN** the script proceeds to run the live-backend test cases

#### Scenario: Unhealthy proxy fails fast with remediation
- **WHEN** `scripts/live-integration.sh` is invoked and the configured proxy
  does not respond successfully to the health check
- **THEN** the script exits non-zero, prints the Codex re-login step and the
  `launchctl kickstart -k gui/501/ai.prometheus.openai-proxy` remediation
  command, and does not execute any test case

#### Scenario: CI falls back to the recorded backend without a proxy
- **WHEN** `scripts/live-integration.sh` runs in an environment where the
  local proxy is unreachable and `--allow-recorded-fallback` is set
- **THEN** the script skips the live health check and runs the test cases
  against the `recorded` backend instead of failing

### Requirement: Per-change feature coverage contract
The system SHALL maintain `tests/integration/live/MATRIX.md`, a table mapping
each phase change identifier (`CH-##`) to the name of at least one live
integration test case covering that change's user-facing behavior. A CI check
SHALL fail when a change's identifier is referenced by the phase plan but is
absent from `MATRIX.md`.

#### Scenario: New feature change adds its matrix row
- **WHEN** a change (e.g. `CH-01`) lands a new user-facing feature and its
  pull request does not add a corresponding row to
  `tests/integration/live/MATRIX.md`
- **THEN** the CI matrix-presence check fails, naming the missing `CH-##`
  identifier

#### Scenario: Matrix stays in sync with landed changes
- **WHEN** a change's pull request adds both the live test case and its
  `MATRIX.md` row referencing the same `CH-##` identifier
- **THEN** the CI matrix-presence check passes

### Requirement: Baseline feature case coverage
The live integration tier SHALL include, at minimum, one case for each of the
following baseline flows: streaming chat under each of the `openai`, `agui`,
and `dual` SSE stream modes; an MCP tool-loop round-trip; agent selection via
the `model` request parameter; a memory write followed by a recall; a RAG
document ingest followed by a retrieval; and credential-chain resolution.

#### Scenario: All three streaming modes are exercised
- **WHEN** the baseline case suite runs against either backend
- **THEN** it includes a passing case for `stream_mode: openai`, one for
  `stream_mode: agui`, and one for `stream_mode: dual`

#### Scenario: Tool-loop round-trip is exercised
- **WHEN** the baseline case suite runs against either backend
- **THEN** it includes a passing case that issues an MCP tool call and
  asserts the tool result is incorporated into the final response

### Requirement: Independence from the eval quality gate
The live integration tier SHALL remain a separate mechanism from the eval
harness (`evals/`, Tier-1/Tier-2 CI gate). Neither mechanism SHALL be modified
to depend on or substitute for the other; the eval harness continues to gate
model-output quality regression, and the live integration tier gates
feature-level correctness.

#### Scenario: Eval harness gate is unaffected
- **WHEN** the live integration tier is added to CI
- **THEN** the existing eval harness Tier-1 structural test and Tier-2
  scheduled real-model workflow continue to run unchanged
