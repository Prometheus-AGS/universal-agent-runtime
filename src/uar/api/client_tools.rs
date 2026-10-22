//! Shared caller-tool ingress. Authentication/readmission belongs to each adapter.
use std::collections::{BTreeMap, BTreeSet};
use serde_json::Value;
use crate::uar::persistence::{PersistenceLayer, client_tools::{ClientToolContinuation, ClientToolContinuationState}};
use crate::uar::runtime::turn::client_tools::ClientToolInput;

#[derive(Debug, Clone)]
pub enum ClientToolPreparation {
    Fresh(ClientToolInput),
    Continue { input: ClientToolInput, claim: ClientToolContinuation },
    Replay { response: Value, events: Vec<Value> },
}

/// Decode the host-issued locator; caller-selected ordinary tool IDs are never resume grants.
pub fn batch_id_from_call_id(call_id: &str) -> anyhow::Result<String> {
    let locator = call_id.strip_prefix("uar_client_")
        .ok_or_else(|| anyhow::anyhow!("invalid_client_tool_call_id"))?;
    let (batch, call) = locator.split_once('_')
        .ok_or_else(|| anyhow::anyhow!("invalid_client_tool_call_id"))?;
    let batch_uuid = uuid::Uuid::parse_str(batch)?;
    let call_uuid = uuid::Uuid::parse_str(call)?;
    anyhow::ensure!(batch_uuid.to_string() == batch && call_uuid.to_string() == call, "invalid_client_tool_call_id");
    Ok(batch.to_owned())
}

/// Exact authenticated scope lookup. A locator locates evidence and grants no authority.
pub async fn load_by_call_id(
    store: &dyn PersistenceLayer, owner: &str, tenant: Option<&str>, agent: &str, call_id: &str,
) -> anyhow::Result<Option<ClientToolContinuation>> {
    let batch = batch_id_from_call_id(call_id)?;
    anyhow::ensure!(store.supports_durable_client_tools(), "durable_client_tools_unsupported");
    let row = store.load_client_tool_continuation(owner, tenant, agent, &batch).await?;
    if let Some(row) = &row {
        anyhow::ensure!(row.pending_call_ids.iter().any(|id| id == call_id), "client_tool_call_not_pending");
    }
    Ok(row)
}

/// Persist actual producer cleanup before a caller handoff becomes public.
/// This acknowledgment survives restart; a historical terminal event cannot create it.
pub async fn confirm_handoff_for_run(
    manager: &crate::uar::runtime::manager::RunManager,
    store: &dyn PersistenceLayer, owner: &str, tenant: Option<&str>, agent: &str, run_id: &str,
) -> anyhow::Result<()> {
    let row = store.load_client_tool_continuation_for_run(owner, tenant, agent, run_id).await?
        .ok_or_else(|| anyhow::anyhow!("client_tool_continuation_not_found"))?;
    row.validate()?;
    if row.source_settled { return Ok(()); }
    anyhow::ensure!(row.state == ClientToolContinuationState::Pending, "client_tool_continuation_state_conflict");
    anyhow::ensure!(manager.await_settled_run_status(run_id).await?
        == crate::uar::domain::runs::RunStatus::Done, "client_tool_source_not_settled");
    if let Some(source) = &row.execution_source {
        anyhow::ensure!(manager.await_settled_run_status(&source.run_id).await?
            == crate::uar::domain::runs::RunStatus::Done, "client_tool_child_source_not_settled");
    }
    let mut settled = row.clone();
    settled.source_settled = true;
    if store.compare_and_swap_client_tool_continuation(&row, &settled).await? { return Ok(()); }
    let saved = store.load_client_tool_continuation_for_run(owner, tenant, agent, run_id).await?
        .ok_or_else(|| anyhow::anyhow!("client_tool_continuation_not_found"))?;
    saved.validate()?;
    anyhow::ensure!(saved.source_settled, "client_tool_source_settlement_conflict");
    Ok(())
}

/// Capture declarations and provider controls together as an immutable continuation binding.
pub fn schema_binding(input: &ClientToolInput) -> anyhow::Result<Value> {
    Ok(serde_json::json!({
        "tools": input.tools,
        "tool_choice": serde_json::to_value(&input.tool_choice)?,
        "parallel_tool_calls": input.parallel_tool_calls,
    }))
}

/// Validate the full visible predecessor/result batch and claim it before any host effects.
/// A Claimed row is never automatically reacquired after interruption or restart.
pub async fn prepare(
    manager: &crate::uar::runtime::manager::RunManager,
    store: &dyn PersistenceLayer, owner: &str, tenant: Option<&str>, agent: &str,
    input: &ClientToolInput,
) -> anyhow::Result<ClientToolPreparation> {
    input.validate()?;
    anyhow::ensure!(input.output_token_cap.is_none_or(|cap| cap > 0), "invalid_client_tool_output_token_cap");
    validate_visible_history(store, owner, tenant, agent, input).await?;
    let suffix_start = input.history.iter().rposition(|message| message.get("role").and_then(Value::as_str) != Some("tool"))
        .map_or(0, |position| position + 1);
    let suffix = &input.history[suffix_start..];
    if suffix.is_empty() {
        return Ok(ClientToolPreparation::Fresh(input.clone()));
    }
    let mut results = BTreeMap::new();
    let mut batch_id = None;
    for message in suffix {
        let id = message.get("tool_call_id").and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("client_tool_result_id_required"))?;
        let batch = batch_id_from_call_id(id)?;
        if let Some(expected) = &batch_id { anyhow::ensure!(expected == &batch, "client_tool_result_batch_mismatch"); }
        batch_id = Some(batch);
        let content = message.get("content").cloned()
            .ok_or_else(|| anyhow::anyhow!("client_tool_result_content_required"))?;
        anyhow::ensure!(results.insert(id.to_owned(), content).is_none(), "duplicate_client_tool_result");
    }
    let batch_id = batch_id.ok_or_else(|| anyhow::anyhow!("client_tool_result_batch_required"))?;
    anyhow::ensure!(store.supports_durable_client_tools(), "durable_client_tools_unsupported");
    let row = store.load_client_tool_continuation(owner, tenant, agent, &batch_id).await?
        .ok_or_else(|| anyhow::anyhow!("client_tool_continuation_not_found"))?;
    row.validate()?;
    // The fully authenticated durable row supplies the source identity. A caller
    // cannot resume while its predecessor still owns cleanup or failed cleanup.
    if row.state == ClientToolContinuationState::Pending {
        anyhow::ensure!(row.source_settled, "client_tool_source_not_settled");
        if manager.get_run(&row.source_run_id).await.is_some() {
            anyhow::ensure!(
                manager.settled_run_status(&row.source_run_id).await
                    == Some(crate::uar::domain::runs::RunStatus::Done),
                "client_tool_source_not_settled"
            );
        }
    }
    anyhow::ensure!(row.protocol == input.protocol, "client_tool_protocol_conflict");
    anyhow::ensure!(visible_transcript_matches(&row.visible_transcript, &input.history[..suffix_start], &input.protocol)?, "client_tool_visible_history_conflict");
    anyhow::ensure!(row.client_schemas == schema_binding(input)?, "client_tool_schema_conflict");
    let mut claimed = row.clone();
    claimed.state = ClientToolContinuationState::Claimed;
    claimed.result_batch = Some(results.clone());
    claimed.completed_response = None;
    claimed.completed_events.clear();
    // This validates the exact full set before a duplicate/completion decision as well.
    claimed.validate()?;
    match row.state {
        ClientToolContinuationState::Completed => return replay(&row, &results),
        ClientToolContinuationState::Claimed => anyhow::bail!("client_tool_continuation_already_claimed"),
        ClientToolContinuationState::Pending => {}
    }
    let canonical_history = claimed.canonical_transcript.as_array().cloned()
        .ok_or_else(|| anyhow::anyhow!("client_tool_canonical_history_invalid"))?;
    if !store.compare_and_swap_client_tool_continuation(&row, &claimed).await? {
        let current = store.load_client_tool_continuation(owner, tenant, agent, &batch_id).await?
            .ok_or_else(|| anyhow::anyhow!("client_tool_continuation_not_found"))?;
        return replay(&current, &results);
    }
    let mut resumed = input.clone();
    resumed.visible_history = Some(input.history.clone());
    resumed.graph_state = claimed.graph_state.clone();
    resumed.budget_snapshot = claimed.budget_snapshot.clone();
    resumed.output_token_cap = match (input.output_token_cap, claimed.output_token_cap) {
        (Some(requested), Some(previous)) => Some(requested.min(previous)),
        (requested, previous) => requested.or(previous),
    };
    resumed.history = canonical_history;
    resumed.history.extend_from_slice(suffix);
    Ok(ClientToolPreparation::Continue { input: resumed, claim: claimed })
}

/// Anthropic represents function arguments as JSON objects and omits empty
/// assistant text. Compare those lossless wire forms without relaxing IDs,
/// function names, nonempty text or any caller result content.
fn visible_transcript_matches(saved: &Value, supplied: &[Value], protocol: &str) -> anyhow::Result<bool> {
    if protocol != "anthropic" { return Ok(saved == &Value::Array(supplied.to_vec())); }
    fn normalized(messages: &[Value]) -> anyhow::Result<Vec<Value>> {
        messages.iter().map(|message| {
            let mut message = message.clone();
            if message.get("role").and_then(Value::as_str) == Some("assistant") {
                if let Some(object) = message.as_object_mut() {
                    if object.get("content").is_none_or(|content| content.is_null() || content.as_str() == Some("")) {
                        object.remove("content");
                    }
                    if object.get("tool_calls").is_some_and(|calls| calls.is_null() || calls.as_array().is_some_and(Vec::is_empty)) {
                        object.remove("tool_calls");
                    }
                    if let Some(calls) = object.get_mut("tool_calls").and_then(Value::as_array_mut) {
                        for call in calls {
                            if let Some(arguments) = call.pointer_mut("/function/arguments") {
                                if let Some(text) = arguments.as_str() {
                                    *arguments = serde_json::from_str(text)?;
                                }
                            }
                        }
                    }
                }
            }
            Ok(message)
        }).collect()
    }
    let saved = saved.as_array().ok_or_else(|| anyhow::anyhow!("client_tool_visible_history_invalid"))?;
    Ok(normalized(saved)? == normalized(supplied)?)
}

/// Every caller-visible tool exchange needs durable evidence. In particular,
/// appending a user message must not turn forged host results into fresh history.
pub(crate) async fn validate_visible_history(
    store: &dyn PersistenceLayer, owner: &str, tenant: Option<&str>, agent: &str,
    input: &ClientToolInput,
) -> anyhow::Result<()> {
    let history = input.visible_history.as_ref().unwrap_or(&input.history);
    let mut position = 0;
    while position < history.len() {
        let message = &history[position];
        anyhow::ensure!(message.get("role").and_then(Value::as_str) != Some("tool"), "orphan_client_tool_result");
        let Some(calls) = message.get("tool_calls") else { position += 1; continue; };
        anyhow::ensure!(message.get("role").and_then(Value::as_str) == Some("assistant"), "invalid_client_tool_call_role");
        let calls = calls.as_array().filter(|calls| !calls.is_empty())
            .ok_or_else(|| anyhow::anyhow!("invalid_client_tool_calls"))?;
        let mut ids = BTreeSet::new();
        for call in calls {
            let id = call.get("id").and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("client_tool_call_id_required"))?;
            batch_id_from_call_id(id)?;
            anyhow::ensure!(ids.insert(id.to_owned()), "duplicate_client_tool_call");
        }
        let first_id = ids.first().ok_or_else(|| anyhow::anyhow!("client_tool_call_id_required"))?;
        let row = load_by_call_id(store, owner, tenant, agent, first_id).await?
            .ok_or_else(|| anyhow::anyhow!("client_tool_continuation_not_found"))?;
        anyhow::ensure!(row.protocol == input.protocol, "client_tool_protocol_conflict");
        anyhow::ensure!(row.pending_call_ids.iter().cloned().collect::<BTreeSet<_>>() == ids, "client_tool_call_batch_mismatch");
        anyhow::ensure!(visible_transcript_matches(&row.visible_transcript, &history[..=position], &input.protocol)?, "client_tool_visible_history_conflict");
        let mut end = position + 1;
        let mut results = BTreeMap::new();
        while let Some(result) = history.get(end).filter(|message| message.get("role").and_then(Value::as_str) == Some("tool")) {
            let id = result.get("tool_call_id").and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("client_tool_result_id_required"))?;
            let content = result.get("content").cloned()
                .ok_or_else(|| anyhow::anyhow!("client_tool_result_content_required"))?;
            anyhow::ensure!(results.insert(id.to_owned(), content).is_none(), "duplicate_client_tool_result");
            end += 1;
        }
        anyhow::ensure!(results.keys().cloned().collect::<BTreeSet<_>>() == ids, "client_tool_result_batch_mismatch");
        if end != history.len() {
            anyhow::ensure!(row.state == ClientToolContinuationState::Completed && row.result_batch.as_ref() == Some(&results), "client_tool_history_not_completed");
        }
        position = end;
    }
    Ok(())
}

fn replay(row: &ClientToolContinuation, results: &BTreeMap<String, Value>) -> anyhow::Result<ClientToolPreparation> {
    row.validate()?;
    anyhow::ensure!(row.state == ClientToolContinuationState::Completed, "client_tool_continuation_already_claimed");
    anyhow::ensure!(row.result_batch.as_ref() == Some(results), "client_tool_result_batch_conflict");
    let response = row.completed_response.clone()
        .ok_or_else(|| anyhow::anyhow!("client_tool_completed_response_missing"))?;
    Ok(ClientToolPreparation::Replay { response, events: row.completed_events.clone() })
}

/// Commit exact adapter response/event projection before public delivery.
pub async fn complete(
    store: &dyn PersistenceLayer, claim: &ClientToolContinuation, response: Value, events: Vec<Value>,
) -> anyhow::Result<()> {
    let mut completed = claim.clone();
    completed.state = ClientToolContinuationState::Completed;
    completed.completed_response = Some(response);
    completed.completed_events = events;
    if store.compare_and_swap_client_tool_continuation(claim, &completed).await? { return Ok(()); }
    let saved = store.load_client_tool_continuation(&claim.owner_key, claim.tenant_id.as_deref(), &claim.agent_id, &claim.batch_id).await?;
    anyhow::ensure!(saved.as_ref() == Some(&completed), "client_tool_completion_conflict");
    Ok(())
}
