## ADDED Requirements

### Requirement: Runtime recovers from supported failures
UAR SHALL terminate, retry, resume or surface external failures according to documented bounded policies without orphaning work or duplicating side effects.

#### Scenario: MCP server restart
- **WHEN** an MCP server crashes during a tool call and restarts
- **THEN** the run reaches a documented terminal/retry state and later calls can reconnect without restarting UAR

#### Scenario: Streaming soak
- **WHEN** the release candidate runs the defined multi-hour streaming workload
- **THEN** error, memory, latency and duplicate-event thresholds remain within the published limits
