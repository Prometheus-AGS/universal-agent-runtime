use super::SurrealDbProvider;
use crate::uar::persistence::approval_decisions::{ApprovalRecord, ApprovalState};

pub(super) async fn create(store: &SurrealDbProvider, record: &ApprovalRecord) -> anyhow::Result<()> {
    anyhow::ensure!(record.state == ApprovalState::Pending && record.decision.is_none(), "New approval must be pending");
    store.db.query("CREATE type::record('approval_records', $key) CONTENT $payload")
        .bind(("key", record.storage_key()))
        .bind(("payload", serde_json::json!({"owner_key": record.owner_key, "root_run_id": record.root_run_id,
            "state": "pending", "data": serde_json::to_string(record)?})))
        .await?.check()?;
    Ok(())
}

pub(super) async fn transition(store: &SurrealDbProvider, before: &ApprovalRecord, after: &ApprovalRecord) -> anyhow::Result<bool> {
    before.validate_transition(after)?;
    let mut response = store.db.query("UPDATE type::record('approval_records', $key) SET state = 'resolved', data = $after WHERE state = 'pending' AND data = $before RETURN AFTER")
        .bind(("key", before.storage_key())).bind(("before", serde_json::to_string(before)?))
        .bind(("after", serde_json::to_string(after)?)).await?.check()?;
    let rows: Vec<surrealdb::types::Value> = response.take(0)?;
    Ok(rows.len() == 1)
}

pub(super) async fn list(store: &SurrealDbProvider, owner: &str, run: &str) -> anyhow::Result<Vec<ApprovalRecord>> {
    let mut response = store.db.query("SELECT VALUE data FROM approval_records WHERE owner_key = $owner AND root_run_id = $run")
        .bind(("owner", owner.to_owned())).bind(("run", run.to_owned())).await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    rows.into_iter().map(|row| Ok(serde_json::from_str(&row)?)).collect()
}
