use std::sync::{Arc, atomic::Ordering};

use chrono::Utc;
use sha2::{Digest, Sha256};

use crate::uar::{
    persistence::observers::{
        ObserverAdmission, ObserverAdmissionStatus, ObserverGap, ObserverOccurrence,
        ObserverSubscription,
    },
    runtime::actor::messages::ActorOwner,
};

use super::{
    ObserverController, ObserverError,
    controller::{instance_error, store_error},
};

const SCAN_BATCH: usize = 1;
const SUBSCRIPTIONS_PER_CYCLE: usize = 128;

impl ObserverController {
    /// Start the controller's owned lifecycle scope for embedding callers.
    pub fn start(self: &Arc<Self>) {
        let scope = self.lifecycle_scope.clone();
        if let Err(error) = self.start_in_scope(&scope) {
            tracing::warn!(%error, "Observer delivery was rejected during lifecycle shutdown");
        }
    }

    /// Register the local delivery owner before it can begin executing. The
    /// scope closes its admission, requests cancellation, and joins this task
    /// before its resource finalizer releases persistence.
    pub fn start_in_scope(
        self: &Arc<Self>,
        scope: &crate::uar::runtime::lifecycle::ResourceScope,
    ) -> Result<(), crate::uar::runtime::lifecycle::AdmissionClosed> {
        if self.started.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        let controller = Arc::clone(self);
        let scope_cancellation = scope.cancellation_token();
        if let Err(error) = scope.spawn("observer-delivery", async move {
            controller.delivery_task(scope_cancellation).await
        }) {
            self.started.store(false, Ordering::Release);
            return Err(error);
        }
        Ok(())
    }

    async fn delivery_task(
        self: &Arc<Self>,
        scope_cancellation: tokio_util::sync::CancellationToken,
    ) -> anyhow::Result<()> {
        let weak = Arc::downgrade(self);
        let work = Arc::clone(&self.work);
        let shutdown = self.shutdown.child_token();
        // A due time can cross between the authoritative scan and arming its
        // timer. Record its business-time identity plus the committed-work
        // generation so that crossing gets one delivery attempt, while an
        // unchanged failed delivery cannot form a zero-delay scan loop.
        let mut last_due_attempt: Option<(chrono::DateTime<Utc>, u64)> = None;
        loop {
            let Some(controller) = weak.upgrade() else {
                break;
            };
            // Capture before the snapshot-driven delivery. A commit arriving
            // during that scan requires one more authoritative pass even if a
            // later `Notify` permit is consumed by another wakeup.
            let observed_generation = controller.work_generation.load(Ordering::Acquire);
            controller.deliver_ready_work().await;
            let work_generation = controller.work_generation.load(Ordering::Acquire);
            if work_generation != observed_generation {
                last_due_attempt = None;
                continue;
            }
            let next_due = controller.next_due_at().await;
            drop(controller);
            match next_due {
                Some(due_at) => {
                    let due_identity = (due_at, work_generation);
                    match due_at.signed_duration_since(Utc::now()).to_std() {
                        Ok(delay) => {
                            tokio::select! {
                                _ = shutdown.cancelled() => break,
                                _ = scope_cancellation.cancelled() => break,
                                _ = work.notified() => {},
                                _ = tokio::time::sleep(delay) => {
                                    last_due_attempt = Some(due_identity);
                                },
                            }
                        }
                        Err(_) if last_due_attempt.as_ref() != Some(&due_identity) => {
                            // The one-shot boundary became due while planning.
                            // Loop once so delivery reads and arbitrates the
                            // durable record after the boundary.
                            last_due_attempt = Some(due_identity);
                        }
                        Err(_) => {
                            tokio::select! {
                                _ = shutdown.cancelled() => break,
                                _ = scope_cancellation.cancelled() => break,
                                _ = work.notified() => {},
                            }
                        }
                    }
                }
                None => {
                    tokio::select! {
                        _ = shutdown.cancelled() => break,
                        _ = scope_cancellation.cancelled() => break,
                        _ = work.notified() => {},
                    }
                }
            }
        }
        Ok(())
    }

    async fn next_due_at(&self) -> Option<chrono::DateTime<Utc>> {
        let schedules = self.schedules.read().await.clone();
        let followups = self.followups.read().await.clone();
        let schedule_due = match schedules {
            Some(schedules) => match schedules.next_due_at().await {
                Ok(due_at) => due_at,
                Err(error) => {
                    tracing::warn!(code = error.code(), "Schedule due-time lookup failed");
                    None
                }
            },
            None => None,
        };
        let followup_due = match followups {
            Some(followups) => match followups.next_due_at().await {
                Ok(due_at) => due_at,
                Err(error) => {
                    tracing::warn!(code = error.code(), "Follow-up due-time lookup failed");
                    None
                }
            },
            None => None,
        };
        match (schedule_due, followup_due) {
            (Some(schedule), Some(followup)) => Some(schedule.min(followup)),
            (Some(schedule), None) => Some(schedule),
            (None, Some(followup)) => Some(followup),
            (None, None) => None,
        }
    }

    async fn deliver_ready_work(&self) {
        let schedules = self.schedules.read().await.clone();
        if let Some(schedules) = schedules {
            if let Err(error) = schedules.scan_once().await {
                tracing::warn!(code = error.code(), "Schedule delivery cycle failed");
            }
        }
        let followups = self.followups.read().await.clone();
        if let Some(followups) = followups {
            if let Err(error) = followups.scan_once().await {
                tracing::warn!(code = error.code(), "Follow-up delivery cycle failed");
            }
        }
        if let Err(error) = self.deliver_once().await {
            tracing::warn!(code = error.code(), "Observer delivery cycle failed");
        }
    }

    /// One pass admits at most one occurrence per source and processes one inbox item per subscription.
    pub async fn deliver_once(&self) -> Result<(), ObserverError> {
        self.require_store()?;
        let subscriptions = self
            .store
            .list_all_observer_subscriptions()
            .await
            .map_err(store_error)?;
        let count = subscriptions.len();
        if count == 0 {
            return Ok(());
        }
        let start = self
            .scan_offset
            .fetch_add(SUBSCRIPTIONS_PER_CYCLE, Ordering::Relaxed)
            % count;
        for offset in 0..count.min(SUBSCRIPTIONS_PER_CYCLE) {
            let listed = &subscriptions[(start + offset) % count];
            let _guard = self.delivery_lock.lock().await;
            let owner = match ActorOwner::from_persisted_identity(
                &listed.owner_user_id,
                listed.owner_tenant_id.as_deref(),
                &listed.owner_id,
            ) {
                Ok(owner) => owner,
                Err(_) => {
                    tracing::warn!(subscription_id = %listed.subscription_id, "Observer owner identity is invalid");
                    continue;
                }
            };
            let subscription = match self
                .load(&owner, &listed.workspace_id, &listed.subscription_id)
                .await
            {
                Ok(subscription) => subscription,
                Err(error) => {
                    tracing::warn!(
                        code = error.code(),
                        "Observer subscription could not be reloaded"
                    );
                    continue;
                }
            };
            if subscription.paused || subscription.revoked {
                continue;
            }
            if let Err(error) = self.process_subscription(&owner, subscription).await {
                if let ObserverError::Conflict(reason) = &error {
                    tracing::warn!(
                        code = error.code(),
                        reason,
                        "Observer subscription delivery failed"
                    );
                } else {
                    tracing::warn!(code = error.code(), "Observer subscription delivery failed");
                }
            }
        }
        Ok(())
    }

    async fn process_subscription(
        &self,
        owner: &ActorOwner,
        subscription: ObserverSubscription,
    ) -> Result<(), ObserverError> {
        if let Some(index) = subscription.inbox.iter().position(|entry| {
            matches!(
                entry.status,
                ObserverAdmissionStatus::Admitted | ObserverAdmissionStatus::Retry
            )
        }) {
            return self.deliver_admission(owner, subscription, index).await;
        }
        let source_count = subscription.source_instance_ids.len();
        let source_start = usize::try_from(subscription.revision).unwrap_or(0) % source_count;
        for offset in 0..source_count {
            let source_id =
                &subscription.source_instance_ids[(source_start + offset) % source_count];
            if subscription
                .gaps
                .iter()
                .any(|gap| gap.source_instance_id == *source_id && gap.acknowledged_at.is_none())
            {
                continue;
            }
            let cursor = subscription.cursors.get(source_id).copied();
            let bounds = self
                .store
                .instance_occurrence_bounds(
                    &subscription.owner_id,
                    &subscription.workspace_id,
                    source_id,
                )
                .await
                .map_err(store_error)?;
            if let Some(low) = bounds.low {
                let expected = cursor.map_or(0, |value| value.saturating_add(1));
                if low > expected {
                    return self
                        .record_gap(subscription.clone(), source_id, expected, low - 1)
                        .await;
                }
            }
            let occurrences = self
                .store
                .list_instance_occurrences(
                    &subscription.owner_id,
                    &subscription.workspace_id,
                    source_id,
                    cursor,
                    SCAN_BATCH,
                )
                .await
                .map_err(store_error)?;
            let Some(occurrence) = occurrences.into_iter().next() else {
                continue;
            };
            let expected = cursor.map_or(0, |value| value.saturating_add(1));
            if occurrence.sequence < expected {
                return Err(ObserverError::Conflict("source outbox ordering changed"));
            }
            if occurrence.sequence > expected {
                return self
                    .record_gap(
                        subscription.clone(),
                        source_id,
                        expected,
                        occurrence.sequence - 1,
                    )
                    .await;
            }
            self.validate_occurrence(&subscription, &occurrence)?;
            let mut next = subscription.clone();
            next.cursors.insert(source_id.clone(), occurrence.sequence);
            if next
                .conversation_ids
                .as_ref()
                .is_none_or(|ids| ids.contains(&occurrence.conversation_id))
            {
                self.require_current_authority(owner, &next, &occurrence)
                    .await?;
                while next.inbox.len() >= next.limits.max_inbox as usize {
                    let Some(index) = next
                        .inbox
                        .iter()
                        .position(|entry| entry.status == ObserverAdmissionStatus::Acknowledged)
                    else {
                        return Err(ObserverError::Capacity);
                    };
                    next.inbox.remove(index);
                }
                let command_id =
                    observer_command_id(&next.subscription_id, &occurrence.occurrence_id);
                next.inbox.push(ObserverAdmission {
                    occurrence_id: occurrence.occurrence_id.clone(),
                    source_instance_id: source_id.clone(),
                    source_sequence: occurrence.sequence,
                    occurrence,
                    observer_command_id: command_id,
                    status: ObserverAdmissionStatus::Admitted,
                    attempts: 0,
                    last_error_code: None,
                    admitted_at: Utc::now(),
                    updated_at: Utc::now(),
                });
            }
            if !self.save(&subscription, next).await? {
                return Err(ObserverError::Conflict("subscription revision changed"));
            }
            return Ok(());
        }
        Ok(())
    }

    async fn record_gap(
        &self,
        subscription: ObserverSubscription,
        source_id: &str,
        missing_from: u64,
        missing_through: u64,
    ) -> Result<(), ObserverError> {
        let mut next = subscription.clone();
        if next.gaps.len() >= crate::uar::persistence::observers::MAX_OBSERVER_GAPS {
            return Err(ObserverError::Capacity);
        }
        next.gaps.push(ObserverGap {
            source_instance_id: source_id.to_owned(),
            missing_from,
            missing_through,
            detected_at: Utc::now(),
            acknowledged_at: None,
        });
        if !self.save(&subscription, next).await? {
            return Err(ObserverError::Conflict("subscription revision changed"));
        }
        Ok(())
    }

    async fn deliver_admission(
        &self,
        owner: &ActorOwner,
        subscription: ObserverSubscription,
        index: usize,
    ) -> Result<(), ObserverError> {
        let admission = &subscription.inbox[index];
        let authority = self
            .require_current_authority(owner, &subscription, &admission.occurrence)
            .await;
        // Reload immediately before the C06 command path, including external CAS revocation.
        let latest = self
            .load(
                owner,
                &subscription.workspace_id,
                &subscription.subscription_id,
            )
            .await?;
        if latest.revision != subscription.revision || latest.paused || latest.revoked {
            return Err(ObserverError::Conflict("subscription authority changed"));
        }
        let result = match authority {
            Ok(()) => {
                let prompt = projection_prompt(&admission.occurrence)?;
                self.instances
                    .submit(
                        owner,
                        &subscription.workspace_id,
                        &subscription.observer_instance_id,
                        &admission.observer_command_id,
                        &prompt,
                    )
                    .await
                    .map(|_| ())
                    .map_err(instance_error)
            }
            Err(error) => Err(error),
        };
        let mut next = subscription.clone();
        let entry = &mut next.inbox[index];
        match result {
            Ok(()) => {
                entry.status = ObserverAdmissionStatus::Acknowledged;
                entry.last_error_code = None;
            }
            Err(error) => {
                entry.attempts = entry.attempts.saturating_add(1);
                entry.last_error_code = Some(error.code().to_owned());
                entry.status = if entry.attempts >= next.limits.max_retries {
                    ObserverAdmissionStatus::DeadLetter
                } else {
                    ObserverAdmissionStatus::Retry
                };
            }
        }
        entry.updated_at = Utc::now();
        while next
            .inbox
            .iter()
            .filter(|item| item.status == ObserverAdmissionStatus::Acknowledged)
            .count()
            > next.limits.retained_acknowledged as usize
        {
            let Some(oldest) = next
                .inbox
                .iter()
                .position(|item| item.status == ObserverAdmissionStatus::Acknowledged)
            else {
                break;
            };
            next.inbox.remove(oldest);
        }
        if !self.save(&subscription, next).await? {
            return Err(ObserverError::Conflict(
                "subscription revision changed after admission",
            ));
        }
        Ok(())
    }

    async fn require_current_authority(
        &self,
        owner: &ActorOwner,
        subscription: &ObserverSubscription,
        occurrence: &ObserverOccurrence,
    ) -> Result<(), ObserverError> {
        if subscription.revoked
            || subscription.paused
            || !subscription
                .source_instance_ids
                .contains(&occurrence.source_instance_id)
            || subscription
                .conversation_ids
                .as_ref()
                .is_some_and(|ids| !ids.contains(&occurrence.conversation_id))
        {
            return Err(ObserverError::Conflict(
                "observer projection is no longer granted",
            ));
        }
        self.validate_occurrence(subscription, occurrence)?;
        let observer = self
            .instances
            .load(
                owner,
                &subscription.workspace_id,
                &subscription.observer_instance_id,
            )
            .await
            .map_err(instance_error)?;
        self.instances
            .bound(&observer)
            .await
            .map_err(instance_error)?;
        let source = self
            .instances
            .load(
                owner,
                &subscription.workspace_id,
                &occurrence.source_instance_id,
            )
            .await
            .map_err(instance_error)?;
        self.instances
            .bound(&source)
            .await
            .map_err(instance_error)?;
        if source.session_id != occurrence.conversation_id {
            return Err(ObserverError::Conflict(
                "source conversation identity changed",
            ));
        }
        Ok(())
    }

    fn validate_occurrence(
        &self,
        subscription: &ObserverSubscription,
        occurrence: &ObserverOccurrence,
    ) -> Result<(), ObserverError> {
        if occurrence.owner_id != subscription.owner_id
            || occurrence.workspace_id != subscription.workspace_id
            || !subscription
                .source_instance_ids
                .contains(&occurrence.source_instance_id)
        {
            return Err(ObserverError::Conflict(
                "source occurrence is outside subscription scope",
            ));
        }
        Ok(())
    }
}

fn observer_command_id(subscription_id: &str, occurrence_id: &str) -> String {
    let digest = Sha256::digest(format!("observe\0{subscription_id}\0{occurrence_id}").as_bytes());
    let suffix = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("observe-{suffix}")
}

fn projection_prompt(occurrence: &ObserverOccurrence) -> Result<String, ObserverError> {
    serde_json::to_string(&serde_json::json!({
        "observation": {
            "occurrenceId": &occurrence.occurrence_id,
            "sourceInstanceId": &occurrence.source_instance_id,
            "conversationId": &occurrence.conversation_id,
            "sequence": occurrence.sequence,
            "kind": &occurrence.kind,
            "commandId": &occurrence.command_id,
            "attemptId": &occurrence.attempt_id,
            "rootRunId": &occurrence.root_run_id,
            "epoch": occurrence.epoch,
            "committedAt": occurrence.committed_at,
        }
    }))
    .map_err(|error| ObserverError::Internal(error.into()))
}
