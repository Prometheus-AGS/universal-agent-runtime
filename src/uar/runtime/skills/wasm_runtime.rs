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

// No derive: `WasiCtx` implements neither Debug nor Default, and Default is
// implemented by hand below so the context is built explicitly.
/// Host state for a wasm skill instance.
///
/// Carries a WASI context because real `wasm32-wasip2` guests import
/// `wasi:cli/*` and `wasi:io/*` whether or not they use them — the linker
/// cannot resolve those imports without it.
///
/// The context is deliberately **empty of capability grants**: no preopened
/// directories, no inherited stdio, no environment. A skill gets what
/// `prometheus:component/capabilities` explicitly hands it and nothing more, so
/// "portable" cannot quietly mean "has ambient filesystem access".
pub struct WasmHostState {
    ctx: wasmtime_wasi::WasiCtx,
    table: wasmtime::component::ResourceTable,
    /// Backing store for `prometheus:component/kv-store`.
    ///
    /// Per-instance and in-memory: a skill gets a scratchpad that dies with the
    /// call. Sharing one map across skills would let an untrusted guest read
    /// another's state, and persisting it would make a "portable" skill quietly
    /// stateful across invocations.
    kv: std::collections::HashMap<String, String>,
}

impl std::fmt::Debug for WasmHostState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasmHostState")
            .field("kv_entries", &self.kv.len())
            .finish_non_exhaustive()
    }
}

impl Default for WasmHostState {
    fn default() -> Self {
        Self {
            ctx: wasmtime_wasi::WasiCtxBuilder::new().build(),
            table: wasmtime::component::ResourceTable::new(),
            kv: std::collections::HashMap::new(),
        }
    }
}

impl wasmtime_wasi::WasiView for WasmHostState {
    fn ctx(&mut self) -> wasmtime_wasi::WasiCtxView<'_> {
        wasmtime_wasi::WasiCtxView {
            ctx: &mut self.ctx,
            table: &mut self.table,
        }
    }
}

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

    /// Wire `prometheus:component/kv-store@0.1.0` into a component linker.
    ///
    /// Split out so the capability surface is readable in one place: whatever a
    /// skill can reach, it reaches through a function listed here.
    fn link_kv_store(linker: &mut Linker<WasmHostState>) -> Result<()> {
        use wasmtime::component::Val;

        // `.instance()` yields a wasmtime::Result, which anyhow's blanket
        // Context impl does not extend to — map it explicitly rather than
        // reaching for a trait that does not apply.
        let mut iface = linker
            .instance("prometheus:component/kv-store@0.1.0")
            .map_err(|e| anyhow::anyhow!("define the kv-store instance: {e}"))?;

        iface
            .func_new("get", |store, _ty, params, results| {
                let Some(Val::String(key)) = params.first() else {
                    return Err(wasmtime::Error::msg("kv-store.get expects a string key"));
                };
                let found = store.data().kv.get(key.as_str()).cloned();
                // result<option<string>, error>
                let inner = match found {
                    Some(v) => Val::Option(Some(Box::new(Val::String(v)))),
                    None => Val::Option(None),
                };
                results[0] = Val::Result(Ok(Some(Box::new(inner))));
                Ok(())
            })
            .map_err(|e| anyhow::anyhow!("link kv-store.get: {e}"))?;

        iface
            .func_new("set", |mut store, _ty, params, results| {
                let (Some(Val::String(k)), Some(Val::String(v))) = (params.first(), params.get(1))
                else {
                    return Err(wasmtime::Error::msg(
                        "kv-store.set expects (string, string)",
                    ));
                };
                store.data_mut().kv.insert(k.clone(), v.clone());
                results[0] = Val::Result(Ok(None));
                Ok(())
            })
            .map_err(|e| anyhow::anyhow!("link kv-store.set: {e}"))?;

        iface
            .func_new("delete", |mut store, _ty, params, results| {
                let Some(Val::String(k)) = params.first() else {
                    return Err(wasmtime::Error::msg("kv-store.delete expects a string key"));
                };
                store.data_mut().kv.remove(k.as_str());
                results[0] = Val::Result(Ok(None));
                Ok(())
            })
            .map_err(|e| anyhow::anyhow!("link kv-store.delete: {e}"))?;

        Ok(())
    }

    /// Invoke a registered skill's `run` export.
    ///
    /// `run(input: string) -> result<string, string>` is the only required
    /// export on the `uar:skill@0.1.0` world. Bindgen is intentionally not used
    /// here so this runtime can host components even when the WIT bindings
    /// drift — error reporting is best-effort.
    pub async fn run(&self, skill_id: &str, input: &str) -> Result<String> {
        let components = self.components.lock().await;
        let component = components
            .get(skill_id)
            .ok_or_else(|| anyhow::anyhow!("wasm skill not loaded: {skill_id}"))?
            .clone();
        drop(components);

        let mut store = Store::new(&self.engine, WasmHostState::default());
        let mut linker: Linker<WasmHostState> = Linker::new(&self.engine);

        // The reference component imports `wasi:cli/*` and `wasi:io/*` (a
        // wasm32-wasip2 guest does even when it never reads stdin). Without
        // these the instantiate call fails on unresolved imports, so WASI is a
        // requirement of hosting real guests, not an optional nicety.
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker)
            .context("add wasi to the component linker")?;

        // `prometheus:component/kv-store` — a real host capability, not a stub.
        //
        // Implemented by hand rather than through bindgen so the WIT stays the
        // contract: a guest whose interface has drifted fails at instantiate
        // with a clear "wrong type" message instead of failing to compile
        // generated code somewhere else.
        //
        // Values are `result<..., error>` where `error` is
        // `record { kind: error-kind, message: string }`. These operations
        // cannot fail against an in-memory map, so they always return `ok` —
        // but the shape must still match or instantiation is rejected.
        // This file imports wasmtime's `Context`, not anyhow's, so a plain
        // `.context()` on an anyhow::Result does not resolve here.
        Self::link_kv_store(&mut linker)
            .map_err(|e| anyhow::anyhow!("add prometheus:component/kv-store to the linker: {e}"))?;

        let instance = linker
            .instantiate(&mut store, &component)
            .context("instantiate the wasm skill component")?;

        // `run(input: string) -> result<string, error>` — the one required
        // export on the skill world. Looked up by name rather than through
        // bindgen so a guest whose WIT has drifted fails with a clear "no such
        // export" instead of a link error in generated code.
        let func = instance
            .get_func(&mut store, "run")
            .ok_or_else(|| anyhow::anyhow!(
                "wasm skill '{skill_id}' exports no `run` function;                  the skill world requires run(string) -> result<string, error>"
            ))?;

        let mut results = [wasmtime::component::Val::Bool(false)];
        func.call(
            &mut store,
            &[wasmtime::component::Val::String(input.to_string())],
            &mut results,
        )
        .context("call the wasm skill's `run` export")?;

        // The guest returns `result<string, error>`. Unwrap it here so a guest
        // error surfaces as a Rust error rather than as a success carrying an
        // error payload — a caller that ignores the discriminant would
        // otherwise treat a failure as output.
        match &results[0] {
            wasmtime::component::Val::Result(Ok(Some(v))) => match v.as_ref() {
                wasmtime::component::Val::String(out) => Ok(out.clone()),
                other => Err(anyhow::anyhow!(
                    "wasm skill '{skill_id}' returned ok({other:?}), expected a string"
                )),
            },
            wasmtime::component::Val::Result(Err(e)) => Err(anyhow::anyhow!(
                "wasm skill '{skill_id}' returned an error: {e:?}"
            )),
            other => Err(anyhow::anyhow!(
                "wasm skill '{skill_id}' returned {other:?}, expected result<string, error>"
            )),
        }
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
