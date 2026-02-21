//! A2UI schema registry — in-memory store of all registered artifact schemas.
//!
//! Populated at startup with built-in schemas. User-defined schemas can be registered
//! at runtime by name. The registry is shared across requests via `Arc`.

use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use super::schema::ArtifactSchema;

/// Thread-safe in-memory registry of [`ArtifactSchema`] entries.
#[derive(Debug, Default)]
pub struct A2uiRegistry {
    schemas: RwLock<HashMap<String, ArtifactSchema>>,
}

impl A2uiRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a registry pre-populated with all five built-in schemas.
    pub fn with_builtins() -> Arc<Self> {
        let registry = Self::new();
        let builtins = vec![
            ArtifactSchema::builtin_form(),
            ArtifactSchema::builtin_confirm(),
            ArtifactSchema::builtin_select(),
            ArtifactSchema::builtin_text_input(),
            ArtifactSchema::builtin_display(),
        ];

        // SAFETY: we hold the only Arc reference at this point so we can safely block.
        // Using try_write succeeds because no other reader exists yet.
        let mut guard = registry
            .schemas
            .try_write()
            .expect("new registry has no contention");
        for schema in builtins {
            guard.insert(schema.schema_id.clone(), schema);
        }
        drop(guard);

        Arc::new(registry)
    }

    /// Register a user-defined schema. Overwrites any existing schema with the same ID.
    pub async fn register(&self, schema: ArtifactSchema) {
        let mut guard = self.schemas.write().await;
        guard.insert(schema.schema_id.clone(), schema);
    }

    /// Look up a schema by its `schema_id`.
    pub async fn get(&self, schema_id: &str) -> Option<ArtifactSchema> {
        let guard = self.schemas.read().await;
        guard.get(schema_id).cloned()
    }

    /// Return all registered schemas sorted by `schema_id`.
    pub async fn list(&self) -> Vec<ArtifactSchema> {
        let guard = self.schemas.read().await;
        let mut schemas: Vec<_> = guard.values().cloned().collect();
        schemas.sort_by(|a, b| a.schema_id.cmp(&b.schema_id));
        schemas
    }

    /// Returns true if the given `schema_id` is registered.
    pub async fn contains(&self, schema_id: &str) -> bool {
        let guard = self.schemas.read().await;
        guard.contains_key(schema_id)
    }
}
