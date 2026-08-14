//! Scope resolution logic for the agent memory system.
//!
//! Determines which [`MemoryScope`] to use when writing a new memory and
//! which scopes are visible for a given user context during searches.

use surreal_memory::MemoryScope;

use crate::uar::security::claims::UserContext;

/// Resolve the effective scope for a new memory given the caller's identity
/// and an optional explicit scope hint.
///
/// Rules:
/// - Anonymous callers may only write `Global` or `Agent` scoped memories.
/// - Authenticated callers may write any scope.
/// - If no explicit scope is provided, authenticated callers default to `User`,
///   anonymous callers default to `Global`.
pub fn resolve_scope(user_ctx: &UserContext, explicit_scope: Option<MemoryScope>) -> MemoryScope {
    let is_anonymous = user_ctx.user_id == "anonymous";

    match explicit_scope {
        Some(scope) => {
            // Downgrade user/session/task scope to global for anonymous callers.
            if is_anonymous {
                match scope {
                    MemoryScope::User | MemoryScope::Session | MemoryScope::Task => {
                        tracing::warn!(
                            user_id = %user_ctx.user_id,
                            requested_scope = ?scope,
                            "Anonymous caller requested user-scoped memory; downgrading to Global"
                        );
                        MemoryScope::Global
                    }
                    other => other,
                }
            } else {
                scope
            }
        }
        None => {
            if is_anonymous {
                MemoryScope::Global
            } else {
                MemoryScope::User
            }
        }
    }
}

/// Check whether a given scope requires a valid user identity.
pub fn scope_requires_auth(scope: &MemoryScope) -> bool {
    matches!(
        scope,
        MemoryScope::User | MemoryScope::Session | MemoryScope::Task
    )
}

/// Return the `user_id` to pass to storage queries based on caller identity.
/// Returns `None` for anonymous callers (so they see Global + Agent memories only).
pub fn user_id_for_query(user_ctx: &UserContext) -> Option<&str> {
    if user_ctx.user_id == "anonymous" {
        None
    } else {
        Some(user_ctx.user_id.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uar::security::claims::UserClaims;

    fn make_anon() -> UserContext {
        UserContext {
            user_id: "anonymous".to_string(),
            tenant_id: None,
            claims: UserClaims {
                sub: "anonymous".to_string(),
                name: None,
                roles: None,
                tenant_id: None,
                exp: usize::MAX,
            },
        }
    }

    fn make_user(id: &str) -> UserContext {
        UserContext {
            user_id: id.to_string(),
            tenant_id: None,
            claims: UserClaims {
                sub: id.to_string(),
                name: Some("Test User".to_string()),
                roles: Some(vec!["user".to_string()]),
                tenant_id: None,
                exp: usize::MAX,
            },
        }
    }

    #[test]
    fn anonymous_defaults_to_global() {
        let scope = resolve_scope(&make_anon(), None);
        assert_eq!(scope, MemoryScope::Global);
    }

    #[test]
    fn authenticated_defaults_to_user() {
        let scope = resolve_scope(&make_user("u1"), None);
        assert_eq!(scope, MemoryScope::User);
    }

    #[test]
    fn anonymous_user_scope_downgrades_to_global() {
        let scope = resolve_scope(&make_anon(), Some(MemoryScope::User));
        assert_eq!(scope, MemoryScope::Global);
    }

    #[test]
    fn anonymous_agent_scope_kept() {
        let scope = resolve_scope(&make_anon(), Some(MemoryScope::Agent));
        assert_eq!(scope, MemoryScope::Agent);
    }

    #[test]
    fn authenticated_user_scope_explicit() {
        let scope = resolve_scope(&make_user("u1"), Some(MemoryScope::Session));
        assert_eq!(scope, MemoryScope::Session);
    }
}
