//! Secret-free inspection of the running authorities, not stored preferences.

use std::sync::Arc;

use axum::{Extension, Json, Router, extract::State, http::{HeaderMap, StatusCode}, routing::get};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{AppState, mcp::config::McpServerEntry};
use crate::uar::{governance::runtime_control::GovernanceStatusHandle,
    security::claims::UserContext, tools::descriptor::{ToolDescriptor, ToolEffect, ToolSource}};

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
    // Pin the active backend once for this diagnostics snapshot.
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
                let builtin = app.mcp.descriptors();
                let mut shell_tools = code_execution_tools(&descriptors, &builtin);
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

/// Provider names of native and built-in tools whose declared effect is
/// [`ToolEffect::CodeExecution`], sorted and de-duplicated. MCP-server tools
/// are excluded: they are counted as servers, not native tools.
fn code_execution_tools(
    native: &[Arc<ToolDescriptor>],
    mcp: &[Arc<ToolDescriptor>],
) -> Vec<String> {
    let mut names = native.iter()
        .filter(|d| d.effect == ToolEffect::CodeExecution)
        .map(|d| d.provider_name.clone()).collect::<Vec<_>>();
    names.extend(mcp.iter()
        .filter(|d| d.source == ToolSource::BuiltIn && d.effect == ToolEffect::CodeExecution)
        .map(|d| d.provider_name.clone()));
    names.sort(); names.dedup();
    names
}

/// Native tools that read or write the UAR host filesystem. The descriptor
/// effect enum has no filesystem or network effect, so these are named
/// explicitly: `file_read` is `ReadOnly` and `file_write`/`file_patch` are
/// `ExternalMutation`, yet all three act on the host's files.
/// `ExternalMutation` as a whole is not counted: it also covers memory, team
/// and schedule tools that mutate UAR state, not the host. `web_fetch` is
/// network egress, not an action on the host, and is not counted.
pub const HOST_FILESYSTEM_TOOLS: [&str; 3] = ["file_read", "file_write", "file_patch"];

/// Deployment-level facts about tools that act on the UAR host, reported in
/// the governance status so a host can refuse to run on a UAR that would
/// expose its own host tools to a run by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct HostToolFacts {
    /// Deployment-level MCP servers (configured or connected) that runs get
    /// by default. Run-scoped `mcp_servers` are host-supplied and excluded.
    pub global_mcp_servers: usize,
    /// Native and built-in tools registered by default that act on the UAR
    /// host: every `CodeExecution`-effect tool (this includes
    /// `terminal_exec`) plus [`HOST_FILESYSTEM_TOOLS`].
    pub native_tools: usize,
    /// True when a deployment-level host tool admission is configured. The
    /// HTTP server has none: paired-host admission is only ever per run and
    /// requires an authenticated sidecar launch, so the server reports
    /// `false`. An embedded runtime may register one at build time.
    pub tool_admission: bool,
}

impl HostToolFacts {
    /// Classify an inventory. `native` are skill-registry descriptors and
    /// `mcp` the MCP registry descriptors, of which only built-in ones count.
    #[must_use]
    pub fn classify(
        global_mcp_servers: usize,
        native: &[Arc<ToolDescriptor>],
        mcp: &[Arc<ToolDescriptor>],
        tool_admission: bool,
    ) -> Self {
        let mut names = code_execution_tools(native, mcp);
        names.extend(native.iter().chain(mcp.iter().filter(|d| d.source == ToolSource::BuiltIn))
            .filter(|d| HOST_FILESYSTEM_TOOLS.contains(&d.provider_name.as_str()))
            .map(|d| d.provider_name.clone()));
        names.sort(); names.dedup();
        Self { global_mcp_servers, native_tools: names.len(), tool_admission }
    }
}

/// Source of [`HostToolFacts`] for the governance status route.
#[async_trait::async_trait]
pub trait HostToolFactsSource: Send + Sync {
    /// Sample the current deployment-level host-tool inventory.
    ///
    /// # Errors
    /// Fails when an owning registry cannot be read.
    async fn host_tool_facts(&self) -> anyhow::Result<HostToolFacts>;
}

/// The running server's registries.
pub struct AppHostToolFacts(pub AppState);

impl std::fmt::Debug for AppHostToolFacts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppHostToolFacts").finish_non_exhaustive()
    }
}

#[async_trait::async_trait]
impl HostToolFactsSource for AppHostToolFacts {
    async fn host_tool_facts(&self) -> anyhow::Result<HostToolFacts> {
        let app = &self.0;
        let native = app.native_skill_registry.inspect_descriptors(|descriptors| descriptors).await;
        let servers = app.mcp.inspect_server_entries(|entries| entries.len())?;
        Ok(HostToolFacts::classify(servers, &native, &app.mcp.descriptors(), false))
    }
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

#[cfg(test)]
mod host_tool_fact_tests {
    use super::*;
    use crate::{config::NativeToolsConfig, uar::runtime::native_skill::NativeSkillRegistry};

    async fn builtin_descriptors(config: &NativeToolsConfig) -> Vec<Arc<ToolDescriptor>> {
        let registry = NativeSkillRegistry::new();
        crate::uar::runtime::native_skills::register_builtins(&registry, config, None)
            .await.expect("built-in tools register");
        registry.inspect_descriptors(|descriptors| descriptors).await
    }

    #[tokio::test]
    async fn default_native_tool_config_reports_no_host_tools() {
        let native = builtin_descriptors(&NativeToolsConfig::default()).await;
        assert!(!native.is_empty(), "core built-ins are always registered");
        assert_eq!(HostToolFacts::classify(0, &native, &[], false),
            HostToolFacts { global_mcp_servers: 0, native_tools: 0, tool_admission: false });
    }

    #[tokio::test]
    async fn code_execution_and_host_filesystem_tools_and_mcp_servers_are_counted() {
        let config = NativeToolsConfig { terminal_exec_enabled: true, file_tools_enabled: true,
            web_fetch_enabled: true, ..NativeToolsConfig::default() };
        let native = builtin_descriptors(&config).await;
        let facts = HostToolFacts::classify(2, &native, &[], false);
        // terminal_exec plus file_read, file_write and file_patch; web_fetch is not host-acting.
        assert_eq!(facts, HostToolFacts { global_mcp_servers: 2, native_tools: 4, tool_admission: false });
        let only_terminal = NativeToolsConfig { terminal_exec_enabled: true, ..NativeToolsConfig::default() };
        let native = builtin_descriptors(&only_terminal).await;
        assert_eq!(HostToolFacts::classify(0, &native, &[], false).native_tools, 1);
        assert!(code_execution_tools(&native, &[]).contains(&"terminal_exec".to_owned()));
    }
}
