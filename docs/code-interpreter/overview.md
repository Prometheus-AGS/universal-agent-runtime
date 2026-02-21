# UAR Code Interpreter — Overview & Design Rationale

_Last updated: 2026-02-21_

---

## 1. Goals

The `uar-code-interpreter` must provide:

| Requirement | Detail |
|---|---|
| **Secure isolation** | Untrusted AI-generated code cannot escape or affect the host system |
| **Multi-language** | Bash, Rust, Python, Node.js — first class, with correct toolchains pre-installed |
| **Multi-platform** | Linux, macOS, Windows, iOS, Android — with appropriate runner per platform |
| **Streaming output** | stdout/stderr delivered to the client in real-time via `uar-realtime` |
| **Session persistence** | Sandbox filesystem survives across agent turns in a session (Kimi-style) |
| **Long-running processes** | Compilation (`cargo build`), test suites, servers — run to completion |
| **Project support** | Full repository checked out inside the sandbox; build tools available |
| **MCP integration** | Exposed as MCP tools (`code_exec`, `shell_exec`, `file_read`, `file_write`) |
| **UAR-native** | Integrates with UAR agents, sessions, realtime streaming, and auth |
| **Tauri/desktop support** | Works on developer machines without cloud dependency |

---

## 2. Prior Art Survey

### 2.1 e2b

**[e2b.dev](https://e2b.dev)** — The closest open-source reference implementation.

- **Runtime:** Firecracker microVMs (KVM-based). Each sandbox is a full Linux VM with a dedicated kernel.
- **Startup:** 150ms via VM snapshotting — pre-warmed VM images restore near-instantly.
- **Session model:** Sandboxes can run up to 24 hours; filesystem persists within a session.
- **SDK:** Python + JavaScript/TypeScript. REST API underneath.
- **Self-hosting:** Terraform on GCP (AWS planned). Non-trivial operational overhead.
- **Limitations:** Cloud-first design; no native macOS/Windows/mobile support; no built-in MCP.

**What we borrow:** Session model, VM snapshot concept, language toolchain containers, streaming stdout/stderr.

**What we do differently:** MCP-native from day one; pluggable runner backends; first-class desktop/mobile support; UAR-integrated auth and realtime events.

---

### 2.2 microsandbox (`zerocore-ai/microsandbox`)

**[github.com/zerocore-ai/microsandbox](https://github.com/zerocore-ai/microsandbox)**

- **Runtime:** libkrun (rust-vmm) — per-sandbox dedicated microVM + dedicated kernel.
- **Language:** Written entirely in Rust. Rust SDK available on crates.io.
- **OCI-compatible:** Runs any standard Docker/OCI image inside the VM.
- **Startup:** <200ms without snapshotting.
- **macOS:** Works on Apple Silicon via HVF (Apple Hypervisor Framework) — libkrun is cross-platform.
- **MCP:** Built-in MCP server support.
- **Project sandboxes:** Persistent filesystem across runs.
- **License:** Apache 2.0.
- **Status:** Experimental but actively developed (2024–2025).

**This is our default runner backend.** The Rust SDK integrates cleanly with the UAR workspace; HVF support means it works on developer macOS machines; MCP is already wired.

---

### 2.3 Firecracker

**[github.com/firecracker-microvm/firecracker](https://github.com/firecracker-microvm/firecracker)**

- Written in Rust by AWS. Powers AWS Lambda and Fargate.
- 125ms cold start, <5 MiB memory overhead per VM.
- **VM snapshotting** — capture VM state, restore in ~10ms (warm pool pattern).
- REST API exposed by the VMM; no Rust SDK — must issue HTTP calls.
- **Linux-only** (KVM required). Only available on x86_64 and aarch64 Linux.

**Role:** Optional P3 backend for production-grade snapshotting and warm-VM pools at cloud scale. Not needed for initial implementation.

---

### 2.4 Kimi (Moonshot AI) — Session Sandbox Pattern

Kimi's web interface demonstrates the production UX target for session sandboxes:

- **Persistent workspace:** A sandbox VM lives for the duration of a conversation. Files, installed packages, and compiled artifacts persist between messages.
- **Long-running processes:** Compilation, test suites, build pipelines run to completion. Results stream back as they produce output.
- **Agent mode + swarm:** The orchestrator can spawn multiple sandboxes in parallel for independent subtasks, then collect results.
- **Repl-like interaction:** Agent can write a file in turn 1, compile in turn 2, debug in turn 3 — all in the same environment.

**What we borrow:** Session sandbox lifecycle (create once per session, destroy on session end), streaming output, project-mode workspace.

---

### 2.5 Existing UAR Wasm Sandbox

UAR already has a `wasm-runtime` feature using `wasmtime`. This handles **trusted, fast, in-process** plugin execution (WASM modules from plugins). The code interpreter is a **complement, not a replacement**:

| | Wasm sandbox (existing) | microVM sandbox (new) |
|---|---|---|
| Target | Trusted plugin WASM modules | Untrusted AI-generated code |
| Isolation | Process-level (wasmtime) | Hardware VM (dedicated kernel) |
| Startup | Microseconds | 10–200ms |
| Languages | WASM-compiled only | Any language with a toolchain |
| Filesystem | Virtual FS only | Full Linux filesystem |
| Long-running | Not designed for | ✅ |
| Platform | All platforms | Linux/macOS full; Windows/mobile fallback |

---

## 3. Architectural Decisions

### ADR-001: microsandbox as default runner, Firecracker as optional

**Decision:** Default runner is microsandbox (libkrun). Firecracker is an optional compile-time feature for production clusters.

**Rationale:**
- microsandbox has a Rust SDK on crates.io — integrates cleanly into the workspace.
- libkrun supports macOS via HVF — works on developer machines.
- Firecracker requires Linux KVM and a non-trivial jailer setup — overkill for the initial implementation.
- Both provide the same security guarantee (hardware VM, dedicated kernel).

---

### ADR-002: `SandboxRunner` trait — pluggable backends

**Decision:** All interaction with the underlying sandbox technology goes through a `SandboxRunner` trait. Concrete implementations are selected at compile time (feature flags) or runtime (config).

**Rationale:** Allows microsandbox → Firecracker promotion without touching agent/MCP code.

---

### ADR-003: Platform-tiered runner selection

**Decision:** Different platforms use different runners to achieve the best possible security within platform constraints.

| Platform | Primary runner | Fallback |
|---|---|---|
| Linux | microsandbox (KVM microVM) | Wasmtime |
| macOS | microsandbox (HVF microVM) | Wasmtime |
| Windows | microsandbox via WSL2 / Hyper-V | Remote execution |
| iOS | Remote execution (cloud uar-code-interpreter) | Wasmtime (restricted) |
| Android | Remote execution | Wasmtime (restricted) |

Mobile platforms cannot run hypervisors in App Store/Play Store distributed apps. The fallback path for mobile is to call the cloud-hosted `uar-code-interpreter` service over the existing UAR API — the same code path, just with the execution happening remotely.

---

### ADR-004: Streaming output via uar-realtime

**Decision:** stdout and stderr from sandbox execution are published as `uar-realtime` events on the `sandbox:{session_id}` channel, not buffered and returned at completion.

**Rationale:** Long-running processes (compilation, test suites) would otherwise have unbounded latency before any output reaches the user. Real-time streaming matches the existing token-delta streaming pattern.

---

### ADR-005: MCP-first tool interface

**Decision:** `uar-code-interpreter` registers its capabilities as MCP tools: `code_exec`, `shell_exec`, `file_read`, `file_write`, `sandbox_create`, `sandbox_destroy`.

**Rationale:** UAR agents already use MCP for all tool calls. This makes code execution a natural tool invocation — no new agent protocol needed. It also means any MCP-compatible external client can use the code interpreter.

---

### ADR-006: Tauri mode — sidecar binary, not in-process

**Decision:** In Tauri desktop mode, `uar-code-interpreter` runs as a **Tauri sidecar binary** (bundled alongside the app), NOT in-process like the realtime broker.

**Rationale:**
- The code interpreter uses KVM/HVF — it requires OS-level privileges that must stay in a separate process.
- Tauri supports bundling sidecar binaries and managing their lifecycle.
- UAR communicates with the sidecar over the same internal HTTP API used in cloud mode — zero code change.

```json
// tauri.conf.json
{
  "bundle": {
    "externalBin": ["binaries/uar-code-interpreter"]
  }
}
```
