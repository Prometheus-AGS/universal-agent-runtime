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
    let before_data = serde_json::to_string(before)?;
    let response = store.db.query("UPDATE type::record('agui_runs',$key) SET data=$after WHERE data=$before RETURN VALUE data")
        .bind(("key",before.storage_key())).bind(("before",before_data.clone())).bind(("after",serde_json::to_string(after)?)).await?;
    let mut response = match response.check() {
        Ok(response) => response,
        Err(error) if matches!(error.query_details(), Some(surrealdb::types::QueryError::TransactionConflict)) => {
            // A transaction conflict can concern another row touched by a schema event.
            // Only an observed mismatch of this UPDATE's exact predicate is a CAS miss.
            // This later read does not identify the cause of the original conflict.
            let current = async {
                let mut response = store.db.query("SELECT VALUE data FROM type::record('agui_runs',$key)")
                    .bind(("key",before.storage_key())).await?.check()?;
                let rows: Vec<String> = response.take(0)?;
                Ok::<_, anyhow::Error>(rows)
            }.await;
            if matches!(current, Ok(ref rows) if rows.is_empty() || (rows.len() == 1 && rows[0] != before_data)) {
                tracing::debug!(kind = "TransactionConflict", "AG-UI run predicate no longer matches; reporting compare-and-swap miss");
                return Ok(false);
            }
            // An unchanged target or an inconclusive read retains the original typed error.
            return Err(error.into());
        }
        Err(error) => return Err(error.into()),
    };
    let rows: Vec<String> = response.take(0)?;
    Ok(rows.len() == 1)
}
