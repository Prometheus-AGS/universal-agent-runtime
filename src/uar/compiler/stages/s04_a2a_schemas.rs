//! Stage 04: Validate A2A JSON Schemas
//!
//! Parses A2A endpoint input/output JSON Schemas and records content fingerprints.

use crate::uar::compiler::error::CompileResult;
use crate::uar::compiler::pipeline::CompileContext;
use crate::uar::compiler::report::{Diagnostic, DiagnosticLevel};

pub async fn run(ctx: &mut CompileContext) -> CompileResult<Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();

    for endpoint in &ctx.ir.a2a.endpoints {
        if endpoint.id.is_empty() {
            diagnostics.push(Diagnostic {
                level: DiagnosticLevel::Error,
                message: "a2a.endpoints[*].id must not be empty".into(),
                section: Some("A2A Contracts".into()),
            });
            continue;
        }

        // Validate and fingerprint input schema
        if let Some(input_schema) = &endpoint.input_schema {
            let schema_json = serde_json::to_string(input_schema).unwrap_or_default();
            ctx.fingerprints.insert(
                format!("a2a.{}.input", endpoint.id),
                CompileContext::sha256_hex(schema_json.as_bytes()),
            );

            // Register in schema registry for cross-stage reference
            let schema_id = format!("a2a.{}.input", endpoint.id);
            let _ = ctx
                .schema_registry
                .register(&schema_id, input_schema.clone())
                .await;
        }

        // Validate and fingerprint output schema
        if let Some(output_schema) = &endpoint.output_schema {
            let schema_json = serde_json::to_string(output_schema).unwrap_or_default();
            ctx.fingerprints.insert(
                format!("a2a.{}.output", endpoint.id),
                CompileContext::sha256_hex(schema_json.as_bytes()),
            );

            let schema_id = format!("a2a.{}.output", endpoint.id);
            let _ = ctx
                .schema_registry
                .register(&schema_id, output_schema.clone())
                .await;
        }
    }

    // Validate dependencies reference valid agents/endpoints
    for dep in &ctx.ir.a2a.dependencies {
        if dep.agent_id.is_empty() {
            diagnostics.push(Diagnostic {
                level: DiagnosticLevel::Error,
                message: "a2a.dependencies[*].agent_id must not be empty".into(),
                section: Some("A2A Contracts".into()),
            });
        }
        if dep.endpoint.is_empty() {
            diagnostics.push(Diagnostic {
                level: DiagnosticLevel::Error,
                message: format!(
                    "a2a.dependencies[agent:{}].endpoint must not be empty",
                    dep.agent_id
                ),
                section: Some("A2A Contracts".into()),
            });
        }
    }

    // Fingerprint the A2A section
    let a2a_json = serde_json::to_string(&ctx.ir.a2a).unwrap_or_default();
    ctx.fingerprints.insert(
        "a2a".into(),
        CompileContext::sha256_hex(a2a_json.as_bytes()),
    );

    Ok(diagnostics)
}
