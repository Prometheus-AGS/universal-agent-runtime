use super::PostgresProvider;
use crate::uar::persistence::mcp_elicitations::{McpElicitationRecord, ElicitationState};
use sqlx::Row;

pub(super) async fn create(store: &PostgresProvider, row: &McpElicitationRecord) -> anyhow::Result<()> {
    row.validate()?;
    anyhow::ensure!(row.state == ElicitationState::Pending, "elicitation_initial_state");
    sqlx::query("INSERT INTO mcp_elicitations (id,owner_key,run_id,data) VALUES ($1,$2,$3,$4)")
        .bind(&row.id).bind(&row.owner_key).bind(&row.run_id).bind(serde_json::to_value(row)?)
        .execute(&store.pool).await?;
    Ok(())
}
pub(super) async fn list(store: &PostgresProvider, owner: &str, run: &str) -> anyhow::Result<Vec<McpElicitationRecord>> {
    let rows = sqlx::query("SELECT data FROM mcp_elicitations WHERE owner_key=$1 AND run_id=$2 ORDER BY id")
        .bind(owner).bind(run).fetch_all(&store.pool).await?;
    rows.into_iter().map(|row| { let row: McpElicitationRecord = serde_json::from_value(row.try_get("data")?)?; row.validate()?; Ok(row) }).collect()
}
pub(super) async fn transition(store: &PostgresProvider, before: &McpElicitationRecord, after: &McpElicitationRecord) -> anyhow::Result<bool> {
    before.validate_transition(after)?;
    let mut tx = store.pool.begin().await?;
    let saved = sqlx::query("SELECT data FROM mcp_elicitations WHERE id=$1 AND owner_key=$2 AND run_id=$3 FOR UPDATE")
        .bind(&before.id).bind(&before.owner_key).bind(&before.run_id).fetch_optional(&mut *tx).await?;
    let saved = saved.map(|row|row.try_get::<serde_json::Value,_>("data")).transpose()?;
    if saved != Some(serde_json::to_value(before)?) { tx.rollback().await?; return Ok(false); }
    let expired = serde_json::to_value(before.finish(ElicitationState::Expired))?;
    let desired = serde_json::to_value(after)?;
    let row = sqlx::query("UPDATE mcp_elicitations SET data=CASE WHEN $3 AND $4 <= clock_timestamp() THEN $5 ELSE $2 END WHERE id=$1 RETURNING data")
        .bind(&before.id).bind(&desired).bind(after.state == ElicitationState::Answered)
        .bind(before.expires_at).bind(expired).fetch_one(&mut *tx).await?;
    let applied = row.try_get::<serde_json::Value,_>("data")? == desired;
    tx.commit().await?; Ok(applied)
}
