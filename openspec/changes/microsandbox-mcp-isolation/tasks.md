## 1. Sandbox Core Types

- [x] 1.1 Create `src/sandbox/mod.rs` with module declarations and re-exports
- [x] 1.2 Create `src/sandbox/types.rs` with `SandboxConfig`, `SandboxHandle`, `ExecutionRequest`, `ExecutionResult`, `Language`, `ExecutionMode`, `SandboxError`
- [x] 1.3 Create `src/sandbox/runner.rs` with `SandboxRunner` trait (create, execute, write_file, read_file, destroy) and `RunnerCapabilities`
- [x] 1.4 Add `pub mod sandbox;` to `src/lib.rs` or `src/uar/mod.rs`
- [x] 1.5 Verify core types compile with unit tests for serialization

## 2. Session Manager

- [x] 2.1 Create `src/sandbox/session_manager.rs` with `SessionManager` struct using `DashMap<String, SessionEntry>`
- [x] 2.2 `SessionEntry` holds `SandboxHandle`, `created_at`, `last_accessed`, `ttl`
- [x] 2.3 Implement `get_or_create(session_id, config)` — returns existing handle or creates new sandbox
- [x] 2.4 Implement `destroy(session_id)` — destroys sandbox and removes entry
- [x] 2.5 Implement TTL-based garbage collection (background task, default 30 min TTL, extend on access)
- [x] 2.6 Add `max_concurrent_sandboxes` limit — return error when exceeded
- [x] 2.7 Add unit test: verify session reuse across multiple get_or_create calls

## 3. Wasmtime Runner (Fallback)

- [x] 3.1 Create `src/sandbox/wasmtime_runner.rs` implementing `SandboxRunner`
- [x] 3.2 Wrap existing `WasmSandbox` from `src/uar/runtime/wasm/sandbox.rs`
- [x] 3.3 Map `ExecutionRequest` to WASM execution (Bash language only for Wasmtime fallback)
- [x] 3.4 Return `RunnerCapabilities { supports_long_running: false, supports_networking: false, runner_type: Wasmtime }`
- [x] 3.5 Implement `write_file` and `read_file` via WASI preopened directories

## 4. Microsandbox Runner

- [x] 4.1 Add `microsandbox = "0.3"` to `Cargo.toml` under `[dependencies]` with feature gate `sandbox-microsandbox`
- [x] 4.2 Add `sandbox-microsandbox = ["dep:microsandbox"]` to `[features]`
- [x] 4.3 Create `src/sandbox/microsandbox_runner.rs` with `MicrosandboxRunner` struct
- [x] 4.4 Implement `create()`: build `Sandbox` from `SandboxConfig` (image, memory, cpus, volumes, network)
- [x] 4.5 Implement `execute()`: run command inside sandbox via `sandbox.command()`, capture stdout/stderr
- [x] 4.6 Implement `write_file()`: use microsandbox filesystem API to write into sandbox
- [x] 4.7 Implement `read_file()`: use microsandbox filesystem API to read from sandbox
- [x] 4.8 Implement `destroy()`: call `sandbox.destroy()` and clean up named volumes if ephemeral
- [x] 4.9 Map `Language` enum to appropriate command: Python → `python3 -c`, Rust → `rustc + run`, Bash → `bash -c`, Node → `node -e`
- [x] 4.10 Set up named volume `workspace` mounted at `/workspace` for session persistence
- [x] 4.11 Return `RunnerCapabilities { supports_long_running: true, supports_networking: configurable, runner_type: MicroVm }`
- [x] 4.12 Add integration test: create sandbox, execute `echo hello`, verify output (requires KVM/HVF)

## 5. Remote Runner

- [x] 5.1 Create `src/sandbox/remote_runner.rs` implementing `SandboxRunner`
- [x] 5.2 HTTP POST to `{base_url}/create`, `/execute`, `/write_file`, `/read_file`, `/destroy` endpoints
- [x] 5.3 Accept `base_url` and optional `auth_token` in constructor
- [x] 5.4 Return `RunnerCapabilities { supports_long_running: true, supports_networking: true, runner_type: Remote }`

## 6. Platform Runner Selection

- [x] 6.1 Create `src/sandbox/platform.rs` with `build_runner()` function
- [x] 6.2 Check `UAR_SANDBOX_RUNNER` env var for explicit override (`microsandbox`, `wasmtime`, `remote`)
- [x] 6.3 On Linux: check `/dev/kvm` exists → use `MicrosandboxRunner`
- [x] 6.4 On macOS: assume HVF available → use `MicrosandboxRunner`
- [x] 6.5 If `UAR_SANDBOX_REMOTE_URL` set → use `RemoteRunner`
- [x] 6.6 Fallback: use `WasmtimeRunner` with warning log
- [x] 6.7 Feature gate: when `sandbox-microsandbox` not compiled, skip microsandbox check
- [x] 6.8 Add unit test: verify correct runner selected for different platform/env configurations

## 7. Sandbox MCP Tools

- [x] 7.1 Create `src/sandbox/mcp_tools.rs` with four `NativeTool` implementations
- [x] 7.2 `SandboxCodeExecTool`: accepts `{language, code, session_id?, timeout_seconds?}`, calls `session_manager.get_or_create()` + `runner.execute()`
- [x] 7.3 `SandboxShellExecTool`: accepts `{command, session_id?, timeout_seconds?}`, executes shell command in sandbox
- [x] 7.4 `SandboxFileReadTool`: accepts `{session_id, path}`, calls `runner.read_file()`
- [x] 7.5 `SandboxFileWriteTool`: accepts `{session_id, path, content}`, calls `runner.write_file()`
- [x] 7.6 Each tool returns JSON with exit_code, stdout, stderr, execution_time_ms (or file content/success)
- [x] 7.7 Register all four tools in `McpRegistry` during server startup via `with_native_tool()`
- [x] 7.8 Add OpenAI-compatible tool schemas (function name, description, parameters JSON schema)

## 8. Sandboxed MCP Server Execution

- [x] 8.1 Add `sandboxed: Option<bool>` field to `McpServerEntry::Stdio` in `src/mcp/config.rs`
- [x] 8.2 In MCP server launch path (registry.rs), check `sandboxed` flag
- [x] 8.3 When `sandboxed: true`, create a microsandbox VM and run the MCP command inside it
- [x] 8.4 Pipe sandbox stdin/stdout to the rmcp `TokioChildProcess` transport equivalent
- [x] 8.5 Apply filesystem restriction: only the server's working directory is accessible
- [x] 8.6 Apply network policy from per-server config or global default

## 9. Configuration

- [x] 9.1 Add `SandboxConfig` struct to `src/config.rs` with runner, memory, timeout, network, max_concurrent fields
- [x] 9.2 Add `#[serde(default)] pub sandbox: SandboxConfig` to `AppConfig`
- [x] 9.3 Implement `Default` for `SandboxConfig` with production defaults
- [x] 9.4 Wire env vars: `UAR_SANDBOX_RUNNER`, `UAR_SANDBOX_REMOTE_URL`, `UAR_SANDBOX_DEFAULT_MEMORY_MIB`, `UAR_SANDBOX_DEFAULT_TIMEOUT_SECS`, `UAR_SANDBOX_NETWORK_ENABLED`, `UAR_SANDBOX_MAX_CONCURRENT`

## 10. Server Integration

- [x] 10.1 In `start_server()`, call `platform::build_runner()` to get the sandbox runner
- [x] 10.2 Create `SessionManager` with the selected runner
- [x] 10.3 Create the four sandbox MCP tools with a reference to the session manager
- [x] 10.4 Register tools in `McpRegistry` before server starts accepting requests
- [x] 10.5 Add `session_manager: Arc<SessionManager>` to `AppState`
- [x] 10.6 Start session GC background task (periodic cleanup of expired sandboxes)

## 11. Sandbox Execution Events

- [x] 11.1 Add `NormalizedEvent::SandboxOutput { stream: String, data: String }` variant (stream = "stdout" | "stderr")
- [x] 11.2 During sandbox execution, emit `SandboxOutput` events for real-time streaming to the UI
- [x] 11.3 Map `SandboxOutput` to AG-UI event `agui.sandbox.output` in SSE layer
- [x] 11.4 Add `sandbox:stdout` and `sandbox:stderr` to frontend event handling

## 12. Metrics

- [x] 12.1 Add `uar_sandbox_created_total` counter (labels: runner_type, language)
- [x] 12.2 Add `uar_sandbox_execution_duration_seconds` histogram (labels: language, exit_code_class)
- [x] 12.3 Add `uar_sandbox_active` gauge (current active sandboxes)
- [x] 12.4 Add `uar_sandbox_errors_total` counter (labels: error_type)

## 13. End-to-End Verification

- [x] 13.1 Test `sandbox__code_exec` with Python code → verify output
- [x] 13.2 Test `sandbox__shell_exec` with bash command → verify output
- [x] 13.3 Test `sandbox__file_write` + `sandbox__file_read` round-trip
- [x] 13.4 Test session persistence: write file in turn 1, read it in turn 2
- [ ] 13.5 Test sandboxed MCP server: spawn with `sandboxed: true`, verify tool calls work
- [x] 13.6 Test platform fallback: disable microsandbox feature, verify Wasmtime is selected
- [x] 13.7 Test TTL expiry: create sandbox, wait > TTL, verify cleanup
