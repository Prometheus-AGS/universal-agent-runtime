//! Dynamic WebAssembly plugin loader.
//!
//! This is the **contract surface** introduced by KBD change
//! `plugin-loader-wit-contract`. The execution paths are intentionally
//! `todo!()` — a follow-up change wires them to [`super::sandbox`].
//!
//! The WIT world this loader hosts is in `wit/uar-plugin.wit`. Bindings
//! will be generated via `wasmtime::component::bindgen!` in a subsequent
//! change once the contract is stable.

use std::path::PathBuf;

/// Where the plugin's bytes live. The loader resolves this to a
/// [`wasmtime::component::Component`] before instantiation.
#[derive(Debug, Clone)]
pub enum PluginSource {
    /// Raw `.wasm` Component module on disk. Compiled by wasmtime on load
    /// (JIT path).
    Wasm(PathBuf),
    /// Wasmtime-precompiled artifact. Loaded via
    /// `Component::deserialize_file` — no Cranelift pass at runtime.
    Cwasm(PathBuf),
}

/// Execution strategy selected per-plugin by configuration.
///
/// The strategy is orthogonal to the [`PluginSource`]: a `.wasm` source can
/// be JIT-compiled or first compiled to `.cwasm` and cached; a `.cwasm`
/// source skips Cranelift entirely.
#[derive(Debug, Clone, Default)]
pub enum PluginStrategy {
    /// Default. Wasmtime compiles on load with Cranelift. Best for
    /// development and infrequently-invoked plugins.
    #[default]
    Jit,
    /// Ahead-of-time. The host either loads an existing `.cwasm` from
    /// `cache_dir` or precompiles the source on first load and writes it
    /// there. Recommended in production when
    /// `PROMETHEUS_PLUGIN_AOT=1` is set.
    Aot {
        /// Version-scoped cache root. Must match the runtime wasmtime
        /// version (the production `Dockerfile` sets this to
        /// `/var/cache/uar/cwasm/${WASMTIME_VERSION}`).
        cache_dir: PathBuf,
    },
    /// Reserved for future WAMR / wasm3 interpreter integration when a
    /// plugin's resource budget is too tight for Cranelift's working set.
    /// Not implemented in v1.
    Interpreted,
}

/// Capability grant the host promises to a plugin at init time. Mirrors
/// `interface types.capability-grant` in `wit/uar-plugin.wit`.
#[derive(Debug, Clone)]
pub struct CapabilityGrant {
    pub net_outbound: bool,
    pub fs_read: bool,
    pub fs_write: bool,
    pub clock_read: bool,
    pub memory_mb_max: u32,
    pub cpu_ms_max: u32,
}

impl Default for CapabilityGrant {
    /// Deny by default. Hosts must opt-in per capability.
    fn default() -> Self {
        Self {
            net_outbound: false,
            fs_read: false,
            fs_write: false,
            clock_read: false,
            memory_mb_max: 32,
            cpu_ms_max: 5_000,
        }
    }
}

/// Loader configuration assembled by callers and passed to
/// [`PluginLoader::load`].
#[derive(Debug, Clone)]
pub struct LoadRequest {
    pub source: PluginSource,
    pub strategy: PluginStrategy,
    pub grant: CapabilityGrant,
}

/// Handle returned by a successful load. Opaque to callers; the loader
/// owns the underlying `wasmtime::component::Component` and instance pool.
#[derive(Debug, Clone, Copy)]
pub struct PluginId(pub u64);

/// Errors the loader can surface. Mirrors `interface types.plugin-error`
/// in the WIT contract plus host-side load failures.
#[derive(Debug, thiserror::Error)]
pub enum PluginLoadError {
    #[error("cwasm cache miss and JIT disabled for strategy: {0:?}")]
    CacheMissNoFallback(PluginStrategy),
    #[error("wasmtime version mismatch: cwasm built for {built_for}, runtime is {runtime}")]
    CwasmVersionMismatch { built_for: String, runtime: String },
    #[error("capability denied at load: {0}")]
    CapabilityDenied(String),
    #[error("guest trap during init: {0}")]
    GuestTrap(String),
    #[error("interpreted strategy not yet supported in v1")]
    InterpretedNotImplemented,
}

/// Loader surface. Implementations are wired in a follow-up change.
pub trait PluginLoader: Send + Sync {
    /// Load (and instantiate) a plugin according to `req`. Returns a
    /// stable [`PluginId`] the dispatcher uses for subsequent invokes.
    fn load(&self, req: LoadRequest) -> Result<PluginId, PluginLoadError>;

    /// Shut down a previously-loaded plugin. Idempotent.
    fn unload(&self, id: PluginId);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_grant_denies_by_default() {
        let g = CapabilityGrant::default();
        assert!(!g.net_outbound);
        assert!(!g.fs_read);
        assert!(!g.fs_write);
        assert!(!g.clock_read);
        assert_eq!(g.memory_mb_max, 32);
        assert_eq!(g.cpu_ms_max, 5_000);
    }

    #[test]
    fn strategy_defaults_to_jit() {
        assert!(matches!(PluginStrategy::default(), PluginStrategy::Jit));
    }
}
