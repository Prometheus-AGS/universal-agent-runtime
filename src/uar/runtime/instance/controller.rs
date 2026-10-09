//! Durable admission and activation over the existing actor/thread execution path.

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use chrono::Utc;
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::uar::{
    compiler::collaboration::{BoundAgentRun, CollaborationCatalogService},
    persistence::{
        PersistenceLayer,
        agent_instances::{
            AGENT_INSTANCE_SCHEMA_VERSION, AgentInstanceCommand, AgentInstanceEvent,
            AgentInstanceRecord, AgentInstanceStoreError, MAX_INSTANCE_COMMAND_BYTES,
        },
    },
    runtime::{
        actor::{
            messages::ActorOwner,
            system::{ActorCollaboration, ActorSession},
        },
        manager::RunManager,
        thread::actor_host::InstanceRootConstraints,
    },
};

use super::{
    AgentInstanceCommandView, AgentInstanceError, AgentInstanceLimits, AgentInstanceView,
    InstanceActivationProfile, InstanceCommandKind, InstanceCommandStatus, InstanceEpochBinding,
    InstanceLifecycle, InstanceRecovery,
};

#[derive(Clone, Hash, PartialEq, Eq)]
pub(super) struct InstanceKey {
    owner_id: String,
    workspace_id: String,
    instance_id: String,
}

impl InstanceKey {
    pub(super) fn new(owner: &ActorOwner, workspace_id: &str, instance_id: &str) -> Self {
        Self {
            owner_id: owner.presentation_owner_key(),
            workspace_id: workspace_id.to_owned(),
            instance_id: instance_id.to_owned(),
        }
    }
}

#[derive(Clone)]
pub(super) struct ActiveActor {
    pub(super) epoch: u64,
    pub(super) session: ActorSession,
}

/// One host-owned control plane for durable instances; the actor owns model turns.
pub struct AgentInstanceController {
    pub(crate) manager: Arc<RunManager>,
    pub(crate) actors: Arc<ActorCollaboration>,
    pub(crate) store: Arc<dyn PersistenceLayer>,
    pub(crate) catalog: Arc<CollaborationCatalogService>,
    booted_at: chrono::DateTime<Utc>,
    pub(super) active: Mutex<HashMap<InstanceKey, ActiveActor>>,
    pub(super) pumps: Mutex<HashSet<InstanceKey>>,
}

impl std::fmt::Debug for AgentInstanceController {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AgentInstanceController")
            .finish_non_exhaustive()
    }
}

impl AgentInstanceController {
    #[must_use]
    pub fn new(
        manager: Arc<RunManager>,
        actors: Arc<ActorCollaboration>,
        store: Arc<dyn PersistenceLayer>,
        catalog: Arc<CollaborationCatalogService>,
    ) -> Arc<Self> {
        Arc::new(Self {
            manager,
            actors,
            store,
            catalog,
            booted_at: Utc::now(),
            active: Mutex::new(HashMap::new()),
            pumps: Mutex::new(HashSet::new()),
        })
    }

    pub async fn create(
        self: &Arc<Self>,
        owner: &ActorOwner,
        workspace: &str,
        binding_id: &str,
        profile: InstanceActivationProfile,
        limits: AgentInstanceLimits,
    ) -> Result<AgentInstanceView, AgentInstanceError> {
        self.require_store()?;
        require_name(workspace)?;
        require_name(binding_id)?;
        let owner_id = owner.presentation_owner_key();
        let bound = self
            .catalog
            .resolve_bound_agent_run(&owner_id, workspace, binding_id)
            .await
            .map_err(|_| {
                AgentInstanceError::Conflict("deployment binding is not currently admitted")
            })?;
        let now = Utc::now();
        let record = AgentInstanceRecord {
            schema_version: AGENT_INSTANCE_SCHEMA_VERSION,
            instance_id: Uuid::new_v4().to_string(),
            owner_id,
            workspace_id: workspace.to_owned(),
            definition: bound.binding.package,
            effective_binding: bound.effective_binding_receipt.binding_ref,
            representation_revision: 0,
            representation_grants: Vec::new(),
            session_id: Uuid::new_v4().to_string(),
            profile,
            lifecycle: InstanceLifecycle::Dormant,
            recovery: InstanceRecovery::Ready,
            limits,
            revision: 0,
            epoch: 0,
            inbox: Vec::new(),
            active_attempt: None,
            events: Vec::new(),
            next_event_sequence: 0,
            newly_appended_events: Vec::new(),
            restart_attempts: 0,
            last_error_code: None,
            reconciliation_receipt: None,
            created_at: now,
            updated_at: now,
        };
        record
            .validate(&record.owner_id, workspace)
            .map_err(|_| AgentInstanceError::Invalid("instance limits are invalid"))?;
        let record = self
            .store
            .create_agent_instance(&record)
            .await
            .map_err(store_error)?;
        if profile == InstanceActivationProfile::Resident {
            return self
                .activate(
                    owner,
                    workspace,
                    &record.instance_id,
                    &Uuid::new_v4().to_string(),
                )
                .await;
        }
        Ok(AgentInstanceView::from(&record))
    }

    pub async fn get(
        &self,
        owner: &ActorOwner,
        workspace: &str,
        id: &str,
    ) -> Result<AgentInstanceView, AgentInstanceError> {
        Ok(AgentInstanceView::from(
            &self.load(owner, workspace, id).await?,
        ))
    }

    pub async fn list(
        &self,
        owner: &ActorOwner,
        workspace: &str,
    ) -> Result<Vec<AgentInstanceView>, AgentInstanceError> {
        self.require_store()?;
        require_name(workspace)?;
        let owner_id = owner.presentation_owner_key();
        let records = self
            .store
            .list_agent_instances(&owner_id, workspace)
            .await
            .map_err(store_error)?;
        let mut views = Vec::with_capacity(records.len());
        for record in records {
            views.push(AgentInstanceView::from(
                &self.load(owner, workspace, &record.instance_id).await?,
            ));
        }
        Ok(views)
    }

    pub async fn submit(
        self: &Arc<Self>,
        owner: &ActorOwner,
        workspace: &str,
        id: &str,
        command_id: &str,
        prompt: &str,
    ) -> Result<AgentInstanceCommandView, AgentInstanceError> {
        require_name(command_id)?;
        if prompt.trim().is_empty() {
            return Err(AgentInstanceError::Invalid("turn prompt is empty"));
        }
        let payload = serde_json::json!({"prompt": prompt});
        let payload_size = serde_json::to_vec(&payload)
            .map_err(|error| AgentInstanceError::Internal(error.into()))?
            .len();
        if payload_size > MAX_INSTANCE_COMMAND_BYTES {
            return Err(AgentInstanceError::Invalid(
                "turn prompt exceeds the durable inbox limit",
            ));
        }
        let digest = sha256_hex(format!("turn\0{prompt}").as_bytes());
        let record = self
            .update(owner, workspace, id, |next| {
                if let Some(existing) = next
                    .inbox
                    .iter()
                    .find(|command| command.command_id == command_id)
                {
                    return if existing.request_digest == digest
                        && existing.kind == InstanceCommandKind::Turn
                    {
                        Ok(false)
                    } else {
                        Err(AgentInstanceError::Conflict(
                            "command ID was already used for another request",
                        ))
                    };
                }
                if !matches!(
                    next.lifecycle,
                    InstanceLifecycle::Dormant | InstanceLifecycle::Active
                ) || next.recovery != InstanceRecovery::Ready
                {
                    return Err(AgentInstanceError::Conflict(
                        "instance is not accepting turns",
                    ));
                }
                if next.inbox.iter().any(|command| {
                    command.kind != InstanceCommandKind::Turn
                        && command.status == InstanceCommandStatus::Accepted
                }) {
                    return Err(AgentInstanceError::Conflict(
                        "instance lifecycle change is pending",
                    ));
                }
                let pending = next
                    .inbox
                    .iter()
                    .filter(|command| {
                        command.kind == InstanceCommandKind::Turn
                            && matches!(
                                command.status,
                                InstanceCommandStatus::Accepted | InstanceCommandStatus::Running
                            )
                    })
                    .count();
                if pending >= next.limits.max_inbox as usize {
                    return Err(AgentInstanceError::Capacity);
                }
                trim_commands(next)?;
                let now = Utc::now();
                next.inbox.push(AgentInstanceCommand {
                    command_id: command_id.to_owned(),
                    request_digest: digest.clone(),
                    kind: InstanceCommandKind::Turn,
                    payload: Some(payload.clone()),
                    payload_ref: None,
                    status: InstanceCommandStatus::Accepted,
                    attempt_id: None,
                    root_run_id: None,
                    accepted_at: now,
                    updated_at: now,
                    outcome: None,
                });
                append_event(next, "turn_accepted", Some(command_id), None, None)?;
                Ok(true)
            })
            .await?;
        let command = record
            .inbox
            .iter()
            .find(|command| command.command_id == command_id)
            .ok_or_else(|| {
                AgentInstanceError::Internal(anyhow::anyhow!("Accepted command was not stored"))
            })?;
        let view = AgentInstanceCommandView::from(command);
        if command.status != InstanceCommandStatus::Accepted
            || record.recovery != InstanceRecovery::Ready
            || !matches!(
                record.lifecycle,
                InstanceLifecycle::Dormant | InstanceLifecycle::Active
            )
        {
            return Ok(view);
        }
        if record.lifecycle == InstanceLifecycle::Dormant
            || !self.has_actor(owner, workspace, id, record.epoch).await
        {
            self.activate(owner, workspace, id, &Uuid::new_v4().to_string())
                .await?;
        }
        self.kick(owner.clone(), workspace, id);
        Ok(view)
    }

    pub(crate) async fn load(
        &self,
        owner: &ActorOwner,
        workspace: &str,
        id: &str,
    ) -> Result<AgentInstanceRecord, AgentInstanceError> {
        self.require_store()?;
        require_name(workspace)?;
        require_name(id)?;
        let owner_id = owner.presentation_owner_key();
        for _ in 0..8 {
            let record = self
                .store
                .load_agent_instance(&owner_id, workspace, id)
                .await
                .map_err(store_error)?
                .ok_or(AgentInstanceError::NotFound)?;
            record.validate(&owner_id, workspace).map_err(|_| {
                AgentInstanceError::Internal(anyhow::anyhow!("Stored instance scope is invalid"))
            })?;
            if record.updated_at >= self.booted_at
                || (record.active_attempt.is_none()
                    && !matches!(
                        record.lifecycle,
                        InstanceLifecycle::Active | InstanceLifecycle::Draining
                    ))
            {
                return Ok(record);
            }
            // An old process-local actor cannot be recovered by a fresh
            // controller. Preserve admitted work and latch any running effect.
            let mut next = record.clone();
            if let Some(attempt) = next.active_attempt.take() {
                if let Some(command) = next
                    .inbox
                    .iter_mut()
                    .find(|command| command.command_id == attempt.command_id)
                {
                    command.status = InstanceCommandStatus::Uncertain;
                    command.updated_at = Utc::now();
                }
                next.recovery = InstanceRecovery::EffectUncertain;
                next.lifecycle = InstanceLifecycle::Failed;
                next.last_error_code = Some("orphaned_attempt".into());
                append_event(
                    &mut next,
                    "orphaned_attempt",
                    Some(&attempt.command_id),
                    Some(&attempt.attempt_id),
                    Some(&attempt.root_run_id),
                )?;
            } else {
                next.lifecycle = InstanceLifecycle::Dormant;
                next.epoch = next
                    .epoch
                    .checked_add(1)
                    .ok_or(AgentInstanceError::Conflict("activation epoch exhausted"))?;
                append_event(&mut next, "activation_lost", None, None, None)?;
            }
            next.revision = record
                .revision
                .checked_add(1)
                .ok_or(AgentInstanceError::Conflict("instance revision exhausted"))?;
            next.updated_at = Utc::now();
            if self
                .store
                .compare_and_swap_agent_instance(
                    &owner_id,
                    workspace,
                    record.revision,
                    record.epoch,
                    &next,
                )
                .await
                .map_err(store_error)?
            {
                return Ok(next);
            }
        }
        Err(AgentInstanceError::Conflict(
            "instance changed during restart reconciliation",
        ))
    }

    pub(crate) async fn update<F>(
        &self,
        owner: &ActorOwner,
        workspace: &str,
        id: &str,
        mut change: F,
    ) -> Result<AgentInstanceRecord, AgentInstanceError>
    where
        F: FnMut(&mut AgentInstanceRecord) -> Result<bool, AgentInstanceError>,
    {
        for _ in 0..8 {
            let current = self.load(owner, workspace, id).await?;
            let mut next = current.clone();
            if !change(&mut next)? {
                return Ok(current);
            }
            next.revision = current
                .revision
                .checked_add(1)
                .ok_or(AgentInstanceError::Conflict("instance revision exhausted"))?;
            next.updated_at = Utc::now();
            let saved = self
                .store
                .compare_and_swap_agent_instance(
                    &current.owner_id,
                    workspace,
                    current.revision,
                    current.epoch,
                    &next,
                )
                .await
                .map_err(store_error)?;
            if saved {
                return Ok(next);
            }
        }
        Err(AgentInstanceError::Conflict(
            "instance changed during concurrent update",
        ))
    }

    pub(crate) async fn bound(
        &self,
        record: &AgentInstanceRecord,
    ) -> Result<BoundAgentRun, AgentInstanceError> {
        let bound = self
            .catalog
            .resolve_bound_agent_run(
                &record.owner_id,
                &record.workspace_id,
                &record.effective_binding.id,
            )
            .await
            .map_err(|_| {
                AgentInstanceError::Conflict("deployment binding is no longer admitted")
            })?;
        if bound.binding.package != record.definition
            || bound.effective_binding_receipt.binding_ref != record.effective_binding
        {
            return Err(AgentInstanceError::Conflict(
                "pinned definition or binding changed",
            ));
        }
        Ok(bound)
    }

    pub(crate) async fn actor(
        &self,
        owner: &ActorOwner,
        record: &AgentInstanceRecord,
    ) -> Result<ActorSession, AgentInstanceError> {
        let key = InstanceKey::new(owner, &record.workspace_id, &record.instance_id);
        let mut active = self.active.lock().await;
        if let Some(existing) = active.get(&key) {
            return if existing.epoch == record.epoch {
                Ok(existing.session.clone())
            } else {
                Err(AgentInstanceError::Conflict(
                    "previous activation has not stopped",
                ))
            };
        }
        let bound = self.bound(record).await?;
        let history = self
            .store
            .load_session(owner.user_id(), &record.session_id)
            .await
            .map_err(store_error)?
            .map(|session| {
                if session.owner_id() != owner.user_id() || session.id() != record.session_id {
                    return Err(AgentInstanceError::Internal(anyhow::anyhow!(
                        "Stored conversation identity changed"
                    )));
                }
                Ok(session.messages())
            })
            .transpose()?
            .unwrap_or_default();
        let instance = InstanceRootConstraints {
            bound,
            owner_key: record.owner_id.clone(),
            workspace_id: record.workspace_id.clone(),
            catalog: Arc::clone(&self.catalog),
            epoch: InstanceEpochBinding::new(
                Arc::clone(&self.store),
                record.owner_id.clone(),
                record.workspace_id.clone(),
                record.instance_id.clone(),
                record.epoch,
                owner.user_id().to_owned(),
                record.representation_revision,
                record.representation_grants.clone(),
            ),
            restored_history: history,
        };
        let actor_name = format!(
            "instance:{}:{}:{}",
            record.workspace_id, record.instance_id, record.epoch
        );
        let session = self
            .actors
            .spawn_instance_session(owner, actor_name, record.session_id.clone(), instance)
            .await
            .map_err(AgentInstanceError::Internal)?;
        active.insert(
            key,
            ActiveActor {
                epoch: record.epoch,
                session: session.clone(),
            },
        );
        Ok(session)
    }

    pub(crate) async fn has_actor(
        &self,
        owner: &ActorOwner,
        workspace: &str,
        id: &str,
        epoch: u64,
    ) -> bool {
        let key = InstanceKey::new(owner, workspace, id);
        self.active
            .lock()
            .await
            .get(&key)
            .is_some_and(|active| active.epoch == epoch)
    }

    pub(crate) async fn stop_actor(
        &self,
        owner: &ActorOwner,
        workspace: &str,
        id: &str,
    ) -> Result<(), AgentInstanceError> {
        let key = InstanceKey::new(owner, workspace, id);
        let active = self.active.lock().await.get(&key).cloned();
        if let Some(active) = active {
            active
                .session
                .stop()
                .await
                .map_err(AgentInstanceError::Internal)?;
            let mut actors = self.active.lock().await;
            if actors
                .get(&key)
                .is_some_and(|current| current.epoch == active.epoch)
            {
                actors.remove(&key);
            }
        }
        Ok(())
    }

    pub(crate) fn require_store(&self) -> Result<(), AgentInstanceError> {
        if self.store.supports_durable_agent_instances() {
            Ok(())
        } else {
            Err(AgentInstanceError::Unavailable)
        }
    }
}

pub(crate) fn require_name(value: &str) -> Result<(), AgentInstanceError> {
    if value.trim().is_empty() || value.len() > 256 {
        Err(AgentInstanceError::Invalid(
            "identifier is empty or too long",
        ))
    } else {
        Ok(())
    }
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut encoded = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    encoded
}

pub(crate) fn store_error(error: anyhow::Error) -> AgentInstanceError {
    match error.downcast_ref::<AgentInstanceStoreError>() {
        Some(AgentInstanceStoreError::Unsupported) => AgentInstanceError::Unavailable,
        Some(AgentInstanceStoreError::NotFound) => AgentInstanceError::NotFound,
        Some(AgentInstanceStoreError::AlreadyExists | AgentInstanceStoreError::Conflict) => {
            AgentInstanceError::Conflict("instance revision changed")
        }
        Some(
            AgentInstanceStoreError::InvalidRecord
            | AgentInstanceStoreError::ImmutableField
            | AgentInstanceStoreError::RevisionExhausted
            | AgentInstanceStoreError::ScopeMismatch,
        )
        | None => AgentInstanceError::Internal(error),
    }
}

pub(crate) fn append_event(
    record: &mut AgentInstanceRecord,
    kind: &str,
    command_id: Option<&str>,
    attempt_id: Option<&str>,
    run_id: Option<&str>,
) -> Result<(), AgentInstanceError> {
    let sequence = record.next_event_sequence;
    record.next_event_sequence = sequence
        .checked_add(1)
        .ok_or(AgentInstanceError::Conflict("event sequence exhausted"))?;
    if record.events.len() >= record.limits.retained_events as usize {
        record.events.remove(0);
    }
    let event = AgentInstanceEvent {
        sequence,
        kind: kind.to_owned(),
        command_id: command_id.map(str::to_owned),
        attempt_id: attempt_id.map(str::to_owned),
        root_run_id: run_id.map(str::to_owned),
        epoch: record.epoch,
        committed_at: Utc::now(),
    };
    record.newly_appended_events.push(event.clone());
    record.events.push(event);
    Ok(())
}

pub(crate) fn trim_commands(record: &mut AgentInstanceRecord) -> Result<(), AgentInstanceError> {
    while record.inbox.len() >= record.limits.retained_commands as usize {
        let removable = record.inbox.first().is_some_and(|command| {
            matches!(
                command.status,
                InstanceCommandStatus::Completed
                    | InstanceCommandStatus::Failed
                    | InstanceCommandStatus::Cancelled,
            )
        });
        if !removable {
            return Err(AgentInstanceError::Capacity);
        }
        record.inbox.remove(0);
    }
    Ok(())
}
