//! Configuration types for the WebAssembly sandbox.

use serde::{Deserialize, Serialize};

/// Configuration for a Wasm sandbox execution environment.
///
/// Controls resource limits, timeouts, and capability grants for
/// sandboxed Wasm modules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmConfig {
    /// Maximum linear memory pages (64 KiB each). Default: 256 (16 MiB).
    #[serde(default = "default_max_memory_pages")]
    pub max_memory_pages: u64,

    /// Maximum fuel (instruction count) before execution is interrupted.
    /// `None` means unlimited. Default: `10_000_000`.
    #[serde(default = "default_max_fuel")]
    pub max_fuel: Option<u64>,

    /// Whether to grant the guest access to WASI filesystem capabilities.
    /// Default: false.
    #[serde(default)]
    pub allow_filesystem: bool,

    /// Whether to grant the guest access to WASI networking capabilities.
    /// Default: false.
    #[serde(default)]
    pub allow_networking: bool,

    /// Whether to grant the guest access to environment variables.
    /// Default: false.
    #[serde(default)]
    pub allow_env: bool,

    /// Directories to pre-open for filesystem access (only if `allow_filesystem` is true).
    #[serde(default)]
    pub preopened_dirs: Vec<String>,

    /// Environment variables to pass to the guest (only if `allow_env` is true).
    #[serde(default)]
    pub env_vars: Vec<(String, String)>,
}

impl Default for WasmConfig {
    fn default() -> Self {
        Self {
            max_memory_pages: default_max_memory_pages(),
            max_fuel: default_max_fuel(),
            allow_filesystem: false,
            allow_networking: false,
            allow_env: false,
            preopened_dirs: Vec::new(),
            env_vars: Vec::new(),
        }
    }
}

fn default_max_memory_pages() -> u64 {
    256 // 16 MiB
}

#[allow(clippy::unnecessary_wraps)] // serde default requires matching `Option<u64>` return type
fn default_max_fuel() -> Option<u64> {
    Some(10_000_000)
}

// Note: `default_max_fuel` intentionally wraps in Option because
// serde defaults require matching types.
