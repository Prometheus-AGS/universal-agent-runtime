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
    if !input.tools.is_empty() { return failure(StatusCode::BAD_REQUEST,"frontend_tools_not_supported"); }
    let workspace = super::targets::workspace(&headers).map(str::to_owned);
    let owner = match owner(&policy, &user, workspace.as_deref()) { Ok(owner) => owner, Err(response) => return response };
    let Some(db) = state.manager.persistence.as_ref().filter(|db| db.supports_durable_agui_runs()) else { return unavailable(); };
    let digest = Sha256::digest(raw.to_string().as_bytes()).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    match db.load_agui_run(&owner,&agent,&input.thread_id,&input.run_id).await {
        Ok(Some(row)) => {
            if row.input_digest != digest || row.workspace_id != workspace { return failure(StatusCode::CONFLICT,"run_identity_conflict"); }
            return stream(state,user,row,&headers).await;
        }
        Err(_) => return unavailable(), Ok(None) => {}
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
        match create_request(&input,&agent,&session_id,resources.clone()) { Ok(request) => Some(request), Err(response) => return response }
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
    if cold_denied {
        let terminal = AguiTerminal { source_cursor:0,event:json!({"type":"RUN_ERROR","code":"approval_denied","message":"The protected tool continuation was denied"}),interrupt_ids:Vec::new() };
        if seal(&state,&row,terminal).await.is_err() { return unavailable(); }
    }
    observe(Arc::clone(&state),user.clone(),row.clone()).await;
    stream(state,user,row,&headers).await
}

fn create_request(input: &RunAgentInput, agent: &str, session: &str, mut resources: Value) -> Result<routes::CreateRunRequest,Response> {
    let Some((last,history)) = input.messages.split_last() else { return Err(failure(StatusCode::BAD_REQUEST,"user_message_required")); };
    if last.get("role").and_then(Value::as_str) != Some("user") { return Err(failure(StatusCode::BAD_REQUEST,"last_message_must_be_user")); }
    let mut prompt = last.get("content").and_then(Value::as_str).filter(|text| !text.trim().is_empty()).ok_or_else(||failure(StatusCode::BAD_REQUEST,"text_message_required"))?.to_owned();
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
    if !history_input.is_empty() { object.insert("history".into(),json!({"session_id":session,"messages":history_input})); }
    serde_json::from_value(resources).map_err(|_|failure(StatusCode::BAD_REQUEST,"invalid_host_resources"))
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
async fn seal(state: &RunApiState,row: &AguiRunBinding,terminal: AguiTerminal) -> anyhow::Result<AguiRunBinding> {
    let db = state.manager.persistence.as_ref().ok_or_else(||anyhow::anyhow!("persistence unavailable"))?;
    let mut next = row.clone(); next.revision += 1; next.terminal = Some(terminal);
    if db.compare_and_swap_agui_run(row,&next).await? { return Ok(next); }
    db.load_agui_run(&row.owner,&row.agent_id,&row.thread_id,&row.wire_run_id).await?.ok_or_else(||anyhow::anyhow!("run correlation unavailable"))
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
    let Some(status) = state.manager.settled_run_status_for_context(user,&row.native_run_id).await else { return Ok(row); };
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
            NormalizedEvent::ToolEnd {tool_call_id,output,..} if self.tools.contains(tool_call_id) || row.resumed_tool_ids.contains(tool_call_id) => vec![json!({"type":"TOOL_CALL_RESULT","messageId":format!("{}:result:{}",row.wire_run_id,tool_call_id),"toolCallId":tool_call_id,"role":"tool","content":output.to_string()})],
            // Mirrors adapters.rs:134 (`uar.skill.activated`): same CUSTOM `name` and `value` shape. The adapter's closure is not callable
            // here, and its `runId`/`threadId` carry the native run id, whereas this stream's identity is the wire thread/run (as RUN_STARTED).
            NormalizedEvent::SkillActivated {skill_id,title,selection_method,..} => vec![json!({"type":"CUSTOM","name":"uar.skill.activated","value":{"skill":{"id":skill_id,"title":title},"selectionMethod":selection_method},"threadId":row.thread_id,"runId":row.wire_run_id,"agentId":row.agent_id})],
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
                if let Some(value) = projector.close(&row) {
                    if after.is_none_or(|after|(cursor,90)>after) { yield Ok(frame(&row,cursor,90,value)); }
                }
                if after.is_none_or(|after|(cursor,100)>after) { yield Ok(frame(&row,cursor,100,terminal.event.clone())); }
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
