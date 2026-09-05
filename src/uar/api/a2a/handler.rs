//! A2A JSON-RPC adapter over the shared persisted-thread service.
//! Compiler and named-agent endpoints preserve the existing JSON-RPC wire types.

use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{Path, State},
    response::IntoResponse,
};

use super::{
    agent_card::build_agent_card,
    thread_service::{A2AThreadService, TaskError},
    types::{
        JsonRpcRequest, JsonRpcResponse, MessageSendParams, TaskCancelParams, TaskGetParams,
        rpc_error,
    },
};
use crate::{
    config::SecurityConfig,
    uar::{runtime::actor::messages::ActorOwner, security::claims::UserContext},
};

/// Shared task execution adapter for both JSON-RPC and gRPC.
#[derive(Debug, Clone)]
pub struct A2AState {
    pub threads: Arc<A2AThreadService>,
    pub security: SecurityConfig,
    /// Public base URL used by the existing compiler AgentCard.
    pub base_url: String,
}

/// Existing compiler endpoint, now executing its registered agent artifact.
pub async fn handle_rpc(
    State(state): State<Arc<A2AState>>,
    user_context: Option<Extension<UserContext>>,
    Json(req): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    Json(dispatch(&state, "compiler-agent", user_context, req).await)
}

/// Named artifacts use the same task/owner checks as the compiler endpoint.
pub async fn handle_agent_rpc(
    State(state): State<Arc<A2AState>>,
    Path(agent_id): Path<String>,
    user_context: Option<Extension<UserContext>>,
    Json(req): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    Json(dispatch(&state, &agent_id, user_context, req).await)
}

async fn dispatch(
    state: &A2AState,
    agent_id: &str,
    user_context: Option<Extension<UserContext>>,
    req: JsonRpcRequest,
) -> JsonRpcResponse {
    if req.jsonrpc != "2.0" {
        return JsonRpcResponse::err(
            req.id,
            rpc_error::INVALID_REQUEST,
            "jsonrpc must be \"2.0\"",
        );
    }
    let user = user_context.map(|Extension(context)| context);
    if state.security.jwt_required && user.as_ref().is_none_or(|user| user.tenant_id.is_none()) {
        return JsonRpcResponse::err(
            req.id,
            rpc_error::INVALID_REQUEST,
            "verified tenant claim required",
        );
    }
    let authenticated_instance_id = user
        .as_ref()
        .and_then(|user| user.claims.uar_instance_id.as_deref());
    let owner = match user
        .as_ref()
        .and_then(|user| ActorOwner::from_verified_context(user).ok())
    {
        Some(owner) => owner,
        None => {
            return JsonRpcResponse::err(
                req.id,
                rpc_error::INVALID_REQUEST,
                "verified user context required",
            );
        }
    };
    let result = match req.method.as_str() {
        "message/send" => match parse_params::<MessageSendParams>(req.params) {
            Ok(params) => {
                state
                    .threads
                    .send(&owner, authenticated_instance_id, agent_id, params)
                    .await
            }
            Err(error) => return JsonRpcResponse::err(req.id, rpc_error::INVALID_PARAMS, error),
        },
        "tasks/get" => match parse_params::<TaskGetParams>(req.params) {
            Ok(params) => state.threads.get(&owner, agent_id, &params.id).await,
            Err(error) => return JsonRpcResponse::err(req.id, rpc_error::INVALID_PARAMS, error),
        },
        "tasks/cancel" => match parse_params::<TaskCancelParams>(req.params) {
            Ok(params) => state.threads.cancel(&owner, agent_id, &params.id).await,
            Err(error) => return JsonRpcResponse::err(req.id, rpc_error::INVALID_PARAMS, error),
        },
        _ => return JsonRpcResponse::err(req.id, rpc_error::METHOD_NOT_FOUND, "method not found"),
    };
    match result {
        Ok(task) => JsonRpcResponse::ok(req.id, task),
        Err(error) => {
            let code = match &error {
                TaskError::NotFound => rpc_error::TASK_NOT_FOUND,
                TaskError::Conflict => rpc_error::TASK_NOT_CANCELABLE,
                TaskError::Invalid(_) => rpc_error::INVALID_PARAMS,
                TaskError::Host(cause) => {
                    tracing::error!(%cause, "A2A thread host failed");
                    rpc_error::INTERNAL_ERROR
                }
            };
            JsonRpcResponse::err(req.id, code, error.to_string())
        }
    }
}

/// Existing public compiler AgentCard; execution requires verified identity.
pub async fn handle_agent_card(State(state): State<Arc<A2AState>>) -> impl IntoResponse {
    Json(build_agent_card(&state.base_url))
}

fn parse_params<T: serde::de::DeserializeOwned>(
    params: Option<serde_json::Value>,
) -> Result<T, String> {
    serde_json::from_value(params.unwrap_or(serde_json::Value::Null))
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use axum::{
        Router,
        body::{Body, to_bytes},
        http::{Request, StatusCode},
        routing::post,
    };
    use tokio::sync::RwLock;
    use tower::ServiceExt;

    use super::*;
    use crate::{
        config::LlmConfig,
        llm::mock_driver::MockLlmDriver,
        mcp::registry::McpRegistry,
        session::SessionStore,
        uar::{
            persistence::{PersistenceLayer, providers::surreal::SurrealDbProvider},
            rag::embeddings::{EmbeddingBackend, UnavailableEmbeddingBackend},
            runtime::{
                actor::system::ActorCollaboration, manager::RunManager, matching::VectorMatcher,
                skills::SkillRegistry,
            },
            security::claims::{TenantId, UserClaims},
        },
    };

    fn security(jwt_required: bool) -> SecurityConfig {
        SecurityConfig {
            jwt_required,
            jwt_secret: "tenant-test-secret".to_owned().into(),
            jwks_url: None,
            jwt_issuer: None,
            jwt_audience: None,
            jwt_validate_nbf: true,
            settings_mutation_auth_required: true,
            settings_admin_key: Some("test-admin-key".to_owned().into()),
        }
    }

    fn context(tenant: &str) -> UserContext {
        UserContext {
            user_id: "verified-user".to_owned(),
            tenant_id: Some(TenantId::for_test(tenant)),
            claims: UserClaims {
                sub: "verified-user".to_owned(),
                name: None,
                roles: None,
                tenant_id: Some(tenant.to_owned()),
                uar_instance_id: None,
                exp: usize::MAX,
            },
        }
    }

    async fn state(jwt_required: bool) -> (Arc<A2AState>, tempfile::TempDir) {
        let database = tempfile::tempdir().expect("A2A test database directory must be created");
        let endpoint = format!("surrealkv://{}", database.path().join("a2a.db").display());
        let persistence: Arc<dyn PersistenceLayer> = Arc::new(
            SurrealDbProvider::new(&endpoint, None, None, Some("a2a-test"), Some("a2a-test"))
                .await
                .expect("A2A test database must open"),
        );
        let embeddings: Arc<dyn EmbeddingBackend> = Arc::new(UnavailableEmbeddingBackend::new(
            384,
            "embeddings are not exercised by A2A transport tests",
        ));
        let manager = Arc::new(
            RunManager::new(
                LlmConfig::default(),
                Arc::new(McpRegistry::new_empty()),
                SessionStore::new(),
                Arc::new(RwLock::new(SkillRegistry::default())),
                Arc::new(VectorMatcher::new(embeddings, 0.75)),
                Some(persistence),
            )
            .await
            .with_llm_driver(Arc::new(MockLlmDriver::echo())),
        );
        let state = Arc::new(A2AState {
            threads: Arc::new(A2AThreadService::new(Arc::new(ActorCollaboration::new(
                manager,
            )))),
            security: security(jwt_required),
            base_url: "http://127.0.0.1:3928".to_owned(),
        });
        (state, database)
    }

    #[tokio::test]
    async fn body_query_and_header_tenant_values_cannot_override_verified_tenant() {
        let (shared_state, _database) = state(true).await;
        let app = Router::new()
            .route("/", post(handle_rpc))
            .with_state(Arc::clone(&shared_state))
            .layer(Extension(context("tenant-a")));
        let request = Request::builder()
            .method("POST")
            .uri("/?tenant_id=tenant-b")
            .header("content-type", "application/json")
            .header("x-uar-tenant-id", "tenant-b")
            .body(Body::from(
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": "tenant-spoof",
                    "method": "message/send",
                    "tenant_id": "tenant-b",
                    "params": {
                        "message": {
                            "role": "user",
                            "parts": [{"type": "text", "text": "continue verified tenant"}],
                            "metadata": {"tenant_id": "tenant-b"}
                        },
                        "context_id": "shared-context",
                        "metadata": {"tenant_id": "tenant-b"},
                        "tenant_id": "tenant-b"
                    }
                })
                .to_string(),
            ))
            .expect("tenant spoof request must build");

        let response = app
            .oneshot(request)
            .await
            .expect("tenant spoof request must complete");
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("tenant spoof response body must read");
        let response: serde_json::Value =
            serde_json::from_slice(&body).expect("tenant spoof response must be JSON");
        let task_id = response["result"]["id"]
            .as_str()
            .expect("tenant A task id must be returned")
            .to_owned();

        let get_request = |id: &str| {
            Request::builder()
                .method("POST")
                .uri("/?tenant_id=tenant-b")
                .header("content-type", "application/json")
                .header("x-uar-tenant-id", "tenant-b")
                .body(Body::from(
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": "tenant-spoof-get",
                        "method": "tasks/get",
                        "tenant_id": "tenant-b",
                        "params": {"id": id, "tenant_id": "tenant-b"}
                    })
                    .to_string(),
                ))
                .expect("tenant task-get request must build")
        };

        let tenant_b = Router::new()
            .route("/", post(handle_rpc))
            .with_state(Arc::clone(&shared_state))
            .layer(Extension(context("tenant-b")))
            .oneshot(get_request(&task_id))
            .await
            .expect("tenant B task get must complete");
        let tenant_b: serde_json::Value = serde_json::from_slice(
            &to_bytes(tenant_b.into_body(), usize::MAX)
                .await
                .expect("tenant B response body must read"),
        )
        .expect("tenant B response must be JSON");
        assert_eq!(tenant_b["error"]["code"], rpc_error::TASK_NOT_FOUND);

        let tenant_a = Router::new()
            .route("/", post(handle_rpc))
            .with_state(shared_state)
            .layer(Extension(context("tenant-a")))
            .oneshot(get_request(&task_id))
            .await
            .expect("tenant A task get must complete");
        let tenant_a: serde_json::Value = serde_json::from_slice(
            &to_bytes(tenant_a.into_body(), usize::MAX)
                .await
                .expect("tenant A response body must read"),
        )
        .expect("tenant A response must be JSON");
        assert_eq!(tenant_a["result"]["id"], task_id);
    }

    #[tokio::test]
    async fn required_jwt_without_verified_tenant_is_rejected() {
        let (state, _database) = state(true).await;
        let app = Router::new().route("/", post(handle_rpc)).with_state(state);
        let request = Request::builder()
            .method("POST")
            .uri("/")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": "missing-tenant",
                    "method": "tasks/get",
                    "params": {"id": "unknown"}
                })
                .to_string(),
            ))
            .expect("missing tenant request must build");

        let response = app
            .oneshot(request)
            .await
            .expect("missing tenant request must complete");
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("missing tenant response body must read");
        let response: serde_json::Value =
            serde_json::from_slice(&body).expect("missing tenant response must be JSON");

        assert_eq!(response["error"]["code"], rpc_error::INVALID_REQUEST);
        assert_eq!(
            response["error"]["message"],
            "verified tenant claim required"
        );
    }
}
