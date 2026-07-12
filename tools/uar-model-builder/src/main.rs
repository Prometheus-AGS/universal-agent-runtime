//! Maintainer-only ONNX-to-Burn model generator.

use std::path::PathBuf;

use burn_import::onnx::ModelGen;

fn main() {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let input = workspace.join("src/uar/runtime/matching/models/bg-small-en-v1.5.onnx");
    let output = workspace.join("src/uar/runtime/matching/generated");

    assert!(
        input.is_file(),
        "model input is missing: {}; download the reviewed ONNX artifact first",
        input.display()
    );

    ModelGen::new()
        .input(input.to_str().expect("model path must be UTF-8"))
        .out_dir(output.to_str().expect("output path must be UTF-8"))
        .run_from_script();
}
