use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=proto/a2a.proto");
    println!("cargo:rerun-if-env-changed=SKIP_MODEL_BUILD");
    println!("cargo:rerun-if-env-changed=SKIP_FRONTEND_BUILD");
    println!("cargo:rerun-if-env-changed=SKIP_CATALOG_BUILD");

    // -------------------------------------------------------------------------
    // gRPC Proto Compilation (A2A v0.3)
    // -------------------------------------------------------------------------
    // A2A v0.3 gRPC: Proto file is at proto/a2a.proto for reference.
    // Code generation requires tonic-build with prost feature which is
    // undergoing API changes in v0.14. The gRPC service is defined manually
    // in src/uar/api/a2a/grpc.rs using tonic's manual service builder.
    // To enable auto-generation, add `tonic-build = { version = "0.14", features = ["prost"] }`
    // to [build-dependencies] and uncomment the compile_protos call below.
    // if Path::new("proto/a2a.proto").exists() {
    //     tonic_build::compile_protos("proto/a2a.proto")
    //         .expect("Failed to compile A2A proto");
    // }

    // -------------------------------------------------------------------------
    // Provider / Model Catalog
    // -------------------------------------------------------------------------
    // Merges liter-llm schemas/providers.json (routing / auth) with
    // models.dev/api.json (capabilities / pricing) into a single embedded
    // artifact at $OUT_DIR/provider_catalog.json.
    if env::var("SKIP_CATALOG_BUILD").is_err() {
        build_provider_catalog();
    } else {
        create_stub_catalog();
    }

    // -------------------------------------------------------------------------
    // Frontend Build
    // -------------------------------------------------------------------------
    if env::var("SKIP_FRONTEND_BUILD").is_err() {
        build_frontend();
    }

    // -------------------------------------------------------------------------
    // ONNX Model Build
    // -------------------------------------------------------------------------
    println!("cargo:rerun-if-env-changed=SKIP_MODEL_BUILD");

    if env::var("SKIP_MODEL_BUILD").is_ok() {
        create_stub_model();
        return;
    }

    #[cfg(feature = "model-build")]
    {
        use burn_import::onnx::ModelGen;

        let model_dir = Path::new("src/uar/runtime/matching/models");
        let model_filename = "bg-small-en-v1.5.onnx";
        let model_path = model_dir.join(model_filename);
        let url = "https://huggingface.co/Xenova/bge-small-en-v1.5/resolve/main/onnx/model_quantized.onnx";

        if !model_path.exists() {
            fs::create_dir_all(model_dir).expect("Failed to create model directory");

            let status = std::process::Command::new("curl")
                .arg("-L")
                .arg("-o")
                .arg(&model_path)
                .arg(url)
                .status()
                .expect("Failed to execute curl");

            assert!(status.success(), "Failed to download model");
        }

        let tokenizer_files = vec![
            "tokenizer.json",
            "tokenizer_config.json",
            "special_tokens_map.json",
        ];
        for file in tokenizer_files {
            let file_path = model_dir.join(file);
            let file_url =
                format!("https://huggingface.co/Xenova/bge-small-en-v1.5/resolve/main/{file}");

            if !file_path.exists() {
                let status = std::process::Command::new("curl")
                    .arg("-L")
                    .arg("-o")
                    .arg(&file_path)
                    .arg(&file_url)
                    .status()
                    .expect("Failed to execute curl");

                if !status.success() {
                    // Non-fatal for config files.
                }
            }
            println!("cargo:rerun-if-changed={}", file_path.display());
        }

        println!("cargo:rerun-if-changed={}", model_path.display());

        ModelGen::new()
            .input(model_path.to_str().expect("Valid path"))
            .out_dir(env::var("OUT_DIR").expect("OUT_DIR not set").as_str())
            .run_from_script();
    }

    #[cfg(not(feature = "model-build"))]
    {
        create_stub_model();
    }
}

// =============================================================================
// Provider Catalog Builder
// =============================================================================

/// Builds the unified provider catalog by merging two sources:
///
/// 1. **liter-llm `schemas/providers.json`** — 142 providers with routing data
///    (`name`, `base_url`, `auth env_var`, `endpoints`, `model_prefixes`, `param_mappings`)
///
/// 2. **models.dev `api.json`** — rich model metadata (capabilities, pricing,
///    context limits, modalities) fetched at build time.
///
/// The merged result is written to `$OUT_DIR/provider_catalog.json` and embedded
/// into the binary via `include_str!`.
fn build_provider_catalog() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let catalog_path = Path::new(&out_dir).join("provider_catalog.json");

    // Skip rebuild if the catalog already exists and neither source changed.
    // The liter-llm JSON is tracked via rerun-if-changed; models.dev is fetched
    // fresh each build (HTTP is cheap, the file is ~1MB).
    let liter_llm_providers_path = find_liter_llm_providers_json();
    if let Some(ref p) = liter_llm_providers_path {
        println!("cargo:rerun-if-changed={}", p.display());
    }

    // --- Load liter-llm provider registry ---
    let liter_providers: serde_json::Value = if let Some(ref path) = liter_llm_providers_path {
        let contents = fs::read_to_string(path).unwrap_or_else(|e| {
            panic!(
                "Failed to read liter-llm providers.json at {}: {e}",
                path.display()
            )
        });
        serde_json::from_str(&contents)
            .unwrap_or_else(|e| panic!("Failed to parse liter-llm providers.json: {e}"))
    } else {
        println!(
            "cargo:warning=liter-llm providers.json not found — catalog will have models.dev data only"
        );
        serde_json::json!({ "providers": [], "complex_providers": [] })
    };

    // --- Fetch models.dev catalog ---
    let models_dev_data = fetch_models_dev_catalog();

    // --- Merge ---
    let catalog = merge_catalogs(&liter_providers, &models_dev_data);

    let json_output =
        serde_json::to_string(&catalog).expect("Failed to serialize provider catalog");
    fs::write(&catalog_path, &json_output)
        .unwrap_or_else(|e| panic!("Failed to write catalog to {}: {e}", catalog_path.display()));

    println!(
        "cargo:warning=Provider catalog built: {} providers",
        catalog.as_array().map_or(0, Vec::len)
    );
}

/// Locate `schemas/providers.json` relative to the liter-llm dependency.
///
/// We try several strategies:
/// 1. Sibling directory `../liter-llm/schemas/providers.json` (workspace layout)
/// 2. Git checkout cache in cargo home
fn find_liter_llm_providers_json() -> Option<std::path::PathBuf> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let manifest_path = Path::new(&manifest_dir);

    // Strategy 1: sibling directory (local development)
    let sibling = manifest_path
        .parent()
        .map(|p| p.join("liter-llm/schemas/providers.json"));
    if sibling.as_ref().is_some_and(|p| p.exists()) {
        return sibling;
    }

    // Strategy 2: check if the git dep was checked out by cargo
    if let Ok(cargo_home) = env::var("CARGO_HOME") {
        let git_db = Path::new(&cargo_home).join("git/checkouts");
        if git_db.exists() {
            for entry in fs::read_dir(&git_db).ok()?.flatten() {
                let name = entry.file_name();
                if name.to_string_lossy().contains("liter-llm") {
                    // Walk one level deeper (the hash directory)
                    if let Ok(subs) = fs::read_dir(entry.path()) {
                        for sub in subs.flatten() {
                            let candidate = sub.path().join("schemas/providers.json");
                            if candidate.exists() {
                                return Some(candidate);
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

/// Fetch the models.dev API catalog.
///
/// Downloads `https://models.dev/api.json` using `ureq` (available as a
/// build-dependency). Falls back to an empty object on failure.
fn fetch_models_dev_catalog() -> serde_json::Value {
    let url = "https://models.dev/api.json";

    match ureq::get(url).call() {
        Ok(resp) => {
            let mut body = resp.into_body();
            match body.read_to_string() {
                Ok(text) => serde_json::from_str(&text).unwrap_or_else(|e| {
                    println!("cargo:warning=Failed to parse models.dev JSON: {e}");
                    serde_json::json!({})
                }),
                Err(e) => {
                    println!("cargo:warning=Failed to read models.dev response: {e}");
                    serde_json::json!({})
                }
            }
        }
        Err(e) => {
            println!(
                "cargo:warning=Failed to fetch models.dev/api.json: {e} — using empty catalog"
            );
            serde_json::json!({})
        }
    }
}

/// Merge liter-llm provider registry with models.dev model metadata.
///
/// Output: a JSON array of provider objects, each with:
/// - Provider-level routing data from liter-llm
/// - Model-level capabilities, pricing, limits from models.dev
fn merge_catalogs(
    liter_providers: &serde_json::Value,
    models_dev: &serde_json::Value,
) -> serde_json::Value {
    let mut result = Vec::new();
    let empty_arr = serde_json::json!([]);
    let providers = liter_providers
        .get("providers")
        .unwrap_or(&empty_arr)
        .as_array()
        .cloned()
        .unwrap_or_default();

    // Build a lookup from models.dev provider ID -> models
    let models_dev_obj = models_dev.as_object();

    for provider in &providers {
        let name = provider.get("name").and_then(|v| v.as_str()).unwrap_or("");

        let mut entry = serde_json::json!({
            "id": name,
            "display_name": provider.get("display_name").and_then(|v| v.as_str()).unwrap_or(name),
            "base_url": provider.get("base_url"),
            "auth": provider.get("auth"),
            "endpoints": provider.get("endpoints").unwrap_or(&serde_json::json!([])),
            "model_prefixes": provider.get("model_prefixes").unwrap_or(&serde_json::json!([])),
            "param_mappings": provider.get("param_mappings"),
        });

        // Try to find matching models.dev entry
        let mut models = Vec::new();
        if let Some(dev_obj) = models_dev_obj {
            // models.dev uses provider IDs that may differ slightly from liter-llm names
            // Try exact match first, then common aliases
            let dev_entry = dev_obj
                .get(name)
                .or_else(|| dev_obj.get(&name.replace('_', "-")))
                .or_else(|| dev_obj.get(&name.replace('-', "_")));

            if let Some(provider_data) = dev_entry {
                if let Some(provider_models) =
                    provider_data.get("models").and_then(|m| m.as_object())
                {
                    for (_model_id, model_data) in provider_models {
                        let model_entry = serde_json::json!({
                            "id": model_data.get("id").and_then(serde_json::Value::as_str).unwrap_or(""),
                            "name": model_data.get("name").and_then(serde_json::Value::as_str).unwrap_or(""),
                            "family": model_data.get("family"),
                            "capabilities": {
                                "tool_call": model_data.get("tool_call").and_then(serde_json::Value::as_bool).unwrap_or(false),
                                "reasoning": model_data.get("reasoning").and_then(serde_json::Value::as_bool).unwrap_or(false),
                                "structured_output": model_data.get("structured_output").and_then(serde_json::Value::as_bool).unwrap_or(false),
                                "attachment": model_data.get("attachment").and_then(serde_json::Value::as_bool).unwrap_or(false),
                                "temperature": model_data.get("temperature").and_then(serde_json::Value::as_bool).unwrap_or(false),
                                "streaming": true,
                            },
                            "modalities": model_data.get("modalities").unwrap_or(&serde_json::json!({"input": ["text"], "output": ["text"]})),
                            "limits": {
                                "context_window": model_data.get("limit").and_then(|l| l.get("context")).and_then(serde_json::Value::as_u64).unwrap_or(0),
                                "max_output": model_data.get("limit").and_then(|l| l.get("output")).and_then(serde_json::Value::as_u64).unwrap_or(0),
                            },
                            "cost": model_data.get("cost"),
                            "release_date": model_data.get("release_date"),
                            "open_weights": model_data.get("open_weights").and_then(serde_json::Value::as_bool).unwrap_or(false),
                        });
                        models.push(model_entry);
                    }
                }

                // Capture the auth env var from models.dev if liter-llm didn't have one
                if (entry.get("auth").is_none() || entry["auth"].is_null())
                    && let Some(env_vars) = provider_data
                        .get("env")
                        .and_then(serde_json::Value::as_array)
                    && let Some(first) = env_vars.first().and_then(serde_json::Value::as_str)
                {
                    entry["auth"] = serde_json::json!({
                        "type": "bearer",
                        "env_var": first,
                    });
                }
            }
        }

        entry["models"] = serde_json::Value::Array(models);
        result.push(entry);
    }

    // Also include models.dev-only providers not in liter-llm
    if let Some(dev_obj) = models_dev_obj {
        let liter_names: std::collections::HashSet<String> = providers
            .iter()
            .filter_map(|p| p.get("name").and_then(|v| v.as_str()).map(String::from))
            .collect();

        for (dev_id, provider_data) in dev_obj {
            let normalized = dev_id.replace('-', "_");
            if liter_names.contains(dev_id)
                || liter_names.contains(&normalized)
                || liter_names.contains(&dev_id.replace('_', "-"))
            {
                continue;
            }

            let mut models = Vec::new();
            if let Some(provider_models) = provider_data.get("models").and_then(|m| m.as_object()) {
                for (_model_id, model_data) in provider_models {
                    let model_entry = serde_json::json!({
                        "id": model_data.get("id").and_then(serde_json::Value::as_str).unwrap_or(""),
                        "name": model_data.get("name").and_then(serde_json::Value::as_str).unwrap_or(""),
                        "family": model_data.get("family"),
                        "capabilities": {
                            "tool_call": model_data.get("tool_call").and_then(serde_json::Value::as_bool).unwrap_or(false),
                            "reasoning": model_data.get("reasoning").and_then(serde_json::Value::as_bool).unwrap_or(false),
                            "structured_output": model_data.get("structured_output").and_then(serde_json::Value::as_bool).unwrap_or(false),
                            "attachment": model_data.get("attachment").and_then(serde_json::Value::as_bool).unwrap_or(false),
                            "temperature": model_data.get("temperature").and_then(serde_json::Value::as_bool).unwrap_or(false),
                            "streaming": true,
                        },
                        "modalities": model_data.get("modalities").unwrap_or(&serde_json::json!({"input": ["text"], "output": ["text"]})),
                        "limits": {
                            "context_window": model_data.get("limit").and_then(|l| l.get("context")).and_then(serde_json::Value::as_u64).unwrap_or(0),
                            "max_output": model_data.get("limit").and_then(|l| l.get("output")).and_then(serde_json::Value::as_u64).unwrap_or(0),
                        },
                        "cost": model_data.get("cost"),
                        "release_date": model_data.get("release_date"),
                        "open_weights": model_data.get("open_weights").and_then(serde_json::Value::as_bool).unwrap_or(false),
                    });
                    models.push(model_entry);
                }
            }

            let auth = provider_data
                .get("env")
                .and_then(serde_json::Value::as_array)
                .and_then(|arr| {
                    arr.first()
                        .and_then(serde_json::Value::as_str)
                        .map(|env_var| serde_json::json!({ "type": "bearer", "env_var": env_var }))
                });

            let entry = serde_json::json!({
                "id": dev_id,
                "display_name": provider_data.get("name").and_then(serde_json::Value::as_str).unwrap_or(dev_id),
                "base_url": provider_data.get("api"),
                "auth": auth,
                "endpoints": ["chat"],
                "model_prefixes": [],
                "param_mappings": null,
                "models": models,
                "source": "models_dev_only",
            });
            result.push(entry);
        }
    }

    serde_json::Value::Array(result)
}

fn create_stub_catalog() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let catalog_path = Path::new(&out_dir).join("provider_catalog.json");
    fs::write(&catalog_path, "[]").expect("Failed to write stub catalog");
    println!("cargo:warning=Catalog build skipped — using empty stub");
}

// =============================================================================
// Frontend Builder
// =============================================================================

/// Build the Vite + React frontend application.
///
/// Runs `bun install --frozen-lockfile` followed by `bun run build` inside the
/// `frontend/` directory.  The Vite config writes output to `../static` (the
/// repo-root `static/` directory) which Axum serves at runtime.
fn build_frontend() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let frontend_dir = Path::new(&manifest_dir).join("frontend");

    if !frontend_dir.exists() {
        println!("cargo:warning=frontend/ directory not found — skipping frontend build");
        return;
    }

    for file in &[
        "frontend/package.json",
        "frontend/vite.config.ts",
        "frontend/vite.config.js",
        "frontend/tailwind.config.ts",
        "frontend/postcss.config.js",
        "frontend/tsconfig.json",
        "frontend/index.html",
    ] {
        println!("cargo:rerun-if-changed={file}");
    }
    println!("cargo:rerun-if-changed=frontend/src");
    println!("cargo:rerun-if-changed=frontend/public");

    // Frontend toolchain preference: pnpm (canonical, supports workspaces) →
    // bun (legacy) → npm. The frontend was migrated to pnpm-workspaces to
    // embed `@prometheus-ags/prometheus-entity-management` as a submodule
    // under `frontend/packages/`.
    let (pm, install_args, build_args): (&str, &[&str], &[&str]) = if which("pnpm") {
        ("pnpm", &["install", "--frozen-lockfile"], &["run", "build"])
    } else if which("bun") {
        ("bun", &["install", "--frozen-lockfile"], &["run", "build"])
    } else {
        ("npm", &["ci", "--prefer-offline"], &["run", "build"])
    };

    println!("cargo:warning=Installing frontend dependencies with {pm}…");

    let install_status = Command::new(pm)
        .args(install_args)
        .current_dir(&frontend_dir)
        .status()
        .unwrap_or_else(|e| panic!("Failed to run `{pm} install`: {e}"));

    assert!(
        install_status.success(),
        "Frontend dependency installation failed (exit code: {install_status})"
    );

    println!("cargo:warning=Building frontend assets with {pm} run build…");

    let build_status = Command::new(pm)
        .args(build_args)
        .current_dir(&frontend_dir)
        .status()
        .unwrap_or_else(|e| panic!("Failed to run `{pm} run build`: {e}"));

    assert!(
        build_status.success(),
        "Frontend build failed (exit code: {build_status})"
    );

    println!("cargo:warning=Frontend build complete — assets written to static/");
}

/// Returns `true` if `name` resolves to an executable on `$PATH`.
fn which(name: &str) -> bool {
    Command::new(if cfg!(windows) { "where" } else { "which" })
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn create_stub_model() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let model_dir = Path::new(&out_dir).join("model");
    fs::create_dir_all(&model_dir).expect("Failed to create model directory");

    let stub_code = r"
use burn::module::Module;
use burn::tensor::backend::Backend;

#[derive(Module, Debug)]
pub struct Model<B: Backend> {
    _phantom: std::marker::PhantomData<B>,
}

impl<B: Backend> Model<B> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<B: Backend> Default for Model<B> {
    fn default() -> Self {
        Self::new()
    }
}
";

    let stub_path = model_dir.join("bg_small_en_v1_5.rs");
    fs::write(stub_path, stub_code).expect("Failed to write stub model");
}
