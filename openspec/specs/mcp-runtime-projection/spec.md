# mcp-runtime-projection Specification

## Purpose

Define authority-preserving MCP catalogs, exact connection bindings, lifecycle and per-step tool projection.

## Requirements

### Requirement: Server definitions are separate from live connections
The runtime SHALL keep an immutable catalog of host-global and skill-contributed MCP server definitions carrying source, authority, configuration hash, required-or-optional status, authentication state, and sandbox policy, separate from live connections, and SHALL resolve each model step to an exact set of servers and tools from that catalog. Local delegated children SHALL receive only narrowed frozen bindings, and a remote UAR peer SHALL resolve its own verified root catalog instead of accepting a connection recipe from its caller.

#### Scenario: Lower authority cannot override
- **WHEN** a skill declares a server whose name matches a globally configured definition
- **THEN** the global definition is used and the lower-authority declaration cannot weaken its sandbox or network policy

#### Scenario: Delegation does not transfer connection recipes
- **WHEN** a parent delegates to a local child or an authenticated remote UAR peer
- **THEN** the local child receives only narrowed frozen bindings, while the remote peer resolves definitions and credentials from its own verified root host

### Requirement: Connections are reused and refreshed safely
The runtime SHALL cache server bindings by owner, configuration hash, authentication identity, and environment, SHALL invalidate by generation, SHALL coalesce concurrent refreshes into one attempt, and SHALL leave a cancelled refresh dirty for the next request.

#### Scenario: Two runs share a skill server
- **WHEN** two runs in sequence use the same skill-declared server with unchanged configuration and credentials
- **THEN** the second run reuses the first run's connection

#### Scenario: Credential change
- **WHEN** the authentication identity for a server changes
- **THEN** the cached binding is not reused and a new connection is established

### Requirement: Startup is lazy where the catalog allows it
Globally configured servers SHALL keep their startup behavior; skill-contributed servers SHALL start on first tool call when their cached tool catalog is complete, and a call to a not-yet-ready server SHALL wait for readiness within the configured timeout.

#### Scenario: First call to a dormant server
- **WHEN** the model calls a tool on a lazily started server
- **THEN** the server starts, the call waits for readiness, and the call completes within the tool timeout

### Requirement: Required and optional servers fail differently
A required server that fails to start SHALL abort run preflight with an actionable error; an optional server that fails SHALL produce a warning and its tools SHALL be omitted.

#### Scenario: Optional server down
- **WHEN** an optional server cannot connect
- **THEN** the run proceeds without its tools and a warning names the server

### Requirement: Tool exposure is bounded with search-based discovery
The runtime SHALL expose a bounded eager tool set, SHALL mark remaining tools as deferred, and SHALL provide a model-only search tool that activates matching deferred tools for the next step.

#### Scenario: Large tool population
- **WHEN** more tools are eligible than the eager bound
- **THEN** the model sees the eager set plus the search tool, and a search activates matches for the following step

### Requirement: Server state is observable without secrets
The runtime SHALL expose server states `disabled`, `connecting`, `ready`, `auth_required`, `failed`, and `shutting_down` as normalized events and metrics, and SHALL NOT include credentials in any state payload.

#### Scenario: Authentication required
- **WHEN** a server rejects a connection for missing or expired credentials
- **THEN** the state becomes `auth_required` and the event carries no token material

### Requirement: The sandbox flag is never inert
A stdio server configured with `sandboxed: true` SHALL be launched under an operating-system sandbox, or the configuration SHALL be rejected at load with an error stating that sandboxing is unavailable.

#### Scenario: Sandbox unavailable on this platform
- **WHEN** an operator sets `sandboxed: true` where no sandbox backend exists
- **THEN** configuration load fails with an error naming the server and the missing backend
