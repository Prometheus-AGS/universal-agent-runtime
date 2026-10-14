//! Principal credentials and explicit legacy mapping transactions.
//! The separate table/map is the ownership discriminator. An encoded owner key
//! in the old subject-only namespace would still collide with arbitrary subjects.

use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::store::{CredentialRecord, CredentialScope, InMemoryCredentialStore};
use crate::uar::security::claims::CredentialPrincipal;

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
    pub reason: String,
    pub completed_at: DateTime<Utc>,
}

/// Capability created only after matching installed operator configuration.
#[derive(Clone, Debug)]
pub struct CredentialMigrationAuthorization {
    pub(super) mapping: LegacyCredentialOwnerMapping,
    pub(super) reason: String,
}

/// Named failures at the legacy ownership boundary.
#[derive(Debug, thiserror::Error)]
pub enum CredentialMigrationError {
    #[error("operator credential owner mapping required")]
    MappingDenied,
    #[error("legacy credential missing")]
    SourceMissing,
    #[error("credential owner mapping conflict")]
    Conflict,
}

/// Principal-aware host storage. Ordinary lookup never reads the legacy table.
#[async_trait]
pub trait PrincipalCredentialStore: Send + Sync + std::fmt::Debug {
    async fn put_principal(
        &self,
        principal: &CredentialPrincipal,
        record: CredentialRecord,
    ) -> anyhow::Result<()>;
    async fn get_principal(
        &self,
        principal: &CredentialPrincipal,
        scope: CredentialScope,
        scope_id: &str,
        provider_id: &str,
    ) -> anyhow::Result<Option<CredentialRecord>>;
    async fn list_principal(
        &self,
        principal: &CredentialPrincipal,
        scope: CredentialScope,
        scope_id: &str,
    ) -> anyhow::Result<Vec<CredentialRecord>>;
    async fn delete_principal(
        &self,
        principal: &CredentialPrincipal,
        scope: CredentialScope,
        scope_id: &str,
        provider_id: &str,
    ) -> anyhow::Result<bool>;
    async fn migrate_legacy(
        &self,
        principal: &CredentialPrincipal,
        authorization: &CredentialMigrationAuthorization,
    ) -> anyhow::Result<CredentialMigrationReceipt>;
}

fn validate_scope(
    principal: &CredentialPrincipal,
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
    principal: &CredentialPrincipal,
    scope: CredentialScope,
    scope_id: &str,
    provider: &str,
) -> String {
    serde_json::json!([principal.storage_key(), scope.as_str(), scope_id, provider]).to_string()
}

fn receipt(
    principal: &CredentialPrincipal,
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
    })
}

fn same_mapping(previous: &CredentialMigrationReceipt, next: &CredentialMigrationReceipt) -> bool {
    previous.destination_owner_key == next.destination_owner_key
        && previous.destination_scope_id == next.destination_scope_id
        && previous.approval_id == next.approval_id
}

#[derive(Debug, Default)]
pub(super) struct PrincipalMemoryState {
    rows: HashMap<String, CredentialRecord>,
    migrations: HashMap<String, CredentialMigrationReceipt>,
}

#[async_trait]
impl PrincipalCredentialStore for InMemoryCredentialStore {
    async fn put_principal(
        &self,
        principal: &CredentialPrincipal,
        mut record: CredentialRecord,
    ) -> anyhow::Result<()> {
        validate_scope(principal, record.scope, &record.scope_id)?;
        let key = principal_key(
            principal,
            record.scope,
            &record.scope_id,
            &record.provider_id,
        );
        let mut state = self.principals.write().await;
        if let Some(old) = state.rows.get(&key) {
            record.created_at = old.created_at;
        }
        state.rows.insert(key, record);
        Ok(())
    }

    async fn get_principal(
        &self,
        principal: &CredentialPrincipal,
        scope: CredentialScope,
        scope_id: &str,
        provider_id: &str,
    ) -> anyhow::Result<Option<CredentialRecord>> {
        validate_scope(principal, scope, scope_id)?;
        Ok(self
            .principals
            .read()
            .await
            .rows
            .get(&principal_key(principal, scope, scope_id, provider_id))
            .cloned())
    }

    async fn list_principal(
        &self,
        principal: &CredentialPrincipal,
        scope: CredentialScope,
        scope_id: &str,
    ) -> anyhow::Result<Vec<CredentialRecord>> {
        validate_scope(principal, scope, scope_id)?;
        let state = self.principals.read().await;
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
        principal: &CredentialPrincipal,
        scope: CredentialScope,
        scope_id: &str,
        provider_id: &str,
    ) -> anyhow::Result<bool> {
        validate_scope(principal, scope, scope_id)?;
        Ok(self
            .principals
            .write()
            .await
            .rows
            .remove(&principal_key(principal, scope, scope_id, provider_id))
            .is_some())
    }

    async fn migrate_legacy(
        &self,
        principal: &CredentialPrincipal,
        authorization: &CredentialMigrationAuthorization,
    ) -> anyhow::Result<CredentialMigrationReceipt> {
        let next = receipt(principal, authorization)?;
        let source = source_key(next.scope, &next.legacy_scope_id, &next.provider_id);
        let mut state = self.principals.write().await;
        if let Some(previous) = state.migrations.get(&source) {
            if !same_mapping(previous, &next) {
                return Err(CredentialMigrationError::Conflict.into());
            }
            return Ok(previous.clone());
        }
        let key = principal_key(
            principal,
            next.scope,
            &next.destination_scope_id,
            &next.provider_id,
        );
        if state.rows.contains_key(&key) {
            return Err(CredentialMigrationError::Conflict.into());
        }
        let mut row = self
            .rows
            .read()
            .await
            .get(&(
                next.scope.as_str().to_owned(),
                next.legacy_scope_id.clone(),
                next.provider_id.clone(),
            ))
            .cloned()
            .ok_or(CredentialMigrationError::SourceMissing)?;
        row.scope_id.clone_from(&next.destination_scope_id);
        state.rows.insert(key, row);
        state.migrations.insert(source, next.clone());
        Ok(next)
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
fn migration_query_error(error: surrealdb::Error) -> anyhow::Error {
    let message = error.to_string();
    if message.contains("uar_credential_source_missing") {
        CredentialMigrationError::SourceMissing.into()
    } else if message.contains("uar_credential_mapping_conflict") {
        CredentialMigrationError::Conflict.into()
    } else {
        anyhow::anyhow!("credential migration persistence failed")
    }
}

#[cfg(feature = "surreal-backend")]
#[async_trait]
impl PrincipalCredentialStore for SurrealCredentialStore {
    async fn put_principal(
        &self,
        principal: &CredentialPrincipal,
        record: CredentialRecord,
    ) -> anyhow::Result<()> {
        use crate::uar::persistence::providers::surreal::to_db_value;
        validate_scope(principal, record.scope, &record.scope_id)?;
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
        principal: &CredentialPrincipal,
        scope: CredentialScope,
        scope_id: &str,
        provider_id: &str,
    ) -> anyhow::Result<Option<CredentialRecord>> {
        validate_scope(principal, scope, scope_id)?;
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
        principal: &CredentialPrincipal,
        scope: CredentialScope,
        scope_id: &str,
    ) -> anyhow::Result<Vec<CredentialRecord>> {
        validate_scope(principal, scope, scope_id)?;
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
        principal: &CredentialPrincipal,
        scope: CredentialScope,
        scope_id: &str,
        provider_id: &str,
    ) -> anyhow::Result<bool> {
        validate_scope(principal, scope, scope_id)?;
        let mut response = self.db.query("DELETE type::record('provider_principal_credentials', $rid) WHERE owner_key = $owner AND scope = $scope AND scope_id = $scope_id AND provider_id = $provider RETURN BEFORE")
            .bind(("rid", principal_key(principal, scope, scope_id, provider_id)))
            .bind(("owner", principal.storage_key())).bind(("scope", scope.as_str().to_owned()))
            .bind(("scope_id", scope_id.to_owned())).bind(("provider", provider_id.to_owned())).await?.check()?;
        let values: Vec<surrealdb::types::Value> = response.take(0)?;
        Ok(!values.is_empty())
    }

    async fn migrate_legacy(
        &self,
        principal: &CredentialPrincipal,
        authorization: &CredentialMigrationAuthorization,
    ) -> anyhow::Result<CredentialMigrationReceipt> {
        use crate::uar::persistence::providers::surreal::to_db_value;
        let next = receipt(principal, authorization)?;
        let source = source_key(next.scope, &next.legacy_scope_id, &next.provider_id);
        let payload = to_db_value(&next)?;
        let mut response = self.db.query("BEGIN TRANSACTION;
            LET $journal = (SELECT * FROM type::record('provider_credential_owner_migrations', $source))[0];
            IF $journal != NONE {
                IF $journal.destination_owner_key != $owner OR $journal.destination_scope_id != $destination_scope OR $journal.approval_id != $approval { THROW 'uar_credential_mapping_conflict'; };
            } ELSE {
                LET $old = (SELECT * FROM type::record('provider_credentials', $legacy))[0];
                IF $old = NONE OR $old.scope != $scope OR $old.scope_id != $legacy_scope OR $old.provider_id != $provider { THROW 'uar_credential_source_missing'; };
                LET $existing = (SELECT * FROM type::record('provider_principal_credentials', $destination))[0];
                IF $existing != NONE { THROW 'uar_credential_mapping_conflict'; };
                CREATE type::record('provider_principal_credentials', $destination) SET owner_key = $owner, scope = $scope, scope_id = $destination_scope, provider_id = $provider, api_key_encrypted = $old.api_key_encrypted, api_key_hint = $old.api_key_hint, created_at = $old.created_at, updated_at = $old.updated_at;
                CREATE type::record('provider_credential_owner_migrations', $source) CONTENT $receipt;
            };
            COMMIT TRANSACTION;")
            .bind(("source", source.clone()))
            .bind(("legacy", SurrealCredentialStore::record_id(next.scope, &next.legacy_scope_id, &next.provider_id)))
            .bind(("destination", principal_key(principal, next.scope, &next.destination_scope_id, &next.provider_id)))
            .bind(("owner", next.destination_owner_key.clone())).bind(("scope", next.scope.as_str().to_owned()))
            .bind(("legacy_scope", next.legacy_scope_id.clone())).bind(("destination_scope", next.destination_scope_id.clone()))
            .bind(("provider", next.provider_id.clone())).bind(("approval", next.approval_id.clone()))
            .bind(("receipt", payload)).await.map_err(migration_query_error)?;
        // An aborted transaction may report cancellation before its named THROW.
        // Preserve our ownership errors without exposing backend payloads.
        let errors = response.take_errors();
        if !errors.is_empty() {
            return Err(errors
                .into_values()
                .map(migration_query_error)
                .find(|error| error.downcast_ref::<CredentialMigrationError>().is_some())
                .unwrap_or_else(|| anyhow::anyhow!("credential migration persistence failed")));
        }
        let mut response = self
            .db
            .query("SELECT * FROM type::record('provider_credential_owner_migrations', $source)")
            .bind(("source", source))
            .await?
            .check()?;
        let values: Vec<surrealdb::types::Value> = response.take(0)?;
        let value = values
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("credential migration receipt missing"))?;
        serde_json::from_value(crate::uar::persistence::providers::surreal::surreal_to_json(value)?)
            .map_err(|_| anyhow::anyhow!("invalid credential migration receipt"))
    }
}

#[cfg(feature = "sqlx")]
use super::store::{PostgresCredentialStore, row_to_record};

#[cfg(feature = "sqlx")]
#[async_trait]
impl PrincipalCredentialStore for PostgresCredentialStore {
    async fn put_principal(
        &self,
        principal: &CredentialPrincipal,
        record: CredentialRecord,
    ) -> anyhow::Result<()> {
        validate_scope(principal, record.scope, &record.scope_id)?;
        sqlx::query("INSERT INTO provider_principal_credentials (owner_key, scope, scope_id, provider_id, api_key_encrypted, api_key_hint, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT (owner_key, scope, scope_id, provider_id) DO UPDATE SET api_key_encrypted = EXCLUDED.api_key_encrypted, api_key_hint = EXCLUDED.api_key_hint, updated_at = EXCLUDED.updated_at")
            .bind(principal.storage_key()).bind(record.scope.as_str()).bind(&record.scope_id)
            .bind(&record.provider_id).bind(&record.api_key_encrypted).bind(&record.api_key_hint)
            .bind(record.created_at).bind(record.updated_at).execute(&self.pool).await?;
        Ok(())
    }

    async fn get_principal(
        &self,
        principal: &CredentialPrincipal,
        scope: CredentialScope,
        scope_id: &str,
        provider_id: &str,
    ) -> anyhow::Result<Option<CredentialRecord>> {
        validate_scope(principal, scope, scope_id)?;
        let row = sqlx::query("SELECT * FROM provider_principal_credentials WHERE owner_key = $1 AND scope = $2 AND scope_id = $3 AND provider_id = $4")
            .bind(principal.storage_key()).bind(scope.as_str()).bind(scope_id).bind(provider_id)
            .fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_record))
    }

    async fn list_principal(
        &self,
        principal: &CredentialPrincipal,
        scope: CredentialScope,
        scope_id: &str,
    ) -> anyhow::Result<Vec<CredentialRecord>> {
        validate_scope(principal, scope, scope_id)?;
        let rows = sqlx::query("SELECT * FROM provider_principal_credentials WHERE owner_key = $1 AND scope = $2 AND scope_id = $3 ORDER BY provider_id")
            .bind(principal.storage_key()).bind(scope.as_str()).bind(scope_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_record).collect())
    }

    async fn delete_principal(
        &self,
        principal: &CredentialPrincipal,
        scope: CredentialScope,
        scope_id: &str,
        provider_id: &str,
    ) -> anyhow::Result<bool> {
        validate_scope(principal, scope, scope_id)?;
        Ok(sqlx::query("DELETE FROM provider_principal_credentials WHERE owner_key = $1 AND scope = $2 AND scope_id = $3 AND provider_id = $4")
            .bind(principal.storage_key()).bind(scope.as_str()).bind(scope_id).bind(provider_id)
            .execute(&self.pool).await?.rows_affected() != 0)
    }

    async fn migrate_legacy(
        &self,
        principal: &CredentialPrincipal,
        authorization: &CredentialMigrationAuthorization,
    ) -> anyhow::Result<CredentialMigrationReceipt> {
        use sqlx::Row;
        let next = receipt(principal, authorization)?;
        let source = source_key(next.scope, &next.legacy_scope_id, &next.provider_id);
        let mut tx = self.pool.begin().await?;
        // Unique source ownership serializes simultaneous mappings on all hosts.
        let inserted = sqlx::query("INSERT INTO provider_credential_owner_migrations (source_key, receipt) VALUES ($1,$2) ON CONFLICT (source_key) DO NOTHING")
            .bind(&source).bind(serde_json::to_value(&next)?).execute(&mut *tx).await?.rows_affected() != 0;
        let row = sqlx::query("SELECT receipt FROM provider_credential_owner_migrations WHERE source_key = $1 FOR UPDATE")
            .bind(&source).fetch_one(&mut *tx).await?;
        let previous: CredentialMigrationReceipt = serde_json::from_value(row.try_get("receipt")?)?;
        if !same_mapping(&previous, &next) {
            return Err(CredentialMigrationError::Conflict.into());
        }
        if inserted {
            let row = sqlx::query("SELECT * FROM provider_credentials WHERE scope = $1 AND scope_id = $2 AND provider_id = $3 FOR UPDATE")
                .bind(next.scope.as_str()).bind(&next.legacy_scope_id).bind(&next.provider_id)
                .fetch_optional(&mut *tx).await?.ok_or(CredentialMigrationError::SourceMissing)?;
            let old = row_to_record(&row);
            let copied = sqlx::query("INSERT INTO provider_principal_credentials (owner_key, scope, scope_id, provider_id, api_key_encrypted, api_key_hint, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT (owner_key, scope, scope_id, provider_id) DO NOTHING")
                .bind(&next.destination_owner_key).bind(next.scope.as_str()).bind(&next.destination_scope_id)
                .bind(&next.provider_id).bind(&old.api_key_encrypted).bind(&old.api_key_hint)
                .bind(old.created_at).bind(old.updated_at).execute(&mut *tx).await?.rows_affected() != 0;
            if !copied {
                return Err(CredentialMigrationError::Conflict.into());
            }
        }
        tx.commit().await?;
        Ok(previous)
    }
}
