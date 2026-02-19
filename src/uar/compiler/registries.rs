//! Trait-based registries for schema resolution and endpoint routing.
//!
//! These abstractions provide functional local defaults (in-memory `HashMap`s)
//! that validate structural correctness within UAR's own scope. External backends
//! (Redis, etcd, external schema registries) can be plugged in by implementing
//! the same traits.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::error::CompileError;

// ─────────────────────────────────────────────
// Schema Registry (for A2UI § 06)
// ─────────────────────────────────────────────

/// Schema resolution abstraction.
///
/// Stage 02 (A2UI) uses this to validate and resolve schema references.
/// The [`InMemorySchemaRegistry`] validates structure locally; future
/// implementations can resolve external URIs.
#[async_trait]
pub trait SchemaRegistry: Send + Sync + std::fmt::Debug {
    /// Resolve a schema by ID. Returns `None` if not found.
    async fn resolve(&self, schema_id: &str) -> Result<Option<serde_json::Value>, CompileError>;

    /// Register a schema under the given ID.
    async fn register(
        &self,
        schema_id: &str,
        schema: serde_json::Value,
    ) -> Result<(), CompileError>;

    /// List all registered schema IDs.
    async fn list_ids(&self) -> Result<Vec<String>, CompileError>;
}

/// In-memory schema registry backed by a `HashMap`.
#[derive(Debug, Default)]
pub struct InMemorySchemaRegistry {
    schemas: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}

impl InMemorySchemaRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl SchemaRegistry for InMemorySchemaRegistry {
    async fn resolve(&self, schema_id: &str) -> Result<Option<serde_json::Value>, CompileError> {
        let guard = self.schemas.read().await;
        Ok(guard.get(schema_id).cloned())
    }

    async fn register(
        &self,
        schema_id: &str,
        schema: serde_json::Value,
    ) -> Result<(), CompileError> {
        let mut guard = self.schemas.write().await;
        guard.insert(schema_id.to_string(), schema);
        Ok(())
    }

    async fn list_ids(&self) -> Result<Vec<String>, CompileError> {
        let guard = self.schemas.read().await;
        Ok(guard.keys().cloned().collect())
    }
}

// ─────────────────────────────────────────────
// Endpoint Registry (for Actor Endpoints § 06)
// ─────────────────────────────────────────────

/// Binding between an A2A endpoint ID and its routing target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointBinding {
    /// Endpoint ID (matches `a2a.endpoints[*].id` in the IR).
    pub endpoint_id: String,
    /// Agent ID that owns this endpoint.
    pub agent_id: String,
    /// HTTP method (e.g., "POST").
    pub method: String,
    /// Local route path.
    pub route: String,
    /// Input JSON Schema fingerprint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema_hash: Option<String>,
    /// Output JSON Schema fingerprint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema_hash: Option<String>,
}

/// Endpoint routing registry abstraction.
///
/// Stage 06 uses this to register A2A endpoints. The [`InMemoryEndpointRegistry`]
/// stores bindings locally; future implementations can use service meshes or
/// distributed registries.
#[async_trait]
pub trait EndpointRegistry: Send + Sync + std::fmt::Debug {
    /// Register an endpoint binding.
    async fn register(&self, binding: EndpointBinding) -> Result<(), CompileError>;

    /// Resolve an endpoint by ID. Returns `None` if not found.
    async fn resolve(&self, endpoint_id: &str) -> Result<Option<EndpointBinding>, CompileError>;

    /// List all registered endpoint bindings.
    async fn list(&self) -> Result<Vec<EndpointBinding>, CompileError>;
}

/// In-memory endpoint registry backed by a `HashMap`.
#[derive(Debug, Default)]
pub struct InMemoryEndpointRegistry {
    bindings: Arc<RwLock<HashMap<String, EndpointBinding>>>,
}

impl InMemoryEndpointRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl EndpointRegistry for InMemoryEndpointRegistry {
    async fn register(&self, binding: EndpointBinding) -> Result<(), CompileError> {
        let mut guard = self.bindings.write().await;
        guard.insert(binding.endpoint_id.clone(), binding);
        Ok(())
    }

    async fn resolve(&self, endpoint_id: &str) -> Result<Option<EndpointBinding>, CompileError> {
        let guard = self.bindings.read().await;
        Ok(guard.get(endpoint_id).cloned())
    }

    async fn list(&self) -> Result<Vec<EndpointBinding>, CompileError> {
        let guard = self.bindings.read().await;
        Ok(guard.values().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_schema_registry_crud() {
        let reg = InMemorySchemaRegistry::new();
        let schema = serde_json::json!({"type": "object"});
        reg.register("test-schema", schema.clone()).await.unwrap();

        let resolved = reg.resolve("test-schema").await.unwrap();
        assert_eq!(resolved, Some(schema));

        let ids = reg.list_ids().await.unwrap();
        assert_eq!(ids, vec!["test-schema"]);

        assert!(reg.resolve("nonexistent").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_endpoint_registry_crud() {
        let reg = InMemoryEndpointRegistry::new();
        let binding = EndpointBinding {
            endpoint_id: "compile".into(),
            agent_id: "compiler-agent".into(),
            method: "POST".into(),
            route: "/a2a/compiler/compile".into(),
            input_schema_hash: None,
            output_schema_hash: None,
        };
        reg.register(binding.clone()).await.unwrap();

        let resolved = reg.resolve("compile").await.unwrap();
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().agent_id, "compiler-agent");

        let all = reg.list().await.unwrap();
        assert_eq!(all.len(), 1);
    }
}
