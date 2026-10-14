//! Scope resolution logic for the agent memory system.
//!
//! Determines which [`MemoryScope`] to use when writing a new memory and
//! which scopes are visible for a given user context during searches.

use surreal_memory::MemoryScope;

use crate::uar::security::claims::UserContext;

/// Scope is a classification within the verified owner's namespace. There is
/// no anonymous-to-global downgrade; MemoryService rejects missing authority.
pub fn resolve_scope(_user_ctx: &UserContext, explicit_scope: Option<MemoryScope>) -> MemoryScope {
    explicit_scope.unwrap_or(MemoryScope::User)
}

pub fn scope_requires_auth(_scope: &MemoryScope) -> bool { true }

/// A missing identity is an error, never an unrestricted storage filter.
pub fn user_id_for_query(user_ctx: &UserContext) -> anyhow::Result<String> {
    Ok(crate::uar::security::claims::CredentialPrincipal::from_verified_context(user_ctx)?.storage_key())
}
