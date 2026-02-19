//! Stage 03: Validate MCP Server Configuration
//!
//! Verifies MCP server endpoints, auth configuration, and cross-references
//! between tools and their declared MCP servers.

use std::collections::HashSet;

use crate::uar::compiler::error::CompileResult;
use crate::uar::compiler::pipeline::CompileContext;
use crate::uar::compiler::report::{Diagnostic, DiagnosticLevel};

pub async fn run(ctx: &mut CompileContext) -> CompileResult<Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();

    // Collect declared server IDs for cross-reference validation
    let server_ids: HashSet<String> = ctx
        .ir
        .mcp_servers
        .servers
        .iter()
        .map(|s| s.id.clone())
        .collect();

    // Validate each MCP server declaration
    for server in &ctx.ir.mcp_servers.servers {
        if server.id.is_empty() {
            diagnostics.push(Diagnostic {
                level: DiagnosticLevel::Error,
                message: "mcp_servers[*].id must not be empty".into(),
                section: Some("MCP Servers".into()),
            });
        }

        if server.url.is_empty() {
            diagnostics.push(Diagnostic {
                level: DiagnosticLevel::Error,
                message: format!("mcp_servers['{}'].url must not be empty", server.id),
                section: Some("MCP Servers".into()),
            });
        }

        // Validate auth config — if auth is declared, token_env must reference
        // a plausible environment variable name
        if let Some(auth) = &server.auth {
            if auth.auth_type.is_empty() {
                diagnostics.push(Diagnostic {
                    level: DiagnosticLevel::Warning,
                    message: format!("mcp_servers['{}'].auth.type is empty", server.id),
                    section: Some("MCP Servers".into()),
                });
            }
            if let Some(token_env) = &auth.token_env {
                if token_env.is_empty() {
                    diagnostics.push(Diagnostic {
                        level: DiagnosticLevel::Warning,
                        message: format!(
                            "mcp_servers['{}'].auth.token_env is declared but empty",
                            server.id
                        ),
                        section: Some("MCP Servers".into()),
                    });
                }
            }
        }
    }

    // Cross-reference: tools that declare a `server` must reference a declared MCP server
    for tool in &ctx.ir.tools.tools {
        if let Some(server_ref) = &tool.server {
            if !server_ref.is_empty() && !server_ids.contains(server_ref) {
                diagnostics.push(Diagnostic {
                    level: DiagnosticLevel::Error,
                    message: format!(
                        "tool '{}' references MCP server '{}' which is not declared",
                        tool.name, server_ref
                    ),
                    section: Some("Tools / MCP Servers".into()),
                });
            }
        }
    }

    // Fingerprint the MCP section
    let mcp_json = serde_json::to_string(&ctx.ir.mcp_servers).unwrap_or_default();
    ctx.fingerprints.insert(
        "mcp_servers".into(),
        CompileContext::sha256_hex(mcp_json.as_bytes()),
    );

    Ok(diagnostics)
}
