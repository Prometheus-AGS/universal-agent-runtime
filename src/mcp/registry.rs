use crate::mcp::config::{
    McpServerEntry, expand_env_map, expand_env_placeholders, expand_env_placeholders_strict,
    load_mcp_config,
};
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use rmcp::{
    model::{CallToolRequestParams, Tool},
    service::ServiceExt,
    transport::{StreamableHttpClientTransport, TokioChildProcess},
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::process::Command;
use url::Url;

/// How long one MCP server gets to complete its handshake before it is
/// skipped.
///
/// **No MCP server may prevent the runtime from serving prompts.** This is a
/// hard invariant, not a tuning knob: an agent runtime whose startup can be
/// blocked by a third-party tool process is one bad `npx` package away from
/// total unavailability, and the failure is silent — ports bind, nothing
/// answers, and no log names the cause.
///
/// 20s is generous for a local process handshake (a cold `npx` fetch is the
/// slow case) while bounding worst-case startup at
/// `servers × MCP_CONNECT_TIMEOUT` rather than infinity.
const MCP_CONNECT_TIMEOUT: Duration = Duration::from_secs(20);

/// How long a connected server gets to answer `tools/list`.
///
/// Separate from the connect budget because the failures differ: a handshake
/// hangs on process startup, `tools/list` hangs on a server that connected and
/// then stopped responding. Shorter, because the connection is already proven.
const MCP_LIST_TOOLS_TIMEOUT: Duration = Duration::from_secs(10);

/// How long any single tool call gets — MCP or native — before it is failed.
///
/// A tool that hangs mid-turn must surface as a failed tool result the model
/// can react to, never as a run that never ends. This was already applied to
/// MCP tool calls as a bare `30` and is now named and shared, so the native
/// path cannot silently diverge from it again.
const MCP_TOOL_CALL_TIMEOUT: Duration = Duration::from_secs(30);

#[async_trait]
pub trait NativeTool: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn schema(&self) -> serde_json::Value;
    async fn call(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value>;
}

type DynClientService = rmcp::service::RunningService<rmcp::service::RoleClient, ()>;
struct ClientServiceState {
    service: Arc<DynClientService>,
    reconnect_entry: McpServerEntry,
    generation: u64,
    reconnects_in_flight: usize,
    shutting_down: bool,
}

type SharedClientService = Arc<RwLock<ClientServiceState>>;

fn new_service_slot(
    service: DynClientService,
    reconnect_entry: McpServerEntry,
) -> SharedClientService {
    Arc::new(RwLock::new(ClientServiceState {
        service: Arc::new(service),
        reconnect_entry,
        generation: 0,
        reconnects_in_flight: 0,
        shutting_down: false,
    }))
}

fn current_service(slot: &SharedClientService) -> Arc<DynClientService> {
    Arc::clone(
        &slot
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .service,
    )
}

fn current_reconnect_entry(slot: &SharedClientService) -> (McpServerEntry, u64) {
    let state = slot
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    (state.reconnect_entry.clone(), state.generation)
}

fn replace_configured_service(
    slot: &SharedClientService,
    service: DynClientService,
    reconnect_entry: McpServerEntry,
) -> Arc<DynClientService> {
    let mut state = slot
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let replaced = std::mem::replace(&mut state.service, Arc::new(service));
    state.reconnect_entry = reconnect_entry;
    state.generation = state.generation.wrapping_add(1);
    replaced
}

fn begin_reconnect(slot: &SharedClientService) -> Option<(McpServerEntry, u64)> {
    let mut state = slot
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if state.shutting_down {
        return None;
    }
    state.reconnects_in_flight += 1;
    Some((state.reconnect_entry.clone(), state.generation))
}

fn install_reconnected_service(
    slot: &SharedClientService,
    expected_generation: u64,
    service: DynClientService,
) -> Result<(), DynClientService> {
    let mut state = slot
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if state.shutting_down || state.generation != expected_generation {
        return Err(service);
    }
    state.service = Arc::new(service);
    state.reconnects_in_flight -= 1;
    Ok(())
}

fn finish_reconnect(slot: &SharedClientService) {
    let mut state = slot
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    state.reconnects_in_flight -= 1;
}

fn begin_service_shutdown(slot: &SharedClientService) -> Arc<DynClientService> {
    let mut state = slot
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    state.shutting_down = true;
    Arc::clone(&state.service)
}

fn service_shutdown_complete(slot: &SharedClientService, service: &DynClientService) -> bool {
    let state = slot
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    state.reconnects_in_flight == 0 && service.is_transport_closed()
}

async fn connect_server(name: &str, entry: &McpServerEntry) -> anyhow::Result<DynClientService> {
    match entry {
        McpServerEntry::Stdio {
            command, args, env, ..
        } => {
            let mut cmd = Command::new(resolve_mcp_command(command));
            cmd.args(args);
            for (key, value) in expand_env_map(env) {
                cmd.env(key, value);
            }
            let transport = TokioChildProcess::new(cmd)
                .with_context(|| format!("failed to spawn stdio MCP server '{name}'"))?;
            ().serve(transport)
                .await
                .with_context(|| format!("failed to connect stdio MCP server '{name}'"))
        }
        McpServerEntry::RemoteHttp { url, env } => {
            let env = expand_env_map(env);
            let endpoint = resolve_remote_http_url(name, url, &env)?;
            ().serve(StreamableHttpClientTransport::from_uri(
                endpoint.to_string(),
            ))
            .await
            .with_context(|| format!("failed to connect remote MCP server '{name}'"))
        }
    }
}

/// Expands `${VAR}` and `${VAR:-default}` placeholders in a remote MCP
/// server's `url` (per the process environment, same as its `env` map).
/// Tavily's config style embeds `${TAVILY_API_KEY}` directly in the URL and
/// additionally declares it in `env`; if the placeholder is still present
/// after process-env expansion, this substitutes it from the entry's own `env`
/// map, matching that convention. Any other `RemoteHttp` entry (e.g.
/// `surreal_memory`, which has no `TAVILY_API_KEY` at all) is left alone --
/// requiring a Tavily-specific key for every remote server was the bug that
/// took down the whole registry whenever a non-Tavily entry was present.
///
/// The Tavily substitution runs first, against the lenient expansion, so its
/// `env`-map indirection still works. Only afterwards is the result required
/// to be placeholder-free: an unexpanded `${...}` is a configuration error, not
/// a hostname, and must not reach the URL parser verbatim.
fn resolve_remote_http_url(
    name: &str,
    url: &str,
    env: &HashMap<String, String>,
) -> anyhow::Result<Url> {
    let mut expanded = expand_env_placeholders(url);
    if expanded.contains("${TAVILY_API_KEY}") {
        let api_key = env
            .get("TAVILY_API_KEY")
            .cloned()
            .filter(|key| !key.is_empty())
            .ok_or_else(|| anyhow!("remote MCP '{name}' missing TAVILY_API_KEY"))?;
        expanded = expanded.replace("${TAVILY_API_KEY}", &api_key);
    }
    let expanded = expand_env_placeholders_strict(&expanded)
        .with_context(|| format!("cannot resolve url for remote MCP '{name}'"))?;
    Url::parse(&expanded)
        .with_context(|| format!("invalid url for remote MCP '{name}': {expanded}"))
}

#[derive(Clone)]
pub struct McpRegistry {
    services: Arc<RwLock<HashMap<String, SharedClientService>>>,
    shutting_down: Arc<AtomicBool>,
    server_config: Arc<RwLock<HashMap<String, McpServerEntry>>>,
    // namespaced_tool_name -> (server_name, tool_name)
    tool_index: Arc<RwLock<HashMap<String, (String, String)>>>,
    tools: Arc<RwLock<Vec<(String, Tool)>>>, // (namespaced_name, Tool)
    // namespaced_tool_name -> NativeTool
    native_tools: Arc<HashMap<String, Arc<dyn NativeTool>>>,
}

impl std::fmt::Debug for McpRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpRegistry")
            .field(
                "tool_count",
                &self
                    .tools
                    .read()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .len(),
            )
            .field(
                "service_count",
                &self
                    .services
                    .read()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .len(),
            )
            .field("native_tool_count", &self.native_tools.len())
            .finish()
    }
}

impl McpRegistry {
    /// Create an empty registry with no MCP servers or tools.
    pub fn empty() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            shutting_down: Arc::new(AtomicBool::new(false)),
            server_config: Arc::new(RwLock::new(HashMap::new())),
            tool_index: Arc::new(RwLock::new(HashMap::new())),
            tools: Arc::new(RwLock::new(Vec::new())),
            native_tools: Arc::new(HashMap::new()),
        }
    }

    pub async fn load_from_file(path: &str) -> anyhow::Result<Self> {
        let resolved = resolve_mcp_config_path(path);
        let cfg = load_mcp_config(resolved)?;
        Self::from_config(&cfg).await
    }

    pub async fn from_config(cfg: &crate::mcp::config::McpConfig) -> anyhow::Result<Self> {
        // 1) connect all servers
        let mut services: HashMap<String, SharedClientService> = HashMap::new();

        for (name, entry) in &cfg.mcp_servers {
            // A server that never completes its handshake must not be able to
            // hang startup. Observed 2026-09-01: a stdio server printed
            // "running on stdio" and neither side ever finished, so the
            // process bound its ports and then served nothing, forever, with
            // no log line naming the cause. Error handling below was already
            // correct — a failure is skipped and the registry continues — but
            // "never answers" is not an error, so it never reached it.
            let svc = match tokio::time::timeout(
                MCP_CONNECT_TIMEOUT,
                Self::connect_configured_server(name, entry),
            )
            .await
            {
                Err(_elapsed) => {
                    crate::uar::telemetry::metrics::set_mcp_server_status(name, false);
                    tracing::warn!(
                        server = %name,
                        timeout_secs = MCP_CONNECT_TIMEOUT.as_secs(),
                        "MCP server did not complete its handshake in time; skipping it, registry continues without it"
                    );
                    continue;
                }
                Ok(Ok(svc)) => svc,
                Ok(Err(e)) => {
                    crate::uar::telemetry::metrics::set_mcp_server_status(name, false);
                    tracing::warn!(server = %name, error = ?e, "MCP server failed to connect; skipping it, registry continues without it");
                    continue;
                }
            };
            crate::uar::telemetry::metrics::set_mcp_server_status(name, true);
            services.insert(name.clone(), new_service_slot(svc, entry.clone()));
        }

        // 2) list tools + build index
        let mut all_tools: Vec<(String, Tool)> = Vec::new();
        let mut tool_index: HashMap<String, (String, String)> = HashMap::new();

        for (server_name, service_slot) in &services {
            let svc = current_service(service_slot);
            // list_tools exists on the rmcp running service in examples
            let result = match tokio::time::timeout(
                MCP_LIST_TOOLS_TIMEOUT,
                svc.list_tools(Default::default()),
            )
            .await
            {
                Err(_elapsed) => {
                    tracing::warn!(
                        server = %server_name,
                        timeout_secs = MCP_LIST_TOOLS_TIMEOUT.as_secs(),
                        "tools/list timed out for MCP server; skipping its tools, registry continues without them"
                    );
                    continue;
                }
                Ok(Ok(result)) => result,
                Ok(Err(e)) => {
                    tracing::warn!(server = %server_name, error = ?e, "tools/list failed for MCP server; skipping its tools, registry continues without them");
                    continue;
                }
            };

            for t in result.tools {
                let tool_name = t.name.to_string();
                // Sanitize tool name for OpenAI compatibility
                // OpenAI requires: ^[a-zA-Z0-9_-]+$ (no colons, dots, or special chars)
                // Replace :: with __ for namespacing, and sanitize any other invalid chars
                let ns_name = Self::sanitize_tool_name(&format!("{server_name}__{tool_name}"));
                tool_index.insert(ns_name.clone(), (server_name.clone(), tool_name));
                all_tools.push((ns_name, t));
            }
        }

        Ok(Self {
            services: Arc::new(RwLock::new(services)),
            shutting_down: Arc::new(AtomicBool::new(false)),
            server_config: Arc::new(RwLock::new(cfg.mcp_servers.clone())),
            tool_index: Arc::new(RwLock::new(tool_index)),
            tools: Arc::new(RwLock::new(all_tools)),
            native_tools: Arc::new(HashMap::new()),
        })
    }

    /// Connects a single configured MCP server. Split out of `from_config` so
    /// one server's failure (bad URL, unreachable process, missing key) can
    /// be caught and logged per-server instead of aborting the whole
    /// registry load -- previously any single misconfigured entry (e.g. a
    /// non-Tavily `RemoteHttp` server tripping the old hardcoded
    /// `TAVILY_API_KEY` requirement) took every other server down with it.
    async fn connect_configured_server(
        name: &str,
        entry: &McpServerEntry,
    ) -> anyhow::Result<DynClientService> {
        match entry {
            McpServerEntry::Stdio {
                command, args, env, ..
            } => {
                let env = expand_env_map(env);

                let mut command_path = resolve_mcp_command(command);
                // Provisioning: only attempted when the configured
                // command isn't already resolvable (preserves the fast
                // path for the common case where it's already
                // installed). A curated ToolSpec exists for `kreuzberg`;
                // any other name gets an Adopt-only spec, so this never
                // silently installs something for an uncurated tool —
                // it just surfaces a clearer error than the raw spawn
                // failure below would.
                if !crate::uar::orchestrator::provisioning::is_on_path(
                    &command_path.to_string_lossy(),
                ) {
                    let spec = crate::uar::orchestrator::provisioning::known_tool_spec(command);
                    let opts = crate::uar::orchestrator::provisioning::ProvisionOptions::default();
                    match crate::uar::orchestrator::provisioning::ToolProvisioner::resolve(
                        &spec, &opts,
                    )
                    .await
                    {
                        Ok(outcome) => {
                            tracing::info!(
                                server = %name,
                                command = %command,
                                strategy = ?outcome.strategy,
                                path = %outcome.path.display(),
                                "provisioned MCP server command"
                            );
                            command_path = outcome.path;
                        }
                        Err(e) => {
                            tracing::warn!(
                                server = %name,
                                command = %command,
                                error = %e,
                                "could not provision MCP server command; falling back to spawning it as configured"
                            );
                        }
                    }
                }
                let mut cmd = Command::new(&command_path);
                cmd.args(args);

                for (k, v) in env {
                    cmd.env(k, v);
                }

                // rmcp docs show TokioChildProcess + configure pattern for adding args
                let transport = TokioChildProcess::new(cmd)?;
                ().serve(transport)
                    .await
                    .with_context(|| format!("failed to connect stdio MCP server '{name}'"))
            }

            McpServerEntry::RemoteHttp { url, env } => {
                let env = expand_env_map(env);
                let endpoint = resolve_remote_http_url(name, url, &env)?;
                ().serve(StreamableHttpClientTransport::from_uri(
                    endpoint.to_string(),
                ))
                .await
                .with_context(|| format!("failed to connect remote MCP server '{name}'"))
            }
        }
    }

    /// Creates an empty registry for testing.
    pub fn new_empty() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            shutting_down: Arc::new(AtomicBool::new(false)),
            server_config: Arc::new(RwLock::new(HashMap::new())),
            tool_index: Arc::new(RwLock::new(HashMap::new())),
            tools: Arc::new(RwLock::new(Vec::new())),
            native_tools: Arc::new(HashMap::new()),
        }
    }

    /// Creates a registry with a single test tool.
    pub fn new_with_test_tool(name: &str, description: &str) -> Self {
        let ns_name = Self::sanitize_tool_name(&format!("test__{name}"));
        // rmcp 1.8: Tool is #[non_exhaustive] -- struct-literal syntax (even
        // with ..Default::default()) is rejected cross-crate; use Tool::new.
        let tool = Tool::new(
            name.to_string(),
            description.to_string(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "mirror": { "type": "string" }
                },
                "required": ["mirror"]
            })
            .as_object()
            .unwrap()
            .clone(),
        );

        let tools = vec![(ns_name.clone(), tool)];
        let mut tool_index = HashMap::new();
        tool_index.insert(ns_name, ("test".to_string(), name.to_string()));

        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            shutting_down: Arc::new(AtomicBool::new(false)),
            server_config: Arc::new(RwLock::new(HashMap::new())),
            tool_index: Arc::new(RwLock::new(tool_index)),
            tools: Arc::new(RwLock::new(tools)),
            native_tools: Arc::new(HashMap::new()),
        }
    }

    /// Sanitize tool names for `OpenAI` API compatibility.
    fn sanitize_tool_name(name: &str) -> String {
        name.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                    c
                } else {
                    // Replace dots, colons, and any other invalid chars with underscore
                    '_'
                }
            })
            .collect()
    }

    /// Return namespaced tools as `(namespaced_name, Tool)`
    pub fn tools(&self) -> Vec<(String, Tool)> {
        self.tools
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Return configured MCP server names.
    pub fn server_names(&self) -> Vec<String> {
        self.services
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .keys()
            .cloned()
            .collect()
    }

    /// Return whether the namespaced tool is backed by an in-process native tool.
    pub fn is_native_tool(&self, namespaced_name: &str) -> bool {
        self.native_tools.contains_key(namespaced_name)
    }

    /// Resolve the backing MCP server and raw tool name for a namespaced tool.
    pub fn resolve_mcp_tool(&self, namespaced_name: &str) -> Option<(String, String)> {
        self.tool_index
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(namespaced_name)
            .cloned()
    }

    /// Return persisted runtime MCP definitions without exposing connected handles.
    pub fn server_entries(&self) -> HashMap<String, McpServerEntry> {
        let mut entries = self
            .server_config
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        for (name, service_slot) in self
            .services
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
        {
            entries.insert(name.clone(), current_reconnect_entry(service_slot).0);
        }
        entries
    }

    /// Add or replace one MCP server and immediately refresh its advertised tools.
    pub async fn upsert_server(&self, name: String, entry: McpServerEntry) -> anyhow::Result<()> {
        if self.shutting_down.load(Ordering::Acquire) {
            return Err(anyhow!("MCP registry is shutting down"));
        }
        let service = Self::connect_configured_server(&name, &entry).await?;
        let result = service
            .list_tools(Default::default())
            .await
            .with_context(|| format!("tools/list failed for MCP server '{name}'"))?;

        let mut discovered = Vec::new();
        let mut indexed = Vec::new();
        for tool in result.tools {
            let raw_name = tool.name.to_string();
            let namespaced = Self::sanitize_tool_name(&format!("{name}__{raw_name}"));
            indexed.push((namespaced.clone(), (name.clone(), raw_name)));
            discovered.push((namespaced, tool));
        }

        if self.shutting_down.load(Ordering::Acquire) {
            let _ = service.cancel().await;
            return Err(anyhow!("MCP registry is shutting down"));
        }

        let replaced = {
            let mut services = self
                .services
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(service_slot) = services.get(&name) {
                Some(replace_configured_service(
                    service_slot,
                    service,
                    entry.clone(),
                ))
            } else {
                services.insert(name.clone(), new_service_slot(service, entry.clone()));
                None
            }
        };
        if let Some(replaced) = replaced {
            replaced.cancellation_token().cancel();
            while !replaced.is_transport_closed() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
        self.server_config
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(name.clone(), entry);
        {
            let mut index = self
                .tool_index
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            index.retain(|_, (server, _)| server != &name);
            index.extend(indexed);
        }
        {
            let mut tools = self
                .tools
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            tools.retain(|(tool_name, _)| {
                self.native_tools.contains_key(tool_name)
                    || self
                        .tool_index
                        .read()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .get(tool_name)
                        .is_some_and(|(server, _)| server != &name)
            });
            tools.extend(discovered);
        }
        crate::uar::telemetry::metrics::set_mcp_server_status(&name, true);
        Ok(())
    }

    /// Remove one MCP server and every tool it contributed.
    pub fn remove_server(&self, name: &str) -> bool {
        let removed = self
            .services
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(name)
            .is_some();
        self.server_config
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(name);
        let removed_names = {
            let mut index = self
                .tool_index
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let names = index
                .iter()
                .filter(|(_, (server, _))| server == name)
                .map(|(tool, _)| tool.clone())
                .collect::<std::collections::HashSet<_>>();
            index.retain(|_, (server, _)| server != name);
            names
        };
        self.tools
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retain(|(tool, _)| !removed_names.contains(tool));
        crate::uar::telemetry::metrics::set_mcp_server_status(name, false);
        removed
    }

    /// Cancel every connected MCP transport and wait for its cleanup to finish.
    pub(crate) async fn shutdown(&self) {
        self.shutting_down.store(true, Ordering::Release);
        let services = self
            .services
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .map(|(name, slot)| {
                let service = begin_service_shutdown(slot);
                (name.clone(), Arc::clone(slot), service)
            })
            .collect::<Vec<_>>();

        // Initiate every close before awaiting any one transport. A slow MCP
        // server must not delay the start of cleanup for its peers.
        for (_, _, service) in &services {
            service.cancellation_token().cancel();
        }

        for (name, slot, service) in services {
            while !service_shutdown_complete(&slot, &service) {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            crate::uar::telemetry::metrics::set_mcp_server_status(&name, false);
            tracing::info!(server = %name, "MCP transport shut down");
        }
    }

    /// Merge another registry into this one, returning a new registry.
    /// This is used to combine global tools with skill-specific tools.
    #[must_use]
    pub fn merge(&self, other: &McpRegistry) -> Self {
        let mut services = self
            .services
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        services.extend(
            other
                .services
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
        );

        let mut server_config = self.server_entries();
        server_config.extend(other.server_entries());

        let mut tool_index = self
            .tool_index
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        tool_index.extend(
            other
                .tool_index
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
        );

        let mut tools = self.tools();
        tools.extend(other.tools());

        let mut native_tools = (*self.native_tools).clone();
        native_tools.extend((*other.native_tools).clone());

        Self {
            services: Arc::new(RwLock::new(services)),
            shutting_down: Arc::new(AtomicBool::new(
                self.shutting_down.load(Ordering::Acquire)
                    || other.shutting_down.load(Ordering::Acquire),
            )),
            server_config: Arc::new(RwLock::new(server_config)),
            tool_index: Arc::new(RwLock::new(tool_index)),
            tools: Arc::new(RwLock::new(tools)),
            native_tools: Arc::new(native_tools),
        }
    }

    /// Return a registry narrowed to the server and tool sets allowed by the
    /// resolved run policy.
    ///
    /// `None` means "all currently registered values" while `Some(empty)`
    /// means "none". Native tools are governed by `allowed_tools` because they
    /// do not belong to an MCP server.
    #[must_use]
    pub fn filtered(
        &self,
        allowed_servers: Option<&std::collections::HashSet<String>>,
        allowed_tools: Option<&std::collections::HashSet<String>>,
    ) -> Self {
        let server_allowed =
            |server: &str| allowed_servers.is_none_or(|allowed| allowed.contains(server));
        let tool_allowed = |tool: &str| allowed_tools.is_none_or(|allowed| allowed.contains(tool));

        let services = self
            .services
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .filter(|(name, _)| server_allowed(name))
            .map(|(name, service)| (name.clone(), Arc::clone(service)))
            .collect();
        let server_config = self
            .server_entries()
            .into_iter()
            .filter(|(name, _)| server_allowed(name))
            .map(|(name, config)| (name.clone(), config.clone()))
            .collect();
        let tool_index = self
            .tool_index
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .filter(|(name, (server, _))| server_allowed(server) && tool_allowed(name))
            .map(|(name, target)| (name.clone(), target.clone()))
            .collect::<HashMap<_, _>>();
        let native_tools = self
            .native_tools
            .iter()
            .filter(|(name, _)| tool_allowed(name))
            .map(|(name, tool)| (name.clone(), Arc::clone(tool)))
            .collect::<HashMap<_, _>>();
        let tools = self
            .tools()
            .into_iter()
            .filter(|(name, _)| {
                tool_allowed(name)
                    && (native_tools.contains_key(name) || tool_index.contains_key(name))
            })
            .collect();

        Self {
            services: Arc::new(RwLock::new(services)),
            shutting_down: Arc::clone(&self.shutting_down),
            server_config: Arc::new(RwLock::new(server_config)),
            tool_index: Arc::new(RwLock::new(tool_index)),
            tools: Arc::new(RwLock::new(tools)),
            native_tools: Arc::new(native_tools),
        }
    }

    #[must_use]
    pub fn with_native_tool(self, tool: Arc<dyn NativeTool>) -> Self {
        let ns_name = Self::sanitize_tool_name(&format!("native__{}", tool.name()));

        let mut tools = self.tools();
        let mcp_tool = Tool::new(
            tool.name().to_string(),
            tool.description().to_string(),
            tool.schema()
                .as_object()
                .unwrap_or(&serde_json::Map::new())
                .clone(),
        );
        tools.push((ns_name.clone(), mcp_tool));

        let mut native_tools = (*self.native_tools).clone();
        native_tools.insert(ns_name, tool);

        Self {
            services: self.services, // Keep ref
            shutting_down: self.shutting_down,
            server_config: self.server_config,
            tool_index: self.tool_index, // Keep ref
            tools: Arc::new(RwLock::new(tools)),
            native_tools: Arc::new(native_tools),
        }
    }

    pub fn openai_tools_json(&self) -> Vec<serde_json::Value> {
        self.tools()
            .into_iter()
            .map(|(ns_name, t)| {
                // rmcp Tool uses input_schema as an Arc<JsonObject>; convert to serde_json.
                let params = serde_json::to_value(&*t.input_schema)
                    .unwrap_or_else(|_| serde_json::json!({"type":"object","properties":{}}));

                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": ns_name,
                        "description": t.description.as_deref().unwrap_or(""),
                        "parameters": params
                    }
                })
            })
            .collect()
    }

    /// Execute a namespaced tool, e.g. "`time__now`" or "`tavily__search`".
    #[tracing::instrument(
        name = "tool.call",
        skip(self, arguments),
        fields(tool = %namespaced_tool),
    )]
    pub async fn call_namespaced_tool(
        &self,
        namespaced_tool: &str,
        arguments: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        if namespaced_tool == "mirror" {
            return Ok(arguments);
        }

        if let Some(tool) = self.native_tools.get(namespaced_tool) {
            // Native tools were the one execution path with no time bound;
            // MCP tools have had one (30s, below) all along. A native tool
            // that never returns held the whole run open with no way to
            // recover, which is the same class of failure as a hung MCP
            // handshake — just later in the turn.
            return match tokio::time::timeout(MCP_TOOL_CALL_TIMEOUT, tool.call(arguments)).await {
                Ok(result) => result,
                Err(_elapsed) => Err(anyhow!(
                    "native tool {namespaced_tool} timed out after {}s",
                    MCP_TOOL_CALL_TIMEOUT.as_secs()
                )),
            };
        }

        // 1. Lookup server + raw_tool_name
        let (server_name, raw_tool_name) = self
            .tool_index
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(namespaced_tool)
            .ok_or_else(|| anyhow!("unknown tool: {namespaced_tool}"))?
            .clone();

        if server_name == "test" {
            return Ok(serde_json::json!({
                "result": format!("executed test tool {} with args {:?}", raw_tool_name, arguments)
            }));
        }

        // 2. Lookup service
        let service_slot = self
            .services
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&server_name)
            .cloned()
            .ok_or_else(|| anyhow!("missing server handle: {server_name}"))?;
        let service = current_service(&service_slot);

        // 3. Call tool
        let args_obj = arguments.as_object().cloned();
        let input_size = serde_json::to_string(&arguments)
            .map(|s| s.len())
            .unwrap_or(0);
        let start = std::time::Instant::now();
        // rmcp 1.8: CallToolRequestParams is #[non_exhaustive] -- use the
        // provided new()/with_arguments() builder instead of a struct literal.
        let mut call_params = CallToolRequestParams::new(raw_tool_name.clone());
        if let Some(args) = args_obj {
            call_params = call_params.with_arguments(args);
        }
        let res = tokio::time::timeout(MCP_TOOL_CALL_TIMEOUT, service.call_tool(call_params))
            .await
            .map_err(|_| {
                anyhow!("tools/call timed out after 30 seconds for {server_name}::{raw_tool_name}")
            })
            .and_then(|result| result.map_err(anyhow::Error::from));
        let duration = start.elapsed();
        let success = res.is_ok();

        let output_size = if let Ok(ref r) = res {
            serde_json::to_string(r).map(|s| s.len()).unwrap_or(0)
        } else {
            0
        };

        tracing::info!(
            target: "mcp.tool.execution",
            tool = %namespaced_tool,
            duration_ms = %duration.as_millis(),
            input_size_bytes = %input_size,
            output_size_bytes = %output_size,
            success = %success,
            "MCP tool executed"
        );

        // Emit raw metrics via the metrics crate
        #[cfg(feature = "telemetry")]
        metrics::counter!("mcp_tool_calls_total", "tool" => namespaced_tool.to_string(), "success" => success.to_string()).increment(1);
        #[allow(clippy::cast_precision_loss)]
        #[cfg(feature = "telemetry")]
        metrics::histogram!("mcp_tool_duration_ms", "tool" => namespaced_tool.to_string())
            .record(duration.as_millis() as f64);

        // Record normalized tool call metrics via telemetry module
        crate::uar::telemetry::metrics::record_tool_call(namespaced_tool, success);

        let res = match res {
            Ok(result) => result,
            Err(error) => {
                crate::uar::telemetry::metrics::set_mcp_server_status(&server_name, false);
                // Re-establish the transport for future calls. Never replay the
                // failed call: it may have completed remotely before transport loss.
                if let Some((reconnect_entry, generation)) = begin_reconnect(&service_slot) {
                    match connect_server(&server_name, &reconnect_entry).await {
                        Ok(replacement) => {
                            match install_reconnected_service(
                                &service_slot,
                                generation,
                                replacement,
                            ) {
                                Ok(()) => {
                                    crate::uar::telemetry::metrics::set_mcp_server_status(
                                        &server_name,
                                        true,
                                    );
                                    tracing::info!(server = %server_name, "MCP transport reconnected for subsequent calls");
                                }
                                Err(replacement) => {
                                    let _ = replacement.cancel().await;
                                    finish_reconnect(&service_slot);
                                    tracing::info!(server = %server_name, "Discarded MCP reconnect because shutdown began or a newer server configuration was installed");
                                }
                            }
                        }
                        Err(reconnect_error) => {
                            finish_reconnect(&service_slot);
                            tracing::warn!(
                                server = %server_name,
                                error = %reconnect_error,
                                "MCP transport reconnect failed; a later call may retry"
                            );
                        }
                    }
                }
                return Err(error).with_context(|| {
                    format!("tools/call failed for {server_name}::{raw_tool_name}")
                });
            }
        };

        // 4. Return content (simplified)
        Ok(serde_json::to_value(res)?)
    }
}

fn resolve_mcp_config_path(path: &str) -> PathBuf {
    if let Ok(env_path) = std::env::var("MCP_CONFIG_PATH") {
        let candidate = PathBuf::from(env_path);
        if candidate.exists() {
            return candidate;
        }
    }

    let path_buf = PathBuf::from(path);
    if path_buf.is_relative()
        && let Ok(dir) = std::env::var("MCP_CONFIG_DIR")
    {
        let candidate = PathBuf::from(dir).join(&path_buf);
        if candidate.exists() {
            return candidate;
        }
    }

    path_buf
}

fn resolve_mcp_command(command: &str) -> PathBuf {
    let path = Path::new(command);
    if path.is_absolute() {
        return path.to_path_buf();
    }

    if let Ok(dir) = std::env::var("MCP_SERVER_DIR") {
        let candidate = PathBuf::from(dir).join(path);
        if candidate.exists() {
            return candidate;
        }
    }

    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::config::McpConfig;
    use serde::Deserialize;
    use std::collections::HashSet;

    const FIXTURE: &str = r#"
import json
import os
import sys

trace_path = sys.argv[1]

for line in sys.stdin:
    try:
        request = json.loads(line)
    except json.JSONDecodeError:
        continue
    request_id = request.get("id")
    if request_id is None:
        continue
    method = request.get("method")
    if method == "initialize":
        result = {
            "protocolVersion": request.get("params", {}).get("protocolVersion", "2024-11-05"),
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "uar-registry-test", "version": "1.0.0"},
        }
    elif method == "tools/list":
        result = {"tools": [{
            "name": "echo",
            "description": "MCP registry recovery fixture",
            "inputSchema": {
                "type": "object",
                "properties": {"mode": {"type": "string"}},
                "required": ["mode"],
            },
        }]}
    elif method == "tools/call":
        mode = request.get("params", {}).get("arguments", {}).get("mode", "echo")
        with open(trace_path, "a", encoding="utf-8") as trace:
            trace.write(json.dumps({"pid": os.getpid(), "mode": mode}) + "\n")
            trace.flush()
            os.fsync(trace.fileno())
        if mode == "crash":
            sys.exit(23)
        result = {"content": [{"type": "text", "text": f"mcp-{mode}"}], "isError": False}
    else:
        print(json.dumps({
            "jsonrpc": "2.0",
            "id": request_id,
            "error": {"code": -32601, "message": "method not found"},
        }), flush=True)
        continue
    print(json.dumps({"jsonrpc": "2.0", "id": request_id, "result": result}), flush=True)

with open(trace_path, "a", encoding="utf-8") as trace:
    trace.write(json.dumps({"pid": os.getpid(), "mode": "stdin_closed"}) + "\n")
    trace.flush()
    os.fsync(trace.fileno())
"#;

    #[derive(Deserialize)]
    struct TraceEntry {
        pid: u32,
        mode: String,
    }

    fn fixture_config() -> (tempfile::TempDir, PathBuf, McpConfig) {
        let fixture_dir = tempfile::tempdir().expect("create MCP fixture directory");
        let script = fixture_dir.path().join("mock-mcp.py");
        let trace = fixture_dir.path().join("trace.jsonl");
        std::fs::write(&script, FIXTURE).expect("write MCP fixture");
        let config = McpConfig {
            mcp_servers: HashMap::from([(
                "resilience".to_string(),
                McpServerEntry::Stdio {
                    command: "python3".to_string(),
                    args: vec![script.display().to_string(), trace.display().to_string()],
                    env: HashMap::new(),
                    sandboxed: false,
                },
            )]),
        };
        (fixture_dir, trace, config)
    }

    fn trace_entries(path: &Path) -> Vec<TraceEntry> {
        std::fs::read_to_string(path)
            .expect("read MCP trace")
            .lines()
            .map(|line| serde_json::from_str(line).expect("parse MCP trace entry"))
            .collect()
    }

    #[tokio::test]
    async fn reconnect_replacement_is_shared_without_widening_filtered_views() {
        let (_fixture_dir, trace, config) = fixture_config();
        let registry = McpRegistry::from_config(&config)
            .await
            .expect("connect MCP fixture");
        let allowed_servers = HashSet::from(["resilience".to_string()]);
        let allowed_tools = HashSet::from(["resilience__echo".to_string()]);
        let first_view = registry.filtered(Some(&allowed_servers), Some(&allowed_tools));
        let second_view = registry.filtered(Some(&allowed_servers), Some(&allowed_tools));
        let merged_view = McpRegistry::empty().merge(&registry);
        let denied_server_view = registry.filtered(Some(&HashSet::new()), None);
        let denied_tool_view = registry.filtered(Some(&allowed_servers), Some(&HashSet::new()));

        first_view
            .call_namespaced_tool("resilience__echo", serde_json::json!({"mode": "crash"}))
            .await
            .expect_err("crashed MCP call must fail without replay");

        second_view
            .call_namespaced_tool("resilience__echo", serde_json::json!({"mode": "echo"}))
            .await
            .expect("independent filtered view must use replacement service");
        merged_view
            .call_namespaced_tool("resilience__echo", serde_json::json!({"mode": "echo"}))
            .await
            .expect("pre-existing merged view must use replacement service");

        assert!(denied_server_view.server_names().is_empty());
        assert!(denied_server_view.tools().is_empty());
        assert!(denied_tool_view.tools().is_empty());
        denied_tool_view
            .call_namespaced_tool("resilience__echo", serde_json::json!({"mode": "echo"}))
            .await
            .expect_err("replacement must not restore a policy-excluded tool");

        registry.shutdown().await;

        let trace = trace_entries(&trace);
        assert_eq!(
            trace.iter().filter(|entry| entry.mode == "crash").count(),
            1,
            "failed call must execute exactly once"
        );
        let calls = trace
            .iter()
            .filter(|entry| entry.mode != "stdin_closed")
            .collect::<Vec<_>>();
        assert_eq!(calls.len(), 3);
        assert_ne!(calls[0].pid, calls[1].pid);
        assert_eq!(calls[1].pid, calls[2].pid);
    }

    #[tokio::test]
    async fn upsert_reconnect_uses_new_config_in_an_existing_filtered_view() {
        let (fixture_dir, original_trace, config) = fixture_config();
        let registry = McpRegistry::from_config(&config)
            .await
            .expect("connect MCP fixture");
        let allowed_servers = HashSet::from(["resilience".to_string()]);
        let allowed_tools = HashSet::from(["resilience__echo".to_string()]);
        let filtered = registry.filtered(Some(&allowed_servers), Some(&allowed_tools));

        filtered
            .call_namespaced_tool("resilience__echo", serde_json::json!({"mode": "before"}))
            .await
            .expect("initial service must respond");
        let replacement_trace = fixture_dir.path().join("replacement-trace.jsonl");
        let replacement_entry = match config
            .mcp_servers
            .get("resilience")
            .expect("fixture server config")
            .clone()
        {
            McpServerEntry::Stdio {
                command,
                mut args,
                env,
                sandboxed,
            } => {
                *args.last_mut().expect("fixture trace argument") =
                    replacement_trace.display().to_string();
                McpServerEntry::Stdio {
                    command,
                    args,
                    env,
                    sandboxed,
                }
            }
            McpServerEntry::RemoteHttp { .. } => panic!("fixture must use stdio"),
        };
        registry
            .upsert_server("resilience".to_string(), replacement_entry)
            .await
            .expect("upsert replacement service");
        filtered
            .call_namespaced_tool("resilience__echo", serde_json::json!({"mode": "crash"}))
            .await
            .expect_err("post-upsert crash must fail without replay");
        filtered
            .call_namespaced_tool("resilience__echo", serde_json::json!({"mode": "after"}))
            .await
            .expect("existing filtered view must reconnect with the upserted config");

        registry.shutdown().await;

        let original_trace = trace_entries(&original_trace);
        let original_calls = original_trace
            .iter()
            .filter(|entry| entry.mode != "stdin_closed")
            .collect::<Vec<_>>();
        assert_eq!(original_calls.len(), 1);
        assert_eq!(original_calls[0].mode, "before");

        let replacement_trace = trace_entries(&replacement_trace);
        let replacement_calls = replacement_trace
            .iter()
            .filter(|entry| entry.mode != "stdin_closed")
            .collect::<Vec<_>>();
        assert_eq!(replacement_calls.len(), 2);
        assert_eq!(replacement_calls[0].mode, "crash");
        assert_eq!(replacement_calls[1].mode, "after");
        assert_ne!(replacement_calls[0].pid, replacement_calls[1].pid);
    }

    #[tokio::test]
    async fn shutdown_waits_for_stdio_eof_and_blocks_filtered_view_reconnect() {
        let (_fixture_dir, trace, config) = fixture_config();
        let registry = McpRegistry::from_config(&config)
            .await
            .expect("connect MCP fixture");
        let allowed_servers = HashSet::from(["resilience".to_string()]);
        let allowed_tools = HashSet::from(["resilience__echo".to_string()]);
        let filtered = registry.filtered(Some(&allowed_servers), Some(&allowed_tools));

        registry.shutdown().await;

        filtered
            .call_namespaced_tool("resilience__echo", serde_json::json!({"mode": "after"}))
            .await
            .expect_err("a pre-existing filtered view must not reconnect after shutdown");
        let trace = trace_entries(&trace);
        assert_eq!(trace.len(), 1);
        assert_eq!(trace[0].mode, "stdin_closed");
    }

    #[tokio::test]
    async fn shutdown_blocks_new_server_upsert() {
        let (_fixture_dir, trace, config) = fixture_config();
        let registry = McpRegistry::from_config(&config)
            .await
            .expect("connect MCP fixture");
        registry.shutdown().await;

        registry
            .upsert_server(
                "replacement".to_string(),
                config.mcp_servers["resilience"].clone(),
            )
            .await
            .expect_err("shutdown must reject a newly configured MCP transport");

        let trace = trace_entries(&trace);
        assert_eq!(trace.len(), 1);
        assert_eq!(trace[0].mode, "stdin_closed");
    }
}
