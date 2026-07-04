//! WebAssembly sandbox execution engine.
//!
//! [`WasmSandbox`] provides a secure, resource-limited execution environment
//! for running Wasm modules with optional WASI capabilities.

// wasmtime::Error no longer implements std::error::Error (wasmtime 46), so
// anyhow's blanket Context impl doesn't apply to wasmtime Results anymore —
// wasmtime ships its own Context trait for exactly this case.
use tracing::{debug, info};
use wasmtime::error::Context as _;
use wasmtime::{Config, Engine, Linker, Module, Store};
use wasmtime_wasi::{WasiCtxBuilder, p1::WasiP1Ctx};

use super::config::WasmConfig;
use super::host_functions::{self, GuestLogs};

// ---------------------------------------------------------------------------
// Sandbox store state
// ---------------------------------------------------------------------------

/// State stored in each [`Store`] — includes WASI P1 context and config.
pub struct SandboxState {
    /// WASI P1 context for core module execution.
    pub wasi: WasiP1Ctx,
    /// Original config for reference.
    #[allow(dead_code)]
    config: WasmConfig,
}

impl std::fmt::Debug for SandboxState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SandboxState")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

// ---------------------------------------------------------------------------
// WasmSandbox
// ---------------------------------------------------------------------------

/// A reusable Wasm execution sandbox.
///
/// The sandbox manages a wasmtime [`Engine`] and can execute multiple
/// modules across separate [`Store`]s with independent resource limits.
///
/// # Example
///
/// ```rust,ignore
/// use uar::runtime::wasm::{config::WasmConfig, sandbox::WasmSandbox};
///
/// let sandbox = WasmSandbox::new(WasmConfig::default())?;
/// let result = sandbox.execute_bytes(wasm_bytes, "_start").await?;
/// ```
pub struct WasmSandbox {
    /// Pre-configured Wasmtime engine (compiled modules cache here).
    engine: Engine,
    /// Default config for new stores.
    default_config: WasmConfig,
}

impl std::fmt::Debug for WasmSandbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasmSandbox")
            .field("config", &self.default_config)
            .finish()
    }
}

impl WasmSandbox {
    /// Create a new sandbox with the given configuration.
    pub fn new(config: WasmConfig) -> anyhow::Result<Self> {
        let mut engine_config = Config::new();

        // Enable fuel-based metering if configured
        if config.max_fuel.is_some() {
            engine_config.consume_fuel(true);
        }

        // wasmtime 46: `Config::async_support` is deprecated and no longer has
        // any effect -- async support is no longer an opt-in Config toggle.

        let engine = Engine::new(&engine_config).context("Failed to create Wasmtime engine")?;

        info!("Wasm sandbox initialized");

        Ok(Self {
            engine,
            default_config: config,
        })
    }

    /// Execute a Wasm module from raw bytes.
    ///
    /// # Arguments
    ///
    /// * `wasm_bytes` — The compiled `.wasm` file bytes.
    /// * `entry_point` — The exported function name to call (e.g., `"_start"` for WASI).
    ///
    /// # Returns
    ///
    /// A [`WasmExecutionResult`] containing the exit status and collected logs.
    pub async fn execute_bytes(
        &self,
        wasm_bytes: &[u8],
        entry_point: &str,
    ) -> anyhow::Result<WasmExecutionResult> {
        self.execute_bytes_with_config(wasm_bytes, entry_point, &self.default_config)
            .await
    }

    /// Execute a Wasm module with a custom config override.
    pub async fn execute_bytes_with_config(
        &self,
        wasm_bytes: &[u8],
        entry_point: &str,
        config: &WasmConfig,
    ) -> anyhow::Result<WasmExecutionResult> {
        let module =
            Module::new(&self.engine, wasm_bytes).context("Failed to compile Wasm module")?;

        self.execute_module(&module, entry_point, config).await
    }

    /// Execute a pre-compiled Wasm module.
    async fn execute_module(
        &self,
        module: &Module,
        entry_point: &str,
        config: &WasmConfig,
    ) -> anyhow::Result<WasmExecutionResult> {
        // Build WASI context
        let mut wasi_builder = WasiCtxBuilder::new();

        // Configure standard I/O to inherit (captured by tracing)
        wasi_builder.inherit_stdio();

        // Configure environment variables
        if config.allow_env {
            for (key, value) in &config.env_vars {
                wasi_builder.env(key, value);
            }
        }

        // Configure filesystem access
        if config.allow_filesystem {
            for dir_path in &config.preopened_dirs {
                let dir = wasmtime_wasi::DirPerms::all();
                let file = wasmtime_wasi::FilePerms::all();
                wasi_builder.preopened_dir(dir_path, dir_path, dir, file)?;
            }
        }

        // Configure networking
        if config.allow_networking {
            wasi_builder.inherit_network();
        }

        // Build WasiP1Ctx for core module execution
        let wasi_p1 = wasi_builder.build_p1();

        let state = SandboxState {
            wasi: wasi_p1,
            config: config.clone(),
        };

        let mut store = Store::new(&self.engine, state);

        // Set fuel limit
        if let Some(fuel) = config.max_fuel {
            store.set_fuel(fuel)?;
        }

        // Create linker and register WASI P1 functions + UAR host functions
        let mut linker = Linker::new(&self.engine);
        wasmtime_wasi::p1::add_to_linker_async(&mut linker, |s: &mut SandboxState| &mut s.wasi)?;

        let logs = GuestLogs::new();
        host_functions::register_host_functions(&mut linker, logs.clone())?;

        // Instantiate and run
        let instance = linker
            .instantiate_async(&mut store, module)
            .await
            .context("Failed to instantiate Wasm module")?;

        let func = instance
            .get_typed_func::<(), ()>(&mut store, entry_point)
            .context(format!(
                "Entry point '{entry_point}' not found or has wrong signature",
            ))?;

        debug!(entry_point = entry_point, "Executing Wasm module");

        let execution_error = match func.call_async(&mut store, ()).await {
            Ok(()) => None,
            Err(e) => {
                // Check if it was a fuel exhaustion (resource limit)
                let msg = e.to_string();
                if msg.contains("fuel") {
                    Some("Execution terminated: fuel limit exceeded".to_string())
                } else {
                    Some(format!("Execution error: {e}"))
                }
            }
        };

        // Collect remaining fuel info
        let fuel_remaining = store.get_fuel().ok();

        Ok(WasmExecutionResult {
            success: execution_error.is_none(),
            error: execution_error,
            logs: logs.drain(),
            fuel_consumed: config
                .max_fuel
                .and_then(|max| fuel_remaining.map(|rem| max.saturating_sub(rem))),
        })
    }

    /// Get a reference to the Wasmtime engine.
    #[must_use]
    pub fn engine(&self) -> &Engine {
        &self.engine
    }
}

// ---------------------------------------------------------------------------
// Execution result
// ---------------------------------------------------------------------------

/// Result of executing a Wasm module in the sandbox.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WasmExecutionResult {
    /// Whether execution completed without error.
    pub success: bool,
    /// Error message if execution failed.
    pub error: Option<String>,
    /// Log messages collected from the guest via `uar_log`.
    pub logs: Vec<String>,
    /// Number of fuel units consumed (if fuel metering was enabled).
    pub fuel_consumed: Option<u64>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sandbox_creation() {
        let config = WasmConfig::default();
        let sandbox = WasmSandbox::new(config);
        assert!(sandbox.is_ok(), "Sandbox should be created successfully");
    }

    #[tokio::test]
    async fn test_sandbox_debug_display() {
        let config = WasmConfig::default();
        let sandbox = WasmSandbox::new(config).unwrap();
        let debug = format!("{:?}", sandbox);
        assert!(debug.contains("WasmSandbox"));
    }
}
