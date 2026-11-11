//! Authored final-phase durable admission fixture. Never runs during source work.
#![cfg(feature = "surreal-backend")]
use std::sync::Arc;
use universal_agent_runtime::uar::persistence::{
    PersistenceLayer,
    a2ui_actions::{A2uiActionReceipt, digest},
    providers::surreal::SurrealDbProvider,
};

#[tokio::test]
async fn concurrent_logical_claims_retain_original_identity_without_effect_replay() {
    let directory = tempfile::tempdir().expect("owned receipt directory");
    let endpoint = format!(
        "surrealkv://{}",
        directory.path().join("receipts.db").display()
    );
    let store = Arc::new(
        SurrealDbProvider::new(&endpoint, None, None, None, None)
            .await
            .expect("durable receipt store"),
    );
    let receipt = A2uiActionReceipt {
        owner: "verified-owner-key".into(),
        agent_id: "agent".into(),
        thread_id: "thread".into(),
        interaction_id: "intent".into(),
        source_run_id: "source".into(),
        surface_id: "surface".into(),
        surface_generation: "generation".into(),
        component_id: "button".into(),
        action_name: "confirm".into(),
        semantic_digest: digest(&serde_json::json!({"note":"private-owned-note"})),
        continuation_run_id: "host-reserved-run".into(),
        wire_binding: None,
    };
    assert!(
        store
            .load_a2ui_action(
                &receipt.owner,
                &receipt.agent_id,
                &receipt.thread_id,
                &receipt.interaction_id,
            )
            .await
            .expect("look up receipt on a fresh store")
            .is_none()
    );
    let owned_effects = std::sync::atomic::AtomicUsize::new(0);
    let (left, right) = tokio::join!(
        store.create_a2ui_action(&receipt),
        store.create_a2ui_action(&receipt)
    );
    // Owned local effect fixture. Only an acknowledged new durable claim can
    // execute it; errors and existing claims retain uncertainty without effect.
    for result in [&left, &right] {
        if result.as_ref().is_ok_and(|created| *created) {
            owned_effects.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }
    assert_eq!(owned_effects.load(std::sync::atomic::Ordering::SeqCst), 1);
    // A contention error is an uncertain write outcome, never a second grant.
    assert_eq!(
        [
            left.as_ref().is_ok_and(|created| *created),
            right.as_ref().is_ok_and(|created| *created)
        ]
        .into_iter()
        .filter(|created| *created)
        .count(),
        1
    );
    let retained = store
        .load_a2ui_action(
            &receipt.owner,
            &receipt.agent_id,
            &receipt.thread_id,
            &receipt.interaction_id,
        )
        .await
        .expect("reconcile receipt")
        .expect("retained claim");
    assert_eq!(retained, receipt);
    assert!(
        !store
            .create_a2ui_action(&receipt)
            .await
            .expect("repeat claim")
    );
    let mut changed = receipt.clone();
    changed.semantic_digest = digest(&serde_json::json!({"note":"changed"}));
    changed.continuation_run_id = "must-not-replace-original".into();
    assert!(
        !store
            .create_a2ui_action(&changed)
            .await
            .expect("conflicting claim")
    );
    assert_eq!(
        store
            .load_a2ui_action(
                &receipt.owner,
                &receipt.agent_id,
                &receipt.thread_id,
                &receipt.interaction_id
            )
            .await
            .expect("unchanged read"),
        Some(receipt.clone())
    );
    assert!(
        store
            .load_a2ui_action(
                "foreign",
                &receipt.agent_id,
                &receipt.thread_id,
                &receipt.interaction_id
            )
            .await
            .expect("foreign read")
            .is_none()
    );
    assert!(
        store
            .load_a2ui_action(
                &receipt.owner,
                &receipt.agent_id,
                "foreign-thread",
                &receipt.interaction_id
            )
            .await
            .expect("foreign thread")
            .is_none()
    );
    let mut second = receipt.clone();
    second.interaction_id = "distinct-intent".into();
    second.continuation_run_id = "distinct-host-run".into();
    assert!(
        store
            .create_a2ui_action(&second)
            .await
            .expect("intentional second click")
    );
    let closed = store.close_scope().await;
    assert!(
        closed.scope_released && closed.engine_close_acknowledged,
        "owned store close: {closed:?}"
    );
    drop(store);
    let reopened = SurrealDbProvider::new(&endpoint, None, None, None, None)
        .await
        .expect("reopen receipt store");
    assert_eq!(
        reopened
            .load_a2ui_action(
                &receipt.owner,
                &receipt.agent_id,
                &receipt.thread_id,
                &receipt.interaction_id
            )
            .await
            .expect("restart reconciliation"),
        Some(receipt.clone())
    );
    let replay_grant = reopened
        .create_a2ui_action(&receipt)
        .await
        .expect("restart retry never grants dispatch");
    if replay_grant {
        owned_effects.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
    assert!(!replay_grant);
    assert_eq!(owned_effects.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert!(reopened.close_scope().await.engine_close_acknowledged);
    // This owned effect fixture qualifies durable claim exclusivity, not the
    // mounted HTTP/manager path. The final API/browser receipt remains separate.
    // No runtime dispatcher is called by the durable claim/read contract. A reserved
    // identity with no run is deliberately not a fabricated successful effect.
}

#[cfg(feature = "server")]
#[expect(dead_code, reason = "shared external host adapter includes profiles used by other functional matrices")]
#[path = "support/external_identity_host.rs"]
mod external_identity_host;

#[cfg(feature = "server")]
mod recovery_routes {
    use super::*;
    use axum::{Extension, Router, body::{Body, to_bytes}, http::{Request, StatusCode}};
    use serde_json::{Value, json};
    use sha2::{Digest, Sha256};
    use std::{pin::Pin, sync::atomic::{AtomicUsize, Ordering}, time::Duration};
    use tokio::sync::{Notify, RwLock};
    use tower::ServiceExt;
    use universal_agent_runtime::{
        config::{LlmConfig, SecurityConfig, ServiceInstanceConfig},
        llm::{LlmDriver, LlmRequest},
        mcp::registry::McpRegistry,
        session::SessionStore,
        uar::{
            a2ui::{routes::{A2uiApiState, build_response_router}, registry::A2uiRegistry,
                realtime::InMemoryReplayBackbone, design_systems::store::InMemoryDesignSystemStore},
            api::{agui, routes::RunApiState},
            compiler::collaboration::CollaborationCatalogService,
            domain::runs::{Run, RunStatus},
            persistence::{agui_runs::{AguiRunBinding, AguiTerminal}, run_history::{RunHistoryRecord, RunHistoryEvent}},
            rag::embeddings::{UnavailableEmbeddingBackend, swappable::SwappableEmbeddingBackend},
            runtime::{manager::RunManager, matching::VectorMatcher, skills::{SkillRegistry, service::{SkillService, SkillMatchingConfig, SkillMatchingAlgorithm, SkillMatchingEngine}}, turn::RunExecutionRequest},
            security::claims::UserContext,
            service_instance::ServiceInstanceAuthority,
        },
    };

    #[derive(Default)]
    struct HeldProducer {
        calls: AtomicUsize,
        entered: Notify,
    }

    #[async_trait::async_trait]
    impl LlmDriver for HeldProducer {
        async fn stream(&self, _request: LlmRequest) -> anyhow::Result<Pin<Box<dyn futures::Stream<Item = anyhow::Result<universal_agent_runtime::normalized::NormalizedEvent>> + Send>>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.entered.notify_one();
            Ok(Box::pin(futures::stream::pending()))
        }
    }

    async fn manager(store: Arc<SurrealDbProvider>, driver: Arc<dyn LlmDriver>) -> Arc<RunManager> {
        manager_with_sources(store, driver, None).await
    }

    async fn manager_with_sources(store: Arc<SurrealDbProvider>, driver: Arc<dyn LlmDriver>,
        sources: Option<(Arc<universal_agent_runtime::llm::ProviderRegistry>, Arc<universal_agent_runtime::uar::settings::manager::SettingsManager>)>) -> Arc<RunManager> {
        let skills = Arc::new(SkillService::new(Some(store.clone()), None));
        skills.set_matching_config(SkillMatchingConfig {
            engine: SkillMatchingEngine::Service, algorithm: SkillMatchingAlgorithm::Keyword,
            threshold: 0.5, margin_threshold: 0.0, top_k: 1, model_name: None,
        }).await;
        let manager = RunManager::new(
            LlmConfig { model: "openai/gpt-4o".into(), ..LlmConfig::default() },
            Arc::new(McpRegistry::new_empty()),
            SessionStore::new(),
            Arc::new(RwLock::new(SkillRegistry::default())),
            Arc::new(VectorMatcher::new(Arc::new(SwappableEmbeddingBackend::new(Arc::new(
                UnavailableEmbeddingBackend::new(384, "receipt recovery does not embed"),
            ))), 0.75)),
            Some(store),
        ).await.with_skill_service(skills).with_llm_driver(driver);
        Arc::new(if let Some((registry, settings)) = sources {
            manager.with_provider_registry(registry).with_settings_manager(settings)
        } else { manager })
    }

    fn rest(manager: Arc<RunManager>, user: &UserContext) -> Router {
        build_response_router().with_state(A2uiApiState {
            registry: Arc::new(A2uiRegistry::new()),
            run_manager: manager,
            realtime_backbone: InMemoryReplayBackbone::new(),
            design_system_store: Arc::new(InMemoryDesignSystemStore::new()),
            persistence: None,
        }).layer(Extension(user.clone()))
    }

    fn agui_admission_policy() -> SecurityConfig {
        // The externally verified context supplies identity to this inner router.
        serde_json::from_value(json!({
            "deployment_profile": "trusted_local",
            "jwt_required": false,
            "jwt_secret": "",
            "local_loopback_owner": false
        }))
        .expect("trusted-local AG-UI fixture policy")
    }

    fn agui_router(manager: Arc<RunManager>, user: &UserContext) -> Router {
        agui::build_router(agui_admission_policy()).with_state(Arc::new(RunApiState {
            manager,
            collaboration_catalog: Arc::new(CollaborationCatalogService::in_memory()),
            service_instance: Arc::new(ServiceInstanceAuthority::new(
                &ServiceInstanceConfig::default(), "receipt-recovery", "http://127.0.0.1", false, &[],
            )),
        })).layer(Extension(user.clone()))
    }

    async fn response(app: &Router, method: &str, path: &str, body: Option<Value>) -> (StatusCode, Value) {
        let request = Request::builder().method(method).uri(path).header("content-type", "application/json")
            .body(body.map_or_else(Body::empty, |value| Body::from(value.to_string()))).unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), 64 * 1024).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
    }

    async fn persist_run(store: &SurrealDbProvider, user: &UserContext, id: &str, agent: &str, thread: &str, status: RunStatus) {
        // Same recorded namespace encoding as ActorOwner::presentation_owner_key;
        // the principal itself comes only from the existing external host adapter.
        let owner_key = match &user.tenant_id {
            Some(tenant) => format!("v1:t:{}:{}:s:{}:{}", tenant.as_str().len(), tenant.as_str(), user.user_id.len(), user.user_id),
            None => format!("v1:s:{}:{}", user.user_id.len(), user.user_id),
        };
        store.upsert_run_history_record(&RunHistoryRecord {
            owner_key: Some(owner_key), run_id: id.into(), revision: 1,
            run: serde_json::to_value(Run { run_id: id.into(), agent_id: agent.into(), conversation_id: Some(thread.into()),
                user_id: Some(user.user_id.clone()), status, context: json!({}) }).unwrap(),
        }).await.unwrap();
    }

    fn intent() -> Value {
        json!({"interaction_id":"original-intent", "surface_generation":"generation",
            "surfaceId":"surface", "name":"confirm", "sourceComponentId":"button", "context":{"value":1}})
    }

    fn receipt(user: &UserContext) -> A2uiActionReceipt {
        A2uiActionReceipt {
            owner: user.session_owner_key().unwrap(), agent_id: "agent".into(), thread_id: "conversation".into(),
            interaction_id: "original-intent".into(), source_run_id: "source".into(), surface_id: "surface".into(),
            surface_generation: "generation".into(), component_id: "button".into(), action_name: "confirm".into(),
            semantic_digest: digest(&json!({"surface":"surface", "generation":"generation", "component":"button",
                "name":"confirm", "context":{"value":1}, "data_model":null, "catalog":null, "user_message":null})),
            continuation_run_id: "original-continuation".into(), wire_binding: None,
        }
    }

    async fn close(manager: Arc<RunManager>) {
        manager.shutdown_graph_roots().await.unwrap();
        manager.shutdown_terminals().await.unwrap();
        manager.shutdown_sandboxes().await.unwrap();
        manager.shutdown_run_history().await.unwrap();
    }

    async fn public_run_errors(manager: &RunManager, run_id: &str) -> Vec<(String, String)> {
        manager.history_since(run_id, None).await.unwrap_or_default().into_iter()
            .filter_map(|entry| match entry.event {
                universal_agent_runtime::uar::domain::events::NormalizedEvent::Error { code, message, .. } => Some((code, message)),
                _ => None,
            }).collect()
    }

    /// Requires externally provisioned a2ui-recovery and a2ui-recovery-foreign
    /// identity-capture profiles. No fixture reads, constructs or sends credentials.
    #[tokio::test]
    async fn reopened_rest_receipt_and_identical_post_reconcile_without_resident_source_or_dispatch() {
        let user = external_identity_host::user("a2ui-recovery", false, "a2ui-owner").await;
        let foreign = external_identity_host::user("a2ui-recovery-foreign", false, "a2ui-foreign").await;
        let root = tempfile::tempdir().unwrap();
        let endpoint = format!("surrealkv://{}", root.path().join("recovery.db").display());
        let original = receipt(&user);
        let store = SurrealDbProvider::new(&endpoint, None, None, None, None).await.unwrap();
        persist_run(&store, &user, "source", "agent", "conversation", RunStatus::Done).await;
        persist_run(&store, &user, "original-continuation", "agent", "conversation", RunStatus::Done).await;
        for (source, agent, thread) in [("other-source", "agent", "conversation"),
            ("other-agent", "different-agent", "conversation"), ("other-conversation", "agent", "different-conversation")] {
            persist_run(&store, &user, source, agent, thread, RunStatus::Done).await;
        }
        assert!(store.create_a2ui_action(&original).await.unwrap());
        let closed = store.close_scope().await;
        assert!(closed.scope_released && closed.engine_close_acknowledged, "owned store close: {closed:?}");
        drop(store);
        let store = Arc::new(SurrealDbProvider::new(&endpoint, None, None, None, None).await.unwrap());
        let driver = Arc::new(HeldProducer::default());
        let manager = manager(store.clone(), driver.clone()).await;
        assert!(manager.get_run("source").await.is_none());
        assert!(manager.get_run("original-continuation").await.is_none());
        let app = rest(manager.clone(), &user);
        let path = "/runs/source/a2ui/actions/original-intent";
        let (status, ack) = response(&app, "GET", path, None).await;
        assert_eq!(status, StatusCode::ACCEPTED);
        assert_eq!(ack["outcome"], "completed");
        assert_eq!(ack["continuation_run_id"], "original-continuation");
        let (status, retry) = response(&app, "POST", "/runs/source/a2ui/actions", Some(intent())).await;
        assert_eq!(status, StatusCode::ACCEPTED);
        assert_eq!(retry, ack);
        let mut changed = intent();
        changed["context"]["value"] = json!(2);
        assert_eq!(response(&app, "POST", "/runs/source/a2ui/actions", Some(changed)).await.0, StatusCode::CONFLICT);
        assert_eq!(response(&rest(manager.clone(), &foreign), "GET", path, None).await.0, StatusCode::NOT_FOUND);
        assert_eq!(response(&rest(manager.clone(), &foreign), "POST", "/runs/source/a2ui/actions", Some(intent())).await.0, StatusCode::NOT_FOUND);
        for source in ["other-source", "other-agent", "other-conversation"] {
            assert_eq!(response(&app, "GET", &format!("/runs/{source}/a2ui/actions/original-intent"), None).await.0, StatusCode::NOT_FOUND);
            let expected = if source == "other-source" { StatusCode::CONFLICT } else { StatusCode::NOT_FOUND };
            assert_eq!(response(&app, "POST", &format!("/runs/{source}/a2ui/actions"), Some(intent())).await.0, expected);
        }
        let mut fresh = intent();
        fresh["interaction_id"] = json!("new-intent");
        assert_eq!(response(&app, "POST", "/runs/source/a2ui/actions", Some(fresh)).await.0, StatusCode::NOT_FOUND);
        assert!(store.load_a2ui_action(&original.owner, "agent", "conversation", "new-intent").await.unwrap().is_none());
        assert_eq!(store.load_a2ui_action(&original.owner, "agent", "conversation", "original-intent").await.unwrap(), Some(original));
        assert_eq!(driver.calls.load(Ordering::SeqCst), 0, "reconciliation never dispatches a producer");
        assert!(manager.get_run("original-continuation").await.is_none());
        drop(app);
        close(manager).await;
        assert!(store.close_scope().await.engine_close_acknowledged);
    }

    #[tokio::test]
    async fn rest_and_agui_distinguish_resident_execution_from_durable_nonterminal_history() {
        let user = external_identity_host::user("a2ui-recovery", false, "a2ui-owner").await;
        let root = tempfile::tempdir().unwrap();
        let store = Arc::new(SurrealDbProvider::new(&format!("surrealkv://{}", root.path().join("outcomes.db").display()), None, None, None, None).await.unwrap());
        persist_run(&store, &user, "source", "agent", "conversation", RunStatus::Done).await;
        let driver = Arc::new(HeldProducer::default());
        let manager = manager(store.clone(), driver.clone()).await;
        let rest = rest(manager.clone(), &user);
        let agui = agui_router(manager.clone(), &user);
        for (index, (status, expected)) in [(RunStatus::Pending, "uncertain"), (RunStatus::Running, "uncertain"),
            (RunStatus::Paused, "uncertain"), (RunStatus::Done, "completed"), (RunStatus::Error, "failed"),
            (RunStatus::Cancelled, "cancelled")].into_iter().enumerate() {
            let mut rest_receipt = receipt(&user);
            rest_receipt.interaction_id = format!("intent-{index}");
            rest_receipt.continuation_run_id = format!("continuation-{index}");
            persist_run(&store, &user, &rest_receipt.continuation_run_id, "agent", "conversation", status.clone()).await;
            assert!(store.create_a2ui_action(&rest_receipt).await.unwrap());
            let (code, ack) = response(&rest, "GET", &format!("/runs/source/a2ui/actions/intent-{index}"), None).await;
            assert_eq!(code, StatusCode::ACCEPTED);
            assert_eq!(ack["outcome"], expected);
            let mut wire_receipt = rest_receipt.clone();
            wire_receipt.thread_id = format!("agui-{}", AguiRunBinding::key(&wire_receipt.owner, "agent", "wire-thread", ""));
            wire_receipt.continuation_run_id = format!("wire-continuation-{index}");
            wire_receipt.source_run_id = "source-wire-thread".into();
            persist_run(&store, &user, &wire_receipt.continuation_run_id, "agent", &wire_receipt.thread_id, status).await;
            wire_receipt.wire_binding = Some(AguiRunBinding {
                native_agent_id: None,
                owner: wire_receipt.owner.clone(), agent_id: "agent".into(), thread_id: "wire-thread".into(),
                wire_run_id: format!("original-wire-{index}"), native_run_id: wire_receipt.continuation_run_id.clone(),
                session_id: wire_receipt.thread_id.clone(), client_message_id: None,
                action_interaction_id: Some(wire_receipt.interaction_id.clone()), workspace_id: None,
                input_digest: wire_receipt.semantic_digest.clone(), ordinal: index as u64 + 2, revision: 1, start_cursor: 0,
                previous_wire_run_id: Some(if index == 0 { "seed-source".into() } else { format!("original-wire-{}", index - 1) }), resumed_tool_ids: vec![], terminal: None,
            });
            if index == 0 {
                seed_source(&store, &user, "agent", "agent", "wire-thread", None).await;
            }
            let wire_row = wire_receipt.wire_binding.as_ref().unwrap();
            let predecessor = store.list_agui_runs(&wire_receipt.owner, "agent", "wire-thread").await.unwrap()
                .into_iter().max_by_key(|row| row.ordinal).unwrap();
            assert!(store.create_agui_run(wire_row, Some(&predecessor)).await.unwrap());
            let mut sealed = wire_row.clone();
            sealed.revision += 1;
            sealed.terminal = Some(AguiTerminal { source_cursor: 0, event: json!({"type":"RUN_FINISHED"}), interrupt_ids: vec![] });
            assert!(store.compare_and_swap_agui_run(wire_row, &sealed).await.unwrap());
            assert!(store.create_a2ui_action(&wire_receipt).await.unwrap());
            let (code, ack) = response(&agui, "GET", &format!("/agents/agent/threads/wire-thread/actions/intent-{index}"), None).await;
            assert_eq!(code, StatusCode::OK);
            assert_eq!(ack["outcome"], expected);
            assert_eq!(ack["wire_run_id"], format!("original-wire-{index}"));
        }
        assert_eq!(driver.calls.load(Ordering::SeqCst), 0);
        for (suffix, agent, thread) in [("missing", "agent", "conversation"),
            ("wrong-agent", "other-agent", "conversation"), ("wrong-conversation", "agent", "other-conversation")] {
            let mut uncertain = receipt(&user);
            uncertain.interaction_id = suffix.into();
            uncertain.continuation_run_id = format!("uncertain-{suffix}");
            if suffix != "missing" {
                persist_run(&store, &user, &uncertain.continuation_run_id, agent, thread, RunStatus::Done).await;
            }
            assert!(store.create_a2ui_action(&uncertain).await.unwrap());
            let (code, ack) = response(&rest, "GET", &format!("/runs/source/a2ui/actions/{suffix}"), None).await;
            assert_eq!(code, StatusCode::ACCEPTED);
            assert_eq!(ack["outcome"], "uncertain", "unrelated terminal history is not this action's outcome");
        }
        for protocol in ["rest", "agui"] {
            let mut live_receipt = receipt(&user);
            live_receipt.interaction_id = format!("live-{protocol}");
            if protocol == "agui" {
                live_receipt.thread_id = format!("agui-{}", AguiRunBinding::key(&live_receipt.owner, "agent", "wire-thread", ""));
                live_receipt.source_run_id = "source-wire-thread".into();
            }
            let mut artifact = universal_agent_runtime::uar::defaults::default_agent();
            artifact.id = "agent".into();
            artifact.policy.skills.max_active = 0;
            artifact.policy.tools.allow.clear();
            artifact.extensions.insert("uar.run_policy".into(), json!({
                "version": 1, "skills": { "mode": "none" }, "tools": { "mode": "none" }
            }));
            artifact.memory.kb.enabled = false;
            let mut request = RunExecutionRequest::new(artifact, "hold producer".into()).with_user_context(&user).unwrap();
            request.session_id = Some(live_receipt.thread_id.clone());
            let live_id = manager.execute_request(request).await;
            if tokio::time::timeout(Duration::from_secs(30), driver.entered.notified()).await.is_err() {
                let diagnostics = tokio::time::timeout(Duration::from_secs(5), async {
                    let status = manager.get_run(&live_id).await.map(|run| run.status);
                    (status, public_run_errors(&manager, &live_id).await)
                }).await;
                panic!("{protocol} producer did not enter for {live_id}: diagnostics={diagnostics:?}");
            }
            live_receipt.continuation_run_id = live_id.clone();
            if protocol == "agui" {
                live_receipt.wire_binding = Some(AguiRunBinding {
                    native_agent_id: None,
                    owner: live_receipt.owner.clone(), agent_id: "agent".into(), thread_id: "wire-thread".into(),
                    wire_run_id: "original-live-wire".into(), native_run_id: live_id.clone(),
                    session_id: live_receipt.thread_id.clone(), client_message_id: None,
                    action_interaction_id: Some(live_receipt.interaction_id.clone()), workspace_id: None,
                    input_digest: live_receipt.semantic_digest.clone(), ordinal: 1, revision: 1, start_cursor: 0,
                    previous_wire_run_id: None, resumed_tool_ids: vec![], terminal: None,
                });
            }
            if protocol == "agui" {
                let predecessor = store.list_agui_runs(&live_receipt.owner, "agent", "wire-thread").await.unwrap()
                    .into_iter().max_by_key(|row| row.ordinal).unwrap();
                let row = live_receipt.wire_binding.as_mut().unwrap();
                row.ordinal = predecessor.ordinal + 1;
                row.previous_wire_run_id = Some(predecessor.wire_run_id.clone());
                assert!(store.create_agui_run(row, Some(&predecessor)).await.unwrap());
            }
            assert!(store.create_a2ui_action(&live_receipt).await.unwrap());
            let before = driver.calls.load(Ordering::SeqCst);
            let (code, ack) = if protocol == "rest" {
                response(&rest, "GET", "/runs/source/a2ui/actions/live-rest", None).await
            } else {
                response(&agui, "GET", "/agents/agent/threads/wire-thread/actions/live-agui", None).await
            };
            assert_eq!(code, if protocol == "rest" { StatusCode::ACCEPTED } else { StatusCode::OK });
            assert_eq!(ack["outcome"], "running");
            assert_eq!(ack["continuation_run_id"], live_id);
            assert_eq!(driver.calls.load(Ordering::SeqCst), before, "lookup does not redispatch resident work");
            assert!(manager.cancel_run_for_context(&user, &live_id).await);
        }
        assert_eq!(driver.calls.load(Ordering::SeqCst), 2, "only the two explicit host submissions dispatch");
        drop(rest);
        drop(agui);
        close(manager).await;
        assert!(store.close_scope().await.engine_close_acknowledged);
    }
    // Synthetic durable source authority. Actual action admission below must
    // independently authenticate this owner/native/session identity. This does
    // not claim a renderer publication or process crash was exercised.
    async fn seed_source(store: &SurrealDbProvider, user: &UserContext, selector: &str,
        native: &str, thread: &str, workspace: Option<&str>) -> (AguiRunBinding, String) {
        use universal_agent_runtime::uar::domain::events::{NormalizedEvent, ArtifactPayload};
        let owner = user.session_owner_key().unwrap();
        let session = format!("agui-{}", AguiRunBinding::key(&owner, selector, thread, ""));
        let source_id = format!("source-{thread}");
        persist_run(store, user, &source_id, native, &session, RunStatus::Done).await;
        let generation: String = Sha256::digest(json!(["uar.a2ui.publication/1", source_id, 1, "surface"])
            .to_string().as_bytes()).iter().map(|byte| format!("{byte:02x}")).collect();
        let content = [json!({"createSurface":{"surfaceId":"surface"}}),
            json!({"updateComponents":{"surfaceId":"surface","components":[{
                "id":"button","component":"Button","action":{"event":{"name":"confirm"}}}]}})]
            .iter().map(Value::to_string).collect::<Vec<_>>().join("\n");
        let events = [NormalizedEvent::ArtifactDisplay { run_id: source_id.clone(), artifact: ArtifactPayload {
            artifact_id: "seeded-source-surface".into(), artifact_type: "a2ui".into(), title: "Seeded source authority".into(),
            content, language: None, metadata: json!({"surface_generations":{"surface":generation}}),
        }}, NormalizedEvent::RunDone { run_id: source_id.clone() }];
        for (index, event) in events.into_iter().enumerate() {
            store.append_run_history_event(&RunHistoryEvent { run_id: source_id.clone(), event_id: index as u64 + 1,
                event: serde_json::to_value(event).unwrap() }).await.unwrap();
        }
        let row = AguiRunBinding { owner, agent_id: selector.into(), native_agent_id: Some(native.into()),
            thread_id: thread.into(), wire_run_id: "seed-source".into(), native_run_id: source_id,
            session_id: session, client_message_id: None, action_interaction_id: None,
            workspace_id: workspace.map(str::to_owned), input_digest: digest(&json!({"seed":"source"})),
            ordinal: 1, revision: 1, start_cursor: 0, previous_wire_run_id: None, resumed_tool_ids: vec![], terminal: None };
        assert!(store.create_agui_run(&row, None).await.unwrap());
        let mut sealed = row.clone();
        sealed.revision = 2;
        sealed.terminal = Some(AguiTerminal { source_cursor: 2, event: json!({"type":"RUN_FINISHED","outcome":{"type":"success"}}), interrupt_ids: vec![] });
        assert!(store.compare_and_swap_agui_run(&row, &sealed).await.unwrap());
        (sealed, generation)
    }

    #[derive(Default)]
    struct BoundActionProducer {
        calls: AtomicUsize,
        admitted: AtomicUsize,
        entered: Notify,
        mode: std::sync::Mutex<String>,
        proof: std::sync::Mutex<Option<(Arc<SurrealDbProvider>, String, String, String)>>,
    }
    #[async_trait::async_trait]
    impl LlmDriver for BoundActionProducer {
        async fn stream(&self, request: LlmRequest) -> anyhow::Result<Pin<Box<dyn futures::Stream<Item = anyhow::Result<universal_agent_runtime::normalized::NormalizedEvent>> + Send>>> {
            use universal_agent_runtime::normalized::NormalizedEvent as ProviderEvent;
            self.calls.fetch_add(1, Ordering::SeqCst);
            let proof = self.proof.lock().unwrap().clone().unwrap();
            let (store, owner, selector, native) = proof;
            let mode = self.mode.lock().unwrap().clone();
            let row = store.load_agui_run(&owner, &selector, &format!("bound-{mode}"), "wire-action").await?.unwrap();
            let receipt = store.load_a2ui_action(&owner, &selector, &row.session_id, "bound-intent").await?.unwrap();
            anyhow::ensure!(row.native_agent_id.as_deref() == Some(native.as_str())
                && receipt.agent_id == selector && receipt.continuation_run_id == row.native_run_id
                && receipt.wire_binding.as_ref().unwrap().native_agent_id.is_none(),
                "dispatch must follow original receipt and captured durable native identity");
            anyhow::ensure!(request.messages.iter().any(|message| message["content"].as_str()
                .is_some_and(|content| content.contains("a2ui.user_action") && content.contains("private-owned-note"))),
                "actual action payload must reach the provider");
            self.admitted.fetch_add(1, Ordering::SeqCst);
            self.entered.notify_one();
            let stream: Pin<Box<dyn futures::Stream<Item = anyhow::Result<ProviderEvent>> + Send>> = match mode.as_str() {
                "cancelled" => Box::pin(futures::stream::pending()),
                "failed" => Box::pin(futures::stream::iter(vec![Ok(ProviderEvent::Error {
                    message: "Controlled action failure".into(), code: Some("fixture_action_error".into()),
                })])),
                _ => Box::pin(futures::stream::iter(vec![Ok(ProviderEvent::MessageDelta { text: "Action complete".into() }), Ok(ProviderEvent::Done)])),
            };
            Ok(stream)
        }
    }

    async fn bound_request(app: &Router, method: &str, path: &str, body: Option<Value>) -> axum::response::Response {
        app.clone().oneshot(Request::builder().method(method).uri(path)
            .header("content-type", "application/json").header("x-uar-workspace-id", "workspace:r028")
            .body(body.map_or_else(Body::empty, |body| Body::from(body.to_string()))).unwrap()).await.unwrap()
    }
    async fn bound_json(app: &Router, path: &str) -> (StatusCode, Value) {
        let reply = bound_request(app, "GET", path, None).await;
        let status = reply.status();
        let bytes = to_bytes(reply.into_body(), 128 * 1024).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }
    async fn bound_stream(reply: axum::response::Response) -> Vec<Value> {
        assert_eq!(reply.status(), StatusCode::OK);
        let bytes = tokio::time::timeout(Duration::from_secs(30), to_bytes(reply.into_body(), 128 * 1024)).await.unwrap().unwrap();
        std::str::from_utf8(&bytes).unwrap().lines().filter_map(|line| line.strip_prefix("data: "))
            .map(|line| serde_json::from_str(line).unwrap()).collect()
    }
    fn fixture_digest(bytes: &[u8]) -> String {
        format!("sha256:{}", Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>())
    }

    fn finalize_fixture(document: &mut Value) {
        fn canonical(value: &Value) -> Value {
            match value {
                Value::Object(object) => Value::Object(object.iter()
                    .map(|(key, value)| (key.clone(), canonical(value)))
                    .collect::<std::collections::BTreeMap<_, _>>().into_iter().collect()),
                Value::Array(values) => Value::Array(values.iter().map(canonical).collect()),
                value => value.clone(),
            }
        }
        document.as_object_mut().unwrap().remove("contentDigest");
        document["contentDigest"] = json!(fixture_digest(&serde_json::to_vec(&canonical(document)).unwrap()));
    }


    #[test]
    fn public_deployment_action_keeps_original_receipt_outcomes_after_reopen_without_catalog_or_redispatch() {
        // Match the production host stack for the full mounted admission path.
        std::thread::Builder::new()
            .name("a2ui-bound-deployment-gate".into())
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(bound_deployment_action_outcomes())
            })
            .unwrap()
            .join()
            .unwrap();
    }

    async fn bound_deployment_action_outcomes() {
        use universal_agent_runtime::{llm::{ProviderRegistry, registry::ProviderConfig},
            uar::{api::model_verification, domain::collaboration::{PackageSourceRequest, BindingCommandRequest},
                settings::{manager::SettingsManager, schema::{SettingsType, Settings}}}};
        let user = external_identity_host::user("a2ui-recovery", false, "a2ui-owner").await;
        let foreign = external_identity_host::user("a2ui-recovery-foreign", false, "a2ui-foreign").await;
        let directory = tempfile::tempdir().unwrap();
        let endpoint = format!("surrealkv://{}", directory.path().join("bound-actions.db").display());
        let store = Arc::new(SurrealDbProvider::new(&endpoint, None, None, None, None).await.unwrap());
        let settings = Arc::new(SettingsManager::new(store.clone()));
        let registry = Arc::new(ProviderRegistry::new());
        // Supply a credential-free configured model and its seeded, exact-config
        // operation receipt. This fixture does not claim a live model verification.
        let config: ProviderConfig = serde_json::from_value(json!({
            "id":"openai","display_name":"Bound identity fixture","base_url":"http://127.0.0.1:1",
            "api_key":null,"protocol":"chat","default_model":"gpt-4o","enabled":true,
            "models":[{"id":"gpt-4o","enabled":true,"supports_tools":true,"supports_streaming":true}]
        })).unwrap();
        registry.register(config).await.unwrap();
        let config = registry.get("openai").await.unwrap();
        let now = chrono::Utc::now();
        let kind = SettingsType { id: uuid::Uuid::new_v4(), name: "Model verification receipts".into(),
            key: "internal-model-verification".into(), schema: json!({"type":"object"}),
            created_at: now, updated_at: None };
        settings.register_extension(kind.clone(), vec![Settings {
            id: uuid::Uuid::new_v4(), settings_type_id: kind.id, name: "Fixture model verification".into(),
            key: model_verification::key("openai", "gpt-4o"),
            data: serde_json::to_value(model_verification::ModelVerification { verified_at: now,
                configuration_digest: model_verification::fingerprint(&config, None).await.unwrap() }).unwrap(),
            parent_id: None, created_at: now, updated_at: None,
        }]).await.unwrap();
        let catalog = Arc::new(CollaborationCatalogService::in_memory().with_provider_registry(registry.clone()));
        let mut agent: Value = serde_json::from_str(include_str!("fixtures/collaboration/agent-definition.json")).unwrap();
        agent["id"] = json!("urn:uar:r028:native-artifact");
        agent["skills"] = json!([]);
        agent["input"] = json!({"type":"string"});
        agent["output"] = json!({"type":"string"});
        agent["sourceDescriptor"]["skills"] = json!([]);
        agent["legacySections"]["skills"] = json!("");
        finalize_fixture(&mut agent);
        let source = format!("{}\n", serde_json::to_string_pretty(&agent).unwrap());
        let reference = json!({"id":agent["id"],"version":agent["version"],"digest":agent["contentDigest"]});
        let mut manifest: Value = serde_json::from_str(include_str!("fixtures/collaboration/package-manifest.json")).unwrap();
        manifest["id"] = json!("urn:uar:r028:package");
        manifest["entrypoints"] = json!([reference.clone()]);
        manifest["files"] = json!([{"path":"agent-definition.json","kind":"AgentDefinition",
            "definition":reference,"byteDigest":fixture_digest(source.as_bytes())}]);
        manifest["lock"] = json!([]);
        finalize_fixture(&mut manifest);
        let owner = user.session_owner_key().unwrap();
        // The deployment catalog and durable run receipts use different
        // established storage keys for the same verified principal.
        let catalog_owner = universal_agent_runtime::uar::api::user_settings::principal_storage_key(&user).unwrap();
        let installed = catalog.install_package(&catalog_owner, PackageSourceRequest {
            command_id: "r028-package".into(), expected_catalog_revision: None,
            manifest: serde_json::to_string_pretty(&manifest).unwrap(),
            files: [("agent-definition.json".into(), source)].into_iter().collect(),
        }).await.unwrap();
        let mut binding: Value = serde_json::from_str(include_str!("fixtures/collaboration/deployment-binding.json")).unwrap();
        binding["id"] = json!("r028-binding");
        binding["ownerId"] = json!(catalog_owner);
        binding["workspaceId"] = json!("workspace:r028");
        binding["package"] = serde_json::to_value(installed.preflight.package).unwrap();
        binding["skillBindings"] = json!([]);
        binding["representationGrantRefs"] = json!([]);
        binding["modelBindings"][0]["modelId"] = json!("gpt-4o");
        finalize_fixture(&mut binding);
        let installed = catalog.install_binding(&catalog_owner, "workspace:r028", BindingCommandRequest {
            command_id: "r028-binding".into(), expected_revision: Some(0), binding,
        }).await.unwrap();
        assert!(installed.preflight.activation_supported, "{:?}", installed.preflight);
        let target = universal_agent_runtime::uar::api::targets::agent_target_id("r028-binding");
        let resolved = catalog.resolve_bound_agent_run(&catalog_owner, "workspace:r028", "r028-binding").await.unwrap();
        assert_ne!(target, resolved.artifact.id);

        let driver = Arc::new(BoundActionProducer::default());
        *driver.proof.lock().unwrap() = Some((store.clone(), owner.clone(), target.clone(), resolved.artifact.id.clone()));
        let manager = manager_with_sources(store.clone(), driver.clone(), Some((registry, settings))).await;
        let app = agui::build_router(agui_admission_policy()).with_state(Arc::new(RunApiState {
            manager: manager.clone(), collaboration_catalog: catalog.clone(),
            service_instance: Arc::new(ServiceInstanceAuthority::new(&ServiceInstanceConfig::default(),
                "bound-action", "http://127.0.0.1", false, &[])),
        })).layer(Extension(user.clone()));
        let mut originals = Vec::new();
        for expected in ["completed", "failed", "cancelled"] {
            let thread = format!("bound-{expected}");
            // Source surface/model verification are seeded; continuation
            // admission, driver invocation and native terminal are actual paths.
            let (source, generation) = seed_source(&store, &user, &target, &resolved.artifact.id,
                &thread, Some("workspace:r028")).await;
            let input = json!({"threadId":thread,"runId":"wire-action","parentRunId":"seed-source","messages":[],
                "forwardedProps":{"a2uiAction":{"interactionId":"bound-intent","surfaceGeneration":generation,
                    "userAction":{"surfaceId":"surface","sourceComponentId":"button","name":"confirm",
                        "context":{"note":"private-owned-note"}}}}});
            let path = format!("/agents/{target}");
            let mut stale = input.clone();
            stale["runId"] = json!("stale-action");
            stale["forwardedProps"]["a2uiAction"]["surfaceGeneration"] = json!("old-generation");
            assert_eq!(bound_request(&app, "POST", &path, Some(stale)).await.status(), StatusCode::UNPROCESSABLE_ENTITY);
            *driver.mode.lock().unwrap() = expected.into();
            let entered = driver.entered.notified();
            let reply = bound_request(&app, "POST", &path, Some(input.clone())).await;
            assert_eq!(reply.status(), StatusCode::OK, "{expected} action admission");
            if tokio::time::timeout(Duration::from_secs(30), entered).await.is_err() {
                let diagnostics = tokio::time::timeout(Duration::from_secs(5), async {
                    match store.load_agui_run(&owner, &target, &thread, "wire-action").await {
                        Ok(Some(row)) => Some(public_run_errors(&manager, &row.native_run_id).await),
                        _ => None,
                    }
                }).await;
                let calls = driver.calls.load(Ordering::SeqCst);
                panic!("{expected} action did not reach the producer checkpoint: calls={calls}, diagnostics={diagnostics:?}");
            }
            let admitted = store.load_agui_run(&owner, &target, &thread, "wire-action").await.unwrap().unwrap();
            if expected == "cancelled" {
                assert!(manager.cancel_run_for_context(&user, &admitted.native_run_id).await);
            }
            let frames = bound_stream(reply).await;
            assert_eq!(frames.iter().filter(|frame| matches!(frame["type"].as_str(), Some("RUN_ERROR" | "RUN_FINISHED"))).count(), 1);
            let original = store.load_a2ui_action(&owner, &target, &source.session_id, "bound-intent").await.unwrap().unwrap();
            assert_eq!(original.source_run_id, source.native_run_id);
            assert_eq!(original.surface_generation, generation);
            assert_eq!(original.agent_id, target);
            assert!(original.wire_binding.as_ref().unwrap().native_agent_id.is_none());
            assert_eq!(admitted.native_agent_id.as_deref(), Some(resolved.artifact.id.as_str()));
            let native = manager.get_run(&admitted.native_run_id).await.unwrap();
            assert_eq!(native.agent_id, resolved.artifact.id);
            assert_eq!(native.status, match expected { "completed" => RunStatus::Done, "failed" => RunStatus::Error, _ => RunStatus::Cancelled });
            let receipt_path = format!("/agents/{target}/threads/{thread}/actions/bound-intent");
            let (code, ack) = bound_json(&app, &receipt_path).await;
            assert_eq!(code, StatusCode::OK);
            assert_eq!(ack["outcome"], expected);
            assert_eq!(ack["wire_run_id"], "wire-action");
            assert_eq!(ack["continuation_run_id"], admitted.native_run_id);
            let before = driver.calls.load(Ordering::SeqCst);
            let mut retry = input.clone(); retry["runId"] = json!("retry-wire");
            let replay = bound_stream(bound_request(&app, "POST", &path, Some(retry.clone())).await).await;
            assert_eq!(replay.iter().filter(|frame| matches!(frame["type"].as_str(), Some("RUN_ERROR" | "RUN_FINISHED"))).count(), 1);
            let mut changed = retry.clone(); changed["forwardedProps"]["a2uiAction"]["userAction"]["context"]["note"] = json!("changed");
            assert_eq!(bound_request(&app, "POST", &path, Some(changed)).await.status(), StatusCode::CONFLICT);
            assert_eq!(driver.calls.load(Ordering::SeqCst), before);
            assert!(store.load_agui_run(&owner, &target, &thread, "retry-wire").await.unwrap().is_none());
            assert_eq!(store.load_a2ui_action(&owner, &target, &source.session_id, "bound-intent").await.unwrap(), Some(original.clone()));
            originals.push((receipt_path, retry, original, ack));
        }
        assert_eq!(driver.calls.load(Ordering::SeqCst), 3);
        assert_eq!(driver.admitted.load(Ordering::SeqCst), 3);
        driver.proof.lock().unwrap().take();
        drop(app); drop(catalog);
        close(manager).await;
        assert!(store.close_scope().await.engine_close_acknowledged);
        drop(store);
        let store = Arc::new(SurrealDbProvider::new(&endpoint, None, None, None, None).await.unwrap());
        let manager = self::manager(store.clone(), driver.clone()).await;
        let app = agui_router(manager.clone(), &user); // Deliberately empty mutable catalog.
        let foreign_app = agui_router(manager.clone(), &foreign);
        for (path, retry, original, ack) in originals {
            assert!(manager.get_run(&original.continuation_run_id).await.is_none());
            assert_eq!(bound_json(&app, &path).await, (StatusCode::OK, ack));
            assert_eq!(bound_request(&foreign_app, "GET", &path, None).await.status(), StatusCode::NOT_FOUND);
            let frames = bound_stream(bound_request(&app, "POST", &format!("/agents/{target}"), Some(retry)).await).await;
            assert_eq!(frames.iter().filter(|frame| matches!(frame["type"].as_str(), Some("RUN_ERROR" | "RUN_FINISHED"))).count(), 1);
            assert_eq!(store.load_a2ui_action(&owner, &target, &original.thread_id, "bound-intent").await.unwrap(), Some(original));
        }
        assert_eq!(driver.calls.load(Ordering::SeqCst), 3, "resident/reopened original intent never dispatches twice");
        drop(app); drop(foreign_app);
        close(manager).await;
        assert!(store.close_scope().await.engine_close_acknowledged);
    }

    #[tokio::test]
    async fn deployed_receipt_requires_original_binding_source_and_native_proof_without_dispatch() {
        let user = external_identity_host::user("a2ui-recovery", false, "a2ui-owner").await;
        let foreign = external_identity_host::user("a2ui-recovery-foreign", false, "a2ui-foreign").await;
        let directory = tempfile::tempdir().unwrap();
        let store = Arc::new(SurrealDbProvider::new(&format!("surrealkv://{}", directory.path().join("bound-proof.db").display()), None, None, None, None).await.unwrap());
        let driver = Arc::new(HeldProducer::default());
        let manager = manager(store.clone(), driver.clone()).await;
        let app = agui_router(manager.clone(), &user);
        let selector = universal_agent_runtime::uar::api::targets::agent_target_id("r028-proof");
        let native = "urn:uar:r028:original-native";
        for name in ["missing-binding", "missing-native-identity", "conflicting-native-identity", "conflicting-input",
            "missing-native-run", "foreign-native-run", "wrong-native-agent", "wrong-native-session",
            "missing-source", "foreign-source", "wrong-source-agent", "wrong-source-session",
            "pending", "running", "paused"] {
            let thread = format!("proof-{name}");
            let (source, generation) = seed_source(&store, &user, &selector, native, &thread, Some("workspace:r028")).await;
            let input = json!({"threadId":thread,"runId":"new-retry-wire","messages":[],
                "forwardedProps":{"a2uiAction":{"interactionId":"proof-intent","surfaceGeneration":generation,
                    "userAction":{"surfaceId":"surface","sourceComponentId":"button","name":"confirm","context":{"value":1}}}}});
            let mut row = AguiRunBinding { owner: source.owner.clone(), agent_id: selector.clone(),
                native_agent_id: Some(native.into()), thread_id: thread.clone(), wire_run_id: "original-proof-wire".into(),
                native_run_id: format!("native-proof-{name}"), session_id: source.session_id.clone(), client_message_id: None,
                action_interaction_id: Some("proof-intent".into()), workspace_id: source.workspace_id.clone(),
                input_digest: digest(&json!({"original":"wire input"})), ordinal: 2, revision: 1, start_cursor: 0,
                previous_wire_run_id: Some(source.wire_run_id.clone()), resumed_tool_ids: vec![], terminal: None };
            if name == "missing-native-identity" { row.native_agent_id = None; }
            let mut snapshot = row.clone();
            snapshot.native_agent_id = if name == "conflicting-native-identity" { Some("different-original-native".into()) } else { None };
            if name == "conflicting-input" { snapshot.input_digest = digest(&json!({"different":"wire input"})); }
            let original = A2uiActionReceipt { owner: row.owner.clone(), agent_id: selector.clone(),
                thread_id: row.session_id.clone(), interaction_id: "proof-intent".into(), source_run_id: if name == "missing-source" { "missing-source".into() } else { source.native_run_id.clone() },
                surface_id: "surface".into(), surface_generation: generation.clone(), component_id: "button".into(), action_name: "confirm".into(),
                semantic_digest: digest(&json!({"surface":"surface","generation":generation,"component":"button","name":"confirm","context":{"value":1},"data_model":null,"catalog":null,"user_message":null})),
                continuation_run_id: row.native_run_id.clone(), wire_binding: Some(snapshot) };
            if name != "missing-binding" { assert!(store.create_agui_run(&row, Some(&source)).await.unwrap()); }
            if name != "missing-native-run" {
                let status = match name { "pending" => RunStatus::Pending, "running" => RunStatus::Running, "paused" => RunStatus::Paused, _ => RunStatus::Done };
                persist_run(&store, if name == "foreign-native-run" { &foreign } else { &user }, &row.native_run_id,
                    if name == "wrong-native-agent" { "wrong-agent" } else { native },
                    if name == "wrong-native-session" { "wrong-session" } else { &row.session_id }, status).await;
            }
            if matches!(name, "foreign-source" | "wrong-source-agent" | "wrong-source-session") {
                persist_run(&store, if name == "foreign-source" { &foreign } else { &user }, &source.native_run_id,
                    if name == "wrong-source-agent" { "wrong-source-agent" } else { native },
                    if name == "wrong-source-session" { "wrong-source-session" } else { &source.session_id }, RunStatus::Done).await;
            }
            assert!(store.create_a2ui_action(&original).await.unwrap());
            let (code, ack) = bound_json(&app, &format!("/agents/{selector}/threads/{thread}/actions/proof-intent")).await;
            assert_eq!(code, StatusCode::OK, "{name}");
            assert_eq!(ack["outcome"], "uncertain", "{name}: historical or conflicting proof never certifies settlement");
            let retry = bound_request(&app, "POST", &format!("/agents/{selector}"), Some(input)).await;
            // A nonresident nonterminal has an authenticated binding but no
            // resumable producer; projection refuses rather than redispatches.
            if !matches!(name, "pending" | "running" | "paused") {
                assert_eq!(retry.status(), StatusCode::CONFLICT, "{name}");
            }
            drop(retry);
            assert_eq!(store.load_a2ui_action(&original.owner, &selector, &original.thread_id, "proof-intent").await.unwrap(), Some(original));
            assert!(store.load_agui_run(&source.owner, &selector, &thread, "new-retry-wire").await.unwrap().is_none());
        }
        assert_eq!(driver.calls.load(Ordering::SeqCst), 0, "proof reconciliation cannot dispatch or resolve today's catalog");
        drop(app);
        close(manager).await;
        assert!(store.close_scope().await.engine_close_acknowledged);
    }

}
