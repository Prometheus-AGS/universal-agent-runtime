use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=SKIP_MODEL_BUILD");
    println!("cargo:rerun-if-env-changed=SKIP_FRONTEND_BUILD");

    // -------------------------------------------------------------------------
    // Frontend Build
    // -------------------------------------------------------------------------
    // Build the Vite + React frontend unless explicitly skipped.
    // Set SKIP_FRONTEND_BUILD=1 to skip (e.g. inside Docker when static/ is
    // already pre-built or copied in a prior layer).
    if env::var("SKIP_FRONTEND_BUILD").is_err() {
        build_frontend();
    }

    // -------------------------------------------------------------------------
    // ONNX Model Build
    // -------------------------------------------------------------------------
    println!("cargo:rerun-if-env-changed=SKIP_MODEL_BUILD");

    // Check if we should skip model building (for testing or CI)
    if env::var("SKIP_MODEL_BUILD").is_ok() {
        create_stub_model();
        return;
    }

    // Only build the model if burn-import is available
    #[cfg(feature = "model-build")]
    {
        use burn_import::onnx::ModelGen;

        // Model configuration
        let model_dir = Path::new("src/uar/runtime/matching/models");
        let model_filename = "bg-small-en-v1.5.onnx";
        let model_path = model_dir.join(model_filename);
        let url = "https://huggingface.co/Xenova/bge-small-en-v1.5/resolve/main/onnx/model_quantized.onnx";

        // Download model if it doesn't exist
        if !model_path.exists() {
            fs::create_dir_all(model_dir).expect("Failed to create model directory");

            // Use a simple command to download to avoid ureq version complexity in build script for now
            // since we know curl is available on the system (mac)
            let status = std::process::Command::new("curl")
                .arg("-L")
                .arg("-o")
                .arg(&model_path)
                .arg(url)
                .status()
                .expect("Failed to execute curl");

            assert!(status.success(), "Failed to download model");
        }

        // Download tokenizer files
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

        // Generate Burn code from ONNX
        // The generated code will be placed in OUT_DIR/model.rs
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

/// Build the Vite + React frontend application.
///
/// Runs `bun install --frozen-lockfile` followed by `bun run build` inside the
/// `frontend/` directory.  The Vite config writes output to `../static` (the
/// repo-root `static/` directory) which Axum serves at runtime.
///
/// Cargo change-tracking:
///   - Re-runs when any file inside `frontend/src` or `frontend/public` changes.
///   - Re-runs when the frontend's `package.json`, `vite.config.ts`, or
///     `tailwind.config.ts` changes.
///   - Does *not* re-run when only `frontend/node_modules` changes.
fn build_frontend() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let frontend_dir = Path::new(&manifest_dir).join("frontend");

    if !frontend_dir.exists() {
        println!("cargo:warning=frontend/ directory not found — skipping frontend build");
        return;
    }

    // Tell Cargo to re-run this script when frontend sources change.
    // We enumerate the key config files individually, and use a broad rerun
    // for the src/ and public/ trees.
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

    // Resolve the package manager: prefer bun, fall back to npm.
    let pm = if which("bun") { "bun" } else { "npm" };

    println!("cargo:warning=Installing frontend dependencies with {pm}…");

    // Install dependencies
    let install_args: &[&str] = if pm == "bun" {
        &["install", "--frozen-lockfile"]
    } else {
        &["ci", "--prefer-offline"]
    };

    let install_status = Command::new(pm)
        .args(install_args)
        .current_dir(&frontend_dir)
        .status()
        .unwrap_or_else(|e| panic!("Failed to run `{pm} install`: {e}"));

    if !install_status.success() {
        panic!("Frontend dependency installation failed (exit code: {install_status})");
    }

    println!("cargo:warning=Building frontend assets with {pm} run build…");

    // Run the build
    let build_status = Command::new(pm)
        .args(["run", "build"])
        .current_dir(&frontend_dir)
        .status()
        .unwrap_or_else(|e| panic!("Failed to run `{pm} run build`: {e}"));

    if !build_status.success() {
        panic!("Frontend build failed (exit code: {build_status})");
    }

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
