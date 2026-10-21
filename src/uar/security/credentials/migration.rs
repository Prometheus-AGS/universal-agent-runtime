//! Metadata-only migration inventory and conditional-copy contracts.
use std::collections::{BTreeMap, HashSet};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::{CredentialScope, CredentialMigrationReceipt};
use super::owner::{decode_owner, is_bound_owner, is_fixed_local};
use super::policy::metadata_digest;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrincipalCredentialOwnerMapping {
    pub source_owner_key: String,
    pub scope: CredentialScope,
    pub source_scope_id: String,
    pub provider_id: String,
    pub source_revision: String,
    pub destination_owner_key: String,
    pub approval_id: String,
}
impl PrincipalCredentialOwnerMapping {
    pub(super) fn validate_all(mappings: &[Self]) -> anyhow::Result<()> {
        let mut seen = HashSet::new();
        for m in mappings {
            if m.scope == CredentialScope::System || !is_fixed_local(&m.source_owner_key)
                || !is_bound_owner(&m.destination_owner_key)
                || [&m.source_scope_id, &m.provider_id, &m.source_revision, &m.approval_id].into_iter().any(|v| v.trim().is_empty())
                || !seen.insert((&m.source_owner_key, m.scope, &m.source_scope_id, &m.provider_id)) {
                anyhow::bail!("invalid or conflicting principal credential mappings");
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RevisionInitializationSummary { pub legacy_initialized: u64, pub principal_initialized: u64 }

/// Projection deliberately excludes encrypted bytes, hints and free-form receipt reasons.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MigrationRowMetadata {
    #[serde(default)] pub owner_key: Option<String>,
    pub scope: CredentialScope, pub scope_id: String, pub provider_id: String,
    #[serde(default)] pub row_revision: Option<String>,
    pub created_at: DateTime<Utc>, pub updated_at: DateTime<Utc>,
}
#[derive(Clone, Debug, Default)]
pub struct MigrationInventory {
    pub legacy: Vec<MigrationRowMetadata>,
    pub principal: Vec<MigrationRowMetadata>,
    pub journals: Vec<(String, CredentialMigrationReceipt)>,
}

#[derive(Debug, thiserror::Error)]
pub enum CredentialPreviewError {
    #[error("credential_preview_request_invalid")] Invalid,
    #[error("credential_preview_stale")] Stale,
    #[error("credential_preview_denied")] Denied,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationPreviewQuery { pub limit: Option<usize>, pub cursor: Option<String> }
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MigrationPreviewItem {
    pub source_kind: String, pub source_owner_key: Option<String>, pub scope: CredentialScope,
    pub source_scope_id: String, pub provider_id: String, pub source_revision: Option<String>,
    pub created_at: Option<DateTime<Utc>>, pub updated_at: Option<DateTime<Utc>>,
    pub destination_owner_key: Option<String>, pub destination_scope_id: Option<String>,
    pub approval_id: Option<String>, pub destination_exists: bool, pub destination_revision: Option<String>,
    pub status: String, pub completed_at: Option<DateTime<Utc>>, pub precondition: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct NamespaceScopeCount { pub namespace: String, pub scope: String, pub total: usize }
#[derive(Debug, Serialize)]
pub struct MigrationPreviewCounts {
    pub page_total: usize, pub inventory_total: usize,
    pub by_namespace_scope: Vec<NamespaceScopeCount>, pub by_status: BTreeMap<String, usize>,
}
#[derive(Debug, Serialize)]
pub struct MigrationPreview {
    pub observed_at: DateTime<Utc>, pub policy_revision: String, pub mapping_revision: String,
    pub consistency: &'static str, pub items: Vec<MigrationPreviewItem>, pub counts: MigrationPreviewCounts,
    pub next_cursor: Option<String>,
}
#[derive(Serialize, Deserialize)]
struct PreviewCursor { domain: String, host: String, policy: String, mappings: String, after: Vec<String> }

pub(super) fn source_key(kind: &str, owner: Option<&str>, scope: CredentialScope, scope_id: &str, provider: &str) -> String {
    if kind == "legacy" { serde_json::json!([scope.as_str(), scope_id, provider]).to_string() }
    else { serde_json::json!(["principal-fixed-local-v1", owner, scope.as_str(), scope_id, provider]).to_string() }
}
pub(super) fn precondition(kind: &str, owner: Option<&str>, scope: CredentialScope, scope_id: &str, provider: &str, revision: &str, destination: &str, approval: &str, mapped_revision: Option<&str>) -> anyhow::Result<String> {
    Ok(format!("\"{}\"", metadata_digest(&serde_json::json!([kind, owner, scope, scope_id, provider, revision, destination, approval, mapped_revision]))?))
}
pub fn validate_if_match(value: Option<&str>) -> anyhow::Result<Option<String>> {
    match value {
        None => Ok(None),
        Some(value) if value.len() >= 3 && value.starts_with('"') && value.ends_with('"')
            && value[1..value.len()-1].bytes().all(|b| b == 0x21 || (0x23..=0x7e).contains(&b)) => Ok(Some(value.to_owned())),
        _ => Err(super::CredentialMigrationError::PreconditionInvalid.into()),
    }
}
fn tuple(item: &MigrationPreviewItem) -> Vec<String> {
    vec![item.source_kind.clone(), item.source_owner_key.clone().unwrap_or_default(), item.scope.as_str().to_owned(), item.source_scope_id.clone(), item.provider_id.clone()]
}

impl super::ProviderService {
    pub async fn migration_preview(&self, context: &crate::uar::security::claims::UserContext, policy: &super::CredentialTenancyPolicy, query: MigrationPreviewQuery) -> anyhow::Result<MigrationPreview> {
        let host_id = context.trusted_host_id().ok_or(CredentialPreviewError::Denied)?;
        let host_principal = crate::uar::security::claims::CredentialPrincipal::from_verified_context(context).map_err(|_| CredentialPreviewError::Denied)?;
        let host = metadata_digest(&(host_id, host_principal.storage_key()))?;
        let limit = query.limit.unwrap_or(100);
        if !(1..=500).contains(&limit) { return Err(CredentialPreviewError::Invalid.into()); }
        let mapping_revision = metadata_digest(&(&*self.legacy_owner_mappings, &*self.principal_owner_mappings))?;
        let after = if let Some(cursor) = query.cursor {
            if cursor.len() > 4096 { return Err(CredentialPreviewError::Invalid.into()); }
            let raw = self.encryption.decrypt(&cursor).map_err(|_| CredentialPreviewError::Invalid)?;
            let cursor: PreviewCursor = serde_json::from_str(&raw).map_err(|_| CredentialPreviewError::Invalid)?;
            if cursor.domain != "uar-credential-preview-v1" || cursor.host != host { return Err(CredentialPreviewError::Invalid.into()); }
            if cursor.policy != policy.revision() || cursor.mappings != mapping_revision { return Err(CredentialPreviewError::Stale.into()); }
            Some(cursor.after)
        } else { None };
        let inventory = self.store.migration_inventory().await?;
        let mut items = BTreeMap::<Vec<String>, MigrationPreviewItem>::new();
        for row in inventory.legacy.iter().filter(|r| r.scope != CredentialScope::System)
            .chain(inventory.principal.iter().filter(|r| r.owner_key.as_deref().is_some_and(is_fixed_local))) {
            let item = MigrationPreviewItem { source_kind: if row.owner_key.is_none() { "legacy" } else { "principal_fixed_local" }.to_owned(), source_owner_key: row.owner_key.clone(), scope: row.scope,
                source_scope_id: row.scope_id.clone(), provider_id: row.provider_id.clone(), source_revision: row.row_revision.clone(), created_at: Some(row.created_at), updated_at: Some(row.updated_at),
                destination_owner_key: None, destination_scope_id: None, approval_id: None, destination_exists: false, destination_revision: None, status: String::new(), completed_at: None, precondition: None };
            items.insert(tuple(&item), item);
        }
        let mappings: Vec<(String, Option<String>, CredentialScope, String, String, String, String, Option<String>)> = self.legacy_owner_mappings.iter().map(|m| ("legacy".to_owned(), None, m.scope, m.legacy_scope_id.clone(), m.provider_id.clone(), m.destination_owner_key.clone(), m.approval_id.clone(), None))
            .chain(self.principal_owner_mappings.iter().map(|m| ("principal_fixed_local".to_owned(), Some(m.source_owner_key.clone()), m.scope, m.source_scope_id.clone(), m.provider_id.clone(), m.destination_owner_key.clone(), m.approval_id.clone(), Some(m.source_revision.clone())))).collect();
        for (kind, owner, scope, scope_id, provider, destination, approval, _) in &mappings {
            let key = vec![kind.clone(), owner.clone().unwrap_or_default(), scope.as_str().to_owned(), scope_id.clone(), provider.clone()];
            let item = items.entry(key).or_insert_with(|| MigrationPreviewItem { source_kind: kind.clone(), source_owner_key: owner.clone(), scope: *scope, source_scope_id: scope_id.clone(), provider_id: provider.clone(), source_revision: None, created_at: None, updated_at: None, destination_owner_key: None, destination_scope_id: None, approval_id: None, destination_exists: false, destination_revision: None, status: String::new(), completed_at: None, precondition: None });
            item.destination_owner_key = Some(destination.clone());
            item.destination_scope_id = Some(if *scope == CredentialScope::User { decode_owner(destination).map(|v| v.3).unwrap_or("").to_owned() } else { scope_id.clone() });
            item.approval_id = Some(approval.clone());
        }
        for item in items.values_mut() {
            let mapping = mappings.iter().find(|m| m.0 == item.source_kind && m.1 == item.source_owner_key && m.2 == item.scope && m.3 == item.source_scope_id && m.4 == item.provider_id);
            let journal_key = source_key(&item.source_kind, item.source_owner_key.as_deref(), item.scope, &item.source_scope_id, &item.provider_id);
            let journal = inventory.journals.iter().find(|j| j.0 == journal_key).map(|j| &j.1);
            let destination_owner = item.destination_owner_key.as_ref().or_else(|| journal.map(|j| &j.destination_owner_key));
            let destination_scope = item.destination_scope_id.as_ref().or_else(|| journal.map(|j| &j.destination_scope_id));
            let destination = inventory.principal.iter().find(|row| row.owner_key.as_ref() == destination_owner && Some(&row.scope_id) == destination_scope && row.scope == item.scope && row.provider_id == item.provider_id);
            item.destination_exists = destination.is_some();
            item.destination_revision = destination.and_then(|r| r.row_revision.clone());
            if let (Some(mapping), Some(revision)) = (mapping, item.source_revision.as_deref()) {
                item.precondition = Some(precondition(&item.source_kind, item.source_owner_key.as_deref(), item.scope, &item.source_scope_id, &item.provider_id, revision, &mapping.5, &mapping.6, mapping.7.as_deref())?);
            }
            item.status = if let Some(journal) = journal {
                item.completed_at = Some(journal.completed_at);
                item.precondition = journal.precondition.clone();
                if mapping.is_none_or(|m| m.5 != journal.destination_owner_key || m.6 != journal.approval_id || (m.7.is_some() && m.7 != journal.source_revision)) { "journal_conflict" }
                else if destination.is_none() { "already_applied_destination_missing" }
                else if journal.destination_revision.is_none() { "already_applied_revision_unknown" }
                else if item.destination_revision != journal.destination_revision { "already_applied_destination_changed" }
                else { "already_applied" }
            } else if item.created_at.is_none() { "missing_source" }
            else if mapping.is_none() { "unmapped_quarantined" }
            else if item.source_revision.is_none() { "revision_unavailable" }
            else if mapping.is_some_and(|m| m.7.is_some() && m.7 != item.source_revision) { "source_changed" }
            else if destination.is_some() { "destination_conflict" }
            else { "ready" }.to_owned();
        }
        let mut namespaces = BTreeMap::<(String, String), usize>::new();
        let mut statuses = BTreeMap::new();
        for item in items.values() {
            let namespace = if item.source_kind == "legacy" { "legacy" } else if item.source_owner_key.as_deref().and_then(decode_owner).is_some_and(|v| v.0 == "local-host") { "local_host" } else { "local_jwt" };
            *namespaces.entry((namespace.to_owned(), item.scope.as_str().to_owned())).or_default() += 1;
            *statuses.entry(item.status.clone()).or_default() += 1;
        }
        let inventory_total = items.len();
        let mut page: Vec<_> = items.into_iter().filter(|(key, _)| after.as_ref().is_none_or(|after| key > after)).map(|(_, item)| item).take(limit + 1).collect();
        let has_more = page.len() > limit;
        page.truncate(limit);
        let next_cursor = if has_more { if let Some(last) = page.last() {
            Some(self.encryption.encrypt(&serde_json::to_string(&PreviewCursor { domain: "uar-credential-preview-v1".to_owned(), host: host.to_owned(), policy: policy.revision().to_owned(), mappings: mapping_revision.clone(), after: tuple(last) })?)?)
        } else { None } } else { None };
        if next_cursor.as_ref().is_some_and(|cursor| cursor.len() > 4096) { return Err(CredentialPreviewError::Invalid.into()); }
        Ok(MigrationPreview { observed_at: Utc::now(), policy_revision: policy.revision().to_owned(), mapping_revision, consistency: "page_observation", counts: MigrationPreviewCounts { page_total: page.len(), inventory_total, by_namespace_scope: namespaces.into_iter().map(|((namespace, scope), total)| NamespaceScopeCount { namespace, scope, total }).collect(), by_status: statuses }, items: page, next_cursor })
    }
}

/// Finalization evidence for accepted credential migration operations.
#[derive(Clone, Debug, Default)]
pub struct MigrationCloseReport {
    pub completed: usize,
    pub unresolved_operation_ids: Vec<String>,
}

type MigrationOutcome = Result<CredentialMigrationReceipt, super::CredentialMigrationError>;

#[derive(Debug)]
struct MigrationFlight {
    fingerprint: String,
    result: tokio::sync::watch::Receiver<Option<MigrationOutcome>>,
}

#[derive(Debug, Default)]
struct MigrationAdmission {
    closed: bool,
    flights: std::collections::HashMap<String, MigrationFlight>,
    report: MigrationCloseReport,
}

/// Host-owned transactions outlive an individual HTTP response. The same lock
/// serializes admission and task registration, including shutdown races.
#[derive(Debug, Default)]
pub(super) struct MigrationCoordinator {
    admission: std::sync::Mutex<MigrationAdmission>,
    tasks: tokio_util::task::TaskTracker,
}

impl MigrationCoordinator {
    pub(super) async fn execute(
        self: &std::sync::Arc<Self>,
        store: std::sync::Arc<dyn super::CredentialStore>,
        principal: super::CredentialOwnerBinding,
        authorization: super::principal_store::CredentialMigrationAuthorization,
    ) -> anyhow::Result<CredentialMigrationReceipt> {
        use futures::FutureExt;
        use super::CredentialMigrationError;
        let mapping = &authorization.mapping;
        let key = source_key(
            if authorization.source_owner_key.is_some() { "principal_fixed_local" } else { "legacy" },
            authorization.source_owner_key.as_deref(), mapping.scope,
            &mapping.legacy_scope_id, &mapping.provider_id,
        );
        let operation_id = metadata_digest(&key)?;
        let fingerprint = metadata_digest(&(
            &key, &mapping.destination_owner_key, &mapping.approval_id,
            &authorization.required_revision, &authorization.if_match,
            principal.storage_key(), &authorization.reason,
        ))?;
        let mut receiver = {
            let mut admission = self.admission.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            if admission.closed {
                return Err(CredentialMigrationError::Unavailable { operation_id }.into());
            }
            if let Some(flight) = admission.flights.get(&key) {
                if flight.fingerprint != fingerprint {
                    return Err(CredentialMigrationError::Contention { operation_id }.into());
                }
                flight.result.clone()
            } else {
                let (sender, receiver) = tokio::sync::watch::channel(None);
                admission.flights.insert(key.clone(), MigrationFlight { fingerprint, result: receiver.clone() });
                let coordinator = std::sync::Arc::clone(self);
                let task_operation_id = operation_id.clone();
                self.tasks.spawn(async move {
                    let operation = std::panic::AssertUnwindSafe(store.migrate_legacy(&principal, &authorization)).catch_unwind().await;
                    let result = match operation {
                        Ok(Ok(receipt)) => Ok(receipt),
                        Ok(Err(error)) => Err(error.downcast_ref::<CredentialMigrationError>().cloned()
                            .unwrap_or_else(|| CredentialMigrationError::Unavailable { operation_id: task_operation_id.clone() })),
                        Err(_) => Err(CredentialMigrationError::OutcomeUnknown { operation_id: task_operation_id.clone() }),
                    };
                    let mut admission = coordinator.admission.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                    admission.report.completed += 1;
                    if result.is_ok() {
                        admission.report.unresolved_operation_ids.retain(|id| id != &task_operation_id);
                    } else if matches!(&result, Err(CredentialMigrationError::OutcomeUnknown { .. }))
                        && !admission.report.unresolved_operation_ids.contains(&task_operation_id) {
                        admission.report.unresolved_operation_ids.push(task_operation_id);
                    }
                    sender.send_replace(Some(result));
                    admission.flights.remove(&key);
                });
                receiver
            }
        };
        loop {
            if let Some(result) = receiver.borrow_and_update().clone() {
                return result.map_err(Into::into);
            }
            if receiver.changed().await.is_err() {
                return Err(CredentialMigrationError::OutcomeUnknown { operation_id }.into());
            }
        }
    }

    pub(super) async fn shutdown(&self) -> MigrationCloseReport {
        {
            let mut admission = self.admission.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            admission.closed = true;
            self.tasks.close();
        }
        self.tasks.wait().await;
        self.admission.lock().unwrap_or_else(std::sync::PoisonError::into_inner).report.clone()
    }
}
