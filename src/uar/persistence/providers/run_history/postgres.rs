use super::PostgresProvider;
use crate::uar::persistence::run_history::{RunHistoryEvent, RunHistoryRecord};
use sqlx::Row;

pub(super) async fn upsert_record(store: &PostgresProvider, record: &RunHistoryRecord) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO durable_run_records (key,owner_key,run_id,data,revision) VALUES ($1,$2,$3,$4,$5)
        ON CONFLICT (key) DO UPDATE SET owner_key=$2, run_id=$3, data=$4, revision=$5")
        .bind(crate::uar::persistence::run_history::record_key(&record.run_id))
        .bind(&record.owner_key)
        .bind(&record.run_id)
        .bind(serde_json::to_value(record)?)
        .bind(record.revision as i64)
        .execute(&store.pool).await?;
    Ok(())
}

pub(super) async fn load_record(store: &PostgresProvider, run_id: &str) -> anyhow::Result<Option<RunHistoryRecord>> {
    let row = sqlx::query("SELECT data FROM durable_run_records WHERE key=$1")
        .bind(crate::uar::persistence::run_history::record_key(run_id))
        .fetch_optional(&store.pool).await?;
    row.map(|row| Ok(serde_json::from_value(row.try_get("data")?)?)).transpose()
}

pub(super) async fn append_event(store: &PostgresProvider, event: &RunHistoryEvent) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO durable_run_events (key,run_id,event_id,data) VALUES ($1,$2,$3,$4)
        ON CONFLICT (key) DO UPDATE SET data=$4")
        .bind(crate::uar::persistence::run_history::event_key(&event.run_id, event.event_id))
        .bind(&event.run_id)
        .bind(event.event_id as i64)
        .bind(serde_json::to_value(event)?)
        .execute(&store.pool).await?;
    Ok(())
}

pub(super) async fn load_events(store: &PostgresProvider, run_id: &str) -> anyhow::Result<Vec<RunHistoryEvent>> {
    sqlx::query("SELECT data FROM durable_run_events WHERE run_id=$1 ORDER BY event_id ASC")
        .bind(run_id).fetch_all(&store.pool).await?.into_iter()
        .map(|row| Ok(serde_json::from_value(row.try_get("data")?)?)).collect()
}
