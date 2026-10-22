use super::SurrealDbProvider;
use crate::uar::persistence::accounting::{self, AccountingCall, AccountingError, AccountingScope, CallAdmission, CallUsage, ScopeIdentity, ScopeLimits, ScopeUpdate};
use anyhow::Result;
use serde::Serialize;

pub(super) async fn initialize(db: &surrealdb::Surreal<surrealdb::engine::any::Any>) -> Result<()> {
    db.query(include_str!("../../../../../migrations/surrealdb/durable_accounting.surql")).await?.check()?;
    Ok(())
}
fn durable(store: &SurrealDbProvider) -> Result<()> {
    if !store.durable_instances { return Err(AccountingError::Unsupported.into()); }
    Ok(())
}
async fn read<T: serde::de::DeserializeOwned>(store: &SurrealDbProvider, table: &str, key: String) -> Result<Option<T>> {
    durable(store)?;
    let mut rows = store.db.query("SELECT VALUE data FROM type::record($table,$key)")
        .bind(("table",table.to_owned())).bind(("key",key)).await?.check()?;
    let data: Vec<String> = rows.take(0)?;
    data.first().map(|data| Ok(serde_json::from_str(data)?)).transpose()
}
pub(super) async fn scope(store: &SurrealDbProvider, id: &ScopeIdentity) -> Result<Option<AccountingScope>> {
    read(store,"accounting_scopes",id.key()).await
}
pub(super) async fn ensure(store: &SurrealDbProvider, id: &ScopeIdentity, limits: &ScopeLimits) -> Result<AccountingScope> {
    durable(store)?; limits.validate()?;
    if let Some(existing) = scope(store,id).await? { return Ok(existing); }
    let mut response = store.db.query("SELECT * FROM cost_ledger WHERE scope=$scope AND scope_id=$scope_id LIMIT 1")
        .bind(("scope",id.scope.clone())).bind(("scope_id",id.scope_id.clone())).await?.check()?;
    let legacy: Vec<serde_json::Value> = response.take(0)?;
    let initial = AccountingScope::new(id.clone(),limits.clone(),!legacy.is_empty());
    // Concurrent initialization can lose CREATE; reread the unique row and never overwrite it.
    let result = store.db.query("CREATE type::record('accounting_scopes',$key) SET data=$data")
        .bind(("key",id.key())).bind(("data",serde_json::to_string(&initial)?)).await?.check();
    match result {
        Ok(_) => Ok(initial),
        Err(error) => scope(store,id).await?.ok_or_else(|| error.into()),
    }
}
#[derive(Serialize)]
struct Change { key: String, before: String, after: String }
async fn changes(store: &SurrealDbProvider, admission: &CallAdmission) -> Result<Vec<AccountingScope>> {
    let mut bindings = admission.scopes.clone();
    bindings.sort_by_key(|binding| binding.identity.key());
    let mut scopes = Vec::new();
    for binding in bindings { scopes.push(scope(store,&binding.identity).await?.ok_or(AccountingError::Missing)?); }
    Ok(scopes)
}
fn scope_changes(before: &[AccountingScope], after: &[AccountingScope]) -> Result<Vec<Change>> {
    before.iter().zip(after).map(|(before,after)| Ok(Change { key: before.identity.key(), before: serde_json::to_string(before)?, after: serde_json::to_string(after)? })).collect()
}
fn checked(mut response: surrealdb::IndexedResults) -> Result<()> {
    let errors = response.take_errors();
    if errors.values().any(|e| e.to_string().contains("accounting_revision_conflict")) { return Err(AccountingError::Conflict.into()); }
    // A failed transaction replaces earlier successful statement results with
    // NotExecuted. Prefer its actual failure over those rollback placeholders.
    if let Some((_,error)) = errors.into_iter().min_by_key(|(index,error)| (
        matches!(error.query_details(), Some(surrealdb::types::QueryError::NotExecuted)), *index,
    )) { return Err(error.into()); }
    Ok(())
}
pub(super) async fn admit(store: &SurrealDbProvider, admission: &CallAdmission) -> Result<AccountingCall> {
    durable(store)?; admission.validate()?;
    if let Some(existing) = read::<AccountingCall>(store,"accounting_calls",admission.identity.key()).await? {
        if existing.admission != *admission { return Err(AccountingError::IdentityConflict.into()); }
        return Ok(existing);
    }
    let source = match &admission.source_reservation {
        Some(id) => {
            let source: AccountingCall = read(store,"accounting_calls",id.key()).await?.ok_or(AccountingError::Missing)?;
            if source.state != accounting::AccountingState::Pending || source.admission.model != "remote-delegation" { return Err(AccountingError::Unresolved.into()); }
            Some(source)
        }
        None => None,
    };
    let before = changes(store,admission).await?;
    let mut after = before.clone();
    for scope in &mut after { accounting::reserve(scope,admission)?; }
    let call = AccountingCall::new(admission.clone());
    checked(store.db.query("BEGIN TRANSACTION;
        IF $has_source {
            LET $funding = (UPDATE type::record('accounting_calls',$source_key) SET data=$source_data WHERE data=$source_data RETURN AFTER);
            IF array::len($funding) != 1 { THROW 'accounting_revision_conflict'; };
        };
        FOR $change IN $changes {
            LET $updated = (UPDATE type::record('accounting_scopes',$change.key) SET data=$change.after WHERE data=$change.before RETURN AFTER);
            IF array::len($updated) != 1 { THROW 'accounting_revision_conflict'; };
        };
        CREATE type::record('accounting_calls',$key) SET owner_key=$owner,run_id=$run,data=$data;
        COMMIT TRANSACTION;")
        .bind(("changes",serde_json::to_value(scope_changes(&before,&after)?)?)).bind(("key",admission.identity.key()))
        .bind(("owner",admission.identity.owner.clone())).bind(("run",admission.identity.run_id.clone()))
        .bind(("data",serde_json::to_string(&call)?))
        .bind(("has_source",source.is_some()))
        .bind(("source_key",source.as_ref().map(|call| call.admission.identity.key()).unwrap_or_default()))
        .bind(("source_data",source.as_ref().map(serde_json::to_string).transpose()?.unwrap_or_default())).await?)?;
    Ok(call)
}
pub(super) async fn usage(store: &SurrealDbProvider, usage: &CallUsage) -> Result<AccountingCall> {
    durable(store)?;
    let before_call: AccountingCall = read(store,"accounting_calls",usage.identity.key()).await?.ok_or(AccountingError::Missing)?;
    let after_call = accounting::observe(&before_call,usage)?;
    if before_call == after_call { return Ok(after_call); }
    let before = changes(store,&before_call.admission).await?;
    let mut after = before.clone();
    for scope in &mut after { accounting::settle(scope,&before_call,&after_call)?; }
    let result = checked(store.db.query("BEGIN TRANSACTION;
        FOR $change IN $changes {
            LET $updated = (UPDATE type::record('accounting_scopes',$change.key) SET data=$change.after WHERE data=$change.before RETURN AFTER);
            IF array::len($updated) != 1 { THROW 'accounting_revision_conflict'; };
        };
        LET $updated = (UPDATE type::record('accounting_calls',$key) SET data=$after WHERE data=$before RETURN AFTER);
        IF array::len($updated) != 1 { THROW 'accounting_revision_conflict'; };
        COMMIT TRANSACTION;")
        .bind(("changes",serde_json::to_value(scope_changes(&before,&after)?)?)).bind(("key",usage.identity.key()))
        .bind(("before",serde_json::to_string(&before_call)?)).bind(("after",serde_json::to_string(&after_call)?)).await?);
    if let Err(error) = result {
        // A concurrent identical terminal replay may have committed this exact
        // call and all its scope changes atomically. Read only that original
        // row once; never treat a generic transaction conflict as success and
        // never retry an unknown call or release its reservation by age.
        if after_call.state == accounting::AccountingState::Settled {
            if let Ok(Some(recorded)) = read::<AccountingCall>(store,"accounting_calls",usage.identity.key()).await {
                if recorded == after_call { return Ok(recorded); }
            }
        }
        return Err(error);
    }
    Ok(after_call)
}
pub(super) async fn list(store: &SurrealDbProvider, owner: &str, run: &str) -> Result<Vec<AccountingCall>> {
    durable(store)?;
    let mut response = store.db.query("SELECT VALUE data FROM accounting_calls WHERE owner_key=$owner AND run_id=$run ORDER BY id")
        .bind(("owner",owner.to_owned())).bind(("run",run.to_owned())).await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    rows.into_iter().map(|data| Ok(serde_json::from_str(&data)?)).collect()
}
pub(super) async fn update(store: &SurrealDbProvider, change: &ScopeUpdate) -> Result<AccountingScope> {
    let before = scope(store,&change.identity).await?.ok_or(AccountingError::Missing)?;
    let after = accounting::update(&before,change)?;
    checked(store.db.query("BEGIN TRANSACTION;
        LET $updated = (UPDATE type::record('accounting_scopes',$key) SET data=$after WHERE data=$before RETURN AFTER);
        IF array::len($updated) != 1 { THROW 'accounting_revision_conflict'; };
        COMMIT TRANSACTION;")
        .bind(("key",change.identity.key())).bind(("before",serde_json::to_string(&before)?)).bind(("after",serde_json::to_string(&after)?)).await?)?;
    Ok(after)
}

pub(super) async fn bind_delegation(store: &SurrealDbProvider, link: &accounting::AccountingDelegation) -> Result<()> {
    durable(store)?;
    anyhow::ensure!(!link.contract_digest.is_empty(), "accounting delegation digest required");
    let call: AccountingCall = read(store,"accounting_calls",link.source.key()).await?.ok_or(AccountingError::Missing)?;
    if call.admission.identity != link.source || call.admission.model != "remote-delegation" || call.state != accounting::AccountingState::Pending {
        return Err(AccountingError::IdentityConflict.into());
    }
    if let Some(existing) = read::<accounting::AccountingDelegation>(store,"accounting_delegations",link.contract_digest.clone()).await? {
        if existing != *link { return Err(AccountingError::IdentityConflict.into()); }
        return Ok(());
    }
    checked(store.db.query("BEGIN TRANSACTION;
        LET $changed_source = (UPDATE type::record('accounting_calls',$source) SET data=$before WHERE data=$before RETURN AFTER);
        IF array::len($changed_source) != 1 { THROW 'accounting_revision_conflict'; };
        CREATE type::record('accounting_delegations',$digest) SET data=$data;
        COMMIT TRANSACTION;")
        .bind(("source",link.source.key())).bind(("before",serde_json::to_string(&call)?))
        .bind(("digest",link.contract_digest.clone())).bind(("data",serde_json::to_string(link)?)).await?)
}
pub(super) async fn delegation(store: &SurrealDbProvider, digest: &str) -> Result<Option<AccountingCall>> {
    let Some(link) = read::<accounting::AccountingDelegation>(store,"accounting_delegations",digest.to_owned()).await? else { return Ok(None); };
    let call = read(store,"accounting_calls",link.source.key()).await?.ok_or(AccountingError::Missing)?;
    Ok(Some(call))
}
