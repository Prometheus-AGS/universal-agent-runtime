//! A2A v0.3 gRPC transport — tonic `AgentService` implementation.
//!
//! Delegates each RPC method to the same [`A2AState`] / [`TaskStore`] /
//! [`CompilerService`] used by the JSON-RPC handler, so both transports
//! share identical server-side behaviour.

use std::pin::Pin;
use std::sync::Arc;

use futures::Stream;
use tonic::{Request, Response, Status};

use super::handler::A2AState;
use super::types::{Message as A2aMessage, Part as A2aPart, Role, Task, TaskState};
use crate::uar::security::{
    claims::TenantId,
    verifier::{VerificationError, verify_token},
};

// ── Generated code from proto/a2a.proto ─────────────────────────────────────
pub mod pb {
    tonic::include_proto!("a2a");
}

use pb::agent_service_server::{AgentService, AgentServiceServer};
use pb::{
    Artifact as PbArtifact, CancelTaskRequest, GetTaskRequest, Message as PbMessage,
    Part as PbPart, SendMessageRequest, TaskEvent as PbTaskEvent, TaskResponse as PbTaskResponse,
};

// ── Service struct ──────────────────────────────────────────────────────────

/// gRPC implementation of the A2A `AgentService`.
///
/// Holds the same [`A2AState`] that the JSON-RPC endpoint uses.
#[derive(Debug, Clone)]
pub struct GrpcAgentService {
    state: Arc<A2AState>,
}

impl GrpcAgentService {
    pub fn new(state: Arc<A2AState>) -> Self {
        Self { state }
    }

    /// Create a [`tonic::transport::server::Router`]-compatible service.
    pub fn into_server(self) -> AgentServiceServer<Self> {
        AgentServiceServer::new(self)
    }
}

// ── Trait implementation ────────────────────────────────────────────────────

#[tonic::async_trait]
impl AgentService for GrpcAgentService {
    /// `MessageSend` — equivalent to JSON-RPC `message/send`.
    async fn message_send(
        &self,
        request: Request<SendMessageRequest>,
    ) -> Result<Response<PbTaskResponse>, Status> {
        let tenant_id = self.tenant_for_request(&request).await?;
        let req = request.into_inner();
        let pb_msg = req
            .message
            .ok_or_else(|| Status::invalid_argument("message is required"))?;

        let a2a_msg = pb_message_to_a2a(&pb_msg);
        let user_text = extract_text_from_a2a(&a2a_msg);

        // Try to continue an existing task if task_id is provided.
        if !req.task_id.is_empty() {
            if let Some(existing) = self
                .state
                .task_store
                .get(tenant_id.as_ref(), &req.task_id)
                .await
            {
                return self
                    .continue_task(tenant_id.as_ref(), existing, a2a_msg, &user_text)
                    .await
                    .map(|t| Response::new(task_to_pb(&t)));
            }
        }

        // New task — create a compiler session.
        let session = self.state.compiler_service.create_session().await;
        let session_id = session.id.clone();

        let task = self
            .state
            .task_store
            .create(tenant_id.as_ref(), None, &session_id, a2a_msg)
            .await;

        let agent_reply = A2aMessage::agent_text(
            "Welcome! I'm the UAR Compiler Agent. I can help you compile a UAR-AGENT-MD \
             specification into a signed agent descriptor.\n\n\
             You can:\n\
             • **Paste a complete spec** — I'll compile it immediately.\n\
             • **Describe what you want** — I'll guide you through building one step by step.\n\n\
             What would you like to do?",
        );

        self.state
            .task_store
            .append_message(tenant_id.as_ref(), &task.id, agent_reply.clone())
            .await;

        let mut result_task = self
            .state
            .task_store
            .get(tenant_id.as_ref(), &task.id)
            .await
            .unwrap_or(task);
        result_task.status.message = Some(agent_reply);

        Ok(Response::new(task_to_pb(&result_task)))
    }

    /// `TaskGet` — equivalent to JSON-RPC `tasks/get`.
    async fn task_get(
        &self,
        request: Request<GetTaskRequest>,
    ) -> Result<Response<PbTaskResponse>, Status> {
        let tenant_id = self.tenant_for_request(&request).await?;
        let req = request.into_inner();
        let task = self
            .state
            .task_store
            .get(tenant_id.as_ref(), &req.task_id)
            .await
            .ok_or_else(|| Status::not_found("task not found"))?;

        Ok(Response::new(task_to_pb(&task)))
    }

    /// `TaskCancel` — equivalent to JSON-RPC `tasks/cancel`.
    async fn task_cancel(
        &self,
        request: Request<CancelTaskRequest>,
    ) -> Result<Response<PbTaskResponse>, Status> {
        let tenant_id = self.tenant_for_request(&request).await?;
        let req = request.into_inner();

        let cancelled = self
            .state
            .task_store
            .cancel(tenant_id.as_ref(), &req.task_id)
            .await;
        if !cancelled {
            return Err(Status::failed_precondition(
                "task not found or already in a terminal state",
            ));
        }

        // Also cancel the underlying compiler session (best-effort).
        if let Some(task) = self
            .state
            .task_store
            .get(tenant_id.as_ref(), &req.task_id)
            .await
        {
            if let Some(ctx_id) = &task.context_id {
                self.state.compiler_service.cancel_session(ctx_id).await;
            }
        }

        let task = self
            .state
            .task_store
            .get(tenant_id.as_ref(), &req.task_id)
            .await
            .ok_or_else(|| Status::not_found("task not found after cancel"))?;

        Ok(Response::new(task_to_pb(&task)))
    }

    type MessageStreamStream =
        Pin<Box<dyn Stream<Item = Result<PbTaskEvent, Status>> + Send + 'static>>;

    /// `MessageStream` — streaming variant of `message/send`.
    ///
    /// Currently performs the same work as `message_send` and emits a single
    /// `status_update` event followed by stream completion. This can be
    /// extended to emit incremental `message` / `artifact` events once the
    /// compiler pipeline supports streaming output.
    async fn message_stream(
        &self,
        request: Request<SendMessageRequest>,
    ) -> Result<Response<Self::MessageStreamStream>, Status> {
        // Reuse message_send logic.
        let resp = self.message_send(request).await?;
        let task_response = resp.into_inner();

        let event = PbTaskEvent {
            task_id: task_response.task_id.clone(),
            event_type: "status_update".to_string(),
            state: Some(task_response),
        };

        let stream = futures::stream::once(async move { Ok(event) });
        Ok(Response::new(Box::pin(stream)))
    }
}

// ── Private helpers (task continuation) ─────────────────────────────────────

impl GrpcAgentService {
    async fn tenant_for_request<T>(
        &self,
        request: &Request<T>,
    ) -> Result<Option<TenantId>, Status> {
        let authorization = request
            .metadata()
            .get("authorization")
            .and_then(|value| value.to_str().ok());

        let Some(token) = authorization.and_then(|value| value.strip_prefix("Bearer ")) else {
            return if self.state.security.jwt_required {
                Err(Status::unauthenticated("verified tenant claim required"))
            } else {
                Ok(None)
            };
        };

        match verify_token(&self.state.security, token).await {
            Ok(principal) if principal.tenant_id.is_some() => Ok(principal.tenant_id),
            Ok(_) if self.state.security.jwt_required => {
                Err(Status::unauthenticated("verified tenant claim required"))
            }
            Ok(_) => Ok(None),
            Err(VerificationError::ProviderConflict) => {
                Err(Status::internal("JWT provider conflict"))
            }
            Err(_) if self.state.security.jwt_required => {
                Err(Status::unauthenticated("token verification failed"))
            }
            Err(_) => Ok(None),
        }
    }

    async fn continue_task(
        &self,
        tenant_id: Option<&TenantId>,
        task: Task,
        message: A2aMessage,
        user_text: &str,
    ) -> Result<Task, Status> {
        let task_id = task.id.clone();

        self.state
            .task_store
            .append_message(tenant_id, &task_id, message)
            .await;

        // Attempt single-shot compilation if input looks like a spec.
        let trimmed = user_text.trim();
        if trimmed.starts_with("# Agent:") || trimmed.contains("## Metadata") {
            if let Ok(output) = self.state.compiler_service.compile_content(trimmed).await {
                let descriptor = serde_json::to_value(&output).unwrap_or(serde_json::Value::Null);
                self.state
                    .task_store
                    .complete_with_descriptor(tenant_id, &task_id, descriptor)
                    .await;

                return self
                    .state
                    .task_store
                    .get(tenant_id, &task_id)
                    .await
                    .ok_or_else(|| Status::internal("task vanished"));
            }
        }

        // Conversational reply — ask for more.
        let snippet: String = user_text.chars().take(120).collect();
        let agent_reply = A2aMessage::agent_text(format!(
            "Got it. I've noted: \"{snippet}\"\n\n\
             To compile your agent, I need a complete UAR-AGENT-MD specification covering all 15 sections. \
             You can paste the full spec at any time, or continue describing your agent and I'll help fill in the details.",
        ));

        self.state
            .task_store
            .append_message(tenant_id, &task_id, agent_reply.clone())
            .await;

        let mut result_task = self
            .state
            .task_store
            .get(tenant_id, &task_id)
            .await
            .unwrap_or(task);
        result_task.status.state = TaskState::InputRequired;
        result_task.status.message = Some(agent_reply);

        Ok(result_task)
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
        status: task_state_to_str(&task.status.state).to_string(),
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

fn extract_text_from_a2a(msg: &A2aMessage) -> String {
    msg.parts
        .iter()
        .filter_map(|p| match p {
            A2aPart::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use jsonwebtoken::{EncodingKey, Header};

    use super::*;
    use crate::{
        config::SecurityConfig,
        uar::{
            api::a2a::TaskStore,
            compiler::service::CompilerService,
            security::{claims::UserClaims, jwt},
        },
    };

    fn service() -> GrpcAgentService {
        GrpcAgentService::new(Arc::new(A2AState {
            compiler_service: Arc::new(CompilerService::in_memory()),
            task_store: TaskStore::new(),
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
        }))
    }

    fn token(subject: &str, tenant: Option<&str>) -> String {
        jwt::encode(
            &Header::default(),
            &UserClaims {
                sub: subject.to_owned(),
                name: None,
                roles: None,
                tenant_id: tenant.map(str::to_owned),
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
        let service = service();
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
        assert_eq!(cross_cancel.code(), tonic::Code::FailedPrecondition);

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
        let service = service();
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
