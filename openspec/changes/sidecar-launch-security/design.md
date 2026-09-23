## Context

See `proposal.md` for why. The facts that shape the design:

- `src/bin/uar-sidecar.rs` does all process-environment writes in a synchronous bootstrap before the Tokio runtime exists (`:76-124`), binds `127.0.0.1:0` there (`:112`), prints `READY:{port}` after the server signals readiness (`:199-205`), and exits on stdin EOF (`:209-221`). It already reads stdin, so stdin is the natural token channel. `UAR_SIDECAR=1` is set (`:98-100`) but nothing in `src/` reads it; the server does not know it is a sidecar today.
- `start_server_sidecar` (`src/server.rs:1998-2016`) is called only by the sidecar binary. The live test harness uses `start_server_sidecar_with_listeners_and_shutdowns` (`tests/integration/live/harness.rs:24,564`) and must keep working.
- Router assembly: routes, then `auth_middleware` (`server.rs:1694-1697`), governance, trace and span layers (`:1697-1717`), the ACP router nested with its own auth layer (`:1721-1732`), then body limit, timeout, rate limit, CORS (`:1803`) and metrics layers (`:1780-1816`). Layers added last run first. A guard that must cover every route, including the nested ACP service and the SPA fallback, has to be the last layer added.
- `auth_middleware` exempts `/health`, `/healthz`, `/readyz`, `/metrics` (`src/uar/security/middleware.rs:94-97`) and treats any unverifiable bearer as anonymous when JWT is off (`:58-64`).
- The governance authority records "installed authentication" as one boolean (`src/uar/governance/runtime_control.rs:357-361`) and derives optional-governance eligibility from host literal, that boolean, and loopback bound addresses (`:440-459`). Preference planning normalizes a persisted `false` to `true` when governance may not be disabled (`:482-488`).
- The A2A gRPC listener is declared and bound unconditionally (`server.rs:510-533`); only the `info!` line and the serve task are feature-gated. The companion listener is bound whenever the host parses as an IP (`server.rs:500-508,1960-1990`).
- Children: the projected-MCP snapshot path already spawns stdio MCP servers with `env_clear()` plus an allowlist (`src/mcp/registry.rs:678-684,803-832`). The older paths (`registry.rs:616-633`, `registry.rs:1271-1283`, `src/mcp/stdio_client.rs:34-45`) inherit the full parent environment. `terminal_exec` (`src/uar/tools/terminal_exec.rs:224-238`) and provisioning `.status()` calls (`src/uar/orchestrator/provisioning.rs:390,412,427,499,595`) inherit the parent's stdin.
- A constant-time byte comparison helper already exists: `crate::config::secret_value_matches` (`src/config.rs:451-467`). `secrecy` is a direct dependency (`Cargo.toml:505`).
- Logging: the default filter is `info,universal_agent_runtime=debug` (`src/uar/telemetry/mod.rs:68-69`); debug lines print full LLM messages and tool arguments (`src/llm/orchestrator.rs:1662-1668,2066-2071`).

## Goals / Non-Goals

**Goals:**
- Only the launching host can use the sidecar port (D4 request controls), implemented and tested in Rust/Axum inside UAR.
- The token exists only in the host's and the sidecar's memory.
- The sidecar's defaults stop mixing one session's content into shared logs, skills, memory or global tool registries.
- Governance stays enforced so D1's run-policy denials and approvals hold.

**Non-Goals:**
- the-boss's own MCP endpoints and their token (D4's UAR→the-boss token). Their Host/Origin/token checks live in the-boss (TypeScript MCP SDK ≥ 1.24.0) and are tested there (D1 falsifier 5). UAR only carries that token inside run-scoped MCP definitions, which is a separate change.
- Per-session identity (`sidecar-session-principal`), run-scoped credentials and run-scoped MCP servers (D1 changes).
- Moving the transport to a Unix socket or named pipe. That happens only on a reachability failure (see Decision 10).
- Defending against a process running as the same OS user with debugger rights. It can read the sidecar's memory. The threat model is other users' processes, same-user processes without ptrace rights that only reach the port, and web pages (DNS rebinding, cross-origin requests).

## Decisions

### 1. Token on the first stdin line, read before binding, no timeout
`prepare_sidecar_process` finishes its environment writes, then reads one line from stdin with a blocking read, then binds. The line must match `^[0-9a-f]{64}\r?\n$`. On EOF or mismatch the process prints a fixed message (no input echo) to stderr and exits with status 2. The token is decoded into a `secrecy::SecretString` (zeroized on drop) inside a `SidecarLaunchToken` type with no `Display`, no `Serialize`, and a redacting `Debug`.

- No read timeout. EOF already covers a dead host, and the-boss applies its own readiness timeout and kills the child. A timeout would need a helper thread, and a live helper thread during the later `set_var("UAR_SERVER__PORT")` (`uar-sidecar.rs:119-121`) would break the bootstrap's single-thread safety argument.
- Standard-library stdin buffers input. Bytes after the first line stay in that buffer; the existing EOF watcher (`uar-sidecar.rs:209-221`) uses `tokio::io::stdin`, which is expected to wrap the same standard-library handle, and EOF on a pipe is sticky either way. Neither point was verified this session; the EOF-after-READY test is the proof.
- Alternatives rejected: environment variable or argv (readable by same-user process inspection; D4 forbids); a file (persists on disk); a pipe on another file descriptor (not portable to Windows).

### 2. One outermost guard layer, enabled only by the token
A new module `src/uar/security/sidecar_guard.rs` holds `SidecarGuard { token, port }` and an Axum `from_fn` middleware. `start_server_sidecar` gains a required `SidecarLaunchToken` argument and threads an `Option<SidecarGuard>` into `run_server_with_listener`; `None` everywhere else, so standalone and harness entry points are unchanged. The guard is added with `tower::util::option_layer` (or equivalent) as the very last layer, after the metrics layer at `server.rs:1804-1815`, so it runs before everything, including ACP and the SPA fallback.

Order inside the guard: authority check → `Origin` check → token check. Host and Origin first means a DNS-rebinding page is refused without learning anything from token timing. Responses: 403 for Host/Origin, 401 for token; fixed JSON bodies; no header values echoed. Logs record only a reason code (`missing_token`, `bad_token`, `bad_host`, `origin_present`) at `debug`. The token comparison uses `secret_value_matches`. On success the guard removes `Authorization` before calling the inner service.

Authority: the `Host` header, or for HTTP/2 the URI authority. Accepted: `127.0.0.1:{port}`, `localhost:{port}` (ASCII case-insensitive). Nothing else, not `[::1]`, not a missing port.

Alternatives rejected: extending `auth_middleware` (it runs inside rate limiting and not in front of ACP or health routes, and its exemptions exist for Kubernetes probes); a JWT signed by the host (more machinery, same property).

### 3. No CORS layer in sidecar mode
`build_permissive_cors_layer()` (`server.rs:2232-2236`) is replaced by an optional layer that is absent when the guard is present. The guard already rejects every request with `Origin`, but the absence of `Access-Control-*` headers on accepted responses is asserted separately, so removal is required and tested, not assumed.

### 4. Listener surface
With a guard present: skip `bind_companion_listener` and its `declare_ingress("companion-http")` (`server.rs:500-508`); skip `declare_ingress("a2a-grpc")` and the bind (`:510-533`), and the gRPC serve task (`:1818-1854`) is not spawned; skip the SPA routes and `fallback_service` (`:1680-1692`), which then fall through to the existing `/api/{*path}` 404 handler or Axum's default 404. The governance inventory therefore holds only `primary-http`.

### 5. Governance: record host-token authentication, not an ingress change
The `SocketAddr`-keyed registration (`runtime_control.rs:381-407,453`) needs **no** change: in sidecar mode it holds one address, `127.0.0.1:<port>`, which is loopback. The needed change is in authentication recording. Add `record_host_token_authentication()` to `GovernanceMutationHandle`, set a `host_token_required` flag in state, add `GovernanceStatusReason::HostTokenRequired` (serialized `host_token_required`), insert it in `seal_ingress_inventory`, and include it in `validate()`'s mandatory reasons (`:79-89`). `jwt_required` in the snapshot stays the true JWT value, so the status does not lie about JWT.

The frontend reason vocabulary (`frontend/src/features/settings/...governance*`) is not served by the sidecar, but its type must accept the new code so the standalone UI does not break if it ever sees one; this is a one-line union addition covered by the existing governance-status test file.

Alternative rejected: pass `true` to `record_installed_authentication` — the status would report `jwt_required: true` and reason `jwt_required`, which is false.

Known side effect: on a data directory shared with a standalone UAR that the operator set to Off, the sidecar normalizes `governance.enabled` to `true` (`runtime_control.rs:485`). the-boss gives the sidecar its own data directory, so this does not arise in the intended deployment.

### 6. Children: null stdin everywhere, allowlisted environment for stdio MCP
- `terminal_exec.rs:224-238` and every provisioning `.status()` call get `.stdin(Stdio::null())`. This applies in standalone mode too; a tool shell reading the server's stdin is never intended. This is a behavior change for standalone UAR only in that a terminal-tool command can no longer read the server process's stdin.
- The three legacy stdio MCP spawn paths (`registry.rs:616-633`, `registry.rs:1271-1283`, `stdio_client.rs:34-45`) call `env_clear()` and apply the same allowlist as `snapshot_child_environment` (`registry.rs:807-816`: `PATH`, `PATHEXT`, `SYSTEMROOT`, `WINDIR`, `COMSPEC`, `TMPDIR`, `TEMP`, `TMP`) plus the definition's declared, expanded variables. The allowlist is extracted into one shared function so the paths cannot drift. This also applies to standalone mode, because stripping undeclared parent variables (such as `CREDENTIAL_ENCRYPTION_KEY` or `UAR_SECURITY__JWT_SECRET`) from MCP children is correct everywhere. **This is a behavior change**: a standalone stdio MCP server that relied on an undeclared inherited variable (for example `HOME`) must now declare it. `HOME`/`USERPROFILE` are deliberately not on the allowlist; Open Question 1 covers it.
- *Operator decision 2026-09-23: approved* — stdio MCP children get the allowlisted environment, and children whose protocol does not use stdin get null stdin, in standalone UAR as well as in sidecar mode. The behaviour change for standalone users who relied on undeclared inherited variables is accepted. A stdio MCP child's stdin remains its MCP protocol pipe (the protocol runs over it); what the rule forbids is any child reading the UAR process's own stdin.
- The token itself is never in the environment, so environment sanitization is defense in depth for other secrets, not the token's primary protection.

### 7. Sidecar default logging
`uar::telemetry::init` gains a `default_directive: &str` argument; `main.rs` passes today's `"info,universal_agent_runtime=debug"`, the sidecar passes `"info"`. `RUST_LOG` still wins (`EnvFilter::try_from_default_env`). The two debug lines at `orchestrator.rs:1662-1668,2066-2071` stay at `debug` — changing them alters standalone debugging. The canary test (task 1.7) is the proof that no `info`-level line carries content; if it fails, the offending line moves to `debug`/`trace` as part of this change.

### 8. Cross-session shared features off in sidecar mode
The bootstrap adds environment overrides next to the existing ones (`uar-sidecar.rs:97-106`): `UAR_SKILL_EVOLUTION__ENABLED=false`, `UAR_MEMORY__ENABLED=false`. The env layer overrides configuration files, as the existing `UAR_SERVER__HOST` override relies on. In the server, a guard-present flag skips `mcp.json` loading (`server.rs:800-812`) and `mcp_admin::hydrate_registry` (`server.rs:1046`); global MCP mutation handlers (`src/uar/api/mcp_admin.rs:185-250`, `src/uar/admin/mcp.rs:118-180`) return 409 `sidecar_mode_global_mcp_disabled`; the settings write path rejects `true` for `skill_evolution.enabled` and `memory.enabled` in sidecar mode. `sidecar-session-principal` re-enables memory once it scopes it per session.

### 9. Identity
The launch token authenticates the host. It is not a principal. After the guard strips `Authorization`, `auth_middleware` resolves the anonymous principal (`middleware.rs:13-27,36-42`) exactly as today. Everything keyed by `user_id` (credentials, conversation policies, memory) therefore shares one `anonymous` identity across all the-boss sessions until `sidecar-session-principal` lands. That is the reason memory is forced off (Decision 8).

### 10. Which failure moves the transport (review round 3, warning 4)
Failures are split by remedy:
- **Header or token validation failures** (a request with a bad token, bad Host or any Origin is accepted): fix the guard. Transport unchanged.
- **Token handling failures** (token found in argv, env, logs, files, responses, child processes): fix the leak site. Transport unchanged.
- **Reachability failures** (a documented, reproduced way for another user's process or a web page to complete an authorized request despite the guard, for example a proxy or rebinding path that forwards the token): move to a Unix socket / Windows named pipe with an OS ACL. Only this class reopens D4's transport choice.

### 11. Process inspection per platform (review round 3, warning 7)
The argv/env test uses the `sysinfo` crate (already resolved at 0.37.2 in `Cargo.lock`; added as a dev-dependency at that version) to read `cmd()` and `environ()` of the sidecar and each child PID, found by walking the process table from the sidecar's PID:
- Linux: `/proc/<pid>/cmdline` and `/proc/<pid>/environ`.
- macOS (x64, arm64): `sysctl(KERN_PROCARGS2)`, same user.
- Windows x64 and ARM64: the process PEB via `NtQueryInformationProcess`/`ReadProcessMemory`, same user.
Before asserting absence, each test proves it can read a sentinel: it launches the sidecar with a known non-secret variable `UAR_PROBE_SENTINEL=<random>` and a known argv marker and asserts both are visible for the sidecar and for each child that is supposed to inherit them. If the sentinel cannot be read, the test fails; it never passes vacuously. That the `sysinfo` implementation reads environments on all four OS/arch combinations is not verified this session; the sentinel check is what makes that visible.

### 12. A capabilities endpoint for the host's version gate
No version or capability route exists today. `/health` and `/healthz` return `{"status":"ok"}` (`src/server.rs:1407-1408`, `:2420-2422`); the package version is exposed only as `info.version` in the OpenAPI document (`src/uar/api/openapi.rs:16`), and that route is compiled only with the `api-docs` feature (`src/server.rs:1401-1405`). the-boss's D2 version gate needs a stable answer to "what does this sidecar support" before it offers UAR agents.

`GET /api/uar/capabilities` returns:
```json
{
  "uar_version": "<CARGO_PKG_VERSION>",
  "agui": { "profile": "uar.agui/1", "profile_revision": 1 },
  "capabilities": ["reasoning_effort", "run_scoped_credentials", "..."]
}
```
- `uar_version` is `env!("CARGO_PKG_VERSION")`, as the MCP servers already report (`src/uar/mcp_server.rs:311`).
- `agui.profile` is the profile id in `docs/protocols/ag-ui-profile.md:1,13`. `agui.profile_revision` is `1` until `agui-runs-stream-fidelity` lands and sets `RUN_STARTED.profileRevision: 2`; both read one constant so they cannot drift.
- `capabilities` is a sorted, duplicate-free list of names from a closed vocabulary. A name is listed only when the running binary implements that behaviour. Each owning change adds its name in the same commit as the behaviour:

| Flag | Owning change | Meaning |
|---|---|---|
| `run_scoped_credentials` | `run-scoped-credentials-and-mcp-servers` | `run_credentials` with provider kind, base URL and key |
| `run_scoped_mcp_servers` | `run-scoped-credentials-and-mcp-servers` | `mcp_servers` on create, resume and continuation |
| `host_history` | `run-request-host-context` | `history` seeds an empty session |
| `working_directory` | `run-request-host-context` | `working_directory` on create |
| `reasoning_effort` | `run-request-host-context` | `reasoning_effort` on create |
| `session_principal` | `sidecar-session-principal` | `X-UAR-Principal` on launch-token requests; run and session eviction |
| `agui_stream_fidelity` | `agui-runs-stream-fidelity` | message boundaries, per-step ids, tool-result outcome, `uar.stream.lagged` and `uar.stream.resync_required` |
| `secrets_at_rest` | `encrypt-persisted-secrets` | sealed secrets and `GET /api/uar/secrets/status` |
| `ingest_scoped_credentials` | `agentic-chunking` | `model_credential` on document upload for agentic chunking; chunking fallback reported on the document |

None of those changes named a flag before this list. `run-request-host-context` and `agui-runs-stream-fidelity` do not yet carry a task to add theirs; the coordinator must add one. This change adds no flag of its own: a host that can call the route already holds a launch token.

The handler lives in a new `src/uar/api/capabilities.rs` and is mounted beside `/health` (`src/server.rs:1407-1415`). It is not added to the auth middleware's exemptions (`src/uar/security/middleware.rs:95`) or the rate limiter's (`src/uar/security/rate_limit.rs:60-61`), and it is not under `/.well-known`. In sidecar mode the guard (Decision 2) covers it like every other route: no token, 401. In standalone mode it follows the normal authentication rules. The body carries no configuration value, path, key source or secret.
- *Alternative rejected:* add `version` to `/health`. `/health` is exempt from authentication in standalone mode and is a Kubernetes probe; a capability list does not belong there.
- *Alternative rejected:* derive capabilities from the OpenAPI document. It is feature-gated, and a field existing in a schema does not prove the behaviour behind it.

## Risks / Trade-offs

- [Standalone stdio MCP servers relied on inherited variables] → Decision 6 is a behavior change, accepted by the operator on 2026-09-23. Mitigation: list it in the change notes; the error surfaces as a connect failure for that server; the operator declares the variable in the server's `env`.
- [A capability flag listed before its behaviour works] → the flag is only as honest as the commit that adds it. Each owning change's contract tests assert its flag is listed; the capabilities test asserts the list is sorted, unique and drawn from the vocabulary above.
- [the-boss connects via `localhost` resolving to `::1`] → the companion is gone, connection refused. Mitigation: the-boss uses `127.0.0.1:<port>` literally; the host check still accepts `localhost:<port>` for clients that resolve to IPv4.
- [A route that bypasses the router, such as a raw hyper service] → not present today; the guard wraps the whole `Router`. The unknown-path and nested-ACP test cases detect a future bypass.
- [Token timing leak through length] → `secret_value_matches` XORs lengths and scans the longer input; both lengths are fixed at 64 for valid tokens.
- [Stdin buffering eats the EOF] → covered by the EOF-after-READY test.
- [`sysinfo` cannot read another process's environment on some platform] → sentinel control fails the test loudly (Decision 11).
- [Memory and skill evolution off reduce sidecar features] → intended until `sidecar-session-principal`. The capabilities route (Decision 12) does not list them as capabilities, so the-boss cannot offer them.
- [Log filter change hides useful sidecar diagnostics] → the host can set `RUST_LOG`; the default protects content.

## Migration Plan

1. Land the guard, token read, listener gating, governance reason and child hardening together; the sidecar protocol change is breaking, so the-boss's UAR driver must ship the token write in the same release pair.
2. Old the-boss builds that do not write a token get a sidecar that exits before `READY`; they fail closed, not open.
3. Rollback: revert the change; no data or schema migration is involved. A persisted `governance.enabled = true` written by the sidecar stays `true` after rollback, which is the safe value.

## Open Questions

1. Should `HOME` / `USERPROFILE` join the stdio MCP launch allowlist? Many Node/Python MCP servers need a home directory for caches. This does not change the spec (declared variables are always passed); it changes only the default list. Decide from the first real server that fails.
