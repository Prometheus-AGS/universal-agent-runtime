use super::PostgresProvider;
use crate::uar::persistence::agui_runs::AguiRunBinding;
use sqlx::Row;

pub(super) async fn create(store: &PostgresProvider, row: &AguiRunBinding, previous: Option<&AguiRunBinding>) -> anyhow::Result<bool> {
    row.validate_create(previous)?;
    let mut tx = store.pool.begin().await?;
    let slot = sqlx::query("INSERT INTO agui_contexts (key,run_key) VALUES ($1,$2) ON CONFLICT DO NOTHING")
        .bind(row.context_key()).bind(row.storage_key()).execute(&mut *tx).await?;
    let current = sqlx::query("SELECT run_key FROM agui_contexts WHERE key=$1 FOR UPDATE")
        .bind(row.context_key()).fetch_one(&mut *tx).await?;
    let current_key: String = current.try_get("run_key")?;
    let matches = match previous {
        Some(previous) if slot.rows_affected() == 0 && current_key == previous.storage_key() => {
            let saved = sqlx::query("SELECT data FROM agui_runs WHERE key=$1 FOR UPDATE")
                .bind(previous.storage_key()).fetch_optional(&mut *tx).await?;
            saved.map(|saved| Ok::<_, anyhow::Error>(serde_json::from_value::<AguiRunBinding>(saved.try_get("data")?)?))
                .transpose()?.as_ref() == Some(previous)
        }
        None => slot.rows_affected() == 1,
        _ => false,
    };
    if !matches { tx.rollback().await?; return Ok(false); }
    let created = sqlx::query("INSERT INTO agui_runs (key,owner_key,agent_id,thread_id,data) VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING")
        .bind(row.storage_key()).bind(&row.owner).bind(&row.agent_id).bind(&row.thread_id)
        .bind(serde_json::to_value(row)?).execute(&mut *tx).await?;
    if created.rows_affected() != 1 { tx.rollback().await?; return Ok(false); }
    sqlx::query("UPDATE agui_contexts SET run_key=$2 WHERE key=$1")
        .bind(row.context_key()).bind(row.storage_key()).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(true)
}
pub(super) async fn load(store: &PostgresProvider, owner: &str, agent: &str, thread: &str, run: &str) -> anyhow::Result<Option<AguiRunBinding>> {
    let row = sqlx::query("SELECT data FROM agui_runs WHERE key=$1 AND owner_key=$2 AND agent_id=$3 AND thread_id=$4")
        .bind(AguiRunBinding::key(owner,agent,thread,run)).bind(owner).bind(agent).bind(thread).fetch_optional(&store.pool).await?;
    row.map(|row| { let row: AguiRunBinding = serde_json::from_value(row.try_get("data")?)?; row.validate()?; Ok(row) }).transpose()
}
pub(super) async fn list(store: &PostgresProvider, owner: &str, agent: &str, thread: &str) -> anyhow::Result<Vec<AguiRunBinding>> {
    sqlx::query("SELECT data FROM agui_runs WHERE owner_key=$1 AND agent_id=$2 AND thread_id=$3")
        .bind(owner).bind(agent).bind(thread).fetch_all(&store.pool).await?.into_iter()
        .map(|row| { let row: AguiRunBinding = serde_json::from_value(row.try_get("data")?)?; row.validate()?; Ok(row) }).collect()
}
pub(super) async fn transition(store: &PostgresProvider, before: &AguiRunBinding, after: &AguiRunBinding) -> anyhow::Result<bool> {
    before.validate_transition(after)?;
    Ok(sqlx::query("UPDATE agui_runs SET data=$2 WHERE key=$1 AND data=$3")
        .bind(before.storage_key()).bind(serde_json::to_value(after)?).bind(serde_json::to_value(before)?)
        .execute(&store.pool).await?.rows_affected() == 1)
}
