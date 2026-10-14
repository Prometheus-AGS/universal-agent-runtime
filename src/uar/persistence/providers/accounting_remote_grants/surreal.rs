use super::SurrealDbProvider;
use crate::uar::api::a2a::contract::UarUsageGrant;
use crate::uar::persistence::accounting::{AccountingError, RemoteGrantActivity, RemoteGrantState};

async fn read(store: &SurrealDbProvider, key: &str) -> anyhow::Result<Option<RemoteGrantState>> {
    let mut response = store.db.query("SELECT VALUE data FROM type::record('accounting_remote_grants',$key)")
        .bind(("key", key.to_owned())).await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    rows.first().map(|row| Ok(serde_json::from_str(row)?)).transpose()
}

pub(super) async fn admit(store: &SurrealDbProvider, owner: &str, id: &str,
    grant: &UarUsageGrant, activity: RemoteGrantActivity) -> anyhow::Result<RemoteGrantState> {
    anyhow::ensure!(store.durable_instances, "durable_accounting_unsupported");
    let key = RemoteGrantState::key(owner, id);
    let before = match read(store, &key).await? {
        Some(before) => before,
        None => {
            let initial = RemoteGrantState::admit(None, owner, id, grant, RemoteGrantActivity::Inspect, chrono::Utc::now())?;
            let result = store.db.query("CREATE type::record('accounting_remote_grants',$key) SET data=$data")
                .bind(("key", key.clone())).bind(("data", serde_json::to_string(&initial)?)).await?.check();
            match result {
                Ok(_) => initial,
                Err(error) => read(store, &key).await?.ok_or_else(|| anyhow::Error::from(error))?,
            }
        }
    };
    let after = RemoteGrantState::admit(Some(&before), owner, id, grant, activity, chrono::Utc::now())?;
    let mut response = store.db.query("UPDATE type::record('accounting_remote_grants',$key) SET data=$after WHERE data=$before RETURN VALUE data")
        .bind(("key", key)).bind(("before", serde_json::to_string(&before)?))
        .bind(("after", serde_json::to_string(&after)?)).await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    if rows.len() != 1 { return Err(AccountingError::Conflict.into()); }
    Ok(after)
}
