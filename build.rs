use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=proto");

    // -------------------------------------------------------------------------
    // Compile both the legacy compatibility service and official A2A 1.0.
    // Protocol inputs and imported annotations are committed with provenance;
    // builds never fetch schema sources from the network.
    // -------------------------------------------------------------------------
    if env::var_os("CARGO_FEATURE_A2A_TRANSPORT").is_some() && Path::new("proto/a2a.proto").exists()
    {
        let protoc = protoc_bin_vendored::protoc_bin_path().expect("Failed to locate vendored protoc");
        // SAFETY: this build script has not started any threads, so no other
        // thread can read or mutate the process environment concurrently.
        unsafe { env::set_var("PROTOC", protoc) };
        let protobuf_include = protoc_bin_vendored::include_path()
            .expect("Failed to locate pinned protobuf well-known types");
        tonic_prost_build::configure()
            .compile_protos(
                &[Path::new("proto/a2a.proto"), Path::new("proto/a2a_v1.proto")],
                &[Path::new("proto"), protobuf_include.as_path()],
            )
            .expect("Failed to compile A2A protocol definitions");
    }

    // -------------------------------------------------------------------------
    // Provider / Model Catalog
    // -------------------------------------------------------------------------
    // Release builds consume the reviewed, digested snapshot. Network access
    // and catalog refresh are explicit maintainer operations, never build steps.
    copy_provider_catalog_snapshot();

    create_stub_model();

    emit_build_metadata();
}

/// Embed the build facts that `GET /api/uar/version` reports.
///
/// The commit comes from the `UAR_GIT_SHA` build variable when the packager
/// sets it (a container build has no `.git`), otherwise from the checkout in
/// release-profile builds only. Debug builds record none: a commit id in the
/// compiler environment would recompile the whole crate after every commit.
fn emit_build_metadata() {
    println!("cargo:rerun-if-env-changed=UAR_GIT_SHA");
    let profile = env::var("PROFILE").unwrap_or_default();
    let target = env::var("TARGET").unwrap_or_default();
    let mut features: Vec<String> = env::vars_os()
        .filter_map(|(name, _)| {
            name.to_str()?
                .strip_prefix("CARGO_FEATURE_")
                .map(|feature| feature.to_ascii_lowercase().replace('_', "-"))
        })
        .collect();
    features.sort_unstable();
    let supplied = env::var("UAR_GIT_SHA")
        .ok()
        .map(|commit| commit.trim().to_ascii_lowercase())
        .filter(|commit| !commit.is_empty());
    // A packager that names a commit means it. Dropping a malformed value
    // would ship a binary that reports no commit without anyone noticing.
    assert!(
        supplied.as_deref().is_none_or(is_commit_id),
        "UAR_GIT_SHA must be a full 40- or 64-character hexadecimal commit id"
    );
    let commit = match supplied {
        Some(commit) => Some(commit),
        None if profile == "release" => checkout_commit(),
        None => None,
    }
    .map(|commit| commit.to_ascii_lowercase())
    .filter(|commit| is_commit_id(commit))
    .unwrap_or_default();

    println!("cargo:rustc-env=UAR_BUILD_PROFILE={profile}");
    println!("cargo:rustc-env=UAR_BUILD_TARGET={target}");
    println!("cargo:rustc-env=UAR_BUILD_FEATURES={}", features.join(","));
    println!("cargo:rustc-env=UAR_BUILD_GIT_COMMIT={commit}");
}

/// A full SHA-1 or SHA-256 git object id in hexadecimal.
fn is_commit_id(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// Commit of the checkout being built, if this is a git checkout with `git`
/// on the path.
///
/// Asks Cargo to rerun this script when the checked-out commit moves. A
/// linked worktree keeps `HEAD` in its own git directory and branch refs in
/// the shared one, so every path is resolved through git instead of assumed.
/// The branch-ref directory is watched as a whole, not the one ref file: a
/// ref that lives only in `packed-refs` gets its loose file on the next
/// commit, and that new file must count as a change. `reftable` covers
/// repositories that store refs in that format.
fn checkout_commit() -> Option<String> {
    for name in ["HEAD", "packed-refs", "refs/heads", "reftable"] {
        if let Some(path) = git(&["rev-parse", "--git-path", name])
            && Path::new(&path).exists()
        {
            println!("cargo:rerun-if-changed={path}");
        }
    }
    git(&["rev-parse", "HEAD"])
}

/// Trimmed stdout of a successful `git` command, or `None`.
fn git(arguments: &[&str]) -> Option<String> {
    let output = Command::new("git").args(arguments).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn copy_provider_catalog_snapshot() {
    const SNAPSHOT: &str = "catalog/provider_catalog.json";
    println!("cargo:rerun-if-changed={SNAPSHOT}");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let destination = Path::new(&out_dir).join("provider_catalog.json");
    fs::copy(SNAPSHOT, &destination).unwrap_or_else(|error| {
        panic!(
            "failed to copy committed provider catalog {SNAPSHOT} to {}: {error}",
            destination.display()
        )
    });
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
