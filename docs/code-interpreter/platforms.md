# UAR Code Interpreter — Platform Support

_Last updated: 2026-02-21_

The code interpreter supports five platform families. Each uses the best available sandbox runner that the platform's security model and OS permits.

---

## Platform Summary

| Platform | Runner | Isolation level | Languages | Notes |
|---|---|---|---|---|
| **Linux x86_64** | microsandbox (KVM) | Hardware VM | All | Full feature set |
| **Linux aarch64** | microsandbox (KVM) | Hardware VM | All | Raspberry Pi, ARM servers |
| **macOS Apple Silicon** | microsandbox (HVF) | Hardware VM | All | Developer default |
| **macOS Intel** | microsandbox (KVM via HVF) | Hardware VM | All | Older Macs |
| **Windows (WSL2)** | microsandbox in WSL2 | Hardware VM | All | Requires WSL2 enabled |
| **Windows (native)** | Remote execution | Network isolation | All | Calls cloud service |
| **iOS** | Remote execution | Network isolation | All | App Store restrictions |
| **Android (device)** | Remote execution | Network isolation | All | Play Store restrictions |
| **Android (dev/root)** | microsandbox (KVM) | Hardware VM | All | KVM available on select devices |

---

## 1. Linux

### Full microVM support

Linux is the primary deployment target. KVM is available on all modern desktop and server hardware.

```toml
# uar-code-interpreter/Cargo.toml
[target.'cfg(target_os = "linux")'.dependencies]
microsandbox = { version = "0.x", features = ["kvm"] }
```

**Kernel requirement:** Linux 5.10+ (KVM enabled). This is satisfied by all major distros from 2021 onwards.

**What works:**
- All four languages at full performance
- All execution modes (ephemeral, session, project, swarm)
- VM networking (sandboxes can fetch packages from the internet)
- Long-running processes without timeout (configurable)
- VM snapshotting (when Firecracker backend is enabled)

### CI / Container

When running inside Docker or a container without KVM access (e.g., standard GitHub Actions runner):

```rust
// Runtime detection in runner selection
pub fn select_runner(config: &AppConfig) -> Arc<dyn SandboxRunner> {
    if kvm_available() {
        Arc::new(MicrosandboxRunner::new())
    } else {
        // Graceful degradation: use Wasmtime for supported languages
        // or fail explicitly for unsupported operations
        tracing::warn!("KVM not available — using Wasmtime fallback (limited language support)");
        Arc::new(WasmtimeRunner::new())
    }
}

fn kvm_available() -> bool {
    std::path::Path::new("/dev/kvm").exists()
}
```

---

## 2. macOS

### Apple Silicon (M1/M2/M3/M4)

libkrun (the library microsandbox is built on) uses **Apple's Hypervisor Framework (HVF)** on ARM macOS. This provides native virtualization without KVM.

```toml
[target.'cfg(all(target_os = "macos", target_arch = "aarch64"))'.dependencies]
microsandbox = { version = "0.x", features = ["hvf"] }
```

- **Performance:** Near-native. HVF is Apple's official hypervisor API, tuned for Apple Silicon.
- **Startup time:** ~150–200ms (same as Linux KVM).
- **All four languages** supported at full capability.
- **No admin/root required** — HVF is available to unprivileged processes on macOS.

### Intel macOS

Intel Macs also support HVF (though less optimal than Apple Silicon native).

- Same `hvf` feature applies.
- Identical capabilities to Apple Silicon.

### macOS in Tauri desktop mode

The `uar-code-interpreter` runs as a **Tauri sidecar binary** (bundled alongside the app):

```json
// src-tauri/tauri.conf.json
{
  "bundle": {
    "externalBin": [
      "binaries/uar-code-interpreter"
    ]
  }
}
```

UAR spawns and manages the sidecar process lifecycle. Sidecar communicates via internal HTTP on `localhost:5002` (internal port, not exposed).

```toml
# src-tauri/Cargo.toml — Tauri command to spawn sidecar
[dependencies]
tauri = { version = "2", features = ["shell-sidecar"] }
```

---

## 3. Windows

### WSL2 (Recommended for Development)

WSL2 runs a full Linux kernel inside Hyper-V — KVM is available inside WSL2 on modern Windows 11.

```
Windows 11
  └── Hyper-V
        └── WSL2 Linux VM
              └── uar-code-interpreter (Linux binary, full KVM support)
```

**Setup for developers:**
```powershell
# Enable WSL2 and KVM passthrough
wsl --install
# uar-code-interpreter runs as a Linux binary inside WSL2
# UAR on Windows communicates via WSL2 network bridge (127.0.0.1:5001)
```

**Result:** Full microsandbox support through WSL2. All four languages at full capability.

### Native Windows (No WSL2)

Without WSL2, microVMs are not straightforwardly available. The fallback is **remote execution**:

```rust
// src/runner/remote.rs — on native Windows without WSL2
// UAR calls the cloud-hosted uar-code-interpreter service
impl SandboxRunner for RemoteRunner {
    async fn execute(&self, handle: &SandboxHandle, request: ExecutionRequest) 
        -> Result<ExecutionResult, SandboxError> 
    {
        self.http_client
            .post(format!("{}/api/v1/execute", self.base_url))
            .bearer_auth(&self.token)
            .json(&request)
            .send()
            .await?
            .json::<ExecutionResult>()
            .await
    }
}
```

**Future:** Native Windows Hyper-V integration is a P4 item. libkrun does not currently support Windows natively.

---

## 4. iOS

### Platform constraints

Apple's App Store distribution model prohibits:
- **JIT compilation** (required for interpreters in the traditional sense)
- **Hypervisor access** (no KVM/HVF available to App Store apps)
- **Arbitrary code downloading and execution**

This rules out running microsandbox, a Python interpreter, or Node.js directly inside an iOS app distributed through the App Store.

### Solution: Remote execution

In iOS mode, `uar-code-interpreter` is **not deployed on-device**. The UAR Tauri/mobile client calls the **cloud-hosted** `uar-code-interpreter` service:

```
iOS app (Tauri mobile / native)
  └── UAR SDK
        └── HTTP POST /api/v1/execute
              └── cloud uar-code-interpreter (Linux, full KVM)
                    └── microVM sandbox
                          └── code runs
                          └── output streams back via uar-realtime WebSocket
```

The iOS app shows the streaming output in real-time over the WebSocket connection it already maintains with `uar-realtime`.

### Wasmtime fallback (restricted / enterprise mode)

For enterprise deployments where cloud connectivity is restricted, a **limited Wasmtime sandbox** can execute pre-compiled WASM modules:

```toml
# Mobile-targeted build (no microVM, Wasmtime only)
[features]
mobile = ["wasmtime-runtime"]
```

**Supports:** WASM-compiled Python (via wasm-python), WASM-compiled JavaScript (via wasm-quickjs). **Does not support** arbitrary `cargo build` or full `pip install`.

---

## 5. Android

### Google Play distribution

Same constraints as iOS for Play Store distributed apps — no JIT, no hypervisor.

**Solution:** Same remote execution pattern as iOS.

### Developer devices / sideloaded apps

On Android devices with root or on test devices, **KVM is available on aarch64 hardware** (Google Pixel, many Qualcomm devices):

```bash
# Check if KVM is available on Android
ls /dev/kvm
# If present, microsandbox can run natively
```

For these cases, the ARM Linux build of `uar-code-interpreter` can run directly on-device as a background service. This is practical for:
- Developer tooling apps
- Enterprise/corporate-managed device fleets
- Android tablet developer environments

---

## 6. Runner Selection Logic

`uar-code-interpreter` selects the best available runner automatically at startup:

```rust
// src/runner/mod.rs

pub fn build_runner(config: &AppConfig) -> Arc<dyn SandboxRunner> {
    // Explicit override from config
    if let Some(runner) = &config.forced_runner {
        return build_specific_runner(runner, config);
    }

    #[cfg(target_os = "linux")]
    if kvm_available() {
        return Arc::new(MicrosandboxRunner::new(RunnerConfig::kvm()));
    }

    #[cfg(all(target_os = "macos"))]
    if hvf_available() {
        return Arc::new(MicrosandboxRunner::new(RunnerConfig::hvf()));
    }

    // Remote runner if external URL is configured (mobile / Windows native)
    if let Some(url) = &config.remote_execution_url {
        return Arc::new(RemoteRunner::new(url, &config.auth_token));
    }

    // Wasmtime fallback (limited capabilities)
    #[cfg(feature = "wasm-fallback")]
    {
        tracing::warn!("No microVM support available — using Wasmtime (limited language support)");
        return Arc::new(WasmtimeRunner::new());
    }

    panic!("No sandbox runner available for this platform. Configure UAR_SANDBOX_REMOTE_URL for remote execution.");
}
```

---

## 7. Platform Capability Matrix

| Feature | Linux | macOS | Win/WSL2 | Win/native | iOS | Android |
|---|:---:|:---:|:---:|:---:|:---:|:---:|
| Hardware VM isolation | ✅ | ✅ | ✅ | ❌ | ❌ | ⚠️ |
| Bash | ✅ | ✅ | ✅ | 🌐 | 🌐 | 🌐 |
| Rust (`cargo build`) | ✅ | ✅ | ✅ | 🌐 | 🌐 | 🌐 |
| Python | ✅ | ✅ | ✅ | 🌐 | 🌐 | 🌐 |
| Node.js | ✅ | ✅ | ✅ | 🌐 | 🌐 | 🌐 |
| Session mode | ✅ | ✅ | ✅ | 🌐 | 🌐 | 🌐 |
| Project mode | ✅ | ✅ | ✅ | 🌐 | 🌐 | 🌐 |
| Swarm mode | ✅ | ✅ | ✅ | 🌐 | 🌐 | 🌐 |
| Realtime streaming | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Offline operation | ✅ | ✅ | ✅ | ⚠️ | ❌ | ❌ |

**Legend:** ✅ = native, 🌐 = remote (cloud service), ⚠️ = partial, ❌ = not supported
