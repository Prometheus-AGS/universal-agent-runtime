//! A2A gRPC transport over the same persisted-thread adapter as JSON-RPC.

use std::pin::Pin;
use std::sync::Arc;

use futures::Stream;
use tonic::{Request, Response, Status};

use super::handler::A2AState;
use super::thread_service::TaskError;
use super::types::{
    Message as A2aMessage, MessageSendParams, Part as A2aPart, Role, Task, TaskState,
};
use crate::uar::{
    runtime::actor::messages::ActorOwner,
    security::{
        claims::UserContext,
        verifier::{VerificationError, verify_token},
    },
};

pub mod pb {
    tonic::include_proto!("a2a");
}

use pb::agent_service_server::{AgentService, AgentServiceServer};
use pb::{
    Artifact as PbArtifact, CancelTaskRequest, GetTaskRequest, Message as PbMessage,
    Part as PbPart, SendMessageRequest, TaskEvent as PbTaskEvent, TaskResponse as PbTaskResponse,
};

/// Shared A2A execution. Optional x-uar-agent-id metadata selects an artifact;
/// it never selects or overrides the verified principal.
#[derive(Debug, Clone)]
pub struct GrpcAgentService {
    state: Arc<A2AState>,
}

impl GrpcAgentService {
    pub fn new(state: Arc<A2AState>) -> Self {
        Self { state }
    }

    pub fn into_server(self) -> AgentServiceServer<Self> {
        AgentServiceServer::new(self)
    }

    async fn caller<T>(
        &self,
        request: &Request<T>,
    ) -> Result<(ActorOwner, Option<String>, String), Status> {
        let token = request
            .metadata()
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or_else(|| Status::unauthenticated("verified user context required"))?;
        let principal =
            verify_token(&self.state.security, token)
                .await
                .map_err(|error| match error {
                    VerificationError::ProviderConflict => {
                        Status::internal("JWT provider conflict")
                    }
                    _ => Status::unauthenticated("token verification failed"),
                })?;
        if self.state.security.jwt_required && principal.tenant_id.is_none() {
            return Err(Status::unauthenticated("verified tenant claim required"));
        }
        let user = UserContext {
            user_id: principal.subject,
            tenant_id: principal.tenant_id,
            claims: principal.claims,
        };
        let instance_id = user.claims.uar_instance_id.clone();
        let owner = ActorOwner::from_verified_context(&user)
            .map_err(|_| Status::unauthenticated("verified user context required"))?;
        let agent_id = match request.metadata().get("x-uar-agent-id") {
            Some(value) => value
                .to_str()
                .map_err(|_| Status::invalid_argument("invalid agent id"))?,
            None => "compiler-agent",
        };
        if agent_id.trim().is_empty() {
            return Err(Status::invalid_argument("agent id must not be empty"));
        }
        Ok((owner, instance_id, agent_id.to_owned()))
    }
}

fn task_error(error: TaskError) -> Status {
    match error {
        TaskError::NotFound => Status::not_found("task not found"),
        TaskError::Conflict => {
            Status::failed_precondition("task is active, closed, or cannot be cancelled")
        }
        TaskError::Invalid(message) => Status::invalid_argument(message),
        TaskError::Host(cause) => {
            tracing::error!(%cause, "A2A thread host failed");
            Status::internal("agent task host failed")
        }
    }
}

#[tonic::async_trait]
impl AgentService for GrpcAgentService {
    async fn message_send(
        &self,
        request: Request<SendMessageRequest>,
    ) -> Result<Response<PbTaskResponse>, Status> {
        let (owner, instance_id, agent_id) = self.caller(&request).await?;
        let req = request.into_inner();
        let message = req
            .message
            .ok_or_else(|| Status::invalid_argument("message is required"))?;
        if message.role != "user" {
            return Err(Status::invalid_argument("agent input must have user role"));
        }
        let task = self
            .state
            .threads
            .send(
                &owner,
                instance_id.as_deref(),
                &agent_id,
                MessageSendParams {
                    message: pb_message_to_a2a(&message),
                    task_id: (!req.task_id.is_empty()).then_some(req.task_id),
                    context_id: None,
                    metadata: Default::default(),
                },
            )
            .await
            .map_err(task_error)?;
        Ok(Response::new(task_to_pb(&task)))
    }

    async fn task_get(
        &self,
        request: Request<GetTaskRequest>,
    ) -> Result<Response<PbTaskResponse>, Status> {
        let (owner, _, agent_id) = self.caller(&request).await?;
        let task = self
            .state
            .threads
            .get(&owner, &agent_id, &request.into_inner().task_id)
            .await
            .map_err(task_error)?;
        Ok(Response::new(task_to_pb(&task)))
    }

    async fn task_cancel(
        &self,
        request: Request<CancelTaskRequest>,
    ) -> Result<Response<PbTaskResponse>, Status> {
        let (owner, _, agent_id) = self.caller(&request).await?;
        let task = self
            .state
            .threads
            .cancel(&owner, &agent_id, &request.into_inner().task_id)
            .await
            .map_err(task_error)?;
        Ok(Response::new(task_to_pb(&task)))
    }

    type MessageStreamStream =
        Pin<Box<dyn Stream<Item = Result<PbTaskEvent, Status>> + Send + 'static>>;

    /// Preserve the existing single status-update stream contract.
    async fn message_stream(
        &self,
        request: Request<SendMessageRequest>,
    ) -> Result<Response<Self::MessageStreamStream>, Status> {
        let task_response = self.message_send(request).await?.into_inner();
        let event = PbTaskEvent {
            task_id: task_response.task_id.clone(),
            event_type: "status_update".to_owned(),
            state: Some(task_response),
        };
        Ok(Response::new(Box::pin(futures::stream::once(async move {
            Ok(event)
        }))))
    }
}

// ── Conversion helpers: A2A types ↔ protobuf types ──────────────────────────

fn pb_message_to_a2a(pb: &PbMessage) -> A2aMessage {
    let role = match pb.role.as_str() {
        "agent" => Role::Agent,
        _ => Role::User,
    };

    let parts = pb.parts.iter().map(pb_part_to_a2a).collect();

    A2aMessage {
        role,
        parts,
        metadata: Default::default(),
    }
}

fn pb_part_to_a2a(pb: &PbPart) -> A2aPart {
    match &pb.content {
        Some(pb::part::Content::Text(text)) => A2aPart::Text { text: text.clone() },
        Some(pb::part::Content::Data(bytes)) => A2aPart::Data {
            data: serde_json::Value::String(base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                bytes,
            )),
        },
        None => A2aPart::Text {
            text: String::new(),
        },
    }
}

fn a2a_part_to_pb(part: &A2aPart) -> PbPart {
    match part {
        A2aPart::Text { text } => PbPart {
            content: Some(pb::part::Content::Text(text.clone())),
            content_type: "text/plain".to_string(),
        },
        A2aPart::File {
            bytes, mime_type, ..
        } => {
            let data = bytes
                .as_ref()
                .and_then(|b| {
                    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b).ok()
                })
                .unwrap_or_default();
            PbPart {
                content: Some(pb::part::Content::Data(data)),
                content_type: mime_type
                    .clone()
                    .unwrap_or_else(|| "application/octet-stream".to_string()),
            }
        }
        A2aPart::Data { data } => PbPart {
            content: Some(pb::part::Content::Text(data.to_string())),
            content_type: "application/json".to_string(),
        },
    }
}

fn a2a_message_to_pb(msg: &A2aMessage) -> PbMessage {
    PbMessage {
        role: match msg.role {
            Role::User => "user".to_string(),
            Role::Agent => "agent".to_string(),
        },
        parts: msg.parts.iter().map(a2a_part_to_pb).collect(),
    }
}

fn task_state_to_str(state: &TaskState) -> &'static str {
    match state {
        TaskState::Submitted => "submitted",
        TaskState::Working => "working",
        TaskState::InputRequired => "working",
        TaskState::Completed => "completed",
        TaskState::Canceled => "canceled",
        TaskState::Failed => "failed",
    }
}

fn task_to_pb(task: &Task) -> PbTaskResponse {
    PbTaskResponse {
        task_id: task.id.clone(),
        // This protobuf task has no metadata field. Keep cleanup nonterminal
        // until confirmed instead of dropping HTTP's uncertainty marker and
        // exposing a misleading terminal Failed/Canceled receipt.
        status: if task.cleanup_unconfirmed() {
            "working"
        } else {
            task_state_to_str(&task.status.state)
        }
        .to_string(),
        messages: task.history.iter().map(a2a_message_to_pb).collect(),
        artifacts: task
            .artifacts
            .iter()
            .map(|a| PbArtifact {
                name: a.name.clone().unwrap_or_default(),
                parts: a.parts.iter().map(a2a_part_to_pb).collect(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use jsonwebtoken::{EncodingKey, Header};
    use tokio::sync::RwLock;

    use super::*;
    use crate::{
        config::{LlmConfig, SecurityConfig},
        llm::mock_driver::MockLlmDriver,
        mcp::registry::McpRegistry,
        session::SessionStore,
        uar::{
            api::a2a::thread_service::A2AThreadService,
            persistence::{PersistenceLayer, providers::surreal::SurrealDbProvider},
            rag::embeddings::{EmbeddingBackend, UnavailableEmbeddingBackend},
            runtime::{
                actor::system::ActorCollaboration, manager::RunManager, matching::VectorMatcher,
                skills::SkillRegistry,
            },
            security::{claims::UserClaims, jwt},
        },
    };

    async fn service() -> (GrpcAgentService, tempfile::TempDir) {
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
        let service = GrpcAgentService::new(Arc::new(A2AState {
            threads: Arc::new(A2AThreadService::new(Arc::new(ActorCollaboration::new(
                manager,
            )))),
            security: SecurityConfig {
                jwt_required: true,
                jwt_secret: "tenant-test-secret".to_owned().into(),
                jwks_url: None,
                jwt_issuer: None,
                jwt_audience: None,
                jwt_validate_nbf: true,
                settings_mutation_auth_required: true,
                settings_admin_key: Some("test-admin-key".to_owned().into()),
            },
            base_url: "http://127.0.0.1:3928".to_owned(),
        }));
        (service, database)
    }

    fn token(subject: &str, tenant: Option<&str>) -> String {
        jwt::encode(
            &Header::default(),
            &UserClaims {
                sub: subject.to_owned(),
                name: None,
                roles: None,
                tenant_id: tenant.map(str::to_owned),
                uar_instance_id: None,
                exp: usize::MAX,
            },
            &EncodingKey::from_secret(b"tenant-test-secret"),
        )
        .expect("tenant gRPC token must encode")
    }

    fn authenticated<T>(message: T, token: &str) -> Request<T> {
        let mut request = Request::new(message);
        request.metadata_mut().insert(
            "authorization",
            format!("Bearer {token}")
                .parse()
                .expect("authorization metadata must parse"),
        );
        request.metadata_mut().insert(
            "x-uar-tenant-id",
            "tenant-b".parse().expect("spoofed metadata must parse"),
        );
        request
    }

    #[tokio::test]
    async fn grpc_task_access_is_partitioned_by_verified_tenant() {
        let (service, _database) = service().await;
        let tenant_a = token("user-a", Some("tenant-a"));
        let tenant_b = token("user-b", Some("tenant-b"));

        let created = service
            .message_send(authenticated(
                SendMessageRequest {
                    task_id: String::new(),
                    message: Some(PbMessage {
                        role: "user".to_owned(),
                        parts: vec![PbPart {
                            content: Some(pb::part::Content::Text("tenant task".to_owned())),
                            content_type: "text/plain".to_owned(),
                        }],
                    }),
                },
                &tenant_a,
            ))
            .await
            .expect("tenant A must create a task")
            .into_inner();

        service
            .task_get(authenticated(
                GetTaskRequest {
                    task_id: created.task_id.clone(),
                },
                &tenant_a,
            ))
            .await
            .expect("same-tenant task get must succeed");

        let cross_get = service
            .task_get(authenticated(
                GetTaskRequest {
                    task_id: created.task_id.clone(),
                },
                &tenant_b,
            ))
            .await
            .expect_err("cross-tenant task get must fail");
        assert_eq!(cross_get.code(), tonic::Code::NotFound);

        let cross_cancel = service
            .task_cancel(authenticated(
                CancelTaskRequest {
                    task_id: created.task_id.clone(),
                },
                &tenant_b,
            ))
            .await
            .expect_err("cross-tenant task cancel must fail");
        assert_eq!(cross_cancel.code(), tonic::Code::NotFound);

        let unchanged = service
            .task_get(authenticated(
                GetTaskRequest {
                    task_id: created.task_id,
                },
                &tenant_a,
            ))
            .await
            .expect("tenant A task must remain readable")
            .into_inner();
        assert_eq!(unchanged.status, "working");
    }

    #[tokio::test]
    async fn grpc_required_jwt_without_verified_tenant_is_rejected() {
        let (service, _database) = service().await;
        let token_without_tenant = token("user-without-tenant", None);

        let error = service
            .task_get(authenticated(
                GetTaskRequest {
                    task_id: "unknown".to_owned(),
                },
                &token_without_tenant,
            ))
            .await
            .expect_err("verified token without tenant must be rejected");

        assert_eq!(error.code(), tonic::Code::Unauthenticated);
        assert_eq!(error.message(), "verified tenant claim required");
    }
}
