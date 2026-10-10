## ADDED Requirements

### Requirement: Attempt-owned MCP resource lifetime
The runtime SHALL retain host-admitted private server inputs and immutable owner/environment in ephemeral team context and SHALL materialize separate request-owned MCP runtime resources for each admitted team attempt. Attempt completion SHALL retain existing resource cleanup without closing another attempt's resource cache. Materialization SHALL NOT re-read ambient configuration, re-admit a different destination, change captured credential leases or broaden member/host authority.

#### Scenario: Coordinator yield followed by worker and continuation
- **WHEN** an admitted coordinator delegates work and yields using team_wait
- **THEN** normal coordinator cleanup closes only its attempt-owned MCP resources
- **AND** the worker and continuation prepare their own resources from the same trusted captured attachment under unchanged binding, workspace, owner, policy and approval fences
