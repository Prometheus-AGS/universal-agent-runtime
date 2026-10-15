use super::SurrealDbProvider;
use crate::uar::api::a2a::durable::A2aBinding;

pub(super) async fn create(store: &SurrealDbProvider, record: &A2aBinding) -> anyhow::Result<bool> {
    anyhow::ensure!(store.durable_instances, "Durable A2A correlation unavailable");
    record.validate()?;
    anyhow::ensure!(record.revision == 1, "Initial A2A revision must be one");
    let response = store.db.query("BEGIN TRANSACTION; LET $old = (SELECT * FROM type::record('a2a_bindings', $key))[0]; IF $old != NONE { THROW 'uar_a2a_binding_exists'; }; LET $slot = (SELECT * FROM type::record('a2a_contexts', $context))[0]; IF $slot != NONE { LET $prior = (SELECT * FROM type::record('a2a_bindings', $slot.binding_key))[0]; IF $prior == NONE OR $prior.context_closed != true { THROW 'uar_a2a_context_active'; }; }; CREATE type::record('a2a_bindings', $key) CONTENT $payload; UPSERT type::record('a2a_contexts', $context) SET binding_key = $key; COMMIT TRANSACTION;")
        .bind(("key", record.storage_key()))
        .bind(("context", A2aBinding::storage_key_for(&record.owner_key, &record.agent_id, &record.context_id)))
        .bind(("payload", serde_json::json!({"owner_key": record.owner_key, "agent_id": record.agent_id,
            "context_closed": record.context_closed(), "data": serde_json::to_string(record)?}))).await?;
    match response.check() {
        Ok(_) => Ok(true),
        Err(error) if error.to_string().contains("uar_a2a_binding_exists") || error.to_string().contains("uar_a2a_context_active") => Ok(false),
        Err(error) => Err(error.into()),
    }
}

async fn load_raw(store: &SurrealDbProvider, owner: &str, agent: &str, task: &str) -> anyhow::Result<Option<(String, A2aBinding)>> {
    anyhow::ensure!(store.durable_instances, "Durable A2A correlation unavailable");
    let mut response = store.db.query("SELECT VALUE data FROM type::record('a2a_bindings', $key) WHERE owner_key = $owner AND agent_id = $agent")
        .bind(("key", A2aBinding::storage_key_for(owner, agent, task)))
        .bind(("owner", owner.to_owned())).bind(("agent", agent.to_owned())).await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    rows.into_iter().next().map(|row| {
        let record: A2aBinding = serde_json::from_str(&row)?;
        record.validate()?;
        anyhow::ensure!(record.owner_key == owner && record.agent_id == agent && record.task_id == task, "A2A persisted scope mismatch");
        Ok((row, record))
    }).transpose()
}

pub(super) async fn load(store: &SurrealDbProvider, owner: &str, agent: &str, task: &str) -> anyhow::Result<Option<A2aBinding>> {
    Ok(load_raw(store, owner, agent, task).await?.map(|(_, record)| record))
}

pub(super) async fn transition(store: &SurrealDbProvider, before: &A2aBinding, after: &A2aBinding) -> anyhow::Result<bool> {
    anyhow::ensure!(store.durable_instances, "Durable A2A correlation unavailable");
    before.validate_transition(after)?;
    let Some((raw, current)) = load_raw(store, &before.owner_key, &before.agent_id, &before.task_id).await? else {
        return Ok(false);
    };
    if serde_json::to_value(&current)? != serde_json::to_value(before)? {
        return Ok(false);
    }
    let mut response = store.db.query("UPDATE type::record('a2a_bindings', $key) SET data = $after, context_closed = $closed WHERE data = $before RETURN VALUE data")
        .bind(("key", before.storage_key())).bind(("before", raw))
        .bind(("after", serde_json::to_string(after)?)).bind(("closed", after.context_closed())).await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    Ok(rows.len() == 1)
}

pub(super) async fn list(store: &SurrealDbProvider, owner: &str, agent: &str) -> anyhow::Result<Vec<A2aBinding>> {
    anyhow::ensure!(store.durable_instances, "Durable A2A correlation unavailable");
    let mut response = store.db.query("SELECT VALUE data FROM a2a_bindings WHERE owner_key = $owner AND agent_id = $agent")
        .bind(("owner", owner.to_owned())).bind(("agent", agent.to_owned())).await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    rows.into_iter().map(|row| {
        let record: A2aBinding = serde_json::from_str(&row)?;
        record.validate()?;
        anyhow::ensure!(record.owner_key == owner && record.agent_id == agent, "A2A persisted scope mismatch");
        Ok(record)
    }).collect()
}

/// Complete a cancellation only after the trusted host has excluded the exact
/// producer and its native subtree has no unresolved effects or liabilities.
/// This never resumes a checkpoint, settles money, or rewrites a terminal result.
pub(super) async fn reconcile_cancellation(store: &SurrealDbProvider, owner: &str, user: &str,
    agent: &str, task: &str) -> anyhow::Result<Option<A2aBinding>> {
    use crate::uar::{persistence::{PersistenceLayer, checkpoint_resume::CheckpointResumeError},
        runtime::thread::{AgentThreadResult, AgentThreadStatus}};
    let Some((before_raw, before)) = load_raw(store,owner,agent,task).await? else { return Ok(None); };
    if !before.cancellation_requested || before.context_closed() { return Ok(None); }
    let Some(epoch) = before.runtime_epoch.as_deref().filter(|epoch| !epoch.is_empty()) else { return Ok(None); };
    let mut response = store.db.query("SELECT VALUE state FROM type::record('checkpoint_runtime_epochs',$epoch)")
        .bind(("epoch",epoch.to_owned())).await?.check()?;
    let states: Vec<String> = response.take(0)?;
    if states.len() != 1 || !matches!(states[0].as_str(),"orderly_excluded"|"supervisor_excluded") { return Ok(None); }
    match super::checkpoint_records::require_resolved_effects(store,epoch,&before.run_id).await {
        Ok(()) => {},
        Err(error) if matches!(error.downcast_ref::<CheckpointResumeError>(),Some(CheckpointResumeError::PredecessorUncertain)) => return Ok(None),
        Err(error) => return Err(error),
    }
    let threads = store.list_agent_threads(user,&before.run_id).await?;
    let mut changes = Vec::new();
    for current in threads {
        let thread = &current.thread;
        // Excluding the local epoch says nothing about a delegated peer. Even a
        // local failed projection cannot establish that peer's cleanup outcome.
        if thread.remote.is_some() && !matches!(&thread.result,
            Some(AgentThreadResult::Completed {..}) | Some(AgentThreadResult::Cancelled)) { return Ok(None); }
        if let Some(AgentThreadResult::Failed {code,..}) = &thread.result {
            if code.contains("unconfirmed") || code.contains("unknown") { return Ok(None); }
        }
        if thread.status.is_terminal() { continue; }
        let mut next = thread.clone();
        next.status = AgentThreadStatus::Cancelled;
        next.result = Some(AgentThreadResult::Cancelled);
        next.updated_at = chrono::Utc::now();
        let next = crate::uar::persistence::agent_threads::next_record(user,&current,current.revision,&next)?;
        changes.push(serde_json::json!({
            "key":crate::uar::persistence::tenant_storage_key(user,&thread.thread_id),
            "before":serde_json::to_string(&current)?,"payload":super::agent_thread_payload(&next)?
        }));
    }
    let mut after = before.clone();
    after.revision = after.revision.checked_add(1).ok_or_else(||anyhow::anyhow!("A2A revision exhausted"))?;
    after.task.status.state = crate::uar::api::a2a::types::TaskState::Canceled;
    after.task.status.timestamp = Some(chrono::Utc::now().to_rfc3339());
    after.task.status.message = None;
    for key in ["execution_unconfirmed","cleanup_unconfirmed","context_unconfirmed"] { after.task.metadata.remove(key); }
    after.task.metadata.insert("uar.cancellation_excluded_epoch".into(),serde_json::json!(epoch));
    after.task.metadata.insert(crate::uar::api::a2a::contract::UAR_CLEANUP_CLOSED_METADATA.into(),serde_json::json!(true));
    before.validate_transition(&after)?;
    let response = store.db.query("BEGIN TRANSACTION;
        LET $epoch = (SELECT * FROM type::record('checkpoint_runtime_epochs',$epoch_id))[0];
        IF $epoch = NONE OR $epoch.state NOT IN ['orderly_excluded','supervisor_excluded'] { THROW 'a2a_cancellation_unconfirmed'; };
        LET $binding = (SELECT * FROM type::record('a2a_bindings',$key))[0];
        IF $binding = NONE OR $binding.data != $before { THROW 'a2a_cancellation_unconfirmed'; };
        IF (SELECT * FROM type::record('checkpoint_resume_claims',$claim_key))[0] != NONE { THROW 'a2a_cancellation_unconfirmed'; };
        FOR $change IN $threads {
            LET $old = (SELECT * FROM type::record('agent_threads',$change.key))[0];
            IF $old = NONE OR $old.data != $change.before { THROW 'a2a_cancellation_unconfirmed'; };
            UPDATE type::record('agent_threads',$change.key) CONTENT $change.payload;
        };
        UPDATE checkpoint_resume_envelopes SET cancelled=true WHERE owner_key=$owner AND source_run_id=$run;
        UPDATE type::record('a2a_bindings',$key) SET data=$after,context_closed=true;
        COMMIT TRANSACTION;")
        .bind(("epoch_id",epoch.to_owned())).bind(("key",before.storage_key()))
        .bind(("before",before_raw)).bind(("after",serde_json::to_string(&after)?))
        .bind(("claim_key",crate::uar::persistence::tenant_storage_key(owner,&before.run_id)))
        .bind(("owner",owner.to_owned())).bind(("run",before.run_id.clone()))
        .bind(("threads",changes)).await?;
    // A transaction may wrap earlier statements as NotExecuted; inspect all
    // errors so a racing continuation remains a held cancellation, not success.
    let mut response = response;
    let errors = response.take_errors();
    if errors.values().any(|error|error.to_string().contains("a2a_cancellation_unconfirmed")) { return Ok(None); }
    if let Some((_,error)) = errors.into_iter().min_by_key(|(index,_)|*index) { return Err(error.into()); }
    Ok(Some(after))
}
