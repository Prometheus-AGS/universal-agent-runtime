use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=proto");

    // The macOS debug test image exceeds Apple's 16 MiB __eh_frame compact-
    // unwind offset range. Keep compact unwind enabled, but suppress only that
    // documented informational diagnostic for test targets. Release links and
    // every other linker warning remain unchanged.
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos")
        && env::var("PROFILE").as_deref() == Ok("debug")
    {
        println!("cargo:rustc-link-arg=-Wl,-no_warn_eh_frame_too_large");
    }

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
