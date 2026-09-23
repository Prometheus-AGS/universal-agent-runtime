## ADDED Requirements

### Requirement: Run-scoped host MCP servers are the run's whole MCP universe
`POST /api/uar/runs` SHALL accept an optional `mcp_servers` list of Streamable HTTP server definitions, each with a `name`, a `url`, and optional request `headers`. When the field is present, even as an empty list, the run SHALL be able to list and call tools only from those servers; global and skill-declared servers SHALL be outside the run's catalog, not merely filtered out of the model's view. Native tools SHALL remain subject to the run policy as today. A run policy that selects an MCP server not supplied in `mcp_servers` SHALL fail the request with HTTP 422 and code `mcp_server_not_run_scoped` before the run starts. A model call to a tool outside the run's catalog SHALL return a tool error and SHALL NOT reach any server.

#### Scenario: Global server present
- **WHEN** the global registry holds server `global-fs` and a run supplies only `boss-tools`
- **THEN** the tools offered to the model come only from `boss-tools` and permitted native tools, and `global-fs` receives no request from the run

#### Scenario: Model names a global tool anyway
- **WHEN** the model emits a call for a tool that belongs to `global-fs`
- **THEN** the call returns a tool error and `global-fs` receives nothing

#### Scenario: Policy selects a server the run did not supply
- **WHEN** the artifact's run policy selects `global-fs` and the run supplies `mcp_servers`
- **THEN** the response is 422 with `mcp_server_not_run_scoped` and no run starts

### Requirement: Run-scoped servers are validated and connected with their own headers
Each run-scoped server `name` SHALL be unique within the run and SHALL match `^[a-z0-9][a-z0-9_-]{0,62}$`. Each `url` SHALL use `http` or `https` and its host SHALL match the configured run-scoped MCP host allowlist, which SHALL default to loopback addresses and `localhost`. Header names `Host`, `Content-Length`, `Content-Type`, `Mcp-Session-Id` and `Mcp-Protocol-Version` SHALL be rejected. Invalid definitions SHALL be rejected with HTTP 422 and code `run_mcp_server_invalid` before any connection is attempted. UAR SHALL send the supplied headers on every request to that server, SHALL NOT follow redirects or use a proxy, and SHALL treat every header value as a secret under the same confinement as a run credential.

#### Scenario: Header sent on every request
- **WHEN** a run-scoped server is defined with `Authorization: Bearer <token>`
- **THEN** every HTTP request UAR makes to that server during the run carries that header

#### Scenario: Non-loopback URL
- **WHEN** a run-scoped server URL points to a host outside the allowlist
- **THEN** the response is 422 with `run_mcp_server_invalid` and no connection is attempted

### Requirement: Run-scoped servers are isolated per run and leave no residue
Run-scoped servers SHALL be connected for their root run only. They SHALL NOT be added to the global registry, the persisted MCP configuration, settings, or the shared binding cache, and SHALL NOT be reused by another run, even one in the same session with an identical definition. Child runs SHALL receive the parent's run-scoped bindings narrowed by child policy and SHALL NOT widen them. When the root run reaches a terminal state, UAR SHALL close every run-scoped connection, and later calls through any retained binding SHALL fail. Resume and A2UI continuation of a run that had run-scoped servers SHALL carry `mcp_servers` again, or be rejected with HTTP 422 and code `run_mcp_servers_required` without falling back to the global registry.

#### Scenario: Two sessions with same-named servers
- **WHEN** two concurrent runs in different sessions each supply a server named `boss-tools` at different URLs with different tokens
- **THEN** each run lists and calls only its own server, and each server receives requests only from its own run

#### Scenario: After the run ends
- **WHEN** a run with run-scoped servers finishes, errors or is cancelled
- **THEN** the MCP server list API names none of its servers, the persisted MCP configuration is byte-identical to before the run, and each server observes its connection closed

#### Scenario: Continuation without servers
- **WHEN** a host continues a run that had run-scoped servers and omits `mcp_servers`
- **THEN** the response is 422 with `run_mcp_servers_required` and no run starts
