//! A2A task projection over the existing persisted-thread mailbox host.
//! Task IDs/context IDs are transport correlation, never kernel authority.

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
            self.task.status.state = TaskState::Working;
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
                    self.task.artifacts.extend(artifacts.iter().map(|artifact| {
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
                    Some(AgentThreadResult::Cancelled) => (TaskState::Canceled, None),
                    Some(AgentThreadResult::Failed { code, .. }) => {
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
        }
    }

    #[must_use]
    pub fn with_instance_id(mut self, instance_id: impl Into<String>) -> Self {
        let instance_id = instance_id.into();
        self.instance_id = (!instance_id.trim().is_empty()).then_some(instance_id);
        self
    }

    pub async fn send(
        &self,
        owner: &ActorOwner,
        authenticated_instance_id: Option<&str>,
        agent_id: &str,
        params: MessageSendParams,
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
                    let id = uuid::Uuid::new_v4().to_string();
                    let actor_name = format!("a2a-{id}");
                    let actor = match &contract {
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
                    }
                    .map_err(TaskError::Host)?;
                    let context_id = params.context_id.clone().unwrap_or_else(|| id.clone());
                    let entry = Arc::new(Mutex::new(Entry {
                        actor,
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
        let turn = entry
            .actor
            .submit_prompt(content)
            .map_err(TaskError::Host)?;
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
        Ok(entry.task.clone())
    }
}
