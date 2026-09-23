## Purpose

Makes the supervised `uar-sidecar` process reachable only by the host that launched it: a per-launch token handed over stdin, checked on every request, never exposed outside the sidecar's memory, and a listener surface reduced to one loopback port.

## ADDED Requirements

### Requirement: Launch token arrives on stdin before any socket is bound
The sidecar SHALL read its launch token as the first line of standard input before it binds any listening socket. The line SHALL be exactly 64 lowercase hexadecimal characters (256 bits), optionally followed by `\r`, then `\n`. The sidecar SHALL NOT accept the token from environment variables, command-line arguments, configuration files or any other file. If standard input reaches end-of-file before a complete line, or the line is malformed, the sidecar SHALL exit with a non-zero status without binding a socket, without printing `READY`, and without echoing the received bytes to any output.

#### Scenario: Valid token line
- **WHEN** the host starts the sidecar and writes a 64-character lowercase hex line followed by a newline to its stdin
- **THEN** the sidecar binds `127.0.0.1` on an ephemeral port and prints exactly one `READY:{port}` line after the HTTP application is initialized

#### Scenario: Stdin closed before a token
- **WHEN** the host starts the sidecar and closes stdin without writing a line
- **THEN** the sidecar exits with a non-zero status, no `READY` line is printed, and no TCP port was opened by the process

#### Scenario: Malformed token line
- **WHEN** the first stdin line is shorter or longer than 64 characters, contains a non-hex or uppercase character, or is empty
- **THEN** the sidecar exits with a non-zero status, prints no `READY` line, and neither stdout nor stderr contains the received line

#### Scenario: Token supplied only through the environment
- **WHEN** a variable that looks like a token is present in the sidecar's environment and stdin is closed without a line
- **THEN** the sidecar exits with a non-zero status exactly as if no token were available

#### Scenario: Stdin EOF after the token still ends the process
- **WHEN** the sidecar has read its token, printed `READY`, and the host later closes stdin
- **THEN** the sidecar shuts down as it does today

### Requirement: Every sidecar request carries the launch token
The sidecar SHALL reject with HTTP 401 every request to its port that lacks `Authorization: Bearer <token>` with the exact launch token of the running process. The check SHALL run before routing, authentication middleware, rate limiting and handlers, and SHALL apply to every path: API routes, OpenAI- and Anthropic-compatible routes, SSE streams, MCP endpoints, `/health`, `/healthz`, `/readyz`, `/metrics`, and paths that match no route. The comparison SHALL take time independent of where the supplied value first differs. After a successful check the sidecar SHALL remove the `Authorization` header so no inner layer, handler, trace or log receives it.

#### Scenario: Missing token
- **WHEN** a client sends any request to the sidecar port without an `Authorization` header
- **THEN** the response is 401 and the request reaches no handler

#### Scenario: Wrong token
- **WHEN** a client sends a request with `Authorization: Bearer` followed by any value other than the launch token
- **THEN** the response is 401

#### Scenario: Previous launch's token
- **WHEN** the sidecar is restarted with a new token and a client presents the token of the previous launch
- **THEN** the response is 401

#### Scenario: SSE stream without the token
- **WHEN** a client opens `GET /api/uar/runs/{id}/stream` or `GET /api/uar/sync/stream` without the token
- **THEN** the response is 401 and no event bytes are written

#### Scenario: Unknown path without the token
- **WHEN** a client requests a path that matches no route and presents no token
- **THEN** the response is 401, not 404

#### Scenario: Valid token
- **WHEN** a client presents the launch token with an allowed `Host` and no `Origin`
- **THEN** the request is handled exactly as it would be by the same route today

### Requirement: Sidecar rejects foreign Host and any Origin
The sidecar SHALL reject with HTTP 403 any request whose authority (the `Host` header, or the request-target authority when no `Host` header is present) is not exactly `127.0.0.1:<port>` or `localhost:<port>`, where `<port>` is the bound port, with `localhost` matched case-insensitively. It SHALL reject with HTTP 403 any request that carries an `Origin` header, whatever its value, including `null`. It SHALL NOT send any `Access-Control-*` response header. Rejection responses SHALL NOT echo request header values.

#### Scenario: Foreign Host with a valid token
- **WHEN** a client presents the launch token with `Host: attacker.example`
- **THEN** the response is 403

#### Scenario: Host without the bound port
- **WHEN** a client presents the launch token with `Host: 127.0.0.1` or `Host: localhost:1`
- **THEN** the response is 403

#### Scenario: Any Origin with a valid token
- **WHEN** a client presents the launch token and an allowed `Host` together with `Origin: http://127.0.0.1:<port>`, `Origin: https://example.com` or `Origin: null`
- **THEN** the response is 403

#### Scenario: CORS preflight
- **WHEN** a client sends `OPTIONS` with `Origin` and `Access-Control-Request-Method` headers
- **THEN** the response is 403 and carries no `Access-Control-*` header

#### Scenario: No CORS headers on accepted requests
- **WHEN** an authorized request succeeds on any route
- **THEN** the response carries no `Access-Control-*` header

### Requirement: Sidecar mode exposes one loopback listener and no operator UI
In sidecar mode the process SHALL listen only on the `127.0.0.1` ephemeral port it reported in `READY`. It SHALL NOT bind the IPv6 companion listener, the A2A gRPC listener or any other port, and SHALL NOT serve the operator single-page application or other static files. Standalone UAR behavior SHALL be unchanged.

#### Scenario: Only the reported port is listening
- **WHEN** the sidecar has printed `READY:{port}`
- **THEN** the process owns exactly one listening TCP socket, bound to `127.0.0.1:{port}`

#### Scenario: Operator UI is not served
- **WHEN** an authorized client requests `/`, `/admin`, `/assets/index.js` or `/favicon.svg`
- **THEN** the response is 404 and contains no SPA content

#### Scenario: Standalone server unchanged
- **WHEN** the standalone `universal-agent-runtime` binary starts with its usual configuration
- **THEN** it binds its companion and A2A listeners and serves the SPA as before

### Requirement: Sidecar mode keeps tool governance mandatory
When the sidecar is authenticated by a launch token, the governance authority SHALL treat the process as authenticated and SHALL report governance as `Required` with reason `host_token_required`. A persisted `governance.enabled = false` SHALL be normalized to `true` at boot, run-policy tool denial and risk-based approval SHALL apply to every tool call, and an attempt to save `governance.enabled = false` SHALL be rejected.

#### Scenario: Fresh sidecar data directory
- **WHEN** the sidecar starts on a data directory with no persisted `governance.enabled`
- **THEN** governance status reports `required` with reason `host_token_required`, and `governance.enabled` is seeded `true`

#### Scenario: Data directory previously set to Off
- **WHEN** the sidecar starts on a data directory whose persisted `governance.enabled` is `false`
- **THEN** the value is normalized to `true` and governance is enforced

#### Scenario: Run policy denial is enforced
- **WHEN** a run started through the sidecar denies a tool in its run policy and the model calls that tool
- **THEN** the tool is not executed and the run records the denial

### Requirement: The launch token never leaves sidecar memory
The sidecar SHALL NOT place the launch token in its own command line or environment, in any child process's command line, environment or standard input, in any file, in stdout or stderr, in logs, traces or metrics, or in any HTTP response. Every child process the sidecar spawns SHALL receive a null standard input unless its protocol requires a pipe, and stdio MCP server children SHALL receive only an allowlisted launch environment plus the variables their definition declares.

#### Scenario: Process inspection of the sidecar and its children
- **WHEN** the sidecar runs with a configured stdio MCP server child and a terminal-tool child, and a same-user inspector reads each process's command line and environment
- **THEN** the token appears in none of them

#### Scenario: Output and files
- **WHEN** a test drives authorized, unauthorized, bad-Host, Origin, SSE and error requests and then stops the sidecar
- **THEN** a byte search for the token over captured stdout, stderr, every file under the sidecar's data and log directories, and every response body finds nothing

#### Scenario: Child cannot read sidecar stdin
- **WHEN** a child process spawned by the sidecar reads its standard input
- **THEN** it reads end-of-file immediately (or its protocol pipe), never bytes the host wrote to the sidecar

#### Scenario: Undeclared parent variables are not inherited by stdio MCP children
- **WHEN** the sidecar's environment contains a sentinel variable that the MCP server definition does not declare and that is not on the launch allowlist
- **THEN** the stdio MCP child's environment does not contain it

### Requirement: Child processes get a restricted environment and stdin in every mode
In standalone UAR and in sidecar mode alike, every stdio MCP server child SHALL receive only the launch allowlist environment plus the variables its definition declares, whichever spawn path starts it, and SHALL use its standard input only as its MCP protocol pipe. Every other child process UAR spawns — terminal-tool commands and provisioning commands — SHALL receive a null standard input. No child SHALL read the standard input of the UAR process.

#### Scenario: Standalone stdio MCP server and an undeclared variable
- **WHEN** standalone UAR runs with a sentinel variable in its environment that a stdio MCP server's definition does not declare, and starts that server through any spawn path
- **THEN** the server's environment contains its declared variables and the allowlisted keys, and does not contain the sentinel

#### Scenario: Standalone terminal command reads stdin
- **WHEN** standalone UAR's terminal tool runs a command that reads its standard input while the UAR process's stdin holds bytes
- **THEN** the command reads end-of-file immediately

### Requirement: Hosts can read the runtime's version and capabilities
UAR SHALL serve `GET /api/uar/capabilities` with the runtime version, the AG-UI profile id and revision of its runs stream, and a sorted, duplicate-free list of capability names. A name SHALL be listed only when the running process implements the behaviour it names, and SHALL come from a closed vocabulary: `run_scoped_credentials`, `run_scoped_mcp_servers`, `host_history`, `working_directory`, `reasoning_effort`, `session_principal`, `agui_stream_fidelity`, `secrets_at_rest`, `ingest_scoped_credentials`. The response SHALL NOT contain configuration values, paths, key material or any secret. In sidecar mode the route SHALL require the launch token, `Host` and `Origin` rules exactly like every other route; in standalone mode it SHALL follow the normal authentication rules and SHALL NOT be exempt from them.

#### Scenario: Host reads capabilities at startup
- **WHEN** the host sends `GET /api/uar/capabilities` to the sidecar with the launch token
- **THEN** the response is 200 with `uar_version` equal to the package version, `agui.profile` `uar.agui/1`, the current `agui.profile_revision`, and a sorted list of capability names from the vocabulary

#### Scenario: Capabilities without the token
- **WHEN** a client requests `GET /api/uar/capabilities` from the sidecar without the launch token
- **THEN** the response is 401 and the body contains no version or capability name

### Requirement: The launch token authenticates the host, not a session
The launch token SHALL establish only that a request comes from the host that launched the sidecar. It SHALL NOT establish a user, tenant or session identity: with no other credential, an authorized sidecar request SHALL resolve to the same anonymous principal it resolves to today. Per-session identity SHALL come only from a separate host-asserted principal mechanism.

#### Scenario: Authorized request without a session principal
- **WHEN** an authorized sidecar request carries no other credential
- **THEN** handlers see the anonymous principal, exactly as with JWT disabled today

### Requirement: Sidecar default logging carries no conversation content
In sidecar mode, when `RUST_LOG` is not set, the default log filter SHALL be `info`. At that level no log line SHALL contain prompt or message content, tool-call arguments, tool results or credentials. An explicit `RUST_LOG` value SHALL remain authoritative.

#### Scenario: Canary in prompt and tool arguments
- **WHEN** the sidecar runs with `RUST_LOG` unset and serves a run whose prompt contains a canary string and whose model response calls a tool with the canary in its arguments
- **THEN** no line on the sidecar's stdout, stderr or log file contains the canary

#### Scenario: Operator opts into debug logging
- **WHEN** the host starts the sidecar with `RUST_LOG=universal_agent_runtime=debug`
- **THEN** the sidecar honors that filter

### Requirement: Sidecar mode disables cross-session shared features
In sidecar mode the process SHALL run with skill evolution disabled, the UAR memory system disabled, and an empty global MCP server registry, regardless of configuration files or persisted settings. The sidecar SHALL NOT load a file-based MCP configuration or persisted global MCP server definitions, and SHALL reject requests that create, update or delete global MCP server definitions or enable skill evolution or the memory system. The memory system MAY be re-enabled in sidecar mode only by a change that scopes it per host-asserted session principal.

#### Scenario: Configuration asks for the features
- **WHEN** the sidecar starts with a configuration file that sets `skill_evolution.enabled: true` and `memory.enabled: true`, an `mcp.json` in its working directory, and persisted `mcp.servers` entries
- **THEN** no skill evolution task runs after a run with many tool calls, no memory service is constructed, and the global MCP server list is empty

#### Scenario: Global MCP mutation is refused
- **WHEN** an authorized client sends `PUT /api/uar/mcp/servers/{name}` or `DELETE /api/uar/mcp/servers/{name}` to the sidecar
- **THEN** the request is rejected with a 409 status and an error code naming sidecar mode, and nothing is persisted or registered

#### Scenario: Enabling a disabled feature through settings is refused
- **WHEN** an authorized client tries to save `skill_evolution.enabled` or `memory.enabled` as `true` through the settings API of the sidecar
- **THEN** the update is rejected and the effective value stays `false`
