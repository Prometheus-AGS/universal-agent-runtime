//! Settings REST API
//!
//! Exposes the runtime configuration management endpoints. All mutation endpoints
//! require `X-UAR-Admin-Key` header. Sensitive fields (schema `x-sensitive: true`)
//! are masked with `"***"` in GET responses.

use crate::uar::settings::manager::SettingsManager;
use crate::uar::settings::schema::{SettingsType, SettingsWithMeta};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;

// =============================================================================
// State
// =============================================================================

#[derive(Clone, Debug)]
pub struct SettingsApiState {
    pub settings_manager: Option<Arc<SettingsManager>>,
    /// When false, mutation handlers do not require `X-UAR-Admin-Key` (local dev only).
    pub settings_mutation_auth_required: bool,
}

// =============================================================================
// Router
// =============================================================================

pub fn build_router() -> Router<Arc<SettingsApiState>> {
    Router::new()
        // Settings types (namespace registry)
        .route("/types", get(list_types))
        .route("/types/{key}", get(get_type))
        .route("/types", post(create_type))
        // Drift listing
        .route("/drift", get(list_drift))
        // Individual settings CRUD
        .route("/", get(list_settings))
        .route("/{key}", get(get_setting))
        .route("/{key}", put(update_setting))
        .route("/{key}", delete(reset_setting))
        // Namespace bulk endpoints
        .route("/server", get(|s, q, h| list_namespace(s, q, h, "server")))
        .route(
            "/server",
            put(|s, q, h, b| update_namespace(s, q, h, b, "server")),
        )
        .route(
            "/security",
            get(|s, q, h| list_namespace(s, q, h, "security")),
        )
        .route(
            "/security",
            put(|s, q, h, b| update_namespace(s, q, h, b, "security")),
        )
        .route(
            "/resilience",
            get(|s, q, h| list_namespace(s, q, h, "resilience")),
        )
        .route(
            "/resilience",
            put(|s, q, h, b| update_namespace(s, q, h, b, "resilience")),
        )
        .route(
            "/file-processing",
            get(|s, q, h| list_namespace(s, q, h, "file_processing")),
        )
        .route(
            "/file-processing",
            put(|s, q, h, b| update_namespace(s, q, h, b, "file_processing")),
        )
        .route("/vision", get(|s, q, h| list_namespace(s, q, h, "vision")))
        .route(
            "/vision",
            put(|s, q, h, b| update_namespace(s, q, h, b, "vision")),
        )
        .route(
            "/knowledge-bases",
            get(|s, q, h| list_namespace(s, q, h, "knowledge_bases")),
        )
        .route(
            "/knowledge-bases",
            put(|s, q, h, b| update_namespace(s, q, h, b, "knowledge_bases")),
        )
        .route(
            "/intent-classifier",
            get(|s, q, h| list_namespace(s, q, h, "intent_classifier")),
        )
        .route(
            "/intent-classifier",
            put(|s, q, h, b| update_namespace(s, q, h, b, "intent_classifier")),
        )
        .route(
            "/providers",
            get(|s, q, h| list_namespace(s, q, h, "provider")),
        )
        .route(
            "/providers",
            put(|s, q, h, b| update_namespace(s, q, h, b, "provider")),
        )
        .route(
            "/unstructured",
            get(|s, q, h| list_namespace(s, q, h, "unstructured")),
        )
        .route(
            "/unstructured",
            put(|s, q, h, b| update_namespace(s, q, h, b, "unstructured")),
        )
        .route(
            "/kreuzberg",
            get(|s, q, h| list_namespace(s, q, h, "kreuzberg")),
        )
        .route(
            "/kreuzberg",
            put(|s, q, h, b| update_namespace(s, q, h, b, "kreuzberg")),
        )
        // New namespaces
        .route(
            "/context-management",
            get(|s, q, h| list_namespace(s, q, h, "context_management")),
        )
        .route(
            "/context-management",
            put(|s, q, h, b| update_namespace(s, q, h, b, "context_management")),
        )
        .route("/rag", get(|s, q, h| list_namespace(s, q, h, "rag")))
        .route(
            "/rag",
            put(|s, q, h, b| update_namespace(s, q, h, b, "rag")),
        )
        .route(
            "/governance",
            get(|s, q, h| list_namespace(s, q, h, "governance")),
        )
        .route(
            "/governance",
            put(|s, q, h, b| update_namespace(s, q, h, b, "governance")),
        )
        .route(
            "/agent-config",
            get(|s, q, h| list_namespace(s, q, h, "agent_config")),
        )
        .route(
            "/agent-config",
            put(|s, q, h, b| update_namespace(s, q, h, b, "agent_config")),
        )
        .route(
            "/skill-config",
            get(|s, q, h| list_namespace(s, q, h, "skill_config")),
        )
        .route(
            "/skill-config",
            put(|s, q, h, b| update_namespace(s, q, h, b, "skill_config")),
        )
        .route(
            "/mistral-ocr",
            get(|s, q, h| list_namespace(s, q, h, "mistral_ocr")),
        )
        .route(
            "/mistral-ocr",
            put(|s, q, h, b| update_namespace(s, q, h, b, "mistral_ocr")),
        )
        .route("/memory", get(|s, q, h| list_namespace(s, q, h, "memory")))
        .route(
            "/memory",
            put(|s, q, h, b| update_namespace(s, q, h, b, "memory")),
        )
        .route(
            "/llm-failover",
            get(|s, q, h| list_namespace(s, q, h, "llm_failover")),
        )
        .route(
            "/llm-failover",
            put(|s, q, h, b| update_namespace(s, q, h, b, "llm_failover")),
        )
        .route(
            "/native-tools",
            get(|s, q, h| list_namespace(s, q, h, "native_tools")),
        )
        .route(
            "/native-tools",
            put(|s, q, h, b| update_namespace(s, q, h, b, "native_tools")),
        )
        .route(
            "/skill-evolution",
            get(|s, q, h| list_namespace(s, q, h, "skill_evolution")),
        )
        .route(
            "/skill-evolution",
            put(|s, q, h, b| update_namespace(s, q, h, b, "skill_evolution")),
        )
        .route("/acp", get(|s, q, h| list_namespace(s, q, h, "acp")))
        .route(
            "/acp",
            put(|s, q, h, b| update_namespace(s, q, h, b, "acp")),
        )
}

// =============================================================================
// Helpers
// =============================================================================

fn mgr_from_state(state: &SettingsApiState) -> Result<&Arc<SettingsManager>, ApiError> {
    state.settings_manager.as_ref().ok_or(ApiError::Unavailable)
}

enum ApiError {
    Unavailable,
    NotFound(String),
    Forbidden,
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::Unavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({"error": "Settings manager not available — no persistence layer configured"})),
            ).into_response(),
            ApiError::NotFound(key) => (
                StatusCode::NOT_FOUND,
                Json(json!({"error": format!("Setting '{key}' not found")})),
            ).into_response(),
            ApiError::Forbidden => (
                StatusCode::FORBIDDEN,
                Json(json!({"error": "X-UAR-Admin-Key header is required for mutation endpoints (set security.settings_mutation_auth_required: false in config for trusted local use)"})),
            ).into_response(),
            ApiError::Internal(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e})),
            ).into_response(),
        }
    }
}

/// Validate admin key header for mutation endpoints (skipped when
/// `settings_mutation_auth_required` is false).
fn require_admin_key(state: &SettingsApiState, headers: &HeaderMap) -> Result<(), ApiError> {
    if !state.settings_mutation_auth_required {
        return Ok(());
    }
    if headers.contains_key("x-uar-admin-key") {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

/// Mask sensitive fields in a settings data value.
///
/// Traverses the JSON data and replaces values whose key is marked `x-sensitive: true`
/// in the schema `properties` with `"***"`.
fn mask_sensitive(data: Value, schema: &Value) -> Value {
    let props = schema
        .get("properties")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    match data {
        Value::Object(mut map) => {
            for (key, schema_prop) in &props {
                if schema_prop
                    .get("x-sensitive")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                {
                    map.insert(key.clone(), Value::String("***".to_string()));
                }
            }
            Value::Object(map)
        }
        other => other,
    }
}

/// Apply sensitive masking to a SettingsWithMeta response.
async fn masked_response(swm: SettingsWithMeta, mgr: &SettingsManager) -> SettingsWithMetaResponse {
    // Try to find the schema for this setting type.
    let type_key = swm.setting.key.split('.').next().unwrap_or("");
    let schema = mgr
        .get_type(type_key)
        .await
        .ok()
        .flatten()
        .map(|t| t.schema)
        .unwrap_or(json!({}));

    let masked_data = mask_sensitive(swm.setting.data.clone(), &schema);
    SettingsWithMetaResponse {
        id: swm.setting.id,
        settings_type_id: swm.setting.settings_type_id,
        name: swm.setting.name,
        key: swm.setting.key,
        data: masked_data,
        parent_id: swm.setting.parent_id,
        created_at: swm.setting.created_at,
        updated_at: swm.setting.updated_at,
        source: format!("{:?}", swm.meta.source),
        is_drift: swm.meta.is_drift,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsWithMetaResponse {
    pub id: uuid::Uuid,
    pub settings_type_id: uuid::Uuid,
    pub name: String,
    pub key: String,
    pub data: Value,
    pub parent_id: Option<uuid::Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub source: String,
    pub is_drift: bool,
}

// =============================================================================
// Query params
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct ListSettingsQuery {
    #[serde(default)]
    pub type_key: Option<String>,
    pub parent_id: Option<uuid::Uuid>,
}

// =============================================================================
// Handlers: Settings Types
// =============================================================================

async fn list_types(
    State(state): State<Arc<SettingsApiState>>,
    _query: Query<ListSettingsQuery>,
    _headers: HeaderMap,
) -> Result<Json<Vec<SettingsType>>, ApiError> {
    let mgr = mgr_from_state(&state)?;
    let types = mgr
        .list_types()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(types))
}

async fn get_type(
    State(state): State<Arc<SettingsApiState>>,
    Path(key): Path<String>,
) -> Result<Json<SettingsType>, ApiError> {
    let mgr = mgr_from_state(&state)?;
    let t = mgr
        .get_type(&key)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound(key))?;
    Ok(Json(t))
}

#[derive(Debug, Deserialize)]
pub struct CreateTypePayload {
    pub name: String,
    pub key: String,
    pub schema: Value,
}

async fn create_type(
    State(state): State<Arc<SettingsApiState>>,
    headers: HeaderMap,
    Json(payload): Json<CreateTypePayload>,
) -> Result<Json<Value>, ApiError> {
    require_admin_key(state.as_ref(), &headers)?;
    let mgr = mgr_from_state(&state)?;
    let st = SettingsType {
        id: uuid::Uuid::new_v4(),
        name: payload.name,
        key: payload.key,
        schema: payload.schema,
        created_at: chrono::Utc::now(),
        updated_at: None,
    };
    mgr.register_extension(st.clone(), vec![])
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(json!({"status": "created", "key": st.key})))
}

// =============================================================================
// Handlers: Settings CRUD
// =============================================================================

async fn list_settings(
    State(state): State<Arc<SettingsApiState>>,
    _query: Query<ListSettingsQuery>,
    _headers: HeaderMap,
) -> Result<Json<Vec<SettingsWithMetaResponse>>, ApiError> {
    let mgr = mgr_from_state(&state)?;
    let all = mgr.list_all_with_meta().await;
    let mut out = Vec::with_capacity(all.len());
    for swm in all {
        out.push(masked_response(swm, mgr).await);
    }
    Ok(Json(out))
}

async fn get_setting(
    State(state): State<Arc<SettingsApiState>>,
    Path(key): Path<String>,
) -> Result<Json<SettingsWithMetaResponse>, ApiError> {
    let mgr = mgr_from_state(&state)?;
    let swm = mgr
        .get_with_meta(&key)
        .await
        .ok_or_else(|| ApiError::NotFound(key))?;
    Ok(Json(masked_response(swm, mgr).await))
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingPayload {
    pub value: Value,
}

async fn update_setting(
    State(state): State<Arc<SettingsApiState>>,
    Path(key): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<UpdateSettingPayload>,
) -> Result<Json<Value>, ApiError> {
    require_admin_key(state.as_ref(), &headers)?;
    let mgr = mgr_from_state(&state)?;
    mgr.set_value(&key, payload.value)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(json!({"status": "updated", "key": key})))
}

async fn reset_setting(
    State(state): State<Arc<SettingsApiState>>,
    Path(key): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    require_admin_key(state.as_ref(), &headers)?;
    let mgr = mgr_from_state(&state)?;
    mgr.reset_to_default(&key)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(json!({"status": "reset", "key": key})))
}

// =============================================================================
// Handlers: Drift
// =============================================================================

async fn list_drift(
    State(state): State<Arc<SettingsApiState>>,
) -> Result<Json<Vec<SettingsWithMetaResponse>>, ApiError> {
    let mgr = mgr_from_state(&state)?;
    let drift = mgr.list_drift().await;
    let mut out = Vec::with_capacity(drift.len());
    for swm in drift {
        out.push(masked_response(swm, mgr).await);
    }
    Ok(Json(out))
}

// =============================================================================
// Handlers: Namespace bulk endpoints
// =============================================================================

async fn list_namespace(
    State(state): State<Arc<SettingsApiState>>,
    _query: Query<ListSettingsQuery>,
    _headers: HeaderMap,
    type_key: &str,
) -> Result<Json<Vec<SettingsWithMetaResponse>>, ApiError> {
    let mgr = mgr_from_state(&state)?;
    let items = mgr.list_namespace_with_meta(type_key).await;
    let mut out = Vec::with_capacity(items.len());
    for swm in items {
        out.push(masked_response(swm, mgr).await);
    }
    Ok(Json(out))
}

#[derive(Debug, Deserialize)]
pub struct BulkUpdatePayload {
    pub data: std::collections::HashMap<String, Value>,
}

async fn update_namespace(
    State(state): State<Arc<SettingsApiState>>,
    _query: Query<ListSettingsQuery>,
    headers: HeaderMap,
    Json(payload): Json<BulkUpdatePayload>,
    type_key: &str,
) -> Result<Json<Value>, ApiError> {
    require_admin_key(state.as_ref(), &headers)?;
    let mgr = mgr_from_state(&state)?;

    let mut updated = Vec::new();
    let mut errors = Vec::new();

    for (field, value) in payload.data {
        let key = format!("{type_key}.{field}");
        match mgr.set_value(&key, value).await {
            Ok(()) => updated.push(key),
            Err(e) => errors.push(json!({"key": key, "error": e.to_string()})),
        }
    }

    if !errors.is_empty() {
        return Ok(Json(json!({
            "status": "partial",
            "updated": updated,
            "errors": errors
        })));
    }

    Ok(Json(json!({"status": "updated", "keys": updated})))
}
