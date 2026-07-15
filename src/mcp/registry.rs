use crate::mcp::config::{McpServerEntry, expand_env_map, expand_env_placeholders, load_mcp_config};
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
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::process::Command;
use url::Url;

#[async_trait]
pub trait NativeTool: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn schema(&self) -> serde_json::Value;
    async fn call(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value>;
}

type DynClientService = rmcp::service::RunningService<
    rmcp::service::RoleClient,
    Box<dyn rmcp::service::DynService<rmcp::service::RoleClient>>,
>;

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
            ().into_dyn()
                .serve(transport)
                .await
                .with_context(|| format!("failed to connect stdio MCP server '{name}'"))
        }
        McpServerEntry::RemoteHttp { url, env } => {
            let env = expand_env_map(env);
            let endpoint = resolve_remote_http_url(name, url, &env)?;
            ().into_dyn()
                .serve(StreamableHttpClientTransport::from_uri(
                    endpoint.to_string(),
                ))
                .await
                .with_context(|| format!("failed to connect remote MCP server '{name}'"))
        }
    }
}

/// Expands `${VAR}` placeholders in a remote MCP server's `url` (per the
/// process environment, same as its `env` map). Tavily's config style embeds
/// `${TAVILY_API_KEY}` directly in the URL and additionally declares it in
/// `env`; if the placeholder is still present after process-env expansion,
/// this substitutes it from the entry's own `env` map, matching that
/// convention. Any other `RemoteHttp` entry (e.g. `surreal_memory`, which has
/// no `TAVILY_API_KEY` at all) is left alone -- requiring a Tavily-specific
/// key for every remote server was the bug that took down the whole registry
/// whenever a non-Tavily entry was present.
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
    Url::parse(&expanded).with_context(|| format!("invalid url for remote MCP '{name}': {expanded}"))
}

#[derive(Clone)]
pub struct McpRegistry {
    services: Arc<RwLock<HashMap<String, Arc<DynClientService>>>>,
    server_config: Arc<HashMap<String, McpServerEntry>>,
    // namespaced_tool_name -> (server_name, tool_name)
    tool_index: Arc<HashMap<String, (String, String)>>,
    tools: Arc<Vec<(String, Tool)>>, // (namespaced_name, Tool)
    // namespaced_tool_name -> NativeTool
    native_tools: Arc<HashMap<String, Arc<dyn NativeTool>>>,
}

impl std::fmt::Debug for McpRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpRegistry")
            .field("tool_count", &self.tools.len())
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
            server_config: Arc::new(HashMap::new()),
            tool_index: Arc::new(HashMap::new()),
            tools: Arc::new(Vec::new()),
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
        let mut services: HashMap<String, Arc<DynClientService>> = HashMap::new();

        for (name, entry) in &cfg.mcp_servers {
            let svc = match Self::connect_configured_server(name, entry).await {
                Ok(svc) => svc,
                Err(e) => {
                    crate::uar::telemetry::metrics::set_mcp_server_status(name, false);
                    tracing::warn!(server = %name, error = ?e, "MCP server failed to connect; skipping it, registry continues without it");
                    continue;
                }
            };
            crate::uar::telemetry::metrics::set_mcp_server_status(name, true);
            services.insert(name.clone(), Arc::new(svc));
        }

        // 2) list tools + build index
        let mut all_tools: Vec<(String, Tool)> = Vec::new();
        let mut tool_index: HashMap<String, (String, String)> = HashMap::new();

        for (server_name, svc) in &services {
            // list_tools exists on the rmcp running service in examples
            let result = match svc.list_tools(Default::default()).await {
                Ok(result) => result,
                Err(e) => {
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
            server_config: Arc::new(cfg.mcp_servers.clone()),
            tool_index: Arc::new(tool_index),
            tools: Arc::new(all_tools),
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
                // store as dyn to keep a homogeneous collection
                ().into_dyn()
                    .serve(transport)
                    .await
                    .with_context(|| format!("failed to connect stdio MCP server '{name}'"))
            }

            McpServerEntry::RemoteHttp { url, env } => {
                let env = expand_env_map(env);
                let endpoint = resolve_remote_http_url(name, url, &env)?;
                ().into_dyn()
                    .serve(StreamableHttpClientTransport::from_uri(
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
            server_config: Arc::new(HashMap::new()),
            tool_index: Arc::new(HashMap::new()),
            tools: Arc::new(Vec::new()),
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
            server_config: Arc::new(HashMap::new()),
            tool_index: Arc::new(tool_index),
            tools: Arc::new(tools),
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
    pub fn tools(&self) -> &[(String, Tool)] {
        &self.tools
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
        self.tool_index.get(namespaced_name).cloned()
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

        let mut server_config = (*self.server_config).clone();
        server_config.extend((*other.server_config).clone());

        let mut tool_index = (*self.tool_index).clone();
        tool_index.extend((*other.tool_index).clone());

        let mut tools = (*self.tools).clone();
        tools.extend((*other.tools).clone());

        let mut native_tools = (*self.native_tools).clone();
        native_tools.extend((*other.native_tools).clone());

        Self {
            services: Arc::new(RwLock::new(services)),
            server_config: Arc::new(server_config),
            tool_index: Arc::new(tool_index),
            tools: Arc::new(tools),
            native_tools: Arc::new(native_tools),
        }
    }

    #[must_use]
    pub fn with_native_tool(self, tool: Arc<dyn NativeTool>) -> Self {
        let ns_name = Self::sanitize_tool_name(&format!("native__{}", tool.name()));

        let mut tools = (*self.tools).clone();
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
            server_config: self.server_config,
            tool_index: self.tool_index, // Keep ref
            tools: Arc::new(tools),
            native_tools: Arc::new(native_tools),
        }
    }

    pub fn openai_tools_json(&self) -> Vec<serde_json::Value> {
        self.tools
            .iter()
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
            return tool.call(arguments).await;
        }

        // 1. Lookup server + raw_tool_name
        let (server_name, raw_tool_name) = self
            .tool_index
            .get(namespaced_tool)
            .ok_or_else(|| anyhow!("unknown tool: {namespaced_tool}"))?
            .clone();

        if server_name == "test" {
            return Ok(serde_json::json!({
                "result": format!("executed test tool {} with args {:?}", raw_tool_name, arguments)
            }));
        }

        // 2. Lookup service
        let service = self
            .services
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&server_name)
            .cloned()
            .ok_or_else(|| anyhow!("missing server handle: {server_name}"))?;

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
        let res = tokio::time::timeout(Duration::from_secs(30), service.call_tool(call_params))
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
                if let Some(entry) = self.server_config.get(&server_name) {
                    match connect_server(&server_name, entry).await {
                        Ok(replacement) => {
                            self.services
                                .write()
                                .unwrap_or_else(std::sync::PoisonError::into_inner)
                                .insert(server_name.clone(), Arc::new(replacement));
                            crate::uar::telemetry::metrics::set_mcp_server_status(
                                &server_name,
                                true,
                            );
                            tracing::info!(server = %server_name, "MCP transport reconnected for subsequent calls");
                        }
                        Err(reconnect_error) => tracing::warn!(
                            server = %server_name,
                            error = %reconnect_error,
                            "MCP transport reconnect failed; a later call may retry"
                        ),
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
