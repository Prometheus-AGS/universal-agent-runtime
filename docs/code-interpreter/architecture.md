# UAR Code Interpreter — System Architecture

_Last updated: 2026-02-21_

---

## 1. High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│ UAR Agent / Plugin                                              │
│   tool_call: code_exec { language: "rust", code: "..." }       │
└────────────────────────────┬────────────────────────────────────┘
                             │ MCP tool call
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│ uar-code-interpreter  (Axum service, port 5001)                 │
│                                                                 │
│  ┌─────────────┐   ┌──────────────────┐   ┌─────────────────┐  │
│  │ MCP Server  │   │  Session Manager │   │ Stream Publisher│  │
│  │ (tool API)  │──►│  (sandbox pool)  │──►│ (uar-realtime)  │  │
│  └─────────────┘   └────────┬─────────┘   └─────────────────┘  │
│                             │                                   │
│             ┌───────────────▼────────────────────┐             │
│             │         SandboxRunner trait         │             │
│             ├─────────────┬──────────┬────────────┤            │
│             │ microsandbox│Firecracker│  Wasmtime  │            │
│             │  (default)  │ (P3 opt) │ (fallback) │            │
│             └─────────────┴──────────┴────────────┘            │
└─────────────────────────────────────────────────────────────────┘
                             │
              ┌──────────────▼──────────────────┐
              │  MicroVM (per session/run)        │
              │  ┌───────────────────────────┐   │
              │  │  /workspace/              │   │
              │  │    main.rs / main.py /    │   │
              │  │    main.js / script.sh    │   │
              │  │  toolchains:              │   │
              │  │    rustup, python3, node  │   │
              │  └───────────────────────────┘   │
              │  Dedicated kernel per sandbox     │
              └──────────────────────────────────┘
```

---

## 2. Execution Modes

### 2.1 Ephemeral Mode

One-shot execution. Sandbox is created, code runs, output is captured, sandbox is destroyed.

```
request → create sandbox → write code → execute → collect output → destroy → return
                                                 └─ stream via uar-realtime while running
```

**Use case:** Quick calculations, data analysis scripts, one-off transformations.

### 2.2 Session Mode (Kimi-style)

The sandbox persists for the entire UAR session. State accumulates across agent turns.

```
Session starts → sandbox created
  Turn 1: agent writes file.py, runs it → output
  Turn 2: agent imports file from Turn 1, extends it → output
  Turn 3: agent reads Turn 1 results, transforms them → output
Session ends → sandbox destroyed
```

**Use case:** Multi-step data analysis, iterative debugging, progressive code development.

### 2.3 Project Mode

Full project directory mounted in the sandbox. Supports compilation, dependency installation, test suites.

```
project starts → sandbox created → project files mounted at /workspace
  → cargo build / npm install / pip install -r requirements.txt
  → cargo test / pytest / node test.js
  → agent reads test output, fixes code, re-runs
project ends → sandbox archived or destroyed
```

**Use case:** Full repository workflows, CI-like pipelines, multi-file projects.

### 2.4 Parallel Swarm Mode

Orchestrator spins up N sandboxes simultaneously for independent parallel subtasks.

```
orchestrator splits task → spawns sandbox_A, sandbox_B, sandbox_C
  sandbox_A: implements feature X
  sandbox_B: writes tests for X
  sandbox_C: implements feature Y
                                └─ all stream output simultaneously
orchestrator collects results, merges
```

**Use case:** Agent swarm execution (same pattern as Kimi K2.5 Agent Swarm).

---

## 3. Workspace Structure

```
universal-agent-runtime/                 ← existing git repo root
  Cargo.toml                             ← workspace root ([workspace])

  crates/
    uar-sandbox-core/                    ← NEW: shared types and traits
      Cargo.toml
      src/
        lib.rs
        runner.rs          ← SandboxRunner trait
        request.rs         ← ExecutionRequest, SandboxConfig
        result.rs          ← ExecutionResult, ExitStatus
        stream.rs          ← SandboxOutputStream (async stream of stdout/stderr chunks)
        session.rs         ← SessionSandbox lifecycle types
        error.rs           ← SandboxError

  uar-code-interpreter/                  ← NEW: standalone Axum service
    Cargo.toml
    Dockerfile
    build/
      images/                            ← OCI base images per language
        rust/Dockerfile                  ← rustup + cargo + build-essential
        python/Dockerfile                ← python3 + pip + numpy, pandas defaults
        node/Dockerfile                  ← node (LTS) + npm + yarn
        bash/Dockerfile                  ← bash + common Unix tools
        universal/Dockerfile             ← all toolchains (larger, for project mode)
    src/
      main.rs
      config.rs
      runner/
        mod.rs             ← SandboxRunner trait re-export
        microsandbox.rs    ← microsandbox SDK integration (default)
        firecracker.rs     ← Firecracker REST API (feature: firecracker)
        wasmtime.rs        ← wasmtime fallback (feature: wasm-fallback)
        remote.rs          ← HTTP to remote uar-code-interpreter (mobile/restricted)
      session/
        manager.rs         ← SessionSandboxManager (per-session sandbox pool)
        store.rs           ← in-memory session → sandbox handle mapping
      executor/
        mod.rs
        ephemeral.rs       ← one-shot execution path
        session.rs         ← session-mode execution
        project.rs         ← project-mode (git clone + build)
      languages/
        mod.rs             ← LanguageConfig trait
        bash.rs            ← Bash: /bin/bash, shebang detection
        rust.rs            ← Rust: rustc direct, cargo project mode
        python.rs          ← Python: python3, venv, pip
        node.rs            ← Node.js: node, npm, npx, yarn
      stream/
        realtime.rs        ← publish stdout/stderr to uar-realtime
        collector.rs       ← buffer mode for one-shot calls
      mcp/
        server.rs          ← MCP tool server (code_exec, shell_exec, file_*)
        tools.rs           ← tool definitions
      api/
        routes.rs          ← Axum router
        handlers.rs        ← HTTP handlers
        internal.rs        ← POST /internal/v1/execute (UAR → interpreter)
      auth.rs              ← JWT verification
```

---

## 4. Core Types

### `SandboxRunner` trait

```rust
// crates/uar-sandbox-core/src/runner.rs

use async_trait::async_trait;
use crate::{ExecutionRequest, ExecutionResult, SandboxHandle, SandboxConfig, SandboxError};

#[async_trait]
pub trait SandboxRunner: Send + Sync {
    /// Create a new sandbox and return a handle to it.
    async fn create(&self, config: SandboxConfig) -> Result<SandboxHandle, SandboxError>;
    
    /// Execute code/command in an existing sandbox, streaming output.
    async fn execute(
        &self,
        handle: &SandboxHandle,
        request: ExecutionRequest,
    ) -> Result<ExecutionResult, SandboxError>;

    /// Write a file into the sandbox filesystem.
    async fn write_file(&self, handle: &SandboxHandle, path: &str, content: &[u8]) -> Result<(), SandboxError>;
    
    /// Read a file from the sandbox filesystem.
    async fn read_file(&self, handle: &SandboxHandle, path: &str) -> Result<Vec<u8>, SandboxError>;

    /// Destroy the sandbox and release all resources.
    async fn destroy(&self, handle: SandboxHandle) -> Result<(), SandboxError>;
    
    /// Runner capabilities — used to pick the best runner for a platform.
    fn capabilities(&self) -> RunnerCapabilities;
}

#[derive(Debug, Clone)]
pub struct RunnerCapabilities {
    pub supports_long_running: bool,
    pub supports_networking: bool,
    pub max_execution_seconds: Option<u64>,
    pub runner_type: RunnerType,
}

#[derive(Debug, Clone, Copy)]
pub enum RunnerType {
    MicroVm,       // microsandbox or Firecracker
    Wasmtime,      // in-process WASM
    Remote,        // HTTP to remote uar-code-interpreter
}
```

### `ExecutionRequest`

```rust
// crates/uar-sandbox-core/src/request.rs

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionRequest {
    /// Language runtime to use.
    pub language: Language,
    /// Source code or shell command.
    pub code: String,
    /// stdin to supply, if any.
    pub stdin: Option<String>,
    /// Environment variables.
    pub env: std::collections::HashMap<String, String>,
    /// Working directory inside the sandbox (default: /workspace).
    pub cwd: Option<String>,
    /// Maximum wall-clock execution time.
    pub timeout_seconds: Option<u64>,
    /// Execution mode.
    pub mode: ExecutionMode,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Language {
    Bash,
    Rust,
    Python,
    Node,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ExecutionMode {
    /// One shot — sandbox created and destroyed per call.
    Ephemeral,
    /// Reuse sandbox for this session ID.
    Session { session_id: String },
    /// Project mode — git repo checked out at startup.
    Project { session_id: String, repo_url: Option<String> },
}
```

---

## 5. Deployment Modes

Same pattern as `uar-realtime` — the deployment scenario determines transport, not the code path.

| Scenario | Config | Transport |
|---|---|---|
| **Tauri / desktop** | `UAR_SANDBOX_MODE=sidecar` | Tauri sidecar binary, UAR talks to it via internal HTTP |
| **Dev / single-node** | No external URL | In-process runner choice (microsandbox on Linux/macOS, wasmtime elsewhere) |
| **Cloud** | `UAR_SANDBOX_EXTERNAL_URL=http://...` | Separate `uar-code-interpreter` service, UAR publishes via HTTP |
| **Mobile** | Platform auto-detected | Remote runner (calls cloud service) |

The UAR `AppState` gains:
```rust
pub interpreter: Arc<dyn SandboxClient>,
```
where `SandboxClient` is either `LocalInterpreterClient` (HTTP to sidecar/service) or `RemoteInterpreterClient` (cloud API).
