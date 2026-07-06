//! CH-15: every `templates/*.agent.md` document must compile+sign cleanly.
//!
//! This is the regression guard for the template library independent of the
//! release workflow (`.github/workflows/release.yml`), which only compiles
//! them on a tag push — this test runs on every `cargo test`.

use std::sync::Arc;

use universal_agent_runtime::uar::compiler::parser::parse;
use universal_agent_runtime::uar::compiler::pipeline::compile;
use universal_agent_runtime::uar::compiler::registries::{
    InMemoryEndpointRegistry, InMemorySchemaRegistry,
};
use universal_agent_runtime::uar::compiler::report::CompileOutcome;
use universal_agent_runtime::uar::compiler::signing::LocalKeyProvider;

const TEMPLATES: &[&str] = &["coding", "vision", "terminal", "data"];

#[tokio::test]
async fn all_templates_compile_and_sign_to_v2_schema() {
    for name in TEMPLATES {
        let path = format!("{}/templates/{name}.agent.md", env!("CARGO_MANIFEST_DIR"));
        let markdown =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"));

        let ir = parse(&markdown).unwrap_or_else(|e| panic!("{name}: failed to parse: {e}"));

        let schema_registry = Arc::new(InMemorySchemaRegistry::new());
        let endpoint_registry = Arc::new(InMemoryEndpointRegistry::new());
        let key_provider = Arc::new(LocalKeyProvider::ephemeral());

        let output = compile(ir, schema_registry, endpoint_registry, key_provider)
            .await
            .unwrap_or_else(|e| panic!("{name}: compilation failed: {e}"));

        assert_eq!(
            output.report.overall,
            CompileOutcome::Pass,
            "{name}: expected a clean compile, got diagnostics: {:?}",
            output
                .report
                .stages
                .iter()
                .flat_map(|s| &s.diagnostics)
                .collect::<Vec<_>>()
        );

        // Every template declares at least one v2 section (that's the point
        // of this library), so every one should bump to the v2 schema, not
        // silently stay at v1.
        assert_eq!(
            output.descriptor.schema, "uar-agent-descriptor/v2",
            "{name}: expected v2 schema (declares v2 sections)"
        );
        assert!(
            !output.descriptor.signer_public_key.is_empty(),
            "{name}: expected a non-empty signer public key"
        );
    }
}
