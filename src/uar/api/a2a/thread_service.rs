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
        self.send_internal(owner, authenticated_instance_id, agent_id, params, credential_capture, None).await
    }

    async fn send_internal(
        &self,
        owner: &ActorOwner,
        authenticated_instance_id: Option<&str>,
        agent_id: &str,
        params: MessageSendParams,
        credential_capture: AuthenticatedCredentialCapture,
        reservation: Option<(String, String)>,
    ) -> Result<Task, TaskError> {
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
                let before = record.clone();
                record.revision = record.revision.checked_add(1).ok_or(TaskError::Conflict)?;
                record.run_id = uuid::Uuid::new_v4().to_string();
                record.cancellation_requested = false;
                record.request_digest = reservation.as_ref().ok_or(TaskError::Conflict)?.1.clone();
                record.task = entry.task.clone();
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
            entry.actor.submit_authorized_prompt(owner, record.run_id, content, credential_capture)
        } else {
            entry.actor.submit_authorized_prompt(
                owner,
                uuid::Uuid::new_v4().to_string(),
                content,
                credential_capture,
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
        let task = entry.task.clone();
        if entry.durable.is_some() {
            let persistence = self.actors.persistence().map_err(TaskError::Host)?;
            // This observer retains completion correlation after an HTTP caller
            // disconnects. It never submits or executes a run.
            let observed_run = entry.run_id.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    let mut entry = entry_handle.lock().await;
                    if entry.run_id != observed_run { break; }
                    if let Some(before) = &entry.durable {
                        match persistence.load_a2a_binding(&before.owner_key, &before.agent_id, &before.task_id).await {
                            Ok(Some(current)) if current.run_id == before.run_id && current.process_id == before.process_id => {
                                entry.durable = Some(current);
                            }
                            _ => break,
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
                    if let Err(error) = persist_entry_projection(persistence.as_ref(), &mut entry).await {
                        if matches!(error, TaskError::Conflict) { continue; }
                        tracing::error!(%error, "A2A durable completion projection failed");
                        break;
                    }
                    if settled { break; }
                }
            });
        }
        self.persist_projection(&mut entry).await?;
        Ok(task)
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
        persist_entry_projection(self.actors.persistence().map_err(TaskError::Host)?.as_ref(), entry).await
    }

    /// A v1 request reserves durable correlation before the existing actor owns
    /// its mailbox turn. A retried message cannot enqueue a second invocation.
    pub(crate) async fn send_durable(
        &self, owner: &ActorOwner, user: &crate::uar::security::claims::UserContext,
        instance: Option<&str>, agent: &str,
        params: MessageSendParams, message_id: &str, capture: AuthenticatedCredentialCapture,
    ) -> Result<Task, TaskError> {
        use sha2::{Digest, Sha256};
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
        self.send_internal(owner, instance, agent, params, capture, Some((id, digest))).await
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
        if let Some(state) = &self.run_api {
            let (_, views) = state.manager.approval_records_for_owner(owner, &record.run_id, None).await.map_err(TaskError::Host)?;
            let pending = views.iter().any(|view| view.record.state == crate::uar::persistence::approval_decisions::ApprovalState::Pending
                && (view.resolvable || view.recovery_available));
            if views.iter().any(|view| view.record.state == crate::uar::persistence::approval_decisions::ApprovalState::Denied) {
                task.metadata.insert("uar_denied".into(), serde_json::json!(true));
            }
            if pending && !record.cancellation_requested {
                task.status.state = TaskState::InputRequired;
            }
            task.metadata.insert("uar.approvals".into(), serde_json::to_value(views).map_err(|error| TaskError::Host(error.into()))?);
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
        // The owning completion observer consumes this durable intent and joins
        // the exact actor. A disconnected caller does not retract cancellation.
        // A nonlocal producer without a completed exclusion receipt remains a
        // pending intent; do not hold an HTTP request indefinitely or claim join.
        let local_actor = self.entry(owner,agent,id).await.is_ok();
        let local_run = if let Some(state) = &self.run_api {
            state.manager.get_run_for_owner(owner,&record.run_id).await.is_some()
        } else { false };
        loop {
            let task = self.get_durable(owner, agent, id).await?;
            match task.status.state {
                TaskState::Canceled if !task.cleanup_unconfirmed() => return Ok(task),
                TaskState::Completed | TaskState::Failed => return Err(TaskError::Conflict),
                _ if (!local_actor && !local_run)
                    || task.metadata.get("execution_unconfirmed") == Some(&serde_json::Value::Bool(true)) => {
                    return Err(TaskError::Conflict);
                }
                _ => tokio::time::sleep(std::time::Duration::from_millis(100)).await,
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

async fn persist_entry_projection(
    persistence: &dyn crate::uar::persistence::PersistenceLayer, entry: &mut Entry,
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
        tokio::spawn(async move {
            let shutdown = observer_manager.root_cancellation_token();
            loop {
                tokio::select! {
                    _ = shutdown.cancelled() => break,
                    _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {}
                }
                let current = match observer_store.load_a2a_binding(&observed.owner_key, &observed.agent_id, &observed.task_id).await {
                    Ok(Some(current)) if current.run_id == observed.run_id => current,
                    _ => break,
                };
                if current.cancellation_requested {
                    observer_manager.cancel_run_for_owner(&observer_owner, &current.run_id).await;
                }
                let task = match project_durable(observer_store.as_ref(), &observer_owner, &current.agent_id, current.clone()).await {
                    Ok(task) => task, Err(_) => break,
                };
                if matches!(task.status.state, TaskState::Completed | TaskState::Failed | TaskState::Canceled | TaskState::InputRequired) {
                    let mut updated = current.clone();
                    let Some(revision) = updated.revision.checked_add(1) else { break; };
                    updated.revision = revision;
                    updated.task = task;
                    match observer_store.compare_and_swap_a2a_binding(&current, &updated).await {
                        Ok(true) => break,
                        Ok(false) => continue,
                        Err(error) => { tracing::error!(%error, "A2A continuation projection failed"); break; }
                    }
                }
            }
        });
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
