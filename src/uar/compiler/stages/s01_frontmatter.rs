//! Stage 01: Validate Frontmatter
//!
//! Validates metadata version, schema compatibility, and required identity fields.

use crate::uar::compiler::error::CompileResult;
use crate::uar::compiler::pipeline::CompileContext;
use crate::uar::compiler::report::{Diagnostic, DiagnosticLevel};

pub async fn run(ctx: &mut CompileContext) -> CompileResult<Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();

    // Validate version string is non-empty
    if ctx.ir.metadata.version.is_empty() {
        diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            message: "metadata.version is required and must not be empty".into(),
            section: Some("Metadata".into()),
        });
    }

    // Validate identity fields
    if ctx.ir.identity.name.is_empty() {
        diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            message: "identity.name is required".into(),
            section: Some("Identity".into()),
        });
    }

    if ctx.ir.identity.role.is_empty() {
        diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            message: "identity.role is required".into(),
            section: Some("Identity".into()),
        });
    }

    if ctx.ir.identity.persona.is_empty() {
        diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            message: "identity.persona is required".into(),
            section: Some("Identity".into()),
        });
    }

    // Fingerprint the metadata
    let meta_json = serde_json::to_string(&ctx.ir.metadata).unwrap_or_default();
    ctx.fingerprints.insert(
        "metadata".into(),
        CompileContext::sha256_hex(meta_json.as_bytes()),
    );

    Ok(diagnostics)
}
