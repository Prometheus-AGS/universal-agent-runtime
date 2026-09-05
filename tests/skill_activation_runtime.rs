use std::collections::BTreeSet;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures::StreamExt;
use tokio::sync::RwLock;
use universal_agent_runtime::config::{
    HarnessConfig, LlmConfig, SkillActivationMode, SkillReattachmentBudget,
};
use universal_agent_runtime::llm::mock_driver::MockLlmDriver;
use universal_agent_runtime::llm::{ExternalDriverStream, LlmDriver, LlmRequest};
use universal_agent_runtime::mcp::binding_cache::{McpBindingCache, McpBindingEnvironment};
use universal_agent_runtime::mcp::catalog::{
    McpCatalog, ServerAuthentication, ServerDefinition, ServerSource,
};
use universal_agent_runtime::mcp::config::{McpConfig, McpServerEntry};
use universal_agent_runtime::mcp::registry::McpRegistry;
use universal_agent_runtime::mcp::runtime::{
    ConfiguredMcpConnector, McpRunResources, McpRuntimeManager,
};
use universal_agent_runtime::normalized::NormalizedEvent as DriverEvent;
use universal_agent_runtime::session::SessionStore;
use universal_agent_runtime::uar::context::ContextStrategy;
use universal_agent_runtime::uar::defaults::default_agent;
use universal_agent_runtime::uar::domain::events::NormalizedEvent as RunEvent;
use universal_agent_runtime::uar::domain::skills::Skill;
use universal_agent_runtime::uar::rag::embeddings::{
    EmbeddingBackend, UnavailableEmbeddingBackend,
};
use universal_agent_runtime::uar::runtime::context::token_service::TokenService;
use universal_agent_runtime::uar::runtime::manager::RunManager;
use universal_agent_runtime::uar::runtime::matching::VectorMatcher;
use universal_agent_runtime::uar::runtime::native_skill::NativeSkillRegistry;
use universal_agent_runtime::uar::runtime::skills::SkillRegistry;
use universal_agent_runtime::uar::runtime::skills::catalog::{
    CatalogBudget, CatalogEntry, render_catalog,
};
use universal_agent_runtime::uar::runtime::skills::service::{
    SkillMatchingAlgorithm, SkillMatchingConfig, SkillService,
};
use universal_agent_runtime::uar::runtime::turn::RunExecutionRequest;
use universal_agent_runtime::uar::security::claims::{UserClaims, UserContext};

#[derive(Debug)]
struct TelemetryMockDriver {
    inner: Arc<MockLlmDriver>,
    provider: &'static str,
    model: &'static str,
}

#[async_trait]
impl LlmDriver for TelemetryMockDriver {
    async fn stream(&self, request: LlmRequest) -> anyhow::Result<ExternalDriverStream> {
        let stream = self.inner.stream(request).await?;
        let provider = self.provider;
        let model = self.model;
        Ok(Box::pin(stream.map(move |event| {
            if let Ok(DriverEvent::Usage {
                prompt_tokens,
                completion_tokens,
                ..
            }) = &event
            {
                universal_agent_runtime::uar::telemetry::metrics::record_llm_tokens(
                    provider,
                    model,
                    u64::from(*prompt_tokens),
                    u64::from(*completion_tokens),
                );
            }
            event
        })))
    }
}

fn metric_value(body: &str, name: &str, labels: &[(&str, &str)]) -> f64 {
    body.lines()
        .filter(|line| line.starts_with(name))
        .find(|line| {
            labels
                .iter()
                .all(|(key, value)| line.contains(&format!(r#"{key}="{value}""#)))
        })
        .and_then(|line| line.split_whitespace().last())
        .and_then(|value| value.parse().ok())
        .unwrap_or(0.0)
}

fn skill(id: &str, body: &str, enabled: bool) -> Skill {
    Skill {
        skill_id: id.to_string(),
        title: id.to_string(),
        description: format!("Description for {id}"),
        prompt_overlay: body.to_string(),
        enabled,
        ..Skill::default()
    }
}

async fn test_manager(skills: Vec<Skill>, driver: Arc<MockLlmDriver>) -> Arc<RunManager> {
    let registry = Arc::new(RwLock::new(SkillRegistry::default()));
    for skill in skills {
        registry.write().await.register_loaded(skill);
    }
    let embeddings: Arc<dyn EmbeddingBackend> = Arc::new(UnavailableEmbeddingBackend::new(
        384,
        "embeddings are not exercised by this test",
    ));
    Arc::new(
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
        .with_harness_config(HarnessConfig {
            skill_activation_mode: SkillActivationMode::Catalog,
            ..HarnessConfig::default()
        })
        .with_llm_driver(driver),
    )
}

async fn matching_manager(
    skill: Skill,
    mode: SkillActivationMode,
    driver: Arc<MockLlmDriver>,
) -> Arc<RunManager> {
    let registry = Arc::new(RwLock::new(SkillRegistry::default()));
    registry.write().await.register_loaded(skill.clone());
    let service = Arc::new(SkillService::new(None, None));
    service.register_builtins(vec![skill]).await;
    service
        .set_matching_config(SkillMatchingConfig {
            algorithm: SkillMatchingAlgorithm::Keyword,
            threshold: 0.5,
            margin_threshold: 0.0,
            top_k: 1,
            model_name: None,
        })
        .await;
    let embeddings: Arc<dyn EmbeddingBackend> = Arc::new(UnavailableEmbeddingBackend::new(
        384,
        "embeddings are not exercised by this test",
    ));
    Arc::new(
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
        .with_harness_config(HarnessConfig {
            skill_activation_mode: mode,
            ..HarnessConfig::default()
        })
        .with_skill_service(service)
        .with_llm_driver(driver),
    )
}

async fn wait_for_done(manager: &RunManager, run_id: &str) {
    let result = tokio::time::timeout(std::time::Duration::from_secs(10), async {
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
    .await;
    if result.is_err() {
        let event_types =
            manager
                .history_since(run_id, None)
                .await
                .map_or_else(Vec::new, |events| {
                    events
                        .iter()
                        .filter_map(|event| {
                            serde_json::to_value(&event.event)
                                .ok()?
                                .get("type")?
                                .as_str()
                                .map(str::to_owned)
                        })
                        .collect()
                });
        panic!("run did not complete; retained event types: {event_types:?}");
    }
}

async fn approve_pending_tool_call(manager: &RunManager, run_id: &str, expected_call_id: &str) {
    let approval_id = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            if let Some(approval_id) =
                manager
                    .history_since(run_id, None)
                    .await
                    .and_then(|events| {
                        events.into_iter().find_map(|event| match event.event {
                            RunEvent::ToolCallApprovalRequired {
                                approval_id,
                                tool_call_id,
                                ..
                            } if tool_call_id == expected_call_id => approval_id,
                            _ => None,
                        })
                    })
            {
                return approval_id;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("tool call requests approval");
    assert!(
        manager
            .resolve_approval_request(run_id, Some(&approval_id), true)
            .await,
        "exact pending approval is accepted"
    );
}

fn activation_turn(call_id: &str, skill_id: &str) -> Vec<DriverEvent> {
    let arguments = format!(r#"{{"skill_id":"{skill_id}"}}"#);
    vec![
        DriverEvent::ToolCallDelta {
            call_index: 0,
            id: Some(call_id.to_string()),
            name: Some("activate_skill".to_string()),
            arguments_delta: Some(arguments.clone()),
        },
        DriverEvent::ToolCallComplete {
            call_index: 0,
            id: call_id.to_string(),
            name: "activate_skill".to_string(),
            arguments_json: arguments,
        },
        DriverEvent::Done,
    ]
}

fn final_turn() -> Vec<DriverEvent> {
    vec![
        DriverEvent::MessageDelta {
            text: "done".to_string(),
        },
        DriverEvent::Done,
    ]
}

#[test]
fn two_thousand_skills_fit_by_fair_description_truncation_before_omission() {
    let long_description = "x".repeat(1_024);
    let entries = (0..2_000)
        .map(|index| CatalogEntry {
            skill_id: index.to_string(),
            title: ["Read", "Write", "Search", "Summarize"][index % 4].to_string(),
            source: "u".to_string(),
            description: long_description.clone(),
            suggested: index % 100 == 0,
        })
        .collect::<Vec<_>>();

    let rendered = render_catalog(&entries, "openai/gpt-4o", Some(1_000_000))
        .expect("the 10,000-token catalog budget fits compact ids and real titles");

    assert_eq!(rendered.budget, CatalogBudget::Tokens(10_000));
    assert!(rendered.used_units <= rendered.budget.limit());
    assert_eq!(rendered.included, 2_000);
    assert_eq!(rendered.omitted, 0);
    assert!(!rendered.content.contains("skills omitted"));

    let rendered_ids = rendered
        .content
        .lines()
        .skip(1)
        .map(|line| {
            line.split_once(" | ")
                .or_else(|| line.split_once(' '))
                .unwrap()
                .0
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(rendered_ids.len(), 2_000);
    for entry in &entries {
        assert!(rendered_ids.contains(entry.skill_id.as_str()));
        let line = rendered
            .content
            .lines()
            .find(|line| {
                line.starts_with(&format!("{} ", entry.skill_id))
                    || line.starts_with(&format!("{} | ", entry.skill_id))
            })
            .expect("each skill retains a catalog line");
        assert!(line.contains(&entry.title), "title missing from {line}");
        assert_eq!(line.contains("[suggested]"), entry.suggested);
    }

    assert!(
        rendered.content.lines().skip(1).all(|line| {
            line.split_once(" — ")
                .is_none_or(|(_, description)| description.len() < long_description.len())
        }),
        "descriptions must be truncated before any identity is omitted"
    );
}

#[test]
fn extreme_catalog_pressure_omits_explicitly_without_erasing_retained_metadata() {
    let entries = (0..300)
        .map(|index| CatalogEntry {
            skill_id: format!("skill-{index:04}"),
            title: "Inspect deployment history and explain failed changes".to_string(),
            source: "registered-provider".to_string(),
            description: "Long description ".repeat(100),
            suggested: true,
        })
        .collect::<Vec<_>>();
    let rendered = render_catalog(&entries, "openai/gpt-4o", None).unwrap();
    assert_eq!(rendered.budget, CatalogBudget::Characters(8_000));
    assert!(rendered.used_units <= 8_000);
    assert!(rendered.included > 0 && rendered.omitted > 0);
    assert_eq!(rendered.included + rendered.omitted, entries.len());
    assert!(
        rendered
            .content
            .ends_with(&format!("[{} skills omitted]", rendered.omitted))
    );
    for entry in entries.iter().take(rendered.included) {
        assert!(
            rendered
                .content
                .contains(&format!("{} {} [suggested]", entry.skill_id, entry.title))
        );
    }
    assert!(!rendered.content.contains("Long description"));
}

#[tokio::test]
async fn explicit_attachments_preload_enabled_body_and_reject_disabled_without_widening() {
    let enabled_body = "ENABLED_SKILL_BODY_SENTINEL";
    let disabled_body = "DISABLED_SKILL_BODY_SENTINEL";
    let driver = Arc::new(MockLlmDriver::new(vec![vec![
        DriverEvent::MessageDelta {
            text: "done".to_string(),
        },
        DriverEvent::Done,
    ]]));
    let manager = test_manager(
        vec![
            skill("s1", enabled_body, true),
            skill("disabled", disabled_body, false),
        ],
        Arc::clone(&driver),
    )
    .await;

    let enabled_run = manager
        .start_run_with_skill_attachments(
            default_agent(),
            "use the attachment".to_string(),
            None,
            None,
            vec![],
            vec!["s1".to_string()],
        )
        .await;
    wait_for_done(&manager, &enabled_run).await;

    let disabled_run = manager
        .start_run_with_skill_attachments(
            default_agent(),
            "do not widen".to_string(),
            None,
            None,
            vec![],
            vec!["disabled".to_string()],
        )
        .await;
    wait_for_done(&manager, &disabled_run).await;

    let requests = driver.requests();
    assert_eq!(requests.len(), 2);
    let enabled_request = serde_json::to_string(&requests[0].messages).expect("request serializes");
    let disabled_request =
        serde_json::to_string(&requests[1].messages).expect("request serializes");
    assert!(enabled_request.contains(enabled_body));
    assert!(!disabled_request.contains(disabled_body));

    let disabled = manager
        .get_run(&disabled_run)
        .await
        .expect("disabled attachment run remains readable");
    assert_eq!(
        disabled.context["activation_failures"][0]["code"],
        "ineligible"
    );
    assert_eq!(
        disabled.context["activation_failures"][0]["skill_id"],
        "disabled"
    );
}

#[tokio::test]
async fn model_activation_updates_the_next_step_and_missing_is_a_typed_result() {
    let skill_body = "MODEL_ACTIVATED_SKILL_BODY_SENTINEL";
    let server_name = "skill-server";
    let projected_tool_name = "skill-server__uar_list_agents";
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("peer UAR listener binds");
    let peer_address = listener
        .local_addr()
        .expect("peer UAR address is available");
    let server_config = McpServerEntry::RemoteHttp {
        url: format!("http://{peer_address}/"),
        env: HashMap::new(),
    };
    let driver = Arc::new(MockLlmDriver::new(vec![
        activation_turn("activate-s2", "s2"),
        final_turn(),
        activation_turn("activate-missing", "missing"),
        final_turn(),
    ]));
    let mut activated_skill = skill("s2", skill_body, true);
    activated_skill.preferred_tools = vec![projected_tool_name.to_string()];
    activated_skill.mcp_config = Some(McpConfig {
        mcp_servers: HashMap::from([(server_name.to_string(), server_config.clone())]),
    });
    let manager = test_manager(vec![activated_skill], Arc::clone(&driver)).await;
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
    let definition = ServerDefinition::new(
        server_name.to_string(),
        ServerSource::Skill {
            skill_id: "s2".to_string(),
        },
        server_config,
        true,
        ServerAuthentication::Authenticated {
            binding_id: "test-binding".to_string(),
        },
    )
    .expect("test server definition is valid");
    let catalog =
        Arc::new(McpCatalog::from_definitions([definition]).expect("test MCP catalog is valid"));
    let runtime = McpRuntimeManager::new(
        McpBindingCache::default(),
        Arc::new(ConfiguredMcpConnector::default()),
        Duration::from_secs(1),
        Duration::from_secs(1),
    )
    .expect("test MCP runtime is valid");
    let environment = Arc::new(
        McpBindingEnvironment::new(std::env::temp_dir(), BTreeMap::new())
            .expect("test MCP environment is valid"),
    );
    let user = UserContext {
        user_id: "skill-test-user".to_string(),
        tenant_id: None,
        claims: UserClaims {
            sub: "skill-test-user".to_string(),
            name: None,
            roles: Some(vec!["user".to_string()]),
            tenant_id: None,
            uar_instance_id: None,
            exp: usize::MAX,
        },
    };
    let owner =
        universal_agent_runtime::uar::runtime::actor::messages::ActorOwner::from_verified_context(
            &user,
        )
        .expect("test owner is verified");

    let mut activated_request =
        RunExecutionRequest::new(default_agent(), "activate s2".to_string())
            .with_user_context(&user)
            .expect("test run owner is valid");
    activated_request.mcp_resources = Some(McpRunResources::new(
        owner.clone(),
        runtime.clone(),
        Arc::clone(&catalog),
        Arc::clone(&environment),
    ));
    let activated_run = manager.execute_request(activated_request).await;
    approve_pending_tool_call(&manager, &activated_run, "activate-s2").await;
    wait_for_done(&manager, &activated_run).await;
    let mut missing_request =
        RunExecutionRequest::new(default_agent(), "activate missing".to_string())
            .with_user_context(&user)
            .expect("test run owner is valid");
    missing_request.mcp_resources = Some(McpRunResources::new(
        owner,
        runtime.clone(),
        catalog,
        environment,
    ));
    let missing_run = manager.execute_request(missing_request).await;
    approve_pending_tool_call(&manager, &missing_run, "activate-missing").await;
    wait_for_done(&manager, &missing_run).await;

    let requests = driver.requests();
    assert_eq!(requests.len(), 4);
    let activated_next =
        serde_json::to_string(&requests[1].messages).expect("activated request serializes");
    assert!(
        activated_next.contains(skill_body),
        "activated request did not contain the skill body: {activated_next}"
    );
    let initial_tools = serde_json::to_string(&requests[0].tools).expect("initial tools serialize");
    let activated_tools =
        serde_json::to_string(&requests[1].tools).expect("activated tools serialize");
    assert!(!initial_tools.contains(projected_tool_name));
    assert!(activated_tools.contains(projected_tool_name));

    let missing_next =
        serde_json::to_string(&requests[3].messages).expect("missing result serializes");
    assert!(missing_next.contains("activation_failed"));
    assert!(missing_next.contains("missing"));
    assert!(!missing_next.contains(skill_body));

    runtime
        .shutdown()
        .await
        .expect("projected MCP runtime shuts down");
    peer_server.abort();
    let _ = peer_server.await;
}

#[tokio::test]
async fn third_model_activation_returns_the_max_active_limit_without_widening() {
    let driver = Arc::new(MockLlmDriver::new(vec![
        activation_turn("activate-s1", "s1"),
        activation_turn("activate-s2", "s2"),
        activation_turn("activate-s3", "s3"),
        final_turn(),
    ]));
    let manager = test_manager(
        vec![
            skill("s1", "FIRST_ACTIVE_SKILL_BODY", true),
            skill("s2", "SECOND_ACTIVE_SKILL_BODY", true),
            skill("s3", "REFUSED_SKILL_BODY", true),
        ],
        Arc::clone(&driver),
    )
    .await;
    let mut artifact = default_agent();
    artifact.policy.skills.max_active = 2;

    let run_id = manager
        .start_run(
            artifact,
            "activate three skills".to_string(),
            None,
            None,
            vec![],
        )
        .await;
    approve_pending_tool_call(&manager, &run_id, "activate-s1").await;
    approve_pending_tool_call(&manager, &run_id, "activate-s2").await;
    approve_pending_tool_call(&manager, &run_id, "activate-s3").await;
    wait_for_done(&manager, &run_id).await;

    let requests = driver.requests();
    assert_eq!(requests.len(), 4);
    let refusal = requests[3]
        .messages
        .iter()
        .find(|message| message["tool_call_id"] == "activate-s3")
        .and_then(|message| message["content"].as_str())
        .and_then(|content| serde_json::from_str::<serde_json::Value>(content).ok())
        .expect("third activation returns a JSON tool result");
    assert_eq!(refusal["status"], "activation_failed");
    assert_eq!(refusal["failure"]["code"], "limit_reached");
    assert_eq!(refusal["failure"]["skill_id"], "s3");
    assert_eq!(refusal["failure"]["limit"], 2);
    let final_request =
        serde_json::to_string(&requests[3].messages).expect("final request serializes");
    assert!(final_request.contains("FIRST_ACTIVE_SKILL_BODY"));
    assert!(final_request.contains("SECOND_ACTIVE_SKILL_BODY"));
    assert!(!final_request.contains("REFUSED_SKILL_BODY"));
}

#[tokio::test]
async fn keyword_threshold_and_activation_mode_control_body_loading() {
    async fn requests_for(
        mode: SkillActivationMode,
    ) -> Vec<universal_agent_runtime::llm::LlmRequest> {
        let mut candidate = skill("mode-skill", "MATCHED_SKILL_BODY", true);
        candidate.triggers.keywords = vec!["deploy".to_string()];
        let driver = Arc::new(MockLlmDriver::new(vec![final_turn(), final_turn()]));
        let manager = matching_manager(candidate, mode, Arc::clone(&driver)).await;

        let below = manager
            .start_run(
                default_agent(),
                "Description for mode-skill".to_string(),
                None,
                None,
                vec![],
            )
            .await;
        wait_for_done(&manager, &below).await;
        let above = manager
            .start_run(
                default_agent(),
                "please deploy now".to_string(),
                None,
                None,
                vec![],
            )
            .await;
        wait_for_done(&manager, &above).await;
        driver.requests()
    }

    let legacy = requests_for(SkillActivationMode::LegacyOverlay).await;
    let catalog = requests_for(SkillActivationMode::Catalog).await;
    assert_eq!(legacy.len(), 2);
    assert_eq!(catalog.len(), 2);

    let legacy_below = serde_json::to_string(&legacy[0].messages).expect("request serializes");
    let legacy_above = serde_json::to_string(&legacy[1].messages).expect("request serializes");
    let catalog_below = serde_json::to_string(&catalog[0].messages).expect("request serializes");
    let catalog_above = serde_json::to_string(&catalog[1].messages).expect("request serializes");

    assert!(!legacy_below.contains("MATCHED_SKILL_BODY"));
    assert!(!legacy_below.contains("[suggested]"));
    assert!(!catalog_below.contains("MATCHED_SKILL_BODY"));
    assert!(!catalog_below.contains("[suggested]"));
    assert!(legacy_above.contains("MATCHED_SKILL_BODY"));
    assert!(!catalog_above.contains("MATCHED_SKILL_BODY"));
    assert!(catalog_above.contains("[suggested]"));
}

#[tokio::test]
async fn compaction_reattaches_the_latest_skill_body_within_budget() {
    let body = format!(
        "LATEST_SKILL_PREFIX {} LATEST_SKILL_TAIL",
        "bounded body ".repeat(500)
    );
    let driver = Arc::new(MockLlmDriver::new(vec![
        activation_turn("activate-latest", "latest"),
        final_turn(),
    ]));
    let registry = Arc::new(RwLock::new(SkillRegistry::default()));
    registry
        .write()
        .await
        .register_loaded(skill("latest", &body, true));
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
        .with_harness_config(HarnessConfig {
            skill_activation_mode: SkillActivationMode::Catalog,
            skill_reattachment: SkillReattachmentBudget {
                per_skill_tokens: 64,
                total_tokens: 64,
            },
            ..HarnessConfig::default()
        })
        .with_message_context_strategy(ContextStrategy::SlidingWindow { max_messages: 1 })
        .with_llm_driver(driver.clone()),
    );

    let run_id = manager
        .start_run(
            default_agent(),
            "USER_TURN_REMOVED_BY_COMPACTION".to_string(),
            None,
            None,
            vec![],
        )
        .await;
    approve_pending_tool_call(&manager, &run_id, "activate-latest").await;
    wait_for_done(&manager, &run_id).await;

    let requests = driver.requests();
    assert_eq!(requests.len(), 2);
    let first = serde_json::to_string(&requests[0].messages).expect("first request serializes");
    let compacted =
        serde_json::to_string(&requests[1].messages).expect("compacted request serializes");
    assert!(first.contains("USER_TURN_REMOVED_BY_COMPACTION"));
    assert!(!compacted.contains("USER_TURN_REMOVED_BY_COMPACTION"));
    assert!(compacted.contains("LATEST_SKILL_PREFIX"));
    assert!(!compacted.contains(&body));

    let reattached = requests[1]
        .messages
        .iter()
        .filter_map(|message| message["content"].as_str())
        .find(|content| content.contains("LATEST_SKILL_PREFIX"))
        .expect("latest skill body is reattached after compaction");
    assert!(TokenService::count("openai/gpt-4o", reattached) <= 64);
}

#[tokio::test]
async fn overlay_only_outcomes_and_multi_skill_usage_are_attributed_without_double_counting() {
    const PROVIDER: &str = "skill-attribution-provider";
    const MODEL: &str = "skill-attribution-model";
    const S1: &str = "attribution-s1";
    const S2: &str = "attribution-s2";

    universal_agent_runtime::uar::telemetry::metrics::init();
    let before = universal_agent_runtime::uar::telemetry::metrics::metrics_handle().render();
    let mock = Arc::new(MockLlmDriver::new(vec![vec![
        DriverEvent::Usage {
            prompt_tokens: 1_000,
            completion_tokens: 0,
            total_tokens: 1_000,
            cached_tokens: None,
            cache_creation_tokens: None,
        },
        DriverEvent::Done,
    ]]));
    let driver: Arc<dyn LlmDriver> = Arc::new(TelemetryMockDriver {
        inner: Arc::clone(&mock),
        provider: PROVIDER,
        model: MODEL,
    });
    let registry = Arc::new(RwLock::new(SkillRegistry::default()));
    registry
        .write()
        .await
        .register_loaded(skill(S1, "OVERLAY_ONLY_ONE", true));
    registry
        .write()
        .await
        .register_loaded(skill(S2, "OVERLAY_ONLY_TWO", true));
    let embeddings: Arc<dyn EmbeddingBackend> = Arc::new(UnavailableEmbeddingBackend::new(
        384,
        "embeddings are not exercised by this test",
    ));
    let manager = Arc::new(
        RunManager::new(
            LlmConfig {
                model: format!("{PROVIDER}/{MODEL}"),
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
        .with_harness_config(HarnessConfig {
            skill_activation_mode: SkillActivationMode::Catalog,
            ..HarnessConfig::default()
        })
        .with_llm_driver(driver),
    );

    let run_id = manager
        .start_run_with_skill_attachments(
            default_agent(),
            "attribute one provider request".to_string(),
            None,
            None,
            vec![],
            vec![S1.to_string(), S2.to_string()],
        )
        .await;
    wait_for_done(&manager, &run_id).await;

    let after = universal_agent_runtime::uar::telemetry::metrics::metrics_handle().render();
    for skill_id in [S1, S2] {
        let outcome_before = metric_value(
            &before,
            "uar_skill_activation_outcome_total",
            &[("skill_id", skill_id), ("success", "true")],
        );
        let outcome_after = metric_value(
            &after,
            "uar_skill_activation_outcome_total",
            &[("skill_id", skill_id), ("success", "true")],
        );
        assert_eq!(outcome_after - outcome_before, 1.0);

        let tokens_before = metric_value(
            &before,
            "uar_skill_request_tokens_total",
            &[("skill", skill_id)],
        );
        let tokens_after = metric_value(
            &after,
            "uar_skill_request_tokens_total",
            &[("skill", skill_id)],
        );
        assert_eq!(tokens_after - tokens_before, 1_000.0);
    }
    let global_before = metric_value(
        &before,
        "uar_llm_tokens_total",
        &[
            ("provider", PROVIDER),
            ("model", MODEL),
            ("direction", "input"),
        ],
    );
    let global_after = metric_value(
        &after,
        "uar_llm_tokens_total",
        &[
            ("provider", PROVIDER),
            ("model", MODEL),
            ("direction", "input"),
        ],
    );
    assert_eq!(global_after - global_before, 1_000.0);
}
