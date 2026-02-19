# WASM Agent Runtime

UAR includes an optional **WebAssembly sandbox** for executing Wasm
agents in a secure, resource-limited environment. It is powered by
[Wasmtime](https://wasmtime.dev/) and supports WASI Preview 1 for
filesystem, networking, and standard I/O capabilities.

## Enabling the Feature

The Wasm runtime is **feature-gated**. To include it in your build:

```bash
cargo build --features wasm-runtime
cargo test  --features wasm-runtime
```

When the feature is disabled (the default), the entire `wasm` module is
excluded at compile time — zero overhead.

## Architecture

```
┌───────────────────────────────────────────┐
│              WasmSandbox                  │
│  ┌────────┐  ┌─────────┐  ┌───────────┐  │
│  │ Engine │  │  Store   │  │  Linker   │  │
│  │(shared)│  │(per-run) │  │(WASI+UAR) │  │
│  └────────┘  └─────────┘  └───────────┘  │
│                    │                      │
│         ┌─────────┴──────────┐            │
│         │   SandboxState     │            │
│         │  ┌──────────────┐  │            │
│         │  │  WasiP1Ctx   │  │            │
│         │  │  WasmConfig  │  │            │
│         │  └──────────────┘  │            │
│         └────────────────────┘            │
└───────────────────────────────────────────┘
```

| Component | Purpose |
|-----------|---------|
| `WasmConfig` | Memory limits, fuel budget, WASI capability grants |
| `WasmSandbox` | Core engine — compiles and executes Wasm modules |
| `SandboxState` | Per-invocation state with WASI context |
| `host_functions` | UAR-specific imports (`uar_log`, `uar_emit_event`) |

## Configuration

`WasmConfig` controls the sandbox at creation time:

```rust
use universal_agent_runtime::uar::runtime::wasm::config::WasmConfig;

let config = WasmConfig {
    max_memory_pages: 256,       // 16 MiB (64 KiB per page)
    max_fuel: Some(10_000_000),  // instruction budget
    allow_filesystem: false,
    allow_networking: false,
    allow_env: false,
    preopened_dirs: vec![],
    env_vars: vec![],
};
```

### Fuel Metering

Setting `max_fuel` enables deterministic resource limiting. Each Wasm
instruction consumes one fuel unit. When fuel runs out, execution is
terminated with a clear error message. Set to `None` for unlimited
execution.

## Host Functions

Wasm guests can import these functions from the `"uar"` module:

| Function | Signature | Description |
|----------|-----------|-------------|
| `uar_log` | `(ptr: i32, len: i32)` | Log a UTF-8 message |
| `uar_emit_event` | `(ptr: i32, len: i32)` | Emit a JSON event |

Both functions read a UTF-8 string from guest linear memory at
`[ptr..ptr+len]`.

## Usage Example

```rust
use universal_agent_runtime::uar::runtime::wasm::{
    config::WasmConfig,
    sandbox::WasmSandbox,
};

let sandbox = WasmSandbox::new(WasmConfig::default())?;
let result = sandbox.execute_bytes(wasm_bytes, "_start").await?;

if result.success {
    println!("Execution succeeded, fuel used: {:?}", result.fuel_consumed);
    for log in &result.logs {
        println!("  Guest log: {log}");
    }
} else {
    eprintln!("Execution failed: {:?}", result.error);
}
```

## Security Model

- **Deny-by-default**: All WASI capabilities (filesystem, networking,
  environment variables) are disabled unless explicitly enabled in
  `WasmConfig`.
- **Fuel metering**: Prevents infinite loops and runaway computation.
- **Separate stores**: Each execution gets its own `Store`, providing
  full memory isolation between invocations.
- **No shared state**: Guest modules cannot access host memory directly;
  only UAR host functions provide a controlled communication channel.

## Compiling Wasm Modules

Any language that compiles to WASI can produce modules for this sandbox.
For example, using Rust:

```bash
rustup target add wasm32-wasip1
cargo build --target wasm32-wasip1 --release
# Output: target/wasm32-wasip1/release/my_agent.wasm
```
