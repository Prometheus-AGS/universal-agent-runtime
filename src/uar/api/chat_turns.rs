//! Native-chat admission correlation over the existing durable AG-UI binding store.
//! Reservations suppress duplicate dispatch; a missing run never permits retry.
use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::routes::RunApiState;
use crate::uar::{
    persistence::{PersistenceLayer, agui_runs::AguiRunBinding},
    security::claims::{CredentialPrincipal, UserContext},
};

const NATIVE_CHAT_AGENT: &str = "native-chat-turn/v1";

/// An existing reservation is a read/reconcile result, never permission to execute.
#[derive(Debug)]
pub enum NativeChatReservation {
    New(AguiRunBinding),
    Existing(AguiRunBinding),
}

fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 512 && id.trim() == id && !id.chars().any(char::is_control)
}

fn thread_key(session: &str, turn: &str) -> String {
    AguiRunBinding::key("native-chat-turn/v1", "session", session, turn)
}

fn canonical_payload(value: &Value) -> Value {
    match value {
        Value::Object(fields) => {
            let sorted = fields.iter().collect::<std::collections::BTreeMap<_, _>>();
            let mut canonical = serde_json::Map::new();
            for (key, value) in sorted {
                canonical.insert(key.clone(), canonical_payload(value));
            }
            Value::Object(canonical)
        }
        Value::Array(values) => Value::Array(values.iter().map(canonical_payload).collect()),
        other => other.clone(),
    }
}

struct NativeChatIntent {
    owner: String,
    thread: String,
    session_id: String,
    turn_id: String,
    message_id: String,
    workspace_id: Option<String>,
    digest: String,
}

impl NativeChatIntent {
    fn new(
        user: &UserContext,
        session: &str,
        turn: &str,
        message: &str,
        workspace_id: Option<String>,
        payload: &Value,
    ) -> anyhow::Result<Self> {
        let owner = CredentialPrincipal::from_verified_context(user)?.storage_key();
        anyhow::ensure!(
            valid_id(session) && valid_id(turn) && valid_id(message),
            "invalid_native_chat_identity"
        );
        let digest = Sha256::digest(serde_json::to_vec(&canonical_payload(&json!({
            "user_message_id":message,"payload":payload,
        })))?)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
        Ok(Self {
            owner,
            thread: thread_key(session, turn),
            session_id: session.into(),
            turn_id: turn.into(),
            message_id: message.into(),
            workspace_id,
            digest,
        })
    }

    fn matches(&self, row: &AguiRunBinding) -> bool {
        row.owner == self.owner
            && row.agent_id == NATIVE_CHAT_AGENT
            && row.thread_id == self.thread
            && row.wire_run_id == self.turn_id
            && row.session_id == self.session_id
            && row.client_message_id.as_deref() == Some(self.message_id.as_str())
            && row.workspace_id == self.workspace_id
            && row.input_digest == self.digest
    }
}

async fn load_native_chat_intent(
    db: &dyn PersistenceLayer,
    intent: &NativeChatIntent,
) -> anyhow::Result<Option<AguiRunBinding>> {
    anyhow::ensure!(
        db.supports_durable_agui_runs() && db.durable_store_eligibility().permits_durable_work(),
        "native_chat_recovery_unavailable"
    );
    let row = db
        .load_agui_run(
            &intent.owner,
            NATIVE_CHAT_AGENT,
            &intent.thread,
            &intent.turn_id,
        )
        .await?;
    if let Some(row) = &row {
        anyhow::ensure!(intent.matches(row), "native_chat_turn_conflict");
    }
    Ok(row)
}

/// Read original caller intent before resolving mutable host settings/catalogs.
/// Absence grants no dispatch; creation still follows ordinary input validation.
pub async fn find_existing_native_chat_turn(
    db: &dyn PersistenceLayer,
    user: &UserContext,
    session_id: &str,
    client_turn_id: &str,
    user_message_id: &str,
    workspace_id: Option<String>,
    semantic_payload: &Value,
) -> anyhow::Result<Option<AguiRunBinding>> {
    let intent = NativeChatIntent::new(
        user,
        session_id,
        client_turn_id,
        user_message_id,
        workspace_id,
        semantic_payload,
    )?;
    load_native_chat_intent(db, &intent).await
}

/// Persist an owner-qualified reservation BEFORE effectful dispatch. The semantic
/// payload binds original client-requested content/configuration and message ID.
/// It excludes dynamic host resolutions, secrets and generated run/wire IDs, so
/// identical intent always recovers the original admission after host changes.
pub async fn reserve_native_chat_turn(
    db: &dyn PersistenceLayer,
    user: &UserContext,
    session_id: &str,
    client_turn_id: &str,
    user_message_id: &str,
    workspace_id: Option<String>,
    semantic_payload: &Value,
) -> anyhow::Result<NativeChatReservation> {
    let intent = NativeChatIntent::new(
        user,
        session_id,
        client_turn_id,
        user_message_id,
        workspace_id,
        semantic_payload,
    )?;
    if let Some(row) = load_native_chat_intent(db, &intent).await? {
        return Ok(NativeChatReservation::Existing(row));
    }
    let row = AguiRunBinding {
        native_agent_id: None,
        owner: intent.owner.clone(),
        agent_id: NATIVE_CHAT_AGENT.into(),
        thread_id: intent.thread.clone(),
        wire_run_id: client_turn_id.into(),
        native_run_id: uuid::Uuid::new_v4().to_string(),
        session_id: session_id.into(),
        client_message_id: Some(user_message_id.into()),
        action_interaction_id: None,
        workspace_id: intent.workspace_id.clone(),
        input_digest: intent.digest.clone(),
        ordinal: 1,
        revision: 1,
        start_cursor: 0,
        previous_wire_run_id: None,
        resumed_tool_ids: Vec::new(),
        terminal: None,
    };
    if db.create_agui_run(&row, None).await? {
        return Ok(NativeChatReservation::New(row));
    }
    // CAS loser reads the winner. No fresh run is dispatched on missing/failed read.
    let existing = load_native_chat_intent(db, &intent)
        .await?
        .ok_or_else(|| anyhow::anyhow!("native_chat_admission_unconfirmed"))?;
    Ok(NativeChatReservation::Existing(existing))
}

#[derive(Deserialize)]
pub(super) struct LookupQuery {
    session_id: String,
}

/// GET `/api/uar/chat-turns/{client_turn_id}?session_id=...`.
/// No dispatch, authority construction, or automatic retry occurs here.
pub(super) async fn lookup(
    State(state): State<Arc<RunApiState>>,
    Extension(user): Extension<UserContext>,
    Path(turn): Path<String>,
    Query(query): Query<LookupQuery>,
) -> Response {
    let Ok(principal) = CredentialPrincipal::from_verified_context(&user) else {
        return (StatusCode::UNAUTHORIZED, Json(json!({"recovery":"unavailable","code":"verified_principal_required","retry_allowed":false}))).into_response();
    };
    if !valid_id(&turn) || !valid_id(&query.session_id) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"code":"invalid_native_chat_identity"})),
        )
            .into_response();
    }
    let Some(db) = state.manager.persistence.as_ref().filter(|db| {
        db.supports_durable_agui_runs() && db.durable_store_eligibility().permits_durable_work()
    }) else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"recovery":"unavailable","retry_allowed":false})),
        )
            .into_response();
    };
    let row = match db.load_agui_run(&principal.storage_key(), NATIVE_CHAT_AGENT, &thread_key(&query.session_id, &turn), &turn).await {
        Ok(Some(row)) => row,
        Ok(None) => return Json(json!({"client_turn_id":turn,"session_id":query.session_id,"state":"unknown","retry_allowed":false})).into_response(),
        Err(_) => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"recovery":"unavailable","retry_allowed":false}))).into_response(),
    };
    if row.owner != principal.storage_key()
        || row.agent_id != NATIVE_CHAT_AGENT
        || row.thread_id != thread_key(&query.session_id, &turn)
        || row.wire_run_id != turn
        || row.session_id != query.session_id
    {
        return (
            StatusCode::CONFLICT,
            Json(json!({"code":"native_chat_turn_conflict","retry_allowed":false})),
        )
            .into_response();
    }
    native_chat_receipt(&state.manager, &user, &row).await
}

/// Safe original-run receipt shared with native chat's identical-submit response.
pub async fn native_chat_receipt(
    manager: &crate::uar::runtime::manager::RunManager,
    user: &UserContext,
    row: &AguiRunBinding,
) -> Response {
    let Ok(principal) = CredentialPrincipal::from_verified_context(user) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"recovery":"unavailable","retry_allowed":false})),
        )
            .into_response();
    };
    if row.owner != principal.storage_key() || row.agent_id != NATIVE_CHAT_AGENT {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"recovery":"unavailable","retry_allowed":false})),
        )
            .into_response();
    }
    let run = manager
        .get_run_for_context(user, &row.native_run_id)
        .await
        .filter(|run| run.conversation_id.as_deref() == Some(row.session_id.as_str()));
    let resident = manager
        .has_resident_run_for_context(user, &row.native_run_id)
        .await;
    let state = match run.as_ref().map(|run| &run.status) {
        Some(crate::uar::domain::runs::RunStatus::Done) => "complete",
        Some(crate::uar::domain::runs::RunStatus::Error) => "error",
        Some(crate::uar::domain::runs::RunStatus::Cancelled) => "cancelled",
        Some(_) if !resident => "interrupted",
        Some(crate::uar::domain::runs::RunStatus::Pending) => "pending",
        Some(crate::uar::domain::runs::RunStatus::Running) => "running",
        Some(crate::uar::domain::runs::RunStatus::Paused) => "paused",
        None => "admission_unconfirmed",
    };
    Json(
        json!({"client_turn_id":row.wire_run_id,"user_message_id":row.client_message_id,
        "session_id":row.session_id,"run_id":row.native_run_id,"state":state,
        "recovery":"available","retry_allowed":false}),
    )
    .into_response()
}

/// Add public host evidence without exposing run context, tool arguments, or
/// approval continuation material. Canonical model messages remain untouched.
pub(super) async fn session_projection(
    state: &crate::AppState,
    user: &UserContext,
    session: &crate::session::Session,
) -> anyhow::Result<Value> {
    use crate::session::SessionMessageStatus;
    use crate::uar::domain::{events::NormalizedEvent, runs::RunStatus};
    let owner_key = user.session_owner_key()?;
    anyhow::ensure!(
        owner_key == session.owner_id(),
        "session_history_owner_mismatch"
    );
    let mut history = session.history();
    let mut seen = std::collections::BTreeSet::new();
    let run_ids = history
        .iter()
        .rev()
        .filter_map(|message| message.run_id.clone())
        .filter(|run_id| seen.insert(run_id.clone()))
        .collect::<Vec<_>>();
    let evidence_truncated = run_ids.len() > 64;
    let mut evidence = serde_json::Map::new();
    for run_id in run_ids.into_iter().take(64) {
        let Some(run) = state
            .run_manager
            .get_run_for_context(user, &run_id)
            .await
            .filter(|run| run.conversation_id.as_deref() == Some(session.id()))
        else {
            for message in history
                .iter_mut()
                .filter(|message| message.run_id.as_deref() == Some(run_id.as_str()))
            {
                message.status = SessionMessageStatus::Unknown;
                message.provenance = "unavailable";
            }
            evidence.insert(run_id, json!({"provenance":"unavailable","approvals_availability":"unavailable","policy_availability":"unavailable","usage_availability":"unavailable","sources_availability":"unavailable"}));
            continue;
        };
        let resident = state
            .run_manager
            .has_resident_run_for_context(user, &run_id)
            .await;
        let status = match run.status {
            RunStatus::Done => SessionMessageStatus::Complete,
            RunStatus::Error => SessionMessageStatus::Error,
            RunStatus::Cancelled => SessionMessageStatus::Cancelled,
            _ if !resident => SessionMessageStatus::Interrupted,
            RunStatus::Pending => SessionMessageStatus::Pending,
            RunStatus::Running | RunStatus::Paused => SessionMessageStatus::Running,
        };
        for message in history
            .iter_mut()
            .filter(|message| message.run_id.as_deref() == Some(run_id.as_str()))
        {
            message.status = status;
        }
        let policy = run
            .context
            .get("effective_run_policy")
            .cloned()
            .and_then(|policy| {
                serde_json::from_value::<crate::uar::domain::policy::EffectiveRunPolicy>(policy)
                    .ok()
            })
            .map(|policy| {
                json!({"version":policy.version,"chat_mode":policy.chat_mode,
                "model":policy.model,"memory_enabled":policy.memory_enabled,
                "prompt_caching_enabled":policy.prompt_caching_enabled,
                "context_strategy":policy.context_strategy,"tool_approval":policy.tool_approval,
                "provenance":policy.provenance})
            });
        let mut partial_text = String::new();
        let mut sources = Vec::new();
        let mut rag_citations = Vec::new();
        let mut usage = None;
        let mut event_history_available = false;
        let mut events_truncated = false;
        let mut approvals = Vec::new();
        let mut approvals_available = false;
        if let Some(db) = &state.persistence {
            // get_run_for_context above already checked the durable owner partition.
            if db.supports_durable_run_history()
                && let Ok(events) = db.load_run_history_events(&run_id).await
            {
                event_history_available = true;
                events_truncated = events.len() > 8192;
                for event in events.into_iter().take(8192) {
                    if event.run_id != run_id {
                        continue;
                    }
                    let Ok(event) = serde_json::from_value::<NormalizedEvent>(event.event) else {
                        continue;
                    };
                    match event {
                        NormalizedEvent::ChatDelta {
                            run_id: event_run,
                            text_delta,
                        } if event_run == run_id => {
                            if partial_text.len().saturating_add(text_delta.len()) <= 1024 * 1024 {
                                partial_text.push_str(&text_delta);
                            } else {
                                events_truncated = true;
                            }
                        }
                        NormalizedEvent::Citation {
                            run_id: event_run,
                            sources: found,
                        } if event_run == run_id => sources.extend(found),
                        NormalizedEvent::RagCitations {
                            run_id: event_run,
                            citations,
                        } if event_run == run_id => rag_citations.extend(citations),
                        NormalizedEvent::RunDoneWithUsage {
                            run_id: event_run,
                            input_tokens,
                            output_tokens,
                            total_tokens,
                            cost_usd_estimate,
                            model,
                            cache_read_input_tokens,
                            cache_creation_input_tokens,
                            uncached_input_tokens,
                        } if event_run == run_id => {
                            let mut reported_usage = json!({"input_tokens":input_tokens,"output_tokens":output_tokens,"total_tokens":total_tokens,"cost_usd_estimate":cost_usd_estimate,"model":model,
                                "cache_read_input_tokens":cache_read_input_tokens,"cache_creation_input_tokens":cache_creation_input_tokens});
                            if let Some(count) = uncached_input_tokens {
                                reported_usage["uncached_input_tokens"] = count.into();
                            }
                            usage = Some(reported_usage);
                        }
                        NormalizedEvent::Cancelled {
                            run_id: event_run,
                            usage: found,
                        } if event_run == run_id => {
                            usage = found.and_then(|usage| serde_json::to_value(usage).ok());
                        }
                        _ => {}
                    }
                }
            }
            if db.supports_durable_approvals()
                && let Ok(records) = db.list_approval_records(&owner_key, &run_id).await
            {
                approvals_available = true;
                for record in records
                    .into_iter()
                    .filter(|record| record.owner_key == owner_key && record.root_run_id == run_id)
                {
                    approvals.push(json!({
                        "approval_id":record.challenge_id,"tool_call_id":record.tool_call_id,
                        "tool_name":record.tool_name,"state":record.state,
                        "decision":record.decision.map(|decision| json!({"approved":decision.approved,"decided_at":decision.decided_at})),
                        "historical":true,"resolvable":false
                    }));
                }
            }
        }
        if !events_truncated
            && !partial_text.is_empty()
            && !history.iter().any(|message| {
                message.run_id.as_deref() == Some(run_id.as_str())
                    && message.role == crate::llm::MessageRole::Assistant
            })
        {
            let insert_at = history
                .iter()
                .rposition(|message| message.run_id.as_deref() == Some(run_id.as_str()))
                .map_or(history.len(), |index| index + 1);
            history.insert(
                insert_at,
                crate::session::SessionHistoryMessage {
                    message_id: format!("run-{run_id}-assistant-0"),
                    role: crate::llm::MessageRole::Assistant,
                    content: crate::llm::MessageContent::text(partial_text),
                    run_id: Some(run_id.clone()),
                    client_turn_id: history
                        .iter()
                        .find(|message| message.run_id.as_deref() == Some(run_id.as_str()))
                        .and_then(|message| message.client_turn_id.clone()),
                    created_at: None,
                    status,
                    provenance: "run_events",
                },
            );
        }
        evidence.insert(run_id, json!({
            "provenance":"available","status":status,"sources":sources,"rag_citations":rag_citations,
            "sources_availability":if events_truncated {"truncated"} else if event_history_available {"available"} else {"unavailable"},
            "usage_availability":if events_truncated {"truncated"} else if usage.is_some() {"available"} else {"unavailable"},"usage":usage,
            "policy_availability":if policy.is_some() {"available"} else {"unavailable"},"policy":policy,
            "approvals_availability":if approvals_available {"available"} else {"unavailable"},"approvals":approvals
        }));
    }
    let mut projection = serde_json::to_value(session)?;
    let object = projection
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("session_history_invalid"))?;
    object.insert("history".into(), serde_json::to_value(history)?);
    object.insert("history_runs".into(), Value::Object(evidence));
    object.insert("history_version".into(), json!(1));
    object.insert(
        "history_runs_availability".into(),
        json!(if evidence_truncated {
            "truncated"
        } else {
            "complete"
        }),
    );
    Ok(projection)
}
