use super::SurrealDbProvider;
use crate::uar::persistence::agui_runs::AguiRunBinding;

pub(super) async fn create(store: &SurrealDbProvider, row: &AguiRunBinding, previous: Option<&AguiRunBinding>) -> anyhow::Result<bool> {
    anyhow::ensure!(store.durable_instances, "durable_agui_unsupported");
    row.validate_create(previous)?;
    let response = store.db.query("BEGIN TRANSACTION;
        LET $slot = (SELECT * FROM type::record('agui_contexts',$context))[0];
        IF $has_previous {
            IF $slot == NONE OR $slot.run_key != $previous_key { THROW 'agui_context_conflict'; };
            LET $old = (SELECT * FROM type::record('agui_runs',$previous_key))[0];
            IF $old == NONE OR $old.data != $previous { THROW 'agui_context_conflict'; };
        } ELSE { IF $slot != NONE { THROW 'agui_context_conflict'; }; };
        IF (SELECT * FROM type::record('agui_runs',$key))[0] != NONE { THROW 'agui_context_conflict'; };
        CREATE type::record('agui_runs',$key) CONTENT $payload;
        UPSERT type::record('agui_contexts',$context) SET run_key=$key;
        COMMIT TRANSACTION;")
        .bind(("key",row.storage_key())).bind(("context",row.context_key()))
        .bind(("has_previous",previous.is_some())).bind(("previous_key",previous.map(AguiRunBinding::storage_key).unwrap_or_default()))
        .bind(("previous",previous.map(serde_json::to_string).transpose()?.unwrap_or_default()))
        .bind(("payload",serde_json::json!({"owner_key":row.owner,"agent_id":row.agent_id,"thread_id":row.thread_id,"data":serde_json::to_string(row)?}))).await?;
    match response.check() { Ok(_) => Ok(true), Err(error) if error.to_string().contains("agui_context_conflict") => Ok(false), Err(error) => Err(error.into()) }
}
pub(super) async fn load(store: &SurrealDbProvider, owner: &str, agent: &str, thread: &str, run: &str) -> anyhow::Result<Option<AguiRunBinding>> {
    anyhow::ensure!(store.durable_instances, "durable_agui_unsupported");
    let mut response = store.db.query("SELECT VALUE data FROM type::record('agui_runs',$key) WHERE owner_key=$owner AND agent_id=$agent AND thread_id=$thread")
        .bind(("key",AguiRunBinding::key(owner,agent,thread,run))).bind(("owner",owner.to_owned())).bind(("agent",agent.to_owned())).bind(("thread",thread.to_owned())).await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    rows.first().map(|row| { let row: AguiRunBinding = serde_json::from_str(row)?; row.validate()?; Ok(row) }).transpose()
}
pub(super) async fn list(store: &SurrealDbProvider, owner: &str, agent: &str, thread: &str) -> anyhow::Result<Vec<AguiRunBinding>> {
    anyhow::ensure!(store.durable_instances, "durable_agui_unsupported");
    let mut response = store.db.query("SELECT VALUE data FROM agui_runs WHERE owner_key=$owner AND agent_id=$agent AND thread_id=$thread")
        .bind(("owner",owner.to_owned())).bind(("agent",agent.to_owned())).bind(("thread",thread.to_owned())).await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    rows.iter().map(|row| { let row: AguiRunBinding = serde_json::from_str(row)?; row.validate()?; Ok(row) }).collect()
}
pub(super) async fn transition(store: &SurrealDbProvider, before: &AguiRunBinding, after: &AguiRunBinding) -> anyhow::Result<bool> {
    anyhow::ensure!(store.durable_instances, "durable_agui_unsupported");
    before.validate_transition(after)?;
    let response = store.db.query("UPDATE type::record('agui_runs',$key) SET data=$after WHERE data=$before RETURN VALUE data")
        .bind(("key",before.storage_key())).bind(("before",serde_json::to_string(before)?)).bind(("after",serde_json::to_string(after)?)).await?;
    // A concurrent writer that commits the same row first makes this implicit transaction fail at COMMIT with the
    // typed `QueryError::TransactionConflict` (surrealdb-types 3.3.0 error.rs:924; surrealkv `TransactionWriteConflict`
    // -> kvs `TransactionConflict`, surrealdb-kvs-surrealkv 3.3.0 lib.rs:37; executor.rs:1469/1944 keep the kind).
    // That is a lost compare-and-swap, not a failure: report `false` so the caller reloads. Any other error stays fatal.
    let mut response = match response.check() {
        Ok(response) => response,
        Err(error) if matches!(error.query_details(), Some(surrealdb::types::QueryError::TransactionConflict)) => {
            tracing::debug!(kind = "TransactionConflict", "AG-UI run transition lost a write race; reporting compare-and-swap miss");
            return Ok(false);
        }
        Err(error) => return Err(error.into()),
    };
    let rows: Vec<String> = response.take(0)?;
    Ok(rows.len() == 1)
}
