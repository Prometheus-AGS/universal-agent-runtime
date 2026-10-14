//! Authenticated MCP elicitation reads and exact-ID responses.
use std::sync::Arc;
use axum::{Extension, Json, Router, extract::{Path, State}, http::{HeaderMap, StatusCode}, response::{IntoResponse, Response}, routing::{get, post}};
use chrono::Utc;
use rmcp::model::ElicitResult;
use serde_json::json;
use crate::{config::SecurityConfig, mcp::interaction::McpInteractionChannel, uar::{
    domain::runs::RunStatus, persistence::mcp_elicitations::{ElicitationState, McpElicitationView},
    runtime::manager::RunManager, security::{authority::authorize_context, claims::{CredentialPrincipal, UserContext}}}};
use super::routes::RunApiState;

#[derive(Clone)]
struct AdmissionPolicy(SecurityConfig);

/// Mount beneath the authenticated `/api/uar` router.
pub fn build_router(security: SecurityConfig) -> Router<Arc<RunApiState>> {
    Router::new().route("/runs/{run_id}/mcp-interactions",get(list))
        .route("/runs/{run_id}/mcp-interactions/{interaction_id}",post(respond))
        .layer(Extension(AdmissionPolicy(security)))
}

/// The caller must apply current workspace authority before entering this seam.
/// Durable history is available after restart; only an original live waiter may
/// receive a response. Terminal records cannot become pending again.
pub(crate) async fn list_for_context(manager: &RunManager, user: &UserContext, run: &str,
    workspace: Option<&str>) -> anyhow::Result<Vec<McpElicitationView>> {
    let principal = CredentialPrincipal::from_verified_context(user)?;
    let Some(store) = manager.persistence.as_ref().filter(|store|store.supports_durable_mcp_elicitations()) else { return Ok(Vec::new()); };
    let live = manager.get_run_for_context(user,run).await
        .is_some_and(|run|matches!(run.status,RunStatus::Pending | RunStatus::Running | RunStatus::Paused));
    let rows = store.list_mcp_elicitations(&principal.storage_key(),run).await?;
    let mut views = Vec::new();
    for mut row in rows.into_iter().filter(|row|row.workspace_id.as_deref().is_none_or(|id|Some(id)==workspace)) {
        if row.state == ElicitationState::Pending && row.expires_at <= Utc::now() {
            let expired = row.finish(ElicitationState::Expired);
            if store.compare_and_swap_mcp_elicitation(&row,&expired).await? { row = expired; }
            else {
                row = store.list_mcp_elicitations(&principal.storage_key(),run).await?
                    .into_iter().find(|saved|saved.id==row.id).ok_or_else(||anyhow::anyhow!("elicitation_record_missing"))?;
            }
        }
        let resolvable = live && row.state == ElicitationState::Pending && row.expires_at > Utc::now()
            && McpInteractionChannel::active_for(&row);
        views.push(McpElicitationView {record:row,resolvable});
    }
    Ok(views)
}

/// Resolve one exact live interaction. ElicitResult is not an approval response.
pub(crate) async fn resolve_for_context(manager: &RunManager, user: &UserContext, run: &str,
    workspace: Option<&str>, id: &str, response: ElicitResult) -> anyhow::Result<bool> {
    let views = list_for_context(manager,user,run,workspace).await?;
    let Some(view) = views.into_iter().find(|view|view.record.id == id && view.resolvable) else { return Ok(false); };
    let next = view.record.answer(response)?;
    let store = manager.persistence.as_ref().ok_or_else(||anyhow::anyhow!("elicitation_persistence_unavailable"))?;
    store.compare_and_swap_mcp_elicitation(&view.record,&next).await
}

async fn list(State(state):State<Arc<RunApiState>>, Extension(policy):Extension<AdmissionPolicy>,
    Extension(user):Extension<UserContext>, Path(run):Path<String>, headers:HeaderMap) -> Response {
    let workspace = super::targets::workspace(&headers);
    if authorize_context(&policy.0,Some(&user),workspace).is_err() { return StatusCode::FORBIDDEN.into_response(); }
    match list_for_context(&state.manager,&user,&run,workspace).await {
        Ok(views) => Json(json!({"interactions":views})).into_response(),
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE,Json(json!({"code":"elicitation_history_unavailable"}))).into_response(),
    }
}
async fn respond(State(state):State<Arc<RunApiState>>, Extension(policy):Extension<AdmissionPolicy>,
    Extension(user):Extension<UserContext>, Path((run,id)):Path<(String,String)>, headers:HeaderMap,
    Json(response):Json<ElicitResult>) -> Response {
    let workspace = super::targets::workspace(&headers);
    if authorize_context(&policy.0,Some(&user),workspace).is_err() { return StatusCode::FORBIDDEN.into_response(); }
    // Input shape/schema errors are client errors; the subsequent CAS still
    // arbitrates a concurrent response, expiry, disconnect or cancellation.
    let views = match list_for_context(&state.manager,&user,&run,workspace).await {
        Ok(views) => views, Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let Some(view) = views.into_iter().find(|view|view.record.id==id && view.resolvable) else { return StatusCode::CONFLICT.into_response(); };
    if view.record.validate_response(&response).is_err() { return StatusCode::BAD_REQUEST.into_response(); }
    match resolve_for_context(&state.manager,&user,&run,workspace,&id,response).await {
        Ok(true) => Json(json!({"interaction_id":id,"response_committed":true,"transport_delivery":"unconfirmed"})).into_response(),
        Ok(false) => StatusCode::CONFLICT.into_response(),
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE,Json(json!({"code":"elicitation_response_unconfirmed"}))).into_response(),
    }
}
