use super::SurrealDbProvider;
use crate::uar::persistence::mcp_elicitations::{McpElicitationRecord, ElicitationState};

pub(super) async fn create(store: &SurrealDbProvider, row: &McpElicitationRecord) -> anyhow::Result<()> {
    anyhow::ensure!(store.durable_instances, "durable_elicitation_unsupported");
    row.validate()?;
    anyhow::ensure!(row.state == ElicitationState::Pending, "elicitation_initial_state");
    store.db.query("CREATE type::record('mcp_elicitations',$id) CONTENT $row")
        .bind(("id",row.id.clone())).bind(("row",serde_json::json!({"owner_key":row.owner_key,"run_id":row.run_id,"data":serde_json::to_string(row)?}))).await?.check()?;
    Ok(())
}
pub(super) async fn list(store: &SurrealDbProvider, owner: &str, run: &str) -> anyhow::Result<Vec<McpElicitationRecord>> {
    anyhow::ensure!(store.durable_instances, "durable_elicitation_unsupported");
    let mut response = store.db.query("SELECT VALUE data FROM mcp_elicitations WHERE owner_key=$owner AND run_id=$run")
        .bind(("owner",owner.to_owned())).bind(("run",run.to_owned())).await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    rows.iter().map(|row| { let row: McpElicitationRecord = serde_json::from_str(row)?; row.validate()?; Ok(row) }).collect()
}
pub(super) async fn transition(store: &SurrealDbProvider, before: &McpElicitationRecord, after: &McpElicitationRecord) -> anyhow::Result<bool> {
    anyhow::ensure!(store.durable_instances, "durable_elicitation_unsupported");
    before.validate_transition(after)?;
    let desired = serde_json::to_string(after)?;
    let mut response = store.db.query("UPDATE type::record('mcp_elicitations',$id) SET data=IF $answer AND <datetime>$deadline <= time::now() { $expired } ELSE { $after } WHERE data=$before AND owner_key=$owner AND run_id=$run RETURN VALUE data")
        .bind(("id",before.id.clone())).bind(("owner",before.owner_key.clone())).bind(("run",before.run_id.clone()))
        .bind(("answer",after.state == ElicitationState::Answered)).bind(("deadline",before.expires_at.to_rfc3339()))
        .bind(("expired",serde_json::to_string(&before.finish(ElicitationState::Expired))?))
        .bind(("before",serde_json::to_string(before)?)).bind(("after",desired.clone())).await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    Ok(rows.as_slice() == [desired])
}
