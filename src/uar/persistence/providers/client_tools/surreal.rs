use super::SurrealDbProvider;
use crate::uar::persistence::client_tools::ClientToolContinuation;

pub(super) async fn create(store: &SurrealDbProvider, row: &ClientToolContinuation) -> anyhow::Result<bool> {
    anyhow::ensure!(store.durable_instances, "durable_client_tools_unsupported");
    row.validate_create()?;
    let response = store.db.query("BEGIN TRANSACTION;
        IF (SELECT * FROM type::record('client_tool_continuations',$key))[0] != NONE { THROW 'client_tool_continuation_exists'; };
        CREATE type::record('client_tool_continuations',$key) CONTENT $payload;
        COMMIT TRANSACTION;")
        .bind(("key",row.storage_key()))
        .bind(("payload",serde_json::json!({"owner_key":row.owner_key,"agent_id":row.agent_id,"source_run_id":row.source_run_id,"data":serde_json::to_string(row)?})))
        .await?;
    match response.check() {
        Ok(_) => Ok(true),
        Err(error) if error.to_string().contains("client_tool_continuation_exists") || matches!(error.query_details(), Some(surrealdb::types::QueryError::TransactionConflict)) => Ok(false),
        Err(error) => Err(error.into()),
    }
}
pub(super) async fn load(store: &SurrealDbProvider, owner: &str, tenant: Option<&str>, agent: &str, batch: &str) -> anyhow::Result<Option<ClientToolContinuation>> {
    anyhow::ensure!(store.durable_instances, "durable_client_tools_unsupported");
    let mut response = store.db.query("SELECT VALUE data FROM type::record('client_tool_continuations',$key) WHERE owner_key=$owner AND agent_id=$agent")
        .bind(("key",ClientToolContinuation::key(owner,tenant,agent,batch))).bind(("owner",owner.to_owned())).bind(("agent",agent.to_owned())).await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    rows.first().map(|row| {
        let row: ClientToolContinuation = serde_json::from_str(row)?;
        row.validate()?;
        anyhow::ensure!(row.owner_key == owner && row.tenant_id.as_deref() == tenant && row.agent_id == agent && row.batch_id == batch, "client_tool_continuation_binding_conflict");
        Ok(row)
    }).transpose()
}
pub(super) async fn transition(store: &SurrealDbProvider, before: &ClientToolContinuation, after: &ClientToolContinuation) -> anyhow::Result<bool> {
    anyhow::ensure!(store.durable_instances, "durable_client_tools_unsupported");
    before.validate_transition(after)?;
    let response = store.db.query("UPDATE type::record('client_tool_continuations',$key) SET data=$after WHERE data=$before RETURN VALUE data")
        .bind(("key",before.storage_key())).bind(("before",serde_json::to_string(before)?)).bind(("after",serde_json::to_string(after)?)).await?;
    let mut response = match response.check() {
        Ok(response) => response,
        Err(error) if matches!(error.query_details(), Some(surrealdb::types::QueryError::TransactionConflict)) => return Ok(false),
        Err(error) => return Err(error.into()),
    };
    let rows: Vec<String> = response.take(0)?;
    Ok(rows.len() == 1)
}

pub(super) async fn load_for_run(store: &SurrealDbProvider, owner: &str, tenant: Option<&str>, agent: &str, run: &str) -> anyhow::Result<Option<ClientToolContinuation>> {
    anyhow::ensure!(store.durable_instances, "durable_client_tools_unsupported");
    let mut response = store.db.query("SELECT VALUE data FROM client_tool_continuations WHERE owner_key=$owner AND agent_id=$agent AND source_run_id=$run")
        .bind(("owner",owner.to_owned())).bind(("agent",agent.to_owned())).bind(("run",run.to_owned())).await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    let mut matched = None;
    for row in rows {
        let row: ClientToolContinuation = serde_json::from_str(&row)?;
        row.validate()?;
        if row.tenant_id.as_deref() == tenant {
            anyhow::ensure!(row.owner_key == owner && row.agent_id == agent && row.source_run_id == run && matched.is_none(), "client_tool_run_binding_conflict");
            matched = Some(row);
        }
    }
    Ok(matched)
}

pub(super) async fn load_for_session(store: &SurrealDbProvider, owner: &str, tenant: Option<&str>, session: &str, batch: &str) -> anyhow::Result<Option<ClientToolContinuation>> {
    anyhow::ensure!(store.durable_instances, "durable_client_tools_unsupported");
    // Existing durable rows store the full binding as serialized data. Filter the
    // authenticated owner's rows without changing their immutable representation.
    let mut response = store.db.query("SELECT VALUE data FROM client_tool_continuations WHERE owner_key=$owner")
        .bind(("owner",owner.to_owned())).await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    let mut matched = None;
    for row in rows {
        let row: ClientToolContinuation = serde_json::from_str(&row)?;
        if row.tenant_id.as_deref() == tenant && row.session_id.as_deref() == Some(session) && row.batch_id == batch {
            row.validate()?;
            anyhow::ensure!(row.owner_key == owner && matched.is_none(), "client_tool_session_binding_conflict");
            matched = Some(row);
        }
    }
    Ok(matched)
}
