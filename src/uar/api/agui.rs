//! AG-UI 1.0 transport over the existing run host and durable approval broker.
//! Wire-run records correlate protocol segments; they never authorize execution.
use std::{collections::HashSet, convert::Infallible, sync::Arc};
use axum::{Extension, Json, Router, extract::{Path, State}, http::{HeaderMap, StatusCode}, response::{IntoResponse, Response, Sse, sse::{Event, KeepAlive}}, routing::{get, post}};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use crate::{config::SecurityConfig, uar::{domain::events::NormalizedEvent, persistence::{agui_runs::{AguiRunBinding, AguiTerminal}, approval_decisions::{ApprovalRecordView, ApprovalState}}, security::{authority::authorize_context, claims::{CredentialPrincipal, UserContext}, credential_capture::AuthenticatedCredentialCapture, sidecar_guard::HostAuthenticated}}};
use anyhow::Context as _;
use super::routes::{self, RunApiState};

#[derive(Clone)]
struct AdmissionPolicy(SecurityConfig);

/// Mount at `/api/uar/ag-ui` inside the normal authenticated API router.
pub fn build_router(security: SecurityConfig) -> Router<Arc<RunApiState>> {
    Router::new().route("/agents/{agent_id}", post(run))
        .route("/agents/{agent_id}/threads/{thread_id}/runs/{run_id}/stream", get(reconnect))
        .layer(Extension(AdmissionPolicy(security)))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunAgentInput {
    thread_id: String,
    run_id: String,
    protocol_version: Option<String>,
    parent_run_id: Option<String>,
    messages: Vec<Value>,
    #[serde(default)] tools: Vec<Value>,
    #[serde(default)] context: Vec<Value>,
    state: Option<Value>,
    forwarded_props: Option<Value>,
    #[serde(default)] resume: Vec<ResumeEntry>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResumeEntry { interrupt_id: String, status: String, payload: Option<Value> }

/// UAR-specific controls use standard forwardedProps, not new AG-UI fields.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CallerToolControls {
    tool_choice: Option<crate::llm::ToolChoice>,
    parallel_tool_calls: Option<bool>,
}
fn caller_controls(input: &RunAgentInput) -> Result<Option<CallerToolControls>,Response> {
    input.forwarded_props.as_ref().and_then(|props|props.pointer("/uar/client_tools"))
        .map(|value|serde_json::from_value(value.clone()).map_err(|_|failure(StatusCode::BAD_REQUEST,"invalid_frontend_tool_controls")))
        .transpose()
}

/// AG-UI tools are caller-owned functions, never host executable registrations.
/// Match the pinned ToolSchema before converting to the shared function shape.
fn caller_tool_definitions(tools: &[Value]) -> Result<Vec<Value>, Response> {
    tools.iter().map(|tool| {
        let Some(name) = tool.get("name").and_then(Value::as_str) else {
            return Err(failure(StatusCode::BAD_REQUEST,"invalid_frontend_tool"));
        };
        let Some(description) = tool.get("description").and_then(Value::as_str) else {
            return Err(failure(StatusCode::BAD_REQUEST,"invalid_frontend_tool"));
        };
        if tool.get("parameters").is_some_and(Value::is_null)
            || tool.get("metadata").is_some_and(|metadata| !metadata.is_object()) {
            return Err(failure(StatusCode::BAD_REQUEST,"invalid_frontend_tool"));
        }
        let parameters = tool.get("parameters").cloned().unwrap_or_else(||json!({"type":"object","properties":{}}));
        Ok(json!({"type":"function","function":{"name":name,"description":description,"parameters":parameters}}))
    }).collect()
}

fn caller_history(messages: &[Value]) -> Result<Vec<Value>, Response> {
    messages.iter().map(|message| {
        let role = message.get("role").and_then(Value::as_str).unwrap_or("");
        if !matches!(role,"user"|"assistant"|"tool")
            || (!message.get("content").is_some_and(Value::is_string)
                && !(role == "assistant" && message.get("toolCalls").is_some_and(Value::is_array))) {
            return Err(failure(StatusCode::BAD_REQUEST,"unsupported_history_message"));
        }
        let mut value = json!({"role":role,"content":message.get("content").cloned().unwrap_or_else(||json!(""))});
        if let Some(calls) = message.get("toolCalls") { value["tool_calls"] = calls.clone(); }
        if let Some(id) = message.get("toolCallId") { value["tool_call_id"] = id.clone(); }
        Ok(value)
    }).collect()
}

async fn prepare_frontend_continuation(
    state: &RunApiState, user: &UserContext, previous: &AguiRunBinding, input: &RunAgentInput,
) -> Result<super::client_tools::ClientToolPreparation,Response> {
    let Some(store) = &state.manager.persistence else { return Err(unavailable()); };
    let source = state.manager.get_run_for_context(user,&previous.native_run_id).await
        .ok_or_else(||failure(StatusCode::CONFLICT,"caller_tool_source_unavailable"))?;
    let principal = CredentialPrincipal::from_verified_context(user)
        .map_err(|_|failure(StatusCode::UNAUTHORIZED,"verified_principal_required"))?;
    let session_owner = user.session_owner_key().map_err(|_|failure(StatusCode::UNAUTHORIZED,"verified_principal_required"))?;
    let saved = store.load_client_tool_continuation_for_run(&session_owner,principal.tenant_id(),&source.agent_id,&previous.native_run_id).await
        .map_err(|_|unavailable())?.ok_or_else(||failure(StatusCode::CONFLICT,"caller_tool_batch_unavailable"))?;
    if saved.session_id.as_deref() != Some(previous.session_id.as_str()) || saved.protocol != "agui" {
        return Err(failure(StatusCode::CONFLICT,"caller_tool_context_conflict"));
    }
    let expected = previous.terminal.as_ref().and_then(|terminal|terminal.event.pointer("/outcome/pendingToolCallIds"))
        .and_then(Value::as_array).ok_or_else(||failure(StatusCode::CONFLICT,"caller_tool_batch_unavailable"))?;
    if expected != &saved.pending_call_ids.iter().map(|id|json!(id)).collect::<Vec<_>>() {
        return Err(failure(StatusCode::CONFLICT,"caller_tool_batch_conflict"));
    }
    let mut history = caller_history(&input.messages)?;
    if history.iter().all(|message|message["role"] == "tool") {
        let mut prefix = saved.visible_transcript.as_array().cloned()
            .ok_or_else(||failure(StatusCode::CONFLICT,"caller_tool_history_unavailable"))?;
        prefix.append(&mut history);
        history = prefix;
    }
    let tools = if input.tools.is_empty() {
        saved.client_schemas.get("tools").and_then(Value::as_array).cloned()
            .ok_or_else(||failure(StatusCode::CONFLICT,"caller_tool_schema_unavailable"))?
    } else { caller_tool_definitions(&input.tools)? };
    let caller = crate::uar::runtime::turn::client_tools::ClientToolInput {
        tools,
        tool_choice: match caller_controls(input)? {
            Some(controls) => controls.tool_choice,
            None => serde_json::from_value(saved.client_schemas.get("tool_choice").cloned().unwrap_or(Value::Null))
                .map_err(|_|failure(StatusCode::CONFLICT,"caller_tool_schema_unavailable"))?,
        },
        parallel_tool_calls: match caller_controls(input)? {
            Some(controls) => controls.parallel_tool_calls,
            None => serde_json::from_value(saved.client_schemas.get("parallel_tool_calls").cloned().unwrap_or(Value::Null))
                .map_err(|_|failure(StatusCode::CONFLICT,"caller_tool_schema_unavailable"))?,
        },
        history,visible_history:None,graph_state:None,budget_snapshot:None,delegation:None,output_token_cap:None,protocol:"agui".into(),
    };
    super::client_tools::prepare(state.manager.as_ref(),store.as_ref(),&session_owner,principal.tenant_id(),&source.agent_id,&caller).await
        .map_err(|_|failure(StatusCode::CONFLICT,"caller_tool_continuation_conflict"))
}

fn failure(status: StatusCode, code: &str) -> Response {
    (status, Json(json!({"code":code}))).into_response()
}
fn unavailable() -> Response { failure(StatusCode::SERVICE_UNAVAILABLE, "agui_persistence_unavailable") }
fn owner(policy: &AdmissionPolicy, user: &UserContext, workspace: Option<&str>) -> Result<String, Response> {
    authorize_context(&policy.0, Some(user), workspace)
        .map_err(|_| failure(StatusCode::FORBIDDEN, "agui_authority_denied"))?;
    CredentialPrincipal::from_verified_context(user).map(|p| p.storage_key())
        .map_err(|_| failure(StatusCode::UNAUTHORIZED, "verified_principal_required"))
}
fn valid_id(id: &str) -> bool { !id.is_empty() && id.len() <= 512 && id.trim() == id && !id.chars().any(char::is_control) }
fn validate_resume(input: &RunAgentInput) -> Result<(), Response> {
    let mut seen = HashSet::new();
    for entry in &input.resume {
        if !valid_id(&entry.interrupt_id) || !seen.insert(entry.interrupt_id.clone())
            || !matches!(entry.status.as_str(),"resolved"|"cancelled") {
            return Err(failure(StatusCode::BAD_REQUEST,"invalid_resume"));
        }
    }
    Ok(())
}
fn approval_answer(entry: &ResumeEntry) -> Result<bool,Response> {
    if entry.status == "cancelled" { return Ok(false); }
    entry.payload.as_ref().and_then(|p|p.get("approved")).and_then(Value::as_bool)
        .ok_or_else(||failure(StatusCode::BAD_REQUEST,"approval_response_requires_boolean"))
}

async fn run(State(state): State<Arc<RunApiState>>, Extension(policy): Extension<AdmissionPolicy>,
    Extension(user): Extension<UserContext>, Path(agent): Path<String>, headers: HeaderMap,
    host: Option<Extension<HostAuthenticated>>, capture: Option<Extension<AuthenticatedCredentialCapture>>,
    Json(raw): Json<Value>) -> Response {
    let input: RunAgentInput = match serde_json::from_value(raw.clone()) { Ok(input) => input, Err(_) => return failure(StatusCode::BAD_REQUEST,"invalid_run_agent_input") };
    if !valid_id(&input.thread_id) || !valid_id(&input.run_id) || !valid_id(&agent)
        || input.protocol_version.as_deref().is_some_and(|version| version != "1.0") {
        return failure(StatusCode::BAD_REQUEST,"unsupported_agui_protocol_or_identity");
    }
    if raw.get("forwardedProps").is_some_and(Value::is_null) { return failure(StatusCode::BAD_REQUEST,"forwarded_props_must_not_be_null"); }
    if input.context.iter().any(|value| !value.get("description").is_some_and(Value::is_string) || !value.get("value").is_some_and(Value::is_string))
        || input.messages.iter().any(|value| !value.get("id").is_some_and(Value::is_string)
            || value.get("metadata").is_some_and(|metadata| !metadata.is_object()))
        || raw.get("resume").and_then(Value::as_array).is_some_and(|entries| entries.iter().any(|entry| entry.get("payload").is_some_and(Value::is_null)
            || entry.get("metadata").is_some_and(|metadata| !metadata.is_object()))) {
        return failure(StatusCode::BAD_REQUEST,"invalid_run_agent_input");
    }
    if let Err(response) = caller_tool_definitions(&input.tools) { return response; }
    if let Err(response) = caller_controls(&input) { return response; }
    let workspace = super::targets::workspace(&headers).map(str::to_owned);
    let owner = match owner(&policy, &user, workspace.as_deref()) { Ok(owner) => owner, Err(response) => return response };
    let Some(db) = state.manager.persistence.as_ref().filter(|db| db.supports_durable_agui_runs()) else { return unavailable(); };
    if !input.tools.is_empty() && !db.supports_durable_client_tools() { return unavailable(); }
    let digest = Sha256::digest(raw.to_string().as_bytes()).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    match db.load_agui_run(&owner,&agent,&input.thread_id,&input.run_id).await {
        Ok(Some(row)) => {
            if row.input_digest != digest || row.workspace_id != workspace { return failure(StatusCode::CONFLICT,"run_identity_conflict"); }
            return stream(state,user,row,&headers).await;
        }
        Err(_) => return unavailable(), Ok(None) => {}
    }
    // An identical result retry may use a fresh wire run ID after another
    // contender already admitted the continuation. Resolve its completed
    // caller batch before enforcing the latest-run rule for new execution.
    if input.resume.is_empty()
        && input.messages.last().and_then(|message|message.get("role")).and_then(Value::as_str) == Some("tool")
        && let Some(parent) = &input.parent_run_id
    {
        let predecessor = match db.load_agui_run(&owner,&agent,&input.thread_id,parent).await {
            Ok(Some(row)) if row.workspace_id == workspace => Some(row),
            Ok(_) => None,
            Err(_) => return unavailable(),
        };
        if let Some(predecessor) = predecessor
            && let Some(source) = state.manager.get_run_for_context(&user,&predecessor.native_run_id).await
        {
            let history = match caller_history(&input.messages) { Ok(history) => history, Err(response) => return response };
            let call_id = history.last().and_then(|message|message.get("tool_call_id")).and_then(Value::as_str);
            if let Some(call_id) = call_id {
                let tenant = user.tenant_id.as_ref().map(|tenant|tenant.as_str());
                let saved = match super::client_tools::load_by_call_id(db.as_ref(),&owner,tenant,&source.agent_id,call_id).await {
                    Ok(saved) => saved,
                    Err(_) => return failure(StatusCode::CONFLICT,"caller_tool_continuation_conflict"),
                };
                if saved.as_ref().is_some_and(|saved|
                    saved.state == crate::uar::persistence::client_tools::ClientToolContinuationState::Completed
                        && saved.source_run_id == predecessor.native_run_id
                        && saved.session_id.as_deref() == Some(predecessor.session_id.as_str())
                        && saved.protocol == "agui")
                {
                    match prepare_frontend_continuation(&state,&user,&predecessor,&input).await {
                        Ok(super::client_tools::ClientToolPreparation::Replay {response,..}) => {
                            let saved: AguiRunBinding = match serde_json::from_value(response) {
                                Ok(saved) => saved, Err(_) => return unavailable(),
                            };
                            if saved.owner != owner || saved.agent_id != agent || saved.thread_id != input.thread_id
                                || saved.workspace_id != workspace || saved.previous_wire_run_id.as_deref() != Some(parent.as_str()) {
                                return failure(StatusCode::CONFLICT,"caller_tool_replay_context_conflict");
                            }
                            let saved = match db.load_agui_run(&saved.owner,&saved.agent_id,&saved.thread_id,&saved.wire_run_id).await {
                                Ok(Some(row)) if row.native_run_id == saved.native_run_id => row,
                                _ => return unavailable(),
                            };
                            return stream(state,user,saved,&headers).await;
                        }
                        Ok(_) => return failure(StatusCode::CONFLICT,"caller_tool_continuation_conflict"),
                        Err(response) => return response,
                    }
                }
            }
        }
    }
    let previous = match db.list_agui_runs(&owner,&agent,&input.thread_id).await {
        Ok(rows) => rows.into_iter().max_by_key(|row| row.ordinal), Err(_) => return unavailable(),
    };
    let previous = match previous {
        Some(row) => match reconcile(&state,&user,row).await { Ok(row) => Some(row), Err(_) => return unavailable() }, None => None,
    };
    if previous.as_ref().is_some_and(|row| row.terminal.is_none() || row.workspace_id != workspace)
        || input.parent_run_id.as_ref().is_some_and(|id| previous.as_ref().is_none_or(|row| &row.wire_run_id != id)) {
        return failure(StatusCode::CONFLICT,"thread_run_not_closed");
    }
    if let Err(response) = validate_resume(&input) { return response; }
    let resuming = !input.resume.is_empty();
    let mut answers = Vec::new();
    let mut elicitation_answers = Vec::new();
    let expected: HashSet<_> = previous.as_ref().and_then(|row| row.terminal.as_ref())
        .map(|terminal| terminal.interrupt_ids.iter().cloned().collect()).unwrap_or_default();
    if expected != input.resume.iter().map(|entry| entry.interrupt_id.clone()).collect::<HashSet<_>>() {
        return failure(StatusCode::CONFLICT,"resume_must_cover_all_open_interrupts");
    }
    let frontend_pending = previous.as_ref().and_then(|row|row.terminal.as_ref())
        .and_then(|terminal|terminal.event.pointer("/outcome/pendingToolCallIds"))
        .and_then(Value::as_array).is_some_and(|calls|!calls.is_empty());
    let mut caller_claim = None;
    let mut caller_continuation = None;
    if frontend_pending {
        if resuming || input.messages.last().and_then(|message|message.get("role")).and_then(Value::as_str) != Some("tool") {
            return failure(StatusCode::CONFLICT,"caller_tool_results_required");
        }
        match prepare_frontend_continuation(&state,&user,previous.as_ref().expect("frontend predecessor"),&input).await {
            Ok(super::client_tools::ClientToolPreparation::Continue {input,claim}) => {
                caller_continuation = Some(input); caller_claim = Some(claim);
            }
            Ok(super::client_tools::ClientToolPreparation::Replay {response,..}) => {
                let saved: AguiRunBinding = match serde_json::from_value(response) {
                    Ok(saved) => saved, Err(_) => return unavailable(),
                };
                if saved.owner != owner || saved.agent_id != agent || saved.thread_id != input.thread_id || saved.workspace_id != workspace {
                    return failure(StatusCode::CONFLICT,"caller_tool_replay_context_conflict");
                }
                let saved = match db.load_agui_run(&saved.owner,&saved.agent_id,&saved.thread_id,&saved.wire_run_id).await {
                    Ok(Some(row)) if row.native_run_id == saved.native_run_id => row,
                    _ => return unavailable(),
                };
                return stream(state,user,saved,&headers).await;
            }
            Ok(super::client_tools::ClientToolPreparation::Fresh(_)) => return failure(StatusCode::CONFLICT,"caller_tool_results_required"),
            Err(response) => return response,
        }
    }
    let mut views = Vec::new();
    let mut elicitation_views = Vec::new();
    if let Some(previous) = previous.as_ref().filter(|_| resuming) {
        views = match pending(&state,&user,previous).await { Ok(views) => views, Err(_) => return unavailable() };
        elicitation_views = match pending_elicitations(&state,&user,previous).await { Ok(views) => views, Err(_) => return unavailable() };
        if views.iter().map(|view| view.record.challenge_id.clone()).chain(elicitation_views.iter().map(|view|view.record.id.clone())).collect::<HashSet<_>>() != expected {
            return failure(StatusCode::CONFLICT,"interrupt_not_pending");
        }
        for entry in &input.resume {
            if views.iter().any(|view|view.record.challenge_id == entry.interrupt_id) {
                match approval_answer(entry) { Ok(approved) => answers.push((entry.interrupt_id.clone(),approved)), Err(response) => return response }
            } else {
                let value = if entry.status == "cancelled" { json!({"action":"cancel"}) } else { entry.payload.clone().unwrap_or(Value::Null) };
                let response: rmcp::model::ElicitResult = match serde_json::from_value(value) { Ok(response) => response, Err(_) => return failure(StatusCode::BAD_REQUEST,"invalid_elicitation_response") };
                let view = elicitation_views.iter().find(|view|view.record.id == entry.interrupt_id).expect("validated interrupt identity");
                if !view.resolvable { return failure(StatusCode::CONFLICT,"elicitation_connection_unavailable"); }
                if view.record.validate_response(&response).is_err() { return failure(StatusCode::BAD_REQUEST,"invalid_elicitation_response"); }
                elicitation_answers.push((entry.interrupt_id.clone(),response));
            }
        }
        // A checkpoint is one protected continuation. Never partially answer a
        // batch whose separate cold continuations cannot be atomically admitted.
        if views.iter().any(|view| !view.resolvable) && (views.len() != 1 || !elicitation_views.is_empty()) {
            return failure(StatusCode::CONFLICT,"multiple_cold_continuations_unavailable");
        }
        if views.iter().any(|view| !view.resolvable && !view.recovery_available
            && answers.iter().any(|(id,approved)| id == &view.record.challenge_id && *approved)) {
            return failure(StatusCode::CONFLICT,"protected_continuation_unavailable");
        }
    }
    let warm = resuming && views.iter().all(|view| view.resolvable) && elicitation_views.iter().all(|view|view.resolvable);
    let cold_denied = !views.is_empty() && elicitation_views.is_empty() && !warm && answers.iter().all(|(_,approved)| !approved);
    let native = if warm || cold_denied { previous.as_ref().expect("resume predecessor").native_run_id.clone() } else { uuid::Uuid::new_v4().to_string() };
    let cursor = if warm { previous.as_ref().and_then(|row|row.terminal.as_ref()).map_or(0,|terminal|terminal.source_cursor) } else { 0 };
    let session_id = previous.as_ref().map(|row|row.session_id.clone()).unwrap_or_else(|| format!("agui-{}",AguiRunBinding::key(&owner,&agent,&input.thread_id,"")));
    let mut resumed_tool_ids: Vec<String> = views.iter().map(|view|view.record.tool_call_id.clone()).collect();
    if warm {
        for event in state.manager.history_since(&native,None).await.unwrap_or_default() {
            if event.id > cursor { continue; }
            if let NormalizedEvent::ToolStart {tool_call_id,..} | NormalizedEvent::ToolCallApprovalRequired {tool_call_id,..} = event.event {
                if !resumed_tool_ids.contains(&tool_call_id) { resumed_tool_ids.push(tool_call_id); }
            }
        }
    }
    let row = AguiRunBinding { owner, agent_id:agent.clone(), thread_id:input.thread_id.clone(), wire_run_id:input.run_id.clone(), native_run_id:native.clone(), session_id:session_id.clone(), workspace_id:workspace.clone(), input_digest:digest, ordinal:previous.as_ref().map_or(1,|row|row.ordinal+1), revision:1, start_cursor:cursor, previous_wire_run_id:previous.as_ref().map(|row|row.wire_run_id.clone()), resumed_tool_ids, terminal:None };
    // Validate client history/resources before reserving transport identity.
    let resources = input.forwarded_props.as_ref().and_then(|props|props.get("uar")).cloned().unwrap_or_else(||json!({}));
    if !resources.is_object() { return failure(StatusCode::BAD_REQUEST,"invalid_host_resources"); }
    let create = if !resuming {
        match create_request(&input,&agent,&session_id,resources.clone(),caller_continuation) { Ok(request) => Some(request), Err(response) => return response }
    } else { None };
    if resuming && ["input","history","session_id","artifact"].iter().any(|key| resources.get(key).is_some_and(|value| !value.is_null())) {
        return failure(StatusCode::BAD_REQUEST,"resume_uses_protected_history_and_identity");
    }
    let resume = if resuming {
        match serde_json::from_value::<routes::ResumeRequest>(resources) { Ok(resume) => Some(resume), Err(_) => return failure(StatusCode::BAD_REQUEST,"invalid_resume_resources") }
    } else { None };
    match db.create_agui_run(&row,previous.as_ref()).await { Ok(true) => {}, Ok(false) => return failure(StatusCode::CONFLICT,"thread_run_conflict"), Err(_) => return unavailable() }
    let result = if let Some(request) = create {
        routes::admit_run(Arc::clone(&state),user.clone(),headers.clone(),host.is_some(),request,Some(native),None,capture.map(|Extension(value)|value)).await.map(|_|()).map_err(IntoResponse::into_response)
    } else {
        let mut result = Ok(());
        for (id,approved) in answers {
            result = routes::resolve_host_approval(Arc::clone(&state),user.clone(),capture.as_ref().map(|Extension(value)|value.clone()),previous.as_ref().expect("resume predecessor").native_run_id.clone(),id,approved,approval_workspace(&row).map(str::to_owned),resume.clone().expect("resume resources"),if warm || cold_denied { None } else { Some(native.clone()) },None).await.map(|_|());
            if result.is_err() { break; }
        }
        if result.is_ok() {
            for (id,response) in elicitation_answers {
                result = super::mcp_interactions::resolve_for_context(&state.manager,&user,&row.native_run_id,approval_workspace(&row),&id,response).await
                    .map_err(|_|failure(StatusCode::SERVICE_UNAVAILABLE,"elicitation_response_unconfirmed"))
                    .and_then(|committed| if committed { Ok(()) } else { Err(failure(StatusCode::CONFLICT,"elicitation_response_conflict")) });
                if result.is_err() { break; }
            }
        }
        result
    };
    if let Err(response) = result {
        // An unknown admission/decision outcome keeps the reservation held.
        // A retry reads this mapping; it never repeats the execution command.
        observe(Arc::clone(&state),user.clone(),row.clone()).await;
        return response;
    }
    if let Some(claim) = caller_claim {
        // For AG-UI, Completed stores a durable admission receipt, not model
        // completion. Replays attach to this exact already-admitted run.
        let receipt = match serde_json::to_value(&row) { Ok(receipt) => receipt, Err(_) => return unavailable() };
        if super::client_tools::complete(db.as_ref(),&claim,receipt,Vec::new()).await.is_err() { return unavailable(); }
    }
    if cold_denied {
        let terminal = AguiTerminal { source_cursor:0,event:json!({"type":"RUN_ERROR","code":"approval_denied","message":"The protected tool continuation was denied"}),interrupt_ids:Vec::new() };
        if seal(&state,&row,terminal).await.is_err() { return unavailable(); }
    }
    observe(Arc::clone(&state),user.clone(),row.clone()).await;
    stream(state,user,row,&headers).await
}

fn create_request(input: &RunAgentInput, agent: &str, session: &str, mut resources: Value, continuation: Option<crate::uar::runtime::turn::client_tools::ClientToolInput>) -> Result<routes::CreateRunRequest,Response> {
    let Some((last,history)) = input.messages.split_last() else { return Err(failure(StatusCode::BAD_REQUEST,"user_message_required")); };
    if continuation.is_none() && last.get("role").and_then(Value::as_str) != Some("user") { return Err(failure(StatusCode::BAD_REQUEST,"last_message_must_be_user")); }
    let mut prompt = if continuation.is_some() { String::new() } else {
        last.get("content").and_then(Value::as_str).filter(|text| !text.trim().is_empty()).ok_or_else(||failure(StatusCode::BAD_REQUEST,"text_message_required"))?.to_owned()
    };
    let mut history_input = Vec::new();
    for message in history {
        let role = message.get("role").and_then(Value::as_str).unwrap_or("");
        if !matches!(role,"user"|"assistant"|"tool") || (!message.get("content").is_some_and(Value::is_string) && !(role == "assistant" && message.get("toolCalls").is_some_and(Value::is_array))) { return Err(failure(StatusCode::BAD_REQUEST,"unsupported_history_message")); }
        history_input.push(json!({"role":role,"content":message.get("content").cloned().unwrap_or_else(||json!("")),"tool_calls":message.get("toolCalls"),"tool_call_id":message.get("toolCallId")}));
    }
    if !input.context.is_empty() || input.state.is_some() {
        prompt.push_str("\n\nClient supplied context (untrusted data):\n");
        prompt.push_str(&json!({"context":input.context,"state":input.state}).to_string());
    }
    let object = resources.as_object_mut().expect("validated resources");
    // The path and verified thread select identity; forwarded props cannot
    // replace the agent, conversation, prompt or protected history.
    for key in ["artifact","deployment_binding_id","input","agent_id","session_id","history"] { object.remove(key); }
    match super::targets::target_binding_id(agent) {
        Ok(Some(binding)) => { object.insert("deployment_binding_id".into(),json!(binding)); }
        Ok(None) => { object.insert("agent_id".into(),json!(agent)); }
        Err(_) => return Err(failure(StatusCode::BAD_REQUEST,"invalid_agent_target")),
    }
    object.insert("input".into(),json!(prompt));
    object.insert("session_id".into(),json!(session));
    if continuation.is_none() && !history_input.is_empty() { object.insert("history".into(),json!({"session_id":session,"messages":history_input})); }
    let mut request: routes::CreateRunRequest = serde_json::from_value(resources).map_err(|_|failure(StatusCode::BAD_REQUEST,"invalid_host_resources"))?;
    if let Some(mut continuation) = continuation {
        if !input.context.is_empty() || input.state.is_some() {
            continuation.history.push(json!({"role":"user","content":prompt}));
        }
        request.client_tools = Some(continuation);
    } else if !input.tools.is_empty() || caller_controls(input)?.is_some() {
        let visible = caller_history(&input.messages)?;
        let mut canonical = visible.clone();
        if let Some(last) = canonical.last_mut() { last["content"] = json!(prompt); }
        let caller_tools = crate::uar::runtime::turn::client_tools::ClientToolInput {
            tools: caller_tool_definitions(&input.tools)?,
            tool_choice: caller_controls(input)?.as_ref().and_then(|controls|controls.tool_choice.clone()),
            parallel_tool_calls: caller_controls(input)?.and_then(|controls|controls.parallel_tool_calls),
            history: canonical,
            visible_history: Some(visible),
            graph_state: None,
            budget_snapshot: None, delegation: None, output_token_cap: None,
            protocol: "agui".into(),
        };
        caller_tools.validate().map_err(|_|failure(StatusCode::BAD_REQUEST,"invalid_frontend_tool"))?;
        request.client_tools = Some(caller_tools);
    }
    Ok(request)
}

async fn reconnect(State(state): State<Arc<RunApiState>>, Extension(policy): Extension<AdmissionPolicy>, Extension(user): Extension<UserContext>, Path((agent,thread,run)): Path<(String,String,String)>, headers: HeaderMap) -> Response {
    let workspace = super::targets::workspace(&headers);
    let owner = match owner(&policy,&user,workspace) { Ok(owner) => owner, Err(response) => return response };
    let Some(db) = &state.manager.persistence else { return unavailable(); };
    let row = match db.load_agui_run(&owner,&agent,&thread,&run).await { Ok(Some(row)) if row.workspace_id.as_deref() == workspace => row, Ok(_) => return StatusCode::NOT_FOUND.into_response(), Err(_) => return unavailable() };
    stream(state,user,row,&headers).await
}

fn approval_workspace(row: &AguiRunBinding) -> Option<&str> {
    // Native catalog runs bind workspace; builtin artifact runs have no catalog
    // workspace in their protected host context. The transport still checks the
    // caller's selected workspace separately on every request.
    if matches!(super::targets::target_binding_id(&row.agent_id),Ok(Some(_))) { row.workspace_id.as_deref() } else { None }
}
async fn pending(state: &RunApiState, user: &UserContext, row: &AguiRunBinding) -> anyhow::Result<Vec<ApprovalRecordView>> {
    let (_,views) = state.manager.approval_records_for_context(user,&row.native_run_id,approval_workspace(row)).await?;
    Ok(views.into_iter().filter(|view| view.record.state == ApprovalState::Pending).collect())
}
async fn pending_elicitations(state: &RunApiState,user: &UserContext,row: &AguiRunBinding) -> anyhow::Result<Vec<crate::uar::persistence::mcp_elicitations::McpElicitationView>> {
    let views = super::mcp_interactions::list_for_context(&state.manager,user,&row.native_run_id,approval_workspace(row)).await?;
    Ok(views.into_iter().filter(|view|view.record.state == crate::uar::persistence::mcp_elicitations::ElicitationState::Pending).collect())
}
fn finished(row: &AguiRunBinding, outcome: Value) -> Value { json!({"type":"RUN_FINISHED","threadId":row.thread_id,"runId":row.wire_run_id,"outcome":outcome}) }
fn frontend_terminal(row: &AguiRunBinding, cursor: u64, calls: &[crate::llm::ToolCall]) -> AguiTerminal {
    let mut event = finished(row,json!({"type":"success","pendingToolCallIds":calls.iter().map(|call|&call.id).collect::<Vec<_>>()}));
    // Standard metadata retains the handoff projection through loss of bounded
    // native event history. It carries no executable registration or authority.
    event["metadata"] = json!({"uar":{"clientToolCalls":calls}});
    AguiTerminal {source_cursor:cursor,event,interrupt_ids:Vec::new()}
}
async fn seal(state: &RunApiState,row: &AguiRunBinding,terminal: AguiTerminal) -> anyhow::Result<AguiRunBinding> {
    let db = state.manager.persistence.as_ref().ok_or_else(||anyhow::anyhow!("persistence unavailable"))?;
    let mut next = row.clone(); next.revision += 1; next.terminal = Some(terminal);
    if db.compare_and_swap_agui_run(row,&next).await? { return Ok(next); }
    db.load_agui_run(&row.owner,&row.agent_id,&row.thread_id,&row.wire_run_id).await?.ok_or_else(||anyhow::anyhow!("run correlation unavailable"))
}
async fn seal_frontend_handoff(
    state: &RunApiState, user: &UserContext, row: &AguiRunBinding,
    cursor: u64, calls: &[crate::llm::ToolCall],
) -> anyhow::Result<AguiRunBinding> {
    let source = state.manager.get_run_for_context(user, &row.native_run_id).await
        .ok_or_else(|| anyhow::anyhow!("caller_tool_source_unavailable"))?;
    let owner = user.session_owner_key()?;
    let store = state.manager.persistence.as_deref()
        .ok_or_else(|| anyhow::anyhow!("durable_client_tools_unsupported"))?;
    if let Err(error) = super::client_tools::confirm_handoff_for_run(
        state.manager.as_ref(), store, &owner,
        user.tenant_id.as_ref().map(|tenant| tenant.as_str()),
        &source.agent_id, &row.native_run_id,
    ).await {
        let category = if error.to_string() == "run_settlement_unconfirmed" {
            "run_settlement_unconfirmed"
        } else {
            "caller_tool_settlement_failed"
        };
        tracing::warn!(run_id = %row.native_run_id, error_category = category,
            "AG-UI caller-tool source cleanup was not confirmed");
        return seal(state, row, AguiTerminal {
            source_cursor: cursor,
            event: json!({"type":"RUN_ERROR","code":"host_cleanup_unconfirmed",
                "message":"The host did not confirm caller-tool source cleanup"}),
            interrupt_ids: Vec::new(),
        }).await;
    }
    seal(state, row, frontend_terminal(row, cursor, calls)).await
}
async fn reconcile(state: &RunApiState,user: &UserContext,row: AguiRunBinding) -> anyhow::Result<AguiRunBinding> {
    if row.terminal.is_some() { return Ok(row); }
    let db = state.manager.persistence.as_ref().ok_or_else(||anyhow::anyhow!("agui reconcile: persistence unavailable"))?;
    let row = db.load_agui_run(&row.owner,&row.agent_id,&row.thread_id,&row.wire_run_id).await.context("agui reconcile: load_agui_run")?.ok_or_else(||anyhow::anyhow!("run correlation unavailable")).context("agui reconcile: run correlation unavailable")?;
    if row.terminal.is_some() { return Ok(row); }
    let history = state.manager.history_since(&row.native_run_id,Some(row.start_cursor)).await.unwrap_or_default();
    let cursor = history.last().map_or(row.start_cursor,|event|event.id);
    let views = pending(state,user,&row).await.context("agui reconcile: pending approvals")?;
    let elicitations = pending_elicitations(state,user,&row).await.context("agui reconcile: pending elicitations")?;
    if !views.is_empty() || !elicitations.is_empty() {
        let ids = views.iter().map(|view|view.record.challenge_id.clone()).chain(elicitations.iter().map(|view|view.record.id.clone())).collect();
        let mut interrupts: Vec<_> = views.iter().map(|view|json!({"id":view.record.challenge_id,"reason":"tool_approval","message":format!("Approve {}",view.record.tool_name),"toolCallId":view.record.tool_call_id,"expiresAt":view.record.expires_at.to_rfc3339(),"responseSchema":{"type":"object","properties":{"approved":{"type":"boolean"}},"required":["approved"],"additionalProperties":false}})).collect();
        for view in elicitations {
            let request = serde_json::to_value(&view.record.request)?;
            let mut properties = json!({"action":{"type":"string","enum":["accept","decline","cancel"]}});
            if let Some(schema) = request.get("requestedSchema") { properties["content"] = schema.clone(); }
            interrupts.push(json!({"id":view.record.id,"reason":"mcp_elicitation","message":request.get("message").and_then(Value::as_str).unwrap_or("MCP server requires input"),"expiresAt":view.record.expires_at.to_rfc3339(),"responseSchema":{"type":"object","properties":properties,"required":["action"],"additionalProperties":false},"metadata":{"uar":{"serverName":view.record.server_name,"request":request,"resolvable":view.resolvable}}}));
        }
        return seal(state,&row,AguiTerminal {source_cursor:cursor,event:finished(&row,json!({"type":"interrupt","interrupts":interrupts})),interrupt_ids:ids}).await.context("agui reconcile: seal");
    }
    // This event is published only after the host commits the caller-owned
    // batch. The native AG-UI frontend handoff is a completed segment whose
    // success outcome names the calls the application must answer next.
    for event in &history {
        if let NormalizedEvent::ClientToolsRequested {calls,..} = &event.event {
            return seal_frontend_handoff(state,user,&row,event.id,calls).await.context("agui reconcile: seal caller-tool handoff");
        }
    }
    // A process may stop after committing a caller batch and before publishing
    // its event. Recover that exact handoff, without submitting another run.
    if let Some(source) = state.manager.get_run_for_context(user,&row.native_run_id).await {
        let principal = CredentialPrincipal::from_verified_context(user)?;
        let session_owner = user.session_owner_key()?;
        if let Some(pending) = db.load_client_tool_continuation_for_run(&session_owner,principal.tenant_id(),&source.agent_id,&row.native_run_id).await? {
            anyhow::ensure!(pending.protocol == "agui" && pending.session_id.as_deref() == Some(row.session_id.as_str()), "caller_tool_context_conflict");
            let calls: Vec<crate::llm::ToolCall> = pending.visible_transcript.as_array()
                .and_then(|messages|messages.last()).and_then(|message|message.get("tool_calls"))
                .cloned().map(serde_json::from_value).transpose()?.ok_or_else(||anyhow::anyhow!("caller_tool_calls_unavailable"))?;
            anyhow::ensure!(calls.iter().map(|call|&call.id).eq(pending.pending_call_ids.iter()), "caller_tool_batch_conflict");
            return seal_frontend_handoff(state,user,&row,cursor,&calls).await.context("agui reconcile: recover caller-tool handoff");
        }
    }
    let terminal_observed = history.iter().any(|event| matches!(&event.event,
        NormalizedEvent::RunDone {..} | NormalizedEvent::RunDoneWithUsage {..}
            | NormalizedEvent::Cancelled {..} | NormalizedEvent::Error {..}));
    let status = if terminal_observed {
        // The terminal frame precedes outer producer finalization. Join that
        // producer rather than waiting for a second event that it never emits.
        state.manager.get_run_for_context(user,&row.native_run_id).await
            .ok_or_else(||anyhow::anyhow!("run correlation unavailable"))?;
        state.manager.await_settled_run_status(&row.native_run_id).await?
    } else {
        let Some(status) = state.manager.settled_run_status_for_context(user,&row.native_run_id).await else { return Ok(row); };
        status
    };
    if status == crate::uar::domain::runs::RunStatus::Error {
        let (source_cursor,event) = history.iter().find_map(|event| match &event.event {
            NormalizedEvent::Error {code,message,..} => Some((event.id,json!({"type":"RUN_ERROR","code":code,"message":message}))),
            _ => None,
        }).unwrap_or_else(||(cursor,json!({"type":"RUN_ERROR","code":"host_cleanup_unconfirmed","message":"The host did not confirm successful cleanup"})));
        return seal(state,&row,AguiTerminal {source_cursor,event,interrupt_ids:Vec::new()}).await.context("agui reconcile: seal");
    }
    for event in &history {
        let terminal = match &event.event {
            NormalizedEvent::RunDone {..} | NormalizedEvent::RunDoneWithUsage {..} => Some(finished(&row,json!({"type":"success"}))),
            NormalizedEvent::Cancelled {..} => Some(finished(&row,json!({"type":"cancelled"}))),
            NormalizedEvent::Error {code,message,..} => Some(json!({"type":"RUN_ERROR","code":code,"message":message})),
            _ => None,
        };
        if let Some(terminal) = terminal { return seal(state,&row,AguiTerminal {source_cursor:event.id,event:terminal,interrupt_ids:Vec::new()}).await.context("agui reconcile: seal"); }
    }
    Ok(row)
}
async fn observe(state: Arc<RunApiState>, user: UserContext, row: AguiRunBinding) {
    // Subscribe before the first reconciliation. A retained broadcast receiver
    // closes the event/snapshot race without repeatedly rereading persistence.
    let Some(mut events) = state.manager.subscribe(&row.native_run_id).await else {
        state.manager.lifecycle_scope().record_unresolved_effect(format!(
            "agui-terminal-projection:{}", row.native_run_id
        ));
        return;
    };
    let mut committed_work = state.manager.subscribe_committed_work();
    let scope = state.manager.lifecycle_scope();
    let cancellation = scope.cancellation_token();
    let task_row = row.clone();
    if scope.spawn("agui-terminal-projection", async move {
        loop {
            match reconcile(&state, &user, task_row.clone()).await {
                Ok(updated) if updated.terminal.is_some() => return Ok(()),
                Ok(_) => {}
                Err(error) => {
                    return Err(anyhow::anyhow!(
                        "AG-UI terminal projection could not reconcile: {error:#}"
                    ));
                }
            }
            tokio::select! {
                _ = cancellation.cancelled() => return Ok(()),
                event = events.recv() => match event {
                    Ok(_) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        return Err(anyhow::anyhow!("AG-UI terminal projection lost its run event source"));
                    }
                },
                changed = committed_work.changed() => {
                    if changed.is_err() {
                        return Err(anyhow::anyhow!("AG-UI terminal projection lost its committed-work source"));
                    }
                },
            }
        }
    }).is_err() {
        scope.record_unresolved_effect(format!("agui-terminal-projection:{}", row.native_run_id));
    }
}

#[derive(Default)]
struct Projector { text: bool, tools: HashSet<String> }
impl Projector {
    fn caller_tools(&mut self,row: &AguiRunBinding,calls: &[crate::llm::ToolCall]) -> Vec<Value> {
        calls.iter().filter_map(|call| {
            if row.resumed_tool_ids.contains(&call.id) || !self.tools.insert(call.id.clone()) { return None; }
            Some(vec![json!({"type":"TOOL_CALL_START","toolCallId":call.id,"toolCallName":call.function.name}),json!({"type":"TOOL_CALL_ARGS","toolCallId":call.id,"delta":call.function.arguments}),json!({"type":"TOOL_CALL_END","toolCallId":call.id})])
        }).flatten().collect()
    }
    fn project(&mut self,row: &AguiRunBinding,event: &NormalizedEvent) -> Vec<Value> {
        let message = format!("{}:assistant",row.wire_run_id);
        match event {
            NormalizedEvent::ChatDelta {text_delta,..} if !text_delta.is_empty() => {
                let mut events = Vec::new();
                if !self.text { self.text = true; events.push(json!({"type":"TEXT_MESSAGE_START","messageId":message,"role":"assistant"})); }
                events.push(json!({"type":"TEXT_MESSAGE_CONTENT","messageId":message,"delta":text_delta})); events
            }
            NormalizedEvent::ToolStart {tool_call_id,tool,input,..} if !row.resumed_tool_ids.contains(tool_call_id) && self.tools.insert(tool_call_id.clone()) => vec![json!({"type":"TOOL_CALL_START","toolCallId":tool_call_id,"toolCallName":tool}),json!({"type":"TOOL_CALL_ARGS","toolCallId":tool_call_id,"delta":input.to_string()}),json!({"type":"TOOL_CALL_END","toolCallId":tool_call_id})],
            NormalizedEvent::ToolCallApprovalRequired {tool_call_id,name,arguments_json,..} if self.tools.insert(tool_call_id.clone()) => vec![json!({"type":"TOOL_CALL_START","toolCallId":tool_call_id,"toolCallName":name}),json!({"type":"TOOL_CALL_ARGS","toolCallId":tool_call_id,"delta":arguments_json}),json!({"type":"TOOL_CALL_END","toolCallId":tool_call_id})],
            NormalizedEvent::ClientToolsRequested {calls,..} => self.caller_tools(row,calls),
            NormalizedEvent::ToolEnd {tool_call_id,output,..} if self.tools.contains(tool_call_id) || row.resumed_tool_ids.contains(tool_call_id) => vec![json!({"type":"TOOL_CALL_RESULT","messageId":format!("{}:result:{}",row.wire_run_id,tool_call_id),"toolCallId":tool_call_id,"role":"tool","content":output.to_string()})],
            // Mirrors adapters.rs:134 (`uar.skill.activated`): same CUSTOM `name` and `value` shape. The adapter's closure is not callable
            // here, and its `runId`/`threadId` carry the native run id, whereas this stream's identity is the wire thread/run (as RUN_STARTED).
            NormalizedEvent::SkillActivated {skill_id,title,selection_method,..} => vec![json!({"type":"CUSTOM","name":"uar.skill.activated","value":{"skill":{"id":skill_id,"title":title},"selectionMethod":selection_method},"threadId":row.thread_id,"runId":row.wire_run_id,"agentId":row.agent_id})],
            NormalizedEvent::AgentSelected {source_agent_id,selected_agent_id,selected_agent_name,selected_thread_id,selection_method,roster_revision,..} => vec![json!({"type":"CUSTOM","name":"uar.agent.selected","value":{"sourceAgentId":source_agent_id,"agent":{"id":selected_agent_id,"title":selected_agent_name},"selectedThreadId":selected_thread_id,"selectionMethod":selection_method,"rosterRevision":roster_revision},"threadId":row.thread_id,"runId":row.wire_run_id,"agentId":row.agent_id})],
            _ => Vec::new(),
        }
    }
    fn close(&mut self,row: &AguiRunBinding) -> Option<Value> {
        if !self.text { return None; } self.text=false;
        Some(json!({"type":"TEXT_MESSAGE_END","messageId":format!("{}:assistant",row.wire_run_id)}))
    }
}
fn frame(row: &AguiRunBinding,cursor:u64,index:usize,value:Value) -> Event {
    Event::default().id(format!("{}:{cursor}:{index}",row.storage_key())).data(value.to_string())
}
async fn stream(state: Arc<RunApiState>,user: UserContext,row: AguiRunBinding,headers: &HeaderMap) -> Response {
    let after = match headers.get("last-event-id") {
        None => None,
        Some(value) => {
            let Some((key,tail)) = value.to_str().ok().and_then(|value|value.split_once(':')) else { return failure(StatusCode::BAD_REQUEST,"invalid_event_cursor"); };
            let Some((cursor,index)) = tail.split_once(':').and_then(|(cursor,index)|Some((cursor.parse::<u64>().ok()?,index.parse::<usize>().ok()?))) else { return failure(StatusCode::BAD_REQUEST,"invalid_event_cursor"); };
            if key != row.storage_key() { return failure(StatusCode::BAD_REQUEST,"event_cursor_run_mismatch"); } Some((cursor,index))
        }
    };
    let body = async_stream::stream! {
        let mut row = row;
        // Subscribe before the first durable reconciliation so an event cannot
        // be lost between snapshot and wait.
        let mut events = state.manager.subscribe(&row.native_run_id).await;
        let mut committed_work = state.manager.subscribe_committed_work();
        let shutdown = state.manager.root_cancellation_token();
        let mut projector = Projector::default();
        let mut cursor = row.start_cursor;
        if after.is_none() { yield Ok::<Event,Infallible>(frame(&row,cursor,0,json!({"type":"RUN_STARTED","threadId":row.thread_id,"runId":row.wire_run_id,"protocolVersion":"1.0"}))); }
        loop {
            row = match reconcile(&state,&user,row.clone()).await { Ok(row) => row, Err(error) => { tracing::warn!(error = %format!("{error:#}"), native_run_id = %row.native_run_id, cursor, "AG-UI stream projection read failed"); yield Ok(frame(&row,cursor,99,json!({"type":"RUN_ERROR","code":"projection_unavailable","message":"Durable stream projection is unavailable"}))); break; } };
            let history = state.manager.history_since(&row.native_run_id,Some(cursor)).await;
            if let Some(history) = history {
                if history.first().is_some_and(|event|event.id > cursor.saturating_add(1)) {
                    yield Ok(frame(&row,cursor,99,json!({"type":"RUN_ERROR","code":"stream_history_unavailable","message":"The requested stream prefix is no longer retained"}))); break;
                }
                for event in history {
                    if row.terminal.as_ref().is_some_and(|terminal| event.id > terminal.source_cursor) { break; }
                    cursor = event.id;
                    for (index,value) in projector.project(&row,&event.event).into_iter().enumerate() {
                        if after.is_none_or(|after|(cursor,index+1)>after) { yield Ok(frame(&row,cursor,index+1,value)); }
                    }
                }
            } else if row.terminal.is_none() {
                yield Ok(frame(&row,cursor,99,json!({"type":"RUN_ERROR","code":"execution_status_unconfirmed","message":"The reserved execution is not available; retrying this run will not replay it"}))); break;
            }
            if let Some(terminal) = &row.terminal {
                let cursor = terminal.source_cursor;
                let mut terminal_index = 100;
                if let Some(calls) = terminal.event.pointer("/metadata/uar/clientToolCalls") {
                    let calls = match serde_json::from_value::<Vec<crate::llm::ToolCall>>(calls.clone()) {
                        Ok(calls) => calls,
                        Err(_) => { yield Ok(frame(&row,cursor,99,json!({"type":"RUN_ERROR","code":"caller_tool_projection_unavailable","message":"The durable caller-tool projection is unavailable"}))); break; }
                    };
                    terminal_index = calls.len().saturating_mul(3).saturating_add(100);
                    for (index,value) in projector.caller_tools(&row,&calls).into_iter().enumerate() {
                        if after.is_none_or(|after|(cursor,index+1)>after) { yield Ok(frame(&row,cursor,index+1,value)); }
                    }
                }
                if let Some(value) = projector.close(&row) {
                    if after.is_none_or(|after|(cursor,terminal_index-10)>after) { yield Ok(frame(&row,cursor,terminal_index-10,value)); }
                }
                if after.is_none_or(|after|(cursor,terminal_index)>after) { yield Ok(frame(&row,cursor,terminal_index,terminal.event.clone())); }
                break;
            }
            let Some(events) = events.as_mut() else {
                yield Ok(frame(&row,cursor,99,json!({"type":"RUN_ERROR","code":"execution_status_unconfirmed","message":"The reserved execution has no live event source"})));
                break;
            };
            tokio::select! {
                _ = shutdown.cancelled() => break,
                event = events.recv() => match event {
                    Ok(_) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {},
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        yield Ok(frame(&row,cursor,99,json!({"type":"RUN_ERROR","code":"execution_status_unconfirmed","message":"The reserved execution event source closed"})));
                        break;
                    }
                },
                changed = committed_work.changed() => {
                    if changed.is_err() {
                        yield Ok(frame(&row,cursor,99,json!({"type":"RUN_ERROR","code":"projection_unavailable","message":"The durable projection event source closed"})));
                        break;
                    }
                },
            }
        }
    };
    Sse::new(body).keep_alive(KeepAlive::default()).into_response()
}
