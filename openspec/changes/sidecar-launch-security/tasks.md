# Tasks — sidecar-launch-security

Contract tests come first. Each test in section 2 must be run once against the unmodified branch and **fail** (record the failure output under `openspec/changes/sidecar-launch-security/evidence/`), then pass after section 3. Tests marked *regression guard* pass before and after; they protect behavior that must not change. All tests run locally; none run in GitHub Actions (AGENTS.md "GitHub Actions policy").

Falsifier map:

| Source | Falsifier / finding | Tests |
|---|---|---|
| D4 falsifier 2 (header part) | no token, wrong token, previous launch's token, `Host: attacker.example`, any `Origin` rejected on every route incl. SSE | 2.6–2.12 |
| D4 falsifier 2 (confinement part) | token absent from argv/env of sidecar and children, stdout/stderr, logs, files | 2.19–2.23 |
| D4 review r3, warning 4 | split reachability vs. token-handling failures | design Decision 10; 2.6–2.23 are all non-reachability failures |
| D4 review r3, warning 5 | Rust/Axum Host/Origin validation stated and tested separately from the-boss's TypeScript SDK | 2.9–2.12 |
| D4 review r3, warning 7 | runnable per-platform process inspection with child coverage | 2.19 (sentinel control, Linux/macOS/Windows x64/Windows ARM64) |
| D4 decision "no `[::1]` twin, gRPC, operator UI" | listener surface | 2.13–2.15 |
| D1 approvals / `policy.tools.deny` | governance stays enforced | 2.16–2.18 |
| D1 falsifier 5 | the-boss MCP endpoint abuse | **not in UAR** — tested in the-boss |
| Coordinator inventory | logging, shared features, identity | 2.24–2.28 |
| Operator decision 2026-09-23 | child environment and stdin rules in standalone mode | 2.21, 2.22, 2.31 |
| D2 version gate (the-boss) | capabilities endpoint | 2.29, 2.30 |

## 1. Bootstrap

- [x] 1.1 Confirm `openspec validate sidecar-launch-security --strict` passes and record the output in `evidence/validate.txt`.
- [x] 1.2 Register this change in the-boss child phase `the-boss-universal-agent-runtime` progress (prometheus-skills-mini `.kbd-orchestrator/`), and in this repo's `.kbd-orchestrator/` if a UAR phase tracks it; verify both files name the change. — 2026-09-23: registered with `prometheus kbd change register --phase the-boss-universal-agent-runtime` (with the nine sibling changes); the child phase's `progress.json` names it. No UAR phase tracks this change (UAR's active KBD phase is `runtime-harness-gap-closure`, owned by a separate run), so the conditional second registration does not apply.
- [x] 1.3 Add `sysinfo = "=0.37.2"` as a dev-dependency (same version already in `Cargo.lock`, so no new lock entry) after checking `versions.toml` allows it; verify `Cargo.lock` gains no new package.
- [x] 1.4 Add a test-only probe binary `uar-env-probe` behind a new `test-probes` feature (never in release feature sets) that writes its argv, full environment and the result of one non-blocking stdin read as JSON to the path given as its first argument; verify `cargo build --bin uar-env-probe --features server,test-probes` produces it and the default build does not.
- [x] 1.5 Add a test helper `tests/support/sidecar_process.rs` that launches `env!("CARGO_BIN_EXE_uar-sidecar")` with a temp data dir, embedded SurrealKV, the `stub-llm` provider, an optional stdin token line, and captures stdout/stderr to files; verify it can launch the current sidecar and read `READY`.
- [x] 1.6 Operator acceptance of design Decision 6 in standalone mode: stdio MCP children get the allowlisted environment and non-protocol children get null stdin in standalone UAR as well as in sidecar mode; standalone users who relied on undeclared inherited variables must declare them.
  Operator decision 2026-09-23: approved — allowlisted environment and null stdin for stdio MCP child processes in standalone UAR as well as in sidecar mode, including the behaviour change for standalone users who relied on undeclared inherited variables.

## 2. Contract tests (write first; each fails before section 3 unless marked)

All in `tests/sidecar_launch_security.rs` unless another file is named.

Launch token:
- [x] 2.1 `launch_with_valid_token_prints_single_ready` — writes a 64-hex line; asserts exactly one `READY:{port}` line, `GET /readyz` with the token returns 200, and the same request without the token returns 401. (Fails before: the request without the token returns 200.)
- [x] 2.2 `launch_without_token_line_exits_nonzero_without_ready` — closes stdin immediately; asserts non-zero exit within 5 s and no `READY` on stdout.
- [x] 2.3 `launch_with_malformed_token_exits_nonzero_without_echo` — cases: 63 chars, 65 chars, uppercase hex, non-hex, empty line; asserts non-zero exit, no `READY`, and neither stdout nor stderr contains the sent line.
- [x] 2.4 `launch_ignores_token_shaped_environment_variable` — sets `UAR_SIDECAR_TOKEN=<64 hex>` and `UAR_TOKEN=<64 hex>` in the environment, closes stdin; asserts the same failure as 2.2.
- [x] 2.5 `stdin_eof_after_ready_exits_cleanly` — writes the token, waits for `READY`, closes stdin; asserts exit status 0 within the shutdown timeout. *Regression guard for the EOF contract after the token read.*

Request guard (D4 falsifier 2, header part):
- [x] 2.6 `every_route_rejects_missing_token` — route list = every path in `GET /api/openapi.json` (fetched with the token) with path parameters filled with dummy values, plus `/`, `/health`, `/healthz`, `/readyz`, `/metrics`, `/v1/models`, `/v1/chat/completions`, `/v1/messages`, `/mcp/uar`, the memory MCP path from config, `/api/uar/runs/{id}/stream`, `/api/uar/sync/stream`, `/acp` (if enabled) and `/no-such-path-{random}`; each with its documented method and no `Authorization`; asserts 401 for every entry.
- [x] 2.7 `every_route_rejects_wrong_token` — same list with a different random 64-hex token and with malformed schemes (`Basic`, `bearer` lowercase, empty); asserts 401.
- [x] 2.8 `previous_launch_token_is_rejected` — launch A with token T1, stop, launch B with T2; asserts T1 gets 401 on B for the full route list and T2 gets a non-401 on `/readyz`.
- [x] 2.9 `sse_streams_reject_without_token_and_write_no_events` — opens both SSE routes without the token and with a wrong token; asserts 401 status and that the body contains no `data:` or `event:` line.
- [x] 2.10 `foreign_host_rejected_with_valid_token` — `Host` values `attacker.example`, `attacker.example:{port}`, `127.0.0.1`, `localhost`, `localhost:1`, `[::1]:{port}`, `127.0.0.2:{port}`; asserts 403 on `/readyz`, `/api/openapi.json` and one SSE route, and that the body does not contain the Host value. Also asserts `LOCALHOST:{port}` is accepted.
- [x] 2.11 `any_origin_rejected_with_valid_token` — `Origin` values `null`, `http://127.0.0.1:{port}`, `http://localhost:{port}`, `https://example.com`; plus an `OPTIONS` preflight with `Access-Control-Request-Method: POST`; asserts 403 for all.
- [x] 2.12 `no_cors_headers_on_any_response` — over the full route list with a valid token and allowed Host, and over all rejected requests from 2.6–2.11; asserts no response header name starts with `access-control-`.

Listener surface:
- [x] 2.13 `sidecar_owns_exactly_one_listening_socket` — after `READY`, lists listening TCP sockets owned by the sidecar PID (Linux: `/proc/<pid>/fd` inodes joined with `/proc/net/tcp` and `/proc/net/tcp6`; macOS: `lsof -nP -a -p <pid> -iTCP -sTCP:LISTEN`; Windows: `Get-NetTCPConnection -State Listen -OwningProcess <pid>` via `powershell -NoProfile`); asserts exactly one, `127.0.0.1:{port}`.
- [x] 2.14 `operator_ui_not_served_in_sidecar_mode` — with a valid token requests `/`, `/threads`, `/admin`, `/admin/x`, `/assets/index.js`, `/favicon.svg`; asserts 404 and that no body contains `<html` or `<!doctype`.
- [x] 2.15 `standalone_server_keeps_companion_grpc_and_spa` — in-process standalone start via `start_server_with_listener`-backed entry; asserts companion bind attempt, gRPC listener and SPA index still served. *Regression guard.*

Governance:
- [x] 2.16 `sidecar_governance_required_with_host_token_reason` — fresh data dir; reads the governance status the settings API serves; asserts `effective_state = required`, reasons contain `host_token_required`, `jwt_required = false`, and persisted `governance.enabled = true`.
- [x] 2.17 `sidecar_normalizes_persisted_governance_off_and_refuses_off` — pre-seeds the data dir with `governance.enabled = false` (via a standalone boot that sets it Off), then boots the sidecar; asserts the value is now `true`, and that saving `false` is rejected with a validation error.
- [x] 2.18 `sidecar_run_policy_denied_tool_is_not_executed` — `stub-llm` returns a tool call for a tool listed in `uar.run_policy` `tools.deny`; asserts the tool was not executed and the run's events record a denial. Also asserts `standalone_loopback_without_jwt_remains_governance_optional` (*regression guard*, separate test fn of that name).

Token confinement (D4 falsifier 2, confinement part):
- [x] 2.19 `token_absent_from_argv_and_env_of_sidecar_and_children` — launches the sidecar with sentinel `UAR_PROBE_SENTINEL=<random>` in its environment and a sentinel in argv (`--config <path-containing-sentinel>`); drives a run in which `stub-llm` calls the terminal tool to execute `uar-env-probe`; walks the process tree from the sidecar PID with `sysinfo`; **first** asserts the sentinel is readable in the sidecar's env and argv (fails the test if not — no vacuous pass), then asserts the token bytes appear in no process's `cmd()` or `environ()`, and in no probe JSON. Runs on Linux, macOS, Windows x64 and Windows ARM64 (manual run on each; results recorded in `evidence/`).
- [x] 2.20 `token_absent_from_output_logs_files_and_responses` — sets `UAR_LOG_FILE` inside the temp dir; drives authorized, unauthorized, bad-Host, Origin, SSE, 404 and error requests; stops the sidecar; byte-searches captured stdout, stderr, every file under the data dir and log dir (recursively, including SurrealKV files), and every response body for the token (raw and as `Bearer <token>`); asserts no match.
- [x] 2.21 `stdio_mcp_children_get_allowlisted_env_on_every_spawn_path` (file `tests/mcp_child_environment.rs`) — sets sentinel `UAR_UNDECLARED_SENTINEL` in the test process; spawns `uar-env-probe` through each stdio spawn path (`connect_server`, the provisioning path at `registry.rs:1271`, and the snapshot path) with one declared variable; asserts the probe saw the declared variable and allowlisted keys only, and not the sentinel. (Fails before on the three legacy paths.) — Criterion amended 2026-09-23: `StdioMcpClient::spawn` (`src/mcp/stdio_client.rs`) is removed from this list because no module declares that file, so it is not compiled and has no runtime spawn path. If it is ever wired in, it must adopt the shared allowlist function from 3.6 and be added back here.
- [x] 2.22 `child_processes_read_eof_from_stdin` (file `tests/mcp_child_environment.rs`) — spawns `uar-env-probe` via the terminal tool executor and via one provisioning command path while the test process's stdin is a pipe holding bytes; asserts the probe read EOF (0 bytes).
- [x] 2.23 Manual evidence: force an abort of a running sidecar (send SIGABRT on macOS/Linux; `taskkill /F` is not a crash on Windows — use Windows Error Reporting's local dump by triggering `abort` through a debug-only hook if one exists, otherwise record "not tested"); search the OS crash-report directory for the token; record the result in `evidence/crash-report.md`. Not an automated test; stated as unverified until done. — 2026-09-23: done on macOS arm64 — SIGABRT produced a ReportCrash `.ips`; the token occurs 0 times in it and 0 times in stdout, stderr and the data directory. Linux and Windows recorded as not tested in `evidence/crash-report.md`.

Logging, shared features, identity (coordinator scope):
- [x] 2.24 `sidecar_default_logs_contain_no_prompt_or_tool_argument_canary` — `RUST_LOG` unset; prompt contains canary `CANARY-<random>`; `stub-llm` responds with a tool call whose arguments contain the canary; asserts no line in stdout, stderr or `UAR_LOG_FILE` contains the canary. (Fails before: `orchestrator.rs:1662-1668,2066-2071` log both at debug, and the default filter is debug for UAR.)
- [x] 2.25 `sidecar_honors_explicit_rust_log` — `RUST_LOG=universal_agent_runtime=debug`; asserts a known debug event appears. *Regression guard.*
- [x] 2.26 `sidecar_forces_skill_evolution_memory_and_global_mcp_off` — config file sets `skill_evolution.enabled: true`, `min_tool_calls: 1`, `memory.enabled: true`; `mcp.json` with one stdio server in the working dir; data dir pre-seeded with one `mcp.servers` entry; asserts after a run with two tool calls that no skill evolution event or skill was created, the memory service is absent (memory API reports disabled), and `GET /api/uar/mcp/servers` lists none and no MCP child process exists.
- [x] 2.27 `sidecar_rejects_global_mcp_mutation_and_feature_enable` — `PUT` and `DELETE /api/uar/mcp/servers/x` return 409 with code `sidecar_mode_global_mcp_disabled` and nothing is persisted; saving `skill_evolution.enabled=true` or `memory.enabled=true` through the settings API is rejected and the effective value stays `false`.
- [x] 2.28 `sidecar_authorized_request_is_anonymous_principal` — with a valid token, creates a record that stores its owner (first candidate: a thread via the threads API; the exact endpoint that exposes `user_id` is not confirmed this session) and asserts the owner is `anonymous`. *Regression guard for Decision 9; must keep passing until `sidecar-session-principal` changes it deliberately.*

Capabilities endpoint (design Decision 12):
- [x] 2.29 `capabilities_route_requires_launch_token_and_reports_version` — sidecar: `GET /api/uar/capabilities` without a token → 401 and the body contains neither the package version nor any vocabulary name; with a wrong token → 401; with a foreign `Host` or any `Origin` → 403; with the token → 200, `uar_version == env!("CARGO_PKG_VERSION")`, `agui.profile == "uar.agui/1"`, `agui.profile_revision` equals the constant the runs stream uses, and `capabilities` is sorted, unique and a subset of the Decision 12 vocabulary. Also add the route to the 2.6/2.7 route list. (Fails before: the route does not exist, so the token case returns 404.)
- [x] 2.30 `capabilities_route_is_not_auth_exempt_in_standalone` — standalone router with `security.jwt_required: true`: no bearer → 401; valid JWT → 200 with the same shape. Byte-search the 200 body for the configured data directory path, the JWT secret and every configured provider key; assert none appears. (Fails before: 404.)
- [x] 2.31 `standalone_stdio_mcp_child_env_and_terminal_stdin` (file `tests/mcp_child_environment.rs`) — standalone in-process server, no launch token, sentinel `UAR_UNDECLARED_SENTINEL` in the test environment: a configured stdio MCP server running `uar-env-probe` sees only allowlisted and declared variables; a terminal-tool command running `uar-env-probe` reads EOF from stdin while the test process's stdin holds bytes. *Proves the 2026-09-23 decision in standalone mode; 2.21 and 2.22 exercise the spawn functions directly.* (Fails before on the environment half.)

## 3. Implementation

- [x] 3.1 Stdin token read in `prepare_sidecar_process` after env writes and before bind; `SidecarLaunchToken` (SecretString, redacting Debug, no Display/Serialize); exit 2 with fixed message on EOF/malformed. Verify 2.2–2.5 pass.
- [x] 3.2 `src/uar/security/sidecar_guard.rs`: authority → Origin → token checks, header removal, reason-code logging; wire as the outermost optional layer; change `start_server_sidecar` to require the token. Verify 2.1, 2.6–2.11 pass and the live harness still starts.
- [x] 3.3 Make the CORS layer optional and absent with the guard. Verify 2.12.
- [x] 3.4 Gate companion listener, A2A gRPC declare/bind/serve, and SPA/static routes on guard absence. Verify 2.13–2.15.
- [x] 3.5 `record_host_token_authentication`, `GovernanceStatusReason::HostTokenRequired`, `validate()` update, frontend reason union addition. Verify 2.16–2.18 and the existing `runtime_control` and frontend governance-status tests.
- [x] 3.6 Shared launch-environment allowlist function; `env_clear()` + allowlist on the three legacy stdio MCP spawn paths; `Stdio::null()` stdin for terminal tool and provisioning commands. Verify 2.21, 2.22, then 2.19.
- [x] 3.7 `telemetry::init` default-directive parameter; sidecar passes `info`. Verify 2.24, 2.25; if 2.24 still fails, lower the offending info-level line and re-run.
- [x] 3.8 Sidecar overrides `UAR_SKILL_EVOLUTION__ENABLED=false`, `UAR_MEMORY__ENABLED=false`; skip `mcp.json` load and `mcp.servers` hydration with the guard; 409 on global MCP mutation in both `mcp_admin.rs` and `admin/mcp.rs`; settings write rejection for enabling the two features. Verify 2.26, 2.27.
- [x] 3.9 Verify 2.20 and 2.28 pass with everything wired.
- [x] 3.10 `src/uar/api/capabilities.rs`: the handler, the closed flag vocabulary as constants, the shared AG-UI profile-revision constant; mount at `/api/uar/capabilities` beside `/health` with no auth or rate-limit exemption (design Decision 12). The list starts with whichever owning changes have landed; it may be empty. Verify 2.29, 2.30, and that 2.6/2.7 include the route.
- [x] 3.11 Verify 2.31 with 3.6 wired.

## 4. Verification at the change boundary

- [x] 4.1 Run `tests/sidecar_launch_security.rs` and `tests/mcp_child_environment.rs` once on macOS; record output in `evidence/`.
- [x] 4.2 Run 2.13 and 2.19 on Linux, Windows x64 and Windows ARM64 (local machines, not GitHub Actions); record per-platform output or record the platform as unverified. — 2026-09-23: recorded in `evidence/platforms.md`: macOS arm64 passed; Linux x64, Windows x64 and Windows ARM64 **unverified** (no machine this session). The Windows rows remain open risks.
- [ ] 4.3 Run the existing governance, middleware and live-harness tests once to confirm no regression; record output.
- [x] 4.4 Update the-boss's UAR driver contract note: write the token line first, send it on every request, connect to `127.0.0.1`. Verify the the-boss change references this change by name. — 2026-09-23: the contract note is prometheus-skills-mini `.kbd-orchestrator/phases/the-boss-shipping-and-settings/children/the-boss-universal-agent-runtime/uar-driver-contract.md`, section "`sidecar-launch-security` (UAR commit `e44846af`)". the-boss's driver code does not exist yet (step 3); the requirement that it cite this change by name is recorded in that note and is verified when the driver lands.
- [x] 4.5 No tool-workflow files change in this change, so the Codex/Claude Code/Cursor/OpenCode validation rule does not apply; confirm `git diff --stat` touches no tool-integration files.

## Implementation notes (2026-09-23, macOS arm64, default `minimal` features + `test-probes`)

- Evidence: `evidence/red-run.txt` (RED, before section 3), `evidence/gate-run-1.txt` (contract targets + `settings_persistence`), `evidence/gate-run-2-live-harness.txt` (`integration` `live::harness`), `evidence/gate-run-3-a2a-transport-surface.txt` (listener-surface and governance-status tests rebuilt with `a2a-transport`, so the gated gRPC listener code is compiled and served).
- 1.4: the probe was built only through the test targets (`--features test-probes`); `cargo build --bin uar-env-probe` and the default-build absence were not run separately. The probe also accepts `--mcp` (minimal MCP stdio server) and `--linger-ms` (stays alive for process-table inspection).
- 2.6/2.7/2.12: the route list comes from `uar::api::openapi::build_openapi_spec()`, the document `/api/openapi.json` serves; the route itself exists only with `api-docs`, which `minimal` omits.
- 2.15, 2.17, 2.18 (regression guard), 2.30, 2.31: the standalone server runs as the real `universal-agent-runtime` process rather than in-process, so its environment and stdin can be controlled without mutating the test process.
- 2.18: `tools.deny` removes the tool from the run's visible tool set, so a call to it fails as an unknown tool whether or not governance is enforced. The test asserts that, and adds `tool_approval: "deny"`, the run-policy denial the governance gate actually bypasses when governance is off (RED: the tool ran).
- 2.19: macOS returns an empty environment for platform binaries such as `/bin/sh`, so the observed child is the lingering probe. Linux and Windows runs are open under 4.2.
- 2.21/3.6: `src/mcp/stdio_client.rs` (`StdioMcpClient::spawn`) is not declared as a module and is not compiled, so it has no runtime path and was left unchanged. The three compiled spawn paths are covered: `connect_configured_server`, `connect_server` (reconnect) and `connect_stdio_snapshot`.
- 3.5: the lib unit tests in `runtime_control.rs` and `middleware.rs` were not run (4.3 open); the frontend governance tests were run (3 files, 28 tests pass).
- 4.3: `settings_persistence` (47) and `integration` `live::harness` (4) passed. `mcp_projection` does not compile under `minimal` (it calls the `telemetry`-only `metrics_handle`), independent of this change.
