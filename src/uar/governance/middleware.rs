//! Axum middleware for governance policy enforcement.
//!
//! Provides an extractable governance guard that can be applied to routes
//! requiring authorization checks.

use std::sync::Arc;

use axum::{
    Json,
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tracing::warn;

use super::engine::GovernanceEngine;
use super::runtime_control::GovernanceGateHandle;

#[derive(Clone, Debug)]
pub struct GovernanceMiddlewareState {
    engine: Arc<GovernanceEngine>,
}

impl GovernanceMiddlewareState {
    pub fn new(engine: Arc<GovernanceEngine>, _gate: GovernanceGateHandle) -> Self {
        Self { engine }
    }
}

/// JSON error body returned when governance denies a request.
#[derive(Serialize)]
struct GovernanceDenied {
    error: String,
    code: String,
}

/// Governance policy evaluation middleware with AppState.
///
/// This middleware extracts the agent ID and action from request
/// extensions or headers and evaluates them against governance policies.
///
/// If no agent ID is provided, the request passes through (anonymous
/// requests are not subject to agent-level governance).
///
/// # Usage
///
/// ```rust,ignore
/// use axum::middleware;
///
/// let app = Router::new()
///     .route("/api/tools", post(execute_tool))
///     .layer(middleware::from_fn_with_state(
///         state.clone(),
///         governance_layer,
///     ));
/// ```
pub async fn governance_layer(
    State(state): State<GovernanceMiddlewareState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    // Direct tool execution is authorized after authentication by RunManager's
    // exact-invocation admission boundary. Never authorize that path from the
    // caller-controlled X-Agent-Id header before auth has bound its principal.
    if is_direct_tool_execution(&request) {
        return next.run(request).await;
    }

    // The header marks an agent-initiated request but never supplies identity.
    if !request.headers().contains_key("X-Agent-Id") {
        return next.run(request).await;
    }
    let owner = request
        .extensions()
        .get::<crate::uar::security::claims::UserContext>()
        .and_then(|user| {
            crate::uar::runtime::actor::messages::ActorOwner::from_verified_context(user).ok()
        });
    let Some(owner) = owner else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(GovernanceDenied {
                error: "Governed agent request requires authenticated identity".to_string(),
                code: "GOVERNANCE_IDENTITY_UNVERIFIED".to_string(),
            }),
        )
            .into_response();
    };
    let agent_id = owner.user_id();

    // Extract the action from request path/method
    let action = extract_action(&request);
    let resource = extract_resource(&request);

    // Evaluate governance policy
    if !state.engine.is_allowed(agent_id, &action, &resource).await {
        warn!(
            agent_id = %agent_id,
            action = %action,
            resource = %resource,
            "Governance policy denied request"
        );

        return (
            StatusCode::FORBIDDEN,
            Json(GovernanceDenied {
                error: format!(
                    "Governance policy denied: agent '{agent_id}' cannot '{action}' on '{resource}'"
                ),
                code: "GOVERNANCE_DENIED".to_string(),
            }),
        )
            .into_response();
    }

    next.run(request).await
}

/// Extract an action name from the HTTP method and path.
fn extract_action(request: &Request<Body>) -> String {
    let method = request.method().as_str();
    let path = request.uri().path();

    // Map known paths to semantic actions
    if path.contains("/collaborate") {
        return "collaborate".to_string();
    }
    if path.contains("/message") {
        return "send_message".to_string();
    }
    if path.contains("/actors") && method == "POST" {
        return "spawn_agent".to_string();
    }
    if path.contains("/runs") && method == "POST" {
        return "execute_tool".to_string();
    }
    if path.starts_with("/api/tools/") && path.ends_with("/execute") && method == "POST" {
        return "execute_tool".to_string();
    }

    // Fallback to method-based action
    format!("http_{}", method.to_lowercase())
}

fn is_direct_tool_execution(request: &Request<Body>) -> bool {
    let path = request.uri().path();
    request.method() == "POST" && path.starts_with("/api/tools/") && path.ends_with("/execute")
}

/// Extract a resource name from the request path.
fn extract_resource(request: &Request<Body>) -> String {
    let path = request.uri().path();

    // Extract the last meaningful path segment as the resource
    path.split('/')
        .filter(|s| !s.is_empty())
        .last()
        .unwrap_or("unknown")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    // Only the cedar-gated tests below build a router and drive it; without
    // that feature these would be unused-import warnings.
    #[cfg(feature = "cedar-governance")]
    use axum::{Router, routing::post};
    #[cfg(feature = "cedar-governance")]
    use tower::ServiceExt;

    #[test]
    fn initializing_governance_does_not_bypass_http_cedar() {
        let (_, gate, _) =
            crate::uar::governance::runtime_control::governance_runtime_handles("localhost");
        assert!(gate.effective_enabled());
    }

    /// Direct execution never uses the unverified agent header as its Cedar
    /// principal. The authenticated handler and RunManager exact-invocation
    /// boundary perform the policy decision instead.
    #[cfg(feature = "cedar-governance")]
    #[tokio::test]
    async fn governance_off_bypasses_direct_tool_http_cedar() {
        let (mutation, gate, _) =
            crate::uar::governance::runtime_control::governance_runtime_handles("localhost");
        mutation.record_installed_authentication(false);
        mutation.declare_ingress("primary-http").expect("declare");
        let proof = mutation
            .register_bound_ingress("primary-http", "127.0.0.1:1906".parse().expect("address"))
            .expect("register");
        mutation
            .seal_ingress_inventory(&[proof])
            .expect("seal inventory");
        let plan = mutation.preference_plan(Some(false)).expect("preference");
        mutation.finalize_preference(&plan).expect("finalize Off");

        let state = GovernanceMiddlewareState::new(Arc::new(GovernanceEngine::new()), gate);
        let app = Router::new()
            .route(
                "/api/tools/web_search/execute",
                post(|| async { StatusCode::OK }),
            )
            .layer(axum::middleware::from_fn_with_state(
                state,
                governance_layer,
            ));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/tools/web_search/execute")
                    .header("X-Agent-Id", "local-agent")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    /// Governance being enabled does not make the unverified agent header an
    /// authority. Direct execution still defers to authenticated RunManager
    /// admission, which evaluates Cedar against the bound subject.
    #[cfg(feature = "cedar-governance")]
    #[tokio::test]
    async fn direct_tool_http_defers_cedar_to_authenticated_run_admission() {
        let (mutation, gate, _) =
            crate::uar::governance::runtime_control::governance_runtime_handles("localhost");
        mutation.record_installed_authentication(false);
        mutation.declare_ingress("primary-http").expect("declare");
        let proof = mutation
            .register_bound_ingress("primary-http", "127.0.0.1:1906".parse().expect("address"))
            .expect("register");
        mutation
            .seal_ingress_inventory(&[proof])
            .expect("seal inventory");
        let plan = mutation.preference_plan(Some(true)).expect("preference");
        mutation.finalize_preference(&plan).expect("finalize On");

        let state = GovernanceMiddlewareState::new(Arc::new(GovernanceEngine::new()), gate);
        let app = Router::new()
            .route(
                "/api/tools/web_search/execute",
                post(|| async { StatusCode::OK }),
            )
            .layer(axum::middleware::from_fn_with_state(
                state,
                governance_layer,
            ));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/tools/web_search/execute")
                    .header("X-Agent-Id", "local-agent")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    /// Exercises the real Cedar decision path, so it needs the engine that
    /// makes one. Without `cedar-governance`, `GovernanceEngine` resolves to
    /// the `engine_disabled.rs` stub whose `is_allowed` unconditionally
    /// returns `true` -- every request is permitted, and an assertion about
    /// denial cannot hold. Gated rather than relaxed: weakening the assertion
    /// to match the stub would delete the security property under test.
    #[cfg(feature = "cedar-governance")]
    #[tokio::test]
    async fn governance_off_preserves_cedar_for_non_tool_actions() {
        let (mutation, gate, _) =
            crate::uar::governance::runtime_control::governance_runtime_handles("localhost");
        mutation.record_installed_authentication(false);
        mutation.declare_ingress("primary-http").expect("declare");
        let proof = mutation
            .register_bound_ingress("primary-http", "127.0.0.1:1906".parse().expect("address"))
            .expect("register");
        mutation
            .seal_ingress_inventory(&[proof])
            .expect("seal inventory");
        let plan = mutation.preference_plan(Some(false)).expect("preference");
        mutation.finalize_preference(&plan).expect("finalize Off");

        let state = GovernanceMiddlewareState::new(Arc::new(GovernanceEngine::new()), gate);
        let app = Router::new()
            .route("/api/actors", post(|| async { StatusCode::OK }))
            .layer(axum::middleware::from_fn_with_state(
                state,
                governance_layer,
            ))
            .layer(axum::Extension(
                crate::uar::security::claims::UserContext {
                    host_authority: None, authority: None,
                    user_id: "authenticated-user".to_string(),
                    tenant_id: None,
                    claims: crate::uar::security::claims::UserClaims {
                        sub: "authenticated-user".to_string(),
                        name: None,
                        roles: None,
                        tenant_id: None,
                        uar_instance_id: None,
                        exp: usize::MAX,
                    },
                },
            ));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/actors")
                    .header("X-Agent-Id", "local-agent")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn direct_tool_execution_uses_governed_action() {
        let request = Request::builder()
            .method("POST")
            .uri("/api/tools/web%3A%3Asearch/execute")
            .body(Body::empty())
            .expect("request");
        assert_eq!(extract_action(&request), "execute_tool");
    }
}
