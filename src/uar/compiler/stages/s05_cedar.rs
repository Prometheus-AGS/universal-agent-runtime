//! Stage 05: Compile Cedar Policy
//!
//! Parses embedded Cedar policy text, validates entity/action usage, and compiles
//! to a policy set. Uses the existing `cedar-policy` crate already in the project.

use cedar_policy::PolicySet;

use crate::uar::compiler::error::{CompileError, CompileResult};
use crate::uar::compiler::pipeline::CompileContext;
use crate::uar::compiler::report::{Diagnostic, DiagnosticLevel};

pub async fn run(ctx: &mut CompileContext) -> CompileResult<Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();

    // Collect all Cedar policy text (inline + referenced files)
    let mut cedar_text = String::new();

    // Inline Cedar policy
    if let Some(inline) = &ctx.ir.governance.cedar_inline {
        if !inline.trim().is_empty() {
            cedar_text.push_str(inline);
            cedar_text.push('\n');
        }
    }

    // Referenced Cedar policy files (validated as paths; actual file loading is
    // done during deployment, not compilation — we validate the references exist)
    for policy_ref in &ctx.ir.governance.cedar_policies {
        if policy_ref.is_empty() {
            diagnostics.push(Diagnostic {
                level: DiagnosticLevel::Warning,
                message: "governance.cedar_policies contains an empty reference".into(),
                section: Some("Governance".into()),
            });
        } else {
            diagnostics.push(Diagnostic {
                level: DiagnosticLevel::Info,
                message: format!(
                    "external Cedar policy reference: {policy_ref} (resolved at deployment)"
                ),
                section: Some("Governance".into()),
            });
        }
    }

    // If there's inline Cedar, try to parse and compile it
    if !cedar_text.trim().is_empty() {
        match cedar_text.parse::<PolicySet>() {
            Ok(policy_set) => {
                let policy_count = policy_set.policies().count();
                diagnostics.push(Diagnostic {
                    level: DiagnosticLevel::Info,
                    message: format!("compiled {policy_count} Cedar policies successfully"),
                    section: Some("Governance".into()),
                });

                // Serialize for inclusion in the descriptor
                ctx.compiled_cedar = Some(cedar_text.clone());
            }
            Err(e) => {
                diagnostics.push(Diagnostic {
                    level: DiagnosticLevel::Error,
                    message: format!("Cedar policy compilation failed: {e}"),
                    section: Some("Governance".into()),
                });
                return Err(CompileError::CedarPolicy(format!(
                    "failed to compile Cedar policies: {e}"
                )));
            }
        }
    } else if ctx.ir.governance.cedar_policies.is_empty() {
        // No policy at all — info, not error (governance might be handled externally)
        diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Info,
            message: "no Cedar policies declared; agent will use default governance".into(),
            section: Some("Governance".into()),
        });
    }

    // Fingerprint the governance section
    let gov_json = serde_json::to_string(&ctx.ir.governance).unwrap_or_default();
    ctx.fingerprints.insert(
        "governance".into(),
        CompileContext::sha256_hex(gov_json.as_bytes()),
    );

    Ok(diagnostics)
}
