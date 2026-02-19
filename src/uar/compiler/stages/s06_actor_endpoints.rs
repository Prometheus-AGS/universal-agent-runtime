//! Stage 06: Register Actor Endpoints
//!
//! Binds `a2a.endpoints[*].id` into the [`EndpointRegistry`] trait for routing.
//! Uses the in-memory registry by default; extensible to distributed registries.

use crate::uar::compiler::error::CompileResult;
use crate::uar::compiler::pipeline::CompileContext;
use crate::uar::compiler::registries::EndpointBinding;
use crate::uar::compiler::report::{Diagnostic, DiagnosticLevel};

pub async fn run(ctx: &mut CompileContext) -> CompileResult<Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();

    for endpoint in &ctx.ir.a2a.endpoints {
        if endpoint.id.is_empty() {
            continue; // Already reported in Stage 04
        }

        let method = endpoint.method.clone().unwrap_or_else(|| "POST".into());

        let route = format!(
            "/a2a/{}/{}",
            ctx.ir.agent_name.to_lowercase().replace(' ', "-"),
            endpoint.id
        );

        let input_hash = ctx
            .fingerprints
            .get(&format!("a2a.{}.input", endpoint.id))
            .cloned();
        let output_hash = ctx
            .fingerprints
            .get(&format!("a2a.{}.output", endpoint.id))
            .cloned();

        let binding = EndpointBinding {
            endpoint_id: endpoint.id.clone(),
            agent_id: ctx.ir.identity.name.clone(),
            method,
            route: route.clone(),
            input_schema_hash: input_hash,
            output_schema_hash: output_hash,
        };

        // Register in the endpoint registry
        if let Err(e) = ctx.endpoint_registry.register(binding.clone()).await {
            diagnostics.push(Diagnostic {
                level: DiagnosticLevel::Warning,
                message: format!("failed to register endpoint '{}': {e}", endpoint.id),
                section: Some("A2A Contracts".into()),
            });
        } else {
            diagnostics.push(Diagnostic {
                level: DiagnosticLevel::Info,
                message: format!("registered endpoint '{}' at {route}", endpoint.id),
                section: Some("A2A Contracts".into()),
            });
        }

        ctx.actor_routes.push(binding);
    }

    Ok(diagnostics)
}
