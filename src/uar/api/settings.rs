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
            "/persistence",
            get(|s, q, h| list_namespace(s, q, h, "persistence")),
        )
        .route(
            "/persistence",
            put(|s, q, h, b| update_namespace(s, q, h, b, "persistence")),
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
        .route("/models", get(|s, q, h| list_namespace(s, q, h, "models")))
        .route(
            "/models",
            put(|s, q, h, b| update_namespace(s, q, h, b, "models")),
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
        .route("/llm", get(|s, q, h| list_namespace(s, q, h, "llm")))
        .route(
            "/llm",
            put(|s, q, h, b| update_namespace(s, q, h, b, "llm")),
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
        .route(
            "/context-strategy",
            get(|s, q, h| list_namespace(s, q, h, "context_strategy")),
        )
        .route(
            "/context-strategy",
            put(|s, q, h, b| update_namespace(s, q, h, b, "context_strategy")),
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
            "/failover",
            get(|s, q, h| list_namespace(s, q, h, "llm_failover")),
        )
        .route(
            "/failover",
            put(|s, q, h, b| update_namespace(s, q, h, b, "llm_failover")),
        )
        .route(
            "/sandbox",
            get(|s, q, h| list_namespace(s, q, h, "sandbox")),
        )
        .route(
            "/sandbox",
            put(|s, q, h, b| update_namespace(s, q, h, b, "sandbox")),
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
        .route(
            "/sycophancy",
            get(|s, q, h| list_namespace(s, q, h, "sycophancy")),
        )
        .route(
            "/sycophancy",
            put(|s, q, h, b| update_namespace(s, q, h, b, "sycophancy")),
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

fn schema_is_sensitive(schema: &Value) -> bool {
    schema
        .get("x-sensitive")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn schema_contains_sensitive(schema: &Value) -> bool {
    schema_is_sensitive(schema)
        || schema
            .get("properties")
            .and_then(Value::as_object)
            .is_some_and(|props| props.values().any(schema_contains_sensitive))
        || schema.get("items").is_some_and(schema_contains_sensitive)
}

fn value_contains_sensitive_placeholder(data: &Value, schema: &Value) -> bool {
    if schema_is_sensitive(schema) {
        return data
            .as_str()
            .is_some_and(|value| !value.is_empty() && value.chars().all(|c| c == '*'));
    }

    match data {
        Value::Object(map) => schema
            .get("properties")
            .and_then(Value::as_object)
            .is_some_and(|properties| {
                map.iter().any(|(key, value)| {
                    properties.get(key).is_some_and(|property| {
                        value_contains_sensitive_placeholder(value, property)
                    })
                })
            }),
        Value::Array(values) => schema.get("items").is_some_and(|item_schema| {
            values
                .iter()
                .any(|value| value_contains_sensitive_placeholder(value, item_schema))
        }),
        _ => false,
    }
}

fn mask_string(value: &str) -> String {
    "*".repeat(value.chars().count())
}

/// Mask sensitive fields in a settings data value without fabricating absent fields.
fn mask_sensitive(data: Value, schema: &Value) -> Value {
    if schema_is_sensitive(schema) {
        return match data {
            Value::String(value) => Value::String(mask_string(&value)),
            Value::Null => Value::Null,
            _ => Value::String("***".to_string()),
        };
    }

    let properties = schema.get("properties").and_then(Value::as_object).cloned();

    match data {
        Value::Object(mut map) => {
            if let Some(properties) = properties {
                for (key, value) in &mut map {
                    if let Some(property_schema) = properties.get(key) {
                        *value = mask_sensitive(std::mem::take(value), property_schema);
                    }
                }
            }
            Value::Object(map)
        }
        Value::Array(mut values) => {
            if let Some(item_schema) = schema.get("items") {
                for value in &mut values {
                    *value = mask_sensitive(std::mem::take(value), item_schema);
                }
            }
            Value::Array(values)
        }
        other => other,
    }
}

fn mask_setting_data(data: Value, schema: &Value, field_key: Option<&str>) -> Value {
    let value_schema = field_key
        .and_then(|field| {
            schema
                .get("properties")
                .and_then(Value::as_object)
                .and_then(|props| props.get(field))
        })
        .unwrap_or(schema);
    mask_sensitive(data, value_schema)
}

/// Apply sensitive masking to a SettingsWithMeta response.
async fn masked_response(swm: SettingsWithMeta, mgr: &SettingsManager) -> SettingsWithMetaResponse {
    // Try to find the schema for this setting type.
    let type_key = swm.setting.key.split('.').next().unwrap_or("");
    let field_key = swm.setting.key.split_once('.').map(|(_, field)| field);
    let schema = mgr
        .get_type(type_key)
        .await
        .ok()
        .flatten()
        .map(|t| t.schema)
        .unwrap_or(json!({}));

    let masked_data = mask_setting_data(swm.setting.data.clone(), &schema, field_key);
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
    let namespace = key.split('.').next().unwrap_or("").to_string();
    let value = preserve_masked_sensitive_value(mgr, &key, payload.value).await?;
    mgr.set_value(&key, value)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let updated = mgr
        .get_with_meta(&key)
        .await
        .ok_or_else(|| ApiError::NotFound(key.clone()))?;
    Ok(Json(json!({
        "status": "updated",
        "key": key,
        "namespace": namespace,
        "setting": masked_response(updated, mgr).await
    })))
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
        let value = preserve_masked_sensitive_value(mgr, &key, value).await?;
        match mgr.set_value(&key, value).await {
            Ok(()) => {
                if let Some(row) = mgr.get_with_meta(&key).await {
                    updated.push(masked_response(row, mgr).await);
                }
            }
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

    Ok(Json(json!({"status": "updated", "updated": updated})))
}

async fn preserve_masked_sensitive_value(
    mgr: &SettingsManager,
    key: &str,
    value: Value,
) -> Result<Value, ApiError> {
    let Some((type_key, field)) = key.split_once('.') else {
        return Ok(value);
    };
    let Some(settings_type) = mgr
        .get_type(type_key)
        .await
        .map_err(|error| ApiError::Internal(error.to_string()))?
    else {
        return Ok(value);
    };
    let value_schema = settings_type
        .schema
        .get("properties")
        .and_then(Value::as_object)
        .and_then(|props| props.get(field))
        .unwrap_or(&settings_type.schema);

    if !schema_contains_sensitive(value_schema) {
        return Ok(value);
    }

    let Some(existing) = mgr.get_with_meta(key).await else {
        if value_contains_sensitive_placeholder(&value, value_schema) {
            return Err(ApiError::NotFound(key.to_string()));
        }
        return Ok(value);
    };
    Ok(preserve_sensitive_value(
        value,
        &existing.setting.data,
        value_schema,
    ))
}

fn preserve_sensitive_value(value: Value, existing: &Value, schema: &Value) -> Value {
    if schema_is_sensitive(schema) {
        return match &value {
            Value::String(submitted)
                if submitted.is_empty()
                    || (!submitted.is_empty() && submitted.chars().all(|c| c == '*')) =>
            {
                existing.clone()
            }
            _ => value,
        };
    }

    match value {
        Value::Object(mut submitted) => {
            if let (Some(properties), Some(current)) = (
                schema.get("properties").and_then(Value::as_object),
                existing.as_object(),
            ) {
                for (key, submitted_value) in &mut submitted {
                    if let (Some(property_schema), Some(current_value)) =
                        (properties.get(key), current.get(key))
                    {
                        *submitted_value = preserve_sensitive_value(
                            std::mem::take(submitted_value),
                            current_value,
                            property_schema,
                        );
                    }
                }
            }
            Value::Object(submitted)
        }
        Value::Array(mut submitted) => {
            if let (Some(item_schema), Some(current)) = (schema.get("items"), existing.as_array()) {
                for (index, submitted_value) in submitted.iter_mut().enumerate() {
                    if let Some(current_value) = current.get(index) {
                        *submitted_value = preserve_sensitive_value(
                            std::mem::take(submitted_value),
                            current_value,
                            item_schema,
                        );
                    }
                }
            }
            Value::Array(submitted)
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        mask_sensitive, mask_setting_data, preserve_sensitive_value,
        value_contains_sensitive_placeholder,
    };
    use serde_json::json;

    #[test]
    fn settings_api_masks_and_preserves_sensitive_values() {
        let schema = json!({
            "type": "object",
            "properties": {
                "api_key": { "type": "string", "x-sensitive": true },
                "base_url": { "type": "string" }
            }
        });

        assert_eq!(
            mask_setting_data(json!("éabc"), &schema, Some("api_key")),
            json!("****")
        );
        assert_eq!(
            mask_setting_data(json!("https://example.test"), &schema, Some("base_url")),
            json!("https://example.test")
        );
        assert_eq!(
            mask_setting_data(
                json!({"api_key": "secret", "base_url": "https://example.test"}),
                &schema,
                None,
            ),
            json!({"api_key": "******", "base_url": "https://example.test"})
        );
        assert_eq!(
            mask_setting_data(json!({"base_url": "https://example.test"}), &schema, None),
            json!({"base_url": "https://example.test"})
        );
        assert_eq!(
            mask_setting_data(json!({"api_key": ""}), &schema, None),
            json!({"api_key": ""})
        );
        assert_eq!(
            mask_sensitive(
                json!({"credential": "secret"}),
                &json!({"x-sensitive": true}),
            ),
            json!("***")
        );
        assert_eq!(
            mask_sensitive(json!(null), &json!({"x-sensitive": true})),
            json!(null)
        );

        let current = json!({"api_key": "secret", "protocol": "chat"});
        assert!(!value_contains_sensitive_placeholder(
            &json!({"api_key": "sk-real"}),
            &schema,
        ));
        assert!(value_contains_sensitive_placeholder(
            &json!({"api_key": "******"}),
            &schema,
        ));
        assert_eq!(
            preserve_sensitive_value(
                json!({"api_key": "******", "protocol": "responses"}),
                &current,
                &schema,
            ),
            json!({"api_key": "secret", "protocol": "responses"})
        );
        assert_eq!(
            preserve_sensitive_value(
                json!({"api_key": "***", "protocol": "responses"}),
                &current,
                &schema,
            ),
            json!({"api_key": "secret", "protocol": "responses"})
        );
        assert_eq!(
            preserve_sensitive_value(
                json!("***"),
                &json!({"credential": "secret"}),
                &json!({"x-sensitive": true}),
            ),
            json!({"credential": "secret"})
        );
        assert_eq!(
            preserve_sensitive_value(
                json!({"api_key": "replacement", "protocol": "chat"}),
                &current,
                &schema,
            ),
            json!({"api_key": "replacement", "protocol": "chat"})
        );
    }
}
