//! Owner-scoped accounting inspection and explicitly trusted reconciliation.
//! Provider evidence is an operator attestation; a timeout never settles a call.
use axum::{Json, Router, extract::{Extension, Path, Query, State}, http::StatusCode, response::{IntoResponse, Response}, routing::{get, post}};
use serde::Deserialize;
use serde_json::json;

use crate::AppState;
use crate::uar::persistence::accounting::{AccountingError, CallIdentity, CallUsage, ScopeIdentity, ScopeLimits, ScopeUpdate};
use crate::uar::security::claims::{CredentialPrincipal, UserContext};

#[derive(Default, Deserialize)]
struct OwnerQuery { owner: Option<String> }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScopeChange {
    expected_generation: u64,
    limits: ScopeLimits,
    opening_cost_usd: Option<f64>,
    opening_tokens: Option<u64>,
    evidence: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReconcileCall {
    cost_usd: f64,
    tokens: u64,
    confirmed_provider_usage: bool,
    evidence: String,
}

pub fn build_router() -> Router<AppState> {
    Router::new()
        .route("/scopes/{scope}/{scope_id}", get(scope).put(update_scope))
        .route("/calls/{run_id}", get(calls))
        .route("/calls/{run_id}/{call_id}/{attempt}/reconcile", post(reconcile_call))
}

fn error(status: StatusCode, code: &str) -> Response {
    (status, Json(json!({"error": code}))).into_response()
}

fn host(user: &UserContext) -> Result<(), Response> {
    if user.trusted_host_id().is_none() {
        return Err(error(StatusCode::FORBIDDEN, "trusted_host_accounting_authority_required"));
    }
    Ok(())
}

fn owner(user: &UserContext, query: &OwnerQuery) -> Result<String, Response> {
    let principal = CredentialPrincipal::from_verified_context(user)
        .map_err(|_| error(StatusCode::UNAUTHORIZED, "verified_accounting_owner_required"))?;
    match query.owner.as_deref() {
        Some(value) if value != principal.storage_key() => {
            host(user)?;
            if value.is_empty() { return Err(error(StatusCode::BAD_REQUEST, "accounting_owner_required")); }
            Ok(value.to_owned())
        }
        _ => Ok(principal.storage_key()),
    }
}

fn identity(user: &UserContext, query: &OwnerQuery, scope: String, scope_id: String) -> Result<ScopeIdentity, Response> {
    if !matches!(scope.as_str(), "run" | "task" | "session" | "agent" | "global") {
        return Err(error(StatusCode::BAD_REQUEST, "invalid_accounting_scope"));
    }
    let owner = if scope == "global" {
        host(user)?;
        if scope_id != "global" || query.owner.as_deref().is_some_and(|value| value != "deployment") {
            return Err(error(StatusCode::BAD_REQUEST, "invalid_deployment_accounting_scope"));
        }
        "deployment".to_owned()
    } else { owner(user, query)? };
    Ok(ScopeIdentity { owner, scope, scope_id })
}

fn failure(cause: anyhow::Error) -> Response {
    let (status, code) = match cause.downcast_ref::<AccountingError>() {
        Some(AccountingError::Missing) => (StatusCode::NOT_FOUND, "accounting_record_missing"),
        Some(AccountingError::Conflict | AccountingError::IdentityConflict) => (StatusCode::CONFLICT, "accounting_revision_or_identity_conflict"),
        Some(AccountingError::ReconciliationRequired | AccountingError::Unresolved) => (StatusCode::CONFLICT, "accounting_reconciliation_required"),
        Some(AccountingError::Exhausted) => (StatusCode::CONFLICT, "accounting_budget_exhausted"),
        _ => (StatusCode::SERVICE_UNAVAILABLE, "accounting_storage_unavailable"),
    };
    error(status, code)
}

async fn scope(State(state): State<AppState>, Extension(user): Extension<UserContext>,
    Path((scope, scope_id)): Path<(String, String)>, Query(query): Query<OwnerQuery>) -> Response {
    let identity = match identity(&user, &query, scope, scope_id) { Ok(value) => value, Err(response) => return response };
    let Some(storage) = state.persistence.as_ref() else { return error(StatusCode::SERVICE_UNAVAILABLE, "durable_accounting_required"); };
    match storage.get_accounting_scope(&identity).await {
        Ok(Some(value)) => Json(value).into_response(),
        Ok(None) => error(StatusCode::NOT_FOUND, "accounting_record_missing"),
        Err(cause) => failure(cause),
    }
}

async fn calls(State(state): State<AppState>, Extension(user): Extension<UserContext>,
    Path(run_id): Path<String>, Query(query): Query<OwnerQuery>) -> Response {
    let owner = match owner(&user, &query) { Ok(value) => value, Err(response) => return response };
    let Some(storage) = state.persistence.as_ref() else { return error(StatusCode::SERVICE_UNAVAILABLE, "durable_accounting_required"); };
    match storage.list_accounting_calls(&owner, &run_id).await {
        Ok(value) => Json(value).into_response(),
        Err(cause) => failure(cause),
    }
}

async fn update_scope(State(state): State<AppState>, Extension(user): Extension<UserContext>,
    Path((scope, scope_id)): Path<(String, String)>, Query(query): Query<OwnerQuery>, Json(change): Json<ScopeChange>) -> Response {
    if let Err(response) = host(&user) { return response; }
    let identity = match identity(&user, &query, scope, scope_id) { Ok(value) => value, Err(response) => return response };
    if change.evidence.trim().is_empty() || change.limits.validate().is_err()
        || change.opening_cost_usd.is_some_and(|cost| !cost.is_finite() || cost < 0.0)
        || change.opening_cost_usd.is_some() != change.opening_tokens.is_some()
    { return error(StatusCode::BAD_REQUEST, "invalid_accounting_reconciliation"); }
    let Some(storage) = state.persistence.as_ref() else { return error(StatusCode::SERVICE_UNAVAILABLE, "durable_accounting_required"); };
    let change = ScopeUpdate { identity, expected_generation: change.expected_generation,
        limits: change.limits, opening_cost_usd: change.opening_cost_usd,
        opening_tokens: change.opening_tokens, evidence: change.evidence };
    match storage.update_accounting_scope(&change).await {
        Ok(value) => Json(value).into_response(),
        Err(cause) => failure(cause),
    }
}

async fn reconcile_call(State(state): State<AppState>, Extension(user): Extension<UserContext>,
    Path((run_id, call_id, attempt)): Path<(String, String, u64)>, Query(query): Query<OwnerQuery>,
    Json(change): Json<ReconcileCall>) -> Response {
    if let Err(response) = host(&user) { return response; }
    let owner = match owner(&user, &query) { Ok(value) => value, Err(response) => return response };
    if !change.confirmed_provider_usage || change.evidence.trim().is_empty()
        || !change.cost_usd.is_finite() || change.cost_usd < 0.0
    { return error(StatusCode::BAD_REQUEST, "confirmed_provider_usage_evidence_required"); }
    let Some(storage) = state.persistence.as_ref() else { return error(StatusCode::SERVICE_UNAVAILABLE, "durable_accounting_required"); };
    let usage = CallUsage { identity: CallIdentity { owner, run_id, call_id, attempt },
        cost_usd: Some(change.cost_usd), tokens: change.tokens, terminal: true,
        evidence: Some(change.evidence) };
    match storage.record_accounting_usage(&usage).await {
        Ok(value) => Json(value).into_response(),
        Err(cause) => failure(cause),
    }
}
