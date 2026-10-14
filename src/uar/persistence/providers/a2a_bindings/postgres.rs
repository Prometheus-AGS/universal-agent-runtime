use super::PostgresProvider;
use crate::uar::api::a2a::durable::A2aBinding;
use sqlx::Row;

pub(super) async fn create(store: &PostgresProvider, record: &A2aBinding) -> anyhow::Result<bool> {
    record.validate()?;
    anyhow::ensure!(record.revision == 1, "Initial A2A revision must be one");
    let mut transaction = store.pool.begin().await?;
    let slot = sqlx::query("INSERT INTO a2a_contexts (owner_key,agent_id,context_id,task_id) VALUES ($1,$2,$3,$4) ON CONFLICT DO NOTHING")
        .bind(&record.owner_key).bind(&record.agent_id).bind(&record.context_id).bind(&record.task_id)
        .execute(&mut *transaction).await?;
    let current = sqlx::query("SELECT task_id FROM a2a_contexts WHERE owner_key=$1 AND agent_id=$2 AND context_id=$3 FOR UPDATE")
        .bind(&record.owner_key).bind(&record.agent_id).bind(&record.context_id)
        .fetch_one(&mut *transaction).await?;
    if slot.rows_affected() == 0 {
        let task_id: String = current.try_get("task_id")?;
        let previous = sqlx::query("SELECT data FROM a2a_bindings WHERE owner_key=$1 AND agent_id=$2 AND task_id=$3")
            .bind(&record.owner_key).bind(&record.agent_id).bind(task_id).fetch_optional(&mut *transaction).await?;
        let previous: Option<A2aBinding> = previous.map(|row| Ok::<_, anyhow::Error>(serde_json::from_value(row.try_get("data")?)?)).transpose()?;
        if !previous.as_ref().is_some_and(A2aBinding::context_closed) {
            transaction.rollback().await?;
            return Ok(false);
        }
    }
    let result = sqlx::query("INSERT INTO a2a_bindings (owner_key, agent_id, task_id, data) VALUES ($1,$2,$3,$4) ON CONFLICT DO NOTHING")
        .bind(&record.owner_key).bind(&record.agent_id).bind(&record.task_id)
        .bind(serde_json::to_value(record)?).execute(&mut *transaction).await?;
    if result.rows_affected() != 1 { transaction.rollback().await?; return Ok(false); }
    sqlx::query("UPDATE a2a_contexts SET task_id=$4 WHERE owner_key=$1 AND agent_id=$2 AND context_id=$3")
        .bind(&record.owner_key).bind(&record.agent_id).bind(&record.context_id).bind(&record.task_id)
        .execute(&mut *transaction).await?;
    transaction.commit().await?;
    Ok(true)
}

pub(super) async fn load(store: &PostgresProvider, owner: &str, agent: &str, task: &str) -> anyhow::Result<Option<A2aBinding>> {
    let row = sqlx::query("SELECT data FROM a2a_bindings WHERE owner_key=$1 AND agent_id=$2 AND task_id=$3")
        .bind(owner).bind(agent).bind(task).fetch_optional(&store.pool).await?;
    row.map(|row| {
        let record: A2aBinding = serde_json::from_value(row.try_get("data")?)?;
        record.validate()?;
        anyhow::ensure!(record.owner_key == owner && record.agent_id == agent && record.task_id == task, "A2A persisted scope mismatch");
        Ok(record)
    }).transpose()
}

pub(super) async fn transition(store: &PostgresProvider, before: &A2aBinding, after: &A2aBinding) -> anyhow::Result<bool> {
    before.validate_transition(after)?;
    let result = sqlx::query("UPDATE a2a_bindings SET data=$4 WHERE owner_key=$1 AND agent_id=$2 AND task_id=$3 AND data=$5")
        .bind(&before.owner_key).bind(&before.agent_id).bind(&before.task_id)
        .bind(serde_json::to_value(after)?).bind(serde_json::to_value(before)?)
        .execute(&store.pool).await?;
    Ok(result.rows_affected() == 1)
}

pub(super) async fn list(store: &PostgresProvider, owner: &str, agent: &str) -> anyhow::Result<Vec<A2aBinding>> {
    let rows = sqlx::query("SELECT data FROM a2a_bindings WHERE owner_key=$1 AND agent_id=$2 ORDER BY task_id")
        .bind(owner).bind(agent).fetch_all(&store.pool).await?;
    rows.into_iter().map(|row| {
        let record: A2aBinding = serde_json::from_value(row.try_get("data")?)?;
        record.validate()?;
        anyhow::ensure!(record.owner_key == owner && record.agent_id == agent, "A2A persisted scope mismatch");
        Ok(record)
    }).collect()
}
