use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::{RwLock, Semaphore};
use universal_agent_runtime::config::{HarnessConfig, LlmConfig, SkillActivationMode};
use universal_agent_runtime::llm::mock_driver::MockLlmDriver;
use universal_agent_runtime::mcp::binding_cache::{
    ConnectedMcpServer, McpBindingCache, McpBindingEnvironment, McpBindingError, McpBindingRequest,
};
use universal_agent_runtime::mcp::catalog::{
    McpCatalog, ServerAuthentication, ServerDefinition, ServerSandboxPolicy, ServerSource,
};
use universal_agent_runtime::mcp::config::{McpConfig, McpServerEntry, load_mcp_config};
use universal_agent_runtime::mcp::exposure::{
    MCP_EAGER_TOOL_LIMIT, MCP_SEARCH_RESULT_LIMIT, McpToolExposure,
};
use universal_agent_runtime::mcp::lifecycle::McpLifecycleSubscription;
use universal_agent_runtime::mcp::preflight::{McpPreflightError, McpServerFailure};
use universal_agent_runtime::mcp::projection::{
    McpProjectionScope, McpServerProjection, ServerToolCatalog,
};
use universal_agent_runtime::mcp::registry::McpRegistry;
use universal_agent_runtime::mcp::runtime::{
    ConfiguredMcpConnector, McpConnector, McpRunResources, McpRuntimeManager,
};
use universal_agent_runtime::normalized::NormalizedEvent as DriverEvent;
use universal_agent_runtime::session::SessionStore;
use universal_agent_runtime::uar::api::a2a::contract::{
    UAR_DELEGATION_CONTRACT_VERSION, UarDelegationContract, UarUsageGrant,
};
use universal_agent_runtime::uar::context::ContextStrategy;
use universal_agent_runtime::uar::defaults::default_agent;
use universal_agent_runtime::uar::domain::artifact::ToolExecutionMode;
use universal_agent_runtime::uar::domain::events::{
    McpServerLifecycle, McpServerState, McpStateReason, NormalizedEvent as RunEvent,
};
use universal_agent_runtime::uar::domain::policy::{
    ChatMode, ModelRoute, PolicyResolutionInput, PolicyUniverse, ResourceSelection, RunPolicy,
    SelectionMode, ToolApprovalPolicy, resolve_run_policy,
};
use universal_agent_runtime::uar::domain::skills::Skill;
use universal_agent_runtime::uar::rag::embeddings::{
    EmbeddingBackend, UnavailableEmbeddingBackend,
};
use universal_agent_runtime::uar::runtime::manager::RunManager;
use universal_agent_runtime::uar::runtime::matching::VectorMatcher;
use universal_agent_runtime::uar::runtime::native_skill::{NativeSkill, NativeSkillRegistry};
use universal_agent_runtime::uar::runtime::native_skills::search_tools::SearchToolsTool;
use universal_agent_runtime::uar::runtime::skills::SkillRegistry;
use universal_agent_runtime::uar::runtime::thread::policy_intersection::{
    SandboxPermissions, ThreadBudgets,
};
use universal_agent_runtime::uar::runtime::turn::RunExecutionRequest;
use universal_agent_runtime::uar::security::claims::{UserClaims, UserContext};
use universal_agent_runtime::uar::tools::descriptor::{
    ApprovalClass, Exposure, ToolDescriptor, ToolEffect, ToolSource,
};
use universal_agent_runtime::uar::tools::validate::ValidatorCompiler;

#[derive(Debug)]
struct CountingConnector {
    inner: ConfiguredMcpConnector,
    attempts: Arc<AtomicUsize>,
}

#[async_trait]
impl McpConnector for CountingConnector {
    async fn connect(
        &self,
        request: Arc<McpBindingRequest>,
    ) -> Result<ConnectedMcpServer, McpBindingError> {
        self.attempts.fetch_add(1, Ordering::SeqCst);
        self.inner.connect(request).await
    }

    async fn shutdown(&self) -> anyhow::Result<()> {
        self.inner.shutdown().await
    }
}

#[derive(Debug)]
struct FailingConnector;

#[async_trait]
impl McpConnector for FailingConnector {
    async fn connect(
        &self,
        request: Arc<McpBindingRequest>,
    ) -> Result<ConnectedMcpServer, McpBindingError> {
        Err(McpBindingError::ConnectionFailed {
            server: request.definition().name().to_string(),
        })
    }

    async fn shutdown(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
struct GatedConnector {
    inner: ConfiguredMcpConnector,
    attempts: Arc<AtomicUsize>,
    gate: Arc<Semaphore>,
}

#[async_trait]
impl McpConnector for GatedConnector {
    async fn connect(
        &self,
        request: Arc<McpBindingRequest>,
    ) -> Result<ConnectedMcpServer, McpBindingError> {
        self.attempts.fetch_add(1, Ordering::SeqCst);
        let permit = self
            .gate
            .acquire()
            .await
            .map_err(|_| McpBindingError::ConnectionFailed {
                server: request.definition().name().to_string(),
            })?;
        permit.forget();
        self.inner.connect(request).await
    }

    async fn shutdown(&self) -> anyhow::Result<()> {
        self.inner.shutdown().await
    }
}

fn skill_with_server(configuration: McpServerEntry) -> Skill {
    Skill {
        skill_id: "peer-skill".to_string(),
        title: "Peer UAR".to_string(),
        description: "Talk to another UAR instance".to_string(),
        prompt_overlay: "Use the authenticated peer UAR when needed.".to_string(),
        preferred_tools: vec!["peer-uar__uar_list_agents".to_string()],
        mcp_config: Some(McpConfig {
            mcp_servers: HashMap::from([("peer-uar".to_string(), configuration)]),
        }),
        enabled: true,
        ..Skill::default()
    }
}

fn catalog(configuration: McpServerEntry) -> Arc<McpCatalog> {
    catalog_with_requirement(configuration, true)
}

fn catalog_with_requirement(configuration: McpServerEntry, required: bool) -> Arc<McpCatalog> {
    let definition = ServerDefinition::new(
        "peer-uar".to_string(),
        ServerSource::Skill {
            skill_id: "peer-skill".to_string(),
        },
        configuration,
        required,
        ServerAuthentication::Authenticated {
            binding_id: "peer-auth-revision".to_string(),
        },
    )
    .expect("peer UAR definition is valid");
    Arc::new(McpCatalog::from_definitions([definition]).expect("peer catalog is valid"))
}

fn user() -> UserContext {
    UserContext {
        user_id: "peer-test-user".to_string(),
        tenant_id: None,
        claims: UserClaims {
            sub: "peer-test-user".to_string(),
            name: None,
            roles: Some(vec!["user".to_string()]),
            tenant_id: None,
            uar_instance_id: Some("source-uar".to_string()),
            exp: usize::MAX,
        },
    }
}

async fn wait_for_done(manager: &RunManager, run_id: &str) {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let done = manager
                .history_since(run_id, None)
                .await
                .is_some_and(|events| {
                    events.iter().any(|event| {
                        matches!(
                            event.event,
                            RunEvent::RunDone { .. } | RunEvent::RunDoneWithUsage { .. }
                        )
                    })
                });
            if done {
                return;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("run completes");
}

async fn wait_for_attempts(attempts: &AtomicUsize, expected: usize) {
    tokio::time::timeout(Duration::from_secs(1), async {
        while attempts.load(Ordering::SeqCst) < expected {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("connector reaches the expected attempt count");
}

fn receipt_count(path: &std::path::Path) -> usize {
    std::fs::read_to_string(path)
        .map(|contents| contents.lines().count())
        .unwrap_or(0)
}

async fn wait_for_receipts(path: &std::path::Path, expected: usize, boundary: &str) {
    let result = tokio::time::timeout(Duration::from_secs(4), async {
        while receipt_count(path) < expected {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await;
    assert!(
        result.is_ok(),
        "stdio fixture receipt timeout at {boundary}: expected {expected}, observed {} in {}",
        receipt_count(path),
        path.display()
    );
}

async fn receive_lifecycle(subscription: &mut McpLifecycleSubscription) -> McpServerLifecycle {
    let event = tokio::time::timeout(Duration::from_secs(1), subscription.recv())
        .await
        .expect("lifecycle event arrives")
        .expect("lifecycle publisher remains open");
    match event {
        RunEvent::McpServerStateChanged {
            run_id: None,
            lifecycle,
        } => lifecycle,
        other => panic!("expected owner-scoped MCP lifecycle event, got {other:?}"),
    }
}

fn mcp_status_metric(server: &str) -> f64 {
    let body = universal_agent_runtime::uar::telemetry::metrics::metrics_handle().render();
    body.lines()
        .find(|line| {
            line.starts_with("uar_mcp_server_status")
                && line.contains(&format!("server_name=\"{server}\""))
        })
        .and_then(|line| line.split_whitespace().last())
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or_else(|| panic!("MCP status metric for {server:?} is absent from:\n{body}"))
}

async fn run_with_attachment(
    manager: &RunManager,
    runtime: McpRuntimeManager,
    catalog: Arc<McpCatalog>,
    environment: Arc<McpBindingEnvironment>,
    user: &UserContext,
) {
    let owner =
        universal_agent_runtime::uar::runtime::actor::messages::ActorOwner::from_verified_context(
            user,
        )
        .expect("peer test owner is verified");
    let mut request = RunExecutionRequest::new(default_agent(), "use the peer".to_string())
        .with_user_context(user)
        .expect("run owner is valid");
    request.skill_attachments = vec!["peer-skill".to_string()];
    request.mcp_resources = Some(McpRunResources::new(owner, runtime, catalog, environment));
    let run_id = manager.execute_request(request).await;
    wait_for_done(manager, &run_id).await;
}

#[tokio::test]
async fn peer_connection_is_reused_until_the_skill_configuration_hash_changes() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("peer UAR listener binds");
    let peer_address = listener
        .local_addr()
        .expect("peer UAR address is available");
    let base_configuration = McpServerEntry::RemoteHttp {
        url: format!("http://{peer_address}/"),
        env: HashMap::new(),
    };
    let registry = Arc::new(RwLock::new(SkillRegistry::default()));
    registry
        .write()
        .await
        .register_loaded(skill_with_server(base_configuration.clone()));
    let driver = Arc::new(MockLlmDriver::new(vec![vec![
        DriverEvent::MessageDelta {
            text: "done".to_string(),
        },
        DriverEvent::Done,
    ]]));
    let embeddings: Arc<dyn EmbeddingBackend> = Arc::new(UnavailableEmbeddingBackend::new(
        384,
        "embeddings are not exercised by this test",
    ));
    let manager = Arc::new(
        RunManager::new(
            LlmConfig {
                model: "openai/gpt-4o".to_string(),
                api_key: Some("test-key".to_string()),
                ..LlmConfig::default()
            },
            Arc::new(McpRegistry::new_empty()),
            SessionStore::new(),
            Arc::clone(&registry),
            Arc::new(VectorMatcher::new(embeddings, 0.75)),
            None,
        )
        .await
        .with_harness_config(HarnessConfig {
            skill_activation_mode: SkillActivationMode::Catalog,
            ..HarnessConfig::default()
        })
        .with_llm_driver(driver),
    );
    let peer = universal_agent_runtime::uar::mcp_server::uar_mcp_router(
        Arc::clone(&manager),
        Arc::new(NativeSkillRegistry::new()),
        None,
    );
    let peer_server = tokio::spawn(async move {
        axum::serve(listener, peer)
            .await
            .expect("peer UAR MCP server runs");
    });
    let attempts = Arc::new(AtomicUsize::new(0));
    let runtime = McpRuntimeManager::new(
        McpBindingCache::default(),
        Arc::new(CountingConnector {
            inner: ConfiguredMcpConnector::default(),
            attempts: Arc::clone(&attempts),
        }),
        Duration::from_secs(1),
        Duration::from_secs(1),
    )
    .expect("projected MCP runtime is valid");
    let environment = Arc::new(
        McpBindingEnvironment::new(std::env::temp_dir(), BTreeMap::new())
            .expect("test environment is valid"),
    );
    let user = user();

    run_with_attachment(
        &manager,
        runtime.clone(),
        catalog(base_configuration.clone()),
        Arc::clone(&environment),
        &user,
    )
    .await;
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
    run_with_attachment(
        &manager,
        runtime.clone(),
        catalog(base_configuration.clone()),
        Arc::clone(&environment),
        &user,
    )
    .await;
    assert_eq!(attempts.load(Ordering::SeqCst), 1);

    let changed_configuration = match base_configuration {
        McpServerEntry::RemoteHttp { url, .. } => McpServerEntry::RemoteHttp {
            url,
            env: HashMap::from([("PEER_CONFIG_REVISION".to_string(), "2".to_string())]),
        },
        McpServerEntry::Stdio { .. } => unreachable!("peer fixture is HTTP"),
    };
    registry
        .write()
        .await
        .register_loaded(skill_with_server(changed_configuration.clone()));
    run_with_attachment(
        &manager,
        runtime.clone(),
        catalog(changed_configuration),
        environment,
        &user,
    )
    .await;
    assert_eq!(attempts.load(Ordering::SeqCst), 2);

    runtime
        .shutdown()
        .await
        .expect("projected MCP runtime shuts down");
    peer_server.abort();
    let _ = peer_server.await;
}

#[tokio::test]
async fn cached_skill_catalog_stays_lazy_until_the_first_tool_call() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("peer UAR listener binds");
    let peer_address = listener
        .local_addr()
        .expect("peer UAR address is available");
    let configuration = McpServerEntry::RemoteHttp {
        url: format!("http://{peer_address}/"),
        env: HashMap::new(),
    };
    let registry = Arc::new(RwLock::new(SkillRegistry::default()));
    let driver = Arc::new(MockLlmDriver::new(Vec::new()));
    let embeddings: Arc<dyn EmbeddingBackend> = Arc::new(UnavailableEmbeddingBackend::new(
        384,
        "embeddings are not exercised by this test",
    ));
    let manager = Arc::new(
        RunManager::new(
            LlmConfig {
                model: "openai/gpt-4o".to_string(),
                api_key: Some("test-key".to_string()),
                ..LlmConfig::default()
            },
            Arc::new(McpRegistry::new_empty()),
            SessionStore::new(),
            registry,
            Arc::new(VectorMatcher::new(embeddings, 0.75)),
            None,
        )
        .await
        .with_llm_driver(driver),
    );
    let peer = universal_agent_runtime::uar::mcp_server::uar_mcp_router(
        manager,
        Arc::new(NativeSkillRegistry::new()),
        None,
    );
    let peer_server = tokio::spawn(async move {
        axum::serve(listener, peer)
            .await
            .expect("peer UAR MCP server runs");
    });
    let attempts = Arc::new(AtomicUsize::new(0));
    let runtime = McpRuntimeManager::new(
        McpBindingCache::default(),
        Arc::new(CountingConnector {
            inner: ConfiguredMcpConnector::default(),
            attempts: Arc::clone(&attempts),
        }),
        Duration::from_secs(1),
        Duration::from_secs(1),
    )
    .expect("projected MCP runtime is valid");
    let environment = McpBindingEnvironment::new(std::env::temp_dir(), BTreeMap::new())
        .expect("test environment is valid");
    let owner =
        universal_agent_runtime::uar::runtime::actor::messages::ActorOwner::from_verified_context(
            &user(),
        )
        .expect("peer test owner is verified");
    let catalog = catalog(configuration);
    let policy = resolve_run_policy(PolicyResolutionInput {
        universe: PolicyUniverse {
            skills: ["peer-skill".to_string()].into(),
            tools: ["peer-uar__uar_list_agents".to_string()].into(),
            mcp_servers: ["peer-uar".to_string()].into(),
            ..PolicyUniverse::default()
        },
        ..PolicyResolutionInput::default()
    });
    let scope = McpProjectionScope {
        active_skills: ["peer-skill".to_string()].into(),
    };

    let initial_projection = McpServerProjection::resolve(&catalog, &policy, &scope)
        .expect("active peer skill is projected");
    let initial = runtime
        .preflight(&initial_projection, &owner, &environment)
        .await
        .expect("initial discovery succeeds");
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
    initial
        .servers()
        .get("peer-uar")
        .expect("peer server is prepared")
        .retire_connection()
        .await
        .expect("peer transport retires while preserving discovery");

    let cached_projection = McpServerProjection::resolve(&catalog, &policy, &scope)
        .expect("active peer skill remains projected");
    let cached = runtime
        .preflight(&cached_projection, &owner, &environment)
        .await
        .expect("complete cached discovery supports lazy preflight");
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
    assert!(
        cached
            .projection()
            .tools()
            .contains_key("peer-uar__uar_list_agents")
    );

    let result = cached
        .call_tool("peer-uar__uar_list_agents", serde_json::json!({}))
        .await
        .expect("first projected peer call reconnects and succeeds");
    assert!(!result.is_null());
    assert_eq!(attempts.load(Ordering::SeqCst), 2);

    runtime
        .shutdown()
        .await
        .expect("projected MCP runtime shuts down");
    peer_server.abort();
    let _ = peer_server.await;
}

#[tokio::test]
async fn required_startup_failure_aborts_while_optional_failure_omits_tools() {
    let configuration = McpServerEntry::RemoteHttp {
        url: "http://127.0.0.1:1/".to_string(),
        env: HashMap::new(),
    };
    let environment = McpBindingEnvironment::new(std::env::temp_dir(), BTreeMap::new())
        .expect("test environment is valid");
    let owner =
        universal_agent_runtime::uar::runtime::actor::messages::ActorOwner::from_verified_context(
            &user(),
        )
        .expect("peer test owner is verified");
    let policy = resolve_run_policy(PolicyResolutionInput {
        universe: PolicyUniverse {
            skills: ["peer-skill".to_string()].into(),
            tools: ["peer-uar__uar_list_agents".to_string()].into(),
            mcp_servers: ["peer-uar".to_string()].into(),
            ..PolicyUniverse::default()
        },
        ..PolicyResolutionInput::default()
    });
    let scope = McpProjectionScope {
        active_skills: ["peer-skill".to_string()].into(),
    };

    let required_catalog = catalog_with_requirement(configuration.clone(), true);
    let required_projection = McpServerProjection::resolve(&required_catalog, &policy, &scope)
        .expect("required peer server is projected");
    let required_runtime = McpRuntimeManager::new(
        McpBindingCache::default(),
        Arc::new(FailingConnector),
        Duration::from_secs(1),
        Duration::from_secs(1),
    )
    .expect("required projected runtime is valid");
    let required_error = required_runtime
        .preflight(&required_projection, &owner, &environment)
        .await
        .expect_err("required startup failure aborts preflight");
    assert_eq!(
        required_error,
        McpPreflightError::RequiredServer {
            server: "peer-uar".to_string(),
            reason: McpServerFailure::Connection,
        }
    );

    let optional_catalog = catalog_with_requirement(configuration, false);
    let optional_projection = McpServerProjection::resolve(&optional_catalog, &policy, &scope)
        .expect("optional peer server is projected");
    let optional_runtime = McpRuntimeManager::new(
        McpBindingCache::default(),
        Arc::new(FailingConnector),
        Duration::from_secs(1),
        Duration::from_secs(1),
    )
    .expect("optional projected runtime is valid");
    let optional = optional_runtime
        .preflight(&optional_projection, &owner, &environment)
        .await
        .expect("optional startup failure does not abort preflight");
    assert_eq!(optional.warnings().len(), 1);
    assert_eq!(optional.warnings()[0].server(), "peer-uar");
    assert_eq!(
        optional.warnings()[0].reason(),
        McpServerFailure::Connection
    );
    assert!(optional.projection().servers().is_empty());
    assert!(optional.projection().tools().is_empty());
    assert!(optional.servers().is_empty());

    required_runtime
        .shutdown()
        .await
        .expect("required runtime shuts down");
    optional_runtime
        .shutdown()
        .await
        .expect("optional runtime shuts down");
}

#[tokio::test]
async fn search_tools_surfaces_deferred_matches_only_on_the_next_step() {
    let configuration = McpServerEntry::RemoteHttp {
        url: "http://127.0.0.1:1/".to_string(),
        env: HashMap::new(),
    };
    let catalog = catalog(configuration);
    let definition = Arc::clone(
        catalog
            .candidates("peer-uar")
            .next()
            .expect("peer definition is present"),
    );
    let compiler = ValidatorCompiler::default();
    let mut names = Vec::new();
    let mut descriptors = Vec::new();
    for index in 0..200 {
        let subject = if index >= 192 {
            "zz_calendar"
        } else {
            "aa_utility"
        };
        let raw_name = format!("{subject}_{index:03}");
        let provider_name = format!("peer-uar__{raw_name}");
        let input_schema = serde_json::json!({
            "type": "object",
            "additionalProperties": false,
        });
        let validator = compiler
            .compile(&provider_name, &input_schema)
            .expect("fixture schema compiles");
        names.push(provider_name.clone());
        descriptors.push(Arc::new(ToolDescriptor {
            id: format!("peer-uar::{raw_name}"),
            provider_name,
            description: format!("Peer UAR {subject} operation"),
            source: ToolSource::Mcp,
            server: Some("peer-uar".to_string()),
            input_schema,
            validator,
            effect: ToolEffect::ReadOnly,
            approval_class: ApprovalClass::Required,
            sandbox_required: false,
            concurrency_key: None,
            exposure: Exposure::Eager,
            output_limit: None,
        }));
    }
    let policy = resolve_run_policy(PolicyResolutionInput {
        universe: PolicyUniverse {
            skills: ["peer-skill".to_string()].into(),
            tools: names.iter().cloned().collect(),
            mcp_servers: ["peer-uar".to_string()].into(),
            ..PolicyUniverse::default()
        },
        ..PolicyResolutionInput::default()
    });
    let scope = McpProjectionScope {
        active_skills: ["peer-skill".to_string()].into(),
    };
    let servers =
        McpServerProjection::resolve(&catalog, &policy, &scope).expect("peer server is projected");
    let step = servers
        .with_tools([ServerToolCatalog::new(definition, descriptors, true)
            .expect("complete peer tool catalog is valid")])
        .expect("peer tools are projected");
    let exposure = McpToolExposure::default();

    let initial = step.exposure(&exposure);
    assert_eq!(initial.visible().len(), MCP_EAGER_TOOL_LIMIT);
    assert!(initial.has_deferred());
    assert_eq!(
        names
            .iter()
            .filter(|name| initial.exposure(name) == Exposure::Deferred)
            .count(),
        200 - MCP_EAGER_TOOL_LIMIT
    );
    let calendar_names = names
        .iter()
        .filter(|name| name.contains("calendar"))
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        calendar_names
            .iter()
            .all(|name| !initial.visible().contains_key(name))
    );

    let search = SearchToolsTool::new(exposure.clone());
    assert_eq!(search.exposure(), Exposure::ModelOnly);
    let selected = search
        .execute(serde_json::json!({"query": "calendar"}))
        .await
        .expect("model-only tool search succeeds");
    assert_eq!(selected["status"], "selected_for_next_step");
    assert_eq!(
        selected["tools"].as_array().map(Vec::len),
        Some(MCP_SEARCH_RESULT_LIMIT)
    );
    assert!(
        calendar_names
            .iter()
            .all(|name| !initial.visible().contains_key(name))
    );

    let next = step.exposure(&exposure);
    assert_eq!(next.visible().len(), MCP_EAGER_TOOL_LIMIT);
    assert!(
        calendar_names
            .iter()
            .all(|name| next.visible().contains_key(name))
    );
}

#[tokio::test]
async fn global_authority_and_peer_delegation_keep_connection_recipes_host_local() {
    let global_definition = ServerDefinition::new(
        "peer-uar".to_string(),
        ServerSource::Global,
        McpServerEntry::Stdio {
            command: "global-peer-host".to_string(),
            args: Vec::new(),
            env: HashMap::new(),
            sandboxed: false,
        },
        true,
        ServerAuthentication::NotRequired,
    )
    .expect("global peer declaration is valid");
    let global_hash = global_definition.config_hash().clone();
    let skill_definition = ServerDefinition::new(
        "peer-uar".to_string(),
        ServerSource::Skill {
            skill_id: "peer-skill".to_string(),
        },
        McpServerEntry::RemoteHttp {
            url: "http://127.0.0.1:1/".to_string(),
            env: HashMap::new(),
        },
        false,
        ServerAuthentication::NotRequired,
    )
    .expect("skill peer declaration is valid");
    let authority_catalog = McpCatalog::from_definitions([global_definition, skill_definition])
        .expect("authority catalog is valid");
    let authority_policy = resolve_run_policy(PolicyResolutionInput {
        universe: PolicyUniverse {
            skills: ["peer-skill".to_string()].into(),
            mcp_servers: ["peer-uar".to_string()].into(),
            ..PolicyUniverse::default()
        },
        ..PolicyResolutionInput::default()
    });
    let scope = McpProjectionScope {
        active_skills: ["peer-skill".to_string()].into(),
    };
    let authority = McpServerProjection::resolve(&authority_catalog, &authority_policy, &scope)
        .expect("global authority resolves without ambiguity");
    let selected = authority
        .servers()
        .get("peer-uar")
        .expect("peer server is selected");
    assert_eq!(selected.source(), &ServerSource::Global);
    assert_eq!(selected.config_hash(), &global_hash);
    assert_eq!(selected.sandbox_policy(), ServerSandboxPolicy::Unrestricted);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("peer UAR listener binds");
    let peer_address = listener
        .local_addr()
        .expect("peer UAR address is available");
    let live_url = format!("http://{peer_address}/");
    let registry = Arc::new(RwLock::new(SkillRegistry::default()));
    let driver = Arc::new(MockLlmDriver::new(Vec::new()));
    let embeddings: Arc<dyn EmbeddingBackend> = Arc::new(UnavailableEmbeddingBackend::new(
        384,
        "embeddings are not exercised by this test",
    ));
    let manager = Arc::new(
        RunManager::new(
            LlmConfig {
                model: "openai/gpt-4o".to_string(),
                api_key: Some("test-key".to_string()),
                ..LlmConfig::default()
            },
            Arc::new(McpRegistry::new_empty()),
            SessionStore::new(),
            registry,
            Arc::new(VectorMatcher::new(embeddings, 0.75)),
            None,
        )
        .await
        .with_llm_driver(driver),
    );
    let peer = universal_agent_runtime::uar::mcp_server::uar_mcp_router(
        manager,
        Arc::new(NativeSkillRegistry::new()),
        None,
    );
    let peer_server = tokio::spawn(async move {
        axum::serve(listener, peer)
            .await
            .expect("peer UAR MCP server runs");
    });
    let live_definition = ServerDefinition::new(
        "peer-uar".to_string(),
        ServerSource::Global,
        McpServerEntry::RemoteHttp {
            url: live_url.clone(),
            env: HashMap::new(),
        },
        true,
        ServerAuthentication::NotRequired,
    )
    .expect("live global peer declaration is valid");
    let live_catalog =
        McpCatalog::from_definitions([live_definition]).expect("live peer catalog is valid");
    let live_policy = resolve_run_policy(PolicyResolutionInput {
        universe: PolicyUniverse {
            tools: ["peer-uar__uar_list_agents".to_string()].into(),
            mcp_servers: ["peer-uar".to_string()].into(),
            ..PolicyUniverse::default()
        },
        ..PolicyResolutionInput::default()
    });
    let live_projection =
        McpServerProjection::resolve(&live_catalog, &live_policy, &McpProjectionScope::default())
            .expect("live global peer is projected");
    let runtime = McpRuntimeManager::new(
        McpBindingCache::default(),
        Arc::new(ConfiguredMcpConnector::default()),
        Duration::from_secs(1),
        Duration::from_secs(1),
    )
    .expect("peer runtime is valid");
    let environment = McpBindingEnvironment::new(std::env::temp_dir(), BTreeMap::new())
        .expect("test environment is valid");
    let owner =
        universal_agent_runtime::uar::runtime::actor::messages::ActorOwner::from_verified_context(
            &user(),
        )
        .expect("peer test owner is verified");
    let preflight = runtime
        .preflight(&live_projection, &owner, &environment)
        .await
        .expect("live peer preflight succeeds");
    let frozen = preflight
        .freeze_bindings(&McpRegistry::new_empty())
        .await
        .expect("local delegation captures the live peer binding");
    assert!(frozen.has_frozen_bindings());
    assert!(frozen.server_entries().is_empty());
    assert_eq!(
        frozen.resolve_mcp_tool("peer-uar__uar_list_agents"),
        Some(("peer-uar".to_string(), "uar_list_agents".to_string()))
    );
    let frozen_result = frozen
        .call_namespaced_tool("peer-uar__uar_list_agents", serde_json::json!({}))
        .await
        .expect("captured local peer binding remains callable");
    assert!(!frozen_result.is_null());

    let none = ResourceSelection {
        mode: SelectionMode::None,
        ..ResourceSelection::default()
    };
    let contract = UarDelegationContract {
        version: UAR_DELEGATION_CONTRACT_VERSION,
        source_instance_id: "source-uar".to_string(),
        target_instance_id: "target-uar".to_string(),
        owner_id: "peer-test-user".to_string(),
        root_run_id: "root-run".to_string(),
        parent_thread_id: "parent-thread".to_string(),
        child_thread_id: "child-thread".to_string(),
        target_agent_id: "peer-agent".to_string(),
        policy: RunPolicy {
            chat_mode: Some(ChatMode::Agent),
            agent_id: Some("peer-agent".to_string()),
            model: Some(ModelRoute {
                provider_id: "peer-provider".to_string(),
                model_id: "peer-model".to_string(),
            }),
            skills: none.clone(),
            tools: ResourceSelection::selected(["peer-uar__uar_list_agents".to_string()]),
            mcp_servers: ResourceSelection::selected(["peer-uar".to_string()]),
            presentations: none.clone(),
            knowledge_bases: none,
            memory_enabled: Some(false),
            prompt_caching_enabled: Some(false),
            context_strategy: Some(ContextStrategy::Auto),
            tool_approval: ToolApprovalPolicy::Auto,
            ..RunPolicy::default()
        }
        .into(),
        budgets: ThreadBudgets::default(),
        usage_grant: UarUsageGrant::default(),
        sandbox: SandboxPermissions {
            execution_mode: ToolExecutionMode::Direct,
            network_enabled: true,
            filesystem: BTreeMap::new(),
            environment: Default::default(),
        },
        presentation_negotiation: None,
    };
    contract
        .validate()
        .expect("peer delegation contract is valid");
    let remote_payload =
        serde_json::to_string(&contract).expect("peer delegation contract serializes");
    assert!(!remote_payload.contains(&live_url));
    for recipe_field in ["\"command\"", "\"args\"", "\"env\"", "\"url\""] {
        assert!(!remote_payload.contains(recipe_field));
    }

    runtime.shutdown().await.expect("peer runtime shuts down");
    peer_server.abort();
    let _ = peer_server.await;
}

#[tokio::test]
async fn refresh_is_single_flight_and_a_cancelled_leader_is_retried() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("peer UAR listener binds");
    let peer_address = listener
        .local_addr()
        .expect("peer UAR address is available");
    let configuration = McpServerEntry::RemoteHttp {
        url: format!("http://{peer_address}/"),
        env: HashMap::new(),
    };
    let registry = Arc::new(RwLock::new(SkillRegistry::default()));
    let driver = Arc::new(MockLlmDriver::new(Vec::new()));
    let embeddings: Arc<dyn EmbeddingBackend> = Arc::new(UnavailableEmbeddingBackend::new(
        384,
        "embeddings are not exercised by this test",
    ));
    let manager = Arc::new(
        RunManager::new(
            LlmConfig {
                model: "openai/gpt-4o".to_string(),
                api_key: Some("test-key".to_string()),
                ..LlmConfig::default()
            },
            Arc::new(McpRegistry::new_empty()),
            SessionStore::new(),
            registry,
            Arc::new(VectorMatcher::new(embeddings, 0.75)),
            None,
        )
        .await
        .with_llm_driver(driver),
    );
    let peer = universal_agent_runtime::uar::mcp_server::uar_mcp_router(
        manager,
        Arc::new(NativeSkillRegistry::new()),
        None,
    );
    let peer_server = tokio::spawn(async move {
        axum::serve(listener, peer)
            .await
            .expect("peer UAR MCP server runs");
    });
    let owner =
        universal_agent_runtime::uar::runtime::actor::messages::ActorOwner::from_verified_context(
            &user(),
        )
        .expect("peer test owner is verified");
    let definition = Arc::new(
        ServerDefinition::new(
            "peer-uar".to_string(),
            ServerSource::Skill {
                skill_id: "peer-skill".to_string(),
            },
            configuration,
            true,
            ServerAuthentication::NotRequired,
        )
        .expect("peer definition is valid"),
    );
    let environment = Arc::new(
        McpBindingEnvironment::new(std::env::temp_dir(), BTreeMap::new())
            .expect("test environment is valid"),
    );
    let request = Arc::new(McpBindingRequest::new(owner, definition, environment));

    let shared_attempts = Arc::new(AtomicUsize::new(0));
    let shared_gate = Arc::new(Semaphore::new(0));
    let shared_runtime = McpRuntimeManager::new(
        McpBindingCache::default(),
        Arc::new(GatedConnector {
            inner: ConfiguredMcpConnector::default(),
            attempts: Arc::clone(&shared_attempts),
            gate: Arc::clone(&shared_gate),
        }),
        Duration::from_secs(1),
        Duration::from_secs(1),
    )
    .expect("single-flight runtime is valid");
    let first = tokio::spawn({
        let runtime = shared_runtime.clone();
        let request = Arc::clone(&request);
        async move { runtime.prepare(request).await }
    });
    wait_for_attempts(&shared_attempts, 1).await;
    let second = tokio::spawn({
        let runtime = shared_runtime.clone();
        let request = Arc::clone(&request);
        async move { runtime.prepare(request).await }
    });
    tokio::task::yield_now().await;
    assert_eq!(shared_attempts.load(Ordering::SeqCst), 1);
    shared_gate.add_permits(1);
    first
        .await
        .expect("first preparation task joins")
        .expect("first preparation succeeds");
    second
        .await
        .expect("second preparation task joins")
        .expect("second preparation shares the successful refresh");
    assert_eq!(shared_attempts.load(Ordering::SeqCst), 1);

    let retry_attempts = Arc::new(AtomicUsize::new(0));
    let retry_gate = Arc::new(Semaphore::new(0));
    let retry_runtime = McpRuntimeManager::new(
        McpBindingCache::default(),
        Arc::new(GatedConnector {
            inner: ConfiguredMcpConnector::default(),
            attempts: Arc::clone(&retry_attempts),
            gate: Arc::clone(&retry_gate),
        }),
        Duration::from_secs(1),
        Duration::from_secs(1),
    )
    .expect("retry runtime is valid");
    let cancelled = tokio::spawn({
        let runtime = retry_runtime.clone();
        let request = Arc::clone(&request);
        async move { runtime.prepare(request).await }
    });
    wait_for_attempts(&retry_attempts, 1).await;
    cancelled.abort();
    assert!(
        cancelled
            .await
            .expect_err("cancelled preparation does not complete")
            .is_cancelled()
    );

    let replacement = tokio::spawn({
        let runtime = retry_runtime.clone();
        let request = Arc::clone(&request);
        async move { runtime.prepare(request).await }
    });
    wait_for_attempts(&retry_attempts, 2).await;
    retry_gate.add_permits(1);
    replacement
        .await
        .expect("replacement preparation task joins")
        .expect("replacement preparation succeeds");
    assert_eq!(retry_attempts.load(Ordering::SeqCst), 2);

    shared_runtime
        .shutdown()
        .await
        .expect("single-flight runtime shuts down");
    retry_runtime
        .shutdown()
        .await
        .expect("retry runtime shuts down");
    peer_server.abort();
    let _ = peer_server.await;
}

#[tokio::test]
async fn lifecycle_events_and_status_metric_follow_binding_transitions_in_order() {
    universal_agent_runtime::uar::telemetry::metrics::init();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("peer UAR listener binds");
    let peer_address = listener
        .local_addr()
        .expect("peer UAR address is available");
    let configuration = McpServerEntry::RemoteHttp {
        url: format!("http://{peer_address}/"),
        env: HashMap::new(),
    };
    let registry = Arc::new(RwLock::new(SkillRegistry::default()));
    let driver = Arc::new(MockLlmDriver::new(Vec::new()));
    let embeddings: Arc<dyn EmbeddingBackend> = Arc::new(UnavailableEmbeddingBackend::new(
        384,
        "embeddings are not exercised by this test",
    ));
    let manager = Arc::new(
        RunManager::new(
            LlmConfig {
                model: "openai/gpt-4o".to_string(),
                api_key: Some("test-key".to_string()),
                ..LlmConfig::default()
            },
            Arc::new(McpRegistry::new_empty()),
            SessionStore::new(),
            registry,
            Arc::new(VectorMatcher::new(embeddings, 0.75)),
            None,
        )
        .await
        .with_llm_driver(driver),
    );
    let peer = universal_agent_runtime::uar::mcp_server::uar_mcp_router(
        manager,
        Arc::new(NativeSkillRegistry::new()),
        None,
    );
    let peer_server = tokio::spawn(async move {
        axum::serve(listener, peer)
            .await
            .expect("peer UAR MCP server runs");
    });
    let owner =
        universal_agent_runtime::uar::runtime::actor::messages::ActorOwner::from_verified_context(
            &user(),
        )
        .expect("peer test owner is verified");
    let server = "lifecycle-peer-uar";
    let definition = Arc::new(
        ServerDefinition::new(
            server.to_string(),
            ServerSource::Global,
            configuration,
            true,
            ServerAuthentication::NotRequired,
        )
        .expect("lifecycle peer definition is valid"),
    );
    let environment = Arc::new(
        McpBindingEnvironment::new(std::env::temp_dir(), BTreeMap::new())
            .expect("test environment is valid"),
    );
    let request = Arc::new(McpBindingRequest::new(owner, definition, environment));
    let attempts = Arc::new(AtomicUsize::new(0));
    let gate = Arc::new(Semaphore::new(0));
    let runtime = McpRuntimeManager::new(
        McpBindingCache::default(),
        Arc::new(GatedConnector {
            inner: ConfiguredMcpConnector::default(),
            attempts: Arc::clone(&attempts),
            gate: Arc::clone(&gate),
        }),
        Duration::from_secs(1),
        Duration::from_secs(1),
    )
    .expect("lifecycle runtime is valid");
    let mut subscription = runtime
        .observe(&request)
        .expect("binding observation starts");

    let initial = receive_lifecycle(&mut subscription).await;
    assert_eq!(initial.server, server);
    assert_eq!(initial.sequence, 0);
    assert_eq!(initial.state, McpServerState::Disabled);
    assert_eq!(mcp_status_metric(server), 0.0);

    let preparing = tokio::spawn({
        let runtime = runtime.clone();
        let request = Arc::clone(&request);
        async move { runtime.prepare(request).await }
    });
    wait_for_attempts(&attempts, 1).await;
    let connecting = receive_lifecycle(&mut subscription).await;
    assert_eq!(connecting.binding_id, initial.binding_id);
    assert_eq!(connecting.sequence, 1);
    assert_eq!(connecting.state, McpServerState::Connecting);
    assert_eq!(mcp_status_metric(server), 0.0);

    gate.add_permits(1);
    preparing
        .await
        .expect("preparation task joins")
        .expect("peer preparation succeeds");
    let ready = receive_lifecycle(&mut subscription).await;
    assert_eq!(ready.binding_id, initial.binding_id);
    assert_eq!(ready.sequence, 2);
    assert_eq!(ready.state, McpServerState::Ready);
    assert_eq!(mcp_status_metric(server), 1.0);

    runtime.invalidate_server(server).await;
    let shutting_down = receive_lifecycle(&mut subscription).await;
    let disabled = receive_lifecycle(&mut subscription).await;
    assert_eq!(shutting_down.binding_id, initial.binding_id);
    assert_eq!(shutting_down.sequence, 3);
    assert_eq!(shutting_down.state, McpServerState::ShuttingDown);
    assert_eq!(shutting_down.reason, Some(McpStateReason::Invalidated));
    assert_eq!(disabled.binding_id, initial.binding_id);
    assert_eq!(disabled.sequence, 4);
    assert_eq!(disabled.state, McpServerState::Disabled);
    assert_eq!(disabled.reason, Some(McpStateReason::Invalidated));
    assert_eq!(mcp_status_metric(server), 0.0);

    runtime
        .shutdown()
        .await
        .expect("lifecycle runtime shuts down");
    let stopped = receive_lifecycle(&mut subscription).await;
    assert_eq!(stopped.binding_id, initial.binding_id);
    assert_eq!(stopped.sequence, 5);
    assert_eq!(stopped.state, McpServerState::ShuttingDown);
    assert_eq!(mcp_status_metric(server), 0.0);

    peer_server.abort();
    let _ = peer_server.await;
}

#[test]
fn sandboxed_stdio_is_rejected_during_config_load_before_process_launch() {
    let directory = tempfile::tempdir().expect("sandbox fixture directory is created");
    let marker = directory.path().join("sandbox-command-ran");
    let config_path = directory.path().join("mcp.json");
    let config = serde_json::json!({
        "mcpServers": {
            "sandbox-required": {
                "command": "/bin/sh",
                "args": ["-c", format!("touch {}", marker.display())],
                "sandboxed": true
            }
        }
    });
    std::fs::write(
        &config_path,
        serde_json::to_vec(&config).expect("sandbox fixture serializes"),
    )
    .expect("sandbox fixture is written");

    let error = load_mcp_config(&config_path)
        .expect_err("unsupported sandbox request must fail config load");
    let message = error.to_string();
    assert!(message.contains("sandbox-required"), "{message}");
    assert!(message.contains("sandboxed: true"), "{message}");
    assert!(
        message.contains("OS-backed stdio sandbox backend is unavailable"),
        "{message}"
    );
    assert!(
        !marker.exists(),
        "rejected config must not launch its command"
    );
}

#[tokio::test]
async fn real_stdio_server_covers_lazy_reconnect_cancel_and_shutdown() {
    let receipt_directory = tempfile::tempdir().expect("stdio receipt directory is created");
    let started = receipt_directory.path().join("started.log");
    let stopped = receipt_directory.path().join("stopped.log");
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mcp_stdio_server.py");
    let path = std::env::var_os("PATH").expect("local integration requires PATH");
    let receipt_value = receipt_directory.path().as_os_str().to_os_string();
    let environment = Arc::new(
        McpBindingEnvironment::new(
            receipt_directory.path().to_path_buf(),
            BTreeMap::from([
                (std::ffi::OsString::from("PATH"), path.clone()),
                (
                    std::ffi::OsString::from("MCP_FIXTURE_RECEIPT_DIR"),
                    receipt_value.clone(),
                ),
            ]),
        )
        .expect("stdio binding environment is valid"),
    );
    let configuration = McpServerEntry::Stdio {
        command: "python3".to_string(),
        args: vec![fixture.display().to_string()],
        env: HashMap::from([(
            "MCP_FIXTURE_RECEIPT_DIR".to_string(),
            "${MCP_FIXTURE_RECEIPT_DIR}".to_string(),
        )]),
        sandboxed: false,
    };
    let definition = Arc::new(
        ServerDefinition::new(
            "stdio-peer".to_string(),
            ServerSource::Skill {
                skill_id: "peer-skill".to_string(),
            },
            configuration,
            true,
            ServerAuthentication::NotRequired,
        )
        .expect("stdio peer definition is valid"),
    );
    let owner =
        universal_agent_runtime::uar::runtime::actor::messages::ActorOwner::from_verified_context(
            &user(),
        )
        .expect("stdio peer owner is verified");
    let request = Arc::new(McpBindingRequest::new(
        owner.clone(),
        Arc::clone(&definition),
        Arc::clone(&environment),
    ));
    let stdio_attempts = Arc::new(AtomicUsize::new(0));
    let runtime = McpRuntimeManager::new(
        McpBindingCache::default(),
        Arc::new(CountingConnector {
            inner: ConfiguredMcpConnector::default(),
            attempts: Arc::clone(&stdio_attempts),
        }),
        Duration::from_secs(2),
        Duration::from_secs(2),
    )
    .expect("stdio runtime is valid");

    let initial = runtime
        .prepare(Arc::clone(&request))
        .await
        .expect("initial stdio discovery succeeds");
    assert_eq!(stdio_attempts.load(Ordering::SeqCst), 1);
    wait_for_receipts(&started, 1, "initial discovery start").await;
    initial
        .retire_connection()
        .await
        .expect("initial stdio connection retires");
    wait_for_receipts(&stopped, 1, "initial retirement stop").await;

    let catalog = McpCatalog::from_definitions([definition.as_ref().clone()])
        .expect("stdio peer catalog is valid");
    let policy = resolve_run_policy(PolicyResolutionInput {
        universe: PolicyUniverse {
            skills: ["peer-skill".to_string()].into(),
            tools: ["stdio-peer__echo".to_string()].into(),
            mcp_servers: ["stdio-peer".to_string()].into(),
            ..PolicyUniverse::default()
        },
        ..PolicyResolutionInput::default()
    });
    let projection = McpServerProjection::resolve(
        &catalog,
        &policy,
        &McpProjectionScope {
            active_skills: ["peer-skill".to_string()].into(),
        },
    )
    .expect("stdio peer projection resolves");
    let lazy = runtime
        .preflight(&projection, &owner, &environment)
        .await
        .expect("cached stdio catalog supports lazy preflight");
    assert_eq!(stdio_attempts.load(Ordering::SeqCst), 1);
    assert_eq!(receipt_count(&started), 1);

    let echoed = lazy
        .call_tool(
            "stdio-peer__echo",
            serde_json::json!({"text": "peer-roundtrip"}),
        )
        .await
        .expect("lazy stdio call reconnects and succeeds");
    assert!(echoed.to_string().contains("peer-roundtrip"), "{echoed}");
    assert_eq!(stdio_attempts.load(Ordering::SeqCst), 2);
    wait_for_receipts(&started, 2, "lazy call reconnect start").await;
    runtime.shutdown().await.expect("stdio runtime shuts down");
    wait_for_receipts(&stopped, 2, "lazy runtime shutdown stop").await;

    let delayed_environment = Arc::new(
        McpBindingEnvironment::new(
            receipt_directory.path().to_path_buf(),
            BTreeMap::from([
                (std::ffi::OsString::from("PATH"), path),
                (
                    std::ffi::OsString::from("MCP_FIXTURE_RECEIPT_DIR"),
                    receipt_value,
                ),
                (
                    std::ffi::OsString::from("MCP_FIXTURE_INITIALIZE_DELAY_MS"),
                    std::ffi::OsString::from("300"),
                ),
            ]),
        )
        .expect("delayed stdio binding environment is valid"),
    );
    let delayed_definition = Arc::new(
        ServerDefinition::new(
            "stdio-cancel".to_string(),
            ServerSource::Skill {
                skill_id: "peer-skill".to_string(),
            },
            McpServerEntry::Stdio {
                command: "python3".to_string(),
                args: vec![fixture.display().to_string()],
                env: HashMap::from([
                    (
                        "MCP_FIXTURE_RECEIPT_DIR".to_string(),
                        "${MCP_FIXTURE_RECEIPT_DIR}".to_string(),
                    ),
                    (
                        "MCP_FIXTURE_INITIALIZE_DELAY_MS".to_string(),
                        "${MCP_FIXTURE_INITIALIZE_DELAY_MS}".to_string(),
                    ),
                ]),
                sandboxed: false,
            },
            true,
            ServerAuthentication::NotRequired,
        )
        .expect("delayed stdio definition is valid"),
    );
    let delayed_request = Arc::new(McpBindingRequest::new(
        owner,
        delayed_definition,
        delayed_environment,
    ));
    let delayed_runtime = McpRuntimeManager::new(
        McpBindingCache::default(),
        Arc::new(ConfiguredMcpConnector::default()),
        Duration::from_secs(2),
        Duration::from_secs(2),
    )
    .expect("delayed stdio runtime is valid");
    let cancelled = tokio::spawn({
        let runtime = delayed_runtime.clone();
        let request = Arc::clone(&delayed_request);
        async move { runtime.prepare(request).await }
    });
    wait_for_receipts(&started, 3, "cancelled initialization start").await;
    cancelled.abort();
    assert!(
        cancelled
            .await
            .expect_err("cancelled stdio preparation stops")
            .is_cancelled()
    );

    delayed_runtime
        .prepare(delayed_request)
        .await
        .expect("cancelled stdio refresh is retried successfully");
    wait_for_receipts(&started, 4, "post-cancellation retry start").await;
    delayed_runtime
        .shutdown()
        .await
        .expect("delayed stdio runtime shuts down");
    wait_for_receipts(&stopped, 4, "delayed runtime shutdown stop").await;

    assert_eq!(receipt_count(&started), 4);
    assert_eq!(receipt_count(&stopped), 4);
}
