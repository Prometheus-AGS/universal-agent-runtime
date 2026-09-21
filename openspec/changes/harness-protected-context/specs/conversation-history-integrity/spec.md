## MODIFIED Requirements

### Requirement: One token service counts every budget
The runtime SHALL use one model-keyed counting service for every budget, including complete destination request framing and non-text content. Known encodings SHALL use their mapped o200k_base or cl100k_base encoding; the documented cl100k_base fallback SHALL remain an explicitly approximate estimate, never proof of an unknown model's hard bound. Character-ratio estimates SHALL NOT replace this service.

#### Scenario: Known and unknown models
- **WHEN** budgets are computed for known and unknown model encodings
- **THEN** both use the same service, the known encoding is selected when mapped, and fallback counts are marked approximate with counting revision and uncertainty

#### Scenario: Unknown multimodal cost
- **WHEN** a destination's media or protocol overhead has no validated count bound or preflight result
- **THEN** preparation returns unsupported counting rather than certifying a hard input limit from text tokens alone

### Requirement: Tool output is bounded once at ingest
The host SHALL enforce declared acquisition and storage limits while preserving each acquired canonical native, MCP, graph and terminal tool receipt before presentation formatting. Within those limits, raw payload bytes where available and typed values SHALL be retained unchanged. Presentation truncation SHALL be explicitly labeled and SHALL NOT replace canonical history or the protected model input. Hitting an acquisition/storage limit SHALL produce an explicit incomplete-result outcome, never a complete-success receipt.

#### Scenario: Oversized terminal output
- **WHEN** a tool receipt exceeds the presentation limit but is within authorized acquisition and storage limits
- **THEN** the complete receipt is preserved and any shortened head/tail display states that it is a projection with original size information; model preparation uses the protected receipt or returns overflow

#### Scenario: Output within policy
- **WHEN** a tool returns output within acquisition, storage and presentation limits
- **THEN** the result is recorded unchanged

#### Scenario: Acquisition or storage limit
- **WHEN** output exceeds an enforced acquisition limit or its receipt cannot be durably stored
- **THEN** the host records an incomplete acquisition or persistence failure with known extent, preserves authorized acquired data under retention policy, and does not report complete success or fabricate the remainder

### Requirement: Checkpoint resume restores checkpoint state
Resuming a run SHALL restore the checkpoint's stored state, messages, protected payload identities and pending work, subject to current authorization and retention policy. Deserialization, integrity and authorization failures SHALL be explicit and SHALL NOT substitute empty state.

#### Scenario: Resume from a graph checkpoint
- **WHEN** a client resumes a valid authorized graph checkpoint
- **THEN** the restored history and graph state equal the stored values, and a checkpoint that fails to deserialize returns an error rather than an empty state

#### Scenario: Legacy or revoked checkpoint
- **WHEN** a checkpoint contains previously truncated tool data or data no longer authorized for use
- **THEN** the runtime reports incomplete history or blocked authorization, never reconstructs missing bytes, and does not dispatch revoked content

## REMOVED Requirements

### Requirement: Every tool call has exactly one tool result before dispatch
**Reason**: Its orphan removal and whole-pair dropping scenarios conflict with required lossless protected context.
**Migration**: Apply “Tool history preserves canonical call and result records”; validate legacy histories explicitly and never fabricate lost payloads.

## ADDED Requirements

### Requirement: Tool history preserves canonical call and result records
The runtime SHALL validate history before every provider request: each completed assistant tool call SHALL have exactly one corresponding result, and each result SHALL have its call. Validation and reduction SHALL preserve canonical protected records, identities, roles and ordering. Repair that would discard or alter protected records SHALL return an explicit invalid-history outcome without dispatch.

#### Scenario: Verified terminal call failure
- **WHEN** a durable host record proves a call ended cancelled or failed before recording a result
- **THEN** the host records one typed cancelled or error result with that provenance before dispatch and retains the original call

#### Scenario: Missing result does not prove cancellation
- **WHEN** a call has no result and execution or approval remains pending or its terminal state is unknown
- **THEN** the runtime waits, recovers or blocks with an explicit outcome and does not invent cancellation or rerun a completed side effect

#### Scenario: Orphan or duplicate record
- **WHEN** canonical history contains an orphaned result, duplicate result or conflicting call identity
- **THEN** validation reports invalid history, preserves the records for authorized recovery and does not silently remove data

#### Scenario: Reduction cannot sever or discard a group
- **WHEN** SlidingWindow, KeepFirstLast, progressive summarization or another reducer crosses a completed parallel call/result group
- **THEN** the whole group survives in its original order with identical payloads or preparation returns explicit overflow; dropping a valid group is forbidden


### Requirement: Meaningful assistant records survive empty text
The runtime SHALL persist and restore assistant messages with tool calls, structured content or multimodal data even when their text is empty. Repeated user turns SHALL remain distinct and ordered.

#### Scenario: Empty assistant and repeated continue
- **WHEN** an assistant emits parallel tool calls with empty text between repeated user messages
- **THEN** persistence, reduction and resume retain its call identities and both distinct user turns
