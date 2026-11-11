fn main() {
    link_ios_compiler_runtime();
    println!("cargo::rustc-check-cfg=cfg(coverage)");

    println!("cargo::rustc-check-cfg=cfg(inference_ort)");
    if std::env::var_os("CARGO_FEATURE_ORT_BUNDLED").is_some()
        || std::env::var_os("CARGO_FEATURE_ORT_DYNAMIC").is_some()
    {
        println!("cargo::rustc-cfg=inference_ort");
    }

    println!("cargo::rustc-check-cfg=cfg(auto_rotate)");
    if std::env::var_os("CARGO_FEATURE_AUTO_ROTATE").is_some()
        || std::env::var_os("CARGO_FEATURE_AUTO_ROTATE_TRACT").is_some()
    {
        println!("cargo::rustc-cfg=auto_rotate");
    }

    println!("cargo::rustc-check-cfg=cfg(sceptre_ocr)");
    if std::env::var_os("CARGO_FEATURE_SCEPTRE_OCR_ORT").is_some()
        || std::env::var_os("CARGO_FEATURE_SCEPTRE_OCR_TRACT").is_some()
        || std::env::var_os("CARGO_FEATURE_SCEPTRE_OCR_CANDLE").is_some()
    {
        println!("cargo::rustc-cfg=sceptre_ocr");
    }

    if std::env::var_os("CARGO_FEATURE_ORT_BUNDLED").is_some()
        && std::env::var_os("CARGO_FEATURE_ORT_DYNAMIC").is_some()
    {
        println!(
            "cargo::warning=features 'ort-bundled' and 'ort-dynamic' are both enabled; bundled ORT remains the default unless dynamic ORT is explicitly selected at runtime"
        );
    }

    println!("cargo::rustc-check-cfg=cfg(layout_detection)");
    if std::env::var_os("CARGO_FEATURE_LAYOUT_DETECTION").is_some()
        || std::env::var_os("CARGO_FEATURE_LAYOUT_TRACT").is_some()
    {
        println!("cargo::rustc-cfg=layout_detection");
    }

    println!("cargo::rustc-check-cfg=cfg(paddle_ocr)");
    if std::env::var_os("CARGO_FEATURE_PADDLE_OCR_ORT").is_some()
        || std::env::var_os("CARGO_FEATURE_PADDLE_OCR_TRACT").is_some()
    {
        println!("cargo::rustc-cfg=paddle_ocr");
    }

    // See `build_id`'s doc comment for what this feeds: `cache::version::cache_version_tag`
    // (crates/xberg/src/cache/version.rs) folds it in alongside the crate version, the cache
    // schema version, and the debug/release profile bit.
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        emit_rerun_directives(&manifest_dir);
    }
    println!("cargo::rustc-env=XBERG_BUILD_ID={}", build_id());
}

fn link_ios_compiler_runtime() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("ios")
        || std::env::var_os("CARGO_FEATURE_ORT_BUNDLED").is_none()
    {
        return;
    }
    let (sdk, library) = if std::env::var("CARGO_CFG_TARGET_ABI").as_deref() == Ok("sim") {
        ("iphonesimulator", "clang_rt.iossim")
    } else {
        ("iphoneos", "clang_rt.ios")
    };
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH unset");
    let apple_arch = if arch == "aarch64" { "arm64" } else { &arch };
    let runtime = ios_compiler_runtime(sdk, library);
    let staged = stage_ios_compiler_runtime(sdk, &runtime, apple_arch);
    println!(
        "cargo::rustc-link-search=native={}",
        staged.parent().expect("runtime has no parent").display()
    );
    println!("cargo::rustc-link-lib=static={library}");
    println!("cargo::rerun-if-changed={}", runtime.display());
    for variable in ["DEVELOPER_DIR", "TOOLCHAINS", "SDKROOT"] {
        println!("cargo::rerun-if-env-changed={variable}");
    }
}

fn ios_compiler_runtime(sdk: &str, library: &str) -> std::path::PathBuf {
    let output = std::process::Command::new("xcrun")
        .args(["--sdk", sdk, "clang", "--print-resource-dir"])
        .output()
        .unwrap_or_else(|error| panic!("query {sdk} compiler runtime directory: {error}"));
    assert!(
        output.status.success(),
        "query {sdk} compiler runtime directory ({}): {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let resource = String::from_utf8(output.stdout)
        .unwrap_or_else(|error| panic!("decode {sdk} compiler runtime directory: {error}"));
    assert!(!resource.trim().is_empty(), "empty {sdk} compiler runtime directory");
    std::path::Path::new(resource.trim())
        .join("lib/darwin")
        .join(format!("lib{library}.a"))
}

fn stage_ios_compiler_runtime(sdk: &str, archive: &std::path::Path, arch: &str) -> std::path::PathBuf {
    use std::io::Read;

    let mut header = [0; 8];
    std::fs::File::open(archive)
        .and_then(|mut file| file.read_exact(&mut header))
        .unwrap_or_else(|error| panic!("read {sdk} compiler runtime archive: {error}"));
    let destination = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR unset"))
        .join(archive.file_name().expect("runtime archive has no filename"));
    // Rust bundles native archives into rlibs, but cannot read Xcode's universal archives. ~keep
    if &header == b"!<arch>\n" {
        std::fs::copy(archive, &destination)
            .unwrap_or_else(|error| panic!("copy {sdk} compiler runtime archive: {error}"));
    } else {
        let output = std::process::Command::new("xcrun")
            .arg("lipo")
            .arg(archive)
            .args(["-thin", arch, "-output"])
            .arg(&destination)
            .output()
            .unwrap_or_else(|error| panic!("extract {arch} from {sdk} compiler runtime: {error}"));
        assert!(
            output.status.success(),
            "extract {arch} from {sdk} compiler runtime ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    destination
}

/// Cache identity for the actual packaged source, including builds without Git.
fn build_id() -> String {
    use std::hash::{Hash, Hasher};
    use std::path::Path;

    const BACKPORT_SOURCE_ID: &str = "ffb3b1a25b6c14c13858142997dd82a16d5b2f35-calamine-0.38.0";
    if let Ok(id) = std::env::var("XBERG_BUILD_ID")
        && !id.is_empty()
    {
        return id;
    }

    let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") else {
        return BACKPORT_SOURCE_ID.to_owned();
    };
    let manifest = Path::new(&manifest_dir);
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for path in ["Cargo.toml", "build.rs", "src", "stopwords"] {
        if hash_input(manifest, Path::new(path), &mut hasher).is_err() {
            return BACKPORT_SOURCE_ID.to_owned();
        }
    }
    let workspace_lock = manifest.join("../../../Cargo.lock");
    if let Ok(lock) = std::fs::read(workspace_lock) {
        lock.hash(&mut hasher);
    }
    format!("{BACKPORT_SOURCE_ID}-{:016x}", hasher.finish())
}

fn emit_rerun_directives(manifest_dir: &str) {
    println!("cargo::rerun-if-env-changed=XBERG_BUILD_ID");
    for path in ["Cargo.toml", "build.rs", "src", "stopwords", "../../../Cargo.lock"] {
        println!("cargo::rerun-if-changed={manifest_dir}/{path}");
    }
}

fn hash_input(
    root: &std::path::Path,
    relative: &std::path::Path,
    hasher: &mut impl std::hash::Hasher,
) -> std::io::Result<()> {
    use std::hash::Hash;
    use std::path::PathBuf;

    let input = root.join(relative);
    relative.hash(hasher);
    if input.is_dir() {
        let mut entries = std::fs::read_dir(input)?
            .map(|entry| entry.map(|entry| relative.join(entry.file_name())))
            .collect::<std::io::Result<Vec<PathBuf>>>()?;
        entries.sort_unstable();
        for entry in entries {
            hash_input(root, &entry, hasher)?;
        }
    } else {
        std::fs::read(input)?.hash(hasher);
    }
    Ok(())
}
