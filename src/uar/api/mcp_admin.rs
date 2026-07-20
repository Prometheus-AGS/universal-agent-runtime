//! Runtime MCP server administration backed by UAR settings storage.

use crate::{
    AppState,
    mcp::{config::McpServerEntry, registry::McpRegistry},
    uar::settings::manager::SettingsManager,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc};

const SETTINGS_KEY: &str = "mcp.servers";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMcpServer {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub entry: McpServerEntry,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct SaveMcpServerRequest {
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub transport: String,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub sandboxed: bool,
}

impl SaveMcpServerRequest {
    fn into_stored(self) -> Result<(String, StoredMcpServer), String> {
        let name = self.name.trim().to_string();
        if name.is_empty() {
            return Err("MCP server name is required".into());
        }
        let entry = match self.transport.as_str() {
            "stdio" => {
                let command = self.command.unwrap_or_default().trim().to_string();
                if command.is_empty() {
                    return Err("stdio MCP server command is required".into());
                }
                McpServerEntry::Stdio {
                    command,
                    args: self.args,
                    env: self.env,
                    sandboxed: self.sandboxed,
                }
            }
            "remote_http" | "http" => {
                let url = self.url.unwrap_or_default().trim().to_string();
                if url.is_empty() {
                    return Err("remote MCP server URL is required".into());
                }
                McpServerEntry::RemoteHttp { url, env: self.env }
            }
            other => return Err(format!("unsupported MCP transport '{other}'")),
        };
        Ok((
            name,
            StoredMcpServer {
                enabled: self.enabled,
                entry,
            },
        ))
    }
}

pub fn build_router() -> Router<AppState> {
    Router::new()
        .route("/servers", get(list_servers))
        .route("/servers/{name}", put(save_server).delete(delete_server))
}

async fn stored_servers(state: &AppState) -> HashMap<String, StoredMcpServer> {
    if let Some(manager) = &state.settings_manager
        && let Some(value) = manager.get_value(SETTINGS_KEY).await
        && let Ok(servers) = serde_json::from_value(value)
    {
        return servers;
    }
    state
        .mcp
        .server_entries()
        .into_iter()
        .map(|(name, entry)| {
            (
                name,
                StoredMcpServer {
                    enabled: true,
                    entry,
                },
            )
        })
        .collect()
}

async fn persist(
    state: &AppState,
    servers: &HashMap<String, StoredMcpServer>,
) -> Result<(), String> {
    let manager = state
        .settings_manager
        .as_ref()
        .ok_or_else(|| "UAR settings storage is unavailable".to_string())?;
    manager
        .set_value(
            SETTINGS_KEY,
            serde_json::to_value(servers).map_err(|error| error.to_string())?,
        )
        .await
        .map_err(|error| error.to_string())
}

fn public_server(name: &str, stored: &StoredMcpServer, state: &AppState) -> Value {
    let connected = state.mcp.server_names().iter().any(|item| item == name);
    let tools = state
        .mcp
        .tools()
        .into_iter()
        .filter(|(tool, _)| tool.starts_with(&format!("{name}__")))
        .map(|(tool, _)| tool)
        .collect::<Vec<_>>();
    match &stored.entry {
        McpServerEntry::Stdio {
            command,
            args,
            env,
            sandboxed,
        } => json!({
            "name": name,
            "enabled": stored.enabled,
            "connected": connected,
            "transport": "stdio",
            "command": command,
            "args": args,
            "envKeys": env.keys().collect::<Vec<_>>(),
            "sandboxed": sandboxed,
            "tools": tools,
        }),
        McpServerEntry::RemoteHttp { url, env } => json!({
            "name": name,
            "enabled": stored.enabled,
            "connected": connected,
            "transport": "remote_http",
            "url": url,
            "envKeys": env.keys().collect::<Vec<_>>(),
            "tools": tools,
        }),
    }
}

async fn list_servers(State(state): State<AppState>) -> Json<Value> {
    let servers = stored_servers(&state).await;
    let mut views = servers
        .iter()
        .map(|(name, stored)| public_server(name, stored, &state))
        .collect::<Vec<_>>();
    views.sort_by(|left, right| left["name"].as_str().cmp(&right["name"].as_str()));
    Json(json!({ "servers": views }))
}

async fn save_server(
    State(state): State<AppState>,
    Path(path_name): Path<String>,
    Json(mut request): Json<SaveMcpServerRequest>,
) -> impl IntoResponse {
    if request.name.trim().is_empty() {
        request.name = path_name.clone();
    }
    if request.name != path_name {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "path and payload MCP server names differ"})),
        )
            .into_response();
    }
    let (name, stored) = match request.into_stored() {
        Ok(value) => value,
        Err(error) => {
            return (StatusCode::BAD_REQUEST, Json(json!({"error": error}))).into_response();
        }
    };
    if stored.enabled {
        if let Err(error) = state
            .mcp
            .upsert_server(name.clone(), stored.entry.clone())
            .await
        {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({"error": error.to_string()})),
            )
                .into_response();
        }
    } else {
        state.mcp.remove_server(&name);
    }
    let mut servers = stored_servers(&state).await;
    servers.insert(name.clone(), stored.clone());
    if let Err(error) = persist(&state, &servers).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": error})),
        )
            .into_response();
    }
    Json(public_server(&name, &stored, &state)).into_response()
}

async fn delete_server(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    state.mcp.remove_server(&name);
    let mut servers = stored_servers(&state).await;
    let removed = servers.remove(&name).is_some();
    if let Err(error) = persist(&state, &servers).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": error})),
        )
            .into_response();
    }
    Json(json!({"deleted": removed, "name": name})).into_response()
}

/// Seed file-based MCP definitions once, then hydrate runtime connections from UAR storage.
pub async fn hydrate_registry(
    registry: &Arc<McpRegistry>,
    manager: &Arc<SettingsManager>,
) -> anyhow::Result<()> {
    let current = registry
        .server_entries()
        .into_iter()
        .map(|(name, entry)| {
            (
                name,
                StoredMcpServer {
                    enabled: true,
                    entry,
                },
            )
        })
        .collect::<HashMap<_, _>>();
    let stored = manager
        .get_value(SETTINGS_KEY)
        .await
        .and_then(|value| serde_json::from_value::<HashMap<String, StoredMcpServer>>(value).ok())
        .unwrap_or_default();
    let effective = if stored.is_empty() && !current.is_empty() {
        manager
            .set_value(SETTINGS_KEY, serde_json::to_value(&current)?)
            .await?;
        current
    } else {
        stored
    };
    for name in registry.server_names() {
        registry.remove_server(&name);
    }
    for (name, server) in effective {
        if server.enabled {
            if let Err(error) = registry.upsert_server(name.clone(), server.entry).await {
                tracing::warn!(server = %name, %error, "persisted MCP server could not reconnect");
            }
        }
    }
    Ok(())
}
