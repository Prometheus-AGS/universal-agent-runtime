//! Dot-04 production boundaries: actual MCP/Liter transports, host admission,
//! canonical persistence, ToolEnd projection, and fresh-process observation.
#![cfg(all(feature = "server", feature = "surreal-backend"))]

#[path = "support/truthful_tool_outcome_peer.rs"]
mod peer;

#[path = "support/truthful_tool_outcome_process.rs"]
mod process;

use axum::{
    Extension, Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tower::ServiceExt;
use universal_agent_runtime::{
    config::{LlmConfig, ServiceInstanceConfig},
    mcp::{
        outcome::{McpToolFailure, ToolOutcome},
        registry::McpRegistry,
    },
    session::SessionStore,
    uar::{
        api::routes::{self, RunApiState},
        compiler::collaboration::{
            CollaborationCatalogService, CollaborationStorage, SurrealCollaborationStorage,
        },
        defaults::default_agent,
        domain::events::NormalizedEvent as RunEvent,
        persistence::{
            PersistenceLayer,
            agent_threads::{CanonicalReceiptSource, CanonicalReceiptStoreError},
            providers::surreal::SurrealDbProvider,
            tool_admission::ToolAdmissionEvidenceState,
        },
        rag::embeddings::{UnavailableEmbeddingBackend, swappable::SwappableEmbeddingBackend},
        runtime::{
            manager::RunManager,
            matching::{VectorMatcher, intent::ClassifierConfig},
            native_skill::NativeSkillRegistry,
            skills::SkillRegistry,
        },
        service_instance::ServiceInstanceAuthority,
    },
};

const CHILD: &str = "UAR_DOT04_CHILD";
const ROOT: &str = "UAR_DOT04_ROOT";
const EVIDENCE_ROOT: &str = "UAR_DOT04_EVIDENCE_ROOT";
const WAIT: Duration = Duration::from_secs(60);

fn expected(mode: &str) -> ToolOutcome {
    match mode {
        "success" => ToolOutcome::Succeeded,
        "application" => ToolOutcome::ApplicationFailed,
        "predispatch" => ToolOutcome::TransportFailed,
        "unknown" => ToolOutcome::OutcomeUnknown,
        _ => panic!("unrecognized fixture scenario"),
    }
}

fn evidence(outcome: ToolOutcome) -> ToolAdmissionEvidenceState {
    match outcome {
        ToolOutcome::Succeeded => ToolAdmissionEvidenceState::Succeeded,
        ToolOutcome::ApplicationFailed => ToolAdmissionEvidenceState::ApplicationFailed,
        ToolOutcome::TransportFailed => ToolAdmissionEvidenceState::TransportFailed,
        ToolOutcome::OutcomeUnknown => ToolAdmissionEvidenceState::OutcomeUnknown,
    }
}

struct Stores {
    runtime: Arc<dyn PersistenceLayer>,
    catalog: Arc<dyn CollaborationStorage>,
}

async fn store(root: &Path, shared: bool) -> Stores {
    let provider = if shared {
        let path = std::env::var("UAR_DOT04_INFRA_FILE")
            .expect("shared gate requires private infrastructure manifest");
        let input: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let infrastructure = &input["surreal"];
        let namespace = std::env::var("UAR_DOT04_NAMESPACE").unwrap();
        Arc::new(
            SurrealDbProvider::new(
                infrastructure["url"].as_str().unwrap(),
                infrastructure["username"].as_str(),
                infrastructure["password"].as_str(),
                Some(&namespace),
                Some("truthful_outcomes"),
            )
            .await
            .expect("isolated shared durable store"),
        )
    } else {
        Arc::new(
            SurrealDbProvider::new(
                &format!("surrealkv://{}", root.join("database").display()),
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap(),
        )
    };
    let catalog = Arc::new(SurrealCollaborationStorage::new(
        provider.client(),
        if shared { "surrealdb" } else { "surrealkv" },
    ));
    Stores {
        runtime: provider,
        catalog,
    }
}

fn llm(peer: &peer::Peer) -> LlmConfig {
    LlmConfig {
        model: "openai/gpt-4o-mini".into(),
        base_url: Some(peer.url.clone()),
        api_key: Some(peer::SECRET.into()),
        host_supplied_connection: true,
        ..LlmConfig::default()
    }
}

fn fixture_peer_pids(trace: &Path) -> Vec<u32> {
    let output = Command::new("/bin/ps")
        .args(["-axo", "pid=,args="])
        .output()
        .unwrap();
    assert!(output.status.success());
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/truthful_tool_outcome_mcp.py");
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let pid = fields.next()?.parse().ok()?;
            let arguments = fields.collect::<Vec<_>>();
            (arguments.contains(&script.to_str().unwrap())
                && arguments.contains(&trace.to_str().unwrap()))
            .then_some(pid)
        })
        .collect()
}

async fn await_fixture_peer_exit(trace: &Path) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    while !fixture_peer_pids(trace).is_empty() {
        assert!(
            tokio::time::Instant::now() < deadline,
            "fixture MCP peer did not close"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn close_fixture_peer_before_dispatch(registry: &McpRegistry, trace: &Path) {
    assert!(registry.resolve_mcp_tool(peer::TOOL).is_some());
    let pids = fixture_peer_pids(trace);
    assert!(!pids.is_empty(), "real discovered MCP peer must exist");
    for pid in pids {
        assert!(
            Command::new("/bin/kill")
                .arg("-TERM")
                .arg(pid.to_string())
                .status()
                .unwrap()
                .success()
        );
    }
    await_fixture_peer_exit(trace).await;
    // Preserve discovery and let the established rmcp reader publish the closed
    // queue before dispatch; the typed outcome and zero-effect assertions follow.
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(registry.resolve_mcp_tool(peer::TOOL).is_some());
}

async fn remove_fixture_server(registry: &McpRegistry, trace: &Path) {
    assert!(registry.remove_server("outcomes"));
    await_fixture_peer_exit(trace).await;
}

async fn registry_boundary(root: &Path, scoped: bool, mode: &str) {
    let trace = root.join(format!("registry-{scoped}-{mode}.jsonl"));
    let (registry, runtime, preflight) = if scoped {
        let (registry, runtime, preflight) = peer::scoped(&trace).await;
        (registry, Some(runtime), Some(preflight))
    } else {
        (
            McpRegistry::from_config(&peer::configuration(&trace))
                .await
                .unwrap(),
            None,
            None,
        )
    };
    if mode == "predispatch" {
        if let Some(runtime) = &runtime {
            runtime.shutdown().await.unwrap();
        }
        close_fixture_peer_before_dispatch(&registry, &trace).await;
    }
    let result = match &preflight {
        Some(preflight) => {
            preflight
                .call_tool(peer::TOOL, json!({"mode":mode}), None)
                .await
        }
        None => {
            registry
                .call_namespaced_tool(peer::TOOL, json!({"mode":mode}))
                .await
        }
    };
    assert_eq!(ToolOutcome::from_mcp_result(&result), expected(mode));
    match mode {
        "success" => peer::assert_content(result.as_ref().unwrap(), mode),
        "application" => {
            let error = result.as_ref().unwrap_err();
            let failure =
                McpToolFailure::find(error).expect("typed failure survives scoped source chain");
            peer::assert_content(failure.result.as_ref().unwrap(), mode);
            assert!(
                !format!("{error:?}").contains("fixture-response-control"),
                "Debug leaked server content"
            );
        }
        "unknown" => assert!(
            McpToolFailure::find(result.as_ref().unwrap_err())
                .unwrap()
                .result
                .is_none()
        ),
        _ => {}
    }
    if let Some(runtime) = runtime {
        runtime.shutdown().await.unwrap();
    }
    remove_fixture_server(&registry, &trace).await;
    assert_eq!(
        peer::effects(&trace).len(),
        usize::from(mode != "predispatch"),
        "external effect was replayed or predispatch rejection executed"
    );
}

async fn activation_boundary(root: &Path, mode: &str) {
    use std::{
        collections::{BTreeMap, HashMap},
        ffi::OsString,
    };
    use universal_agent_runtime::{
        mcp::{
            binding_cache::{McpBindingCache, McpBindingEnvironment},
            catalog::{McpCatalog, ServerAuthentication, ServerDefinition, ServerSource},
            config::McpConfig,
            runtime::{ConfiguredMcpConnector, McpRuntimeManager},
        },
        uar::{
            domain::{
                policy::{
                    PolicyResolutionInput, PolicyUniverse, ResourceSelection, RunPolicy,
                    resolve_run_policy,
                },
                skills::{Skill, SkillKind},
            },
            runtime::{
                actor::messages::ActorOwner,
                skills::activation::{
                    ActivationContext, ActivationFailure, InvokeType, ProjectedActivationHost,
                    activate,
                },
            },
        },
    };
    let trace = root.join(format!("activation-{mode}.jsonl"));
    let server = "dot04__outcomes";
    let tool = "dot04__outcomes__effect";
    let missing = "dot04__outcomes__missing";
    let entry = peer::configuration(&trace).mcp_servers["outcomes"].clone();
    let configuration = McpConfig {
        mcp_servers: HashMap::from([(server.into(), entry.clone())]),
    };
    let mut registry = SkillRegistry::default();
    registry.register_loaded(Skill {
        skill_id: "dot04".into(),
        enabled: true,
        kind: SkillKind::Manifest,
        required_tools: vec![if mode == "missing" { missing } else { tool }.into()],
        mcp_config: Some(configuration),
        ..Skill::default()
    });
    let definition = ServerDefinition::new(
        server.into(),
        ServerSource::Skill {
            skill_id: "dot04".into(),
        },
        entry,
        true,
        ServerAuthentication::NotRequired,
    )
    .unwrap();
    let policy = resolve_run_policy(PolicyResolutionInput {
        global: Some(RunPolicy {
            skills: ResourceSelection::selected(["dot04".into()]),
            tools: ResourceSelection::selected([tool.into(), missing.into()]),
            mcp_servers: ResourceSelection::selected(if mode == "denied" {
                Vec::new()
            } else {
                vec![server.into()]
            }),
            ..RunPolicy::default()
        }),
        universe: PolicyUniverse {
            skills: ["dot04".into()].into(),
            tools: [tool.into(), missing.into()].into(),
            mcp_servers: [server.into()].into(),
            ..PolicyUniverse::default()
        },
        ..PolicyResolutionInput::default()
    });
    let runtime = McpRuntimeManager::new(
        McpBindingCache::default(),
        Arc::new(ConfiguredMcpConnector::default()),
        Duration::from_secs(15),
        Duration::from_secs(15),
    )
    .unwrap();
    let environment = Arc::new(
        McpBindingEnvironment::new(
            root.to_path_buf(),
            BTreeMap::from([(OsString::from("PATH"), OsString::from("/usr/bin:/bin"))]),
        )
        .unwrap(),
    );
    let owner = ActorOwner::from_verified_context(&peer::user().await).unwrap();
    let host = ProjectedActivationHost::new(
        runtime.clone(),
        Arc::new(McpCatalog::from_definitions([definition]).unwrap()),
        policy,
        owner.clone(),
        environment,
        tokio_util::sync::CancellationToken::new(),
    );
    let mut context = ActivationContext::new_projected(
        Arc::new(RwLock::new(registry)),
        "dot04-agent".into(),
        "dot04-session".into(),
        1,
        McpRegistry::empty(),
        Vec::new(),
        host,
    )
    .await
    .unwrap();
    assert!(context.active().is_empty());
    assert!(
        context
            .mcp_preflight()
            .unwrap()
            .projection()
            .tools()
            .is_empty()
    );
    assert!(context.mcp_preflight().unwrap().servers().is_empty());
    assert!(
        fixture_peer_pids(&trace).is_empty(),
        "inactive skill dependency was started"
    );
    let result = activate("dot04", &mut context, InvokeType::Attachment).await;
    if mode == "eligible" {
        result.unwrap();
        assert_eq!(context.active().len(), 1);
        let preflight = context.mcp_preflight().unwrap();
        assert_eq!(preflight.owner(), &owner);
        assert!(preflight.projection().tools().contains_key(tool));
        let frozen = context.freeze_mcp_bindings().await.unwrap();
        assert_eq!(
            frozen.resolve_mcp_tool(tool),
            Some((server.into(), "effect".into()))
        );
        peer::assert_content(
            &preflight
                .call_tool(tool, json!({"mode":"success"}), None)
                .await
                .unwrap(),
            "success",
        );
    } else {
        let error = result.unwrap_err();
        assert!(matches!(
            &error,
            ActivationFailure::DependencyInvalid { .. }
        ));
        let message = error.to_string();
        assert!(message.contains(if mode == "denied" {
            "not eligible"
        } else {
            "unavailable"
        }));
        assert!(
            context.active().is_empty(),
            "failed activation was committed"
        );
        assert!(
            context
                .mcp_preflight()
                .unwrap()
                .projection()
                .tools()
                .is_empty()
        );
        assert!(
            context.mcp_preflight().unwrap().servers().is_empty(),
            "prospective binding was committed"
        );
        assert_eq!(
            fixture_peer_pids(&trace).len(),
            usize::from(mode == "missing"),
            "denied dependency started or missing-tool control never discovered its real server"
        );
    }
    runtime.shutdown().await.unwrap();
    await_fixture_peer_exit(&trace).await;
    assert_eq!(
        peer::effects(&trace).len(),
        usize::from(mode == "eligible"),
        "activation failure executed or replayed an effect"
    );
    let path = root.join(format!("activation-{mode}.json"));
    std::fs::write(&path, serde_json::to_vec_pretty(&json!({
        "boundary":"production_activation_host","mode":mode,"server":server,"tool":tool,
        "activeSkills":context.active().len(),
        "preparedTools":context.mcp_preflight().unwrap().projection().tools().keys().collect::<Vec<_>>(),
        "effects":peer::effects(&trace).len(),"mcpChildrenExited":true
    })).unwrap()).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
}

async fn assert_receipt(store: &Arc<dyn PersistenceLayer>, run: &str, mode: &str) {
    let receipts = store
        .list_canonical_tool_receipts(peer::OWNER, run)
        .await
        .unwrap();
    assert_eq!(
        receipts.len(),
        1,
        "actual execution must produce one durable receipt"
    );
    let receipt = &receipts[0];
    assert_eq!(receipt.source, CanonicalReceiptSource::Mcp);
    assert_eq!(receipt.outcome, Some(expected(mode)));
    let value = receipt.typed_value.as_ref().unwrap();
    if mode == "success" {
        peer::assert_content(value, mode);
    } else {
        assert_eq!(
            value["outcome"],
            serde_json::to_value(expected(mode)).unwrap()
        );
        if mode == "application" {
            peer::assert_content(&value["result"], mode);
        } else {
            assert!(value["result"].is_null());
        }
    }
    let same = store.save_canonical_tool_receipt(receipt).await.unwrap();
    assert_eq!(&same, receipt, "same result receipt must be idempotent");
    let mut contradictory = receipt.clone();
    contradictory.outcome = Some(if mode == "success" {
        ToolOutcome::ApplicationFailed
    } else {
        ToolOutcome::Succeeded
    });
    let error = store
        .save_canonical_tool_receipt(&contradictory)
        .await
        .unwrap_err();
    assert!(
        error
            .downcast_ref::<CanonicalReceiptStoreError>()
            .is_some_and(|error| matches!(error, CanonicalReceiptStoreError::Conflict)),
        "outcome reinterpretation must conflict"
    );
    let records = store
        .list_tool_admission_evidence(peer::OWNER)
        .await
        .unwrap();
    let terminal = records
        .iter()
        .filter(|record| record.run_id == run && record.state == evidence(expected(mode)))
        .count();
    assert_eq!(terminal, 1, "admission outcome disagrees with receipt");
    if mode != "success" {
        assert!(
            !records.iter().any(|record| record.run_id == run
                && record.state == ToolAdmissionEvidenceState::Succeeded)
        );
    }
}

async fn legacy_receipt_is_not_migrated_to_success(store: &Arc<dyn PersistenceLayer>) {
    use universal_agent_runtime::uar::persistence::agent_threads::CanonicalToolReceipt;
    let legacy = CanonicalToolReceipt::acquire(
        peer::OWNER,
        "dot04-legacy",
        0,
        "legacy-call",
        peer::TOOL,
        CanonicalReceiptSource::Mcp,
        json!({"isError":true,"content":[{"type":"text","text":"historical-original-error"}]}),
        Vec::<(String, Vec<u8>)>::new(),
        true,
        0,
    )
    .unwrap();
    let stored = store.save_canonical_tool_receipt(&legacy).await.unwrap();
    assert!(stored.outcome.is_none());
    let mut reinterpretation = stored.clone();
    reinterpretation.outcome = Some(ToolOutcome::Succeeded);
    let error = store
        .save_canonical_tool_receipt(&reinterpretation)
        .await
        .unwrap_err();
    assert!(
        error
            .downcast_ref::<CanonicalReceiptStoreError>()
            .is_some_and(|error| matches!(error, CanonicalReceiptStoreError::Conflict))
    );
    let loaded = store
        .list_canonical_tool_receipts(peer::OWNER, "dot04-legacy")
        .await
        .unwrap();
    assert_eq!(
        loaded,
        vec![legacy],
        "historical content or certainty was rewritten"
    );
}

async fn direct_run(root: &Path, store: &Arc<dyn PersistenceLayer>, mode: &str) -> String {
    let trace = root.join(format!("manager-{mode}.jsonl"));
    let registry = Arc::new(
        McpRegistry::from_config(&peer::configuration(&trace))
            .await
            .unwrap(),
    );
    if mode == "predispatch" {
        close_fixture_peer_before_dispatch(&registry, &trace).await;
    }
    let peer = peer::Peer::start(root).await;
    let manager = Arc::new(
        RunManager::with_classifier_config(
            llm(&peer),
            registry.clone(),
            SessionStore::new(),
            Arc::new(RwLock::new(SkillRegistry::default())),
            Arc::new(VectorMatcher::new(
                Arc::new(SwappableEmbeddingBackend::new(Arc::new(UnavailableEmbeddingBackend::new(
                    384,
                    "outcome fixture does not use embeddings",
                )))),
                0.75,
            )),
            Some(store.clone()),
            ClassifierConfig::default(),
            Arc::new(NativeSkillRegistry::new()),
        )
        .await,
    );
    let state = Arc::new(RunApiState {
        manager: manager.clone(),
        collaboration_catalog: Arc::new(CollaborationCatalogService::in_memory()),
        service_instance: Arc::new(ServiceInstanceAuthority::new(
            &ServiceInstanceConfig::default(),
            "dot04-instance",
            "http://127.0.0.1",
            false,
            &[],
        )),
    });
    let app = Router::new()
        .nest("/api/uar", routes::build_router().with_state(state))
        .layer(Extension(peer::user().await));
    let mut artifact = default_agent();
    artifact.id = format!("outcomes-{mode}");
    artifact.memory.kb.enabled = false;
    artifact.policy.tools.allow = vec![peer::TOOL.into()];
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/uar/runs")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"artifact":artifact,"input":mode}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let admitted: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 1024 * 1024).await.unwrap())
            .unwrap();
    let run = admitted["run_id"].as_str().unwrap().to_owned();
    tokio::time::timeout(WAIT, async {
        loop {
            let history = manager.history_since(&run, None).await.unwrap();
            for event in &history {
                if let RunEvent::ToolCallApprovalRequired {
                    approval_id: Some(id),
                    ..
                } = &event.event
                {
                    manager.resolve_approval_request(&run, Some(id), true).await;
                }
            }
            if history
                .iter()
                .any(|event| matches!(&event.event, RunEvent::ToolEnd { .. }))
                && history.iter().any(|event| {
                    matches!(
                        &event.event,
                        RunEvent::RunDone { .. }
                            | RunEvent::RunDoneWithUsage { .. }
                            | RunEvent::Error { .. }
                    )
                })
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("actual run reached tool and terminal boundary");
    let history = manager.history_since(&run, None).await.unwrap();
    let (output, ok) = history
        .iter()
        .find_map(|event| match &event.event {
            RunEvent::ToolEnd {
                tool, output, ok, ..
            } if tool == peer::TOOL => Some((output, *ok)),
            _ => None,
        })
        .unwrap();
    assert_eq!(ok, expected(mode).succeeded());
    let event_text = serde_json::to_string(output).unwrap();
    if mode == "application" {
        assert!(event_text.contains("fixture-response-control"));
        assert!(event_text.contains("application_failed"));
    }
    if mode == "unknown" {
        assert!(event_text.contains("outcome_unknown"));
        assert!(history.iter().any(|event| matches!(&event.event, RunEvent::Error { code, .. } if code == "TOOL_OUTCOME_UNKNOWN")));
    }
    assert_eq!(
        peer.seen.requests.lock().unwrap().len(),
        if mode == "unknown" { 1 } else { 2 },
        "unknown result triggered a model retry turn"
    );
    assert_receipt(store, &run, mode).await;
    remove_fixture_server(&registry, &trace).await;
    assert_eq!(
        peer::effects(&trace).len(),
        usize::from(mode != "predispatch")
    );
    run
}

async fn observe(root: &Path, store: &Arc<dyn PersistenceLayer>) {
    let legacy = store
        .list_canonical_tool_receipts(peer::OWNER, "dot04-legacy")
        .await
        .unwrap();
    assert_eq!(legacy.len(), 1);
    assert!(legacy[0].outcome.is_none());
    assert_eq!(
        legacy[0].typed_value.as_ref().unwrap()["content"][0]["text"],
        "historical-original-error"
    );
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(root.join("receipts.json")).unwrap()).unwrap();
    for scenario in manifest.as_array().unwrap() {
        let mode = scenario["mode"].as_str().unwrap();
        let scoped_store;
        let receipt_store = if scenario["scopedDatabase"] == true {
            let shared = std::env::var("UAR_DOT04_SHARED").as_deref() == Ok("1");
            scoped_store = process::scoped_outcome_store(root, shared, mode).await;
            &scoped_store
        } else {
            store
        };
        assert_receipt(receipt_store, scenario["run"].as_str().unwrap(), mode).await;
        if scenario["scopedDatabase"] == true {
            process::assert_scoped_effect_count(root, mode);
        }
    }
    for trace in std::fs::read_dir(root)
        .unwrap()
        .map(Result::unwrap)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "jsonl"))
    {
        assert!(
            peer::effects(&trace.path()).len() <= 1,
            "observing durable outcomes replayed an effect"
        );
    }
}

async fn settle_dependency(
    service: &CollaborationCatalogService,
    storage: &Arc<dyn CollaborationStorage>,
    store: &Arc<dyn PersistenceLayer>,
    run: &str,
    mode: &str,
    source: &str,
) {
    use universal_agent_runtime::uar::domain::{
        collaboration::ImmutableDefinitionRef,
        team_execution::{TeamExecutionAttempt, TeamReservation},
        team_planning::{TeamBindingRef, TeamInstance, TeamTask},
    };
    let now = chrono::Utc::now();
    let team_id = format!("{source}-{mode}");
    let task = |id: &str, dependencies: Vec<String>, status: &str| TeamTask {
        id: id.into(),
        title: id.into(),
        role: "worker".into(),
        input: Value::Null,
        output_contract: json!({}),
        output: None,
        depends_on: dependencies,
        status: status.into(),
        revision: 1,
        assignee_member_id: Some("worker".into()),
        ownership_epoch: 1,
        assignment_authority: None,
        reviewer_member_id: None,
        reviewer_epoch: 0,
        state_reason: None,
        created_at: now,
        updated_at: now,
    };
    let identity = ImmutableDefinitionRef {
        id: "dot04-fixture".into(),
        version: "1.0.0".into(),
        digest: format!("sha256:{}", "0".repeat(64)),
    };
    let team = TeamInstance {
        id: team_id.clone(),
        owner_id: peer::OWNER.into(),
        workspace_id: "dot04".into(),
        revision: 1,
        status: "active".into(),
        definition: identity.clone(),
        package: identity,
        binding: TeamBindingRef {
            id: "fixture-binding".into(),
            revision: 1,
        },
        input: Value::Null,
        members: Vec::new(),
        tasks: vec![
            task("effect", Vec::new(), "running"),
            task("dependent", vec!["effect".into()], "queued"),
        ],
        created_at: now,
        updated_at: now,
    };
    let attempt = TeamExecutionAttempt {
        id: format!("attempt-{team_id}"),
        run_id: run.into(),
        root_id: run.into(),
        approval_scope_id: run.into(),
        queue_sequence: 1,
        continuation_of_wait_id: None,
        owner_id: peer::OWNER.into(),
        workspace_id: "dot04".into(),
        team_id: team_id.clone(),
        task_id: "effect".into(),
        member_id: "worker".into(),
        member_revision: 1,
        ownership_epoch: 1,
        binding_revision: 1,
        execution_epoch: 1,
        execution_fence: Some(service.execution_identity().unwrap()),
        effective_models: Vec::new(),
        effect_disposition: "uncertain".into(),
        accounting_state: "reserved-unknown".into(),
        status: "running".into(),
        execution_outcome: None,
        reservation: TeamReservation {
            tokens: 1000,
            cost_microunits: 1000,
            elapsed_seconds: 60,
        },
        context_artifact_ids: Vec::new(),
        usage: None,
        usage_revision: 0,
        output: None,
        state_reason: None,
        diagnostic: None,
        created_at: now,
        updated_at: now,
    };
    // Controlled catalog input, then actual catalog CAS and settlement. No
    // substitute registry/host or simulated external receipt is used.
    let current = storage.load_state().await.unwrap();
    let mut next = current.clone();
    next.generation += 1;
    next.team_instances
        .insert(format!("{}\u{1f}dot04\u{1f}{team_id}", peer::OWNER), team);
    next.team_execution_attempts.insert(
        format!(
            "{}\u{1f}dot04\u{1f}{team_id}\u{1f}{}",
            peer::OWNER,
            attempt.id
        ),
        attempt.clone(),
    );
    assert!(
        storage
            .compare_and_swap(current.generation, &next)
            .await
            .unwrap()
    );
    let receipts = store
        .list_canonical_tool_receipts(peer::OWNER, run)
        .await
        .unwrap();
    let status = match receipts[0].outcome {
        Some(ToolOutcome::Succeeded) => "succeeded",
        Some(ToolOutcome::ApplicationFailed | ToolOutcome::TransportFailed) => "failed",
        Some(ToolOutcome::OutcomeUnknown) | None => "uncertain",
    };
    let settled = service
        .settle_team_attempt(
            &attempt,
            status,
            None,
            None,
            Some("actual-mcp-receipt".into()),
        )
        .await
        .unwrap();
    let team = service
        .get_team_instance(peer::OWNER, "dot04", &team_id)
        .await
        .unwrap();
    assert_eq!(
        team.tasks
            .iter()
            .find(|task| task.id == "dependent")
            .unwrap()
            .status,
        if mode == "success" { "ready" } else { "queued" }
    );
    assert!(
        service.claim_team_dispatch(&settled).await.is_err(),
        "settled or unknown external work was replayable"
    );
    if mode == "unknown" {
        assert_eq!(settled.effect_disposition, "uncertain");
    }
}

async fn observe_catalog(storage: &Arc<dyn CollaborationStorage>) {
    let state = storage.load_state().await.unwrap();
    assert_eq!(state.team_instances.len(), 8);
    for team in state.team_instances.values() {
        let success = team.id.ends_with("-success");
        assert_eq!(
            team.tasks
                .iter()
                .find(|task| task.id == "dependent")
                .unwrap()
                .status,
            if success { "ready" } else { "queued" }
        );
        let attempt = state
            .team_execution_attempts
            .values()
            .find(|attempt| attempt.team_id == team.id)
            .unwrap();
        if team.id.ends_with("-unknown") {
            assert_eq!(attempt.status, "uncertain");
            assert_eq!(attempt.effect_disposition, "uncertain");
        }
    }
}

#[test]
fn child_outcome_boundary() {
    let Ok(stage) = std::env::var(CHILD) else {
        return;
    };
    // The observed child overflow happened on the libtest caller. Match the
    // shipped runtime's bounded 8 MiB stack for both root polling and workers.
    const STACK_BYTES: usize = 8 * 1024 * 1024;
    std::thread::Builder::new()
        .name("dot04-outcome-child".into())
        .stack_size(STACK_BYTES)
        .spawn(move || {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .thread_stack_size(STACK_BYTES)
                .enable_all()
                .build()
                .expect("bounded outcome fixture runtime builds")
                .block_on(async {
                    let root = PathBuf::from(std::env::var(ROOT).unwrap());
                    tokio::select! {
                        _ = run_child_outcome_boundary(stage) => {}
                        _ = async {
                            while !root.join("cancel").exists() {
                                tokio::time::sleep(Duration::from_millis(20)).await;
                            }
                        } => panic!("parent cancelled owned outcome helper"),
                    }
                });
        })
        .expect("bounded outcome fixture thread starts")
        .join()
        .expect("outcome fixture thread completes");
}

async fn run_child_outcome_boundary(stage: String) {
    let root = PathBuf::from(std::env::var(ROOT).unwrap());
    let shared = std::env::var("UAR_DOT04_SHARED").as_deref() == Ok("1");
    let stores = store(&root, shared).await;
    let store = stores.runtime.clone();
    if stage == "observe" {
        observe(&root, &store).await;
        observe_catalog(&stores.catalog).await;
        return;
    }
    let catalog = CollaborationCatalogService::new(stores.catalog.clone());
    catalog.acquire_execution_owner().await.unwrap();
    legacy_receipt_is_not_migrated_to_success(&store).await;
    for mode in ["eligible", "denied", "missing"] {
        activation_boundary(&root, mode).await;
    }
    let mut receipts = Vec::new();
    for mode in ["success", "application", "predispatch", "unknown"] {
        registry_boundary(&root, false, mode).await;
        registry_boundary(&root, true, mode).await;
        let direct = direct_run(&root, &store, mode).await;
        settle_dependency(&catalog, &stores.catalog, &store, &direct, mode, "direct").await;
        receipts.push(json!({"mode":mode,"run":direct}));
        let (scoped, scoped_store) = process::scoped_outcome_run(&root, shared, mode).await;
        assert_receipt(&scoped_store, &scoped, mode).await;
        settle_dependency(&catalog, &stores.catalog, &scoped_store, &scoped, mode, "scoped").await;
        receipts.push(json!({"mode":mode,"run":scoped,"scopedDatabase":true}));
    }
    std::fs::write(
        root.join("receipts.json"),
        serde_json::to_vec(&receipts).unwrap(),
    )
    .unwrap();
    let process_receipts = process::production_process_cases(&root, shared).await;
    std::fs::write(
        root.join("process-receipts.json"),
        serde_json::to_vec(&process_receipts).unwrap(),
    )
    .unwrap();
    if shared {
        std::fs::write(root.join("ready"), b"ready").unwrap();
        tokio::time::timeout(WAIT, async {
            while !root.join("release").exists() {
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap();
    }
}

fn child(root: &Path, stage: &str, shared: bool, namespace: &str) -> std::process::Child {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", "child_outcome_boundary", "--nocapture"])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env(CHILD, stage)
        .env(ROOT, root)
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if shared {
        command
            .env("UAR_DOT04_SHARED", "1")
            .env("UAR_REMOTE_SURREAL_DURABILITY_ATTESTED", "1")
            .env("UAR_DOT04_NAMESPACE", namespace)
            .env(
                "UAR_DOT04_INFRA_FILE",
                std::env::var("UAR_DOT04_INFRA_FILE")
                    .expect("shared acceptance infrastructure is mandatory"),
            );
    }
    command.spawn().unwrap()
}

fn finish(child: std::process::Child, root: &Path) {
    let pid = child.id();
    let output = child.wait_with_output().unwrap();
    let stdout = root.join(format!("helper-{pid}.stdout.log"));
    let stderr = root.join(format!("helper-{pid}.stderr.log"));
    std::fs::write(&stdout, output.stdout).unwrap();
    std::fs::write(&stderr, output.stderr).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&stdout, std::fs::Permissions::from_mode(0o600)).unwrap();
        std::fs::set_permissions(&stderr, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    assert!(
        output.status.success(),
        "isolated outcome helper exited {}; private logs at {}",
        output.status,
        root.display()
    );
}

fn profile_root(shared: bool) -> (PathBuf, Option<tempfile::TempDir>) {
    match std::env::var_os(EVIDENCE_ROOT) {
        Some(base) => {
            let base = PathBuf::from(base);
            std::fs::create_dir_all(&base).unwrap();
            let root = base.join(format!(
                "{}-{}",
                if shared { "shared" } else { "local" },
                uuid::Uuid::new_v4().simple()
            ));
            std::fs::create_dir(&root).unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
            }
            eprintln!(
                "Private dot04 profile evidence retained at {}",
                root.display()
            );
            (root, None)
        }
        None => {
            let directory = tempfile::tempdir().unwrap();
            (directory.path().to_path_buf(), Some(directory))
        }
    }
}

#[test]
fn actual_mcp_outcomes_are_truthful_locally_and_between_shared_runtimes() {
    for shared in [false, true] {
        let (root, _temporary) = profile_root(shared);
        let namespace = format!("dot04_{}", uuid::Uuid::new_v4().simple());
        let mut writer = child(&root, "exercise", shared, &namespace);
        if shared {
            // The shared sequence boots 26 real binaries, including concurrent
            // and cold observers. Retained startups took 19–54 seconds each;
            // the old 600-second cap expired during a progressing application
            // case. Budget the whole sequence separately from each bounded
            // readiness, consumer and shutdown operation.
            let deadline = Instant::now() + Duration::from_secs(1800);
            while !root.join("ready").exists() {
                if writer.try_wait().unwrap().is_some() {
                    finish(writer, &root);
                    panic!("writer exited before shared observation");
                }
                if Instant::now() >= deadline {
                    std::fs::write(root.join("cancel"), b"cancel").unwrap();
                    eprintln!(
                        "shared exercise deadline; cancelling owned helper; evidence at {}",
                        root.display()
                    );
                    finish(writer, &root);
                    panic!("shared exercise deadline");
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            // The second independent process observes while the first remains alive.
            finish(child(&root, "observe", true, &namespace), &root);
            std::fs::write(root.join("release"), b"release").unwrap();
            finish(writer, &root);
            finish(child(&root, "observe", true, &namespace), &root);
        } else {
            finish(writer, &root);
            finish(child(&root, "observe", false, &namespace), &root);
        }
    }
}
