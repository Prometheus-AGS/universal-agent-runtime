//! Transport-free memory administration.
//!
//! WHY THIS DOES NOT GO THROUGH `PersistenceLayer`
//!
//! `PersistenceLayer::save_memory` and `search_memory` are documented NO-OP
//! STUBS on `SurrealDbProvider` — they log "use AppState::memory_service for
//! real persistence" and return `Ok(())` / `vec![]`. Memories live in
//! surreal-memory's own SurrealDB instance, which `MemoryService` owns. Routing
//! admin calls through the persistence layer would compile, run, and silently
//! do nothing — the worst possible failure for a store.
//!
//! WHY THE SERVICE IS AN `Option` HERE
//!
//! `MemoryService` opens a second embedded store and can fail to initialise, so
//! the embedded kernel holds it as `Option`. Every function takes that Option
//! and returns a typed error rather than panicking or returning an empty list,
//! so a caller can tell "no memories saved" from "memory is off on this host".

use std::sync::Arc;

use crate::uar::domain::memory::{Memory, MemoryScope, MemoryType};
use crate::uar::memory::service::MemoryService;
use crate::uar::security::claims::UserContext;
use crate::{Result, UarError};

/// Uniform error when the runtime was built without a memory service.
fn unavailable() -> UarError {
    UarError::config(
        "E_MEMORY_SERVICE_UNAVAILABLE",
        "this runtime was built without a memory service (memory.enabled = false)",
    )
}

fn require(service: Option<&Arc<MemoryService>>) -> Result<&Arc<MemoryService>> {
    service.ok_or_else(unavailable)
}

/// Memories visible to a user, optionally narrowed to an agent or session.
///
/// `user_ctx` is required, not optional: memories are per-user state, and a
/// listing that ignored it would show one user's memories to another.
pub async fn list(
    memory: Option<&Arc<MemoryService>>,
    user_ctx: &UserContext,
    agent_id: Option<&str>,
    session_id: Option<&str>,
) -> Result<Vec<Memory>> {
    require(memory)?
        .list(user_ctx, agent_id, session_id)
        .await
        .map_err(|error| UarError::config("E_MEMORY_LIST_FAILED", error.to_string()))
}

/// Load one memory by id.
pub async fn get(memory: Option<&Arc<MemoryService>>, id: &str) -> Result<Option<Memory>> {
    require(memory)?
        .get(id)
        .await
        .map_err(|error| UarError::config("E_MEMORY_GET_FAILED", error.to_string()))
}

/// Add a memory.
///
/// Goes through the service rather than the store because the service owns
/// EMBEDDING: a row written without its vector is invisible to every later
/// semantic search, which presents as data loss rather than a missing index.
pub async fn add(
    memory: Option<&Arc<MemoryService>>,
    content: impl Into<String>,
    scope: MemoryScope,
    memory_type: MemoryType,
    user_ctx: &UserContext,
    agent_id: Option<&str>,
    session_id: Option<&str>,
) -> Result<Memory> {
    require(memory)?
        // The service API gained categories/metadata/importance; this shim keeps
        // its original signature, so it forwards the crate's standard defaults
        // (no categories, no metadata, importance 0.5 — see
        // `AddMemoryParams::default_importance` in memory/mcp_server.rs).
        .add(
            content,
            scope,
            memory_type,
            user_ctx,
            agent_id,
            session_id,
            Vec::new(),
            None,
            0.5,
        )
        .await
        .map_err(|error| UarError::config("E_MEMORY_ADD_FAILED", error.to_string()))
}

/// Replace a memory's content.
///
/// The service records a history entry, so an edit is auditable rather than
/// destructive — a user can see what a memory used to say.
pub async fn update(
    memory: Option<&Arc<MemoryService>>,
    id: &str,
    content: String,
) -> Result<Memory> {
    require(memory)?
        .update(id, content)
        .await
        .map_err(|error| UarError::config("E_MEMORY_UPDATE_FAILED", error.to_string()))
}

/// Delete a single memory.
pub async fn delete(memory: Option<&Arc<MemoryService>>, id: &str) -> Result<()> {
    require(memory)?
        .delete(id)
        .await
        .map_err(|error| UarError::config("E_MEMORY_DELETE_FAILED", error.to_string()))
}

/// Semantic search over a user's memories.
pub async fn search(
    memory: Option<&Arc<MemoryService>>,
    query: &str,
    user_ctx: &UserContext,
    agent_id: Option<&str>,
    session_id: Option<&str>,
    limit: usize,
    categories: Option<&[String]>,
) -> Result<Vec<Memory>> {
    require(memory)?
        .search(query, user_ctx, agent_id, session_id, limit, categories)
        .await
        .map_err(|error| UarError::config("E_MEMORY_SEARCH_FAILED", error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uar::security::claims::UserClaims;

    fn ctx() -> UserContext {
        // Constructed field-by-field rather than via `Default`: `UserClaims` is an
        // auth token payload, and a blanket `Default` on it would let production
        // code conjure an unauthenticated identity by accident.
        UserContext {
            user_id: "u1".to_string(),
            tenant_id: None,
            claims: UserClaims {
                sub: "u1".to_string(),
                name: None,
                roles: None,
                tenant_id: None,
                uar_instance_id: None,
                exp: 0,
            },
        }
    }

    /// A runtime built without memory must report WHY. Returning an empty list
    /// instead would read as "no memories saved", which is a different fact and
    /// the exact ambiguity the `unavailable` contract exists to remove.
    #[tokio::test]
    async fn every_operation_reports_a_reason_when_memory_is_off() {
        let user = ctx();

        let listed = list(None, &user, None, None).await;
        assert!(listed.is_err(), "listing must not silently return empty");
        assert!(get(None, "m1").await.is_err());
        assert!(
            add(
                None,
                "x",
                MemoryScope::default(),
                MemoryType::default(),
                &user,
                None,
                None
            )
            .await
            .is_err()
        );
        assert!(update(None, "m1", "x".to_string()).await.is_err());
        assert!(delete(None, "m1").await.is_err());
        assert!(search(None, "q", &user, None, None, 5, None).await.is_err());

        let error = listed.expect_err("checked above").to_string();
        assert!(
            error.contains("without a memory service"),
            "the error must name a cause a UI can show, got: {error}"
        );
    }
}
