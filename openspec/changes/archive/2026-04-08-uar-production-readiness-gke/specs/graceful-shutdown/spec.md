## ADDED Requirements

### Requirement: Server handles SIGTERM gracefully
The server SHALL intercept SIGTERM and SIGINT signals and initiate graceful shutdown, allowing in-flight HTTP requests and SSE streams to complete within a configurable timeout before process exit.

#### Scenario: Rolling update with active SSE stream
- **WHEN** Kubernetes sends SIGTERM to the UAR pod while an SSE chat stream is active
- **THEN** the server stops accepting new connections, allows the active stream to complete (up to 30s default), then exits with code 0

#### Scenario: No active connections during shutdown
- **WHEN** SIGTERM is received with no active connections
- **THEN** the server exits within 1 second

#### Scenario: Shutdown timeout exceeded
- **WHEN** in-flight requests do not complete within the configured timeout
- **THEN** the server forcefully terminates remaining connections and exits

### Requirement: Resource cleanup on shutdown
The server SHALL close database connection pools, Redis connections, MCP client connections, and SurrealDB connections during graceful shutdown.

#### Scenario: Database connections released
- **WHEN** graceful shutdown is initiated
- **THEN** all SQLx connection pool handles are dropped and connections returned before process exit

#### Scenario: MCP servers notified
- **WHEN** graceful shutdown is initiated
- **THEN** all stdio MCP server child processes receive shutdown signals
