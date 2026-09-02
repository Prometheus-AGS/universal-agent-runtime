<!-- mirror of openspec/changes/projected-mcp-runtime/proposal.md and specs/*/spec.md -->
# projected-mcp-runtime

Rank 7 of the codex-harness-comparative-analysis change set. Source: gap G7 and the MCP-state item of G11 in the phase `analysis.md`.

## Why

Skill-contributed MCP servers are respawned by `McpRegistry::from_config` for every run (`src/uar/runtime/manager.rs:1457`) with no connection reuse, paying up to 30 seconds of connect and list on the critical path (`src/mcp/registry.rs:36`, `:43`). `McpServerEntry::Stdio.sandboxed` is stored and echoed but never applied (`src/mcp/config.rs:21`; `registry.rs:423-428`). Tool exposure is all-or-nothing; there is no deferred loading or tool search, and every surveyed harness has moved to deferred loading (Claude Code reports an 85% token reduction). The generation-guarded reconnect slot (`registry.rs:64-161`, decision 2026-08-21) is the right foundation and is kept.

Codex separates immutable server definitions from live connections, defers startup only when a cached catalog already holds a model-visible tool (`codex-mcp/src/connection_manager.rs:258-259`, `:559-576`), reuses a connection only when config and credential identity match (`:94-129`), projects tools per step with `Direct`, `Deferred`, and `Hidden` exposure (`core/src/mcp_tool_exposure.rs:75-147`), and registers a search tool only when a deferred tool exists (`core/src/tools/spec_plan.rs:371-406`). `rmcp` stays at 3.1.x; the installed crate already carries the 2026-07-28 constant but negotiates 2025-11-25, and the spec move is its own later change. Codex paths are outside this repository; verbatim excerpts for the cited lines are in the phase `analysis.md` appendix "verified Codex excerpts".

## What changes

- `McpCatalog` (immutable definitions with source, authority, config hash, required or optional, auth state, sandbox policy) separated from `McpRuntimeManager` (live connections).
- Per-step projection: each `ResolvedStep` names the exact servers and tools; a lower-authority skill or child cannot replace a definition or weaken sandbox or network restrictions.
- Bindings cached by owner, config hash, auth identity, and environment; generation-based invalidation; single-flight refresh; a cancelled refresh leaves the entry dirty.
- Global servers keep eager startup; skill-contributed and child-only servers start lazily on first call when their cached catalog is complete, with `wait_until_ready` on the call path.
- Exposure: bounded eager set plus `Deferred` tools discoverable through a model-only `search_tools` tool that activates matches for the next step; `Hidden` for policy-omitted tools.
- Required-server failure aborts preflight with an actionable error; optional-server failure warns and omits.
- States `disabled`, `connecting`, `ready`, `auth_required`, `failed`, `shutting_down` emitted as normalized events and fed to the existing `set_mcp_server_status` recorder.
- Stdio sandbox: `sandboxed: true` either launches under an OS-native sandbox (evaluated in task 0.1 by porting Codex `sandboxing`) or is rejected at config load with a clear error. It is never silently inert.

## Scope

- `src/mcp/{registry.rs,config.rs,stdio_client.rs}`
- `src/uar/runtime/manager.rs` (MCP block `:1448-1470`, `:1554-1583`)
- `src/uar/api/mcp_admin.rs`
- `src/uar/telemetry/metrics.rs`
- new: `src/mcp/{catalog.rs,projection.rs,binding_cache.rs}`, `src/uar/runtime/native_skills/search_tools.rs`, optionally `src/sandbox/os_native/`
- tests: `tests/mcp_projection.rs`; extend the existing reconnect tests

Out of scope: rmcp version bump and MCP 2026-07-28 semantics; MCP Apps.

## Dependencies

fail-closed-tool-arguments (descriptor `exposure`); typed-turn-assembly (per-step projection). Can be developed against the descriptor alone and integrated when the step type lands.

## Verification

Tier 0 per edit; Tier 1 the new tests; Tier 2 at the boundary; local integration with a real stdio server for lazy start, reconnect, cancellation, and shutdown, recorded with commands and output.

## The uncomfortable thing

Porting an OS-native sandbox adds platform-specific code (Seatbelt, Landlock, bwrap) and a test matrix UAR does not have. If the port is out of budget, the honest fallback is to reject `sandboxed: true` at load time so no operator believes isolation exists when it does not.


## Spec delta: mcp-runtime-projection

## ADDED Requirements

### Requirement: Server definitions are separate from live connections
The runtime SHALL keep an immutable catalog of MCP server definitions carrying source, authority, configuration hash, required-or-optional status, authentication state, and sandbox policy, separate from live connections, and SHALL resolve each model step to an exact set of servers and tools from that catalog.

#### Scenario: Lower authority cannot override
- **WHEN** a skill or child scope declares a server whose name matches a globally configured definition
- **THEN** the global definition is used and the lower-authority declaration cannot weaken its sandbox or network policy

### Requirement: Connections are reused and refreshed safely
The runtime SHALL cache server bindings by owner, configuration hash, authentication identity, and environment, SHALL invalidate by generation, SHALL coalesce concurrent refreshes into one attempt, and SHALL leave a cancelled refresh dirty for the next request.

#### Scenario: Two runs share a skill server
- **WHEN** two runs in sequence use the same skill-declared server with unchanged configuration and credentials
- **THEN** the second run reuses the first run's connection

#### Scenario: Credential change
- **WHEN** the authentication identity for a server changes
- **THEN** the cached binding is not reused and a new connection is established

### Requirement: Startup is lazy where the catalog allows it
Globally configured servers SHALL keep their startup behavior; skill-contributed and child-only servers SHALL start on first tool call when their cached tool catalog is complete, and a call to a not-yet-ready server SHALL wait for readiness within the configured timeout.

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
