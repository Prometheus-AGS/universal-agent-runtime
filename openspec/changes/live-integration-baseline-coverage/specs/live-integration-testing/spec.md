## ADDED Requirements

### Requirement: Baseline feature case coverage
The live integration tier SHALL include, at minimum, one case for each of the
following baseline flows, run against a real booted server instance:
streaming chat under each of the `openai`, `agui`, and `dual` SSE stream
modes; an MCP tool-loop round-trip; agent selection via the `model` request
parameter; a memory write followed by a recall; a RAG document ingest
followed by a retrieval; and credential-chain resolution.

#### Scenario: All three streaming modes are exercised
- **WHEN** the baseline case suite runs against either backend
- **THEN** it includes a passing case for `stream_mode: openai`, one for
  `stream_mode: agui`, and one for `stream_mode: dual`

#### Scenario: Tool-loop round-trip is exercised
- **WHEN** the baseline case suite runs against either backend
- **THEN** it includes a passing case that issues an MCP tool call and
  asserts the tool result is incorporated into the final response

#### Scenario: Agent selection is exercised
- **WHEN** the baseline case suite runs against either backend
- **THEN** it includes a passing case that selects a non-default agent via
  the `model` request parameter and asserts that agent's configuration was
  used

#### Scenario: Memory and RAG cases are exercised or explicitly excused
- **WHEN** the baseline case suite runs
- **THEN** it includes passing cases for memory write→recall and RAG
  ingest→retrieval, OR (if no embedded/in-memory backend is available for
  either) those specific cases are marked `#[ignore]` with a reason
  referencing design.md Decision D1, and `MATRIX.md` notes the gap
  explicitly rather than silently omitting the row

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

### Requirement: Independence from the eval quality gate
The live integration tier SHALL remain a separate mechanism from the eval
harness (`evals/`, Tier-1/Tier-2 CI gate). Neither mechanism SHALL be modified
to depend on or substitute for the other; the eval harness continues to gate
model-output quality regression, and the live integration tier gates
feature-level correctness.

#### Scenario: Eval harness gate is unaffected
- **WHEN** the baseline case CI job is added
- **THEN** the existing eval harness Tier-1 structural test and Tier-2
  scheduled real-model workflow continue to run unchanged
