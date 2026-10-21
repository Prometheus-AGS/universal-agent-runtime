use super::SurrealDbProvider;
use crate::uar::persistence::run_history::{RunHistoryEvent, RunHistoryRecord};

pub(super) async fn upsert_record(store: &SurrealDbProvider, record: &RunHistoryRecord) -> anyhow::Result<()> {
    anyhow::ensure!(store.durable_instances, "durable_run_history_unsupported");
    store.db.query("UPSERT type::record('durable_run_records',$key) SET owner_key=$owner_key, run_id=$run_id, data=$data, revision=$revision")
        .bind(("key", crate::uar::persistence::run_history::record_key(&record.run_id)))
        .bind(("owner_key", record.owner_key.clone()))
        .bind(("run_id", record.run_id.clone()))
        .bind(("data", serde_json::to_string(record)?))
        .bind(("revision", record.revision as i64)).await?.check()?;
    Ok(())
}

pub(super) async fn load_record(store: &SurrealDbProvider, run_id: &str) -> anyhow::Result<Option<RunHistoryRecord>> {
    anyhow::ensure!(store.durable_instances, "durable_run_history_unsupported");
    let mut response = store.db.query("SELECT VALUE data FROM type::record('durable_run_records',$key)")
        .bind(("key", crate::uar::persistence::run_history::record_key(run_id))).await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    rows.first().map(|row| Ok(serde_json::from_str(row)?)).transpose()
}

pub(super) async fn append_event(store: &SurrealDbProvider, event: &RunHistoryEvent) -> anyhow::Result<()> {
    anyhow::ensure!(store.durable_instances, "durable_run_history_unsupported");
    store.db.query("UPSERT type::record('durable_run_events',$key) SET run_id=$run_id, event_id=$event_id, data=$data")
        .bind(("key", crate::uar::persistence::run_history::event_key(&event.run_id, event.event_id)))
        .bind(("run_id", event.run_id.clone()))
        .bind(("event_id", event.event_id as i64))
        .bind(("data", serde_json::to_string(event)?)).await?.check()?;
    Ok(())
}

pub(super) async fn load_events(store: &SurrealDbProvider, run_id: &str) -> anyhow::Result<Vec<RunHistoryEvent>> {
    anyhow::ensure!(store.durable_instances, "durable_run_history_unsupported");
    let mut response = store.db.query("SELECT VALUE data FROM durable_run_events WHERE run_id=$run_id ORDER BY event_id ASC")
        .bind(("run_id", run_id.to_owned())).await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    rows.iter().map(|row| Ok(serde_json::from_str(row)?)).collect()
}
