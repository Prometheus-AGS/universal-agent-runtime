//! Compiler CLI surface (CH-15): the `compile` subcommand.
//!
//! Compiles a single UAR-AGENT-MD document into a signed
//! [`super::pipeline::CompiledDescriptor`], reusing the same 8-stage
//! pipeline ([`super::pipeline::compile`]) the conversational/service compile
//! paths use — not a separate implementation. Local release tooling uses it
//! to compile+sign the agent template library (`templates/*.agent.md`), and
//! it is usable standalone for any `.agent.md` document.

use std::sync::Arc;

use super::parser;
use super::pipeline::compile;
use super::registries::{InMemoryEndpointRegistry, InMemorySchemaRegistry};
use super::report::CompileOutcome;
use super::signing::LocalKeyProvider;

/// Compile the `.agent.md` document at `path`. Writes the signed descriptor
/// JSON to `out` if given, else prints it to stdout.
///
/// Returns a process exit code: `0` on success, `1` on read/parse/compile
/// failure or a failing stage diagnostic.
pub async fn run_compile(path: &str, out: Option<&str>) -> i32 {
    let markdown = match std::fs::read_to_string(path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error: failed to read {path}: {e}");
            return 1;
        }
    };

    let ir = match parser::parse(&markdown) {
        Ok(ir) => ir,
        Err(e) => {
            eprintln!("error: failed to parse {path}: {e}");
            return 1;
        }
    };

    // Ephemeral in-memory registries/key — matches the convention already
    // used by the conversational and service compile paths
    // (`compiler_skill.rs`, `service.rs`, `conversational.rs`): each compile
    // call is a self-contained structural attestation, not tied to a
    // persistent signing identity across runs.
    let schema_registry = Arc::new(InMemorySchemaRegistry::new());
    let endpoint_registry = Arc::new(InMemoryEndpointRegistry::new());
    let key_provider = Arc::new(LocalKeyProvider::ephemeral());

    let output = match compile(ir, schema_registry, endpoint_registry, key_provider).await {
        Ok(output) => output,
        Err(e) => {
            eprintln!("error: compilation failed for {path}: {e}");
            return 1;
        }
    };

    if output.report.overall != CompileOutcome::Pass {
        eprintln!("error: {path} compiled with failing diagnostics:");
        for stage in &output.report.stages {
            for diag in &stage.diagnostics {
                eprintln!(
                    "  [{}] {}: {}",
                    stage.name,
                    diag.section.as_deref().unwrap_or("-"),
                    diag.message
                );
            }
        }
        return 1;
    }

    let json = match serde_json::to_string_pretty(&output.descriptor) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("error: failed to serialize compiled descriptor: {e}");
            return 1;
        }
    };

    match out {
        Some(out_path) => {
            if let Err(e) = std::fs::write(out_path, &json) {
                eprintln!("error: failed to write {out_path}: {e}");
                return 1;
            }
            println!(
                "compiled {path} -> {out_path} (schema {})",
                output.descriptor.schema
            );
        }
        None => println!("{json}"),
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn compiles_a_minimal_document_to_a_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let src_path = dir.path().join("test.agent.md");
        std::fs::write(&src_path, parser::minimal_agent_md()).expect("write fixture");
        let out_path = dir.path().join("test.compiled.json");

        let code = run_compile(
            src_path.to_str().expect("utf8 path"),
            Some(out_path.to_str().expect("utf8 path")),
        )
        .await;

        assert_eq!(code, 0, "expected success exit code");
        let written = std::fs::read_to_string(&out_path).expect("read output");
        let parsed: serde_json::Value =
            serde_json::from_str(&written).expect("output should be valid JSON");
        assert_eq!(parsed["agent_id"], "test-agent");
        assert_eq!(parsed["schema"], "uar-agent-descriptor/v1");
    }

    #[tokio::test]
    async fn missing_file_returns_error_exit_code() {
        let code = run_compile("/nonexistent/path/does-not-exist.agent.md", None).await;
        assert_eq!(code, 1);
    }

    #[tokio::test]
    async fn unparsable_document_returns_error_exit_code() {
        let dir = tempfile::tempdir().expect("tempdir");
        let src_path = dir.path().join("bad.agent.md");
        std::fs::write(&src_path, "not a valid agent document").expect("write fixture");

        let code = run_compile(src_path.to_str().expect("utf8 path"), None).await;
        assert_eq!(code, 1);
    }
}
