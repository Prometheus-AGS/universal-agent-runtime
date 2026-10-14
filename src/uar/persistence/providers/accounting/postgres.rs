use super::PostgresProvider;
use crate::uar::persistence::accounting::{self, AccountingCall, AccountingError, AccountingScope, CallAdmission, CallUsage, ScopeIdentity, ScopeLimits, ScopeUpdate};
use anyhow::Result;
use sqlx::Row;

pub(super) async fn scope(store: &PostgresProvider, id: &ScopeIdentity) -> Result<Option<AccountingScope>> {
    sqlx::query("SELECT data FROM accounting_scopes WHERE key=$1").bind(id.key()).fetch_optional(&store.pool).await?
        .map(|r| Ok(serde_json::from_value(r.try_get("data")?)?)).transpose()
}
pub(super) async fn ensure(store: &PostgresProvider, id: &ScopeIdentity, limits: &ScopeLimits) -> Result<AccountingScope> {
    limits.validate()?;
    let mut tx = store.pool.begin().await?;
    let legacy = sqlx::query("SELECT 1 FROM cost_ledger WHERE scope=$1 AND scope_id=$2 LIMIT 1")
        .bind(&id.scope).bind(&id.scope_id).fetch_optional(&mut *tx).await?.is_some();
    let initial = AccountingScope::new(id.clone(), limits.clone(), legacy);
    sqlx::query("INSERT INTO accounting_scopes (key,data) VALUES ($1,$2) ON CONFLICT DO NOTHING")
        .bind(id.key()).bind(serde_json::to_value(initial)?).execute(&mut *tx).await?;
    let row = sqlx::query("SELECT data FROM accounting_scopes WHERE key=$1").bind(id.key()).fetch_one(&mut *tx).await?;
    let record = serde_json::from_value(row.try_get("data")?)?;
    tx.commit().await?;
    Ok(record)
}
async fn lock_scopes(tx: &mut sqlx::PgConnection, admission: &CallAdmission) -> Result<Vec<AccountingScope>> {
    let mut keys: Vec<_> = admission.scopes.iter().map(|s| s.identity.key()).collect();
    keys.sort();
    let mut scopes = Vec::new();
    for key in keys {
        let row = sqlx::query("SELECT data FROM accounting_scopes WHERE key=$1 FOR UPDATE").bind(key)
            .fetch_optional(&mut *tx).await?.ok_or(AccountingError::Missing)?;
        scopes.push(serde_json::from_value(row.try_get("data")?)?);
    }
    Ok(scopes)
}
async fn write_scopes(tx: &mut sqlx::PgConnection, scopes: &[AccountingScope]) -> Result<()> {
    for scope in scopes {
        sqlx::query("UPDATE accounting_scopes SET data=$2 WHERE key=$1").bind(scope.identity.key())
            .bind(serde_json::to_value(scope)?).execute(&mut *tx).await?;
    }
    Ok(())
}
pub(super) async fn admit(store: &PostgresProvider, admission: &CallAdmission) -> Result<AccountingCall> {
    admission.validate()?;
    let mut tx = store.pool.begin().await?;
    let mut scopes = lock_scopes(&mut tx, admission).await?;
    if let Some(row) = sqlx::query("SELECT data FROM accounting_calls WHERE key=$1").bind(admission.identity.key()).fetch_optional(&mut *tx).await? {
        let existing: AccountingCall = serde_json::from_value(row.try_get("data")?)?;
        if existing.admission != *admission { return Err(AccountingError::IdentityConflict.into()); }
        return Ok(existing);
    }
    if let Some(source) = &admission.source_reservation {
        let row = sqlx::query("SELECT data FROM accounting_calls WHERE key=$1 FOR UPDATE")
            .bind(source.key()).fetch_optional(&mut *tx).await?.ok_or(AccountingError::Missing)?;
        let source_call: AccountingCall = serde_json::from_value(row.try_get("data")?)?;
        if source_call.state != accounting::AccountingState::Pending || source_call.admission.model != "remote-delegation" {
            return Err(AccountingError::Unresolved.into());
        }
    }
    for scope in &mut scopes { accounting::reserve(scope, admission)?; }
    let call = AccountingCall::new(admission.clone());
    sqlx::query("INSERT INTO accounting_calls (key,owner_key,run_id,data) VALUES ($1,$2,$3,$4)")
        .bind(admission.identity.key()).bind(&admission.identity.owner).bind(&admission.identity.run_id)
        .bind(serde_json::to_value(&call)?).execute(&mut *tx).await?;
    write_scopes(&mut tx, &scopes).await?;
    tx.commit().await?;
    Ok(call)
}
pub(super) async fn usage(store: &PostgresProvider, usage: &CallUsage) -> Result<AccountingCall> {
    let mut tx = store.pool.begin().await?;
    let row = sqlx::query("SELECT data FROM accounting_calls WHERE key=$1").bind(usage.identity.key()).fetch_optional(&mut *tx).await?.ok_or(AccountingError::Missing)?;
    let initial: AccountingCall = serde_json::from_value(row.try_get("data")?)?;
    let mut scopes = lock_scopes(&mut tx, &initial.admission).await?;
    let row = sqlx::query("SELECT data FROM accounting_calls WHERE key=$1 FOR UPDATE").bind(usage.identity.key()).fetch_one(&mut *tx).await?;
    let before: AccountingCall = serde_json::from_value(row.try_get("data")?)?;
    let after = accounting::observe(&before, usage)?;
    for scope in &mut scopes { accounting::settle(scope, &before, &after)?; }
    write_scopes(&mut tx, &scopes).await?;
    sqlx::query("UPDATE accounting_calls SET data=$2 WHERE key=$1").bind(usage.identity.key())
        .bind(serde_json::to_value(&after)?).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(after)
}
pub(super) async fn list(store: &PostgresProvider, owner: &str, run: &str) -> Result<Vec<AccountingCall>> {
    sqlx::query("SELECT data FROM accounting_calls WHERE owner_key=$1 AND run_id=$2 ORDER BY key")
        .bind(owner).bind(run).fetch_all(&store.pool).await?.into_iter()
        .map(|row| Ok(serde_json::from_value(row.try_get("data")?)?)).collect()
}
pub(super) async fn update(store: &PostgresProvider, change: &ScopeUpdate) -> Result<AccountingScope> {
    let mut tx = store.pool.begin().await?;
    let row = sqlx::query("SELECT data FROM accounting_scopes WHERE key=$1 FOR UPDATE").bind(change.identity.key()).fetch_optional(&mut *tx).await?.ok_or(AccountingError::Missing)?;
    let before = serde_json::from_value(row.try_get("data")?)?;
    let after = accounting::update(&before, change)?;
    write_scopes(&mut tx, std::slice::from_ref(&after)).await?;
    tx.commit().await?;
    Ok(after)
}

pub(super) async fn bind_delegation(store: &PostgresProvider, link: &accounting::AccountingDelegation) -> Result<()> {
    anyhow::ensure!(!link.contract_digest.is_empty(), "accounting delegation digest required");
    let mut tx = store.pool.begin().await?;
    let row = sqlx::query("SELECT data FROM accounting_calls WHERE key=$1 FOR UPDATE")
        .bind(link.source.key()).fetch_optional(&mut *tx).await?.ok_or(AccountingError::Missing)?;
    let call: AccountingCall = serde_json::from_value(row.try_get("data")?)?;
    if call.admission.identity != link.source || call.admission.model != "remote-delegation" || call.state != accounting::AccountingState::Pending {
        return Err(AccountingError::IdentityConflict.into());
    }
    sqlx::query("INSERT INTO accounting_delegations (digest,data) VALUES ($1,$2) ON CONFLICT DO NOTHING")
        .bind(&link.contract_digest).bind(serde_json::to_value(link)?).execute(&mut *tx).await?;
    let row = sqlx::query("SELECT data FROM accounting_delegations WHERE digest=$1")
        .bind(&link.contract_digest).fetch_one(&mut *tx).await?;
    let existing: accounting::AccountingDelegation = serde_json::from_value(row.try_get("data")?)?;
    if existing != *link { return Err(AccountingError::IdentityConflict.into()); }
    tx.commit().await?;
    Ok(())
}
pub(super) async fn delegation(store: &PostgresProvider, digest: &str) -> Result<Option<AccountingCall>> {
    let Some(row) = sqlx::query("SELECT data FROM accounting_delegations WHERE digest=$1").bind(digest).fetch_optional(&store.pool).await? else { return Ok(None); };
    let link: accounting::AccountingDelegation = serde_json::from_value(row.try_get("data")?)?;
    let row = sqlx::query("SELECT data FROM accounting_calls WHERE key=$1").bind(link.source.key()).fetch_optional(&store.pool).await?.ok_or(AccountingError::Missing)?;
    Ok(Some(serde_json::from_value(row.try_get("data")?)?))
}
