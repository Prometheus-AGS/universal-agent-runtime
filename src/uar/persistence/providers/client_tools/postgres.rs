use super::PostgresProvider;
use crate::uar::persistence::client_tools::ClientToolContinuation;
use sqlx::Row;

pub(super) async fn create(store: &PostgresProvider, row: &ClientToolContinuation) -> anyhow::Result<bool> {
    row.validate_create()?;
    Ok(sqlx::query("INSERT INTO client_tool_continuations (key,owner_key,agent_id,source_run_id,data) VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING")
        .bind(row.storage_key()).bind(&row.owner_key).bind(&row.agent_id).bind(&row.source_run_id).bind(serde_json::to_value(row)?)
        .execute(&store.pool).await?.rows_affected() == 1)
}
pub(super) async fn load(store: &PostgresProvider, owner: &str, tenant: Option<&str>, agent: &str, batch: &str) -> anyhow::Result<Option<ClientToolContinuation>> {
    let row = sqlx::query("SELECT data FROM client_tool_continuations WHERE key=$1 AND owner_key=$2 AND agent_id=$3")
        .bind(ClientToolContinuation::key(owner,tenant,agent,batch)).bind(owner).bind(agent)
        .fetch_optional(&store.pool).await?;
    row.map(|row| {
        let row: ClientToolContinuation = serde_json::from_value(row.try_get("data")?)?;
        row.validate()?;
        anyhow::ensure!(row.owner_key == owner && row.tenant_id.as_deref() == tenant && row.agent_id == agent && row.batch_id == batch, "client_tool_continuation_binding_conflict");
        Ok(row)
    }).transpose()
}
pub(super) async fn transition(store: &PostgresProvider, before: &ClientToolContinuation, after: &ClientToolContinuation) -> anyhow::Result<bool> {
    before.validate_transition(after)?;
    Ok(sqlx::query("UPDATE client_tool_continuations SET data=$2 WHERE key=$1 AND data=$3")
        .bind(before.storage_key()).bind(serde_json::to_value(after)?).bind(serde_json::to_value(before)?)
        .execute(&store.pool).await?.rows_affected() == 1)
}

pub(super) async fn load_for_run(store: &PostgresProvider, owner: &str, tenant: Option<&str>, agent: &str, run: &str) -> anyhow::Result<Option<ClientToolContinuation>> {
    let rows = sqlx::query("SELECT data FROM client_tool_continuations WHERE owner_key=$1 AND agent_id=$2 AND source_run_id=$3")
        .bind(owner).bind(agent).bind(run).fetch_all(&store.pool).await?;
    let mut matched = None;
    for row in rows {
        let row: ClientToolContinuation = serde_json::from_value(row.try_get("data")?)?;
        row.validate()?;
        if row.tenant_id.as_deref() == tenant {
            anyhow::ensure!(row.owner_key == owner && row.agent_id == agent && row.source_run_id == run && matched.is_none(), "client_tool_run_binding_conflict");
            matched = Some(row);
        }
    }
    Ok(matched)
}
