//! Transport-free A2UI artifact-schema enumeration.
//!
//! The embedded control plane had no A2UI surface at all — a client could not
//! ask which artifact schemas the runtime knows how to render.
//!
//! This matters for a specific, observed reason: an A2UI surface has never
//! appeared in any device certification dump in this suite, because nothing on
//! a mobile build emits an artifact. Being able to LIST the registered schemas
//! separates "the renderer supports nothing" from "nothing produced an artifact
//! this run" — two very different diagnoses that currently look identical.
//!
//! Read-only by design. Registering a schema mutates how every future artifact
//! renders, so it belongs to composition at build time rather than to an admin
//! call that could change rendering behaviour mid-session.

use std::sync::Arc;

use crate::uar::a2ui::registry::A2uiRegistry;
use crate::uar::a2ui::schema::ArtifactSchema;

/// Every artifact schema this runtime can render, sorted by id.
pub async fn list(registry: &Arc<A2uiRegistry>) -> Vec<ArtifactSchema> {
    registry.list().await
}

/// Whether a specific schema is registered.
///
/// Cheaper than `list` when a caller only needs to answer "can this runtime
/// render X?" before emitting an artifact that would otherwise fail closed.
pub async fn supports(registry: &Arc<A2uiRegistry>, schema_id: &str) -> bool {
    registry.contains(schema_id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_runtime_with_builtins_reports_the_schemas_it_can_render() {
        // The distinction this exists to make: an empty list means the renderer
        // supports nothing, which is NOT the same as "no artifact was emitted".
        let registry = A2uiRegistry::with_builtins();
        let schemas = list(&registry).await;
        assert!(
            !schemas.is_empty(),
            "a runtime built with builtins must report them, or a client cannot \
             tell an unsupported schema from an absent artifact"
        );
    }

    #[tokio::test]
    async fn supports_matches_the_listing() {
        let registry = A2uiRegistry::with_builtins();
        let schemas = list(&registry).await;
        let Some(first) = schemas.first() else {
            return;
        };
        assert!(supports(&registry, &first.schema_id).await);
        assert!(
            !supports(&registry, "definitely-not-a-registered-schema").await,
            "a probe that always returns true would be useless"
        );
    }
}
