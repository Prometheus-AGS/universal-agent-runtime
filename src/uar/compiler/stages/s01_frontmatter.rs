//! Stage 01: Validate Frontmatter
//!
//! Validates metadata version, schema compatibility, and required identity
//! fields. Also validates the v2 sections (CH-12/CH-13): model requirements,
//! prompt dialect, RAG configuration, context strategy, API harness.

use crate::uar::compiler::error::CompileResult;
use crate::uar::compiler::ir::ContextStrategySection;
use crate::uar::compiler::pipeline::CompileContext;
use crate::uar::compiler::report::{Diagnostic, DiagnosticLevel};

/// Dialect names `PromptDialect::detect` (`src/llm/prompt_dialect.rs`)
/// recognizes as explicit overrides. Kept in sync by hand — CH-04's
/// dialect module has no public "list known variants" helper to import.
const KNOWN_DIALECTS: [&str; 7] = [
    "anthropic_xml",
    "openai_json",
    "kimi_markdown",
    "glm_thinking",
    "qwen_hybrid",
    "minimax_structured",
    "generic",
];

/// Transport protocol names `api_harness.protocols` recognizes.
const KNOWN_PROTOCOLS: [&str; 4] = ["a2a", "agui", "openai", "rest"];

/// `stream_mode` values the SSE surface (`src/server.rs`, CH-21) recognizes.
const KNOWN_STREAM_MODES: [&str; 4] = ["openai", "agui", "dual", "agui_spec"];

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

    // ── v2 sections (CH-12/CH-13) ────────────────────────────────────────

    // model_requirements: numeric bounds only (booleans have no invalid state).
    if let Some(min_context) = ctx.ir.model_requirements.min_context
        && min_context == 0
    {
        diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Warning,
            message: "model_requirements.min_context is 0 — likely meant to be unset (omit the field) rather than zero".into(),
            section: Some("Model Requirements".into()),
        });
    }
    if let Some(cost) = ctx.ir.model_requirements.max_cost_per_1m_input
        && cost < 0.0
    {
        diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            message: "model_requirements.max_cost_per_1m_input must not be negative".into(),
            section: Some("Model Requirements".into()),
        });
    }

    // prompt_dialect: an explicit override must be a dialect CH-04 recognizes —
    // an unrecognized value would silently fall through to auto-detect at
    // request time, masking an author typo.
    if let Some(dialect) = &ctx.ir.prompt_dialect.dialect
        && !KNOWN_DIALECTS.contains(&dialect.as_str())
    {
        diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            message: format!(
                "prompt_dialect.dialect '{dialect}' is not recognized; expected one of {KNOWN_DIALECTS:?}"
            ),
            section: Some("Prompt Dialect".into()),
        });
    }

    // rag_configuration: enabling RAG with no knowledge bases declared is
    // very likely an oversight (nothing to retrieve from), but not fatal —
    // KBs can be attached post-compile via the admin UI.
    if ctx.ir.rag_configuration.enabled && ctx.ir.rag_configuration.knowledge_base_ids.is_empty() {
        diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Warning,
            message: "rag_configuration.enabled is true but knowledge_base_ids is empty — this agent will have nothing to retrieve from until knowledge bases are attached".into(),
            section: Some("RAG Configuration".into()),
        });
    }

    // context_strategy: numeric fields that are explicitly zero are almost
    // certainly author mistakes (a real "keep nothing" strategy is `None`).
    let zero_field_error = |field: &str| Diagnostic {
        level: DiagnosticLevel::Error,
        message: format!("context_strategy.{field} must be greater than 0"),
        section: Some("Context Strategy".into()),
    };
    match &ctx.ir.context_strategy {
        ContextStrategySection::SlidingWindow {
            max_messages: Some(0),
        } => diagnostics.push(zero_field_error("max_messages")),
        ContextStrategySection::Summarize {
            threshold: Some(0), ..
        } => diagnostics.push(zero_field_error("threshold")),
        ContextStrategySection::Hierarchical {
            short_term_turns: Some(0),
            ..
        } => diagnostics.push(zero_field_error("short_term_turns")),
        _ => {}
    }

    // api_harness: unrecognized protocol/stream_mode names are warnings, not
    // errors — a deployment may add a transport this compiler version
    // doesn't know about yet, and the declaration is still meaningful
    // documentation even if unenforceable here.
    for protocol in &ctx.ir.api_harness.protocols {
        if !KNOWN_PROTOCOLS.contains(&protocol.as_str()) {
            diagnostics.push(Diagnostic {
                level: DiagnosticLevel::Warning,
                message: format!(
                    "api_harness.protocols entry '{protocol}' is not one of the known transports {KNOWN_PROTOCOLS:?}"
                ),
                section: Some("API Harness".into()),
            });
        }
    }
    if let Some(mode) = &ctx.ir.api_harness.stream_mode
        && !KNOWN_STREAM_MODES.contains(&mode.as_str())
    {
        diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Warning,
            message: format!(
                "api_harness.stream_mode '{mode}' is not one of the known stream modes {KNOWN_STREAM_MODES:?}"
            ),
            section: Some("API Harness".into()),
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
