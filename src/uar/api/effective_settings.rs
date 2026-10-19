//! Secret-free inspection of the running authorities, not stored preferences.

use std::sync::Arc;

use axum::{Extension, Json, Router, extract::State, http::{HeaderMap, StatusCode}, routing::get};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{AppState, mcp::config::McpServerEntry};
use crate::uar::{governance::runtime_control::GovernanceStatusHandle,
    security::claims::UserContext, tools::descriptor::{ToolEffect, ToolSource}};

#[derive(Clone)]
pub struct EffectiveSettingsState {
    pub app: AppState,
    pub governance: GovernanceStatusHandle,
}

impl std::fmt::Debug for EffectiveSettingsState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EffectiveSettingsState").finish_non_exhaustive()
    }
}

pub fn router(state: EffectiveSettingsState) -> Router<AppState> {
    Router::new().route("/api/uar/settings/effective", get(read_effective))
        .with_state(Arc::new(state))
}

async fn read_effective(
    State(state): State<Arc<EffectiveSettingsState>>,
    headers: HeaderMap,
    user: Option<Extension<UserContext>>,
) -> Result<Json<Value>, StatusCode> {
    // The outer verifier establishes identity. A settings key, role-shaped
    // synthetic context or sidecar launch secret cannot substitute for it.
    if !headers.get("authorization").and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.strip_prefix("Bearer ").is_some_and(|t| !t.is_empty())) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let user = user.ok_or(StatusCode::UNAUTHORIZED)?.0;
    let identity = user.verified_identity().ok_or(StatusCode::UNAUTHORIZED)?;
    if !identity.roles().iter().any(|role| role == "admin") {
        return Err(StatusCode::FORBIDDEN);
    }
    let app = &state.app;
    let memory = match &app.memory_service {
        None => json!({"provider_host":"disabled", "model":""}),
        Some(service) => {
            let config = service.config();
            // These endpoints are fixed in the pinned memory service adapters.
            let host = match config.embedding_provider.as_str() {
                "openai" => "api.openai.com",
                "cohere" => "api.cohere.ai",
                _ => "local",
            };
            json!({"provider_host":host, "model":config.embedding_model})
        }
    };
    let embedder = app.embedding_backend.pin();
    let embedding_host = match embedder.endpoint() {
        Some(endpoint) => endpoint_host(endpoint)?,
        None => match embedder.backend_name() {
            "fastembed" | "candle" => "local".to_owned(),
            "unavailable" => "disabled".to_owned(),
            _ => return Err(StatusCode::SERVICE_UNAVAILABLE),
        },
    };
    let knowledge = json!({"provider_host":embedding_host, "model":embedder.model_id()});
    // Hold each owning registry's read locks until the final synchronous
    // governance sample. That sample is the common snapshot point. No network
    // or persistence operation runs while these guards are held.
    app.provider_registry.inspect_default(|selected| async {
        let llm = match selected.filter(|p| p.enabled) {
            Some(provider) if provider.default_model.as_deref().is_some_and(|m| !m.trim().is_empty()) => {
                json!({"provider_host":endpoint_host(&provider.base_url)?,
                    "model":provider.default_model, "provider_id":provider.id})
            }
            _ => json!({"provider_host":"disabled", "model":""}),
        };
        app.native_skill_registry.inspect_descriptors(|descriptors| {
            app.mcp.inspect_server_entries(|entries| {
                let mut servers = Vec::with_capacity(entries.len());
                for (name, entry) in entries {
                    servers.push(match entry {
                        McpServerEntry::Stdio { command, .. } =>
                            json!({"name":name,"transport":"stdio","command":command}),
                        McpServerEntry::RemoteHttp { url, .. } =>
                            json!({"name":name,"transport":"streamable-http","url":public_endpoint(&url)?}),
                    });
                }
                servers.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
                let mut shell_tools = descriptors.iter()
                    .filter(|d| d.effect == ToolEffect::CodeExecution)
                    .map(|d| d.provider_name.clone()).collect::<Vec<_>>();
                shell_tools.extend(app.mcp.descriptors().into_iter()
                    .filter(|d| d.source == ToolSource::BuiltIn && d.effect == ToolEffect::CodeExecution)
                    .map(|d| d.provider_name.clone()));
                shell_tools.sort(); shell_tools.dedup();
                let terminal = shell_tools.iter().any(|name| name == "terminal_exec");
                shell_tools.retain(|name| name != "terminal_exec");
                let governance = state.governance.snapshot();
                governance.validate().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
                let mut result = json!({
                    "schema_version":1,
                    "llm":llm,
                    "knowledge_base_embedder":knowledge,
                    "memory_embedder":memory,
                    "governance":{"enabled":governance.effective_enabled,
                        "tool_approval":"per_run"},
                    "native_tools":{"terminal_exec_enabled":terminal,"shell_tools":shell_tools},
                    "mcp_servers":servers,
                    "scope":"runtime_defaults; explicit owner/run overrides retain their own admission"
                });
                let bytes = serde_json::to_vec(&result).map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
                result["revision"] = json!(format!("sha256:{}", Sha256::digest(bytes)
                    .iter().map(|byte| format!("{byte:02x}")).collect::<String>()));
                Ok(Json(result))
            }).map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
        }).await
    }).await
}

fn endpoint_host(endpoint: &str) -> Result<String, StatusCode> {
    let url = url::Url::parse(endpoint).map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    if !matches!(url.scheme(), "http" | "https") { return Err(StatusCode::SERVICE_UNAVAILABLE); }
    let host = url.host_str().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    if host.eq_ignore_ascii_case("localhost") || host.trim_matches(['[', ']'])
        .parse::<std::net::IpAddr>().is_ok_and(|ip| ip.is_loopback()) {
        return Ok("local".to_owned());
    }
    Ok(match url.port() { Some(port) => format!("{host}:{port}"), None => host.to_owned() })
}

fn public_endpoint(endpoint: &str) -> Result<String, StatusCode> {
    let mut url = url::Url::parse(endpoint).map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    endpoint_host(endpoint)?;
    url.set_username("").map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    url.set_password(None).map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    url.set_query(None); url.set_fragment(None);
    Ok(url.to_string())
}
