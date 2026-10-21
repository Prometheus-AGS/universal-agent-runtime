use std::{
    collections::BTreeMap,
    sync::{Arc, atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering}},
};

use chrono::Utc;
use serde::Serialize;
use tokio::sync::{Mutex, Notify};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::uar::{
    persistence::{
        PersistenceLayer,
        observers::{
            OBSERVER_SCHEMA_VERSION, ObserverAdmissionStatus, ObserverLimits,
            ObserverOccurrenceBounds, ObserverProjection, ObserverStoreError, ObserverSubscription,
        },
    },
    runtime::{actor::messages::ActorOwner, instance::AgentInstanceController},
};

use super::ObserverError;

/// One source's retained outbox range and this subscription's last processed event.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObserverSourceStatus {
    pub source_instance_id: String,
    pub cursor: Option<u64>,
    pub retained_low: Option<u64>,
    pub source_high: Option<u64>,
}

/// Authenticated administration view, excluding any model prompt or outcome.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObserverStatus {
    pub subscription: ObserverSubscription,
    pub sources: Vec<ObserverSourceStatus>,
    pub backlog_depth: usize,
    pub dead_letter_count: usize,
    pub recovery_actions: Vec<&'static str>,
}

/// Host-owned control plane. The persistent outbox and inbox are its source of truth.
pub struct ObserverController {
    pub(super) instances: Arc<AgentInstanceController>,
    pub(super) store: Arc<dyn PersistenceLayer>,
    // The local phase has one scheduler. This also linearizes revocation with dispatch.
    pub(super) delivery_lock: Mutex<()>,
    pub(super) scan_offset: AtomicUsize,
    pub(super) schedules: tokio::sync::RwLock<Option<Arc<crate::uar::runtime::schedules::ScheduleController>>>,
    pub(super) followups: tokio::sync::RwLock<Option<Arc<crate::uar::runtime::followups::FollowupController>>>,
    pub(super) work: Arc<Notify>,
    /// Advances only with confirmed durable/realtime work. Delivery uses this
    /// retained identity to give an overdue business boundary one attempt
    /// without turning unchanged failed work into a scan loop.
    pub(super) work_generation: AtomicU64,
    pub(super) started: AtomicBool,
    pub(super) shutdown: CancellationToken,
    pub(super) lifecycle_scope: crate::uar::runtime::lifecycle::ResourceScope,
}

impl std::fmt::Debug for ObserverController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Do not traverse host state, stores, or attached control planes.
        f.debug_struct("ObserverController").finish_non_exhaustive()
    }
}

impl ObserverController {
    #[must_use]
    pub fn new(
        instances: Arc<AgentInstanceController>,
        store: Arc<dyn PersistenceLayer>,
    ) -> Arc<Self> {
        Self::new_with_lifecycle(instances, store, None)
    }

    #[must_use]
    pub fn new_with_lifecycle(
        instances: Arc<AgentInstanceController>,
        store: Arc<dyn PersistenceLayer>,
        lifecycle_scope: Option<crate::uar::runtime::lifecycle::ResourceScope>,
    ) -> Arc<Self> {
        Arc::new(Self {
            instances,
            store,
            delivery_lock: Mutex::new(()),
            scan_offset: AtomicUsize::new(0),
            schedules: tokio::sync::RwLock::new(None),
            followups: tokio::sync::RwLock::new(None),
            work: Arc::new(Notify::new()),
            work_generation: AtomicU64::new(0),
            started: AtomicBool::new(false),
            shutdown: CancellationToken::new(),
            lifecycle_scope: lifecycle_scope.unwrap_or_else(|| {
                crate::uar::runtime::lifecycle::ResourceScope::standalone(
                    "observer-delivery",
                    "embedded-observer",
                )
            }),
        })
    }

    /// Wake the local observer after a confirmed durable commit or a retained
    /// realtime-generation change. The wakeup is only a signal: delivery still
    /// reads the authoritative catalog and occurrence records.
    pub fn notify_work(&self) {
        self.work_generation.fetch_add(1, Ordering::AcqRel);
        self.work.notify_one();
    }

    /// Request the observer task to stop. This does not claim that any
    /// persistence work has settled; the host lifecycle owns that join.
    pub fn shutdown(&self) {
        self.shutdown.cancel();
    }

    /// Join the local delivery task and retain its lifecycle report.
    pub async fn close(&self) -> crate::uar::runtime::lifecycle::CloseReport {
        self.shutdown();
        self.lifecycle_scope.close().await
    }

    /// Forward the retained local catalog generation stream into the observer
    /// work signal. Callers acquire the receiver before their initial durable
    /// snapshot, then start this forwarding task in the same lifecycle scope.
    pub fn observe_catalog_generations(
        self: &Arc<Self>,
        scope: &crate::uar::runtime::lifecycle::ResourceScope,
        mut generations: tokio::sync::watch::Receiver<u64>,
    ) -> Result<(), crate::uar::runtime::lifecycle::AdmissionClosed> {
        let observer = Arc::downgrade(self);
        let cancellation = scope.cancellation_token();
        scope.spawn("observer-catalog-generation", async move {
            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => break,
                    changed = generations.changed() => {
                        if changed.is_err() {
                            break;
                        }
                        let Some(observer) = observer.upgrade() else {
                            break;
                        };
                        observer.notify_work();
                    }
                }
            }
            Ok(())
        })
    }

    /// Attach durable schedules to the existing observer wakeup loop.
    pub async fn attach_schedules(&self, schedules: Arc<crate::uar::runtime::schedules::ScheduleController>) {
        *self.schedules.write().await = Some(schedules);
        self.notify_work();
    }

    pub async fn attach_followups(&self, followups: Arc<crate::uar::runtime::followups::FollowupController>) {
        *self.followups.write().await = Some(followups);
        self.notify_work();
    }

    pub async fn create(
        &self,
        owner: &ActorOwner,
        workspace: &str,
        observer_instance_id: &str,
        mut source_instance_ids: Vec<String>,
        conversation_ids: Option<Vec<String>>,
        limits: ObserverLimits,
    ) -> Result<ObserverStatus, ObserverError> {
        self.require_store()?;
        if workspace.trim().is_empty() || observer_instance_id.trim().is_empty() {
            return Err(ObserverError::Invalid(
                "workspace and observer instance are required",
            ));
        }
        source_instance_ids.sort();
        if source_instance_ids.is_empty()
            || source_instance_ids.iter().any(|id| id.trim().is_empty())
            || source_instance_ids
                .windows(2)
                .any(|pair| pair[0] == pair[1])
        {
            return Err(ObserverError::Invalid("source instances must be distinct"));
        }
        let mut conversation_ids = conversation_ids;
        if let Some(ids) = &mut conversation_ids {
            ids.sort();
            if ids.is_empty()
                || ids.iter().any(|id| id.trim().is_empty())
                || ids.windows(2).any(|pair| pair[0] == pair[1])
            {
                return Err(ObserverError::Invalid("conversation IDs must be distinct"));
            }
        }
        let observer = self
            .instances
            .load(owner, workspace, observer_instance_id)
            .await
            .map_err(instance_error)?;
        self.instances
            .bound(&observer)
            .await
            .map_err(instance_error)?;
        let mut permitted_conversations = Vec::with_capacity(source_instance_ids.len());
        for source_id in &source_instance_ids {
            if source_id == observer_instance_id {
                return Err(ObserverError::Invalid("an observer cannot watch itself"));
            }
            let source = self
                .instances
                .load(owner, workspace, source_id)
                .await
                .map_err(instance_error)?;
            self.instances
                .bound(&source)
                .await
                .map_err(instance_error)?;
            permitted_conversations.push(source.session_id);
        }
        if conversation_ids
            .as_ref()
            .is_some_and(|ids| ids.iter().any(|id| !permitted_conversations.contains(id)))
        {
            return Err(ObserverError::Invalid(
                "conversation intersection must belong to selected sources",
            ));
        }
        let now = Utc::now();
        let record = ObserverSubscription {
            schema_version: OBSERVER_SCHEMA_VERSION,
            subscription_id: Uuid::new_v4().to_string(),
            owner_id: owner.presentation_owner_key(),
            owner_user_id: owner.user_id().to_owned(),
            owner_tenant_id: owner.tenant_id().map(str::to_owned),
            workspace_id: workspace.to_owned(),
            observer_instance_id: observer_instance_id.to_owned(),
            source_instance_ids,
            conversation_ids,
            projection: ObserverProjection::ControlMetadata,
            revision: 0,
            paused: false,
            revoked: false,
            cursors: BTreeMap::new(),
            inbox: Vec::new(),
            limits,
            gaps: Vec::new(),
            created_at: now,
            updated_at: now,
        };
        record
            .validate(&record.owner_id, workspace)
            .map_err(|_| ObserverError::Invalid("observer scope or limits are invalid"))?;
        let created = self
            .store
            .create_observer_subscription(&record)
            .await
            .map_err(store_error)?;
        self.status_for(created).await
    }

    pub async fn list(
        &self,
        owner: &ActorOwner,
        workspace: &str,
    ) -> Result<Vec<ObserverStatus>, ObserverError> {
        self.require_store()?;
        let records = self
            .store
            .list_observer_subscriptions(&owner.presentation_owner_key(), workspace)
            .await
            .map_err(store_error)?;
        let mut views = Vec::with_capacity(records.len());
        for record in records {
            views.push(self.status_for(record).await?);
        }
        Ok(views)
    }

    pub async fn get(
        &self,
        owner: &ActorOwner,
        workspace: &str,
        subscription_id: &str,
    ) -> Result<ObserverStatus, ObserverError> {
        self.status_for(self.load(owner, workspace, subscription_id).await?)
            .await
    }

    pub async fn set_paused(
        &self,
        owner: &ActorOwner,
        workspace: &str,
        subscription_id: &str,
        expected_revision: u64,
        paused: bool,
    ) -> Result<ObserverStatus, ObserverError> {
        self.mutate(
            owner,
            workspace,
            subscription_id,
            expected_revision,
            |next| {
                if next.revoked {
                    return Err(ObserverError::Conflict("subscription is revoked"));
                }
                next.paused = paused;
                Ok(())
            },
        )
        .await
    }

    pub async fn revoke(
        &self,
        owner: &ActorOwner,
        workspace: &str,
        subscription_id: &str,
        expected_revision: u64,
    ) -> Result<ObserverStatus, ObserverError> {
        self.mutate(
            owner,
            workspace,
            subscription_id,
            expected_revision,
            |next| {
                next.revoked = true;
                Ok(())
            },
        )
        .await
    }

    pub async fn acknowledge_gap(
        &self,
        owner: &ActorOwner,
        workspace: &str,
        subscription_id: &str,
        expected_revision: u64,
        source_id: &str,
        missing_from: u64,
        missing_through: u64,
    ) -> Result<ObserverStatus, ObserverError> {
        self.mutate(
            owner,
            workspace,
            subscription_id,
            expected_revision,
            |next| {
                let gap = next
                    .gaps
                    .iter_mut()
                    .find(|gap| {
                        gap.source_instance_id == source_id
                            && gap.missing_from == missing_from
                            && gap.missing_through == missing_through
                            && gap.acknowledged_at.is_none()
                    })
                    .ok_or(ObserverError::NotFound)?;
                gap.acknowledged_at = Some(Utc::now());
                next.cursors.insert(source_id.to_owned(), missing_through);
                Ok(())
            },
        )
        .await
    }

    pub(super) async fn load(
        &self,
        owner: &ActorOwner,
        workspace: &str,
        subscription_id: &str,
    ) -> Result<ObserverSubscription, ObserverError> {
        self.require_store()?;
        self.store
            .load_observer_subscription(&owner.presentation_owner_key(), workspace, subscription_id)
            .await
            .map_err(store_error)?
            .ok_or(ObserverError::NotFound)
    }

    pub(super) async fn save(
        &self,
        previous: &ObserverSubscription,
        mut next: ObserverSubscription,
    ) -> Result<bool, ObserverError> {
        next.revision = previous
            .revision
            .checked_add(1)
            .ok_or(ObserverError::Conflict("subscription revision exhausted"))?;
        next.updated_at = Utc::now();
        self.store
            .compare_and_swap_observer_subscription(
                &previous.owner_id,
                &previous.workspace_id,
                previous.revision,
                &next,
            )
            .await
            .map_err(store_error)
    }

    async fn mutate(
        &self,
        owner: &ActorOwner,
        workspace: &str,
        subscription_id: &str,
        expected_revision: u64,
        change: impl FnOnce(&mut ObserverSubscription) -> Result<(), ObserverError>,
    ) -> Result<ObserverStatus, ObserverError> {
        let _guard = self.delivery_lock.lock().await;
        let current = self.load(owner, workspace, subscription_id).await?;
        if current.revision != expected_revision {
            return Err(ObserverError::Conflict("subscription revision changed"));
        }
        let mut next = current.clone();
        change(&mut next)?;
        if !self.save(&current, next).await? {
            return Err(ObserverError::Conflict("subscription revision changed"));
        }
        let updated = self.load(owner, workspace, subscription_id).await?;
        self.status_for(updated).await
    }

    pub(super) async fn status_for(
        &self,
        subscription: ObserverSubscription,
    ) -> Result<ObserverStatus, ObserverError> {
        let mut sources = Vec::with_capacity(subscription.source_instance_ids.len());
        for source_id in &subscription.source_instance_ids {
            let ObserverOccurrenceBounds { low, high } = self
                .store
                .instance_occurrence_bounds(
                    &subscription.owner_id,
                    &subscription.workspace_id,
                    source_id,
                )
                .await
                .map_err(store_error)?;
            sources.push(ObserverSourceStatus {
                source_instance_id: source_id.clone(),
                cursor: subscription.cursors.get(source_id).copied(),
                retained_low: low,
                source_high: high,
            });
        }
        let admitted_backlog = subscription
            .inbox
            .iter()
            .filter(|entry| {
                matches!(
                    entry.status,
                    ObserverAdmissionStatus::Admitted | ObserverAdmissionStatus::Retry
                )
            })
            .count();
        let source_backlog = sources.iter().fold(0usize, |count, source| {
            let pending = match (source.cursor, source.source_high) {
                (Some(cursor), Some(high)) => high.saturating_sub(cursor),
                (None, Some(high)) => high.saturating_add(1),
                _ => 0,
            };
            count.saturating_add(usize::try_from(pending).unwrap_or(usize::MAX))
        });
        let dead_letter_count = subscription
            .inbox
            .iter()
            .filter(|entry| entry.status == ObserverAdmissionStatus::DeadLetter)
            .count();
        let mut recovery_actions = Vec::new();
        if subscription
            .gaps
            .iter()
            .any(|gap| gap.acknowledged_at.is_none())
        {
            recovery_actions.push("acknowledge_retention_gap_or_resnapshot");
        }
        if dead_letter_count > 0 {
            recovery_actions.push("reconcile_observer_instance_then_recreate_subscription");
        }
        Ok(ObserverStatus {
            subscription,
            sources,
            backlog_depth: admitted_backlog.saturating_add(source_backlog),
            dead_letter_count,
            recovery_actions,
        })
    }

    pub(super) fn require_store(&self) -> Result<(), ObserverError> {
        if self.store.supports_durable_observers() {
            Ok(())
        } else {
            Err(ObserverError::Unavailable)
        }
    }
}

pub(super) fn store_error(error: anyhow::Error) -> ObserverError {
    match error.downcast_ref::<ObserverStoreError>() {
        Some(ObserverStoreError::Unsupported) => ObserverError::Unavailable,
        Some(ObserverStoreError::NotFound) => ObserverError::NotFound,
        Some(ObserverStoreError::AlreadyExists | ObserverStoreError::Conflict) => {
            ObserverError::Conflict("subscription revision changed")
        }
        Some(
            ObserverStoreError::InvalidRecord
            | ObserverStoreError::ImmutableField
            | ObserverStoreError::RevisionExhausted
            | ObserverStoreError::ScopeMismatch,
        ) => ObserverError::Invalid("subscription scope or state is invalid"),
        None => ObserverError::Internal(error),
    }
}

pub(super) fn instance_error(
    error: crate::uar::runtime::instance::AgentInstanceError,
) -> ObserverError {
    use crate::uar::runtime::instance::AgentInstanceError;
    match error {
        AgentInstanceError::NotFound => ObserverError::NotFound,
        AgentInstanceError::Conflict(reason) => ObserverError::Conflict(reason),
        AgentInstanceError::Capacity => ObserverError::Capacity,
        AgentInstanceError::Unavailable => ObserverError::Unavailable,
        AgentInstanceError::Invalid(_) => ObserverError::Invalid("instance request is invalid"),
        AgentInstanceError::Internal(error) => ObserverError::Internal(error),
    }
}
