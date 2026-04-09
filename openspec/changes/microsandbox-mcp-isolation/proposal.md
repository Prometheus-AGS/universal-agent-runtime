## Why

MCP servers currently run as unsandboxed child processes with full host access — they can read/write any file, access the network, and consume unlimited resources. This is a critical security gap: an agent calling a malicious or buggy MCP tool can compromise the host system. The `docs/code-interpreter/` specification (50+ pages) fully designs a microsandbox-based isolation layer using libkrun hardware microVMs, but zero lines of implementation exist. Meanwhile, the `microsandbox` crate (v0.3.12) on crates.io provides a production-ready async Rust API for creating OCI-compatible microVMs with sub-100ms boot times. This change implements the sandboxed execution layer end-to-end: from the `SandboxRunner` trait through microsandbox integration, session management, MCP tool interface (`code_exec`, `shell_exec`, `file_read`, `file_write`), and platform-aware runner selection.

## What Changes

### Core Sandbox Infrastructure
- Add `SandboxRunner` trait as the universal abstraction for sandbox backends (microsandbox, Wasmtime fallback, remote HTTP)
- Add core types: `SandboxConfig`, `SandboxHandle`, `ExecutionRequest`, `ExecutionResult`, `Language`, `ExecutionMode`
- Add `SessionManager` mapping session IDs to persistent sandbox handles with lifecycle management (create, reuse, destroy)

### Microsandbox Runner (Default)
- Add `microsandbox` crate dependency (v0.3.x)
- Implement `MicrosandboxRunner` as the primary `SandboxRunner` backend
- OCI image support for language toolchains (bash, python, rust, node)
- Hardware VM isolation via libkrun (KVM on Linux, HVF on macOS)
- Named volume support for workspace persistence across turns
- Network policy control per sandbox

### Platform-Aware Runner Selection
- Auto-detect KVM (`/dev/kvm`) on Linux, HVF on macOS
- Fall back to Wasmtime (existing) when no hypervisor available
- Fall back to remote HTTP runner for mobile/restricted platforms
- Configurable override via `UAR_SANDBOX_RUNNER` env var

### MCP Tool Interface
- Register sandboxed code execution as MCP native tools: `sandbox__code_exec`, `sandbox__shell_exec`, `sandbox__file_read`, `sandbox__file_write`
- Tools available to agents alongside existing MCP tools (tavily, time, etc.)
- Session-aware: reuse sandbox across tool calls within the same agent run

### Sandboxed MCP Server Execution (Optional)
- Add `sandboxed: true` option to `mcp.json` server entries
- When enabled, spawn MCP server process inside a microsandbox VM instead of directly on the host
- Restrict filesystem access, network, and resource usage per server config

### Streaming Output
- Sandbox stdout/stderr streamed as `NormalizedEvent` variants during execution
- Integrates with existing SSE/AG-UI event pipeline for real-time output in chat UI

## Capabilities

### New Capabilities
- `sandbox-runner-trait`: Universal `SandboxRunner` trait with create/execute/write_file/read_file/destroy methods and capability introspection
- `sandbox-core-types`: Core types for sandbox execution: `SandboxConfig`, `SandboxHandle`, `ExecutionRequest`, `ExecutionResult`, `Language`, `ExecutionMode`
- `microsandbox-runner`: Default runner using microsandbox crate for hardware VM isolation with OCI image support
- `session-sandbox-manager`: Session-to-sandbox lifecycle management with persistent workspaces across agent turns
- `platform-runner-selection`: Auto-detection of hypervisor availability and fallback chain (microsandbox → Wasmtime → remote)
- `sandbox-mcp-tools`: Native MCP tools for code execution, shell commands, and file I/O inside sandboxes
- `sandboxed-mcp-servers`: Option to run MCP server processes inside microsandbox VMs
- `sandbox-config`: Configuration for runner selection, resource limits, OCI images, timeouts, and network policies

### Modified Capabilities
- None (additive; existing MCP execution path unchanged when sandbox not enabled)

## Impact

### Backend (Rust)
- `Cargo.toml`: Add `microsandbox` crate dependency (feature-gated)
- `src/sandbox/mod.rs`: New module — `SandboxRunner` trait, core types
- `src/sandbox/microsandbox_runner.rs`: New — microsandbox backend implementation
- `src/sandbox/wasmtime_runner.rs`: New — Wasmtime fallback (wraps existing `WasmSandbox`)
- `src/sandbox/remote_runner.rs`: New — HTTP client for remote execution
- `src/sandbox/session_manager.rs`: New — session-to-sandbox mapping
- `src/sandbox/platform.rs`: New — runner selection logic
- `src/sandbox/mcp_tools.rs`: New — `NativeTool` implementations for code_exec, shell_exec, file_read, file_write
- `src/mcp/config.rs`: Add `sandboxed: bool` option to `McpServerEntry`
- `src/mcp/registry.rs`: When `sandboxed: true`, launch MCP server inside microsandbox
- `src/server.rs`: Register sandbox MCP tools in `McpRegistry` during startup
- `src/config.rs`: Add `SandboxConfig` to `AppConfig`

### Dependencies
- `microsandbox = "0.3"` (feature-gated: `sandbox-microsandbox`)
- No other new dependencies (reqwest, tokio, serde already available)

### Configuration
- `UAR_SANDBOX_RUNNER`: `auto` | `microsandbox` | `wasmtime` | `remote` (default: `auto`)
- `UAR_SANDBOX_REMOTE_URL`: URL for remote runner (mobile/restricted)
- `UAR_SANDBOX_DEFAULT_MEMORY_MIB`: Default memory per sandbox (default: 512)
- `UAR_SANDBOX_DEFAULT_TIMEOUT_SECS`: Default execution timeout (default: 300)
- `UAR_SANDBOX_NETWORK_ENABLED`: Allow network access in sandboxes (default: false)
