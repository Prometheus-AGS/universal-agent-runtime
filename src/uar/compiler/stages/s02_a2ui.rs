//! Stage 02: Validate A2UI Schemas
//!
//! Validates UI schema structure and registers schemas in the [`SchemaRegistry`].
//! Uses the trait-based registry — in-memory by default, extensible to external
//! registries in the future.

use crate::uar::compiler::error::CompileResult;
use crate::uar::compiler::pipeline::CompileContext;
use crate::uar::compiler::report::{Diagnostic, DiagnosticLevel};

pub async fn run(ctx: &mut CompileContext) -> CompileResult<Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();

    // Validate form schemas — each form must have a valid ID
    for form in &ctx.ir.ui.forms {
        if form.id.is_empty() {
            diagnostics.push(Diagnostic {
                level: DiagnosticLevel::Error,
                message: "ui.forms[*].id must not be empty".into(),
                section: Some("UI (A2UI)".into()),
            });
            continue;
        }

        // Register the form schema if provided
        if let Some(schema) = &form.schema {
            if let Err(e) = ctx.schema_registry.register(&form.id, schema.clone()).await {
                diagnostics.push(Diagnostic {
                    level: DiagnosticLevel::Warning,
                    message: format!("failed to register form schema '{}': {e}", form.id),
                    section: Some("UI (A2UI)".into()),
                });
            }
        }
    }

    // Validate artifact declarations
    for artifact in &ctx.ir.ui.artifacts {
        if artifact.id.is_empty() {
            diagnostics.push(Diagnostic {
                level: DiagnosticLevel::Error,
                message: "ui.artifacts[*].id must not be empty".into(),
                section: Some("UI (A2UI)".into()),
            });
        }
        if artifact.artifact_type.is_empty() {
            diagnostics.push(Diagnostic {
                level: DiagnosticLevel::Error,
                message: format!("ui.artifacts['{}']. type must not be empty", artifact.id),
                section: Some("UI (A2UI)".into()),
            });
        }

        // If schema ref exists, try to resolve it (future: external resolution)
        if let Some(schema_ref) = &artifact.schema {
            if !schema_ref.is_empty() {
                let resolved = ctx.schema_registry.resolve(schema_ref).await?;
                if resolved.is_none() {
                    diagnostics.push(Diagnostic {
                        level: DiagnosticLevel::Info,
                        message: format!(
                            "artifact '{}' references schema '{}' which is not in the registry (external resolution not yet supported)",
                            artifact.id, schema_ref
                        ),
                        section: Some("UI (A2UI)".into()),
                    });
                }
            }
        }
    }

    // Validate action declarations
    for action in &ctx.ir.ui.actions {
        if action.id.is_empty() {
            diagnostics.push(Diagnostic {
                level: DiagnosticLevel::Error,
                message: "ui.actions[*].id must not be empty".into(),
                section: Some("UI (A2UI)".into()),
            });
        }

        // Register action schemas
        if let Some(input_schema) = &action.input_schema {
            let schema_id = format!("{}.input", action.id);
            let _ = ctx
                .schema_registry
                .register(&schema_id, input_schema.clone())
                .await;
        }
        if let Some(output_schema) = &action.output_schema {
            let schema_id = format!("{}.output", action.id);
            let _ = ctx
                .schema_registry
                .register(&schema_id, output_schema.clone())
                .await;
        }
    }

    // Fingerprint the UI section
    let ui_json = serde_json::to_string(&ctx.ir.ui).unwrap_or_default();
    ctx.fingerprints
        .insert("ui".into(), CompileContext::sha256_hex(ui_json.as_bytes()));

    Ok(diagnostics)
}
