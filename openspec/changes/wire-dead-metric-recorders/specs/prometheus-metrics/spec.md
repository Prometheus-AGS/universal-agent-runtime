## ADDED Requirements

### Requirement: Sandbox execution metrics are emitted

The server SHALL record sandbox lifecycle and execution metrics when a
code-execution tool runs in a sandbox: creation (by runner type + language),
execution outcome (by language + exit-code class, with duration), errors (by
type), and the count of concurrent in-flight sandbox executions.

#### Scenario: Successful sandbox execution
- **WHEN** a sandboxed code-execution tool call completes with exit code 0
- **THEN** `uar_sandbox_created_total` is incremented for the runner type + language, and `uar_sandbox_execution_duration_seconds` is recorded with `exit_code_class="success"`

#### Scenario: Failed sandbox execution
- **WHEN** a sandboxed execution returns a non-zero exit code or the runner errors
- **THEN** the execution is recorded with `exit_code_class="error"` (non-zero exit) and/or `uar_sandbox_errors_total` is incremented with the error type

#### Scenario: Active sandbox gauge reflects concurrency
- **WHEN** sandbox executions begin and end
- **THEN** `uar_sandbox_active` reflects the current number of in-flight sandbox executions (incremented at create, decremented after destroy)

### Requirement: MCP server connection status is emitted

The server SHALL set a per-server status gauge when connecting to configured MCP
servers at startup: healthy when the connection succeeds, unhealthy when it
fails. (Connect-time status; an ongoing health loop is out of scope.)

#### Scenario: MCP server connects
- **WHEN** a configured MCP server (stdio or http) connects successfully at startup
- **THEN** `uar_mcp_server_status{server_name=...}` is set to 1 (healthy)

#### Scenario: MCP server fails to connect
- **WHEN** a configured MCP server fails to connect at startup
- **THEN** `uar_mcp_server_status{server_name=...}` is set to 0 (unhealthy)
