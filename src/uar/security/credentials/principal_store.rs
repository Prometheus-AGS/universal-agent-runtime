//! Principal credentials and explicit legacy mapping transactions.
//! The separate table/map is the ownership discriminator. An encoded owner key
//! in the old subject-only namespace would still collide with arbitrary subjects.

use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::store::{CredentialRecord, CredentialScope, InMemoryCredentialStore};
use super::owner::CredentialOwnerBinding;

/// Operator-reviewed mapping. Loading this configuration does not mint identity.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyCredentialOwnerMapping {
    pub scope: CredentialScope,
    pub legacy_scope_id: String,
    pub provider_id: String,
    pub destination_owner_key: String,
    pub approval_id: String,
}

/// Audit record committed atomically with the ciphertext copy. Source rows stay
/// quarantined; repeating this mapping never resurrects a deleted destination.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CredentialMigrationReceipt {
    pub scope: CredentialScope,
    pub legacy_scope_id: String,
    pub provider_id: String,
    pub destination_owner_key: String,
    pub destination_scope_id: String,
    pub approval_id: String,
    pub mapped_by_owner_key: String,
    #[serde(default)]
    pub reason: String,
    pub completed_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub source_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub source_owner_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub source_scope_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub source_revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub destination_revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub precondition: Option<String>,
}

/// Capability created only after matching installed operator configuration.
#[derive(Clone, Debug)]
pub struct CredentialMigrationAuthorization {
    pub(super) mapping: LegacyCredentialOwnerMapping,
    pub(super) reason: String,
    pub(super) source_owner_key: Option<String>,
    pub(super) required_revision: Option<String>,
    pub(super) if_match: Option<String>,
}

/// Named failures at the legacy ownership boundary.
#[derive(Clone, Debug, thiserror::Error)]
pub enum CredentialMigrationError {
    #[error("operator credential owner mapping required")]
    MappingDenied,
    #[error("legacy credential missing")]
    SourceMissing,
    #[error("credential owner mapping conflict")]
    Conflict,
    #[error("credential migration precondition failed")] PreconditionFailed,
    #[error("credential migration precondition invalid")] PreconditionInvalid,
    #[error("credential migration contention")] Contention { operation_id: String },
    #[error("credential migration unavailable")] Unavailable { operation_id: String },
    #[error("credential migration outcome unknown")] OutcomeUnknown { operation_id: String },
}

/// Principal-aware host storage. Ordinary lookup never reads the legacy table.
#[async_trait]
pub trait PrincipalCredentialStore: Send + Sync + std::fmt::Debug {
    async fn put_principal(
        &self,
        principal: &CredentialOwnerBinding,
        record: CredentialRecord,
    ) -> anyhow::Result<()>;
    async fn get_principal(
        &self,
        principal: &CredentialOwnerBinding,
        scope: CredentialScope,
        scope_id: &str,
        provider_id: &str,
    ) -> anyhow::Result<Option<CredentialRecord>>;
    async fn list_principal(
        &self,
        principal: &CredentialOwnerBinding,
        scope: CredentialScope,
        scope_id: &str,
    ) -> anyhow::Result<Vec<CredentialRecord>>;
    async fn delete_principal(
        &self,
        principal: &CredentialOwnerBinding,
        scope: CredentialScope,
        scope_id: &str,
        provider_id: &str,
    ) -> anyhow::Result<bool>;
    async fn migrate_legacy(
        &self,
        principal: &CredentialOwnerBinding,
        authorization: &CredentialMigrationAuthorization,
    ) -> anyhow::Result<CredentialMigrationReceipt>;
    async fn migration_inventory(&self) -> anyhow::Result<super::migration::MigrationInventory>;
    async fn initialize_revisions(&self) -> anyhow::Result<super::migration::RevisionInitializationSummary>;
    async fn unresolved_local(&self, owner: &CredentialOwnerBinding, scope: CredentialScope, scope_id: &str, provider_id: Option<&str>) -> anyhow::Result<bool> {
        let Some(old) = owner.fixed_local_key() else { return Ok(false); };
        let inventory = self.migration_inventory().await?;
        Ok(inventory.principal.iter().any(|row| row.owner_key.as_deref() == Some(old.as_str()) && row.scope == scope && row.scope_id == scope_id && provider_id.is_none_or(|p| p == row.provider_id)
            && !inventory.principal.iter().any(|destination| destination.owner_key.as_deref() == Some(owner.storage_key().as_str()) && destination.scope == scope && destination.scope_id == scope_id && destination.provider_id == row.provider_id)
            && !inventory.journals.iter().any(|(_, receipt)| receipt.source_owner_key.as_deref() == Some(old.as_str()) && receipt.scope == scope && receipt.legacy_scope_id == scope_id && receipt.provider_id == row.provider_id && receipt.destination_owner_key == owner.storage_key())))
    }

}

fn validate_scope(
    principal: &CredentialOwnerBinding,
    scope: CredentialScope,
    scope_id: &str,
) -> anyhow::Result<()> {
    // A user record cannot be relabelled with another subject, and the platform
    // account has its own operator path rather than a user-owned scope.
    if scope == CredentialScope::System
        || (scope == CredentialScope::User && scope_id != principal.subject())
    {
        anyhow::bail!("credential scope does not match verified principal");
    }
    Ok(())
}

fn source_key(scope: CredentialScope, scope_id: &str, provider: &str) -> String {
    serde_json::json!([scope.as_str(), scope_id, provider]).to_string()
}

fn principal_key(
    principal: &CredentialOwnerBinding,
    scope: CredentialScope,
    scope_id: &str,
    provider: &str,
) -> String {
    serde_json::json!([principal.storage_key(), scope.as_str(), scope_id, provider]).to_string()
}

fn receipt(
    principal: &CredentialOwnerBinding,
    authorization: &CredentialMigrationAuthorization,
) -> anyhow::Result<CredentialMigrationReceipt> {
    let mapping = &authorization.mapping;
    if mapping.destination_owner_key != principal.storage_key()
        || mapping.scope == CredentialScope::System
    {
        return Err(CredentialMigrationError::MappingDenied.into());
    }
    Ok(CredentialMigrationReceipt {
        scope: mapping.scope,
        legacy_scope_id: mapping.legacy_scope_id.clone(),
        provider_id: mapping.provider_id.clone(),
        destination_owner_key: principal.storage_key(),
        destination_scope_id: if mapping.scope == CredentialScope::User {
            principal.subject().to_owned()
        } else {
            mapping.legacy_scope_id.clone()
        },
        approval_id: mapping.approval_id.clone(),
        mapped_by_owner_key: principal.storage_key(),
        reason: authorization.reason.clone(),
        completed_at: Utc::now(),
        source_kind: Some(if authorization.source_owner_key.is_some() { "principal_fixed_local" } else { "legacy" }.to_owned()),
        source_owner_key: authorization.source_owner_key.clone(),
        source_scope_id: authorization.source_owner_key.as_ref().map(|_| mapping.legacy_scope_id.clone()),
        source_revision: authorization.required_revision.clone(),
        destination_revision: Some(uuid::Uuid::new_v4().to_string()),
        precondition: authorization.if_match.clone(),
    })
}

fn same_mapping(previous: &CredentialMigrationReceipt, next: &CredentialMigrationReceipt) -> bool {
    previous.destination_owner_key == next.destination_owner_key
        && previous.destination_scope_id == next.destination_scope_id
        && previous.approval_id == next.approval_id
        && previous.source_owner_key == next.source_owner_key
        && (next.source_owner_key.is_none() || previous.source_revision == next.source_revision)
}

fn migration_source_key(next: &CredentialMigrationReceipt) -> String {
    if let Some(owner) = &next.source_owner_key {
        super::migration::source_key("principal_fixed_local", Some(owner), next.scope, &next.legacy_scope_id, &next.provider_id)
    } else { source_key(next.scope, &next.legacy_scope_id, &next.provider_id) }
}
fn replay(previous: &CredentialMigrationReceipt, next: &CredentialMigrationReceipt, authorization: &CredentialMigrationAuthorization) -> anyhow::Result<CredentialMigrationReceipt> {
    if !same_mapping(previous, next) { return Err(CredentialMigrationError::Conflict.into()); }
    if authorization.if_match.as_ref().is_some_and(|tag| previous.precondition.as_ref() != Some(tag)) { return Err(CredentialMigrationError::PreconditionFailed.into()); }
    Ok(previous.clone())
}
fn bind_source_revision(next: &mut CredentialMigrationReceipt, authorization: &CredentialMigrationAuthorization, revision: Option<&str>) -> anyhow::Result<()> {
    if authorization.required_revision.as_deref().is_some_and(|expected| Some(expected) != revision) { return Err(CredentialMigrationError::PreconditionFailed.into()); }
    if let Some(tag) = &authorization.if_match {
        let revision = revision.ok_or(CredentialMigrationError::PreconditionFailed)?;
        let expected = super::migration::precondition(next.source_kind.as_deref().unwrap_or("legacy"), next.source_owner_key.as_deref(), next.scope, &next.legacy_scope_id, &next.provider_id, revision, &next.destination_owner_key, &next.approval_id, authorization.required_revision.as_deref())?;
        if &expected != tag { return Err(CredentialMigrationError::PreconditionFailed.into()); }
    }
    next.source_revision = revision.map(str::to_owned);
    Ok(())
}
fn metadata(owner_key: Option<String>, row: &CredentialRecord) -> super::migration::MigrationRowMetadata {
    super::migration::MigrationRowMetadata { owner_key, scope: row.scope, scope_id: row.scope_id.clone(), provider_id: row.provider_id.clone(), row_revision: row.row_revision.clone(), created_at: row.created_at, updated_at: row.updated_at }
}

#[derive(Debug, Default)]
pub(super) struct PrincipalMemoryState {
    pub(super) legacy: HashMap<(String, String, String), CredentialRecord>,
    pub(super) rows: HashMap<String, CredentialRecord>,
    pub(super) migrations: HashMap<String, CredentialMigrationReceipt>,
}

#[async_trait]
impl PrincipalCredentialStore for InMemoryCredentialStore {
    async fn put_principal(
        &self,
        principal: &CredentialOwnerBinding,
        mut record: CredentialRecord,
    ) -> anyhow::Result<()> {
        validate_scope(principal, record.scope, &record.scope_id)?;
        if self.unresolved_local(principal, record.scope, &record.scope_id, Some(&record.provider_id)).await? { return Err(super::CredentialPolicyError::MigrationRequired.into()); }
        record.row_revision = Some(uuid::Uuid::new_v4().to_string());
        let key = principal_key(
            principal,
            record.scope,
            &record.scope_id,
            &record.provider_id,
        );
        let mut state = self.state.write().await;
        if let Some(old) = state.rows.get(&key) {
            record.created_at = old.created_at;
        }
        state.rows.insert(key, record);
        Ok(())
    }

    async fn get_principal(
        &self,
        principal: &CredentialOwnerBinding,
        scope: CredentialScope,
        scope_id: &str,
        provider_id: &str,
    ) -> anyhow::Result<Option<CredentialRecord>> {
        validate_scope(principal, scope, scope_id)?;
        if self.unresolved_local(principal, scope, scope_id, Some(provider_id)).await? { return Err(super::CredentialPolicyError::MigrationRequired.into()); }
        Ok(self
            .state
            .read()
            .await
            .rows
            .get(&principal_key(principal, scope, scope_id, provider_id))
            .cloned())
    }

    async fn list_principal(
        &self,
        principal: &CredentialOwnerBinding,
        scope: CredentialScope,
        scope_id: &str,
    ) -> anyhow::Result<Vec<CredentialRecord>> {
        validate_scope(principal, scope, scope_id)?;
        if self.unresolved_local(principal, scope, scope_id, None).await? { return Err(super::CredentialPolicyError::MigrationRequired.into()); }
        let state = self.state.read().await;
        Ok(state
            .rows
            .iter()
            .filter(|(key, row)| {
                row.scope == scope
                    && row.scope_id == scope_id
                    && **key == principal_key(principal, row.scope, &row.scope_id, &row.provider_id)
            })
            .map(|(_, row)| row.clone())
            .collect())
    }

    async fn delete_principal(
        &self,
        principal: &CredentialOwnerBinding,
        scope: CredentialScope,
        scope_id: &str,
        provider_id: &str,
    ) -> anyhow::Result<bool> {
        validate_scope(principal, scope, scope_id)?;
        if self.unresolved_local(principal, scope, scope_id, Some(provider_id)).await? { return Err(super::CredentialPolicyError::MigrationRequired.into()); }
        Ok(self
            .state
            .write()
            .await
            .rows
            .remove(&principal_key(principal, scope, scope_id, provider_id))
            .is_some())
    }

    async fn migrate_legacy(&self, principal: &CredentialOwnerBinding, authorization: &CredentialMigrationAuthorization) -> anyhow::Result<CredentialMigrationReceipt> {
        let mut next = receipt(principal, authorization)?;
        let source = migration_source_key(&next);
        let mut state = self.state.write().await;
        if let Some(previous) = state.migrations.get(&source) { return replay(previous, &next, authorization); }
        let key = principal_key(principal, next.scope, &next.destination_scope_id, &next.provider_id);
        if state.rows.contains_key(&key) { return Err(CredentialMigrationError::Conflict.into()); }
        let mut row = if let Some(owner) = &authorization.source_owner_key {
            state.rows.get(&serde_json::json!([owner, next.scope.as_str(), next.legacy_scope_id, next.provider_id]).to_string())
        } else { state.legacy.get(&(next.scope.as_str().to_owned(), next.legacy_scope_id.clone(), next.provider_id.clone())) }
            .cloned().ok_or(CredentialMigrationError::SourceMissing)?;
        bind_source_revision(&mut next, authorization, row.row_revision.as_deref())?;
        row.scope_id.clone_from(&next.destination_scope_id);
        row.row_revision.clone_from(&next.destination_revision);
        state.rows.insert(key, row);
        state.migrations.insert(source, next.clone());
        Ok(next)
    }
    async fn migration_inventory(&self) -> anyhow::Result<super::migration::MigrationInventory> {
        let state = self.state.read().await;
        Ok(super::migration::MigrationInventory {
            legacy: state.legacy.values().map(|row| metadata(None, row)).collect(),
            principal: state.rows.iter().map(|(key, row)| {
                let tuple: Vec<String> = serde_json::from_str(key)?;
                Ok(metadata(tuple.first().cloned(), row))
            }).collect::<anyhow::Result<_>>()?,
            journals: state.migrations.iter().map(|(key, receipt)| { let mut receipt = receipt.clone(); receipt.reason.clear(); (key.clone(), receipt) }).collect(),
        })
    }
    async fn initialize_revisions(&self) -> anyhow::Result<super::migration::RevisionInitializationSummary> {
        let mut state = self.state.write().await;
        let mut result = super::migration::RevisionInitializationSummary::default();
        for row in state.legacy.values_mut().filter(|row| row.row_revision.is_none()) { row.row_revision = Some(uuid::Uuid::new_v4().to_string()); result.legacy_initialized += 1; }
        for row in state.rows.values_mut().filter(|row| row.row_revision.is_none()) { row.row_revision = Some(uuid::Uuid::new_v4().to_string()); result.principal_initialized += 1; }
        Ok(result)
    }

}

#[cfg(feature = "surreal-backend")]
use super::store::SurrealCredentialStore;

#[cfg(feature = "surreal-backend")]
#[derive(Serialize, Deserialize)]
struct PrincipalRow {
    owner_key: String,
    #[serde(flatten)]
    credential: CredentialRecord,
}

#[cfg(feature = "surreal-backend")]
fn surreal_rows(values: Vec<surrealdb::types::Value>) -> anyhow::Result<Vec<PrincipalRow>> {
    values
        .into_iter()
        .map(|value| {
            let value = crate::uar::persistence::providers::surreal::surreal_to_json(value)?;
            serde_json::from_value(value)
                .map_err(|_| anyhow::anyhow!("invalid stored principal credential"))
        })
        .collect()
}

#[cfg(feature = "surreal-backend")]
fn migration_query_error(error: surrealdb::Error, operation_id: &str) -> anyhow::Error {
    if matches!(error.query_details(), Some(surrealdb::types::QueryError::TransactionConflict)) {
        return CredentialMigrationError::Contention { operation_id: operation_id.to_owned() }.into();
    }
    let message = error.to_string();
    if message.contains("uar_credential_source_missing") {
        CredentialMigrationError::SourceMissing.into()
    } else if message.contains("uar_credential_precondition_failed") {
        CredentialMigrationError::PreconditionFailed.into()
    } else if message.contains("uar_credential_mapping_conflict") {
        CredentialMigrationError::Conflict.into()
    } else {
        CredentialMigrationError::Unavailable { operation_id: operation_id.to_owned() }.into()
    }
}

#[cfg(feature = "surreal-backend")]
impl SurrealCredentialStore {
    async fn reconcile_migration_receipt(
        &self, source: &str, next: &CredentialMigrationReceipt,
        authorization: &CredentialMigrationAuthorization,
    ) -> anyhow::Result<Option<CredentialMigrationReceipt>> {
        use crate::uar::persistence::providers::surreal::surreal_to_json;
        let mut response = self.db.query("SELECT * FROM type::record('provider_credential_owner_migrations', $source)")
            .bind(("source", source.to_owned())).await?.check()?;
        let values: Vec<surrealdb::types::Value> = response.take(0)?;
        let Some(value) = values.into_iter().next() else { return Ok(None); };
        let previous: CredentialMigrationReceipt = serde_json::from_value(surreal_to_json(value)?)?;
        replay(&previous, next, authorization).map(Some)
    }
}

#[cfg(feature = "surreal-backend")]
#[async_trait]
impl PrincipalCredentialStore for SurrealCredentialStore {
    async fn put_principal(
        &self,
        principal: &CredentialOwnerBinding,
        mut record: CredentialRecord,
    ) -> anyhow::Result<()> {
        use crate::uar::persistence::providers::surreal::to_db_value;
        validate_scope(principal, record.scope, &record.scope_id)?;
        if self.unresolved_local(principal, record.scope, &record.scope_id, Some(&record.provider_id)).await? { return Err(super::CredentialPolicyError::MigrationRequired.into()); }
        record.row_revision = Some(uuid::Uuid::new_v4().to_string());
        let rid = principal_key(
            principal,
            record.scope,
            &record.scope_id,
            &record.provider_id,
        );
        let payload = to_db_value(&PrincipalRow {
            owner_key: principal.storage_key(),
            credential: record,
        })?;
        self.db.query("BEGIN TRANSACTION; LET $old = (SELECT * FROM type::record('provider_principal_credentials', $rid))[0]; UPSERT type::record('provider_principal_credentials', $rid) CONTENT $data; IF $old != NONE { UPDATE type::record('provider_principal_credentials', $rid) SET created_at = $old.created_at; }; COMMIT TRANSACTION;")
            .bind(("rid", rid)).bind(("data", payload)).await?.check()?;
        Ok(())
    }

    async fn get_principal(
        &self,
        principal: &CredentialOwnerBinding,
        scope: CredentialScope,
        scope_id: &str,
        provider_id: &str,
    ) -> anyhow::Result<Option<CredentialRecord>> {
        validate_scope(principal, scope, scope_id)?;
        if self.unresolved_local(principal, scope, scope_id, Some(provider_id)).await? { return Err(super::CredentialPolicyError::MigrationRequired.into()); }
        let mut response = self.db.query("SELECT * FROM type::record('provider_principal_credentials', $rid) WHERE owner_key = $owner AND scope = $scope AND scope_id = $scope_id AND provider_id = $provider")
            .bind(("rid", principal_key(principal, scope, scope_id, provider_id)))
            .bind(("owner", principal.storage_key())).bind(("scope", scope.as_str().to_owned()))
            .bind(("scope_id", scope_id.to_owned())).bind(("provider", provider_id.to_owned())).await?.check()?;
        Ok(surreal_rows(response.take(0)?)?
            .into_iter()
            .next()
            .map(|row| row.credential))
    }

    async fn list_principal(
        &self,
        principal: &CredentialOwnerBinding,
        scope: CredentialScope,
        scope_id: &str,
    ) -> anyhow::Result<Vec<CredentialRecord>> {
        validate_scope(principal, scope, scope_id)?;
        if self.unresolved_local(principal, scope, scope_id, None).await? { return Err(super::CredentialPolicyError::MigrationRequired.into()); }
        let mut response = self.db.query("SELECT * FROM provider_principal_credentials WHERE owner_key = $owner AND scope = $scope AND scope_id = $scope_id ORDER BY provider_id")
            .bind(("owner", principal.storage_key())).bind(("scope", scope.as_str().to_owned()))
            .bind(("scope_id", scope_id.to_owned())).await?.check()?;
        Ok(surreal_rows(response.take(0)?)?
            .into_iter()
            .map(|row| row.credential)
            .collect())
    }

    async fn delete_principal(
        &self,
        principal: &CredentialOwnerBinding,
        scope: CredentialScope,
        scope_id: &str,
        provider_id: &str,
    ) -> anyhow::Result<bool> {
        validate_scope(principal, scope, scope_id)?;
        if self.unresolved_local(principal, scope, scope_id, Some(provider_id)).await? { return Err(super::CredentialPolicyError::MigrationRequired.into()); }
        let mut response = self.db.query("DELETE type::record('provider_principal_credentials', $rid) WHERE owner_key = $owner AND scope = $scope AND scope_id = $scope_id AND provider_id = $provider RETURN BEFORE")
            .bind(("rid", principal_key(principal, scope, scope_id, provider_id)))
            .bind(("owner", principal.storage_key())).bind(("scope", scope.as_str().to_owned()))
            .bind(("scope_id", scope_id.to_owned())).bind(("provider", provider_id.to_owned())).await?.check()?;
        let values: Vec<surrealdb::types::Value> = response.take(0)?;
        Ok(!values.is_empty())
    }

    async fn migrate_legacy(&self, principal: &CredentialOwnerBinding, authorization: &CredentialMigrationAuthorization) -> anyhow::Result<CredentialMigrationReceipt> {
        use crate::uar::persistence::providers::surreal::to_db_value;
        let mut next = receipt(principal, authorization)?;
        let source = migration_source_key(&next);
        let operation_id = super::policy::metadata_digest(&source)?;
        // Preflight uses only metadata. The transaction repeats the actual compare.
        let inventory = self.migration_inventory().await?;
        if let Some((_, previous)) = inventory.journals.iter().find(|(key, _)| key == &source) {
            // Fetch the immutable full receipt only after exact authorization.
            replay(previous, &next, authorization)?;
        } else if authorization.if_match.is_some() || authorization.required_revision.is_some() {
            let row = inventory.legacy.iter().chain(inventory.principal.iter()).find(|row| row.owner_key == authorization.source_owner_key && row.scope == next.scope && row.scope_id == next.legacy_scope_id && row.provider_id == next.provider_id).ok_or(CredentialMigrationError::SourceMissing)?;
            bind_source_revision(&mut next, authorization, row.row_revision.as_deref())?;
        }
        let old_table = if authorization.source_owner_key.is_some() { "provider_principal_credentials" } else { "provider_credentials" };
        let old_id = if let Some(owner) = &authorization.source_owner_key { serde_json::json!([owner, next.scope.as_str(), next.legacy_scope_id, next.provider_id]).to_string() } else { SurrealCredentialStore::record_id(next.scope, &next.legacy_scope_id, &next.provider_id) };
        let compare_revision = authorization.if_match.is_some() || authorization.required_revision.is_some();
        let transaction = self.db.query("BEGIN TRANSACTION;
            LET $journal = (SELECT * FROM type::record('provider_credential_owner_migrations', $source))[0];
            IF $journal != NONE {
                IF $journal.destination_owner_key != $owner OR $journal.destination_scope_id != $destination_scope OR $journal.approval_id != $approval OR ($source_owner != NONE AND ($journal.source_owner_key != $source_owner OR $journal.source_revision != $required_revision)) { THROW 'uar_credential_mapping_conflict'; };
                IF $condition != NONE AND $journal.precondition != $condition { THROW 'uar_credential_precondition_failed'; };
            } ELSE {
                LET $old = (SELECT * FROM type::record($old_table, $old_id))[0];
                IF $old = NONE OR $old.scope != $scope OR $old.scope_id != $legacy_scope OR $old.provider_id != $provider OR ($source_owner != NONE AND $old.owner_key != $source_owner) { THROW 'uar_credential_source_missing'; };
                IF $compare_revision AND ($old.row_revision = NONE OR $old.row_revision != $revision) { THROW 'uar_credential_precondition_failed'; };
                LET $existing = (SELECT * FROM type::record('provider_principal_credentials', $destination))[0];
                IF $existing != NONE { THROW 'uar_credential_mapping_conflict'; };
                CREATE type::record('provider_principal_credentials', $destination) SET owner_key = $owner, scope = $scope, scope_id = $destination_scope, provider_id = $provider, api_key_encrypted = $old.api_key_encrypted, api_key_hint = $old.api_key_hint, created_at = $old.created_at, updated_at = $old.updated_at, row_revision = $destination_revision;
                CREATE type::record('provider_credential_owner_migrations', $source) CONTENT $receipt;
                UPDATE type::record('provider_credential_owner_migrations', $source) SET source_revision = $old.row_revision;
            };
            COMMIT TRANSACTION;")
            .bind(("source", source.clone())).bind(("old_table", old_table.to_owned())).bind(("old_id", old_id))
            .bind(("destination", principal_key(principal, next.scope, &next.destination_scope_id, &next.provider_id)))
            .bind(("owner", next.destination_owner_key.clone())).bind(("scope", next.scope.as_str().to_owned()))
            .bind(("legacy_scope", next.legacy_scope_id.clone())).bind(("destination_scope", next.destination_scope_id.clone()))
            .bind(("provider", next.provider_id.clone())).bind(("approval", next.approval_id.clone()))
            .bind(("source_owner", authorization.source_owner_key.clone())).bind(("required_revision", authorization.required_revision.clone()))
            .bind(("condition", authorization.if_match.clone())).bind(("compare_revision", compare_revision))
            .bind(("revision", next.source_revision.clone())).bind(("destination_revision", next.destination_revision.clone()))
            .bind(("receipt", to_db_value(&next)?)).await;
        match transaction {
            Ok(mut response) => {
                let errors = response.take_errors();
                if !errors.is_empty() {
                    // Keep explicit transaction rejection ahead of secondary abort errors.
                    let mapped: Vec<anyhow::Error> = errors.into_values()
                        .map(|error| migration_query_error(error, &operation_id)).collect();
                    let selected = mapped.into_iter().min_by_key(|error| match error.downcast_ref::<CredentialMigrationError>() {
                        Some(CredentialMigrationError::SourceMissing | CredentialMigrationError::Conflict
                            | CredentialMigrationError::PreconditionFailed) => 0,
                        Some(CredentialMigrationError::Contention { .. }) => 1,
                        _ => 2,
                    });
                    if let Some(error) = selected {
                        if matches!(error.downcast_ref::<CredentialMigrationError>(), Some(CredentialMigrationError::Contention { .. })) {
                            // Arbitration rejected this transaction. One authoritative
                            // receipt lookup can establish a matching committed replay;
                            // absence cannot establish that the competing work failed.
                            match self.reconcile_migration_receipt(&source, &next, authorization).await {
                                Ok(Some(receipt)) => return Ok(receipt),
                                Err(reconciliation) if reconciliation.downcast_ref::<CredentialMigrationError>().is_some() => return Err(reconciliation),
                                Ok(None) | Err(_) => {}
                            }
                        }
                        return Err(error);
                    }
                }
            }
            // The complete transaction was dispatched. A missing response does not
            // establish rollback, and a second read cannot disprove a late commit.
            Err(_) => return Err(CredentialMigrationError::OutcomeUnknown { operation_id }.into()),
        }
        match self.reconcile_migration_receipt(&source, &next, authorization).await {
            Ok(Some(receipt)) => Ok(receipt),
            Err(error) if error.downcast_ref::<CredentialMigrationError>().is_some() => Err(error),
            Ok(None) | Err(_) => Err(CredentialMigrationError::OutcomeUnknown { operation_id }.into()),
        }
    }
    async fn migration_inventory(&self) -> anyhow::Result<super::migration::MigrationInventory> {
        use crate::uar::persistence::providers::surreal::surreal_to_json;
        let mut response = self.db.query("BEGIN TRANSACTION;
            SELECT scope, scope_id, provider_id, row_revision, created_at, updated_at FROM provider_credentials WHERE scope != 'system';
            SELECT owner_key, scope, scope_id, provider_id, row_revision, created_at, updated_at FROM provider_principal_credentials;
            SELECT scope, legacy_scope_id, provider_id, destination_owner_key, destination_scope_id, approval_id, mapped_by_owner_key, completed_at, source_kind, source_owner_key, source_scope_id, source_revision, destination_revision, precondition FROM provider_credential_owner_migrations;
            COMMIT TRANSACTION;").await?.check()?;
        let legacy: Vec<surrealdb::types::Value> = response.take(1)?;
        let principal: Vec<surrealdb::types::Value> = response.take(2)?;
        let journals: Vec<surrealdb::types::Value> = response.take(3)?;
        Ok(super::migration::MigrationInventory {
            legacy: legacy.into_iter().map(|v| Ok(serde_json::from_value(surreal_to_json(v)?)?)).collect::<anyhow::Result<_>>()?,
            principal: principal.into_iter().map(|v| Ok(serde_json::from_value(surreal_to_json(v)?)?)).collect::<anyhow::Result<_>>()?,
            journals: journals.into_iter().map(|v| { let receipt: CredentialMigrationReceipt = serde_json::from_value(surreal_to_json(v)?)?; Ok((migration_source_key(&receipt), receipt)) }).collect::<anyhow::Result<_>>()?,
        })
    }
    async fn initialize_revisions(&self) -> anyhow::Result<super::migration::RevisionInitializationSummary> {
        let mut response = self.db.query("BEGIN TRANSACTION;
            LET $legacy = SELECT id FROM provider_credentials WHERE row_revision = NONE;
            LET $principal = SELECT id FROM provider_principal_credentials WHERE row_revision = NONE;
            FOR $row IN $legacy { UPDATE $row.id SET row_revision = <string>rand::uuid::v4(); };
            FOR $row IN $principal { UPDATE $row.id SET row_revision = <string>rand::uuid::v4(); };
            RETURN { legacy_initialized: array::len($legacy), principal_initialized: array::len($principal) };
            COMMIT TRANSACTION;").await?.check()?;
        let value: Option<surrealdb::types::Value> = response.take(5)?;
        serde_json::from_value(crate::uar::persistence::providers::surreal::surreal_to_json(value.ok_or_else(|| anyhow::anyhow!("credential initialization result missing"))?)?).map_err(Into::into)
    }

}

#[cfg(feature = "sqlx")]
use super::store::{PostgresCredentialStore, row_to_record};

#[cfg(feature = "sqlx")]
#[async_trait]
impl PrincipalCredentialStore for PostgresCredentialStore {
    async fn put_principal(
        &self,
        principal: &CredentialOwnerBinding,
        mut record: CredentialRecord,
    ) -> anyhow::Result<()> {
        validate_scope(principal, record.scope, &record.scope_id)?;
        if self.unresolved_local(principal, record.scope, &record.scope_id, Some(&record.provider_id)).await? { return Err(super::CredentialPolicyError::MigrationRequired.into()); }
        record.row_revision = Some(uuid::Uuid::new_v4().to_string());
        sqlx::query("INSERT INTO provider_principal_credentials (owner_key, scope, scope_id, provider_id, api_key_encrypted, api_key_hint, created_at, updated_at, row_revision) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9) ON CONFLICT (owner_key, scope, scope_id, provider_id) DO UPDATE SET api_key_encrypted = EXCLUDED.api_key_encrypted, api_key_hint = EXCLUDED.api_key_hint, updated_at = EXCLUDED.updated_at, row_revision = EXCLUDED.row_revision")
            .bind(principal.storage_key()).bind(record.scope.as_str()).bind(&record.scope_id)
            .bind(&record.provider_id).bind(&record.api_key_encrypted).bind(&record.api_key_hint)
            .bind(record.created_at).bind(record.updated_at).bind(record.row_revision).execute(&self.pool).await?;
        Ok(())
    }

    async fn get_principal(
        &self,
        principal: &CredentialOwnerBinding,
        scope: CredentialScope,
        scope_id: &str,
        provider_id: &str,
    ) -> anyhow::Result<Option<CredentialRecord>> {
        validate_scope(principal, scope, scope_id)?;
        if self.unresolved_local(principal, scope, scope_id, Some(provider_id)).await? { return Err(super::CredentialPolicyError::MigrationRequired.into()); }
        let row = sqlx::query("SELECT * FROM provider_principal_credentials WHERE owner_key = $1 AND scope = $2 AND scope_id = $3 AND provider_id = $4")
            .bind(principal.storage_key()).bind(scope.as_str()).bind(scope_id).bind(provider_id)
            .fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_record))
    }

    async fn list_principal(
        &self,
        principal: &CredentialOwnerBinding,
        scope: CredentialScope,
        scope_id: &str,
    ) -> anyhow::Result<Vec<CredentialRecord>> {
        validate_scope(principal, scope, scope_id)?;
        if self.unresolved_local(principal, scope, scope_id, None).await? { return Err(super::CredentialPolicyError::MigrationRequired.into()); }
        let rows = sqlx::query("SELECT * FROM provider_principal_credentials WHERE owner_key = $1 AND scope = $2 AND scope_id = $3 ORDER BY provider_id")
            .bind(principal.storage_key()).bind(scope.as_str()).bind(scope_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_record).collect())
    }

    async fn delete_principal(
        &self,
        principal: &CredentialOwnerBinding,
        scope: CredentialScope,
        scope_id: &str,
        provider_id: &str,
    ) -> anyhow::Result<bool> {
        validate_scope(principal, scope, scope_id)?;
        if self.unresolved_local(principal, scope, scope_id, Some(provider_id)).await? { return Err(super::CredentialPolicyError::MigrationRequired.into()); }
        Ok(sqlx::query("DELETE FROM provider_principal_credentials WHERE owner_key = $1 AND scope = $2 AND scope_id = $3 AND provider_id = $4")
            .bind(principal.storage_key()).bind(scope.as_str()).bind(scope_id).bind(provider_id)
            .execute(&self.pool).await?.rows_affected() != 0)
    }

    async fn migrate_legacy(&self, principal: &CredentialOwnerBinding, authorization: &CredentialMigrationAuthorization) -> anyhow::Result<CredentialMigrationReceipt> {
        use sqlx::Row;
        use futures::FutureExt as _;
        let mut next = receipt(principal, authorization)?;
        let source = migration_source_key(&next);
        let operation_id = super::policy::metadata_digest(&source)?;
        let outcome: anyhow::Result<CredentialMigrationReceipt> = async {
        let mut tx = self.pool.begin().await?;
        let prepared: anyhow::Result<(CredentialMigrationReceipt, bool)> = std::panic::AssertUnwindSafe(async {
        // This source-unique insertion serializes concurrent attempts even before a journal exists.
        let inserted = sqlx::query("INSERT INTO provider_credential_owner_migrations (source_key, receipt) VALUES ($1,$2) ON CONFLICT (source_key) DO NOTHING")
            .bind(&source).bind(serde_json::to_value(&next)?).execute(&mut *tx).await?.rows_affected() != 0;
        let journal = sqlx::query("SELECT receipt FROM provider_credential_owner_migrations WHERE source_key = $1 FOR UPDATE").bind(&source).fetch_one(&mut *tx).await?;
        let previous: CredentialMigrationReceipt = serde_json::from_value(journal.try_get("receipt")?)?;
        if !inserted { return Ok((replay(&previous, &next, authorization)?, false)); }
        let row = if let Some(owner) = &authorization.source_owner_key {
            sqlx::query("SELECT * FROM provider_principal_credentials WHERE owner_key = $1 AND scope = $2 AND scope_id = $3 AND provider_id = $4 FOR UPDATE")
                .bind(owner).bind(next.scope.as_str()).bind(&next.legacy_scope_id).bind(&next.provider_id).fetch_optional(&mut *tx).await?
        } else {
            sqlx::query("SELECT * FROM provider_credentials WHERE scope = $1 AND scope_id = $2 AND provider_id = $3 FOR UPDATE")
                .bind(next.scope.as_str()).bind(&next.legacy_scope_id).bind(&next.provider_id).fetch_optional(&mut *tx).await?
        }.ok_or(CredentialMigrationError::SourceMissing)?;
        let old = row_to_record(&row);
        bind_source_revision(&mut next, authorization, old.row_revision.as_deref())?;
        let copied = sqlx::query("INSERT INTO provider_principal_credentials (owner_key, scope, scope_id, provider_id, api_key_encrypted, api_key_hint, created_at, updated_at, row_revision) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9) ON CONFLICT (owner_key, scope, scope_id, provider_id) DO NOTHING")
            .bind(&next.destination_owner_key).bind(next.scope.as_str()).bind(&next.destination_scope_id)
            .bind(&next.provider_id).bind(&old.api_key_encrypted).bind(&old.api_key_hint).bind(old.created_at).bind(old.updated_at).bind(&next.destination_revision)
            .execute(&mut *tx).await?.rows_affected() != 0;
        if !copied { return Err(CredentialMigrationError::Conflict.into()); }
        sqlx::query("UPDATE provider_credential_owner_migrations SET receipt = $2 WHERE source_key = $1").bind(&source).bind(serde_json::to_value(&next)?).execute(&mut *tx).await?;
        Ok((next, true))
        }).catch_unwind().await.unwrap_or_else(|_| Err(CredentialMigrationError::OutcomeUnknown {
            operation_id: operation_id.clone(),
        }.into()));
        match prepared {
            Ok((receipt, true)) => {
                tx.commit().await.map_err(|_| CredentialMigrationError::OutcomeUnknown {
                    operation_id: operation_id.clone(),
                })?;
                Ok(receipt)
            }
            Ok((receipt, false)) => {
                tx.rollback().await?;
                Ok(receipt)
            }
            Err(error) => {
                // A rejected precondition or destination collision still owns
                // an open SQL transaction. Await rollback rather than relying
                // on Transaction::drop to queue driver cleanup after completion.
                if tx.rollback().await.is_err() {
                    return Err(CredentialMigrationError::Unavailable {
                        operation_id: operation_id.clone(),
                    }.into());
                }
                Err(error)
            }
        }
        }.await;
        outcome.map_err(|error| {
            if error.downcast_ref::<CredentialMigrationError>().is_some() { return error; }
            if let Some(sqlx::Error::Database(database)) = error.downcast_ref::<sqlx::Error>() {
                if database.code().is_some_and(|code| matches!(code.as_ref(), "40001" | "40P01")) {
                    return CredentialMigrationError::Contention { operation_id }.into();
                }
            }
            CredentialMigrationError::Unavailable { operation_id }.into()
        })
    }
    async fn migration_inventory(&self) -> anyhow::Result<super::migration::MigrationInventory> {
        use sqlx::Row;
        let mut tx = self.pool.begin().await?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY").execute(&mut *tx).await?;
        let legacy = sqlx::query("SELECT NULL::text AS owner_key, scope, scope_id, provider_id, row_revision, created_at, updated_at FROM provider_credentials WHERE scope != 'system'").fetch_all(&mut *tx).await?;
        let principal = sqlx::query("SELECT owner_key, scope, scope_id, provider_id, row_revision, created_at, updated_at FROM provider_principal_credentials").fetch_all(&mut *tx).await?;
        let journals = sqlx::query("SELECT source_key, receipt - 'reason' AS receipt FROM provider_credential_owner_migrations").fetch_all(&mut *tx).await?;
        let rows = |rows: Vec<sqlx::postgres::PgRow>| -> anyhow::Result<Vec<super::migration::MigrationRowMetadata>> { rows.iter().map(|r| Ok(super::migration::MigrationRowMetadata { owner_key: r.try_get("owner_key")?, scope: CredentialScope::from_str_lenient(&r.try_get::<String, _>("scope")?), scope_id: r.try_get("scope_id")?, provider_id: r.try_get("provider_id")?, row_revision: r.try_get("row_revision")?, created_at: r.try_get("created_at")?, updated_at: r.try_get("updated_at")? })).collect() };
        let inventory = super::migration::MigrationInventory { legacy: rows(legacy)?, principal: rows(principal)?, journals: journals.iter().map(|r| Ok((r.try_get("source_key")?, serde_json::from_value(r.try_get("receipt")?)?))).collect::<anyhow::Result<_>>()? };
        tx.commit().await?;
        Ok(inventory)
    }
    async fn initialize_revisions(&self) -> anyhow::Result<super::migration::RevisionInitializationSummary> {
        use sqlx::Row;
        let mut tx = self.pool.begin().await?;
        // The explicit operator quiescence acknowledgement is still required across replicas.
        sqlx::query("LOCK TABLE provider_credentials, provider_principal_credentials IN ACCESS EXCLUSIVE MODE").execute(&mut *tx).await?;
        let mut summary = super::migration::RevisionInitializationSummary::default();
        for row in sqlx::query("SELECT scope, scope_id, provider_id FROM provider_credentials WHERE row_revision IS NULL").fetch_all(&mut *tx).await? {
            sqlx::query("UPDATE provider_credentials SET row_revision=$4 WHERE scope=$1 AND scope_id=$2 AND provider_id=$3 AND row_revision IS NULL").bind(row.try_get::<String,_>("scope")?).bind(row.try_get::<String,_>("scope_id")?).bind(row.try_get::<String,_>("provider_id")?).bind(uuid::Uuid::new_v4().to_string()).execute(&mut *tx).await?;
            summary.legacy_initialized += 1;
        }
        for row in sqlx::query("SELECT owner_key, scope, scope_id, provider_id FROM provider_principal_credentials WHERE row_revision IS NULL").fetch_all(&mut *tx).await? {
            sqlx::query("UPDATE provider_principal_credentials SET row_revision=$5 WHERE owner_key=$1 AND scope=$2 AND scope_id=$3 AND provider_id=$4 AND row_revision IS NULL").bind(row.try_get::<String,_>("owner_key")?).bind(row.try_get::<String,_>("scope")?).bind(row.try_get::<String,_>("scope_id")?).bind(row.try_get::<String,_>("provider_id")?).bind(uuid::Uuid::new_v4().to_string()).execute(&mut *tx).await?;
            summary.principal_initialized += 1;
        }
        tx.commit().await?;
        Ok(summary)
    }

}
