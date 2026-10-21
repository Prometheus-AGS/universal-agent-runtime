//! Standalone tool-definition CRUD (R11, operator ruling 2026-10-20).
//!
//! A *stored tool definition* is a catalog/configuration record: metadata,
//! JSON schemas, and a safe executor binding. It is deliberately NOT an
//! execution path:
//!
//! - `transport: "http"` is REJECTED for stored definitions (arbitrary egress
//!   is refused; a stored record must never become arbitrary shell/HTTP
//!   execution).
//! - `transport: "mcp"` must reference a currently registered MCP server;
//!   execution still flows exclusively through the governed dispatcher
//!   (`RunManager::execute_direct_tool` → global MCP registry, approvals,
//!   schema validation, admission context). This module adds no executor.
//! - `transport: "internal"` is declaration-only (a native tool with the same
//!   name remains the only executable binding, resolved by the existing
//!   registry — this surface cannot shadow it: ids containing `__` are
//!   rejected to prevent native/MCP namespace impersonation).
//!
//! Storage mirrors `mcp_admin`: one JSON settings row (`tools.definitions`,
//! seeded by the settings manager) mapping `tool_id` → record. Governance:
//! every record carries the creating principal's owner key
//! (`user_settings::principal_storage_key`); read/update/delete require an
//! exact owner match, and cross-owner reads answer 404 (existence is not
//! leaked). Anonymous local callers (no-user-login contract,
//! `jwt_required=false`) share the `None`-owner local bucket; sidecar
//! host-token callers are scoped to the bound principal. Creating a
//! definition grants nothing: run-engine tool availability is unchanged
//! (`collect_authorized_tools` does not read this row), and no agent policy
//! is touched.
//!
//! Auth posture mirrors `mcp_admin` (no admin-key requirement): the frozen
//! R11 contract is anonymous local REST; the sidecar lane is already gated by
//! the launch-token guard. Unlike global MCP definitions — which stay
//! spec-locked to 409 in sidecar mode — tool definitions are owner-scoped
//! records and are permitted for the host-bound principal (operator ruling
//! 2026-10-20; the global-MCP 409 is unchanged).

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::AppState;
use crate::uar::api::user_settings::principal_storage_key;
use crate::uar::domain::tools::{ToolDefinition, ToolTransport};
use crate::uar::security::claims::UserContext;
use crate::uar::settings::manager::SettingsManager;

/// Single settings row holding all stored tool definitions, mirroring
/// `mcp_admin::SETTINGS_KEY` ("mcp.servers").
pub const SETTINGS_KEY: &str = "tools.definitions";

/// One persisted tool definition with owner governance metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredToolDefinition {
    pub definition: ToolDefinition,
    /// Owner key from `principal_storage_key`; `None` = anonymous local
    /// bucket (no-user-login contract).
    pub owner: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

type DefinitionMap = BTreeMap<String, StoredToolDefinition>;

pub fn build_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_tool_definition))
        .route(
            "/{name}",
            get(read_tool_definition)
                .put(update_tool_definition)
                .delete(delete_tool_definition),
        )
}

fn error_response(status: StatusCode, code: &str, message: &str) -> Response {
    (
        status,
        Json(json!({
            "error": { "code": code, "message": message }
        })),
    )
        .into_response()
}

fn manager(state: &AppState) -> Result<Arc<SettingsManager>, Response> {
    state.settings_manager.clone().ok_or_else(|| {
        error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "settings_unavailable",
            "tool definition storage is unavailable on this process",
        )
    })
}

fn owner_key(user: Option<&Extension<UserContext>>) -> Option<String> {
    user.map(|Extension(ctx)| ctx).and_then(principal_storage_key)
}

async fn load_definitions(manager: &SettingsManager) -> DefinitionMap {
    match manager.get_value(SETTINGS_KEY).await {
        Some(value) => serde_json::from_value::<DefinitionMap>(value).unwrap_or_default(),
        None => DefinitionMap::new(),
    }
}

async fn persist(manager: &SettingsManager, definitions: &DefinitionMap) -> Result<(), Response> {
    let value = serde_json::to_value(definitions).map_err(|error| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "definition_serialize_failed",
            &error.to_string(),
        )
    })?;
    manager.set_value(SETTINGS_KEY, value).await.map_err(|error| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "definition_persist_failed",
            &error.to_string(),
        )
    })
}

/// Safety rules for a stored definition. Executor binding is a validated
/// REFERENCE to an already-governed executor; nothing here creates a new
/// execution path and nothing broadens an agent grant.
fn validate(state: &AppState, definition: &ToolDefinition) -> Result<(), Response> {
    if definition.tool_id.trim().is_empty() || definition.name.trim().is_empty() {
        return Err(error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "definition_invalid",
            "tool_id and name must be non-empty",
        ));
    }
    // Reserved-namespace guard: native tools are `native__*` and MCP tools are
    // `<server>__<tool>`; a stored id containing `__` could impersonate either
    // in merged catalog surfaces.
    if definition.tool_id.contains("__") {
        return Err(error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "definition_reserved_namespace",
            "tool_id must not contain '__' (reserved for native/MCP namespacing)",
        ));
    }
    if !definition.input_schema.is_object() {
        return Err(error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "definition_invalid",
            "input_schema must be a JSON object",
        ));
    }
    match &definition.transport {
        // Arbitrary HTTP egress from a stored definition is refused by design
        // (operator ruling 2026-10-20: never turn storage into execution).
        ToolTransport::Http { .. } => Err(error_response(
            StatusCode::BAD_REQUEST,
            "transport_not_permitted",
            "http transport is not permitted for stored tool definitions; bind 'internal' (declaration only) or 'mcp' (registered server reference)",
        )),
        ToolTransport::Mcp { server_name } => {
            if state.mcp.server_names().contains(server_name) {
                Ok(())
            } else {
                Err(error_response(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "mcp_server_not_registered",
                    "transport.mcp.server_name must reference a currently registered MCP server",
                ))
            }
        }
        ToolTransport::Internal => Ok(()),
    }
}

async fn create_tool_definition(
    State(state): State<AppState>,
    user: Option<Extension<UserContext>>,
    Json(definition): Json<ToolDefinition>,
) -> Response {
    let manager = match manager(&state) {
        Ok(manager) => manager,
        Err(response) => return response,
    };
    if let Err(response) = validate(&state, &definition) {
        return response;
    }
    let owner = owner_key(user.as_ref());
    let mut definitions = load_definitions(&manager).await;
    if definitions.contains_key(&definition.tool_id) {
        return error_response(
            StatusCode::CONFLICT,
            "definition_exists",
            "a tool definition with this tool_id already exists",
        );
    }
    let now = chrono::Utc::now().to_rfc3339();
    let stored = StoredToolDefinition {
        definition: definition.clone(),
        owner,
        created_at: now.clone(),
        updated_at: now,
    };
    definitions.insert(definition.tool_id.clone(), stored.clone());
    if let Err(response) = persist(&manager, &definitions).await {
        return response;
    }
    (StatusCode::CREATED, Json(stored)).into_response()
}

async fn read_tool_definition(
    State(state): State<AppState>,
    user: Option<Extension<UserContext>>,
    Path(tool_id): Path<String>,
) -> Response {
    let manager = match manager(&state) {
        Ok(manager) => manager,
        Err(response) => return response,
    };
    let definitions = load_definitions(&manager).await;
    let owner = owner_key(user.as_ref());
    match definitions.get(&tool_id) {
        // Cross-owner reads answer 404: existence is not leaked.
        Some(stored) if stored.owner == owner => (StatusCode::OK, Json(stored.clone())).into_response(),
        _ => error_response(
            StatusCode::NOT_FOUND,
            "definition_not_found",
            "no tool definition with this tool_id is visible to the caller",
        ),
    }
}

async fn update_tool_definition(
    State(state): State<AppState>,
    user: Option<Extension<UserContext>>,
    Path(tool_id): Path<String>,
    Json(definition): Json<ToolDefinition>,
) -> Response {
    let manager = match manager(&state) {
        Ok(manager) => manager,
        Err(response) => return response,
    };
    if definition.tool_id != tool_id {
        return error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "definition_id_mismatch",
            "body tool_id must equal the path tool_id",
        );
    }
    if let Err(response) = validate(&state, &definition) {
        return response;
    }
    let owner = owner_key(user.as_ref());
    let mut definitions = load_definitions(&manager).await;
    match definitions.get(&tool_id) {
        Some(stored) if stored.owner == owner => {}
        Some(_) => {
            return error_response(
                StatusCode::NOT_FOUND,
                "definition_not_found",
                "no tool definition with this tool_id is visible to the caller",
            );
        }
        None => {
            return error_response(
                StatusCode::NOT_FOUND,
                "definition_not_found",
                "no tool definition with this tool_id",
            );
        }
    }
    let created_at = definitions
        .get(&tool_id)
        .map(|stored| stored.created_at.clone())
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let stored = StoredToolDefinition {
        definition,
        owner,
        created_at,
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    definitions.insert(tool_id, stored.clone());
    if let Err(response) = persist(&manager, &definitions).await {
        return response;
    }
    (StatusCode::OK, Json(stored)).into_response()
}

async fn delete_tool_definition(
    State(state): State<AppState>,
    user: Option<Extension<UserContext>>,
    Path(tool_id): Path<String>,
) -> Response {
    let manager = match manager(&state) {
        Ok(manager) => manager,
        Err(response) => return response,
    };
    let owner = owner_key(user.as_ref());
    let mut definitions = load_definitions(&manager).await;
    match definitions.get(&tool_id) {
        Some(stored) if stored.owner == owner => {}
        _ => {
            return error_response(
                StatusCode::NOT_FOUND,
                "definition_not_found",
                "no tool definition with this tool_id is visible to the caller",
            );
        }
    }
    definitions.remove(&tool_id);
    if let Err(response) = persist(&manager, &definitions).await {
        return response;
    }
    StatusCode::NO_CONTENT.into_response()
}

/// Owner-visible stored definitions for catalog merging
/// (`discovery::list_tools`). Errors collapse to an empty list: the catalog
/// must stay available even when the settings row is unreadable.
pub async fn visible_definitions(
    state: &AppState,
    user: Option<&Extension<UserContext>>,
) -> Vec<StoredToolDefinition> {
    let Some(manager) = state.settings_manager.as_ref() else {
        return Vec::new();
    };
    let definitions = load_definitions(manager).await;
    let owner = owner_key(user);
    definitions
        .into_values()
        .filter(|stored| stored.owner == owner)
        .collect()
}
