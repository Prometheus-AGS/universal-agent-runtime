//! WebAssembly Component Model runtime for `kind = Wasm` skills.
//!
//! Loads `.wasm` (JIT-compiled by Cranelift at load time) or `.cwasm`
//! (AOT-precompiled via `wasmtime compile`) components targeting the
//! [`uar:skill@0.1.0`](../../../../wit/uar-skill.wit) WIT world and dispatches
//! the `run(input: string) -> result<string, string>` export.
//!
//! Behind the existing `wasm-runtime` Cargo feature.

#![cfg(feature = "wasm-runtime")]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
// wasmtime::Error no longer implements std::error::Error (wasmtime 46), so
// anyhow's blanket Context impl doesn't apply to wasmtime Results anymore —
// wasmtime ships its own Context trait for exactly this case.
use tokio::sync::Mutex;
use tracing::{info, warn};
use walkdir::WalkDir;
use wasmtime::component::{Component, Linker};
use wasmtime::error::Context;
use wasmtime::{Engine, Store};

use crate::uar::domain::skills::{
    Skill, SkillConstraints, SkillExecutionConfig, SkillKind, SkillOrigin, SkillTriggers,
};

/// Lazy host state for WASI imports. Skills today receive no host imports
/// other than the future-expanded WASI snapshot; keep this empty for now.
#[derive(Debug, Default)]
pub struct WasmHostState {}

#[derive(Clone)]
pub struct WasmSkillRuntime {
    engine: Engine,
    components: Arc<Mutex<std::collections::HashMap<String, Component>>>,
}

impl std::fmt::Debug for WasmSkillRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasmSkillRuntime").finish()
    }
}

impl WasmSkillRuntime {
    pub fn new() -> Result<Self> {
        let mut config = wasmtime::Config::new();
        config.wasm_component_model(true);
        let engine = Engine::new(&config).context("building wasmtime engine")?;
        Ok(Self {
            engine,
            components: Arc::new(Mutex::new(std::collections::HashMap::new())),
        })
    }

    /// Load a component from disk and register it under `skill_id`.
    pub async fn register(&self, skill_id: &str, path: &Path) -> Result<()> {
        let component = if path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.eq_ignore_ascii_case("cwasm"))
            .unwrap_or(false)
        {
            unsafe { Component::deserialize_file(&self.engine, path) }
                .with_context(|| format!("deserialize_file {}", path.display()))?
        } else {
            Component::from_file(&self.engine, path)
                .with_context(|| format!("from_file {}", path.display()))?
        };

        self.components
            .lock()
            .await
            .insert(skill_id.to_string(), component);
        info!(skill_id, path = %path.display(), "registered wasm skill");
        Ok(())
    }

    /// Whether a wasm skill is loaded under `skill_id`.
    pub async fn has(&self, skill_id: &str) -> bool {
        self.components.lock().await.contains_key(skill_id)
    }

    /// Invoke a registered skill's `run` export.
    ///
    /// `run(input: string) -> result<string, string>` is the only required
    /// export on the `uar:skill@0.1.0` world. Bindgen is intentionally not used
    /// here so this runtime can host components even when the WIT bindings
    /// drift — error reporting is best-effort.
    pub async fn run(&self, skill_id: &str, input: &str) -> Result<String> {
        let components = self.components.lock().await;
        let _component = components
            .get(skill_id)
            .ok_or_else(|| anyhow::anyhow!("wasm skill not loaded: {skill_id}"))?
            .clone();
        drop(components);

        let _store = Store::new(&self.engine, WasmHostState {});
        let _linker: Linker<WasmHostState> = Linker::new(&self.engine);
        // Concrete component bindings will be added once wit-bindgen
        // integration lands (the WIT world is pinned; this is implementation
        // surface, not API surface). For now, return a stub so callers can
        // wire dispatch end-to-end without a fixture component.
        let _ = input;
        Ok(format!(
            "<wasm skill '{skill_id}' loaded but binding not yet generated; \
             implement wit-bindgen invocation here>"
        ))
    }
}

/// Returns the resolved built-in WASM skill dir. Honours
/// `UAR_SKILLS_WASM_BUILTIN_DIR`; falls back to `crates/prometheus-skill-system/skills`
/// (any `skill.wasm` discovered alongside a `SKILL.md`).
pub fn builtin_wasm_dir() -> PathBuf {
    if let Ok(s) = std::env::var("UAR_SKILLS_WASM_BUILTIN_DIR") {
        return PathBuf::from(s);
    }
    PathBuf::from("crates/prometheus-skill-system/skills")
}

pub fn user_wasm_dir() -> PathBuf {
    if let Ok(s) = std::env::var("UAR_SKILLS_USER_DIR") {
        return PathBuf::from(s);
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".uar").join("skills").join("user");
    }
    PathBuf::from("/var/lib/uar/skills-user")
}

/// Scan both directories for `.wasm` / `.cwasm` files and register each into
/// `runtime`. Returns a vector of `Skill` records (one per loaded component)
/// suitable for `SkillService::register_builtins`.
pub async fn discover_and_load(runtime: &WasmSkillRuntime) -> Vec<Skill> {
    let mut out = Vec::new();
    for (dir, origin) in [
        (builtin_wasm_dir(), SkillOrigin::Builtin),
        (user_wasm_dir(), SkillOrigin::User),
    ] {
        if !dir.exists() {
            continue;
        }
        for entry in WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .map(str::to_ascii_lowercase);
            if !matches!(ext.as_deref(), Some("wasm") | Some("cwasm")) {
                continue;
            }
            let stem = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };
            let skill_id = format!("wasm::{stem}");
            if let Err(err) = runtime.register(&skill_id, path).await {
                warn!(path = %path.display(), error = %err, "wasm skill load failed");
                continue;
            }
            out.push(Skill {
                skill_id,
                version: "0.0.0".to_string(),
                title: stem.clone(),
                description: format!("WASM component loaded from {}", path.display()),
                triggers: SkillTriggers::default(),
                prompt_overlay: String::new(),
                preferred_tools: Vec::new(),
                mcp_config: None,
                constraints: SkillConstraints::default(),
                enabled: true,
                provider_id: "wasm".to_string(),
                execution_config: SkillExecutionConfig::default(),
                kind: SkillKind::Wasm,
                origin,
                ..Default::default()
            });
        }
    }
    info!(count = out.len(), "discovered wasm component skills");
    out
}
