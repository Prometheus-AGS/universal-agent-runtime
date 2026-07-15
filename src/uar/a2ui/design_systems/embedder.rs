//! Component embedder — computes and stores semantic-search embeddings for
//! the A2UI design-system component catalog.
//!
//! Ported in spirit (not verbatim) from
//! `flint-forge/crates/fdb-gateway/src/a2ui_embedder.rs`. The flint-forge
//! source is a long-running Postgres `LISTEN`/`NOTIFY` background task that
//! calls an in-database `llm.embed()` function. UAR has no equivalent
//! in-database embedding function and its default/`server-full` persistence
//! backend is SurrealDB, not Postgres — so this port:
//!
//! - Uses UAR's existing [`EmbeddingBackend`] trait
//!   (`src/uar/rag/embeddings/mod.rs`) to generate embeddings, instead of a
//!   bespoke in-database function call.
//! - Operates over the backend-agnostic [`DesignSystemStore`] trait instead
//!   of a raw `sqlx::PgPool` + Postgres channel listener, so it works with
//!   the in-memory, SurrealDB, and Postgres store implementations alike.
//! - Exposes `embed_component` (embed one component on demand — the
//!   equivalent of flint-forge's per-notification handler) and
//!   `backfill_missing` (embed every component that doesn't have one yet —
//!   the equivalent of flint-forge's startup backfill). Live-notification
//!   fan-out (the `a2ui_embed` channel) is deferred to Change 20
//!   (`a2ui-realtime-backbone-from-flint-realtime-fabric`), which wires the
//!   `flint-realtime-fabric` SSE/fan-out backbone UAR will use for all A2UI
//!   live updates — building a second, one-off notification mechanism here
//!   would be redundant with that change's scope.
//!
//! The fallback-on-unavailable-model behavior (try a primary model, then a
//! secondary) from flint-forge's `text-embedding-3-large` →
//! `text-embedding-3-small` fallback is preserved as an optional secondary
//! backend parameter.

use std::sync::Arc;

use thiserror::Error;

use super::store::DesignSystemStore;
use super::types::Component;
use crate::uar::rag::embeddings::EmbeddingBackend;

/// Errors that can occur while embedding a component.
#[derive(Debug, Error)]
pub enum EmbedError {
    #[error("storage error: {0}")]
    Store(#[from] anyhow::Error),

    #[error("component not found: {0}")]
    ComponentNotFound(String),

    #[error("embedding backend error: {0}")]
    Backend(#[from] crate::uar::rag::embeddings::EmbeddingError),
}

/// Build the embedding input string from component metadata. Mirrors
/// flint-forge's `build_embedding_text`.
#[must_use]
pub fn build_embedding_text(component: &Component) -> String {
    let mut parts = vec![
        component.slug.clone(),
        component.primitive_type.clone(),
        component.category.clone(),
    ];

    if let Some(desc) = &component.description {
        parts.push(desc.clone());
    }

    parts.push("Usage:".to_string());
    if let Some(examples) = &component.usage_examples {
        parts.push(serde_json::to_string(examples).unwrap_or_default());
    }

    parts.push("Props:".to_string());
    if let Some(props) = component
        .schema
        .get("properties")
        .and_then(|v| v.as_object())
    {
        for key in props.keys() {
            parts.push(key.clone());
        }
    }

    parts.join(" ")
}

/// Embed a single component and persist the result via the store's
/// `set_component_embedding`. If `fallback` is provided and the primary
/// `backend` fails with `BackendUnavailable`, retries with `fallback`.
///
/// # Errors
///
/// Returns [`EmbedError::ComponentNotFound`] if `component_id` does not
/// exist, or [`EmbedError::Backend`] if both the primary and (if provided)
/// fallback embedding backends fail.
pub async fn embed_component(
    store: &dyn DesignSystemStore,
    backend: &Arc<dyn EmbeddingBackend>,
    fallback: Option<&Arc<dyn EmbeddingBackend>>,
    component_id: &str,
) -> Result<(), EmbedError> {
    let component = store
        .get_component(component_id)
        .await?
        .ok_or_else(|| EmbedError::ComponentNotFound(component_id.to_string()))?;

    let text = build_embedding_text(&component);

    let (embedding, model) = match backend.embed_one(&text).await {
        Ok(v) => (v, backend.backend_name().to_string()),
        Err(e) if fallback.is_some() => {
            tracing::info!(
                error = %e,
                backend = backend.backend_name(),
                "a2ui-design-systems embedder: primary backend failed, trying fallback"
            );
            let fb = fallback.expect("checked is_some above");
            (fb.embed_one(&text).await?, fb.backend_name().to_string())
        }
        Err(e) => return Err(e.into()),
    };

    store
        .set_component_embedding(&component.id, embedding, &model)
        .await?;
    Ok(())
}

/// Embed every component that does not yet have an embedding. Errors on
/// individual components are logged and do not abort the backfill (matches
/// flint-forge's `backfill_missing` best-effort behavior). Returns the count
/// of components successfully embedded.
pub async fn backfill_missing(
    store: &dyn DesignSystemStore,
    backend: &Arc<dyn EmbeddingBackend>,
    fallback: Option<&Arc<dyn EmbeddingBackend>>,
) -> Result<usize, EmbedError> {
    let missing = store.list_components_missing_embedding().await?;
    tracing::info!(
        count = missing.len(),
        "a2ui-design-systems embedder: backfill starting"
    );

    let mut embedded = 0usize;
    for component in missing {
        match embed_component(store, backend, fallback, &component.id).await {
            Ok(()) => embedded += 1,
            Err(e) => tracing::warn!(
                error = %e,
                component_id = %component.id,
                "a2ui-design-systems embedder: backfill item failed"
            ),
        }
    }

    tracing::info!(embedded, "a2ui-design-systems embedder: backfill complete");
    Ok(embedded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uar::a2ui::design_systems::store::InMemoryDesignSystemStore;
    use crate::uar::a2ui::design_systems::types::Renderers;
    use crate::uar::rag::embeddings::EmbeddingError;
    use async_trait::async_trait;
    use chrono::Utc;

    #[derive(Debug)]
    struct StubBackend {
        name: &'static str,
        fail: bool,
    }

    #[async_trait]
    impl EmbeddingBackend for StubBackend {
        fn backend_name(&self) -> &str {
            self.name
        }

        fn vector_dimension(&self) -> usize {
            3
        }

        async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
            if self.fail {
                return Err(EmbeddingError::BackendUnavailable(self.name.to_string()));
            }
            Ok(texts.iter().map(|_| vec![0.1, 0.2, 0.3]).collect())
        }
    }

    fn component(id: &str, slug: &str) -> Component {
        let now = Utc::now();
        Component {
            id: id.to_string(),
            slug: slug.to_string(),
            primitive_type: "TextInput".into(),
            category: "input".into(),
            schema: serde_json::json!({
                "properties": { "label": {"type": "string"}, "placeholder": {"type": "string"} }
            }),
            description: Some("A text input field".into()),
            usage_examples: None,
            renderers: Renderers::default(),
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn build_embedding_text_includes_props() {
        let c = component("c1", "text-input");
        let text = build_embedding_text(&c);
        assert!(text.contains("text-input"));
        assert!(text.contains("A text input field"));
        assert!(text.contains("label"));
        assert!(text.contains("placeholder"));
    }

    #[tokio::test]
    async fn embed_component_persists_embedding() {
        let store = InMemoryDesignSystemStore::new();
        store
            .put_component(component("c1", "button"))
            .await
            .unwrap();
        let backend: Arc<dyn EmbeddingBackend> = Arc::new(StubBackend {
            name: "stub",
            fail: false,
        });

        embed_component(&store, &backend, None, "c1").await.unwrap();

        let missing = store.list_components_missing_embedding().await.unwrap();
        assert!(missing.is_empty());
    }

    #[tokio::test]
    async fn embed_component_falls_back_on_primary_failure() {
        let store = InMemoryDesignSystemStore::new();
        store
            .put_component(component("c1", "button"))
            .await
            .unwrap();
        let primary: Arc<dyn EmbeddingBackend> = Arc::new(StubBackend {
            name: "primary",
            fail: true,
        });
        let fallback: Arc<dyn EmbeddingBackend> = Arc::new(StubBackend {
            name: "fallback",
            fail: false,
        });

        embed_component(&store, &primary, Some(&fallback), "c1")
            .await
            .unwrap();

        let missing = store.list_components_missing_embedding().await.unwrap();
        assert!(missing.is_empty());
    }

    #[tokio::test]
    async fn embed_component_missing_component_errors() {
        let store = InMemoryDesignSystemStore::new();
        let backend: Arc<dyn EmbeddingBackend> = Arc::new(StubBackend {
            name: "stub",
            fail: false,
        });
        let err = embed_component(&store, &backend, None, "does-not-exist")
            .await
            .unwrap_err();
        assert!(matches!(err, EmbedError::ComponentNotFound(_)));
    }

    #[tokio::test]
    async fn backfill_missing_embeds_all_and_is_best_effort() {
        let store = InMemoryDesignSystemStore::new();
        store
            .put_component(component("c1", "button"))
            .await
            .unwrap();
        store.put_component(component("c2", "card")).await.unwrap();
        let backend: Arc<dyn EmbeddingBackend> = Arc::new(StubBackend {
            name: "stub",
            fail: false,
        });

        let embedded = backfill_missing(&store, &backend, None).await.unwrap();
        assert_eq!(embedded, 2);

        let missing = store.list_components_missing_embedding().await.unwrap();
        assert!(missing.is_empty());
    }
}
