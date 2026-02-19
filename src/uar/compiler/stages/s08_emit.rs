//! Stage 08: Emit Signed Descriptor
//!
//! Emits the final canonical JSON descriptor with all collected fingerprints,
//! Cedar policy, endpoint bindings, and PEP bindings. The actual signing is
//! handled by the pipeline orchestrator after this stage.

use crate::uar::compiler::error::CompileResult;
use crate::uar::compiler::pipeline::{CompileContext, CompiledDescriptor};
use crate::uar::compiler::report::{Diagnostic, DiagnosticLevel};

pub async fn run(ctx: &mut CompileContext) -> CompileResult<Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();

    // Get the public key for embedding in the descriptor
    let pub_key_bytes = ctx.key_provider.public_key_bytes().await?;
    let pub_key_hex = pub_key_bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();

    // Build the canonical descriptor
    let descriptor = CompiledDescriptor {
        schema: "uar-agent-descriptor/v1".into(),
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
