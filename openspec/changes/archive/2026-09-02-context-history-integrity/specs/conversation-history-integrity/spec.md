## ADDED Requirements

### Requirement: Every tool call has exactly one tool result before dispatch
The runtime SHALL normalize conversation history before every provider request so that every assistant tool call has exactly one corresponding tool result and every tool result corresponds to an assistant tool call.

#### Scenario: Missing result is synthesized
- **WHEN** an assistant message contains a tool call whose result was never recorded because the run was cancelled or the call failed before completion
- **THEN** the runtime inserts a typed `cancelled` or `error` tool result for that call id immediately after the call before dispatch

#### Scenario: Orphaned result is removed
- **WHEN** history contains a tool result whose call id matches no assistant tool call
- **THEN** the runtime removes that result before dispatch and records a warning

#### Scenario: Reduction cannot sever a pair
- **WHEN** a context reduction window boundary would fall between an assistant tool call and its result
- **THEN** the pair is kept or dropped as a unit

### Requirement: The system message survives every context reduction
Context reducers SHALL treat the system message as pinned and SHALL NOT drop, reorder, or summarize it.

#### Scenario: Long conversation under a sliding window
- **WHEN** history exceeds the configured window
- **THEN** the system message remains at index 0 and only conversation turns are reduced

### Requirement: Identical repeated user messages are preserved
Context reducers SHALL NOT remove a user message because its content equals an earlier user message.

#### Scenario: User repeats "continue"
- **WHEN** two consecutive user turns have identical content
- **THEN** both remain in the reduced history in order

### Requirement: One token service counts every budget
The runtime SHALL count tokens through one model-keyed token service, using `o200k_base` or `cl100k_base` when the model maps to a known encoding and `cl100k_base` as the documented fallback, and SHALL NOT use a character-ratio estimate on any path.

#### Scenario: Known and unknown models
- **WHEN** a budget is computed for a model with a known encoding and for a model without one
- **THEN** the known model uses its encoding and the unknown model uses `cl100k_base`, and both counts come from the same service

### Requirement: Tool output is bounded once at ingest
The runtime SHALL truncate tool output once, when recording the model-visible result, using a middle-out policy expressed in bytes or tokens, and SHALL prefix truncated output with a warning that states the original token count and total line count.

#### Scenario: Oversized terminal output
- **WHEN** a native, MCP, or terminal tool returns output larger than its policy
- **THEN** the recorded result is within the policy, begins with the warning header, and retains the head and tail of the original output

#### Scenario: Output within policy
- **WHEN** a tool returns output within its policy
- **THEN** the result is recorded unchanged

### Requirement: Checkpoint resume restores checkpoint state
Resuming a run from a checkpoint SHALL seed the new run with the checkpoint's stored state and messages.

#### Scenario: Resume from a graph checkpoint
- **WHEN** a client resumes a run from a checkpoint id
- **THEN** the new run's history equals the checkpoint's messages and its graph state equals the checkpoint's state, and a checkpoint that fails to deserialize returns an error rather than an empty state
