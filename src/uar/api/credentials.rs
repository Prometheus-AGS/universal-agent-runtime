//! Authenticated provider credential management and trusted-host migration preview.
use std::sync::Arc;
use axum::{Json, Router, extract::{Path, State, Query, rejection::{JsonRejection, QueryRejection}}, http::{StatusCode, HeaderMap}, response::{IntoResponse, Response}, routing::{get, put, delete, post}};
use serde::{Deserialize, Serialize};
use crate::uar::security::{claims::{CredentialPrincipal, UserContext}, credentials::{CredentialEncryption, CredentialMetadata, CredentialRecord, CredentialScope, CredentialMigrationError, CredentialOwnerBinding, CredentialTenancyPolicy, CredentialPolicyError, ProviderService, migration::{MigrationPreviewQuery, CredentialPreviewError}}};

/// Independent policy remains active when encrypted storage is unavailable.
#[derive(Clone)]
pub struct CredentialApiState { pub policy: Arc<CredentialTenancyPolicy>, pub service: Option<Arc<ProviderService>> }

impl std::fmt::Debug for CredentialApiState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CredentialApiState")
            .field("service_available", &self.service.is_some())
            .finish_non_exhaustive()
    }
}

pub fn build_router() -> Router<CredentialApiState> {
    Router::new().route("/", get(list_credentials)).route("/principal", get(credential_principal))
        .route("/migrate-legacy", post(migrate_legacy)).route("/migrate-principal", post(migrate_principal))
        .route("/migration-preview", get(migration_preview))
        .route("/{provider}", get(read_credential)).route("/{provider}", put(put_credential)).route("/{provider}", delete(delete_credential))
}
#[derive(Debug, Serialize)]
struct CredentialView { provider_id: String, api_key_hint: String, created_at: chrono::DateTime<chrono::Utc>, updated_at: chrono::DateTime<chrono::Utc> }
impl From<CredentialMetadata> for CredentialView {
    fn from(m: CredentialMetadata) -> Self { Self { provider_id: m.provider_id, api_key_hint: m.api_key_hint, created_at: m.created_at, updated_at: m.updated_at } }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PutCredentialRequest { api_key: String }
fn safe_error(status: StatusCode, code: &str) -> Response { (status, Json(serde_json::json!({"error":{"code":code}}))).into_response() }
fn unauthorized() -> Response { (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Authentication required"}))).into_response() }
fn unavailable() -> Response { (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error":"Multi-tenant credentials are not enabled on this server"}))).into_response() }
fn policy_error(error: &CredentialPolicyError) -> Response {
    let (status, code) = match error {
        CredentialPolicyError::IdentityRequired => (StatusCode::FORBIDDEN, "credential_identity_required"),
        CredentialPolicyError::InstallationIdRequired => (StatusCode::SERVICE_UNAVAILABLE, "credential_installation_id_required"),
        CredentialPolicyError::MigrationRequired => (StatusCode::CONFLICT, "credential_migration_required"),
        CredentialPolicyError::SourceDenied => (StatusCode::FORBIDDEN, "credential_source_denied"),
    }; safe_error(status, code)
}
fn owner(state: &CredentialApiState, context: Option<axum::Extension<UserContext>>) -> Result<CredentialOwnerBinding, Response> {
    let context = context.ok_or_else(unauthorized)?;
    let principal = CredentialPrincipal::from_verified_context(&context.0).map_err(|_| unauthorized())?;
    state.policy.bind_principal(&principal).map_err(|error| policy_error(&error))
}
fn storage_error(error: anyhow::Error) -> Response {
    if let Some(error) = error.downcast_ref::<CredentialPolicyError>() { policy_error(error) }
    else { safe_error(StatusCode::SERVICE_UNAVAILABLE, "credential_migration_unavailable") }
}
async fn list_credentials(State(state): State<CredentialApiState>, context: Option<axum::Extension<UserContext>>) -> Response {
    let principal = match owner(&state, context) { Ok(owner) => owner, Err(error) => return error };
    let Some(service) = state.service else { return unavailable(); };
    match service.store().list_principal(&principal, CredentialScope::User, principal.subject()).await {
        Ok(rows) => Json(rows.iter().map(|row| CredentialView::from(CredentialMetadata::from(row))).collect::<Vec<_>>()).into_response(),
        Err(error) => storage_error(error),
    }
}
async fn put_credential(State(state): State<CredentialApiState>, context: Option<axum::Extension<UserContext>>, Path(provider): Path<String>, Json(request): Json<PutCredentialRequest>) -> Response {
    let principal = match owner(&state, context) { Ok(owner) => owner, Err(error) => return error };
    let Some(service) = state.service else { return unavailable(); };
    let encrypted = match service.encryption().encrypt(&request.api_key) { Ok(value) => value, Err(_) => return safe_error(StatusCode::SERVICE_UNAVAILABLE, "credential_migration_unavailable") };
    let now = chrono::Utc::now();
    let record = CredentialRecord { scope: CredentialScope::User, scope_id: principal.subject().to_owned(), provider_id: provider, api_key_encrypted: encrypted, api_key_hint: CredentialEncryption::key_hint(&request.api_key, 4), created_at: now, updated_at: now, row_revision: None };
    match service.store().put_principal(&principal, record.clone()).await {
        Ok(()) => Json(CredentialView::from(CredentialMetadata::from(&record))).into_response(), Err(error) => storage_error(error),
    }
}
async fn delete_credential(State(state): State<CredentialApiState>, context: Option<axum::Extension<UserContext>>, Path(provider): Path<String>) -> Response {
    let principal = match owner(&state, context) { Ok(owner) => owner, Err(error) => return error };
    let Some(service) = state.service else { return unavailable(); };
    match service.store().delete_principal(&principal, CredentialScope::User, principal.subject(), &provider).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(), Ok(false) => StatusCode::NOT_FOUND.into_response(), Err(error) => storage_error(error),
    }
}
async fn read_credential(State(state): State<CredentialApiState>, context: Option<axum::Extension<UserContext>>, Path(provider): Path<String>) -> Response {
    let principal = match owner(&state, context) { Ok(owner) => owner, Err(error) => return error };
    let Some(service) = state.service else { return unavailable(); };
    match service.store().get_principal(&principal, CredentialScope::User, principal.subject(), &provider).await {
        Ok(Some(row)) => Json(CredentialView::from(CredentialMetadata::from(&row))).into_response(), Ok(None) => StatusCode::NOT_FOUND.into_response(), Err(error) => storage_error(error),
    }
}
async fn credential_principal(State(state): State<CredentialApiState>, context: Option<axum::Extension<UserContext>>) -> Response {
    let principal = match owner(&state, context) { Ok(owner) => owner, Err(error) => return error };
    Json(serde_json::json!({"owner_key":principal.storage_key(),"issuer":principal.issuer(),"tenant_id":principal.tenant_id(),"subject":principal.subject()})).into_response()
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyMappingRequest { scope: CredentialScope, legacy_scope_id: String, provider_id: String, reason: String }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PrincipalMappingRequest { source_owner_key: String, scope: CredentialScope, source_scope_id: String, provider_id: String, reason: String }
fn condition(headers: &HeaderMap) -> Result<Option<String>, Response> {
    let values: Vec<_> = headers.get_all(axum::http::header::IF_MATCH).iter().collect();
    if values.len() > 1 { return Err(safe_error(StatusCode::BAD_REQUEST, "credential_migration_precondition_invalid")); }
    let value = values.first().map(|value| value.to_str()).transpose().map_err(|_| safe_error(StatusCode::BAD_REQUEST, "credential_migration_precondition_invalid"))?;
    crate::uar::security::credentials::migration::validate_if_match(value).map_err(|_| safe_error(StatusCode::BAD_REQUEST, "credential_migration_precondition_invalid"))
}
fn migration_result(result: anyhow::Result<crate::uar::security::credentials::CredentialMigrationReceipt>, principal_source: bool) -> Response {
    match result { Ok(receipt) => Json(receipt).into_response(), Err(error) => {
        let (status, code) = match error.downcast_ref::<CredentialMigrationError>() {
            Some(CredentialMigrationError::MappingDenied) => (StatusCode::FORBIDDEN,"credential_mapping_denied"),
            Some(CredentialMigrationError::SourceMissing) => (StatusCode::NOT_FOUND,if principal_source {"principal_credential_missing"} else {"legacy_credential_missing"}),
            Some(CredentialMigrationError::Conflict) => (StatusCode::CONFLICT,"credential_mapping_conflict"),
            Some(CredentialMigrationError::PreconditionFailed) => (StatusCode::PRECONDITION_FAILED,"credential_migration_precondition_failed"),
            Some(CredentialMigrationError::PreconditionInvalid) => (StatusCode::BAD_REQUEST,"credential_migration_precondition_invalid"),
            None => (StatusCode::SERVICE_UNAVAILABLE,"credential_migration_unavailable"),
        }; safe_error(status,code)
    }}
}
async fn migrate_legacy(State(state): State<CredentialApiState>, context: Option<axum::Extension<UserContext>>, headers: HeaderMap, body: Result<Json<LegacyMappingRequest>, JsonRejection>) -> Response {
    let principal = match owner(&state, context) { Ok(owner) => owner, Err(error) => return error };
    let Some(service) = state.service else { return unavailable(); };
    let request = match body { Ok(Json(request)) => request, Err(_) => return StatusCode::BAD_REQUEST.into_response() };
    let condition = match condition(&headers) { Ok(value) => value, Err(error) => return error };
    migration_result(service.migrate_legacy_conditioned(&principal,request.scope,&request.legacy_scope_id,&request.provider_id,&request.reason,condition.as_deref()).await,false)
}
async fn migrate_principal(State(state): State<CredentialApiState>, context: Option<axum::Extension<UserContext>>, headers: HeaderMap, body: Result<Json<PrincipalMappingRequest>, JsonRejection>) -> Response {
    let principal = match owner(&state, context) { Ok(owner) => owner, Err(error) => return error };
    let Some(service) = state.service else { return unavailable(); };
    let request = match body { Ok(Json(request)) if request.reason == "operator_approved_local_namespace_migration" && !request.source_owner_key.trim().is_empty() && !request.source_scope_id.trim().is_empty() && !request.provider_id.trim().is_empty() && request.scope != CredentialScope::System => request, _ => return StatusCode::BAD_REQUEST.into_response() };
    let condition = match condition(&headers) { Ok(value) => value, Err(error) => return error };
    migration_result(service.migrate_principal_credential(&principal,&request.source_owner_key,request.scope,&request.source_scope_id,&request.provider_id,&request.reason,condition.as_deref()).await,true)
}
async fn migration_preview(State(state): State<CredentialApiState>, context: Option<axum::Extension<UserContext>>, query: Result<Query<MigrationPreviewQuery>, QueryRejection>) -> Response {
    let Some(context) = context else { return unauthorized(); };
    if context.verified_identity().is_none() && context.trusted_host_id().is_none() { return unauthorized(); }
    let Some(_host) = context.trusted_host_id() else { return safe_error(StatusCode::FORBIDDEN,"credential_preview_denied"); };
    let Some(service) = state.service else { return safe_error(StatusCode::SERVICE_UNAVAILABLE,"credential_migration_unavailable"); };
    let query = match query { Ok(Query(query)) => query, Err(_) => return safe_error(StatusCode::BAD_REQUEST,"credential_preview_request_invalid") };
    match service.migration_preview(&context.0,&state.policy,query).await {
        Ok(preview) => Json(preview).into_response(),
        Err(error) => match error.downcast_ref::<CredentialPreviewError>() {
            Some(CredentialPreviewError::Invalid) => safe_error(StatusCode::BAD_REQUEST,"credential_preview_request_invalid"),
            Some(CredentialPreviewError::Denied) => safe_error(StatusCode::FORBIDDEN,"credential_preview_denied"),
            Some(CredentialPreviewError::Stale) => safe_error(StatusCode::CONFLICT,"credential_preview_stale"),
            None => safe_error(StatusCode::SERVICE_UNAVAILABLE,"credential_migration_unavailable"),
        }
    }
}
