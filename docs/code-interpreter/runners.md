# UAR Code Interpreter — Sandbox Runner Backends

_Last updated: 2026-02-21_

All sandbox interaction goes through the `SandboxRunner` trait (defined in `crates/uar-sandbox-core`). Concrete backends are selected at startup based on platform and config.

---

## Runner Comparison

| Backend | Technology | Platform | Isolation | Startup | Status |
|---|---|---|---|---|---|
| **microsandbox** | libkrun (rust-vmm) | Linux (KVM), macOS (HVF) | Hardware VM | ~150–200ms | **Default** |
| **Firecracker** | KVM microVM | Linux only | Hardware VM | ~125ms (snapshot: ~10ms) | Optional (P3) |
| **Wasmtime** | WASM sandbox | All (in-process) | Process-level | <1ms | Fallback |
| **Remote** | HTTP to uar-code-interpreter | All | Delegated | Network latency | Mobile/restricted |

---

## 1. microsandbox (Default)

**crate:** `microsandbox` on crates.io
**GitHub:** `zerocore-ai/microsandbox`
**License:** Apache 2.0

### Why it's the default

- Pure Rust SDK — integrates cleanly into the Cargo workspace
- libkrun uses HVF on macOS Apple Silicon — works on developer machines without Linux
- Built-in MCP server support
- OCI-compatible — runs any Docker image
- Self-hostable with zero infrastructure overhead
- Apache licensed

### Integration

```toml
# uar-code-interpreter/Cargo.toml
[dependencies]
microsandbox = "0.x"

[features]
default = ["microsandbox-runner"]
microsandbox-runner = []
```

```rust
// src/runner/microsandbox.rs
use microsandbox::{Sandbox, SandboxBuilder, ExecInput};
use uar_sandbox_core::{SandboxRunner, SandboxHandle, ExecutionRequest, ExecutionResult, SandboxConfig};

pub struct MicrosandboxRunner {
    config: RunnerConfig,
}

#[async_trait::async_trait]
impl SandboxRunner for MicrosandboxRunner {
    async fn create(&self, config: SandboxConfig) -> Result<SandboxHandle, SandboxError> {
        let sandbox = SandboxBuilder::new()
            .image(&config.image)          // OCI image tag
            .memory_mib(config.memory_mib) // default: 512
            .cpus(config.cpus)             // default: 1
            .build()
            .await?;
        
        Ok(SandboxHandle {
            id: sandbox.id().to_owned(),
            inner: Arc::new(Mutex::new(sandbox)),
        })
    }
    
    async fn execute(
        &self,
        handle: &SandboxHandle,
        request: ExecutionRequest,
    ) -> Result<ExecutionResult, SandboxError> {
        let sandbox = handle.inner.lock().await;
        
        // Write code to file in sandbox
        sandbox.exec(&["bash", "-c", &format!(
            "cat > /workspace/{} << 'EOF'\n{}\nEOF",
            request.language.file_name(),
            request.code
        )]).await?;
        
        // Run setup commands (pip install, npm install, etc.)
        for cmd in request.language.setup_commands() {
            sandbox.exec(&cmd).await?;
        }
        
        // Execute with streaming output
        let mut output = sandbox.exec_streaming(&request.language.run_command()).await?;
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        
        while let Some(chunk) = output.next().await {
            match chunk? {
                OutputChunk::Stdout(data) => {
                    // Publish to uar-realtime
                    self.stream_publisher.publish_stdout(&request.session_id, &data).await;
                    stdout.extend_from_slice(&data);
                }
                OutputChunk::Stderr(data) => {
                    self.stream_publisher.publish_stderr(&request.session_id, &data).await;
                    stderr.extend_from_slice(&data);
                }
                OutputChunk::Exit(code) => {
                    return Ok(ExecutionResult {
                        exit_code: code,
                        stdout: String::from_utf8_lossy(&stdout).into_owned(),
                        stderr: String::from_utf8_lossy(&stderr).into_owned(),
                    });
                }
            }
        }
        
        unreachable!()
    }
    
    fn capabilities(&self) -> RunnerCapabilities {
        RunnerCapabilities {
            supports_long_running: true,
            supports_networking: true,
            max_execution_seconds: None, // configurable via sandbox timeout
            runner_type: RunnerType::MicroVm,
        }
    }
}
```

### OCI Images

microsandbox pulls OCI images on first use. Images are cached in `/var/cache/uar-sandbox/images/`.

```
prometheus-ags/sandbox-bash:latest      (~200 MB)
prometheus-ags/sandbox-rust:latest      (~1.2 GB, includes rustup + toolchain)
prometheus-ags/sandbox-python:latest    (~500 MB, includes common data science libs)
prometheus-ags/sandbox-node:latest      (~400 MB, includes Node LTS + common tools)
prometheus-ags/sandbox-universal:latest (~2.5 GB, all runtimes)
```

---

## 2. Firecracker (P3 — Optional, Cloud Scale)

**GitHub:** `firecracker-microvm/firecracker`
**License:** Apache 2.0
**Platform:** Linux x86_64 / aarch64 only (KVM required)

### What it adds over microsandbox

- **VM snapshotting** — capture VM state at "post-boot, pre-user-code" point and restore in ~10ms. Enables a warm-VM pool for sub-50ms first-run latency.
- **Production proven** — powers AWS Lambda, AWS Fargate.
- **Fine-grained resource limits** — CPU bandwidth throttling, balloon memory management.

### When to enable

Enable when:
1. You're running a production cloud deployment with >100 concurrent sandboxes
2. <50ms cold-start latency is required (snapshot restore)
3. All nodes are Linux x86_64 or aarch64

```toml
# Enable Firecracker backend
[features]
firecracker = []

# Cargo.toml [dependencies]
# No crate — talk to Firecracker's REST API directly
```

### Integration sketch

```rust
// src/runner/firecracker.rs
// Firecracker exposes a REST API via a Unix socket or VSOCK

pub struct FirecrackerRunner {
    vmm_socket_path: PathBuf,
    snapshot_dir: PathBuf,
    jailer_binary: PathBuf,
}

impl FirecrackerRunner {
    /// Restore a VM from snapshot (warm start — ~10ms)
    async fn restore_from_snapshot(&self, snapshot_id: &str) -> Result<VmHandle> {
        // PUT /snapshot/load to Firecracker REST API
        // ...
    }
    
    /// Take a snapshot of a warm, booted VM
    async fn take_snapshot(&self, vm: &VmHandle, id: &str) -> Result<()> {
        // PUT /snapshot/create
        // ...
    }
}
```

### Warm pool pattern

```
Boot 10 VMs per language at startup → take snapshot
On request → restore from snapshot (~10ms)
On completion → snapshot again + return to pool
```

---

## 3. Wasmtime (Fallback — All Platforms)

**crate:** `wasmtime` (already a UAR dependency via `wasm-runtime` feature)
**Platform:** All (Linux, macOS, Windows, iOS, Android)

### Role

Wasmtime is the **last-resort fallback** when no microVM is available — primarily for:
- CI environments without KVM
- iOS / Android on-device execution (limited)
- Integration testing without hypervisor infrastructure

### Limitations vs microVM

| | Wasmtime | microsandbox |
|---|---|---|
| Isolation | WASM capability-based | Hardware VM (dedicated kernel) |
| arbitrary `pip install` | ❌ | ✅ |
| `cargo build` | ❌ | ✅ |
| Network access | ❌ (WASI restricted) | ✅ |
| Startup time | <1ms | 150–200ms |
| Platform | Everywhere | Linux (KVM) / macOS (HVF) |

### Supported via Wasmtime

- Pre-compiled Python WASM (via wasm-python / wasmer-python)
- Pre-compiled JavaScript WASM (via QuickJS compiled to WASM)
- Bash scripts (basic builtins via WASI)
- Rust: **not supported** (cannot compile Rust to WASM and re-execute at runtime in a sandbox)

```rust
// src/runner/wasmtime.rs
use wasmtime::{Engine, Module, Store};

pub struct WasmtimeRunner {
    engine: Engine,
    python_module: Module,
    quickjs_module: Module,
}

impl WasmtimeRunner {
    pub fn new() -> Self {
        let engine = Engine::default();
        // Embed wasm modules at compile time
        let python_module = Module::new(&engine, include_bytes!("../wasm/python.wasm")).unwrap();
        let quickjs_module = Module::new(&engine, include_bytes!("../wasm/quickjs.wasm")).unwrap();
        Self { engine, python_module, quickjs_module }
    }
}
```

---

## 4. Remote Runner (Mobile / Restricted Environments)

When local execution is impossible (iOS, Android Play Store, Windows without WSL2), `uar-code-interpreter` delegates to a cloud-hosted instance.

```rust
// src/runner/remote.rs

pub struct RemoteRunner {
    client: reqwest::Client,
    base_url: String,
    auth_token: String,
}

#[async_trait::async_trait]
impl SandboxRunner for RemoteRunner {
    async fn create(&self, config: SandboxConfig) -> Result<SandboxHandle, SandboxError> {
        let resp = self.client
            .post(format!("{}/api/v1/sandboxes", self.base_url))
            .bearer_auth(&self.auth_token)
            .json(&config)
            .send().await?
            .json::<CreateSandboxResponse>().await?;
        
        Ok(SandboxHandle::remote(resp.sandbox_id))
    }
    
    async fn execute(&self, handle: &SandboxHandle, request: ExecutionRequest)
        -> Result<ExecutionResult, SandboxError>
    {
        // Fire-and-forget: output streams via uar-realtime WebSocket
        // connection the mobile client already maintains
        self.client
            .post(format!("{}/api/v1/sandboxes/{}/execute", self.base_url, handle.id))
            .bearer_auth(&self.auth_token)
            .json(&request)
            .send().await?;
        
        // ExecutionResult arrives via uar-realtime sandbox:{session_id}:completed event
        Ok(ExecutionResult::streaming()) // indicates output is on the realtime channel
    }
}
```

**Auth:** Same JWT as UAR auth. The mobile client's token is forwarded — no separate credential.

---

## 5. Runner Config Reference

```yaml
# uar-code-interpreter config.yaml

runner:
  # "auto" = platform detection (default)
  # "microsandbox" = force microsandbox
  # "firecracker" = force Firecracker
  # "wasmtime" = force Wasmtime
  # "remote" = force remote
  backend: auto
  
  microsandbox:
    image_cache_dir: /var/cache/uar-sandbox/images
    default_memory_mib: 512
    default_cpus: 1
    network_enabled: true
    
  firecracker:
    socket_path: /run/firecracker.sock
    jailer_binary: /usr/bin/jailer
    snapshot_dir: /var/cache/uar-sandbox/snapshots
    warm_pool_size: 10    # pre-warmed VMs per language
    
  remote:
    base_url: https://sandbox.example.com
    # auth shared with UAR JWT
    
  timeouts:
    default_seconds: 300   # 5 min default
    max_seconds: 86400      # 24 hours (session sandboxes)
    ephemeral_max_seconds: 300
```
