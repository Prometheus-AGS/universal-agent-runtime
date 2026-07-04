//! Stage 08: Emit Signed Descriptor
//!
//! Emits the final canonical JSON descriptor with all collected fingerprints,
//! Cedar policy, endpoint bindings, and PEP bindings. The actual signing is
//! handled by the pipeline orchestrator after this stage.

use crate::uar::compiler::error::CompileResult;
use crate::uar::compiler::ir::ContextStrategySection;
use crate::uar::compiler::pipeline::{CompileContext, CompiledDescriptor};
use crate::uar::compiler::report::{Diagnostic, DiagnosticLevel};

/// True if the document declares any v2 section (CH-12) with a non-default
/// value. Used only to pick the descriptive `schema` string below — v1
/// documents compile and run identically regardless of this flag.
fn uses_any_v2_section(ctx: &CompileContext) -> bool {
    let mr = &ctx.ir.model_requirements;
    let has_model_requirements = mr.needs_tools
        || mr.needs_reasoning
        || mr.needs_vision
        || mr.needs_structured_output
        || mr.min_context.is_some()
        || mr.max_cost_per_1m_input.is_some()
        || mr.preferred_provider.is_some();

    let pd = &ctx.ir.prompt_dialect;
    let has_prompt_dialect = pd.dialect.is_some() || pd.wants_reasoning || pd.hard;

    let has_rag_configuration = ctx.ir.rag_configuration.enabled
        || ctx.ir.rag_configuration.decomposition
        || ctx.ir.rag_configuration.verification
        || ctx.ir.rag_configuration.audit
        || !ctx.ir.rag_configuration.knowledge_base_ids.is_empty();

    let has_context_strategy = !matches!(ctx.ir.context_strategy, ContextStrategySection::Auto);

    let ah = &ctx.ir.api_harness;
    let has_api_harness = !ah.protocols.is_empty() || ah.stream_mode.is_some();

    has_model_requirements
        || has_prompt_dialect
        || has_rag_configuration
        || has_context_strategy
        || has_api_harness
}

pub async fn run(ctx: &mut CompileContext) -> CompileResult<Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();

    // Get the public key for embedding in the descriptor
    let pub_key_bytes = ctx.key_provider.public_key_bytes().await?;
    let pub_key_hex = pub_key_bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();

    // CH-12/CH-13: a document that declares any v2 section is tagged
    // schema/v2 in the emitted descriptor. This is purely descriptive
    // metadata (not a compile gate) — v1 documents keep the original schema
    // string and compile/run identically either way.
    let schema = if uses_any_v2_section(ctx) {
        "uar-agent-descriptor/v2"
    } else {
        "uar-agent-descriptor/v1"
    };

    // Build the canonical descriptor
    let descriptor = CompiledDescriptor {
        schema: schema.into(),
        agent_id: ctx.ir.identity.name.clone(),
        version: ctx.ir.metadata.version.clone(),
        content_hash: String::new(), // Will be set below
        signer_public_key: pub_key_hex,
        payload: ctx.ir.clone(),
        fingerprints: ctx.fingerprints.clone(),
        cedar_policy: ctx.compiled_cedar.clone(),
        endpoints: ctx.actor_routes.clone(),
        pep_bindings: ctx.pep_bindings.clone(),
    };

    // Compute the content hash of the descriptor (excluding the hash field itself)
    let mut hash_target = descriptor.clone();
    hash_target.content_hash = String::new();
    let canonical_json = serde_json::to_string(&hash_target).unwrap_or_default();
    let content_hash = CompileContext::sha256_hex(canonical_json.as_bytes());

    let mut final_descriptor = descriptor;
    final_descriptor.content_hash = content_hash.clone();

    diagnostics.push(Diagnostic {
        level: DiagnosticLevel::Info,
        message: format!(
            "emitted descriptor for agent '{}' v{} (hash: {:.16}…)",
            final_descriptor.agent_id, final_descriptor.version, content_hash
        ),
        section: None,
    });

    ctx.descriptor = Some(final_descriptor);

    Ok(diagnostics)
}
