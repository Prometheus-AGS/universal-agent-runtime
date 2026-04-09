## ADDED Requirements

### Requirement: Sandboxed MCP Server Configuration

The `McpServerEntry` in `mcp.json` SHALL support an optional `"sandboxed": true` field. When this field is present and set to `true`, the MCP server process SHALL be launched inside a microsandbox VM instead of directly on the host.

#### Scenario: MCP server with sandboxed flag
- **WHEN** an MCP server entry in `mcp.json` includes `"sandboxed": true`
- **THEN** the system SHALL launch the server's command inside a sandbox VM rather than as a host child process.

#### Scenario: MCP server without sandboxed flag
- **WHEN** an MCP server entry in `mcp.json` does not include the `"sandboxed"` field or sets it to `false`
- **THEN** the system SHALL launch the server as a normal host child process, preserving existing behavior.

### Requirement: Stdio Transport Through Sandbox

When an MCP server runs inside a sandbox, the stdio transport SHALL still function correctly. The VM's stdin and stdout SHALL be piped to the MCP client so that JSON-RPC messages flow transparently between the host and the sandboxed server.

#### Scenario: JSON-RPC initialize handshake through sandbox
- **WHEN** a sandboxed MCP server starts and the MCP client sends an `initialize` request via stdin
- **THEN** the server SHALL receive the request inside the VM, process it, and return the response via stdout to the host MCP client.

#### Scenario: Tool invocation through sandboxed server
- **WHEN** the MCP client invokes a tool on a sandboxed MCP server via stdio
- **THEN** the tool call and response SHALL be transmitted correctly through the VM's piped stdin/stdout with no corruption or loss.

### Requirement: Filesystem Restriction

A sandboxed MCP server SHALL have filesystem access restricted to its designated working directory inside the VM. The server SHALL NOT be able to read or write files outside this directory.

#### Scenario: Server accesses files within working directory
- **WHEN** a sandboxed MCP server attempts to read a file within its `/workspace` directory
- **THEN** the read SHALL succeed normally.

#### Scenario: Server cannot access host filesystem
- **WHEN** a sandboxed MCP server attempts to read `/etc/passwd` or any path outside its working directory
- **THEN** the access SHALL be denied or the path SHALL not exist inside the sandbox.

### Requirement: Per-Server Network Configuration

Each sandboxed MCP server entry SHALL support an optional `"network"` field (boolean, default `false`) that controls whether the sandbox has outbound network access. This is independent of the global sandbox network setting.

#### Scenario: Sandboxed server with network enabled
- **WHEN** an MCP server entry specifies `"sandboxed": true` and `"network": true`
- **THEN** the sandbox SHALL allow outbound network access so the server can reach external APIs.

#### Scenario: Sandboxed server with network disabled
- **WHEN** an MCP server entry specifies `"sandboxed": true` and does not set `"network"` or sets it to `false`
- **THEN** the sandbox SHALL block all outbound network access from the server.

### Requirement: Backward Compatibility

Adding the `"sandboxed"` and `"network"` fields to `McpServerEntry` SHALL NOT affect existing MCP server configurations. Servers without these fields SHALL continue to work exactly as before.

#### Scenario: Existing mcp.json without sandbox fields
- **WHEN** an existing `mcp.json` file is loaded that contains no `"sandboxed"` or `"network"` fields
- **THEN** all MCP servers SHALL start as host child processes with no behavioral change.

#### Scenario: Mixed sandboxed and unsandboxed servers
- **WHEN** `mcp.json` contains one server with `"sandboxed": true` and another without
- **THEN** the sandboxed server SHALL run in a VM and the other SHALL run as a host process, both operating concurrently.
