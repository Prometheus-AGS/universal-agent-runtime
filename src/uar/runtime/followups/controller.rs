use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use chrono::Utc;
use crate::uar::{compiler::collaboration::CollaborationCatalogService,
    domain::followups::{FollowupAuthority, FollowupDecision, FollowupDisposition, FollowupInboxItem, FollowupRequest, FollowupRule, FollowupView},
    persistence::agent_instances::{AgentInstanceRecord, InstanceLifecycle},
    runtime::{actor::messages::ActorOwner, instance::AgentInstanceController}, security::claims::FollowupOwnerGrant};
use super::{FollowupError, quiet_hours};

/// Metadata-only delivery into the protected owner conversation inbox. This
/// host has no external connector transport and never starts a model turn.
pub struct FollowupController {
    catalog: Arc<CollaborationCatalogService>,
    instances: Arc<AgentInstanceController>,
    scan_offset: AtomicUsize,
}
impl std::fmt::Debug for FollowupController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Inbox and catalog contents remain behind their owner boundary.
        f.debug_struct("FollowupController").finish_non_exhaustive()
    }
}

impl FollowupController {
    pub fn new(catalog: Arc<CollaborationCatalogService>, instances: Arc<AgentInstanceController>) -> Arc<Self> {
        Arc::new(Self { catalog, instances, scan_offset: AtomicUsize::new(0) })
    }
    fn require_store(&self) -> Result<(), FollowupError> {
        self.catalog.require_followup_store()?;
        self.instances.require_store().map_err(|_| FollowupError::Unsupported)?;
        if !self.instances.store.supports_durable_observers() { return Err(FollowupError::Unsupported); }
        Ok(())
    }
    pub fn supported(&self) -> bool { self.require_store().is_ok() }
    /// The next deferred follow-up availability boundary. The durable catalog
    /// is still reloaded and arbitrated when this one-shot wakeup fires.
    pub async fn next_due_at(&self) -> Result<Option<chrono::DateTime<Utc>>, FollowupError> {
        self.require_store()?;
        let now = Utc::now();
        Ok(self.catalog.followup_rules().await?.iter()
            .filter(|rule| rule.enabled && !rule.revoked)
            .flat_map(|rule| rule.decisions.iter())
            .filter(|decision| decision.disposition == FollowupDisposition::Deferred)
            .filter_map(|decision| decision.available_at)
            .filter(|available_at| *available_at > now)
            .min())
    }
    async fn instance(&self, owner: &ActorOwner, workspace: &str, id: &str) -> Result<AgentInstanceRecord, FollowupError> {
        let record = self.instances.store.load_agent_instance(&owner.presentation_owner_key(), workspace, id)
            .await.map_err(|_| FollowupError::Storage)?.ok_or(FollowupError::Denied)?;
        record.validate(&owner.presentation_owner_key(), workspace).map_err(|_| FollowupError::Denied)?;
        if matches!(record.lifecycle, InstanceLifecycle::Disabled) { return Err(FollowupError::Denied); }
        let bound = self.catalog.resolve_bound_agent_run(&record.owner_id, workspace, &record.effective_binding.id)
            .await.map_err(catalog_error)?;
        if bound.binding.package != record.definition || bound.effective_binding_receipt.binding_ref != record.effective_binding {
            return Err(FollowupError::Denied);
        }
        self.catalog.execution_representation_grants(&record.owner_id, workspace, &record.representation_grants)
            .await.map_err(catalog_error)?;
        Ok(record)
    }
    async fn authority(&self, owner: &ActorOwner, rule: &FollowupRule) -> Result<(), FollowupError> {
        let source = self.instance(owner, &rule.workspace_id, &rule.source_instance_id).await?;
        if captured(&source) != rule.source_authority { return Err(FollowupError::Denied); }
        let destination = self.instance(owner, &rule.workspace_id, &rule.destination_instance_id).await?;
        if destination.session_id != rule.conversation_id || captured(&destination) != rule.destination_authority {
            return Err(FollowupError::Denied);
        }
        Ok(())
    }
    pub async fn create(&self, owner: &ActorOwner, workspace: &str, request: FollowupRequest) -> Result<FollowupView, FollowupError> {
        self.require_store()?; validate(&request, workspace)?;
        let principal = owner.credential_principal().ok_or(FollowupError::Denied)?;
        let source = self.instance(owner, workspace, &request.source_instance_id).await?;
        let destination = self.instance(owner, workspace, &request.destination_instance_id).await?;
        if destination.session_id != request.conversation_id { return Err(FollowupError::Denied); }
        let bounds = self.instances.store.instance_occurrence_bounds(&owner.presentation_owner_key(), workspace, &source.instance_id)
            .await.map_err(|_| FollowupError::Storage)?;
        let now = Utc::now(); let id = uuid::Uuid::new_v4().to_string();
        let rule = FollowupRule { owner_grant: FollowupOwnerGrant::capture(principal, &id, workspace, &destination.instance_id), id,
            owner_key: principal.storage_key(), catalog_owner_id: owner.presentation_owner_key(), workspace_id: workspace.into(),
            revision: 1, enabled: request.enabled, revoked: false, source_instance_id: request.source_instance_id,
            destination_instance_id: request.destination_instance_id, conversation_id: request.conversation_id,
            source_authority: captured(&source), destination_authority: captured(&destination), event_kinds: request.event_kinds,
            quiet_hours: request.quiet_hours, cursor: bounds.high, decisions: vec![], created_at: now, updated_at: now };
        self.catalog.create_followup_rule(rule).await.map(|r| FollowupView::from(&r))
    }
    pub async fn list(&self, owner: &ActorOwner, workspace: &str) -> Result<Vec<FollowupView>, FollowupError> {
        self.require_store()?; let owner = key(owner)?;
        Ok(self.catalog.followup_rules().await?.iter().filter(|r| r.owner_key == owner && r.workspace_id == workspace).map(FollowupView::from).collect())
    }
    pub async fn get(&self, owner: &ActorOwner, workspace: &str, id: &str) -> Result<FollowupView, FollowupError> {
        self.require_store()?;
        self.catalog.followup_rule(&key(owner)?, workspace, id).await.map(|r| FollowupView::from(&r))
    }
    pub async fn revise(&self, owner: &ActorOwner, workspace: &str, id: &str, revision: u64,
        request: FollowupRequest) -> Result<FollowupView, FollowupError> {
        self.require_store()?;
        validate(&request, workspace)?;
        let source = self.instance(owner, workspace, &request.source_instance_id).await?;
        let destination = self.instance(owner, workspace, &request.destination_instance_id).await?;
        if destination.session_id != request.conversation_id { return Err(FollowupError::Denied); }
        self.catalog.change_followup(&key(owner)?, workspace, id, |r, _, _| {
            expected(r, revision)?;
            if r.source_instance_id != request.source_instance_id || r.destination_instance_id != request.destination_instance_id
                || r.conversation_id != request.conversation_id { return Err(FollowupError::Invalid("create a new rule to change source or destination")); }
            r.revision = r.revision.checked_add(1).ok_or(FollowupError::Conflict("rule revision exhausted"))?;
            r.enabled = request.enabled; r.quiet_hours = request.quiet_hours.clone(); r.event_kinds = request.event_kinds.clone();
            r.source_authority = captured(&source); r.destination_authority = captured(&destination);
            for decision in r.decisions.iter_mut().filter(|d| matches!(d.disposition, FollowupDisposition::Queued | FollowupDisposition::Deferred)) {
                decision.rule_revision = r.revision;
                decision.available_at = None;
                if !r.enabled || !r.event_kinds.contains(&decision.event_kind) {
                    decision.disposition = FollowupDisposition::Suppressed;
                    decision.reason = if !r.enabled { "consent_disabled" } else { "event_not_selected" }.into();
                } else { decision.disposition = FollowupDisposition::Queued; decision.reason = "policy_recheck".into(); }
            }
            Ok(true)
        }).await.map(|r| FollowupView::from(&r))
    }
    pub async fn control(&self, owner: &ActorOwner, workspace: &str, id: &str, revision: u64,
        operation: &str) -> Result<FollowupView, FollowupError> {
        self.require_store()?;
        if !matches!(operation, "enable" | "disable" | "revoke") { return Err(FollowupError::Invalid("unknown follow-up operation")); }
        if operation == "enable" {
            let rule = self.catalog.followup_rule(&key(owner)?, workspace, id).await?;
            self.authority(owner, &rule).await?;
        }
        self.catalog.change_followup(&key(owner)?, workspace, id, |r, _, authority| {
            expected(r, revision)?;
            if operation == "enable" && !authority { return Err(FollowupError::Denied); }
            r.revision = r.revision.checked_add(1).ok_or(FollowupError::Conflict("rule revision exhausted"))?;
            r.enabled = operation == "enable"; r.revoked = operation == "revoke";
            for decision in r.decisions.iter_mut().filter(|d| matches!(d.disposition, FollowupDisposition::Queued | FollowupDisposition::Deferred)) {
                decision.rule_revision = r.revision;
                if !r.enabled { decision.disposition = FollowupDisposition::Suppressed; decision.available_at = None;
                    decision.reason = if r.revoked { "rule_revoked" } else { "consent_disabled" }.into(); }
            }
            Ok(true)
        }).await.map(|r| FollowupView::from(&r))
    }

    pub async fn inbox(&self, owner: &ActorOwner, workspace: &str, conversation: &str) -> Result<Vec<FollowupInboxItem>, FollowupError> {
        self.require_store()?;
        let owner_key = key(owner)?;
        let items = self.catalog.followup_items(&owner_key, workspace, conversation).await?;
        let mut visible = Vec::new();
        for item in items {
            let rule = self.catalog.followup_rule(&owner_key, workspace, &item.rule_id).await?;
            if rule.enabled && !rule.revoked && rule.conversation_id == conversation {
                match self.authority(owner, &rule).await {
                    Ok(()) => visible.push(item),
                    Err(FollowupError::Storage | FollowupError::Unsupported) => return Err(FollowupError::Storage),
                    Err(_) => {},
                }
            }
        }
        Ok(visible)
    }
    pub async fn acknowledge(&self, owner: &ActorOwner, workspace: &str, conversation: &str, item_id: &str) -> Result<(), FollowupError> {
        let items = self.inbox(owner, workspace, conversation).await?;
        let item = items.iter().find(|i| i.id == item_id).ok_or(FollowupError::NotFound)?;
        self.catalog.change_followup(&key(owner)?, workspace, &item.rule_id, |rule, inbox, authority| {
            if !rule.enabled || rule.revoked || !authority { return Err(FollowupError::Denied); }
            let item = inbox.get_mut(item_id).ok_or(FollowupError::NotFound)?;
            if item.acknowledged_at.is_some() { return Ok(false); }
            item.acknowledged_at = Some(Utc::now());
            Ok(true)
        }).await.map(|_| ())
    }

    pub async fn scan_once(&self) -> Result<(), FollowupError> {
        self.require_store()?;
        let rules = self.catalog.followup_rules().await?;
        if rules.is_empty() { return Ok(()); }
        let start = self.scan_offset.fetch_add(128, Ordering::Relaxed) % rules.len();
        for offset in 0..rules.len().min(128) {
            let rule = &rules[(start + offset) % rules.len()];
            if rule.revoked { continue; }
            if let Err(error) = self.process(rule).await {
                tracing::warn!(rule_id = %rule.id, code = error.code(), "Follow-up delivery deferred");
            }
        }
        Ok(())
    }
    async fn process(&self, listed: &FollowupRule) -> Result<(), FollowupError> {
        let owner = ActorOwner::from_followup_grant(&listed.owner_grant, &listed.owner_key, &listed.id,
            &listed.workspace_id, &listed.destination_instance_id).map_err(|_| FollowupError::Denied)?;
        if owner.presentation_owner_key() != listed.catalog_owner_id { return Err(FollowupError::Denied); }
        let authority = self.authority(&owner, listed).await;
        // An outage is not consent revocation and must not permanently consume
        // an occurrence as suppressed.
        if matches!(authority, Err(FollowupError::Storage | FollowupError::Unsupported)) { return authority; }
        let bounds = self.instances.store.instance_occurrence_bounds(&listed.catalog_owner_id, &listed.workspace_id, &listed.source_instance_id)
            .await.map_err(|_| FollowupError::Storage)?;
        let expected_sequence = listed.cursor.map_or(0, |v| v.saturating_add(1));
        if let Some(low) = bounds.low.filter(|low| *low > expected_sequence) {
            self.catalog.change_followup(&listed.owner_key, &listed.workspace_id, &listed.id, |r, _, _| {
                if r.cursor != listed.cursor || r.revision != listed.revision || r.revoked { return Ok(false); }
                r.decisions.push(FollowupDecision { id: format!("gap-{}-{expected_sequence}-{low}", r.id), rule_revision: r.revision,
                    occurrence_id: String::new(), source_sequence: low - 1, source_instance_id: r.source_instance_id.clone(),
                    event_kind: "source_gap".into(), occurred_at: Utc::now(), disposition: FollowupDisposition::Suppressed,
                    reason: format!("source_gap:{expected_sequence}-{}", low - 1), available_at: None, decided_at: Utc::now() });
                r.cursor = Some(low - 1); Ok(true)
            }).await?;
            return Ok(());
        }
        let events = self.instances.store.list_instance_occurrences(&listed.catalog_owner_id, &listed.workspace_id,
            &listed.source_instance_id, listed.cursor, 1).await.map_err(|_| FollowupError::Storage)?;
        if let Some(event) = events.first() {
            if event.owner_id != listed.catalog_owner_id || event.workspace_id != listed.workspace_id
                || event.source_instance_id != listed.source_instance_id || event.sequence != expected_sequence {
                return Err(FollowupError::Conflict("source occurrence scope or cursor changed"));
            }
            self.catalog.change_followup(&listed.owner_key, &listed.workspace_id, &listed.id, |r, _, current_authority| {
                if r.cursor != listed.cursor || r.revision != listed.revision || r.revoked { return Ok(false); }
                let reason = if !r.enabled { "consent_disabled" } else if authority.is_err() || !current_authority {
                    "source_or_destination_authority_revoked"
                } else if !r.event_kinds.contains(&event.kind) { "event_not_selected" } else { "admitted" };
                r.decisions.push(FollowupDecision { id: format!("followup-{}-{}", r.id, event.occurrence_id), rule_revision: r.revision,
                    occurrence_id: event.occurrence_id.clone(), source_sequence: event.sequence, source_instance_id: event.source_instance_id.clone(),
                    event_kind: event.kind.clone(), occurred_at: event.committed_at,
                    disposition: if reason == "admitted" { FollowupDisposition::Queued } else { FollowupDisposition::Suppressed },
                    reason: reason.into(), available_at: None, decided_at: Utc::now() });
                r.cursor = Some(event.sequence); Ok(true)
            }).await?;
        }
        self.dispatch(&owner, &listed.owner_key, &listed.workspace_id, &listed.id).await
    }
    async fn dispatch(&self, owner: &ActorOwner, owner_key: &str, workspace: &str, id: &str) -> Result<(), FollowupError> {
        let rule = self.catalog.followup_rule(owner_key, workspace, id).await?;
        let Some(decision) = rule.decisions.iter().find(|d| matches!(d.disposition, FollowupDisposition::Queued | FollowupDisposition::Deferred)) else { return Ok(()); };
        let authority = self.authority(owner, &rule).await;
        if matches!(authority, Err(FollowupError::Storage | FollowupError::Unsupported)) { return authority; }
        self.catalog.change_followup(owner_key, workspace, id, |r, inbox, current_authority| {
            if r.revision != rule.revision { return Ok(false); }
            let Some(index) = r.decisions.iter().position(|d| d.id == decision.id
                && matches!(d.disposition, FollowupDisposition::Queued | FollowupDisposition::Deferred)) else { return Ok(false); };
            let reason = if r.revoked { Some("rule_revoked") } else if !r.enabled { Some("consent_disabled") }
                else if authority.is_err() || !current_authority { Some("source_or_destination_authority_revoked") } else { None };
            let entry = &mut r.decisions[index];
            if let Some(reason) = reason { entry.disposition = FollowupDisposition::Suppressed; entry.reason = reason.into(); entry.available_at = None; return Ok(true); }
            let now = Utc::now();
            if let Some(hours) = &r.quiet_hours {
                if let Some(release) = quiet_hours::release_at(hours, now)? {
                    if entry.disposition == FollowupDisposition::Deferred && entry.available_at == Some(release) { return Ok(false); }
                    entry.disposition = FollowupDisposition::Deferred; entry.reason = "quiet_hours".into(); entry.available_at = Some(release); return Ok(true);
                }
            }
            inbox.entry(entry.id.clone()).or_insert_with(|| FollowupInboxItem { id: entry.id.clone(), owner_key: r.owner_key.clone(),
                workspace_id: r.workspace_id.clone(), conversation_id: r.conversation_id.clone(), rule_id: r.id.clone(), rule_revision: r.revision,
                occurrence_id: entry.occurrence_id.clone(), source_instance_id: entry.source_instance_id.clone(), event_kind: entry.event_kind.clone(),
                occurred_at: entry.occurred_at, delivered_at: now, acknowledged_at: None });
            entry.disposition = FollowupDisposition::Delivered; entry.reason = "conversation_inbox".into(); entry.available_at = None;
            Ok(true)
        }).await.map(|_| ())
    }
}
fn catalog_error(error: crate::uar::compiler::collaboration::CollaborationError) -> FollowupError {
    match error { crate::uar::compiler::collaboration::CollaborationError::Storage(_) => FollowupError::Storage,
        _ => FollowupError::Denied }
}
fn key(owner: &ActorOwner) -> Result<String, FollowupError> { owner.credential_principal().map(|p| p.storage_key()).ok_or(FollowupError::Denied) }
fn captured(record: &AgentInstanceRecord) -> FollowupAuthority { FollowupAuthority { binding: record.effective_binding.clone(),
    representation_revision: record.representation_revision, representation_grants: record.representation_grants.clone() } }
fn expected(rule: &FollowupRule, revision: u64) -> Result<(), FollowupError> {
    if rule.revoked || rule.revision != revision { return Err(FollowupError::Conflict("rule revoked or revision changed")); } Ok(())
}
fn validate(request: &FollowupRequest, workspace: &str) -> Result<(), FollowupError> {
    if [workspace, &request.source_instance_id, &request.destination_instance_id, &request.conversation_id].iter().any(|s| s.trim().is_empty() || s.len() > 256)
        || request.event_kinds.is_empty() || request.event_kinds.len() > 32
        || request.event_kinds.iter().any(|k| k.is_empty() || k.len() > 128 || !k.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_'))
        || request.event_kinds.iter().collect::<std::collections::BTreeSet<_>>().len() != request.event_kinds.len() {
        return Err(FollowupError::Invalid("follow-up source, destination or event selection is invalid"));
    }
    if let Some(hours) = &request.quiet_hours { quiet_hours::validate(hours)?; }
    Ok(())
}
