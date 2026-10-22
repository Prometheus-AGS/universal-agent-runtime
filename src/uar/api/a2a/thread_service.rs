//! A2A task projection over the existing persisted-thread mailbox host.
//! Task IDs/context IDs are transport correlation, never kernel authority.

use crate::uar::security::credential_capture::AuthenticatedCredentialCapture;
use std::{collections::HashMap, sync::Arc};

use tokio::sync::{Mutex, oneshot::error::TryRecvError};

use crate::uar::runtime::{
    actor::{
        messages::{ActorOwner, ActorRunError},
        system::{ActorCollaboration, ActorSession, ActorTurn},
    },
    thread::AgentThreadResult,
};

use super::contract::{
    UAR_CLEANUP_CLOSED_METADATA, UAR_DELEGATION_ACK_METADATA, UAR_DELEGATION_CONTRACT_METADATA,
    UAR_USAGE_METADATA, UarDelegationAcknowledgement, UarDelegationContract, UarUsageReceipt,
};
use super::types::{Message, MessageSendParams, Part, Role, Task, TaskState, TaskStatus};

#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("task not found")]
    NotFound,
    #[error("task is active, closed, or cannot be cancelled")]
    Conflict,
    #[error("{0}")]
    Invalid(&'static str),
    #[error("agent task host failed")]
    Host(#[source] anyhow::Error),
}

type Key = (ActorOwner, String, String);

#[derive(Default)]
struct Bindings {
    tasks: HashMap<Key, Arc<Mutex<Entry>>>,
    contexts: HashMap<Key, String>,
}

struct Entry {
    tenant_id: Option<String>,
    actor: ActorSession,
    durable: Option<super::durable::A2aBinding>,
    task: Task,
    turn: Option<ActorTurn>,
    closed: bool,
    cleanup_pending: bool,
    contract: Option<UarDelegationContract>,
    run_id: Option<String>,
    accounted_run_id: Option<String>,
    usage: UarUsageReceipt,
}

impl Entry {
    fn refresh(&mut self) {
        let Some(turn) = &mut self.turn else {
            return;
        };
        let record = turn
            .thread
            .borrow()
            .clone()
            .filter(|record| record.thread.run_id.as_deref() == Some(turn.run_id.as_str()));
        if let Some(record) = &record {
            let thread = &record.thread;
            self.task
                .metadata
                .insert("thread_id".into(), serde_json::json!(thread.thread_id));
            self.task
                .metadata
                .insert("root_run_id".into(), serde_json::json!(thread.root_run_id));
            self.task.metadata.insert(
                "canonical_path".into(),
                serde_json::json!(thread.canonical_path),
            );
            self.task.status.timestamp = Some(thread.updated_at.to_rfc3339());
            // A durable terminal record precedes the mailbox reply. Wait for
            // both before accepting a follow-up or reporting terminal success.
            self.task.status.state = if thread.status == crate::uar::runtime::thread::AgentThreadStatus::Waiting {
                TaskState::InputRequired
            } else { TaskState::Working };
        }
        let reply = match turn.completion.try_recv() {
            Ok(reply) => Some(reply),
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Closed) => None,
        };
        let (state, message) = match reply {
            Some(Ok(record))
                if record.thread.run_id.as_deref() == Some(turn.run_id.as_str())
                    && record.thread.status.is_terminal() =>
            {
                let thread = record.thread;
                self.task
                    .metadata
                    .insert("thread_id".into(), serde_json::json!(thread.thread_id));
                self.task
                    .metadata
                    .insert("root_run_id".into(), serde_json::json!(thread.root_run_id));
                self.task.metadata.insert(
                    "canonical_path".into(),
                    serde_json::json!(thread.canonical_path),
                );
                self.task.status.timestamp = Some(thread.updated_at.to_rfc3339());
                // A compiler tool can succeed before a later model failure or
                // cancellation. Preserve its receipt without relabelling the
                // run successful. Only the exact, closed invocation is read.
                let artifacts = turn.artifacts.snapshot();
                if let Ok(artifacts) = &artifacts {
                    let existing_ids = self.task.artifacts.iter().map(|artifact| artifact.artifact_id.clone()).collect::<std::collections::HashSet<_>>();
                    self.task.artifacts.extend(artifacts.iter().filter(|artifact| !existing_ids.contains(&artifact.artifact_id)).map(|artifact| {
                        super::types::Artifact {
                            artifact_id: artifact.artifact_id.clone(),
                            name: Some(artifact.name.clone()),
                            description: Some(artifact.description.clone()),
                            parts: vec![Part::Data {
                                data: artifact.data.clone(),
                            }],
                            metadata: HashMap::new(),
                        }
                    }));
                }
                match thread.result {
                    Some(AgentThreadResult::Completed { output }) if artifacts.is_ok() => {
                        (TaskState::Completed, Some(Message::agent_text(output)))
                    }
                    Some(AgentThreadResult::Completed { .. }) => {
                        self.closed = true;
                        (
                            TaskState::Failed,
                            Some(Message::agent_text("Agent artifact receipt is unavailable")),
                        )
                    }
                    Some(AgentThreadResult::Yielded { .. }) => (TaskState::InputRequired, None),
                    Some(AgentThreadResult::Cancelled) => (TaskState::Canceled, None),
                    Some(AgentThreadResult::Failed { code, .. }) => {
                        if code.ends_with("persistence_unconfirmed") {
                            self.task.metadata.insert("context_unconfirmed".into(), serde_json::json!(true));
                        }
                        if matches!(
                            code.as_str(),
                            "thread_cleanup_unconfirmed"
                                | "sandbox_cleanup_unconfirmed"
                                | "terminal_cleanup_unconfirmed"
                        ) {
                            self.cleanup_pending = true;
                            self.closed = true;
                            self.task
                                .metadata
                                .insert("cleanup_unconfirmed".into(), serde_json::json!(true));
                        }
                        (
                            TaskState::Failed,
                            Some(Message::agent_text("Agent execution failed")),
                        )
                    }
                    None => (TaskState::Failed, None),
                }
            }
            Some(Err(ActorRunError::Stopped)) if self.closed => (TaskState::Canceled, None),
            _ => {
                // Failed/unknown host persistence is not successful execution
                // and cannot authorize another prompt on this task.
                self.closed = true;
                self.cleanup_pending = true;
                self.task
                    .metadata
                    .insert("cleanup_unconfirmed".into(), serde_json::json!(true));
                self.task
                    .metadata
                    .insert("execution_unconfirmed".into(), serde_json::json!(true));
                (
                    TaskState::Failed,
                    Some(Message::agent_text("Agent task completion is unconfirmed")),
                )
            }
        };
        if let Some(message) = &message {
            self.task.history.push(message.clone());
        }
        self.task.status = TaskStatus {
            state,
            message,
            timestamp: self.task.status.timestamp.clone(),
        };
        self.turn = None;
        self.refresh_contract_receipt();
    }

    fn refresh_contract_receipt(&mut self) {
        let Some(contract) = &self.contract else {
            return;
        };
        let remote_thread_id = self
            .task
            .metadata
            .get("thread_id")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned);
        if let Ok(receipt) = UarDelegationAcknowledgement::for_contract(contract, remote_thread_id)
            && let Ok(value) = serde_json::to_value(receipt)
        {
            self.task
                .metadata
                .insert(UAR_DELEGATION_ACK_METADATA.into(), value);
        }
    }
}

/// Shared HTTP/gRPC adapter. Mailboxes own work; this registry owns only its
/// owner/artifact-qualified correlation and lossless completion receivers.
pub struct A2AThreadService {
    actors: Arc<ActorCollaboration>,
    bindings: Mutex<Bindings>,
    instance_id: Option<String>,
    full_harness: Option<Arc<crate::uar::api::full_harness::FullHarnessTaskAuthority>>,
    guardrails: crate::config::GuardrailsConfig,
    process_id: String,
    run_api: Option<Arc<crate::uar::api::routes::RunApiState>>,
}

impl std::fmt::Debug for A2AThreadService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("A2AThreadService").finish_non_exhaustive()
    }
}

impl A2AThreadService {
    fn refresh_governed_metadata(&self, entry: &mut Entry) {
        entry.refresh_contract_receipt();
        if entry.contract.is_some()
            && matches!(
                entry.task.status.state,
                TaskState::Completed | TaskState::Canceled | TaskState::Failed
            )
            && let Some(run_id) = &entry.run_id
        {
            if entry.accounted_run_id.as_deref() != Some(run_id) {
                let turn = self.actors.run_usage(run_id);
                entry.usage.total_tokens =
                    entry.usage.total_tokens.saturating_add(turn.total_tokens);
                entry.usage.cost_usd += turn.cost_usd;
                entry.accounted_run_id = Some(run_id.clone());
            }
            if let Some(contract) = &entry.contract
                && let Ok(digest) = contract.digest()
            {
                let activity = self.actors.run_usage(&format!("a2a:{digest}"));
                entry.usage.model_requests = activity.model_requests;
                entry.usage.tool_calls = activity.tool_calls;
            }
            if let Ok(value) = serde_json::to_value(entry.usage) {
                entry.task.metadata.insert(UAR_USAGE_METADATA.into(), value);
            }
        }
    }

    pub(crate) fn supports_client_tools(&self) -> bool {
        self.run_api.is_some() && self.actors.persistence().is_ok_and(|store|
            store.supports_durable_a2a_bindings() && store.supports_durable_client_tools())
    }

    pub fn new(actors: Arc<ActorCollaboration>) -> Self {
        Self {
            actors,
            bindings: Mutex::new(Bindings::default()),
            instance_id: None,
            full_harness: None,
            guardrails: crate::config::GuardrailsConfig::default(),
            process_id: uuid::Uuid::new_v4().to_string(),
            run_api: None,
        }
    }

    #[must_use]
    pub fn with_run_api(mut self, state: Arc<crate::uar::api::routes::RunApiState>) -> Self {
        self.run_api = Some(state);
        self
    }

    #[must_use]
    pub fn with_guardrails(mut self, guardrails: crate::config::GuardrailsConfig) -> Self {
        self.guardrails = guardrails;
        self
    }

    #[must_use]
    pub fn with_instance_id(mut self, instance_id: impl Into<String>) -> Self {
        let instance_id = instance_id.into();
        self.instance_id = (!instance_id.trim().is_empty()).then_some(instance_id);
        self
    }

    #[must_use]
    pub fn with_full_harness(
        mut self,
        authority: Arc<crate::uar::api::full_harness::FullHarnessTaskAuthority>,
    ) -> Self {
        self.full_harness = Some(authority);
        self
    }

    pub(crate) fn full_harness(
        &self,
    ) -> Option<&Arc<crate::uar::api::full_harness::FullHarnessTaskAuthority>> {
        self.full_harness.as_ref()
    }

    pub async fn send(
        &self,
        owner: &ActorOwner,
        authenticated_instance_id: Option<&str>,
        agent_id: &str,
        params: MessageSendParams,
    ) -> Result<Task, TaskError> {
        self.send_with_capture(owner, authenticated_instance_id, agent_id, params,
            AuthenticatedCredentialCapture::default()).await
    }

    pub(crate) async fn send_with_capture(
        &self,
        owner: &ActorOwner,
        authenticated_instance_id: Option<&str>,
        agent_id: &str,
        params: MessageSendParams,
        credential_capture: AuthenticatedCredentialCapture,
    ) -> Result<Task, TaskError> {
        self.send_internal(owner, authenticated_instance_id, agent_id, params, credential_capture, None, None).await
    }

    async fn send_internal(
        &self,
        owner: &ActorOwner,
        authenticated_instance_id: Option<&str>,
        agent_id: &str,
        params: MessageSendParams,
        credential_capture: AuthenticatedCredentialCapture,
        reservation: Option<(String, String)>,
        prepared_client_tools: Option<crate::uar::runtime::turn::client_tools::ClientToolInput>,
    ) -> Result<Task, TaskError> {
        let mut client_tools = prepared_client_tools.or(
            super::client_tools::ClientToolsExtension::parse(&params.metadata)
                .map_err(TaskError::Invalid)?.map(|extension| extension.into_input())
        );
        if let Some(input) = &client_tools {
            input.validate().map_err(|_| TaskError::Invalid("Caller-tools declaration is invalid"))?;
        }
        if client_tools.is_some() && reservation.is_none() {
            return Err(TaskError::Invalid("Caller tools require the durable A2A 1.0 interface"));
        }
        if params.message.role != Role::User
            || params.message.parts.is_empty()
            || params
                .message
                .parts
                .iter()
                .any(|part| !matches!(part, Part::Text { .. }))
        {
            return Err(TaskError::Invalid(
                "agent input must contain user text parts",
            ));
        }
        let content = params
            .message
            .parts
            .iter()
            .filter_map(|part| match part {
                Part::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");
        if content.trim().is_empty() {
            return Err(TaskError::Invalid("agent input must not be empty"));
        }

        if let Some(input) = &mut client_tools
            && input.history.is_empty() {
            input.history.push(serde_json::json!({"role": "user", "content": content}));
        }

        // Input guardrails: screen A2A input for injection/PII (#328).
        if let Some(finding) = crate::uar::guardrails::screen_input(&content, &self.guardrails) {
            crate::uar::telemetry::metrics::record_guardrail_flagged(finding.category.as_str());
            tracing::warn!(
                category = %finding.category.as_str(),
                reason = %finding.reason,
                "A2A input flagged by guardrail"
            );
            let should_block = match finding.category {
                crate::uar::guardrails::GuardrailCategory::Injection => {
                    self.guardrails.block_on_injection
                }
                crate::uar::guardrails::GuardrailCategory::Pii => self.guardrails.block_on_pii,
            };
            if should_block {
                return Err(TaskError::Invalid("Input rejected by guardrail policy"));
            }
        }

        let contract = params
            .metadata
            .get(UAR_DELEGATION_CONTRACT_METADATA)
            .map(|value| {
                serde_json::from_value::<UarDelegationContract>(value.clone())
                    .map_err(|_| TaskError::Invalid("UAR delegation contract is malformed"))
            })
            .transpose()?;
        if let Some(contract) = &contract {
            contract
                .validate()
                .map_err(|_| TaskError::Invalid("UAR delegation contract is invalid"))?;
            if contract.owner_id != owner.user_id()
                || authenticated_instance_id != Some(contract.source_instance_id.as_str())
                || contract.target_agent_id != agent_id
                || self.instance_id.as_deref() != Some(contract.target_instance_id.as_str())
            {
                return Err(TaskError::Invalid(
                    "UAR delegation contract does not match the authenticated peer or target",
                ));
            }
        }
        let entry = {
            let mut bindings = self.bindings.lock().await;
            let existing = params.task_id.clone().or_else(|| {
                if reservation.is_some() { return None; }
                params.context_id.as_ref().and_then(|context| {
                    bindings
                        .contexts
                        .get(&(owner.clone(), agent_id.to_owned(), context.clone()))
                        .cloned()
                })
            });
            match existing {
                Some(id) => bindings
                    .tasks
                    .get(&(owner.clone(), agent_id.to_owned(), id))
                    .cloned()
                    .ok_or(TaskError::NotFound)?,
                None => {
                    let id = reservation.as_ref().map_or_else(
                        || uuid::Uuid::new_v4().to_string(), |(id, _)| id.clone());
                    let context_id = params.context_id.clone().unwrap_or_else(|| id.clone());
                    let initial_task = Task {
                        id: id.clone(), context_id: Some(context_id.clone()),
                        status: TaskStatus { state: TaskState::Submitted, message: None,
                            timestamp: Some(chrono::Utc::now().to_rfc3339()) },
                        history: vec![params.message.clone()], artifacts: Vec::new(),
                        metadata: HashMap::from([("agent_id".into(), serde_json::json!(agent_id))]),
                    };
                    let durable = if let Some((_, digest)) = &reservation {
                        let record = super::durable::A2aBinding {
                            schema_version: 1, owner_key: durable_owner(owner)?, agent_id: agent_id.into(),
                            task_id: id.clone(), context_id: context_id.clone(), revision: 1,
                            process_id: self.process_id.clone(), runtime_epoch: self.run_api.as_ref().map(|state|state.manager.runtime_epoch().to_owned()), run_id: uuid::Uuid::new_v4().to_string(),
                            request_digest: digest.clone(), cancellation_requested: false, task: initial_task, contract: contract.clone(),
                        };
                        let persistence = self.actors.persistence().map_err(TaskError::Host)?;
                        if !persistence.create_a2a_binding(&record).await.map_err(TaskError::Host)? {
                            return Err(TaskError::Conflict);
                        }
                        Some(record)
                    } else { None };
                    let actor_name = format!("a2a-{id}");
                    let actor = if durable.is_some() {
                        self.actors.spawn_a2a_session(owner, &id, agent_id, &context_id, contract.as_ref()).await
                    } else { match &contract {
                        Some(contract) => {
                            self.actors
                                .spawn_governed_session(
                                    owner,
                                    actor_name,
                                    agent_id.to_owned(),
                                    contract.execution_policy(),
                                    contract.presentation_negotiation.clone(),
                                    contract.budgets.clone(),
                                    contract.usage_grant.clone(),
                                    contract.sandbox.clone(),
                                    format!("a2a:{}", contract.digest().map_err(TaskError::Host)?),
                                )
                                .await
                        }
                        None => {
                            self.actors
                                .spawn_session(owner, actor_name, agent_id.to_owned(), None)
                                .await
                        }
                    }}
                    .map_err(TaskError::Host)?;
                    let context_id = params.context_id.clone().unwrap_or_else(|| id.clone());
                    let entry = Arc::new(Mutex::new(Entry {
                        tenant_id: owner.tenant_id().map(str::to_owned),
                        actor,
                        durable,
                        turn: None,
                        closed: false,
                        cleanup_pending: false,
                        contract: contract.clone(),
                        run_id: None,
                        accounted_run_id: None,
                        usage: UarUsageReceipt::default(),
                        task: Task {
                            id: id.clone(),
                            context_id: Some(context_id.clone()),
                            status: TaskStatus {
                                state: TaskState::Submitted,
                                message: None,
                                timestamp: Some(chrono::Utc::now().to_rfc3339()),
                            },
                            history: Vec::new(),
                            artifacts: Vec::new(),
                            metadata: HashMap::from([(
                                "agent_id".into(),
                                serde_json::json!(agent_id),
                            )]),
                        },
                    }));
                    bindings.tasks.insert(
                        (owner.clone(), agent_id.to_owned(), id.clone()),
                        Arc::clone(&entry),
                    );
                    bindings
                        .contexts
                        .insert((owner.clone(), agent_id.to_owned(), context_id), id);
                    entry
                }
            }
        };
        let entry_handle = Arc::clone(&entry);
        let mut entry = entry.lock().await;
        entry.refresh();
        if entry.closed || entry.turn.is_some() {
            return Err(TaskError::Conflict);
        }
        if entry.contract != contract {
            return Err(TaskError::Invalid(
                "task cannot change its UAR delegation contract",
            ));
        }
        if params
            .context_id
            .as_ref()
            .is_some_and(|context| entry.task.context_id.as_ref() != Some(context))
        {
            return Err(TaskError::Invalid("task and context do not match"));
        }
        let turn = if let Some(mut record) = entry.durable.clone() {
            if !entry.task.history.is_empty() {
                // A public get may have committed approval/status metadata
                // after the local projection. Rebase on that exact same run,
                // never on a continuation or cancellation won by another caller.
                let current = self.actors.persistence().map_err(TaskError::Host)?
                    .load_a2a_binding(&record.owner_key, &record.agent_id, &record.task_id)
                    .await.map_err(TaskError::Host)?.ok_or(TaskError::NotFound)?;
                if current.run_id != record.run_id || current.process_id != record.process_id
                    || current.cancellation_requested { return Err(TaskError::Conflict); }
                record = current;
                let before = record.clone();
                record.revision = record.revision.checked_add(1).ok_or(TaskError::Conflict)?;
                record.run_id = uuid::Uuid::new_v4().to_string();
                record.cancellation_requested = false;
                record.request_digest = reservation.as_ref().ok_or(TaskError::Conflict)?.1.clone();
                record.task.history.push(params.message.clone());
                record.task.status.state = TaskState::Submitted;
                record.process_id = self.process_id.clone();
                record.runtime_epoch = self.run_api.as_ref().map(|state|state.manager.runtime_epoch().to_owned());
                if !self.actors.persistence().map_err(TaskError::Host)?
                    .compare_and_swap_a2a_binding(&before, &record).await.map_err(TaskError::Host)? {
                    return Err(TaskError::Conflict);
                }
                entry.durable = Some(record.clone());
            }
            entry.actor.submit_authorized_prompt_with_client_tools(owner, record.run_id, content, credential_capture, client_tools)
        } else {
            entry.actor.submit_authorized_prompt_with_client_tools(
                owner,
                uuid::Uuid::new_v4().to_string(),
                content,
                credential_capture,
                client_tools,
            )
        }.map_err(TaskError::Host)?;
        // Publication is synchronous after enqueue: request cancellation cannot
        // abandon the completion receiver between dispatch and registration.
        entry.task.history.push(params.message);
        entry.task.metadata.retain(|key, _| {
            key == "agent_id" || key == UAR_DELEGATION_ACK_METADATA || key == UAR_USAGE_METADATA
        });
        entry
            .task
            .metadata
            .insert("run_id".into(), serde_json::json!(turn.run_id));
        entry.run_id = Some(turn.run_id.clone());
        entry.task.status = TaskStatus {
            state: TaskState::Working,
            message: None,
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
        };
        entry.turn = Some(turn);
        entry.refresh();
        self.refresh_governed_metadata(&mut entry);
        if entry.durable.is_some() {
            let persistence = self.actors.persistence().map_err(TaskError::Host)?;
            // This observer retains completion correlation after an HTTP caller
            // disconnects. It never submits or executes a run.
            let observed_run = entry.run_id.clone();
            let manager = Arc::clone(self.actors.run_manager());
            let scope = manager.lifecycle_scope();
            // Enqueueing an actor turn precedes manager run registration. Keep
            // the exact mailbox completion as an observation source even when
            // admission fails before a run event channel can be installed.
            if entry.turn.is_none() {
                self.persist_projection(&mut entry).await?;
                return Ok(entry.task.clone());
            }
            let mut registrations = manager.subscribe_run_registrations();
            let mut committed_work = manager.subscribe_committed_work();
            let mut events = manager.subscribe(observed_run.as_deref().unwrap_or_default()).await;
            let cancellation = scope.cancellation_token();
            if scope.spawn("a2a-durable-projection", async move {
                // Extract only after lifecycle admission. Rejected task admission
                // must leave Entry's original completion receiver untouched.
                let (mut actor_completion, mut completion_sender) = {
                    let mut entry = entry_handle.lock().await;
                    if entry.run_id != observed_run { return Ok(()); }
                    match entry.turn.as_mut() {
                        Some(turn) => {
                            let (sender, receiver) = tokio::sync::oneshot::channel();
                            (Some(std::mem::replace(&mut turn.completion, receiver)), Some(sender))
                        }
                        None => (None, None),
                    }
                };
                let outcome = async {
                let mut first_snapshot = true;
                loop {
                    if cancellation.is_cancelled() { return Ok(()); }
                    if !first_snapshot {
                        tokio::select! {
                            _ = cancellation.cancelled() => return Ok(()),
                            completed = async {
                                match &mut actor_completion {
                                    Some(completion) => completion.await,
                                    None => std::future::pending().await,
                                }
                            } => {
                                actor_completion = None;
                                if let Some(sender) = completion_sender.take() {
                                    if let Ok(result) = completed { let _ = sender.send(result); }
                                }
                            },
                            event = async {
                                match &mut events {
                                    Some(events) => events.recv().await,
                                    None => std::future::pending().await,
                                }
                            } => match event {
                                Ok(_) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                                    return Err(anyhow::anyhow!("A2A durable projection lost its run event source"));
                                }
                            },
                            changed = registrations.changed() => {
                                if changed.is_err() {
                                    return Err(anyhow::anyhow!("A2A durable projection lost its run registration source"));
                                }
                            },
                            changed = committed_work.changed() => {
                                if changed.is_err() {
                                    return Err(anyhow::anyhow!("A2A durable projection lost its committed-work source"));
                                }
                            },
                        }
                    }
                    first_snapshot = false;
                    if events.is_none() {
                        events = manager.subscribe(observed_run.as_deref().unwrap_or_default()).await;
                    }
                    let mut entry = entry_handle.lock().await;
                    if entry.run_id != observed_run { return Ok(()); }
                    if let Some(before) = &entry.durable {
                        match persistence.load_a2a_binding(&before.owner_key, &before.agent_id, &before.task_id).await {
                            Ok(Some(current)) if current.run_id == before.run_id && current.process_id == before.process_id => {
                                entry.durable = Some(current);
                            }
                            _ => return Ok(()),
                        }
                    }
                    if entry.durable.as_ref().is_some_and(|record| record.cancellation_requested) {
                        entry.closed = true;
                        entry.cleanup_pending = true;
                        entry.task.metadata.insert("cleanup_unconfirmed".into(), serde_json::json!(true));
                        let stopped = entry.actor.stop().await;
                        entry.refresh();
                        if let Err(error) = stopped {
                            tracing::error!(%error, "A2A cancellation cleanup remains unconfirmed");
                            entry.task.status.state = TaskState::Failed;
                        } else {
                            entry.cleanup_pending = false;
                            entry.task.metadata.remove("cleanup_unconfirmed");
                            entry.task.metadata.insert(UAR_CLEANUP_CLOSED_METADATA.into(), serde_json::json!(true));
                        }
                    }
                    entry.refresh();
                    let settled = entry.turn.is_none();
                    if let Err(error) = persist_entry_projection(persistence.as_ref(), manager.as_ref(), &mut entry).await {
                        if matches!(error, TaskError::Conflict) {
                            first_snapshot = true;
                            continue;
                        }
                        return Err(anyhow::anyhow!("A2A durable completion projection failed: {error}"));
                    }
                    if settled {
                        // Mailbox completion is distinct from producer settlement.
                        // Wake exact-task waiters only after consuming and projecting it.
                        manager.notify_committed_work();
                        return Ok(());
                    }
                }
                }.await;
                // Cancellation or another observation failure must not close a
                // still-live actor completion merely because its observer exits.
                if let Some(original) = actor_completion.take() {
                    let mut entry = entry_handle.lock().await;
                    if entry.run_id == observed_run
                        && let Some(turn) = entry.turn.as_mut()
                        && Some(turn.run_id.as_str()) == observed_run.as_deref()
                    {
                        turn.completion = original;
                    }
                }
                drop(completion_sender);
                outcome
            }).is_err() {
                scope.record_unresolved_effect("a2a-durable-projection:scope-admission-closed");
            }
        }
        self.persist_projection(&mut entry).await?;
        Ok(entry.task.clone())
    }

    async fn entry(
        &self,
        owner: &ActorOwner,
        agent_id: &str,
        task_id: &str,
    ) -> Result<Arc<Mutex<Entry>>, TaskError> {
        self.bindings
            .lock()
            .await
            .tasks
            .get(&(owner.clone(), agent_id.to_owned(), task_id.to_owned()))
            .cloned()
            .ok_or(TaskError::NotFound)
    }

    pub async fn get(
        &self,
        owner: &ActorOwner,
        agent_id: &str,
        task_id: &str,
    ) -> Result<Task, TaskError> {
        let entry = self.entry(owner, agent_id, task_id).await?;
        let mut entry = entry.lock().await;
        entry.refresh();
        self.refresh_governed_metadata(&mut entry);
        self.persist_projection(&mut entry).await?;
        Ok(entry.task.clone())
    }

    pub async fn cancel(
        &self,
        owner: &ActorOwner,
        agent_id: &str,
        task_id: &str,
    ) -> Result<Task, TaskError> {
        let entry = self.entry(owner, agent_id, task_id).await?;
        // Serialize settlement and its projection together. A delayed failed
        // attempt must not overwrite a newer successful cancellation receipt.
        let mut entry = entry.lock().await;
        entry.refresh();
        self.refresh_governed_metadata(&mut entry);
        if entry.closed && entry.turn.is_none() && !entry.cleanup_pending {
            return Err(TaskError::Conflict);
        }
        entry.closed = true;
        entry.cleanup_pending = true;
        // If this waiter disappears during stop, subsequent reads must expose
        // the outstanding settlement rather than only the execution outcome.
        entry
            .task
            .metadata
            .insert("cleanup_unconfirmed".into(), serde_json::json!(true));
        // The registry, not this HTTP/gRPC future, retains the actor join.
        let stopped = entry.actor.stop().await;
        entry.refresh();
        if let Err(error) = stopped {
            entry.task.status.state = TaskState::Failed;
            entry
                .task
                .metadata
                .insert("cleanup_unconfirmed".into(), serde_json::json!(true));
            return Err(TaskError::Host(error));
        }
        entry.cleanup_pending = false;
        entry.task.metadata.remove("cleanup_unconfirmed");
        entry
            .task
            .metadata
            .insert(UAR_CLEANUP_CLOSED_METADATA.into(), serde_json::json!(true));
        self.refresh_governed_metadata(&mut entry);
        self.persist_projection(&mut entry).await?;
        Ok(entry.task.clone())
    }
}

fn durable_owner(owner: &ActorOwner) -> Result<String, TaskError> {
    owner.credential_principal().map(|principal| principal.storage_key())
        .ok_or(TaskError::Invalid("verified complete credential principal required"))
}

impl A2AThreadService {
    async fn persist_projection(&self, entry: &mut Entry) -> Result<(), TaskError> {
        persist_entry_projection(self.actors.persistence().map_err(TaskError::Host)?.as_ref(), self.actors.run_manager().as_ref(), entry).await
    }

    /// A v1 request reserves durable correlation before the existing actor owns
    /// its mailbox turn. A retried message cannot enqueue a second invocation.
    pub(crate) async fn send_durable(
        &self, owner: &ActorOwner, user: &crate::uar::security::claims::UserContext,
        instance: Option<&str>, agent: &str,
        params: MessageSendParams, message_id: &str, capture: AuthenticatedCredentialCapture,
    ) -> Result<Task, TaskError> {
        use sha2::{Digest, Sha256};
        let caller_tools = super::client_tools::ClientToolsExtension::parse(&params.metadata)
            .map_err(TaskError::Invalid)?;
        if caller_tools.as_ref().is_some_and(|extension| extension.history.iter().any(|message| message["role"] == "tool"))
            && (params.task_id.is_none() || params.context_id.is_none()) {
            return Err(TaskError::Invalid("Caller-tool results require the exact taskId and contextId"));
        }
        if caller_tools.is_some() && params.metadata.contains_key("uar.approval") {
            return Err(TaskError::Invalid("Caller-tool results and approval decisions require separate messages"));
        }
        if params.metadata.contains_key("uar.approval") && params.task_id.is_none() {
            return Err(TaskError::Invalid("Approval requires its existing taskId"));
        }
        let owner_key = durable_owner(owner)?;
        let persistence = self.actors.persistence().map_err(TaskError::Host)?;
        if !persistence.supports_durable_a2a_bindings() {
            return Err(TaskError::Host(anyhow::anyhow!("Durable A2A correlation unavailable")));
        }
        let digest = Sha256::digest(serde_json::to_vec(&serde_json::json!({
            "message": params.message, "context": params.context_id,
            "metadata": params.metadata, "message_id": message_id,
        })).map_err(|error| TaskError::Host(error.into()))?)
            .iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        let id = params.task_id.clone().unwrap_or_else(|| format!("a2a-{}",
            super::durable::A2aBinding::storage_key_for(&owner_key, agent, message_id)));
        if let Some(record) = persistence.load_a2a_binding(&owner_key, agent, &id).await.map_err(TaskError::Host)? {
            if params.metadata.contains_key("uar.approval") {
                return self.decide_durable(owner, user, agent, record, params, digest, capture).await;
            }
            if record.request_digest == digest {
                return self.get_durable(owner, agent, &id).await;
            }
            if params.task_id.is_none() {
                return Err(TaskError::Invalid("messageId already identifies a different message"));
            }
            if caller_tools.is_some() {
                let current = persistence.load_a2a_binding(&owner_key, agent, &id).await
                    .map_err(TaskError::Host)?.ok_or(TaskError::NotFound)?;
                return self.resume_client_tools(owner, instance, agent, current, params, digest, capture).await;
            }
            let task = self.get_durable(owner, agent, &id).await?;
            if task.status.state != TaskState::InputRequired {
                return Err(TaskError::Conflict);
            }
            if self.entry(owner, agent, &id).await.is_err() {
                return self.resume_durable(owner, user, agent, record, params, digest, capture, None).await;
            }
        } else if params.task_id.is_some() {
            return Err(TaskError::NotFound);
        }
        if params.task_id.is_none() && let Some(context) = &params.context_id {
            // A previous process can commit its terminal root before its
            // projection observer runs. Reconcile that receipt before the
            // atomic context-slot reservation; never release an uncertain run.
            for record in persistence.list_a2a_bindings(&owner_key, agent).await.map_err(TaskError::Host)? {
                if record.context_id == *context && !record.context_closed() {
                    self.get_durable(owner, agent, &record.task_id).await?;
                }
            }
        }
        let prepared = if let Some(extension) = caller_tools {
            let input = extension.into_input();
            match crate::uar::api::client_tools::prepare(self.actors.run_manager().as_ref(), persistence.as_ref(), &owner_key, owner.tenant_id(), agent, &input)
                .await.map_err(|_| TaskError::Invalid("Caller-tool history conflicts with durable evidence"))? {
                crate::uar::api::client_tools::ClientToolPreparation::Fresh(input) => Some(input),
                _ => return Err(TaskError::Invalid("Caller-tool results require their existing task")),
            }
        } else { None };
        self.send_internal(owner, instance, agent, params, capture, Some((id, digest)), prepared).await
    }

    async fn resume_client_tools(
        &self, owner: &ActorOwner, instance: Option<&str>, agent: &str,
        record: super::durable::A2aBinding, mut params: MessageSendParams, digest: String,
        capture: AuthenticatedCredentialCapture,
    ) -> Result<Task, TaskError> {
        use crate::uar::api::client_tools::{self, ClientToolPreparation};
        if params.context_id.as_deref() != Some(record.context_id.as_str()) {
            return Err(TaskError::Invalid("Caller-tool continuation requires its exact task and context"));
        }
        if params.message.role != Role::User { return Err(TaskError::Invalid("Caller-tool results require a user message")); }
        let contract = params.metadata.get(UAR_DELEGATION_CONTRACT_METADATA)
            .map(|value| serde_json::from_value::<UarDelegationContract>(value.clone()))
            .transpose().map_err(|_| TaskError::Invalid("UAR delegation contract is malformed"))?;
        if contract != record.contract { return Err(TaskError::Invalid("Continuation requires its current peer delegation contract")); }
        if let Some(contract) = &contract {
            contract.validate().map_err(|_| TaskError::Invalid("UAR delegation contract is invalid"))?;
            if contract.owner_id != owner.user_id() || instance != Some(contract.source_instance_id.as_str())
                || self.instance_id.as_deref() != Some(contract.target_instance_id.as_str())
                || contract.target_agent_id != agent {
                return Err(TaskError::Invalid("Continuation delegation does not match the authenticated peer"));
            }
        }
        let input = super::client_tools::ClientToolsExtension::parse(&params.metadata)
            .map_err(TaskError::Invalid)?.ok_or(TaskError::Invalid("Caller-tools extension required"))?.into_input();
        let call_id = input.history.last().and_then(|message| message.get("tool_call_id"))
            .and_then(serde_json::Value::as_str).ok_or(TaskError::Invalid("Caller-tool results required"))?;
        let batch_id = client_tools::batch_id_from_call_id(call_id)
            .map_err(|_| TaskError::Invalid("Invalid caller-tool call identity"))?;
        let persistence = self.actors.persistence().map_err(TaskError::Host)?;
        let tenant = owner.tenant_id();
        let batch = persistence.load_client_tool_continuation(&record.owner_key, tenant, agent, &batch_id)
            .await.map_err(TaskError::Host)?.ok_or(TaskError::Conflict)?;
        if batch.batch_id != batch_id { return Err(TaskError::Invalid("Caller-tool batch does not belong to this task turn")); }
        let session_id = format!("a2a-context-{}", super::durable::A2aBinding::storage_key_for(
            &record.owner_key, agent, &record.context_id));
        if batch.session_id.as_deref() != Some(session_id.as_str()) {
            return Err(TaskError::Invalid("Caller-tool batch belongs to another context session"));
        }
        if batch.state != crate::uar::persistence::client_tools::ClientToolContinuationState::Completed {
            if record.cancellation_requested { return Err(TaskError::Conflict); }
            if batch.source_run_id != record.run_id { return Err(TaskError::Conflict); }
            let task = self.get_durable(owner, agent, &record.task_id).await?;
            if task.status.state != TaskState::InputRequired { return Err(TaskError::Conflict); }
            if let Ok(entry) = self.entry(owner, agent, &record.task_id).await {
                let mut entry = entry.lock().await;
                entry.refresh();
                if entry.closed || entry.turn.is_some() { return Err(TaskError::Conflict); }
            }
            self.actors.run_manager().resolve_registered_agent(agent).await.map_err(|error| TaskError::Host(error.into()))?;
        }
        let preparation = client_tools::prepare(self.actors.run_manager().as_ref(), persistence.as_ref(), &record.owner_key, tenant, agent, &input)
            .await.map_err(|_| TaskError::Invalid("Caller-tool continuation conflicts with its durable batch"))?;
        let (input, claim) = match preparation {
            ClientToolPreparation::Continue { input, claim } => (input, claim),
            ClientToolPreparation::Replay { response, .. } => {
                let task: Task = serde_json::from_value(response).map_err(|error| TaskError::Host(error.into()))?;
                if task.id != record.task_id || task.context_id.as_deref() != Some(record.context_id.as_str()) {
                    return Err(TaskError::Invalid("Caller-tool replay belongs to another task"));
                }
                return Ok(task);
            },
            ClientToolPreparation::Fresh(_) => return Err(TaskError::Invalid("Caller-tool results required")),
        };
        if self.entry(owner, agent, &record.task_id).await.is_err() {
            let actor = self.actors.spawn_a2a_session(owner, &record.task_id, agent, &record.context_id, record.contract.as_ref())
                .await.map_err(TaskError::Host)?;
            let entry = Arc::new(Mutex::new(Entry { tenant_id: owner.tenant_id().map(str::to_owned), actor, durable: Some(record.clone()),
                task: record.task.clone(), turn: None, closed: false, cleanup_pending: false,
                contract: record.contract.clone(), run_id: Some(record.run_id.clone()),
                accounted_run_id: None, usage: UarUsageReceipt::default() }));
            self.bindings.lock().await.tasks.insert((owner.clone(), agent.into(), record.task_id.clone()), entry);
        }
        // The common service has already restored the host transcript and claimed
        // the exact complete batch. This is a fresh root in the same A2A context.
        if !params.message.parts.iter().any(|part| matches!(part, Part::Text { text } if !text.trim().is_empty())) {
            params.message.parts = vec![Part::text("Continue with caller-tool results")];
        }
        let task = self.send_internal(owner, instance, agent, params, capture, Some((record.task_id, digest)), Some(input)).await?;
        client_tools::complete(persistence.as_ref(), &claim, serde_json::to_value(&task)
            .map_err(|error| TaskError::Host(error.into()))?, Vec::new()).await.map_err(TaskError::Host)?;
        Ok(task)
    }

    /// Recover task identity through an exact full-principal binding, then read
    /// the already committed run. A process ID is never takeover authority.
    pub(crate) async fn get_durable(&self, owner: &ActorOwner, agent: &str, id: &str) -> Result<Task, TaskError> {
        let persistence = self.actors.persistence().map_err(TaskError::Host)?;
        let mut record = persistence.load_a2a_binding(&durable_owner(owner)?, agent, id)
            .await.map_err(TaskError::Host)?.ok_or(TaskError::NotFound)?;
        if record.cancellation_requested && !record.context_closed() {
            if let Some(reconciled) = persistence.reconcile_a2a_cancellation(&record.owner_key,owner.user_id(),agent,id)
                .await.map_err(TaskError::Host)? { record = reconciled; }
        }
        let local = match self.entry(owner, agent, id).await {
            Ok(entry) => entry.lock().await.run_id.as_deref() == Some(record.run_id.as_str()),
            Err(_) => false,
        };
        let mut task = if local && !record.context_closed() { self.get(owner, agent, id).await? }
            else { project_durable(persistence.as_ref(), owner, agent, record.clone()).await? };
        let mut pending_approval = false;
        if let Some(state) = &self.run_api {
            let (_, views) = state.manager.approval_records_for_owner(owner, &record.run_id, None).await.map_err(TaskError::Host)?;
            let pending = views.iter().any(|view| view.record.state == crate::uar::persistence::approval_decisions::ApprovalState::Pending
                && (view.resolvable || view.recovery_available));
            if views.iter().any(|view| view.record.state == crate::uar::persistence::approval_decisions::ApprovalState::Denied) {
                task.metadata.insert("uar_denied".into(), serde_json::json!(true));
            }
            if pending && !record.cancellation_requested {
                pending_approval = true;
                task.status.state = TaskState::InputRequired;
            }
            task.metadata.insert("uar.approvals".into(), serde_json::to_value(views).map_err(|error| TaskError::Host(error.into()))?);
        }
        if local && !record.context_closed() {
            // Serialize approval metadata with mailbox projection and new-turn
            // admission. An out-of-lock projection otherwise leaves Entry's
            // expected revision stale and can race the caller-tool claim.
            let entry = self.entry(owner, agent, id).await?;
            let mut entry = entry.lock().await;
            if entry.run_id.as_deref() != Some(record.run_id.as_str()) { return Err(TaskError::Conflict); }
            entry.refresh();
            if let Some(approvals) = task.metadata.get("uar.approvals") {
                entry.task.metadata.insert("uar.approvals".into(), approvals.clone());
            }
            if let Some(denied) = task.metadata.get("uar_denied") {
                entry.task.metadata.insert("uar_denied".into(), denied.clone());
            }
            // Pending approval is a distinct live interaction from a completed
            // caller-tool handoff; retain its existing projection semantics.
            if pending_approval {
                entry.task.status.state = TaskState::InputRequired;
            }
            self.refresh_governed_metadata(&mut entry);
            self.persist_projection(&mut entry).await?;
            return Ok(entry.task.clone());
        }
        // The local get already projects caller-tool handoff after consuming
        // the mailbox completion. Do not override its still-working snapshot
        // merely because the execution producer has finished first.
        if !local && !record.cancellation_requested && persistence.supports_durable_client_tools()
            && let Some(batch) = persistence.load_client_tool_continuation_for_run(
                &record.owner_key, owner.tenant_id(), agent, &record.run_id,
            ).await.map_err(TaskError::Host)?
            && batch.state == crate::uar::persistence::client_tools::ClientToolContinuationState::Pending {
            project_settled_client_tools(&mut task, self.actors.run_manager().as_ref(), persistence.as_ref(), &record.run_id, &batch).await?;
        }
        let current = persistence.load_a2a_binding(&record.owner_key, agent, id).await.map_err(TaskError::Host)?.ok_or(TaskError::NotFound)?;
        if current.run_id != record.run_id { return Err(TaskError::Conflict); }
        if serde_json::to_value(&task).map_err(|error| TaskError::Host(error.into()))?
            != serde_json::to_value(&current.task).map_err(|error| TaskError::Host(error.into()))? {
            let mut after = current.clone();
            after.revision = after.revision.checked_add(1).ok_or(TaskError::Conflict)?;
            after.task = task.clone();
            // A lost CAS means another host projection or continuation won.
            // It never authorizes releasing the newer run's context slot.
            if !persistence.compare_and_swap_a2a_binding(&current, &after).await.map_err(TaskError::Host)? {
                return Err(TaskError::Conflict);
            }
        }
        Ok(task)
    }

    /// Produce task snapshots from retained event notifications. Registration
    /// precedes producer execution; receiver EOF and lifecycle cancellation
    /// settle the producer without claiming the remote task was canceled.
    pub(crate) async fn subscribe_durable(
        self: &Arc<Self>, owner: &ActorOwner, agent: &str, initial: &Task,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<Task, TaskError>>, TaskError> {
        let manager = Arc::clone(self.actors.run_manager());
        let run_id = initial.metadata.get("run_id").and_then(serde_json::Value::as_str).map(str::to_owned);
        let mut registrations = manager.subscribe_run_registrations();
        let mut committed_work = manager.subscribe_committed_work();
        let mut run_events = match run_id.as_deref() {
            Some(run_id) => manager.subscribe(run_id).await,
            None => None,
        };
        let scope = manager.lifecycle_scope();
        let cancellation = scope.cancellation_token();
        let service = Arc::clone(self);
        let owner = owner.clone();
        let agent = agent.to_owned();
        let id = initial.id.clone();
        let (sender, receiver) = tokio::sync::mpsc::channel(16);
        scope.spawn(format!("a2a-stream:{id}"), async move {
            let mut previous = None;
            loop {
                if cancellation.is_cancelled() || sender.is_closed() { return Ok(()); }
                // This projection can commit a durable CAS. Retain its result
                // through cancellation rather than abandoning an accepted write.
                let task = match service.get_durable(&owner, &agent, &id).await {
                    Ok(task) => task,
                    Err(error) => { let _ = sender.try_send(Err(error)); return Ok(()); }
                };
                let active = matches!(task.status.state, TaskState::Submitted | TaskState::Working)
                    && task.metadata.get("execution_unconfirmed") != Some(&serde_json::Value::Bool(true));
                let snapshot = serde_json::to_value(&task)?;
                if previous.as_ref() != Some(&snapshot) {
                    tokio::select! {
                        biased;
                        () = cancellation.cancelled() => return Ok(()),
                        result = sender.send(Ok(task)) => if result.is_err() { return Ok(()); },
                    }
                    previous = Some(snapshot);
                }
                if !active { return Ok(()); }
                if run_events.is_none() && let Some(run_id) = run_id.as_deref() {
                    run_events = manager.subscribe(run_id).await;
                }
                let event = tokio::select! {
                    biased;
                    () = cancellation.cancelled() => {
                        let _ = sender.try_send(Err(TaskError::Conflict));
                        return Ok(());
                    },
                    () = sender.closed() => return Ok(()),
                    event = async {
                        match &mut run_events {
                            Some(events) => events.recv().await.map(|_| ()),
                            None => std::future::pending::<Result<(), tokio::sync::broadcast::error::RecvError>>().await,
                        }
                    } => event.map_err(|_| TaskError::Conflict),
                    changed = registrations.changed() => changed.map_err(|_| TaskError::Conflict),
                    changed = committed_work.changed() => changed.map_err(|_| TaskError::Conflict),
                };
                if let Err(error) = event {
                    // Lag is loss of event continuity, never successful stop.
                    let _ = sender.try_send(Err(error));
                    return Ok(());
                }
            }
        }).map_err(|error| TaskError::Host(error.into()))?;
        Ok(receiver)
    }

    /// Wait for a durable task through retained run and committed-work events.
    /// The task row remains authoritative; each wakeup causes one fresh read.
    /// A receiver is installed before that read, so a transition racing setup
    /// leaves a retained event or changed generation instead of requiring a
    /// fixed-rate status poll.
    pub(crate) async fn wait_durable_until_settled(
        &self,
        owner: &ActorOwner,
        agent: &str,
        id: &str,
        initial: &Task,
    ) -> Result<Task, TaskError> {
        let manager = Arc::clone(self.actors.run_manager());
        let run_id = initial
            .metadata
            .get("run_id")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned);
        let mut registrations = manager.subscribe_run_registrations();
        let mut committed_work = manager.subscribe_committed_work();
        let mut run_events = match run_id.as_deref() {
            Some(run_id) => manager.subscribe(run_id).await,
            None => None,
        };
        loop {
            let task = self.get_durable(owner, agent, id).await?;
            let active = matches!(task.status.state, TaskState::Submitted | TaskState::Working)
                && task.metadata.get("execution_unconfirmed")
                    != Some(&serde_json::Value::Bool(true));
            if !active {
                return Ok(task);
            }
            if run_events.is_none() && let Some(run_id) = run_id.as_deref() {
                run_events = manager.subscribe(run_id).await;
            }
            tokio::select! {
                event = async {
                    match &mut run_events {
                        Some(events) => events.recv().await.map(|_| ()),
                        None => std::future::pending::<Result<(), tokio::sync::broadcast::error::RecvError>>().await,
                    }
                } => match event {
                    Ok(()) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {},
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => return Err(TaskError::Conflict),
                },
                changed = registrations.changed() => {
                    if changed.is_err() { return Err(TaskError::Conflict); }
                },
                changed = committed_work.changed() => {
                    if changed.is_err() {
                        return Err(TaskError::Conflict);
                    }
                },
            }
        }
    }

    pub(crate) async fn cancel_durable(&self, owner: &ActorOwner, agent: &str, id: &str) -> Result<Task, TaskError> {
        let persistence = self.actors.persistence().map_err(TaskError::Host)?;
        let task = self.get_durable(owner, agent, id).await?;
        let record = persistence.load_a2a_binding(&durable_owner(owner)?, agent, id)
            .await.map_err(TaskError::Host)?.ok_or(TaskError::NotFound)?;
        if matches!(task.status.state, TaskState::Completed | TaskState::Canceled | TaskState::Failed) {
            return Err(TaskError::Conflict);
        }
        let mut next = record.clone();
        next.revision = next.revision.checked_add(1).ok_or(TaskError::Conflict)?;
        next.cancellation_requested = true;
        if !persistence.compare_and_swap_a2a_binding(&record, &next).await.map_err(TaskError::Host)? {
            return Err(TaskError::Conflict);
        }
        self.actors.run_manager().notify_committed_work();
        // The owning completion observer consumes this durable intent and joins
        // the exact actor. A disconnected caller does not retract cancellation.
        // A nonlocal producer without a completed exclusion receipt remains a
        // pending intent; do not hold an HTTP request indefinitely or claim join.
        let local_actor = self.entry(owner,agent,id).await.is_ok();
        let local_run = if let Some(state) = &self.run_api {
            state.manager.get_run_for_owner(owner,&record.run_id).await.is_some()
        } else { false };
        let manager = Arc::clone(self.actors.run_manager());
        let mut run_events = manager.subscribe(&record.run_id).await;
        let mut committed_work = manager.subscribe_committed_work();
        loop {
            let task = self.get_durable(owner, agent, id).await?;
            match task.status.state {
                TaskState::Canceled if !task.cleanup_unconfirmed() => return Ok(task),
                TaskState::Completed | TaskState::Failed => return Err(TaskError::Conflict),
                _ if (!local_actor && !local_run)
                    || task.metadata.get("execution_unconfirmed") == Some(&serde_json::Value::Bool(true)) => {
                    return Err(TaskError::Conflict);
                }
                _ => {
                    tokio::select! {
                        event = async {
                            match &mut run_events {
                                Some(events) => events.recv().await.map(|_| ()),
                                None => std::future::pending::<Result<(), tokio::sync::broadcast::error::RecvError>>().await,
                            }
                        } => match event {
                            Ok(()) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {},
                            Err(tokio::sync::broadcast::error::RecvError::Closed) => return Err(TaskError::Conflict),
                        },
                        changed = committed_work.changed() => {
                            if changed.is_err() { return Err(TaskError::Conflict); }
                        },
                    }
                }
            }
        }
    }

    pub(crate) async fn list_durable(&self, owner: &ActorOwner, agent: &str) -> Result<Vec<Task>, TaskError> {
        let records = self.actors.persistence().map_err(TaskError::Host)?
            .list_a2a_bindings(&durable_owner(owner)?, agent).await.map_err(TaskError::Host)?;
        let mut tasks = Vec::with_capacity(records.len());
        for record in records { tasks.push(self.get_durable(owner, agent, &record.task_id).await?); }
        Ok(tasks)
    }
}

/// A durable pending row precedes producer cleanup; it is not yet a handoff.
async fn project_settled_client_tools(
    task: &mut Task, manager: &crate::uar::runtime::manager::RunManager,
    persistence: &dyn crate::uar::persistence::PersistenceLayer, run_id: &str,
    batch: &crate::uar::persistence::client_tools::ClientToolContinuation,
) -> Result<(), TaskError> {
    use crate::uar::domain::runs::RunStatus;
    let status = if manager.get_run(run_id).await.is_some() {
        manager.await_settled_run_status(run_id).await.map_err(TaskError::Host)?
    } else {
        // A cold process has no producer to join. Only the persisted settlement
        // acknowledgment below can establish that a predecessor already joined.
        RunStatus::Done
    };
    match status {
        RunStatus::Done => {
            crate::uar::api::client_tools::confirm_handoff_for_run(manager, persistence,
                &batch.owner_key, batch.tenant_id.as_deref(), &batch.agent_id, run_id)
                .await.map_err(TaskError::Host)?;
            // Mixed turns append hidden host-tool results to the canonical
            // transcript. Only the visible assistant record contains exactly
            // the caller-owned calls that this handoff may disclose.
            let calls = batch.visible_transcript.as_array().and_then(|history| history.last())
                .filter(|message| message.get("role").and_then(serde_json::Value::as_str) == Some("assistant"))
                .and_then(|message| message.get("tool_calls"))
                .and_then(serde_json::Value::as_array)
                .ok_or_else(|| TaskError::Host(anyhow::anyhow!("A2A public caller-tool handoff is malformed")))?;
            let ids = calls.iter().map(|call| call.get("id").and_then(serde_json::Value::as_str))
                .collect::<Option<std::collections::BTreeSet<_>>>();
            let expected = batch.pending_call_ids.iter().map(String::as_str).collect::<std::collections::BTreeSet<_>>();
            if ids.as_ref() != Some(&expected) || calls.len() != expected.len() {
                return Err(TaskError::Host(anyhow::anyhow!("A2A public caller-tool handoff IDs do not match the durable batch")));
            }
            task.status.state = TaskState::InputRequired;
            task.metadata.insert(super::client_tools::CLIENT_TOOLS_EXTENSION.into(), serde_json::json!({
                "batch_id": batch.batch_id, "pending_calls": calls,
                "history": batch.visible_transcript, "declarations": batch.client_schemas,
            }));
        }
        RunStatus::Error => {
            task.status.state = TaskState::Failed;
            task.metadata.remove(super::client_tools::CLIENT_TOOLS_EXTENSION);
        }
        RunStatus::Cancelled => {
            task.status.state = TaskState::Canceled;
            task.metadata.remove(super::client_tools::CLIENT_TOOLS_EXTENSION);
        }
        _ => return Err(TaskError::Conflict),
    }
    Ok(())
}

async fn persist_entry_projection(
    persistence: &dyn crate::uar::persistence::PersistenceLayer,
    manager: &crate::uar::runtime::manager::RunManager, entry: &mut Entry,
) -> Result<(), TaskError> {
    let Some(before) = entry.durable.clone() else { return Ok(()); };
    let current = persistence.load_a2a_binding(&before.owner_key, &before.agent_id, &before.task_id)
        .await.map_err(TaskError::Host)?.ok_or(TaskError::NotFound)?;
    if current.run_id != before.run_id || current.process_id != before.process_id { return Err(TaskError::Conflict); }
    for artifact in &current.task.artifacts {
        if !entry.task.artifacts.iter().any(|existing| existing.artifact_id == artifact.artifact_id) {
            entry.task.artifacts.push(artifact.clone());
        }
    }
    entry.durable = Some(current.clone());
    if !current.cancellation_requested && persistence.supports_durable_client_tools()
        && let Some(batch) = persistence.load_client_tool_continuation_for_run(
            &current.owner_key, entry.tenant_id.as_deref(), &current.agent_id, &current.run_id,
        ).await.map_err(TaskError::Host)?
        && batch.state == crate::uar::persistence::client_tools::ClientToolContinuationState::Pending {
        if entry.turn.is_none() {
            project_settled_client_tools(&mut entry.task, manager, persistence, &current.run_id, &batch).await?;
        } else {
            // A pending durable batch proves the model yielded, not that the
            // actor mailbox has released this turn for another prompt.
            entry.task.status.state = TaskState::Working;
            entry.task.metadata.remove(super::client_tools::CLIENT_TOOLS_EXTENSION);
        }
    }
    let before = &current;
    if serde_json::to_value(&before.task).map_err(|error| TaskError::Host(error.into()))?
        == serde_json::to_value(&entry.task).map_err(|error| TaskError::Host(error.into()))? {
        return Ok(());
    }
    let mut after = before.clone();
    after.revision = after.revision.checked_add(1).ok_or(TaskError::Conflict)?;
    after.task = entry.task.clone();
    if !persistence.compare_and_swap_a2a_binding(before, &after).await.map_err(TaskError::Host)? {
        return Err(TaskError::Conflict);
    }
    entry.durable = Some(after);
    Ok(())
}

impl A2AThreadService {
    async fn decide_durable(
        &self, owner: &ActorOwner, user: &crate::uar::security::claims::UserContext, agent: &str,
        binding: super::durable::A2aBinding, params: MessageSendParams, digest: String,
        capture: AuthenticatedCredentialCapture,
    ) -> Result<Task, TaskError> {
        if binding.cancellation_requested { return Err(TaskError::Conflict); }
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Decision { id: String, approved: bool }
        let decision: Decision = serde_json::from_value(params.metadata.get("uar.approval").cloned()
            .ok_or(TaskError::Invalid("Approval decision is required"))?)
            .map_err(|_| TaskError::Invalid("Approval requires id and approved"))?;
        if params.task_id.as_deref() != Some(binding.task_id.as_str())
            || params.context_id.as_ref().is_some_and(|context| context != &binding.context_id) {
            return Err(TaskError::Invalid("Approval requires the exact task and context"));
        }
        let fresh = params.metadata.get(UAR_DELEGATION_CONTRACT_METADATA)
            .map(|value| serde_json::from_value::<UarDelegationContract>(value.clone()))
            .transpose().map_err(|_| TaskError::Invalid("UAR delegation contract is malformed"))?;
        if fresh != binding.contract { return Err(TaskError::Invalid("Approval requires its current peer delegation contract")); }
        if let Some(contract) = &fresh {
            contract.validate().map_err(|_| TaskError::Invalid("UAR delegation contract is invalid"))?;
            if contract.owner_id != owner.user_id() || contract.target_agent_id != agent
                || user.claims.uar_instance_id.as_deref() != Some(contract.source_instance_id.as_str())
                || self.instance_id.as_deref() != Some(contract.target_instance_id.as_str()) {
                return Err(TaskError::Invalid("Approval delegation does not match the authenticated peer"));
            }
        }
        let state = self.run_api.as_ref().ok_or_else(|| TaskError::Host(anyhow::anyhow!("Approval host unavailable")))?;
        let source_run = binding.task.metadata.get("uar.approval_sources")
            .and_then(|sources| sources.get(&decision.id)).and_then(serde_json::Value::as_str)
            .unwrap_or(&binding.run_id).to_owned();
        let (_, views) = state.manager.approval_records_for_context(user, &source_run, None).await.map_err(TaskError::Host)?;
        let view = views.into_iter().find(|view| view.record.challenge_id == decision.id).ok_or(TaskError::NotFound)?;
        if decision.approved && view.record.state == crate::uar::persistence::approval_decisions::ApprovalState::Pending
            && !view.resolvable {
            if source_run != binding.run_id {
                if binding.request_digest == digest { return self.get_durable(owner, agent, &binding.task_id).await; }
                return Err(TaskError::Conflict);
            }
            return self.resume_durable(owner, user, agent, binding, params, digest, capture, Some(view.record)).await;
        }
        crate::uar::api::routes::resolve_host_approval(Arc::clone(state), user.clone(), Some(capture),
            source_run, decision.id, decision.approved, view.record.workspace_id,
            crate::uar::api::routes::ResumeRequest::for_approval(), None, fresh).await
            .map_err(|_| TaskError::Conflict)?;
        if !decision.approved {
            let persistence = self.actors.persistence().map_err(TaskError::Host)?;
            let before = persistence.load_a2a_binding(&binding.owner_key, agent, &binding.task_id).await
                .map_err(TaskError::Host)?.ok_or(TaskError::NotFound)?;
            let mut after = before.clone();
            after.revision = after.revision.checked_add(1).ok_or(TaskError::Conflict)?;
            after.cancellation_requested = true;
            after.task.metadata.insert("uar_denied".into(), serde_json::json!(true));
            if !persistence.compare_and_swap_a2a_binding(&before, &after).await.map_err(TaskError::Host)? {
                return Err(TaskError::Conflict);
            }
        }
        self.get_durable(owner, agent, &binding.task_id).await
    }

    async fn resume_durable(
        &self, owner: &ActorOwner, user: &crate::uar::security::claims::UserContext, agent: &str,
        before: super::durable::A2aBinding, params: MessageSendParams, digest: String,
        capture: AuthenticatedCredentialCapture,
        approval: Option<crate::uar::persistence::approval_decisions::ApprovalRecord>,
    ) -> Result<Task, TaskError> {
        if before.cancellation_requested { return Err(TaskError::Conflict); }
        let state = self.run_api.as_ref().ok_or_else(|| TaskError::Host(anyhow::anyhow!("A2A host resume is unavailable")))?;
        let fresh_contract = params.metadata.get(UAR_DELEGATION_CONTRACT_METADATA)
            .map(|value| serde_json::from_value::<UarDelegationContract>(value.clone()))
            .transpose().map_err(|_| TaskError::Invalid("UAR delegation contract is malformed"))?;
        if fresh_contract != before.contract {
            return Err(TaskError::Invalid("Continuation requires its current peer delegation contract"));
        }
        if let Some(contract) = &fresh_contract {
            contract.validate().map_err(|_| TaskError::Invalid("UAR delegation contract is invalid"))?;
            if contract.owner_id != owner.user_id()
                || user.claims.uar_instance_id.as_deref() != Some(contract.source_instance_id.as_str())
                || self.instance_id.as_deref() != Some(contract.target_instance_id.as_str())
                || contract.target_agent_id != agent {
                return Err(TaskError::Invalid("Continuation delegation does not match the current authenticated peer"));
            }
        }
        if params.context_id.as_ref().is_some_and(|context| context != &before.context_id) {
            return Err(TaskError::Invalid("task and context do not match"));
        }
        let content = params.message.parts.iter().filter_map(|part| match part {
            Part::Text { text } => Some(text.as_str()), _ => None,
        }).collect::<Vec<_>>().join("\n");
        if approval.is_none() && content.trim().is_empty() { return Err(TaskError::Invalid("agent input must not be empty")); }
        if let Some(finding) = crate::uar::guardrails::screen_input(&content, &self.guardrails) {
            let blocked = match finding.category {
                crate::uar::guardrails::GuardrailCategory::Injection => self.guardrails.block_on_injection,
                crate::uar::guardrails::GuardrailCategory::Pii => self.guardrails.block_on_pii,
            };
            if blocked { return Err(TaskError::Invalid("Input rejected by guardrail policy")); }
        }
        let persistence = self.actors.persistence().map_err(TaskError::Host)?;
        let mut envelopes = persistence.list_checkpoint_resumes(&before.owner_key, &before.run_id)
            .await.map_err(TaskError::Host)?;
        let envelope = if let Some(record) = &approval {
            let checkpoint = &record.continuation.as_ref().ok_or(TaskError::Conflict)?.checkpoint_id;
            envelopes.into_iter().find(|envelope| &envelope.checkpoint_id == checkpoint).ok_or(TaskError::Conflict)?
        } else { envelopes.pop().ok_or(TaskError::Conflict)? };
        let source = envelope.source_run().map_err(TaskError::Host)?;
        if source.agent_id != agent { return Err(TaskError::NotFound); }
        let mut next = before.clone();
        next.revision = next.revision.checked_add(1).ok_or(TaskError::Conflict)?;
        next.run_id = uuid::Uuid::new_v4().to_string();
        next.process_id = self.process_id.clone();
        next.runtime_epoch = Some(state.manager.runtime_epoch().to_owned());
        next.request_digest = digest;
        next.cancellation_requested = false;
        next.task.history.push(params.message);
        next.task.status.state = TaskState::Submitted;
        next.task.status.message = None;
        next.task.metadata.insert("run_id".into(), serde_json::json!(next.run_id));
        if let Some(record) = &approval {
            let sources = next.task.metadata.entry("uar.approval_sources".into()).or_insert_with(|| serde_json::json!({}));
            if let Some(sources) = sources.as_object_mut() {
                sources.insert(record.challenge_id.clone(), serde_json::json!(before.run_id));
            }
        }
        if !persistence.compare_and_swap_a2a_binding(&before, &next).await.map_err(TaskError::Host)? {
            return Err(TaskError::Conflict);
        }
        // Correlation is durable before execution. A canceled admission waiter
        // leaves an exact reserved identity, never a replayable task promise.
        let observed = next.clone();
        let observer_store = Arc::clone(&persistence);
        let observer_owner = owner.clone();
        let observer_manager = Arc::clone(&state.manager);
        let scope = observer_manager.lifecycle_scope();
        let events = observer_manager.subscribe(&observed.run_id).await;
        let observed_run_id = observed.run_id.clone();
        let cancellation = scope.cancellation_token();
        if events.is_none() {
            scope.record_unresolved_effect(format!("a2a-continuation-projection:{observed_run_id}"));
        } else if scope.spawn("a2a-continuation-projection", async move {
            let mut events = events.expect("checked above");
            let mut committed_work = observer_manager.subscribe_committed_work();
            let mut first_snapshot = true;
            loop {
                if !first_snapshot {
                    tokio::select! {
                        _ = cancellation.cancelled() => return Ok(()),
                        event = events.recv() => match event {
                            Ok(_) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                                return Err(anyhow::anyhow!("A2A continuation lost its run event source"));
                            }
                        },
                        changed = committed_work.changed() => {
                            if changed.is_err() {
                                return Err(anyhow::anyhow!("A2A continuation lost its committed-work source"));
                            }
                        },
                    }
                }
                first_snapshot = false;
                let current = match observer_store.load_a2a_binding(&observed.owner_key, &observed.agent_id, &observed.task_id).await {
                    Ok(Some(current)) if current.run_id == observed.run_id => current,
                    _ => return Ok(()),
                };
                if current.cancellation_requested {
                    observer_manager.cancel_run_for_owner(&observer_owner, &current.run_id).await;
                }
                let task = match project_durable(observer_store.as_ref(), &observer_owner, &current.agent_id, current.clone()).await {
                    Ok(task) => task,
                    Err(error) => return Err(anyhow::anyhow!("A2A continuation projection failed: {error}")),
                };
                if matches!(task.status.state, TaskState::Completed | TaskState::Failed | TaskState::Canceled | TaskState::InputRequired) {
                    let mut updated = current.clone();
                    let Some(revision) = updated.revision.checked_add(1) else {
                        return Err(anyhow::anyhow!("A2A continuation revision overflow"));
                    };
                    updated.revision = revision;
                    updated.task = task;
                    match observer_store.compare_and_swap_a2a_binding(&current, &updated).await {
                        Ok(true) => return Ok(()),
                        Ok(false) => continue,
                        Err(error) => return Err(anyhow::anyhow!("A2A continuation projection failed: {error}")),
                    }
                }
            }
        }).is_err() {
            scope.record_unresolved_effect(format!("a2a-continuation-projection:{observed_run_id}"));
        }
        let resumed = if let Some(approval) = approval {
            crate::uar::api::routes::resolve_host_approval(Arc::clone(state), user.clone(), Some(capture),
                before.run_id, approval.challenge_id, true, approval.workspace_id,
                crate::uar::api::routes::ResumeRequest::for_approval(), Some(next.run_id.clone()), fresh_contract).await
                .map(|resolution| crate::uar::api::routes::CheckpointResumeReceipt { run_id: resolution.run_id, response: serde_json::json!({}) })
        } else {
            crate::uar::api::routes::resume_checkpoint_admitted(
                Arc::clone(state), user.clone(), Some(axum::Extension(capture)), before.run_id,
                envelope.checkpoint_id, crate::uar::api::routes::ResumeRequest::for_input(content),
                None, Some(next.run_id.clone()), fresh_contract,
            ).await
        };
        match resumed {
            Ok(receipt) if receipt.run_id == next.run_id => self.get_durable(owner, agent, &next.task_id).await,
            Ok(_) => Err(TaskError::Host(anyhow::anyhow!("A2A continuation correlation mismatch"))),
            Err(response) => {
                tracing::warn!(status = %response.status(), "A2A protected continuation was not confirmed");
                Err(TaskError::Conflict)
            }
        }
    }
}

async fn project_durable(
    persistence: &dyn crate::uar::persistence::PersistenceLayer, owner: &ActorOwner, agent: &str,
    record: super::durable::A2aBinding,
) -> Result<Task, TaskError> {
        let mut task = record.task;
        if task.status.state == TaskState::Canceled
            && task.metadata.get("uar.cancellation_excluded_epoch").is_some() { return Ok(task); }
        let roots = persistence.list_agent_threads(owner.user_id(), &record.run_id)
            .await.map_err(TaskError::Host)?;
        if let Some(root) = roots.into_iter().find(|root| root.thread.parent_thread_id.is_none()
            && root.thread.artifact_id == agent && root.thread.run_id.as_deref() == Some(record.run_id.as_str())) {
            task.metadata.remove("execution_unconfirmed");
            task.metadata.insert("thread_id".into(), serde_json::json!(root.thread.thread_id));
            task.metadata.insert("run_id".into(), serde_json::json!(record.run_id));
            task.status.timestamp = Some(root.thread.updated_at.to_rfc3339());
            match root.thread.result {
                Some(AgentThreadResult::Completed { output }) => {
                    task.status.state = TaskState::Completed;
                    task.status.message = Some(Message::agent_text(output.clone()));
                    if !task.history.last().is_some_and(|message| message.role == Role::Agent) {
                        task.history.push(Message::agent_text(output));
                    }
                }
                Some(AgentThreadResult::Yielded { .. }) => task.status.state = TaskState::InputRequired,
                Some(AgentThreadResult::Cancelled) => task.status.state = TaskState::Canceled,
                Some(AgentThreadResult::Failed { code, .. }) => {
                    if code.ends_with("persistence_unconfirmed") {
                        task.metadata.insert("context_unconfirmed".into(), serde_json::json!(true));
                    }
                    task.status.state = TaskState::Failed;
                    if code.contains("cleanup_unconfirmed") {
                        task.metadata.insert("cleanup_unconfirmed".into(), serde_json::json!(true));
                    }
                }
                None => {
                    task.status.state = if root.thread.status == crate::uar::runtime::thread::AgentThreadStatus::Waiting {
                        TaskState::InputRequired
                    } else { TaskState::Working };
                }
            }
        } else {
            task.metadata.insert("execution_unconfirmed".into(), serde_json::json!(true));
        }
        Ok(task)
}
