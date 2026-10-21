//! Official A2A v1.0.0 gRPC binding over the durable A2A task host.
//!
//! This service deliberately coexists with the v0.3 compatibility service in
//! `grpc.rs`. The generated types come from the unmodified normative schema in
//! `proto/a2a_v1.proto`.

use std::{collections::HashMap, pin::Pin, sync::Arc};

use futures::Stream;
use prost_types::{Struct, Timestamp, Value, value::Kind};
use tonic::{Request, Response, Status};

use super::{
    admission::A2aAdmissionError,
    grpc::{VerifiedGrpcCaller, authenticate},
    handler::A2AState,
    thread_service::TaskError,
    types::{self, Artifact as A2aArtifact, Message as A2aMessage, Part as A2aPart, Role, Task},
};

pub mod pb {
    tonic::include_proto!("lf.a2a.v1");
}

use pb::a2a_service_server::{A2aService, A2aServiceServer};
use pb::{
    CancelTaskRequest, DeleteTaskPushNotificationConfigRequest, GetExtendedAgentCardRequest,
    GetTaskPushNotificationConfigRequest, GetTaskRequest, ListTaskPushNotificationConfigsRequest,
    ListTasksRequest, ListTasksResponse, Message, Part, SendMessageRequest, SendMessageResponse,
    StreamResponse, SubscribeToTaskRequest, TaskPushNotificationConfig,
};

const STATE_UNSPECIFIED: i32 = 0;
const STATE_SUBMITTED: i32 = 1;
const STATE_WORKING: i32 = 2;
const STATE_COMPLETED: i32 = 3;
const STATE_FAILED: i32 = 4;
const STATE_CANCELED: i32 = 5;
const STATE_INPUT_REQUIRED: i32 = 6;
const STATE_REJECTED: i32 = 7;
const STATE_AUTH_REQUIRED: i32 = 8;
const ROLE_USER: i32 = 1;
const ROLE_AGENT: i32 = 2;

/// Official A2A v1 gRPC service. It has no dependency on the legacy service's
/// string status projection or request messages.
#[derive(Debug, Clone)]
pub struct GrpcV1Service {
    state: Arc<A2AState>,
    api_keys: Option<Arc<crate::uar::security::api_keys::ApiKeyService>>,
}

impl GrpcV1Service {
    pub fn new(state: Arc<A2AState>) -> Self {
        Self {
            state,
            api_keys: None,
        }
    }

    pub fn with_api_key_service(
        mut self,
        service: Arc<crate::uar::security::api_keys::ApiKeyService>,
    ) -> Self {
        self.api_keys = Some(service);
        self
    }

    pub fn into_server(self) -> A2aServiceServer<Self> {
        A2aServiceServer::new(self)
    }

    async fn caller<T>(&self, request: &Request<T>) -> Result<VerifiedGrpcCaller, Status> {
        authenticate(&self.state, self.api_keys.as_ref(), request).await
    }

    async fn admit(&self, caller: &VerifiedGrpcCaller) -> Result<(), Status> {
        self.state
            .admission
            .admit(&caller.user, &caller.agent_id)
            .await
            .map_err(admission_error)
    }
    async fn send_task(
        &self, request: Request<SendMessageRequest>, force_immediate: bool,
    ) -> Result<(Task, VerifiedGrpcCaller, Option<usize>), Status> {
        let caller = self.caller(&request).await?;
        let request = request.into_inner();
        verify_tenant(&caller, &request.tenant)?;
        self.admit(&caller).await?;
        let configuration = request.configuration.unwrap_or_default();
        if configuration.task_push_notification_config.is_some() {
            return Err(Status::unimplemented("push notifications are not supported"));
        }
        if !configuration.accepted_output_modes.is_empty()
            && !configuration
                .accepted_output_modes
                .iter()
                .any(|mode| matches!(mode.as_str(), "text/plain" | "application/json"))
        {
            return Err(Status::invalid_argument("content type is not supported"));
        }
        let message = request
            .message
            .ok_or_else(|| Status::invalid_argument("message is required"))?;
        let (message_id, mut params) = message_from_pb(message)?;
        params.metadata = metadata_from_struct(request.metadata);
        let mut task = self
            .state
            .threads
            .send_durable(
                &caller.owner,
                &caller.user,
                caller.instance_id.as_deref(),
                &caller.agent_id,
                params,
                &message_id,
                caller.credential_capture.clone(),
            )
            .await
            .map_err(task_error)?;
        if !force_immediate && !configuration.return_immediately && is_active(&task) {
            task = self
                .state
                .threads
                .wait_durable_until_settled(&caller.owner, &caller.agent_id, &task.id, &task)
                .await
                .map_err(task_error)?;
        }
        Ok((task, caller, configuration.history_length.map(|value| value.max(0) as usize)))
    }

    async fn task_stream(
        &self, caller: VerifiedGrpcCaller, task: Task, history_length: Option<usize>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamResponse, Status>> + Send + 'static>>, Status> {
        let mut updates = self.state.threads.subscribe_durable(&caller.owner, &caller.agent_id, &task).await.map_err(task_error)?;
        Ok(Box::pin(async_stream::stream! {
            let mut first = true;
            while let Some(update) = updates.recv().await {
                let task = match update {
                    Ok(task) => task,
                    Err(_) => { yield Err(Status::unavailable("task event stream lost; reconcile exact task before resubscribing")); break; }
                };
                let projected = task_to_pb(&task, history_length, true);
                if first {
                    first = false;
                    yield Ok(StreamResponse { payload: Some(pb::stream_response::Payload::Task(projected)) });
                } else {
                    for artifact in projected.artifacts {
                        yield Ok(StreamResponse { payload: Some(pb::stream_response::Payload::ArtifactUpdate(pb::TaskArtifactUpdateEvent {
                            task_id: task.id.clone(), context_id: task.context_id.clone().unwrap_or_default(),
                            artifact: Some(artifact), append: false, last_chunk: true,
                            metadata: struct_from_metadata(&task.metadata),
                        })) });
                    }
                    yield Ok(StreamResponse { payload: Some(pb::stream_response::Payload::StatusUpdate(pb::TaskStatusUpdateEvent {
                        task_id: task.id.clone(), context_id: task.context_id.clone().unwrap_or_default(),
                        status: projected.status, metadata: struct_from_metadata(&task.metadata),
                    })) });
                }
            }
        }))
    }

}

fn admission_error(error: A2aAdmissionError) -> Status {
    match error {
        A2aAdmissionError::Forbidden => Status::permission_denied("insufficient scope"),
        A2aAdmissionError::Unavailable => Status::unavailable("task admission unavailable"),
    }
}

fn task_error(error: TaskError) -> Status {
    match error {
        TaskError::NotFound => Status::not_found("task not found"),
        TaskError::Conflict => Status::failed_precondition("task cannot perform this operation"),
        TaskError::Invalid(message) => Status::invalid_argument(message),
        TaskError::Host(cause) => {
            tracing::error!(%cause, "A2A v1 gRPC durable task host failed");
            Status::internal("agent task host failed")
        }
    }
}

/// The optional A2A `tenant` path field can only repeat the verified caller's
/// tenant. Agent routing remains the gRPC request metadata route.
fn verify_tenant(caller: &VerifiedGrpcCaller, tenant: &str) -> Result<(), Status> {
    if !tenant.is_empty()
        && caller
            .user
            .tenant_id
            .as_ref()
            .is_none_or(|actual| actual.as_str() != tenant)
    {
        return Err(Status::permission_denied("insufficient scope"));
    }
    Ok(())
}

fn is_active(task: &Task) -> bool {
    matches!(task.status.state, types::TaskState::Submitted | types::TaskState::Working)
}

fn task_state(task: &Task) -> i32 {
    if task
        .metadata
        .get("execution_unconfirmed")
        .is_some_and(|value| value.as_bool() != Some(false))
    {
        return STATE_UNSPECIFIED;
    }
    if task
        .metadata
        .get("uar_auth_required")
        .is_some_and(|value| value.as_bool() == Some(true))
    {
        return STATE_AUTH_REQUIRED;
    }
    if task
        .metadata
        .get("uar_denied")
        .is_some_and(|value| value.as_bool() == Some(true))
        && matches!(
            task.status.state,
            types::TaskState::Completed | types::TaskState::Canceled | types::TaskState::Failed
        )
    {
        return STATE_REJECTED;
    }
    if task.cleanup_unconfirmed() {
        return STATE_WORKING;
    }
    match task.status.state {
        types::TaskState::Submitted => STATE_SUBMITTED,
        types::TaskState::Working => STATE_WORKING,
        types::TaskState::InputRequired => STATE_INPUT_REQUIRED,
        types::TaskState::Completed => STATE_COMPLETED,
        types::TaskState::Canceled => STATE_CANCELED,
        types::TaskState::Failed => STATE_FAILED,
    }
}

fn json_to_value(value: &serde_json::Value) -> Value {
    let kind = match value {
        serde_json::Value::Null => Kind::NullValue(0),
        serde_json::Value::Bool(value) => Kind::BoolValue(*value),
        serde_json::Value::Number(value) => Kind::NumberValue(value.as_f64().unwrap_or_default()),
        serde_json::Value::String(value) => Kind::StringValue(value.clone()),
        serde_json::Value::Array(values) => Kind::ListValue(prost_types::ListValue {
            values: values.iter().map(json_to_value).collect(),
        }),
        serde_json::Value::Object(values) => Kind::StructValue(Struct {
            fields: values
                .iter()
                .map(|(key, value)| (key.clone(), json_to_value(value)))
                .collect(),
        }),
    };
    Value { kind: Some(kind) }
}

fn struct_from_metadata(metadata: &HashMap<String, serde_json::Value>) -> Option<Struct> {
    (!metadata.is_empty()).then(|| Struct {
        fields: metadata
            .iter()
            .map(|(key, value)| (key.clone(), json_to_value(value)))
            .collect(),
    })
}

fn value_to_json(value: &Value) -> serde_json::Value {
    match value.kind.as_ref() {
        Some(Kind::NullValue(_)) | None => serde_json::Value::Null,
        Some(Kind::BoolValue(value)) => serde_json::Value::Bool(*value),
        Some(Kind::NumberValue(value)) => serde_json::json!(value),
        Some(Kind::StringValue(value)) => serde_json::Value::String(value.clone()),
        Some(Kind::ListValue(values)) => {
            serde_json::Value::Array(values.values.iter().map(value_to_json).collect())
        }
        Some(Kind::StructValue(values)) => serde_json::Value::Object(
            values
                .fields
                .iter()
                .map(|(key, value)| (key.clone(), value_to_json(value)))
                .collect(),
        ),
    }
}

fn metadata_from_struct(metadata: Option<Struct>) -> HashMap<String, serde_json::Value> {
    metadata
        .map(|metadata| {
            metadata
                .fields
                .iter()
                .map(|(key, value)| (key.clone(), value_to_json(value)))
                .collect()
        })
        .unwrap_or_default()
}

fn timestamp(timestamp: Option<&str>) -> Option<Timestamp> {
    timestamp.and_then(|timestamp| {
        chrono::DateTime::parse_from_rfc3339(timestamp)
            .ok()
            .map(|timestamp| Timestamp {
                seconds: timestamp.timestamp(),
                nanos: timestamp.timestamp_subsec_nanos() as i32,
            })
    })
}

fn message_to_pb(
    message: &A2aMessage,
    fallback_id: String,
    task_id: Option<String>,
    context_id: Option<String>,
) -> Message {
    let message_id = message
        .metadata
        .get("uar_a2a_message_id")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .unwrap_or(fallback_id);
    let mut metadata = message.metadata.clone();
    metadata.remove("uar_a2a_message_id");
    Message {
        message_id,
        context_id: context_id.unwrap_or_default(),
        task_id: task_id.unwrap_or_default(),
        role: match message.role {
            Role::User => ROLE_USER,
            Role::Agent => ROLE_AGENT,
        },
        parts: message.parts.iter().map(part_to_pb).collect(),
        metadata: struct_from_metadata(&metadata),
        extensions: Vec::new(),
        reference_task_ids: Vec::new(),
    }
}

fn part_to_pb(part: &A2aPart) -> Part {
    let (content, filename, media_type) = match part {
        A2aPart::Text { text } => (
            Some(pb::part::Content::Text(text.clone())),
            String::new(),
            "text/plain".to_owned(),
        ),
        A2aPart::File {
            name,
            mime_type,
            uri,
            bytes,
        } => {
            let content = if let Some(uri) = uri {
                Some(pb::part::Content::Url(uri.clone()))
            } else {
                bytes.as_ref().and_then(|bytes| {
                    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, bytes)
                        .ok()
                        .map(pb::part::Content::Raw)
                })
            };
            (
                content,
                name.clone().unwrap_or_default(),
                mime_type
                    .clone()
                    .unwrap_or_else(|| "application/octet-stream".to_owned()),
            )
        }
        A2aPart::Data { data } => (Some(pb::part::Content::Data(json_to_value(data))), String::new(), "application/json".to_owned()),
    };
    Part {
        content,
        metadata: None,
        filename,
        media_type,
    }
}

fn artifact_to_pb(artifact: &A2aArtifact) -> pb::Artifact {
    pb::Artifact {
        artifact_id: artifact.artifact_id.clone(),
        name: artifact.name.clone().unwrap_or_default(),
        description: artifact.description.clone().unwrap_or_default(),
        parts: artifact.parts.iter().map(part_to_pb).collect(),
        metadata: struct_from_metadata(&artifact.metadata),
        extensions: Vec::new(),
    }
}

fn task_to_pb(task: &Task, history_length: Option<usize>, include_artifacts: bool) -> pb::Task {
    let history_start = task
        .history
        .len()
        .saturating_sub(history_length.unwrap_or(usize::MAX));
    let history = task
        .history
        .iter()
        .enumerate()
        .skip(history_start)
        .map(|(index, message)| {
            message_to_pb(
                message,
                format!("{}:{index}", task.id),
                Some(task.id.clone()),
                task.context_id.clone(),
            )
        })
        .collect();
    let mut artifacts = include_artifacts
        .then(|| task.artifacts.iter().map(artifact_to_pb).collect::<Vec<_>>())
        .unwrap_or_default();
    if include_artifacts
        && task.status.state == types::TaskState::Completed
        && let Some(output) = &task.status.message
    {
        artifacts.push(pb::Artifact {
            artifact_id: format!("{}:output", task.id),
            name: String::new(),
            description: String::new(),
            parts: output.parts.iter().map(part_to_pb).collect(),
            metadata: None,
            extensions: Vec::new(),
        });
    }
    pb::Task {
        id: task.id.clone(),
        context_id: task.context_id.clone().unwrap_or_default(),
        status: Some(pb::TaskStatus {
            state: task_state(task),
            message: task.status.message.as_ref().map(|message| {
                message_to_pb(
                    message,
                    format!("{}:status", task.id),
                    Some(task.id.clone()),
                    task.context_id.clone(),
                )
            }),
            timestamp: timestamp(task.status.timestamp.as_deref()),
        }),
        artifacts,
        history,
        metadata: struct_from_metadata(&task.metadata),
    }
}

fn message_from_pb(message: Message) -> Result<(String, types::MessageSendParams), Status> {
    if message.role != ROLE_USER {
        return Err(Status::invalid_argument("agent input must have ROLE_USER"));
    }
    if message.message_id.trim().is_empty() {
        return Err(Status::invalid_argument("message_id is required"));
    }
    let parts = message
        .parts
        .into_iter()
        .map(part_from_pb)
        .collect::<Result<Vec<_>, _>>()?;
    if parts.is_empty() {
        return Err(Status::invalid_argument("message requires at least one text part"));
    }
    let message_id = message.message_id;
    let mut metadata = metadata_from_struct(message.metadata);
    metadata.insert(
        "uar_a2a_message_id".to_owned(),
        serde_json::Value::String(message_id.clone()),
    );
    Ok((
        message_id,
        types::MessageSendParams {
            message: A2aMessage {
                role: Role::User,
                parts,
                metadata,
            },
            task_id: (!message.task_id.is_empty()).then_some(message.task_id),
            context_id: (!message.context_id.is_empty()).then_some(message.context_id),
            metadata: HashMap::new(),
        },
    ))
}

fn part_from_pb(part: Part) -> Result<A2aPart, Status> {
    match part.content {
        Some(pb::part::Content::Text(text)) => {
            if !part.media_type.is_empty() && part.media_type != "text/plain" {
                return Err(Status::invalid_argument("this agent accepts text/plain parts"));
            }
            Ok(A2aPart::Text { text })
        }
        _ => Err(Status::invalid_argument("this agent accepts text/plain parts")),
    }
}

#[tonic::async_trait]
impl A2aService for GrpcV1Service {
    async fn send_message(
        &self,
        request: Request<SendMessageRequest>,
    ) -> Result<Response<SendMessageResponse>, Status> {
        let (task, _, history_length) = self.send_task(request, false).await?;
        Ok(Response::new(SendMessageResponse {
            payload: Some(pb::send_message_response::Payload::Task(task_to_pb(&task, history_length, true))),
        }))
    }

    type SendStreamingMessageStream =
        Pin<Box<dyn Stream<Item = Result<StreamResponse, Status>> + Send + 'static>>;

    async fn send_streaming_message(
        &self,
        request: Request<SendMessageRequest>,
    ) -> Result<Response<Self::SendStreamingMessageStream>, Status> {
        let (task, caller, history_length) = self.send_task(request, true).await?;
        Ok(Response::new(self.task_stream(caller, task, history_length).await?))
    }

    async fn get_task(
        &self,
        request: Request<GetTaskRequest>,
    ) -> Result<Response<pb::Task>, Status> {
        let caller = self.caller(&request).await?;
        let request = request.into_inner();
        verify_tenant(&caller, &request.tenant)?;
        if request.id.trim().is_empty() {
            return Err(Status::invalid_argument("id is required"));
        }
        let task = self
            .state
            .threads
            .get_durable(&caller.owner, &caller.agent_id, &request.id)
            .await
            .map_err(task_error)?;
        Ok(Response::new(task_to_pb(
            &task,
            request.history_length.map(|value| value.max(0) as usize),
            true,
        )))
    }

    async fn list_tasks(
        &self,
        request: Request<ListTasksRequest>,
    ) -> Result<Response<ListTasksResponse>, Status> {
        let caller = self.caller(&request).await?;
        let request = request.into_inner();
        verify_tenant(&caller, &request.tenant)?;
        let requested_page_size = request.page_size.unwrap_or(50);
        if !(1..=100).contains(&requested_page_size) {
            return Err(Status::invalid_argument("page_size must be between 1 and 100"));
        }
        let status_timestamp_after = request.status_timestamp_after.as_ref().and_then(|timestamp| {
            chrono::DateTime::from_timestamp(timestamp.seconds, timestamp.nanos as u32)
        });
        let mut tasks = self
            .state
            .threads
            .list_durable(&caller.owner, &caller.agent_id)
            .await
            .map_err(task_error)?;
        let status_filter = request.status;
        tasks.retain(|task| {
            (request.context_id.is_empty()
                || task.context_id.as_deref() == Some(&request.context_id))
                && (status_filter == STATE_UNSPECIFIED || task_state(task) == status_filter)
                && status_timestamp_after.as_ref().is_none_or(|after| {
                    task.status
                        .timestamp
                        .as_deref()
                        .and_then(|timestamp| chrono::DateTime::parse_from_rfc3339(timestamp).ok())
                        .is_some_and(|timestamp| timestamp.with_timezone(&chrono::Utc) >= *after)
                })
        });
        tasks.sort_by(|left, right| {
            right
                .status
                .timestamp
                .cmp(&left.status.timestamp)
                .then_with(|| left.id.cmp(&right.id))
        });
        let total_size = tasks.len() as i32;
        let start = match request.page_token.as_str() {
            "" => 0,
            token => tasks
                .iter()
                .position(|task| task.id == token)
                .map(|index| index + 1)
                .ok_or_else(|| Status::invalid_argument("invalid page_token"))?,
        };
        let page = tasks
            .into_iter()
            .skip(start)
            .take(requested_page_size as usize)
            .map(|task| {
                task_to_pb(
                    &task,
                    request.history_length.map(|value| value.max(0) as usize),
                    request.include_artifacts.unwrap_or(false),
                )
            })
            .collect::<Vec<_>>();
        let next_page_token = if start + page.len() < total_size as usize {
            page.last().map(|task| task.id.clone()).unwrap_or_default()
        } else {
            String::new()
        };
        Ok(Response::new(ListTasksResponse {
            tasks: page,
            next_page_token,
            page_size: requested_page_size,
            total_size,
        }))
    }

    async fn cancel_task(
        &self,
        request: Request<CancelTaskRequest>,
    ) -> Result<Response<pb::Task>, Status> {
        let caller = self.caller(&request).await?;
        let request = request.into_inner();
        verify_tenant(&caller, &request.tenant)?;
        if request.id.trim().is_empty() {
            return Err(Status::invalid_argument("id is required"));
        }
        let task = self
            .state
            .threads
            .cancel_durable(&caller.owner, &caller.agent_id, &request.id)
            .await
            .map_err(task_error)?;
        Ok(Response::new(task_to_pb(&task, None, true)))
    }

    type SubscribeToTaskStream =
        Pin<Box<dyn Stream<Item = Result<StreamResponse, Status>> + Send + 'static>>;

    async fn subscribe_to_task(
        &self,
        request: Request<SubscribeToTaskRequest>,
    ) -> Result<Response<Self::SubscribeToTaskStream>, Status> {
        let caller = self.caller(&request).await?;
        let request = request.into_inner();
        verify_tenant(&caller, &request.tenant)?;
        if request.id.trim().is_empty() { return Err(Status::invalid_argument("id is required")); }
        let task = self.state.threads.get_durable(&caller.owner, &caller.agent_id, &request.id).await.map_err(task_error)?;
        if matches!(task_state(&task), STATE_COMPLETED | STATE_FAILED | STATE_CANCELED | STATE_REJECTED) {
            return Err(Status::unimplemented("terminal tasks do not support subscription"));
        }
        Ok(Response::new(self.task_stream(caller, task, None).await?))
    }

    async fn create_task_push_notification_config(
        &self,
        _request: Request<TaskPushNotificationConfig>,
    ) -> Result<Response<TaskPushNotificationConfig>, Status> {
        Err(Status::unimplemented("push notifications are not supported"))
    }

    async fn get_task_push_notification_config(
        &self,
        _request: Request<GetTaskPushNotificationConfigRequest>,
    ) -> Result<Response<TaskPushNotificationConfig>, Status> {
        Err(Status::unimplemented("push notifications are not supported"))
    }

    async fn list_task_push_notification_configs(
        &self,
        _request: Request<ListTaskPushNotificationConfigsRequest>,
    ) -> Result<Response<pb::ListTaskPushNotificationConfigsResponse>, Status> {
        Err(Status::unimplemented("push notifications are not supported"))
    }

    async fn get_extended_agent_card(
        &self,
        _request: Request<GetExtendedAgentCardRequest>,
    ) -> Result<Response<pb::AgentCard>, Status> {
        Err(Status::unimplemented("extended agent cards are not supported"))
    }

    async fn delete_task_push_notification_config(
        &self,
        _request: Request<DeleteTaskPushNotificationConfigRequest>,
    ) -> Result<Response<()>, Status> {
        Err(Status::unimplemented("push notifications are not supported"))
    }
}
