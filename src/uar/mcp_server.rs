//! UAR runtime MCP server — exposes UAR agent capabilities as MCP tools.
//!
//! Any MCP-aware system (Claude Desktop, LangGraph, AutoGen, etc.) can invoke UAR
//! agents, inspect compiled agent registries, start runs, and trigger compilation
//! without implementing the UAR REST API directly.
//!
//! ## Exposed Tools
//!
//! | Tool | Description |
//! |------|-------------|
//! | `uar_list_agents` | List all compiled agents in the registry |
//! | `uar_create_run` | Create a new agent run and return the run ID + SSE URL |
//! | `uar_get_run_status` | Get the status of an active or completed run |
//! | `uar_list_skills` | List all registered native skills |
//! | `uar_compile_spec` | Compile a UAR-AGENT-MD Markdown document |
//!
//! ## HTTP exposure
//!
//! Mount the router at `/mcp/uar`:
//!
//! ```rust,ignore
//! let router = uar_mcp_router(Arc::clone(&run_manager), Arc::clone(&native_skills), persistence.clone());
//! app = app.nest("/mcp/uar", router);
//! ```

use std::{collections::BTreeMap, sync::Arc};

use anyhow::Result;
use axum::Router;
use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{
        router::tool::ToolRouter, tool::Extension as McpExtension, wrapper::Parameters,
    },
    model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    },
};

use crate::uar::{
    compiler::{
        collaboration::{CollaborationCatalogService, CollaborationError},
        pipeline,
    },
    domain::collaboration::{BindingCommandRequest, PackageSourceRequest},
    persistence::PersistenceLayer,
    runtime::{
        actor::messages::ActorOwner, manager::RunManager, native_skill::NativeSkillRegistry,
        turn::RunExecutionRequest,
    },
    security::claims::UserContext,
};

// ── Helper utilities ──────────────────────────────────────────────────────────

fn ok_json<T: serde::Serialize>(value: &T) -> CallToolResult {
    let json =
        serde_json::to_string_pretty(value).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"));
    CallToolResult::success(vec![ContentBlock::text(json)])
}

fn err_mcp(e: impl std::fmt::Display) -> McpError {
    McpError::invalid_params(e.to_string(), None)
}

fn verified_owner(parts: &axum::http::request::Parts) -> Result<ActorOwner, McpError> {
    let user = parts
        .extensions
        .get::<UserContext>()
        .ok_or_else(|| McpError::invalid_params("verified user context required", None))?;
    ActorOwner::from_verified_context(user)
        .map_err(|_| McpError::invalid_params("verified user context required", None))
}

fn collaboration_owner(parts: &axum::http::request::Parts) -> Result<String, McpError> {
    let user = parts
        .extensions
        .get::<UserContext>()
        .ok_or_else(|| McpError::invalid_params("verified user context required", None))?;
    crate::uar::api::user_settings::principal_storage_key(user)
        .ok_or_else(|| McpError::invalid_params("verified user context required", None))
}

fn collaboration_workspace(parts: &axum::http::request::Parts) -> Result<String, McpError> {
    parts
        .headers
        .get("x-uar-workspace-id")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| McpError::invalid_params("verified workspace context required", None))
}

fn collaboration_mcp_error(error: CollaborationError) -> McpError {
    match error {
        CollaborationError::Invalid(detail) => McpError::invalid_params(detail, None),
        CollaborationError::NotFound(_) => {
            McpError::invalid_params("collaboration resource not found", None)
        }
        CollaborationError::Conflict(detail) => McpError::invalid_params(detail, None),
        CollaborationError::Storage(error) => {
            tracing::error!(%error, "collaboration catalog persistence failed");
            McpError::internal_error("collaboration catalog unavailable", None)
        }
    }
}

// ── Parameter structs ─────────────────────────────────────────────────────────

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CreateRunParams {
    /// ID of the compiled agent to use (from `uar_list_agents`).
    pub agent_id: String,
    /// The user message / input to the agent.
    pub input: String,
    /// Optional session ID for conversation continuity.
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RunIdParams {
    /// The run ID returned by `uar_create_run`.
    pub run_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CompileSpecParams {
    /// The complete UAR-AGENT-MD Markdown document to compile.
    pub spec: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CollaborationPackageParams {
    pub command_id: String,
    #[serde(default)]
    pub expected_catalog_revision: Option<u64>,
    pub manifest: String,
    pub files: BTreeMap<String, String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CollaborationBindingParams {
    pub command_id: String,
    #[serde(default)]
    pub expected_revision: Option<u64>,
    pub binding: serde_json::Value,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CollaborationPackageStatusParams {
    pub id: String,
    pub version: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CollaborationBindingStatusParams {
    pub id: String,
}

// ── MCP server handler ────────────────────────────────────────────────────────

#[derive(Clone)]
struct UarRuntimeMcpServer {
    run_manager: Arc<RunManager>,
    native_skills: Arc<NativeSkillRegistry>,
    collaboration_catalog: Arc<CollaborationCatalogService>,
    #[expect(
        dead_code,
        reason = "rmcp's generated tool handler retains this router for runtime dispatch"
    )]
    tool_router: ToolRouter<UarRuntimeMcpServer>,
}

impl std::fmt::Debug for UarRuntimeMcpServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UarRuntimeMcpServer")
            .finish_non_exhaustive()
    }
}

impl UarRuntimeMcpServer {
    fn new(
        run_manager: Arc<RunManager>,
        native_skills: Arc<NativeSkillRegistry>,
        collaboration_catalog: Arc<CollaborationCatalogService>,
    ) -> Self {
        Self {
            run_manager,
            native_skills,
            collaboration_catalog,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl UarRuntimeMcpServer {
    /// List all compiled agents registered in the UAR agent registry.
    ///
    /// Returns an array of agent summaries including ID, title, description, version,
    /// and available tools. Use the `id` field with `uar_create_run` to start a run.
    #[tool(description = "List all compiled agents in the UAR registry")]
    async fn uar_list_agents(&self) -> Result<CallToolResult, McpError> {
        let agents = self
            .run_manager
            .list_registered_agents()
            .await
            .map_err(err_mcp)?;

        let summaries: Vec<serde_json::Value> = agents
            .iter()
            .map(|a| {
                serde_json::json!({
                    "id": a.id,
                    "title": a.metadata.title,
                    "description": a.metadata.description,
                    "version": a.version,
                    "kind": a.kind,
                })
            })
            .collect();

        Ok(ok_json(&summaries))
    }

    /// Create a new UAR agent run.
    ///
    /// Looks up the compiled agent by `agent_id`, starts an asynchronous run, and
    /// returns the `run_id` and the SSE stream URL where token/tool events can be
    /// consumed. The run executes asynchronously — use `uar_get_run_status` to poll
    /// completion or connect to the SSE URL for real-time events.
    #[tool(description = "Create a new agent run and return the run_id and SSE stream URL")]
    async fn uar_create_run(
        &self,
        Parameters(p): Parameters<CreateRunParams>,
        McpExtension(parts): McpExtension<axum::http::request::Parts>,
    ) -> Result<CallToolResult, McpError> {
        let owner = verified_owner(&parts)?;
        let agent = self
            .run_manager
            .resolve_registered_agent(&p.agent_id)
            .await
            .map_err(err_mcp)?;

        let mut request = RunExecutionRequest::new(agent, p.input).with_verified_owner(owner);
        request.session_id = p.session_id;
        let run_id = self.run_manager.execute_request(request).await;

        let response = serde_json::json!({
            "run_id": run_id,
            "sse_url": format!("/api/uar/runs/{}/stream", run_id),
            "status_tool": "uar_get_run_status",
        });

        Ok(ok_json(&response))
    }

    /// Get the status and metadata of an active or recently completed run.
    ///
    /// Returns the run ID, current status (pending / running / completed / failed),
    /// and the SSE stream URL.
    #[tool(description = "Get the status of a UAR agent run by run_id")]
    async fn uar_get_run_status(
        &self,
        Parameters(p): Parameters<RunIdParams>,
        McpExtension(parts): McpExtension<axum::http::request::Parts>,
    ) -> Result<CallToolResult, McpError> {
        let owner = verified_owner(&parts)?;
        let run = self
            .run_manager
            .get_run_for_owner(&owner, &p.run_id)
            .await
            .ok_or_else(|| {
                McpError::invalid_params(format!("run '{}' not found", p.run_id), None)
            })?;

        let status = serde_json::json!({
            "run_id": run.run_id,
            "status": format!("{:?}", run.status),
            "agent_id": run.agent_id,
            "conversation_id": run.conversation_id,
            "sse_url": format!("/api/uar/runs/{}/stream", run.run_id),
        });

        Ok(ok_json(&status))
    }

    /// List all native skills registered in the UAR skill registry.
    ///
    /// Native skills are high-performance in-process tools (e.g., the PMPO compiler,
    /// memory tools, document ingestion). Use this to discover available capabilities.
    #[tool(description = "List all registered UAR native skills")]
    async fn uar_list_skills(&self) -> Result<CallToolResult, McpError> {
        let tools_json = self.native_skills.openai_tools_json().await;

        let skills: Vec<serde_json::Value> = tools_json
            .iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.get("function").and_then(|f| f.get("name")).unwrap_or(&serde_json::Value::Null),
                    "description": t.get("function").and_then(|f| f.get("description")).unwrap_or(&serde_json::Value::Null),
                })
            })
            .collect();

        Ok(ok_json(&skills))
    }

    /// Compile a UAR-AGENT-MD Markdown document into a signed agent artifact.
    ///
    /// Accepts a complete UAR-AGENT-MD document as a Markdown string, runs the
    /// 8-stage PMPO compiler pipeline, and returns the compiled descriptor and
    /// signature on success, or structured error details on failure.
    #[tool(description = "Compile a UAR-AGENT-MD Markdown document into a signed agent artifact")]
    async fn uar_compile_spec(
        &self,
        Parameters(p): Parameters<CompileSpecParams>,
    ) -> Result<CallToolResult, McpError> {
        use crate::uar::compiler::{
            parser,
            registries::{InMemoryEndpointRegistry, InMemorySchemaRegistry},
            signing::LocalKeyProvider,
        };

        let ir = parser::parse(&p.spec)
            .map_err(|e| McpError::invalid_params(format!("parse failed: {e}"), None))?;

        let key_provider = Arc::new(
            LocalKeyProvider::new(None)
                .map_err(|e| McpError::invalid_params(format!("key init failed: {e}"), None))?,
        );
        let schema_registry = Arc::new(InMemorySchemaRegistry::default());
        let endpoint_registry = Arc::new(InMemoryEndpointRegistry::default());

        let output = pipeline::compile(ir, schema_registry, endpoint_registry, key_provider)
            .await
            .map_err(|e| McpError::invalid_params(format!("compilation failed: {e}"), None))?;

        let result = serde_json::json!({
            "agent_id": output.descriptor.agent_id,
            "version": output.descriptor.version,
            "signature": output.signature,
            "descriptor": output.descriptor,
            "report": output.report,
        });

        Ok(ok_json(&result))
    }

    /// Report the I1 collaboration capabilities without claiming team execution.
    #[tool(description = "Report UAR collaboration package and binding capabilities")]
    async fn uar_collaboration_capabilities(
        &self,
        McpExtension(parts): McpExtension<axum::http::request::Parts>,
    ) -> Result<CallToolResult, McpError> {
        collaboration_owner(&parts)?;
        Ok(ok_json(&serde_json::json!({
            "capabilities": [
                "collaboration_definition_packages_v1",
                "collaboration_deployment_bindings_v1"
            ]
        })))
    }

    /// Validate one exact-byte collaboration package without storing it.
    #[tool(description = "Preflight an immutable UAR collaboration package without installing it")]
    async fn uar_collaboration_preflight_package(
        &self,
        Parameters(p): Parameters<CollaborationPackageParams>,
        McpExtension(parts): McpExtension<axum::http::request::Parts>,
    ) -> Result<CallToolResult, McpError> {
        collaboration_owner(&parts)?;
        let response = self
            .collaboration_catalog
            .preflight_package(&PackageSourceRequest {
                command_id: p.command_id,
                expected_catalog_revision: p.expected_catalog_revision,
                manifest: p.manifest,
                files: p.files,
            })
            .await
            .map_err(collaboration_mcp_error)?;
        Ok(ok_json(&response))
    }

    /// Atomically install an immutable collaboration package and command receipt.
    #[tool(description = "Install an immutable UAR collaboration package into the catalog")]
    async fn uar_collaboration_install_package(
        &self,
        Parameters(p): Parameters<CollaborationPackageParams>,
        McpExtension(parts): McpExtension<axum::http::request::Parts>,
    ) -> Result<CallToolResult, McpError> {
        let owner = collaboration_owner(&parts)?;
        let response = self
            .collaboration_catalog
            .install_package(
                &owner,
                PackageSourceRequest {
                    command_id: p.command_id,
                    expected_catalog_revision: p.expected_catalog_revision,
                    manifest: p.manifest,
                    files: p.files,
                },
            )
            .await
            .map_err(collaboration_mcp_error)?;
        Ok(ok_json(&response))
    }

    /// Retrieve an installed package by immutable identity and semantic version.
    #[tool(description = "Get one installed UAR collaboration package version")]
    async fn uar_collaboration_package_status(
        &self,
        Parameters(p): Parameters<CollaborationPackageStatusParams>,
        McpExtension(parts): McpExtension<axum::http::request::Parts>,
    ) -> Result<CallToolResult, McpError> {
        collaboration_owner(&parts)?;
        let package = self
            .collaboration_catalog
            .get_package_version(&p.id, &p.version)
            .await
            .map_err(collaboration_mcp_error)?;
        Ok(ok_json(&package))
    }

    /// Validate a private deployment binding without storing it.
    #[tool(description = "Preflight a private UAR collaboration deployment binding")]
    async fn uar_collaboration_preflight_binding(
        &self,
        Parameters(p): Parameters<CollaborationBindingParams>,
        McpExtension(parts): McpExtension<axum::http::request::Parts>,
    ) -> Result<CallToolResult, McpError> {
        let owner = collaboration_owner(&parts)?;
        let workspace = collaboration_workspace(&parts)?;
        let response = self
            .collaboration_catalog
            .preflight_binding(
                &owner,
                &workspace,
                &BindingCommandRequest {
                    command_id: p.command_id,
                    expected_revision: p.expected_revision,
                    binding: p.binding,
                },
            )
            .await
            .map_err(collaboration_mcp_error)?;
        Ok(ok_json(&response))
    }

    /// Validate and install a private revisioned binding for one workspace.
    #[tool(description = "Install a private UAR collaboration deployment binding")]
    async fn uar_collaboration_install_binding(
        &self,
        Parameters(p): Parameters<CollaborationBindingParams>,
        McpExtension(parts): McpExtension<axum::http::request::Parts>,
    ) -> Result<CallToolResult, McpError> {
        let owner = collaboration_owner(&parts)?;
        let workspace = collaboration_workspace(&parts)?;
        let response = self
            .collaboration_catalog
            .install_binding(
                &owner,
                &workspace,
                BindingCommandRequest {
                    command_id: p.command_id,
                    expected_revision: p.expected_revision,
                    binding: p.binding,
                },
            )
            .await
            .map_err(collaboration_mcp_error)?;
        Ok(ok_json(&response))
    }

    /// Read one private binding in the caller's exact workspace scope.
    #[tool(description = "Get one UAR collaboration deployment binding")]
    async fn uar_collaboration_binding_status(
        &self,
        Parameters(p): Parameters<CollaborationBindingStatusParams>,
        McpExtension(parts): McpExtension<axum::http::request::Parts>,
    ) -> Result<CallToolResult, McpError> {
        let owner = collaboration_owner(&parts)?;
        let workspace = collaboration_workspace(&parts)?;
        let binding = self
            .collaboration_catalog
            .get_binding(&owner, &workspace, &p.id)
            .await
            .map_err(collaboration_mcp_error)?;
        Ok(ok_json(&binding))
    }
}

#[tool_handler]
impl ServerHandler for UarRuntimeMcpServer {
    fn get_info(&self) -> ServerInfo {
        // rmcp 1.8: ServerInfo (InitializeResult) and Implementation are both
        // #[non_exhaustive] -- struct-literal syntax (even with
        // ..Default::default()) is rejected cross-crate; use the provided
        // constructors + field mutation instead (all fields are `pub`).
        let mut info = ServerInfo::new(ServerCapabilities::builder().enable_tools().build());
        info.server_info = Implementation::new("uar-runtime-mcp", env!("CARGO_PKG_VERSION"));
        info.instructions = Some(
            "UAR Runtime MCP server. Tools: \
            uar_list_agents — list compiled agents in the registry; \
            uar_create_run — start an agent run (returns run_id + SSE URL); \
            uar_get_run_status — poll run status by run_id; \
            uar_list_skills — enumerate available native skills; \
            uar_compile_spec — compile a UAR-AGENT-MD Markdown document; \
            uar_collaboration_capabilities / preflight_package / install_package / package_status — administer immutable multi-agent packages; \
            uar_collaboration_preflight_binding / install_binding / binding_status — administer private workspace bindings."
                .to_string(),
        );
        info
    }
}

// ── Public router builder ─────────────────────────────────────────────────────

/// Build an Axum router that exposes the UAR runtime as an MCP server over the
/// streamable-HTTP transport.
///
/// Mount at `/mcp/uar` in the main Axum router:
///
/// ```rust,ignore
/// let uar_mcp = uar_mcp_router(Arc::clone(&run_manager), Arc::clone(&native_skills), persistence.clone());
/// app = app.nest("/mcp/uar", uar_mcp);
/// ```
pub fn uar_mcp_router_with_collaboration(
    run_manager: Arc<RunManager>,
    native_skills: Arc<NativeSkillRegistry>,
    _persistence: Option<Arc<dyn PersistenceLayer>>,
    collaboration_catalog: Arc<CollaborationCatalogService>,
) -> Router {
    let session_manager = Arc::new(LocalSessionManager::default());

    // #[non_exhaustive] -- struct-literal syntax rejected cross-crate even
    // with ..Default::default(); mutate the public field on a default instance.
    let mut config = StreamableHttpServerConfig::default();
    config.legacy_session_mode = true;

    let http_service = StreamableHttpService::new(
        move || -> Result<UarRuntimeMcpServer, std::io::Error> {
            Ok(UarRuntimeMcpServer::new(
                Arc::clone(&run_manager),
                Arc::clone(&native_skills),
                Arc::clone(&collaboration_catalog),
            ))
        },
        Arc::clone(&session_manager),
        config,
    );

    Router::new().route_service("/", http_service)
}

/// Backward-compatible builder for embedders that do not yet provide a durable
/// collaboration catalog. Existing MCP behavior remains unchanged and the new
/// administration tools use a process-local catalog.
pub fn uar_mcp_router(
    run_manager: Arc<RunManager>,
    native_skills: Arc<NativeSkillRegistry>,
    persistence: Option<Arc<dyn PersistenceLayer>>,
) -> Router {
    uar_mcp_router_with_collaboration(
        run_manager,
        native_skills,
        persistence,
        Arc::new(CollaborationCatalogService::in_memory()),
    )
}
