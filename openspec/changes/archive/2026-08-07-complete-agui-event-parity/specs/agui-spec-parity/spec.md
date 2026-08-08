## ADDED Requirements

### Requirement: AG-UI spec mode emits the complete official vocabulary
The agui_spec stream SHALL emit STATE_SNAPSHOT, MESSAGES_SNAPSHOT and RAW
events per the official protocol, and tool lifecycle events SHALL map
start/args/end faithfully.

#### Scenario: Snapshot on subscribe
- **WHEN** a client attaches to an in-flight run's stream
- **THEN** it receives STATE_SNAPSHOT/MESSAGES_SNAPSHOT before deltas

#### Scenario: Tool lifecycle fidelity
- **WHEN** a tool call starts
- **THEN** TOOL_CALL_START is emitted (not a remapped TOOL_CALL_END)
