## Context

UAR's MCP tool execution currently spawns child processes with full host access. The `docs/code-interpreter/` specification designs a comprehensive sandboxed execution layer, but none of it is implemented. The [microsandbox crate](https://crates.io/crates/microsandbox) (v0.3.12) provides an async Rust API for creating OCI-compatible microVMs via libkrun with sub-100ms boot, named volumes, command execution, and filesystem access — exactly what we need.

The existing `WasmSandbox` in `src/uar/runtime/wasm/` handles trusted WASM plugins but can't run arbitrary code (no pip/cargo/npm, limited language support). Microsandbox fills this gap with hardware VM isolation for untrusted AI-generated code.

## Goals / Non-Goals

**Goals:**
- Agents can execute arbitrary code safely inside hardware-isolated VMs
- MCP tools (`code_exec`, `shell_exec`, `file_read`, `file_write`) available out-of-the-box
- Session persistence: sandbox state survives across agent turns within a conversation
- Platform-aware: works on Linux (KVM), macOS (HVF), falls back gracefully elsewhere
- MCP servers can optionally run inside sandboxes (`sandboxed: true` in mcp.json)
- Zero breaking changes to existing unsandboxed MCP execution

**Non-Goals:**
- Firecracker integration (P3, separate change)
- uar-code-interpreter as a separate service (this embeds sandbox execution in-process)
- uar-realtime event streaming for sandbox output (use existing NormalizedEvent/SSE)
- OCI image build pipeline (use pre-built images from registries)
- GPU passthrough (future microsandbox feature)

## Decisions

### D1: Embed Sandbox in UAR Process (Not Separate Service)
**Decision**: The sandbox runner lives in-process as part of UAR, not as a separate `uar-code-interpreter` service.

**Rationale**: The spec describes a separate microservice, but for initial implementation, embedding simplifies deployment (one binary), reduces latency (no HTTP hop), and aligns with the Tauri single-binary model. Extract to separate service later if needed.

### D2: microsandbox as Default Runner
**Decision**: Use `microsandbox` crate (libkrun) as the default runner on Linux/macOS. Feature-gated via `sandbox-microsandbox` Cargo feature.

**Rationale**: libkrun provides hardware VM isolation with sub-200ms startup, OCI image support, and a clean Rust API. It works on both Linux (KVM) and macOS (HVF/Apple Silicon), covering the primary development and deployment platforms.

**microsandbox API usage pattern:**
```rust
use microsandbox::Sandbox;

let sandbox = Sandbox::builder()
    .image("prometheus-ags/sandbox-python:latest")
    .memory(512)  // MiB
    .cpus(1)
    .volume("workspace", "/workspace")
    .build()
    .await?;

let result = sandbox.command("python3")
    .arg("-c")
    .arg(&code)
    .output()
    .await?;

sandbox.destroy().await?;
```

### D3: SandboxRunner Trait Matches Spec Exactly
**Decision**: Implement the `SandboxRunner` trait from `docs/code-interpreter/architecture.md` verbatim. Five methods: `create`, `execute`, `write_file`, `read_file`, `destroy`.

**Rationale**: The spec is well-designed and was written with the microsandbox API in mind. Following it exactly enables future Firecracker/remote runners to plug in without interface changes.

### D4: Session Manager Uses In-Memory HashMap with TTL
**Decision**: `SessionManager` stores `session_id → SandboxHandle` in a `DashMap` with configurable TTL (default 30 minutes). Expired sessions are garbage collected.

**Rationale**: Simpler than database-backed sessions for the initial implementation. Aligns with how `SessionStore` already works for chat sessions.

### D5: Sandbox MCP Tools as NativeTools
**Decision**: Register `code_exec`, `shell_exec`, `file_read`, `file_write` as `NativeTool` implementations in the `McpRegistry`, namespaced as `sandbox__code_exec` etc.

**Rationale**: `NativeTool` is the existing pattern for high-performance in-process tools. They bypass MCP server overhead and integrate directly with the tool execution pipeline. Agents discover them like any other MCP tool.

### D6: Sandboxed MCP Servers Via Config Flag
**Decision**: Add `"sandboxed": true` to `McpServerEntry` in `mcp.json`. When set, the MCP server process is spawned inside a microsandbox VM with restricted filesystem/network.

**Rationale**: This is the simplest path to sandboxed MCP servers — no protocol changes, just wrapping the launch command in a VM. The MCP server still communicates via stdio, but the process runs in an isolated environment.

### D7: Feature-Gated Compilation
**Decision**: All microsandbox code behind `#[cfg(feature = "sandbox-microsandbox")]`. When the feature is off, only Wasmtime and remote runners are available.

**Rationale**: microsandbox requires KVM/HVF. Feature-gating allows compilation on platforms without hypervisor support (CI, containers without /dev/kvm).

## Risks / Trade-offs

| Risk | Severity | Mitigation |
|------|----------|------------|
| microsandbox crate is experimental (v0.3) | Medium | Pin version, wrap in SandboxRunner trait for easy replacement |
| KVM not available in CI/Docker containers | Medium | Feature gate; Wasmtime fallback for testing |
| OCI image pull adds cold-start latency | Low | Pre-pull images at server startup; image cache in named volume |
| Session sandbox TTL may be too short for long conversations | Low | Configurable TTL; extend on each tool call |
| Large sandbox memory usage with many concurrent sessions | Medium | `max_concurrent_sandboxes` config limit; LRU eviction |

## Migration Plan

### Phase 1: Core Types + Wasmtime Runner
1. Create `src/sandbox/` module with trait, types, and Wasmtime runner
2. Implement session manager
3. Register sandbox MCP tools (NativeTool)
4. Test with Wasmtime backend (works everywhere)

### Phase 2: Microsandbox Runner
5. Add `microsandbox` crate dependency (feature-gated)
6. Implement `MicrosandboxRunner`
7. Platform detection and runner selection
8. Test on Linux with KVM

### Phase 3: Sandboxed MCP Servers
9. Add `sandboxed` option to MCP config
10. Wrap MCP server launch in microsandbox when enabled
11. Test with a sandboxed MCP server

### Phase 4: Configuration + Polish
12. Full config integration (env vars, config.yaml)
13. Prometheus metrics for sandbox lifecycle
14. Admin UI sandbox status page (future)
