use super::PostgresProvider;
use crate::uar::api::a2a::contract::UarUsageGrant;
use crate::uar::persistence::accounting::{RemoteGrantActivity, RemoteGrantState};
use sqlx::Row;

pub(super) async fn admit(store: &PostgresProvider, owner: &str, id: &str,
    grant: &UarUsageGrant, activity: RemoteGrantActivity) -> anyhow::Result<RemoteGrantState> {
    let key = RemoteGrantState::key(owner, id);
    let initial = RemoteGrantState::admit(None, owner, id, grant, RemoteGrantActivity::Inspect, chrono::Utc::now())?;
    let mut transaction = store.pool.begin().await?;
    sqlx::query("INSERT INTO accounting_remote_grants (key,data) VALUES ($1,$2) ON CONFLICT DO NOTHING")
        .bind(&key).bind(serde_json::to_value(&initial)?).execute(&mut *transaction).await?;
    let row = sqlx::query("SELECT data FROM accounting_remote_grants WHERE key=$1 FOR UPDATE")
        .bind(&key).fetch_one(&mut *transaction).await?;
    let before: RemoteGrantState = serde_json::from_value(row.try_get("data")?)?;
    let after = RemoteGrantState::admit(Some(&before), owner, id, grant, activity, chrono::Utc::now())?;
    sqlx::query("UPDATE accounting_remote_grants SET data=$2 WHERE key=$1")
        .bind(&key).bind(serde_json::to_value(&after)?).execute(&mut *transaction).await?;
    transaction.commit().await?;
    Ok(after)
}
