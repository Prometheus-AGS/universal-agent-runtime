//! Stage 07: Install PEP Enforcement bindings
//!
//! Derives Policy Enforcement Point (PEP) bindings for each capability surface
//! declared in the IR. Every sensitive boundary (LLM invocation, tool execution,
//! file access, network, plugin, A2A) gets a corresponding PEP binding.

use crate::uar::compiler::error::CompileResult;
use crate::uar::compiler::pipeline::{CompileContext, PepBinding};
use crate::uar::compiler::report::{Diagnostic, DiagnosticLevel};

pub async fn run(ctx: &mut CompileContext) -> CompileResult<Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    let mut bindings = Vec::new();

    // LLM invocation PEP — always required
    bindings.push(PepBinding {
        surface: "llm_invocation".into(),
        action: "Action::\"invoke_llm\"".into(),
        required: true,
    });

    // Tool execution PEP — one per declared tool
    if !ctx.ir.tools.tools.is_empty() {
        bindings.push(PepBinding {
            surface: "tool_execution".into(),
            action: "Action::\"execute_tool\"".into(),
            required: true,
        });
    }

    // MCP server access PEP
    if !ctx.ir.mcp_servers.servers.is_empty() {
        bindings.push(PepBinding {
            surface: "mcp_access".into(),
            action: "Action::\"access_mcp\"".into(),
            required: true,
        });
    }

    // Knowledge base access PEP
    if !ctx.ir.knowledge.sources.is_empty() {
        bindings.push(PepBinding {
            surface: "knowledge_access".into(),
            action: "Action::\"access_knowledge\"".into(),
            required: true,
        });
    }

    // A2A communication PEP
    if !ctx.ir.a2a.endpoints.is_empty() || !ctx.ir.a2a.dependencies.is_empty() {
        bindings.push(PepBinding {
            surface: "a2a_communication".into(),
            action: "Action::\"a2a_communicate\"".into(),
            required: true,
        });
    }

    // File access PEP (if code execution is enabled)
    if ctx.ir.capabilities.code_execution {
        bindings.push(PepBinding {
            surface: "file_access".into(),
            action: "Action::\"access_file\"".into(),
            required: true,
        });
    }

    // Network access PEP (if web browsing is enabled)
    if ctx.ir.capabilities.web_browsing {
        bindings.push(PepBinding {
            surface: "network_access".into(),
            action: "Action::\"access_network\"".into(),
            required: true,
        });
    }

    diagnostics.push(Diagnostic {
        level: DiagnosticLevel::Info,
        message: format!("derived {} PEP enforcement bindings", bindings.len()),
        section: Some("Governance".into()),
    });

    ctx.pep_bindings = bindings;

    Ok(diagnostics)
}
