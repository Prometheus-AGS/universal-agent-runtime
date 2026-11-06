//! R022/R023 real durable terminal recovery regression source. Execution requires a
//! later explicit gate and serial external a2ui-recovery identity profiles.
#![cfg(all(feature = "server", feature = "surreal-backend"))]

#[expect(dead_code, reason = "shared identity adapter exposes profiles for other functional matrices")]
#[path = "support/external_identity_host.rs"]
mod external_identity_host;

use axum::{Extension, Router, body::{Body, to_bytes}, http::{Request, StatusCode}};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{pin::Pin, sync::{Arc, atomic::{AtomicUsize, Ordering}}};
use tokio::sync::RwLock;
use tower::ServiceExt;
use universal_agent_runtime::{
    config::{LlmConfig, SecurityConfig, ServiceInstanceConfig},
    llm::{LlmDriver, LlmRequest}, mcp::registry::McpRegistry, session::SessionStore,
    uar::{api::{agui, routes::RunApiState},
        compiler::collaboration::CollaborationCatalogService,
        domain::{events::NormalizedEvent, runs::{Run, RunStatus}},
        persistence::{PersistenceLayer, agui_runs::{AguiRunBinding, AguiTerminal},
            providers::surreal::SurrealDbProvider, run_history::{RunHistoryRecord, RunHistoryEvent}},
        rag::embeddings::{UnavailableEmbeddingBackend, swappable::SwappableEmbeddingBackend},
        runtime::{manager::RunManager, matching::VectorMatcher,
            skills::{SkillRegistry, service::{SkillService, SkillMatchingConfig, SkillMatchingAlgorithm, SkillMatchingEngine}}},
        security::claims::UserContext, service_instance::ServiceInstanceAuthority},
};

#[derive(Default)]
struct NoDispatch(
    // Historical/undeclared dispatch attempts remain forbidden.
    AtomicUsize,
    // Only deliberate new successor requests may reach the supplied driver.
    AtomicUsize,
    std::sync::Mutex<Vec<Vec<Value>>>,
    // Optional actual-admission observer: owner, wire selector, thread, artifact.
    std::sync::Mutex<Option<(Arc<SurrealDbProvider>, String, String, String, String)>>,
    AtomicUsize,
);
#[async_trait::async_trait]
impl LlmDriver for NoDispatch {
    async fn stream(&self, request: LlmRequest) -> anyhow::Result<Pin<Box<dyn futures::Stream<Item = anyhow::Result<universal_agent_runtime::normalized::NormalizedEvent>> + Send>>> {
        let successor = request.messages.iter().rev().find(|message| message["role"] == "user")
            .and_then(|message| message["content"].as_str())
            .is_some_and(|content| content.starts_with("R022 deliberate successor for native-"));
        if !successor {
            self.0.fetch_add(1, Ordering::SeqCst);
            anyhow::bail!("historical reconnect must not dispatch")
        }
        let reservation = self.3.lock().unwrap().clone();
        if let Some((store, owner, selector, thread, artifact)) = reservation {
            let rows = store.list_agui_runs(&owner, &selector, &thread).await?;
            let row = rows.into_iter().max_by_key(|row| row.ordinal)
                .ok_or_else(|| anyhow::anyhow!("native dispatch preceded durable wire reservation"))?;
            anyhow::ensure!(row.native_agent_id.as_deref() == Some(artifact.as_str())
                && row.terminal.is_none(), "dispatch reservation did not capture actual native identity");
            self.4.fetch_add(1, Ordering::SeqCst);
        }
        self.1.fetch_add(1, Ordering::SeqCst);
        self.2.lock().unwrap().push(request.messages);
        use universal_agent_runtime::normalized::NormalizedEvent as ProviderEvent;
        Ok(Box::pin(futures::stream::iter(vec![
            Ok(ProviderEvent::MessageDelta { text: "Successor controlled driver input confirmed".into() }),
            Ok(ProviderEvent::Done),
        ])))
    }
}

async fn manager(store: Arc<dyn PersistenceLayer>, driver: Arc<dyn LlmDriver>) -> Arc<RunManager> {
    manager_with_sources(store, driver, None).await
}

async fn manager_with_sources(store: Arc<dyn PersistenceLayer>, driver: Arc<dyn LlmDriver>,
    sources: Option<(Arc<universal_agent_runtime::llm::ProviderRegistry>,
        Arc<universal_agent_runtime::uar::settings::manager::SettingsManager>)>) -> Arc<RunManager> {
    // The default persisted matching engine is Service. Supply its real owned
    // resource and explicit supported Keyword algorithm; no auxiliary model or
    // absent-service fallback may stand in for successor admission.
    let skills = Arc::new(SkillService::new(Some(store.clone()), None));
    skills.set_matching_config(SkillMatchingConfig {
        engine: SkillMatchingEngine::Service, algorithm: SkillMatchingAlgorithm::Keyword,
        threshold: 0.5, margin_threshold: 0.0, top_k: 1, model_name: None,
    }).await;
    let manager = RunManager::new(
        LlmConfig { model: "openai/gpt-4o".into(), ..LlmConfig::default() },
        Arc::new(McpRegistry::new_empty()), SessionStore::new(),
        Arc::new(RwLock::new(SkillRegistry::default())),
        Arc::new(VectorMatcher::new(Arc::new(SwappableEmbeddingBackend::new(Arc::new(
            UnavailableEmbeddingBackend::new(384, "historical recovery does not embed"),
        ))), 0.75)), Some(store),
    ).await.with_skill_service(skills).with_llm_driver(driver);
    Arc::new(if let Some((registry, settings)) = sources {
        manager.with_provider_registry(registry).with_settings_manager(settings)
    } else { manager })
}

fn router(manager: Arc<RunManager>, user: &UserContext) -> Router {
    agui::build_router(SecurityConfig::default()).with_state(Arc::new(RunApiState {
        manager, collaboration_catalog: Arc::new(CollaborationCatalogService::in_memory()),
        service_instance: Arc::new(ServiceInstanceAuthority::new(
            &ServiceInstanceConfig::default(), "agui-recovery", "http://127.0.0.1", false, &[],
        )),
    })).layer(Extension(user.clone()))
}

fn original_input(thread: &str) -> Value {
    json!({"threadId":thread,"runId":"wire-run","messages":[{
        "id":"original-user-message","role":"user","content":"R022 original native request must not dispatch"
    }]})
}

fn binding(user: &UserContext, name: &str) -> AguiRunBinding {
    let thread = format!("wire-thread-{name}");
    let input_digest = Sha256::digest(original_input(&thread).to_string().as_bytes())
        .iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    AguiRunBinding {
        native_agent_id: None,
        owner: user.session_owner_key().unwrap(), agent_id: "agent".into(),
        thread_id: thread, wire_run_id: "wire-run".into(),
        native_run_id: format!("native-{name}"), session_id: format!("session-{name}"),
        client_message_id: None, action_interaction_id: None, workspace_id: None,
        input_digest, ordinal: 1, revision: 1, start_cursor: 0,
        previous_wire_run_id: None, resumed_tool_ids: Vec::new(), terminal: None,
    }
}

async fn seed(store: &SurrealDbProvider, user: &UserContext, row: &AguiRunBinding,
    status: RunStatus, event: Option<NormalizedEvent>, agent: &str, session: &str) {
    let owner_key = match &user.tenant_id {
        Some(tenant) => format!("v1:t:{}:{}:s:{}:{}", tenant.as_str().len(), tenant.as_str(), user.user_id.len(), user.user_id),
        None => format!("v1:s:{}:{}", user.user_id.len(), user.user_id),
    };
    store.upsert_run_history_record(&RunHistoryRecord {
        owner_key: Some(owner_key), run_id: row.native_run_id.clone(), revision: 1,
        run: serde_json::to_value(Run { run_id: row.native_run_id.clone(), agent_id: agent.into(),
            conversation_id: Some(session.into()), user_id: Some(user.user_id.clone()), status,
            context: json!({}) }).unwrap(),
    }).await.unwrap();
    if let Some(event) = event {
        store.append_run_history_event(&RunHistoryEvent { run_id: row.native_run_id.clone(), event_id: 1,
            event: serde_json::to_value(event).unwrap() }).await.unwrap();
    }
    assert!(store.create_agui_run(row, None).await.unwrap());
}

async fn reconnect(app: &Router, row: &AguiRunBinding, after: Option<&str>) -> (StatusCode, String) {
    let mut request = Request::builder().uri(format!("/agents/{}/threads/{}/runs/{}/stream",
        row.agent_id, row.thread_id, row.wire_run_id));
    if let Some(after) = after { request = request.header("last-event-id", after); }
    if let Some(workspace) = &row.workspace_id { request = request.header("x-uar-workspace-id", workspace); }
    let response = app.clone().oneshot(request.body(Body::empty()).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 128 * 1024).await.unwrap();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

fn terminals(body: &str) -> Vec<Value> {
    body.lines().filter_map(|line| line.strip_prefix("data: "))
        .map(|data| serde_json::from_str::<Value>(data).unwrap())
        .filter(|value| matches!(value["type"].as_str(), Some("RUN_ERROR" | "RUN_FINISHED")))
        .collect()
}

async fn retained(store: &SurrealDbProvider, row: &AguiRunBinding) -> AguiRunBinding {
    store.load_agui_run(&row.owner, &row.agent_id, &row.thread_id, &row.wire_run_id)
        .await.unwrap().unwrap()
}

#[tokio::test]
async fn reopened_unsealed_native_outcomes_close_truthfully_once_without_dispatch() {
    let user = external_identity_host::user("a2ui-recovery", false, "a2ui-owner").await;
    let foreign = external_identity_host::user("a2ui-recovery-foreign", false, "a2ui-foreign").await;
    let directory = tempfile::tempdir().unwrap();
    let endpoint = format!("surrealkv://{}", directory.path().join("agui-terminal.db").display());
    let store = SurrealDbProvider::new(&endpoint, None, None, None, None).await.unwrap();
    // Public AG-UI resolves its path selector through the actual persisted
    // agent catalog. This owned definition survives the same store reopen.
    let mut agent = universal_agent_runtime::uar::defaults::default_agent();
    agent.id = "agent".into();
    agent.policy.skills.max_active = 0;
    agent.policy.tools.allow.clear();
    agent.memory.kb.enabled = false;
    universal_agent_runtime::uar::domain::agent_store::upsert_agent(&store, &agent).await.unwrap();
    let mut originals = Vec::new();
    for (name, status) in [("done", RunStatus::Done), ("error", RunStatus::Error), ("cancel", RunStatus::Cancelled),
        ("cancel-persist-error", RunStatus::Cancelled), ("cancel-multiple-errors", RunStatus::Cancelled),
        ("cancel-persist-diagnostic", RunStatus::Cancelled)] {
        let row = binding(&user, name);
        let event = match status {
            RunStatus::Done => NormalizedEvent::RunDone { run_id: row.native_run_id.clone() },
            RunStatus::Error => NormalizedEvent::Error { run_id: row.native_run_id.clone(), code: "original_error".into(), message: "Original failure".into() },
            RunStatus::Cancelled => NormalizedEvent::Cancelled { run_id: row.native_run_id.clone(),
                usage: (name != "cancel").then(|| universal_agent_runtime::uar::domain::events::RunUsage {
                    input_tokens: Some(7), output_tokens: Some(3), total_tokens: Some(10),
                    model: Some("original-cancelled-model".into()), ..Default::default()
                }) },
            _ => unreachable!(),
        };
        let mut history = Vec::new();
        if name == "cancel-persist-diagnostic" {
            // New host diagnostic ordering; seeded evidence alone is not an
            // actual persistence fault (the armed manager scenario below is).
            history.push(NormalizedEvent::PresentationDiagnostic { run_id: row.native_run_id.clone(),
                code: "session_persist_failed".into(), message: "Conversation terminal could not be saved".into() });
        }
        if name == "cancel-persist-error" || name == "cancel-multiple-errors" {
            // Seed the legacy production manager ordering. This is durable
            // recovery evidence, not injected save failure or crash timing.
            history.push(NormalizedEvent::Error { run_id: row.native_run_id.clone(),
                code: "session_persist_failed".into(), message: "Conversation terminal could not be saved".into() });
        }
        if name == "cancel-multiple-errors" {
            // The optional native observer can emit another diagnostic after
            // session persistence and before cancellation. Only seed its source
            // event shape here; this does not execute or qualify that observer.
            history.push(NormalizedEvent::Error { run_id: row.native_run_id.clone(),
                code: "NATIVE_ADMISSION_GATE_FINALIZATION_FAILED".into(),
                message: "Native acceptance observer could not finalize".into() });
        }
        history.push(event.clone());
        let terminal_cursor = history.len() as u64;
        if status == RunStatus::Error {
            // Real ordinary error paths append a final Done marker. It cannot
            // replace the original error or become successful wire completion.
            history.push(NormalizedEvent::RunDone { run_id: row.native_run_id.clone() });
        }
        seed(&store, &user, &row, status.clone(), Some(history[0].clone()), &row.agent_id, &row.session_id).await;
        let mut expected_evidence = Vec::new();
        for (index, native) in history.into_iter().enumerate() {
            let cursor = index as u64 + 1;
            let native = serde_json::to_value(native).unwrap();
            if index > 0 {
                store.append_run_history_event(&RunHistoryEvent { run_id: row.native_run_id.clone(),
                    event_id: cursor, event: native.clone() }).await.unwrap();
            }
            expected_evidence.push(json!({"cursor":cursor,"event":native}));
        }
        originals.push((row, status, serde_json::to_value(event).unwrap(), terminal_cursor, expected_evidence));
    }
    let closed = store.close_scope().await;
    assert!(closed.scope_released && closed.engine_close_acknowledged, "{closed:?}");
    drop(store);
    let store = Arc::new(SurrealDbProvider::new(&endpoint, None, None, None, None).await.unwrap());
    let driver = Arc::new(NoDispatch::default());
    let manager = manager(store.clone(), driver.clone()).await;
    let app = router(manager.clone(), &user);
    let foreign_app = router(manager.clone(), &foreign);
    let mut admitted_successors = 0;
    for (original, status, event, terminal_cursor, expected_evidence) in originals {
        let arrivals_before_original = driver.1.load(Ordering::SeqCst);
        assert!(manager.get_run(&original.native_run_id).await.is_none());
        assert!(retained(&store, &original).await.terminal.is_none());
        assert_eq!(reconnect(&foreign_app, &original, None).await.0, StatusCode::NOT_FOUND);
        // Concurrent independent reconnects contend on the actual store CAS.
        let (first, second) = tokio::join!(reconnect(&app, &original, None), reconnect(&app, &original, None));
        assert_eq!(first.0, StatusCode::OK);
        assert_eq!(second.0, StatusCode::OK);
        let sealed = retained(&store, &original).await;
        let terminal = sealed.terminal.as_ref().unwrap();
        assert_eq!(sealed.revision, 2);
        assert_eq!(terminal.source_cursor, terminal_cursor);
        assert_eq!(terminal.event["type"], "RUN_ERROR");
        assert_eq!(terminal.event["code"], "host_cleanup_unconfirmed");
        assert_eq!(terminal.event["metadata"]["uar"]["cleanup_unconfirmed"], true);
        assert_eq!(terminal.event["metadata"]["uar"]["nativeStatus"], serde_json::to_value(&status).unwrap());
        assert_eq!(terminal.event["metadata"]["uar"]["nativeTerminal"], event);
        assert_eq!(terminal.event["metadata"]["uar"]["nativeTerminalCursor"], terminal_cursor);
        let evidence = terminal.event["metadata"]["uar"]["nativeTerminalEvidence"].as_array().unwrap();
        assert_eq!(evidence, &expected_evidence);
        assert_eq!(evidence[(terminal_cursor - 1) as usize], json!({"cursor":terminal_cursor,"event":event}));
        assert_eq!(evidence.len(), terminal_cursor as usize + usize::from(status == RunStatus::Error));
        assert_eq!(terminals(&first.1), vec![terminal.event.clone()]);
        assert_eq!(terminals(&second.1), vec![terminal.event.clone()]);
        let cursor = format!("{}:{terminal_cursor}:100", original.storage_key());
        assert!(first.1.contains(&format!("id: {cursor}")));
        assert!(terminals(&reconnect(&app, &original, Some(&cursor)).await.1).is_empty());
        assert_eq!(retained(&store, &original).await, sealed);
        let retry = app.clone().oneshot(Request::builder().method("POST").uri("/agents/agent")
            .header("content-type", "application/json")
            .body(Body::from(original_input(&original.thread_id).to_string())).unwrap()).await.unwrap();
        assert_eq!(retry.status(), StatusCode::OK);
        let retry = to_bytes(retry.into_body(), 128 * 1024).await.unwrap();
        assert_eq!(terminals(std::str::from_utf8(&retry).unwrap()), vec![terminal.event.clone()]);
        assert_eq!(retained(&store, &original).await, sealed);
        let mut competing = original.clone();
        competing.revision += 1;
        competing.terminal = Some(AguiTerminal { source_cursor: 1,
            event: json!({"type":"RUN_FINISHED","outcome":{"type":"success"}}), interrupt_ids: vec![] });
        assert!(!store.compare_and_swap_agui_run(&original, &competing).await.unwrap());
        assert_eq!(retained(&store, &original).await, sealed);
        assert_eq!(driver.0.load(Ordering::SeqCst), 0, "original never dispatches");
        assert_eq!(driver.1.load(Ordering::SeqCst), arrivals_before_original,
            "original reconnect, retry and CAS cannot create successor work");

        let successor_input = format!("R022 deliberate successor for {}", original.native_run_id);
        let successor = json!({"threadId":original.thread_id,"runId":"next-wire-run",
            "parentRunId":original.wire_run_id,"messages":[{
                "id":"successor-user-message","role":"user","content":successor_input
            }]});
        let mut wrong_parent = successor.clone();
        wrong_parent["parentRunId"] = json!("wrong-predecessor");
        let rejected = app.clone().oneshot(Request::builder().method("POST").uri("/agents/agent")
            .header("content-type", "application/json")
            .body(Body::from(wrong_parent.to_string())).unwrap()).await.unwrap();
        assert_eq!(rejected.status(), StatusCode::CONFLICT);
        let rejected = to_bytes(rejected.into_body(), 128 * 1024).await.unwrap();
        assert_eq!(serde_json::from_slice::<Value>(&rejected).unwrap()["code"], "thread_run_not_closed");
        assert!(store.load_agui_run(&original.owner, &original.agent_id, &original.thread_id,
            "next-wire-run").await.unwrap().is_none(), "rejected parent reserves no successor");
        assert_eq!(driver.1.load(Ordering::SeqCst), arrivals_before_original);
        assert_eq!(driver.0.load(Ordering::SeqCst), 0);

        // This POST traverses the unchanged public parent/authority/catalog and
        // runtime admission path. No lower-level binding insertion stands in.
        let accepted = app.clone().oneshot(Request::builder().method("POST").uri("/agents/agent")
            .header("content-type", "application/json")
            .body(Body::from(successor.to_string())).unwrap()).await.unwrap();
        assert_eq!(accepted.status(), StatusCode::OK);
        let accepted = to_bytes(accepted.into_body(), 128 * 1024).await.unwrap();
        let accepted = std::str::from_utf8(&accepted).unwrap();
        let successor_terminals = terminals(accepted);
        assert_eq!(successor_terminals.len(), 1, "{accepted}");
        assert_eq!(successor_terminals[0]["type"], "RUN_FINISHED", "{accepted}");
        assert_eq!(successor_terminals[0]["outcome"]["type"], "success");
        let next = store.load_agui_run(&original.owner, &original.agent_id, &original.thread_id,
            "next-wire-run").await.unwrap().unwrap();
        assert_eq!(next.ordinal, 2);
        assert_eq!(next.previous_wire_run_id.as_deref(), Some(original.wire_run_id.as_str()));
        assert_eq!(next.session_id, original.session_id);
        assert_ne!(next.native_run_id, original.native_run_id);
        let native = manager.get_run(&next.native_run_id).await.unwrap();
        assert_eq!(native.status, RunStatus::Done);
        assert_eq!(native.agent_id, original.agent_id);
        assert_eq!(next.native_agent_id.as_deref(), Some(native.agent_id.as_str()));
        assert_eq!(native.conversation_id.as_deref(), Some(original.session_id.as_str()));
        assert_eq!(native.user_id.as_deref(), Some(user.user_id.as_str()));
        admitted_successors += 1;
        assert_eq!(driver.1.load(Ordering::SeqCst), arrivals_before_original + 1,
            "one distinct successor controlled driver-input arrival");
        assert_eq!(driver.0.load(Ordering::SeqCst), 0, "original still never dispatches");
        let captures = driver.2.lock().unwrap();
        let latest = captures.last().unwrap();
        assert_eq!(latest.iter().rev().find(|message| message["role"] == "user").unwrap()["content"],
            successor_input, "controlled driver input belongs to the new native request");
        drop(captures);
        assert_eq!(retained(&store, &original).await, sealed, "new admission cannot rewrite original");
        assert_eq!(terminals(&reconnect(&app, &original, None).await.1), vec![terminal.event.clone()]);
        assert_eq!(driver.1.load(Ordering::SeqCst), arrivals_before_original + 1,
            "old reconnect after new admission still cannot dispatch");
    }
    assert_eq!(driver.0.load(Ordering::SeqCst), 0);
    assert_eq!(driver.1.load(Ordering::SeqCst), admitted_successors);
    assert_eq!(driver.2.lock().unwrap().len(), admitted_successors);
    drop(foreign_app);
    drop(app);
    manager.shutdown_graph_roots().await.unwrap();
    manager.shutdown_terminals().await.unwrap();
    manager.shutdown_sandboxes().await.unwrap();
    manager.shutdown_run_history().await.unwrap();
    drop(manager);
    assert!(store.close_scope().await.engine_close_acknowledged);
}

#[tokio::test]
async fn absent_foreign_mismatched_or_nonterminal_native_evidence_never_seals_or_dispatches() {
    let user = external_identity_host::user("a2ui-recovery", false, "a2ui-owner").await;
    let foreign = external_identity_host::user("a2ui-recovery-foreign", false, "a2ui-foreign").await;
    let directory = tempfile::tempdir().unwrap();
    let endpoint = format!("surrealkv://{}", directory.path().join("negative.db").display());
    let store = Arc::new(SurrealDbProvider::new(&endpoint, None, None, None, None).await.unwrap());
    for name in ["foreign-owner", "agent", "session", "event-run", "status", "missing-event", "nonterminal", "missing-record", "cursor", "malformed-event", "conflicting-terminal", "record-run"] {
        let mut row = binding(&user, name);
        if name == "cursor" { row.start_cursor = 1; }
        if name == "missing-record" {
            assert!(store.create_agui_run(&row, None).await.unwrap());
            continue;
        }
        let run_id = if name == "event-run" { "other-native".into() } else { row.native_run_id.clone() };
        let event = (name != "missing-event").then_some(NormalizedEvent::RunDone { run_id });
        let status = match name { "nonterminal" => RunStatus::Running, "status" => RunStatus::Cancelled, _ => RunStatus::Done };
        seed(&store, if name == "foreign-owner" { &foreign } else { &user }, &row, status, event,
            if name == "agent" { "other-agent" } else { &row.agent_id },
            if name == "session" { "other-session" } else { &row.session_id }).await;
        if name == "malformed-event" {
            store.append_run_history_event(&RunHistoryEvent { run_id: row.native_run_id.clone(), event_id: 1,
                event: json!({"unknown-terminal-shape":true}) }).await.unwrap();
        }
        if name == "conflicting-terminal" {
            store.append_run_history_event(&RunHistoryEvent { run_id: row.native_run_id.clone(), event_id: 2,
                event: serde_json::to_value(NormalizedEvent::Error { run_id: row.native_run_id.clone(),
                    code: "conflicting".into(), message: "Conflicting outcome".into() }).unwrap() }).await.unwrap();
        }
        if name == "record-run" {
            let mut record = store.load_run_history_record(&row.native_run_id).await.unwrap().unwrap();
            record.revision += 1;
            record.run["run_id"] = json!("different-native-run");
            store.upsert_run_history_record(&record).await.unwrap();
        }
    }
    let driver = Arc::new(NoDispatch::default());
    let manager = manager(store.clone(), driver.clone()).await;
    let app = router(manager.clone(), &user);
    for name in ["foreign-owner", "agent", "session", "event-run", "status", "missing-event", "nonterminal", "missing-record", "cursor", "malformed-event", "conflicting-terminal", "record-run"] {
        let row = binding(&user, name);
        let result = reconnect(&app, &row, None).await;
        assert_eq!(result.0, StatusCode::OK);
        let events = terminals(&result.1);
        assert_eq!(events.len(), 1, "{name}: {}", result.1);
        assert_eq!(events[0]["type"], "RUN_ERROR", "{name}");
        assert!(retained(&store, &row).await.terminal.is_none(), "{name}");
        assert_eq!(retained(&store, &row).await.revision, 1);
    }
    assert_eq!(driver.0.load(Ordering::SeqCst), 0);
    drop(app);
    manager.shutdown_graph_roots().await.unwrap();
    manager.shutdown_terminals().await.unwrap();
    manager.shutdown_sandboxes().await.unwrap();
    manager.shutdown_run_history().await.unwrap();
    drop(manager);
    assert!(store.close_scope().await.engine_close_acknowledged);
}

#[tokio::test]
async fn reopened_cancellation_requires_ordered_matching_terminal_evidence_without_dispatch() {
    let user = external_identity_host::user("a2ui-recovery", false, "a2ui-owner").await;
    let foreign = external_identity_host::user("a2ui-recovery-foreign", false, "a2ui-foreign").await;
    let directory = tempfile::tempdir().unwrap();
    let endpoint = format!("surrealkv://{}", directory.path().join("cancel-negative.db").display());
    let store = SurrealDbProvider::new(&endpoint, None, None, None, None).await.unwrap();
    let cases = ["missing-cancel", "error-after-cancel", "foreign-host-diagnostic", "host-diagnostic-after-cancel",
        "done-before-cancel", "done-after-cancel",
        "duplicate-cancel", "foreign-diagnostic", "foreign-cancel", "cursor-cancel", "malformed-before-cancel",
        "cancel-foreign-owner", "cancel-agent", "cancel-session", "cancel-under-error", "cancel-under-done"];
    let mut originals = Vec::new();
    for name in cases {
        let mut row = binding(&user, name);
        if name == "cursor-cancel" { row.start_cursor = 2; }
        let diagnostic_run = if matches!(name, "foreign-diagnostic" | "foreign-host-diagnostic") {
            "other-native".into()
        } else { row.native_run_id.clone() };
        let diagnostic = if matches!(name, "foreign-host-diagnostic" | "host-diagnostic-after-cancel") {
            NormalizedEvent::PresentationDiagnostic { run_id: diagnostic_run,
                code: "session_persist_failed".into(), message: "Conversation terminal could not be saved".into() }
        } else {
            NormalizedEvent::Error { run_id: diagnostic_run,
                code: "session_persist_failed".into(), message: "Conversation terminal could not be saved".into() }
        };
        let cancelled = serde_json::to_value(NormalizedEvent::Cancelled {
            run_id: if name == "foreign-cancel" { "other-native".into() } else { row.native_run_id.clone() },
            usage: None,
        }).unwrap();
        let done = serde_json::to_value(NormalizedEvent::RunDone { run_id: row.native_run_id.clone() }).unwrap();
        let mut history = vec![serde_json::to_value(&diagnostic).unwrap()];
        if name == "done-before-cancel" { history.push(done.clone()); }
        if name == "malformed-before-cancel" { history.push(json!({"unknown-terminal-shape":true})); }
        if name != "missing-cancel" { history.push(cancelled.clone()); }
        match name {
            "error-after-cancel" | "host-diagnostic-after-cancel" => history.push(serde_json::to_value(&diagnostic).unwrap()),
            "done-after-cancel" => history.push(done),
            "duplicate-cancel" => history.push(cancelled),
            _ => {},
        }
        let status = match name {
            "cancel-under-error" => RunStatus::Error,
            "cancel-under-done" => RunStatus::Done,
            _ => RunStatus::Cancelled,
        };
        seed(&store, if name == "cancel-foreign-owner" { &foreign } else { &user }, &row,
            status, Some(diagnostic),
            if name == "cancel-agent" { "other-agent" } else { &row.agent_id },
            if name == "cancel-session" { "other-session" } else { &row.session_id }).await;
        for (index, event) in history.into_iter().enumerate().skip(1) {
            store.append_run_history_event(&RunHistoryEvent { run_id: row.native_run_id.clone(),
                event_id: index as u64 + 1, event }).await.unwrap();
        }
        originals.push((name, row));
    }
    let closed = store.close_scope().await;
    assert!(closed.scope_released && closed.engine_close_acknowledged, "{closed:?}");
    drop(store);
    let store = Arc::new(SurrealDbProvider::new(&endpoint, None, None, None, None).await.unwrap());
    let driver = Arc::new(NoDispatch::default());
    let manager = manager(store.clone(), driver.clone()).await;
    let app = router(manager.clone(), &user);
    for (name, row) in originals {
        assert!(manager.get_run(&row.native_run_id).await.is_none(), "{name}");
        let original = retained(&store, &row).await;
        for _ in 0..2 {
            let result = reconnect(&app, &row, None).await;
            assert_eq!(result.0, StatusCode::OK, "{name}");
            let events = terminals(&result.1);
            assert_eq!(events.len(), 1, "{name}: {}", result.1);
            assert_eq!(events[0]["type"], "RUN_ERROR", "{name}");
            assert_ne!(events[0]["code"], "host_cleanup_unconfirmed", "{name}: invalid evidence cannot seal");
            assert!(retained(&store, &row).await.terminal.is_none(), "{name}");
            assert_eq!(retained(&store, &row).await, original, "{name}: uncertain reconnect cannot rewrite binding");
            assert_eq!(driver.0.load(Ordering::SeqCst), 0, "{name}: original never dispatches");
            assert_eq!(driver.1.load(Ordering::SeqCst), 0, "{name}: no successor dispatch");
        }
    }
    assert!(driver.2.lock().unwrap().is_empty());
    drop(app);
    manager.shutdown_graph_roots().await.unwrap();
    manager.shutdown_terminals().await.unwrap();
    manager.shutdown_sandboxes().await.unwrap();
    manager.shutdown_run_history().await.unwrap();
    drop(manager);
    assert!(store.close_scope().await.engine_close_acknowledged);
}


#[derive(Default)]
struct HeldCancellation {
    entered: Arc<tokio::sync::Notify>,
    calls: AtomicUsize,
}
#[async_trait::async_trait]
impl LlmDriver for HeldCancellation {
    async fn stream(&self, _request: LlmRequest) -> anyhow::Result<Pin<Box<dyn futures::Stream<Item = anyhow::Result<universal_agent_runtime::normalized::NormalizedEvent>> + Send>>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        use futures::StreamExt;
        let entered = self.entered.clone();
        let mut announced = false;
        let usage = universal_agent_runtime::normalized::NormalizedEvent::Usage {
            prompt_tokens: 7, completion_tokens: 3, total_tokens: 10,
            cached_tokens: None, uncached_input_tokens: None, cache_creation_tokens: None,
        };
        Ok(Box::pin(futures::stream::iter(vec![Ok(usage)]).chain(futures::stream::poll_fn(move |_| {
            if !announced { entered.notify_one(); announced = true; }
            std::task::Poll::Pending
        }))))
    }
}

#[tokio::test]
async fn resident_cancellation_selects_final_cursor_and_refuses_conflicting_evidence() {
    use universal_agent_runtime::uar::runtime::turn::RunExecutionRequest;
    let user = external_identity_host::user("a2ui-recovery", false, "a2ui-owner").await;
    let directory = tempfile::tempdir().unwrap();
    let store = Arc::new(SurrealDbProvider::new(&format!("surrealkv://{}",
        directory.path().join("resident-cancellation.db").display()), None, None, None, None).await.unwrap());
    let driver = Arc::new(HeldCancellation::default());
    let manager = manager(store.clone(), driver.clone()).await;
    let app = router(manager.clone(), &user);
    let cases = ["ordered", "multiple", "host-diagnostic", "foreign-host-diagnostic", "trailing-host-diagnostic",
        "prefix", "done-before", "done-after", "duplicate",
        "foreign-diagnostic", "foreign-cancel", "trailing-error", "wrong-agent", "wrong-session", "wrong-run"];
    for (index, name) in cases.into_iter().enumerate() {
        let mut row = binding(&user, &format!("resident-{name}"));
        row.native_agent_id = Some("agent".into());
        let mut artifact = universal_agent_runtime::uar::defaults::default_agent();
        artifact.id = "agent".into();
        artifact.policy.skills.max_active = 0;
        artifact.policy.tools.allow.clear();
        artifact.memory.kb.enabled = false;
        let mut request = RunExecutionRequest::new(artifact, "hold cancellation producer".into())
            .with_user_context(&user).unwrap();
        request.session_id = Some(row.session_id.clone());
        row.native_run_id = manager.execute_request(request).await;
        tokio::time::timeout(std::time::Duration::from_secs(30), driver.entered.notified()).await.unwrap();
        let diagnostic_run = if matches!(name, "foreign-diagnostic" | "foreign-host-diagnostic") {
            "foreign-native".into()
        } else { row.native_run_id.clone() };
        let diagnostic = if matches!(name, "host-diagnostic" | "foreign-host-diagnostic" | "trailing-host-diagnostic") {
            NormalizedEvent::PresentationDiagnostic { run_id: diagnostic_run,
                code: "session_persist_failed".into(), message: "Conversation terminal could not be saved".into() }
        } else {
            NormalizedEvent::Error { run_id: diagnostic_run,
                code: "session_persist_failed".into(), message: "Conversation terminal could not be saved".into() }
        };
        // Public host event injection supplies diagnostic evidence; cancellation
        // itself is produced by the actual retained manager, not a seeded status.
        manager.emit_to_run(&row.native_run_id, diagnostic.clone()).await;
        if name == "multiple" { manager.emit_to_run(&row.native_run_id, diagnostic.clone()).await; }
        if name == "done-before" {
            manager.emit_to_run(&row.native_run_id, NormalizedEvent::RunDone { run_id: row.native_run_id.clone() }).await;
        }
        let mut events = manager.subscribe(&row.native_run_id).await.unwrap();
        assert!(manager.cancel_run_for_context(&user, &row.native_run_id).await);
        let final_cursor = tokio::time::timeout(std::time::Duration::from_secs(30), async {
            loop {
                let event = events.recv().await.unwrap();
                if matches!(event.event, NormalizedEvent::Cancelled { .. }) { break event.id; }
            }
        }).await.unwrap();
        assert_eq!(manager.get_run(&row.native_run_id).await.unwrap().status, RunStatus::Cancelled);
        let original_cancel = manager.history_since(&row.native_run_id, None).await.unwrap().into_iter()
            .find(|event| event.id == final_cursor).unwrap().event;
        match name {
            "prefix" => {
                row.start_cursor = final_cursor;
                manager.emit_to_run(&row.native_run_id, diagnostic.clone()).await;
            }
            "trailing-error" | "trailing-host-diagnostic" => manager.emit_to_run(&row.native_run_id, diagnostic).await,
            "done-after" => manager.emit_to_run(&row.native_run_id, NormalizedEvent::RunDone { run_id: row.native_run_id.clone() }).await,
            "duplicate" => manager.emit_to_run(&row.native_run_id, original_cancel.clone()).await,
            "foreign-cancel" => manager.emit_to_run(&row.native_run_id,
                NormalizedEvent::Cancelled { run_id: "foreign-native".into(), usage: None }).await,
            "wrong-agent" => row.native_agent_id = Some("other-agent".into()),
            "wrong-session" => row.session_id = "other-session".into(),
            "wrong-run" => row.native_run_id = "other-native-run".into(),
            _ => {},
        }
        assert!(store.create_agui_run(&row, None).await.unwrap());
        let before = retained(&store, &row).await;
        let result = if name == "prefix" {
            // Diagnostic-only settled history cannot invent a final cancellation.
            assert!(tokio::time::timeout(std::time::Duration::from_millis(100),
                reconnect(&app, &row, None)).await.is_err());
            (StatusCode::OK, String::new())
        } else { reconnect(&app, &row, None).await };
        assert_eq!(result.0, StatusCode::OK, "{name}");
        let after = retained(&store, &row).await;
        if matches!(name, "ordered" | "multiple" | "host-diagnostic") {
            let terminal = after.terminal.as_ref().unwrap();
            assert_eq!(terminal.source_cursor, final_cursor);
            assert_eq!(terminal.event["outcome"]["type"], "cancelled");
            assert_eq!(terminal.event["metadata"]["uar"]["nativeTerminal"], serde_json::to_value(&original_cancel).unwrap());
            let NormalizedEvent::Cancelled { usage: Some(usage), .. } = &original_cancel else {
                panic!("actual cancellation must preserve provider usage");
            };
            assert_eq!(usage.input_tokens, Some(7));
            assert_eq!(usage.output_tokens, Some(3));
            assert_eq!(usage.total_tokens, Some(10));
            let evidence = terminal.event["metadata"]["uar"]["nativeTerminalEvidence"].as_array().unwrap();
            let expected: Vec<_> = manager.history_since(&row.native_run_id, None).await.unwrap().into_iter()
                .filter(|event| matches!(&event.event, NormalizedEvent::Error { .. }
                    | NormalizedEvent::PresentationDiagnostic { .. } | NormalizedEvent::Cancelled { .. }))
                .map(|event| json!({"cursor":event.id,"event":event.event})).collect();
            assert_eq!(evidence, &expected);
            assert_eq!(evidence.iter().filter(|event| event["event"]["code"] == "session_persist_failed").count(),
                if name == "multiple" { 2 } else { 1 });
            assert_eq!(evidence[0]["event"]["code"], "session_persist_failed");
            if name == "host-diagnostic" {
                let frames: Vec<Value> = result.1.lines().filter_map(|line| line.strip_prefix("data: "))
                    .filter_map(|data| serde_json::from_str(data).ok()).collect();
                let projected = frames.iter().find(|frame| frame["name"] == "uar.presentation.diagnostic").unwrap();
                assert_eq!(projected["type"], "CUSTOM");
                assert_eq!(projected["metadata"]["uar"]["nativeEvent"], evidence[0]["event"]);
                assert_eq!(projected["metadata"]["uar"]["nativeCursor"], evidence[0]["cursor"]);
                assert_eq!(projected["metadata"]["uar"]["nativeRunId"], row.native_run_id);
            }
            assert_eq!(evidence.last().unwrap()["cursor"], final_cursor);
            assert_eq!(terminals(&result.1), vec![terminal.event.clone()]);
            assert_eq!(retained(&store, &row).await, after);
            assert_eq!(terminals(&reconnect(&app, &row, None).await.1), vec![terminal.event.clone()]);
        } else {
            assert_eq!(after, before, "{name}: contradictory/incomplete native proof must not seal");
            assert!(after.terminal.is_none());
        }
        assert_eq!(driver.calls.load(Ordering::SeqCst), index + 1, "reconnect cannot redispatch");
    }
    drop(app);
    manager.shutdown_graph_roots().await.unwrap();
    manager.shutdown_terminals().await.unwrap();
    manager.shutdown_sandboxes().await.unwrap();
    manager.shutdown_run_history().await.unwrap();
    drop(manager);
    assert!(store.close_scope().await.engine_close_acknowledged);
}

#[tokio::test]
async fn historical_target_requires_original_artifact_proof_without_catalog_resolution() {
    let user = external_identity_host::user("a2ui-recovery", false, "a2ui-owner").await;
    let foreign = external_identity_host::user("a2ui-recovery-foreign", false, "a2ui-foreign").await;
    let directory = tempfile::tempdir().unwrap();
    let endpoint = format!("surrealkv://{}", directory.path().join("target-history.db").display());
    let store = SurrealDbProvider::new(&endpoint, None, None, None, None).await.unwrap();
    let mut rows = Vec::new();
    for name in ["captured", "legacy-target", "wrong-agent", "wrong-session", "wrong-owner", "wrong-run", "legacy-direct"] {
        let mut row = binding(&user, name);
        if name != "legacy-direct" {
            row.agent_id = universal_agent_runtime::uar::api::targets::agent_target_id("original-binding");
            row.native_agent_id = (name != "legacy-target").then(|| "original-artifact".into());
        }
        let artifact = if name == "legacy-direct" { "agent" } else if name == "wrong-agent" { "rebound-artifact" } else { "original-artifact" };
        seed(&store, if name == "wrong-owner" { &foreign } else { &user }, &row, RunStatus::Done,
            Some(NormalizedEvent::RunDone { run_id: row.native_run_id.clone() }), artifact,
            if name == "wrong-session" { "other-session" } else { &row.session_id }).await;
        if name == "wrong-run" {
            let mut record = store.load_run_history_record(&row.native_run_id).await.unwrap().unwrap();
            record.revision += 1;
            record.run["run_id"] = json!("foreign-native");
            store.upsert_run_history_record(&record).await.unwrap();
        }
        rows.push((name, row));
    }
    assert!(store.close_scope().await.engine_close_acknowledged);
    drop(store);
    let store = Arc::new(SurrealDbProvider::new(&endpoint, None, None, None, None).await.unwrap());
    let driver = Arc::new(NoDispatch::default());
    let manager = manager(store.clone(), driver.clone()).await;
    // This router has an empty catalog: recovery cannot consult or depend on
    // the current deployment binding, including after its removal or replacement.
    let app = router(manager.clone(), &user);
    for (name, row) in rows {
        let before = retained(&store, &row).await;
        let response = reconnect(&app, &row, None).await;
        assert_eq!(response.0, StatusCode::OK);
        let after = retained(&store, &row).await;
        if matches!(name, "captured" | "legacy-direct") {
            assert_eq!(after.terminal.as_ref().unwrap().event["code"], "host_cleanup_unconfirmed");
            assert_eq!(after.native_agent_id, before.native_agent_id);
            assert_eq!(after.agent_id, before.agent_id);
        } else {
            assert_eq!(after, before, "{name}: never infer or rewrite original identity");
            assert_eq!(terminals(&response.1)[0]["code"], "projection_unavailable");
        }
    }
    assert_eq!(driver.0.load(Ordering::SeqCst), 0);
    assert_eq!(driver.1.load(Ordering::SeqCst), 0);
    drop(app);
    manager.shutdown_graph_roots().await.unwrap();
    manager.shutdown_terminals().await.unwrap();
    manager.shutdown_sandboxes().await.unwrap();
    manager.shutdown_run_history().await.unwrap();
    drop(manager);
    assert!(store.close_scope().await.engine_close_acknowledged);
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

#[tokio::test]
async fn public_bound_admission_captures_native_artifact_and_preserves_successor_and_reopen_identity() {
    use universal_agent_runtime::{llm::{ProviderRegistry, registry::ProviderConfig},
        uar::{api::model_verification, domain::collaboration::{PackageSourceRequest, BindingCommandRequest},
            settings::{manager::SettingsManager, schema::{SettingsType, Settings}}}};
    let user = external_identity_host::user("a2ui-recovery", false, "a2ui-owner").await;
    let directory = tempfile::tempdir().unwrap();
    let endpoint = format!("surrealkv://{}", directory.path().join("bound-admission.db").display());
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
    agent["id"] = json!("urn:uar:r026:native-artifact");
    agent["skills"] = json!([]);
    agent["input"] = json!({"type":"string"});
    agent["output"] = json!({"type":"string"});
    agent["sourceDescriptor"]["skills"] = json!([]);
    agent["legacySections"]["skills"] = json!("");
    finalize_fixture(&mut agent);
    let source = format!("{}\n", serde_json::to_string_pretty(&agent).unwrap());
    let reference = json!({"id":agent["id"],"version":agent["version"],"digest":agent["contentDigest"]});
    let mut manifest: Value = serde_json::from_str(include_str!("fixtures/collaboration/package-manifest.json")).unwrap();
    manifest["id"] = json!("urn:uar:r026:package");
    manifest["entrypoints"] = json!([reference.clone()]);
    manifest["files"] = json!([{"path":"agent-definition.json","kind":"AgentDefinition",
        "definition":reference,"byteDigest":fixture_digest(source.as_bytes())}]);
    manifest["lock"] = json!([]);
    finalize_fixture(&mut manifest);
    let owner = user.session_owner_key().unwrap();
    let installed = catalog.install_package(&owner, PackageSourceRequest {
        command_id: "r026-package".into(), expected_catalog_revision: None,
        manifest: serde_json::to_string_pretty(&manifest).unwrap(),
        files: [("agent-definition.json".into(), source)].into_iter().collect(),
    }).await.unwrap();
    let mut binding: Value = serde_json::from_str(include_str!("fixtures/collaboration/deployment-binding.json")).unwrap();
    binding["id"] = json!("r026-binding");
    binding["ownerId"] = json!(owner);
    binding["workspaceId"] = json!("workspace:r026");
    binding["package"] = serde_json::to_value(installed.preflight.package).unwrap();
    binding["skillBindings"] = json!([]);
    binding["representationGrantRefs"] = json!([]);
    binding["modelBindings"] = json!([{"requestedAlias":"local-default","providerId":"openai","modelId":"gpt-4o"}]);
    finalize_fixture(&mut binding);
    let installed = catalog.install_binding(&owner, "workspace:r026", BindingCommandRequest {
        command_id: "r026-binding".into(), expected_revision: Some(0), binding,
    }).await.unwrap();
    assert!(installed.preflight.activation_supported, "{:?}", installed.preflight);
    let target = universal_agent_runtime::uar::api::targets::agent_target_id("r026-binding");
    let resolved = catalog.resolve_bound_agent_run(&owner, "workspace:r026", "r026-binding").await.unwrap();
    assert_ne!(target, resolved.artifact.id);
    let driver = Arc::new(NoDispatch::default());
    *driver.3.lock().unwrap() = Some((store.clone(), owner.clone(), target.clone(),
        "bound-thread".into(), resolved.artifact.id.clone()));
    let manager = manager_with_sources(store.clone(), driver.clone(), Some((registry, settings))).await;
    let app = agui::build_router(SecurityConfig::default()).with_state(Arc::new(RunApiState {
        manager: manager.clone(), collaboration_catalog: catalog.clone(),
        service_instance: Arc::new(ServiceInstanceAuthority::new(&ServiceInstanceConfig::default(),
            "bound-identity", "http://127.0.0.1", false, &[])),
    })).layer(Extension(user.clone()));
    let mut original: Option<AguiRunBinding> = None;
    for (wire_id, parent) in [("wire-bound", None), ("wire-successor", Some("wire-bound"))] {
        let input = json!({"threadId":"bound-thread","runId":wire_id,"parentRunId":parent,
            "messages":[{"id":format!("message-{wire_id}"),"role":"user",
                "content":"R022 deliberate successor for native-bound"}]});
        let response = app.clone().oneshot(Request::post(format!("/agents/{target}"))
            .header("content-type", "application/json").header("x-uar-workspace-id", "workspace:r026")
            .body(Body::from(input.to_string())).unwrap()).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 128 * 1024).await.unwrap();
        let body = std::str::from_utf8(&body).unwrap();
        assert_eq!(terminals(body)[0]["outcome"]["type"], "success", "{body}");
        let row = store.load_agui_run(&owner, &target, "bound-thread", wire_id).await.unwrap().unwrap();
        assert_eq!(row.agent_id, target);
        assert_eq!(row.native_agent_id.as_deref(), Some(resolved.artifact.id.as_str()));
        let native = manager.get_run(&row.native_run_id).await.unwrap();
        assert_eq!(native.agent_id, resolved.artifact.id);
        assert_eq!(native.conversation_id.as_deref(), Some(row.session_id.as_str()));
        if let Some(original) = &original {
            assert_eq!(row.previous_wire_run_id.as_deref(), Some(original.wire_run_id.as_str()));
            assert_ne!(row.native_run_id, original.native_run_id);
            assert_eq!(row.session_id, original.session_id);
            assert_eq!(row.native_agent_id, original.native_agent_id);
            assert_eq!(retained(&store, original).await, *original);
        } else { original = Some(row.clone()); }
        assert_eq!(terminals(&reconnect(&app, &row, None).await.1), terminals(body));
    }
    let original = original.unwrap();
    // Reconstruct the admitted-but-unsealed crash-window fixture from the real
    // admission identity, not from today's target lookup. No crash is executed.
    let mut historical = original.clone();
    historical.thread_id = "bound-historical-window".into();
    historical.wire_run_id = "wire-unsealed".into();
    historical.terminal = None;
    historical.revision = 1;
    assert!(store.create_agui_run(&historical, None).await.unwrap());
    assert_eq!(driver.0.load(Ordering::SeqCst), 0);
    assert_eq!(driver.1.load(Ordering::SeqCst), 2);
    assert_eq!(driver.4.load(Ordering::SeqCst), 2, "both actual dispatches observed captured identity already durable");
    let _ = driver.3.lock().unwrap().take();
    drop(app);
    drop(catalog);
    manager.shutdown_graph_roots().await.unwrap();
    manager.shutdown_terminals().await.unwrap();
    manager.shutdown_sandboxes().await.unwrap();
    manager.shutdown_run_history().await.unwrap();
    drop(manager);
    assert!(store.close_scope().await.engine_close_acknowledged);
    drop(store);
    let store = Arc::new(SurrealDbProvider::new(&endpoint, None, None, None, None).await.unwrap());
    let manager = manager_with_sources(store.clone(), driver.clone(), None).await;
    let app = router(manager.clone(), &user);
    // New manager/catalog carries no current deployment definition or binding.
    let response = reconnect(&app, &historical, None).await;
    assert_eq!(response.0, StatusCode::OK);
    let sealed = retained(&store, &historical).await;
    assert_eq!(sealed.native_agent_id, original.native_agent_id);
    assert_eq!(sealed.agent_id, original.agent_id);
    assert_eq!(sealed.terminal.as_ref().unwrap().event["code"], "host_cleanup_unconfirmed");
    assert_eq!(terminals(&response.1), vec![sealed.terminal.as_ref().unwrap().event.clone()]);
    assert_eq!(driver.1.load(Ordering::SeqCst), 2, "recovery cannot dispatch the original or a successor");
    assert_eq!(driver.0.load(Ordering::SeqCst), 0);
    drop(app);
    manager.shutdown_graph_roots().await.unwrap();
    manager.shutdown_terminals().await.unwrap();
    manager.shutdown_sandboxes().await.unwrap();
    manager.shutdown_run_history().await.unwrap();
    drop(manager);
    assert!(store.close_scope().await.engine_close_acknowledged);
}

// Local, armed persistence fault: only the named conversation's terminal save
// fails. Required methods and the recovery capabilities used here forward to
// the real store. This is a synthetic save failure, not a store crash experiment.
mod terminal_save_fault {
    use super::*;
    use anyhow::Result;
    use universal_agent_runtime::{session::Session, uar::{
        a2ui::presentations::{Presentation, PresentationDraft},
        domain::{artifact::AgentArtifact, knowledge::{KnowledgeBase, KnowledgeChunk, KnowledgeDocument, KnowledgeMatch, DocumentStatus},
            policy::{ConversationPolicyRecord, RunPolicy, ResourceSelection},
            prompt_caching::UserPromptCachingSettings, skills::{Skill, SkillMatch, ScopedSkillConfig},
            memory::{Memory, MemoryMatch}},
        persistence::{accounting::*, DurableStoreEligibility, agent_threads::{PersistedAgentThread, CanonicalToolReceipt}, approval_decisions::ApprovalRecord},
        runtime::thread::{AgentThread, AgentEdge}, settings::schema::{Settings, SettingsType},
    }};

    #[derive(Debug)]
    pub struct Store {
        pub inner: Arc<SurrealDbProvider>,
        pub session: String,
        pub armed: std::sync::atomic::AtomicBool,
        pub failures: AtomicUsize,
        pub cleanup_fault: std::sync::atomic::AtomicBool,
        pub cleanup_failures: AtomicUsize,
        pub last_session: std::sync::Mutex<Option<Session>>,
    }
    #[async_trait::async_trait]
    impl PersistenceLayer for Store {
        fn durable_store_eligibility(&self) -> DurableStoreEligibility { self.inner.durable_store_eligibility() }
        fn supports_durable_accounting(&self) -> bool { self.inner.supports_durable_accounting() }
        async fn ensure_accounting_scope(&self, id: &ScopeIdentity, limits: &ScopeLimits) -> Result<AccountingScope> { self.inner.ensure_accounting_scope(id, limits).await }
        async fn get_accounting_scope(&self, id: &ScopeIdentity) -> Result<Option<AccountingScope>> { self.inner.get_accounting_scope(id).await }
        async fn update_accounting_scope(&self, change: &ScopeUpdate) -> Result<AccountingScope> { self.inner.update_accounting_scope(change).await }
        async fn admit_accounting_call(&self, admission: &CallAdmission) -> Result<AccountingCall> { self.inner.admit_accounting_call(admission).await }
        async fn record_accounting_usage(&self, usage: &CallUsage) -> Result<AccountingCall> { self.inner.record_accounting_usage(usage).await }
        async fn list_accounting_calls(&self, owner: &str, run: &str) -> Result<Vec<AccountingCall>> { self.inner.list_accounting_calls(owner, run).await }
        async fn bind_accounting_delegation(&self, link: &AccountingDelegation) -> Result<()> { self.inner.bind_accounting_delegation(link).await }
        async fn get_accounting_delegation(&self, digest: &str) -> Result<Option<AccountingCall>> { self.inner.get_accounting_delegation(digest).await }
        async fn admit_remote_grant(&self, owner: &str, id: &str, grant: &universal_agent_runtime::uar::api::a2a::contract::UarUsageGrant, activity: RemoteGrantActivity) -> Result<RemoteGrantState> { self.inner.admit_remote_grant(owner, id, grant, activity).await }
        async fn save_session(&self, session: &Session) -> Result<()> {
            if session.id() == self.session && self.armed.load(Ordering::SeqCst) {
                self.failures.fetch_add(1, Ordering::SeqCst);
                *self.last_session.lock().unwrap() = Some(session.clone());
                anyhow::bail!("R027 injected terminal save_session failure");
            }
            self.inner.save_session(session).await
        }
        async fn create_presentation(&self, owner: &str, draft: &PresentationDraft) -> Result<Presentation> { self.inner.create_presentation(owner, draft).await }
        async fn get_presentation(&self, owner: &str, id: &str) -> Result<Option<Presentation>> { self.inner.get_presentation(owner, id).await }
        async fn list_presentations(&self, owner: &str) -> Result<Vec<Presentation>> { self.inner.list_presentations(owner).await }
        async fn update_presentation(&self, owner: &str, id: &str, revision: u64, draft: &PresentationDraft) -> Result<Presentation> { self.inner.update_presentation(owner, id, revision, draft).await }
        async fn delete_presentation(&self, owner: &str, id: &str, revision: u64) -> Result<()> { self.inner.delete_presentation(owner, id, revision).await }
        async fn load_session(&self, owner: &str, id: &str) -> Result<Option<Session>> { self.inner.load_session(owner, id).await }
        async fn delete_session(&self, owner: &str, id: &str) -> Result<bool> { self.inner.delete_session(owner, id).await }
        async fn save_conversation_policy(&self, record: &ConversationPolicyRecord) -> Result<()> { self.inner.save_conversation_policy(record).await }
        async fn load_conversation_policy(&self, owner: &str, id: &str) -> Result<Option<ConversationPolicyRecord>> { self.inner.load_conversation_policy(owner, id).await }
        async fn save_principal_conversation_policy(&self, record: &ConversationPolicyRecord, expected: Option<&RunPolicy>) -> Result<bool> { self.inner.save_principal_conversation_policy(record, expected).await }
        async fn load_principal_conversation_policy(&self, owner: &str, id: &str) -> Result<Option<ConversationPolicyRecord>> { self.inner.load_principal_conversation_policy(owner, id).await }
        async fn delete_conversation_policy(&self, owner: &str, id: &str) -> Result<()> { self.inner.delete_conversation_policy(owner, id).await }
        async fn save_user_prompt_caching_settings(&self, settings: &UserPromptCachingSettings) -> Result<()> { self.inner.save_user_prompt_caching_settings(settings).await }
        async fn load_user_prompt_caching_settings(&self, owner: &str) -> Result<Option<UserPromptCachingSettings>> { self.inner.load_user_prompt_caching_settings(owner).await }
        async fn save_skill(&self, skill: &Skill, embedding: &[f32]) -> Result<()> { self.inner.save_skill(skill, embedding).await }
        async fn update_skill_selection(&self, id: &str, enabled: bool, config: &[ScopedSkillConfig]) -> Result<bool> { self.inner.update_skill_selection(id, enabled, config).await }
        async fn search_skills(&self, query: &[f32], limit: usize) -> Result<Vec<SkillMatch>> { self.inner.search_skills(query, limit).await }
        async fn save_knowledge_base(&self, kb: &KnowledgeBase) -> Result<()> { self.inner.save_knowledge_base(kb).await }
        async fn get_knowledge_base(&self, owner: &str, id: &str) -> Result<Option<KnowledgeBase>> { self.inner.get_knowledge_base(owner, id).await }
        async fn get_knowledge_base_by_name(&self, owner: &str, name: &str) -> Result<Option<KnowledgeBase>> { self.inner.get_knowledge_base_by_name(owner, name).await }
        async fn list_knowledge_bases(&self, owner: &str) -> Result<Vec<KnowledgeBase>> { self.inner.list_knowledge_bases(owner).await }
        async fn delete_knowledge_base(&self, owner: &str, id: &str) -> Result<()> { self.inner.delete_knowledge_base(owner, id).await }
        async fn save_chunk(&self, chunk: &KnowledgeChunk) -> Result<()> { self.inner.save_chunk(chunk).await }
        async fn search_knowledge(&self, owner: &str, query: &[f32], limit: usize, score: f32) -> Result<Vec<KnowledgeMatch>> { self.inner.search_knowledge(owner, query, limit, score).await }
        async fn search_knowledge_scoped(&self, owner: &str, ids: &[&str], query: &[f32], limit: usize, score: f32) -> Result<Vec<KnowledgeMatch>> { self.inner.search_knowledge_scoped(owner, ids, query, limit, score).await }
        async fn save_document(&self, doc: &KnowledgeDocument) -> Result<()> { self.inner.save_document(doc).await }
        async fn get_document(&self, owner: &str, id: &str) -> Result<Option<KnowledgeDocument>> { self.inner.get_document(owner, id).await }
        async fn list_documents(&self, owner: &str, id: &str) -> Result<Vec<KnowledgeDocument>> { self.inner.list_documents(owner, id).await }
        async fn update_document_status(&self, owner: &str, id: &str, status: &DocumentStatus) -> Result<()> { self.inner.update_document_status(owner, id, status).await }
        async fn delete_document(&self, owner: &str, id: &str) -> Result<()> { self.inner.delete_document(owner, id).await }
        async fn create_agent_root(&self, owner: &str, thread: &AgentThread) -> Result<PersistedAgentThread> { self.inner.create_agent_root(owner, thread).await }
        async fn create_agent_child(&self, owner: &str, thread: &AgentThread, edge: &AgentEdge) -> Result<PersistedAgentThread> { self.inner.create_agent_child(owner, thread, edge).await }
        async fn load_agent_thread(&self, owner: &str, id: &str) -> Result<Option<PersistedAgentThread>> { self.inner.load_agent_thread(owner, id).await }
        async fn update_agent_thread(&self, owner: &str, revision: u64, thread: &AgentThread) -> Result<PersistedAgentThread> { self.inner.update_agent_thread(owner, revision, thread).await }
        async fn list_agent_threads(&self, owner: &str, id: &str) -> Result<Vec<PersistedAgentThread>> { self.inner.list_agent_threads(owner, id).await }
        async fn list_agent_edges(&self, owner: &str, id: &str) -> Result<Vec<AgentEdge>> { self.inner.list_agent_edges(owner, id).await }
        async fn save_canonical_tool_receipt(&self, receipt: &CanonicalToolReceipt) -> Result<CanonicalToolReceipt> { self.inner.save_canonical_tool_receipt(receipt).await }
        async fn list_canonical_tool_receipts(&self, owner: &str, id: &str) -> Result<Vec<CanonicalToolReceipt>> { self.inner.list_canonical_tool_receipts(owner, id).await }
        async fn save_agent(&self, agent: &AgentArtifact) -> Result<()> { self.inner.save_agent(agent).await }
        async fn save_agent_if_unchanged(&self, expected: &AgentArtifact, updated: &AgentArtifact) -> Result<bool> { self.inner.save_agent_if_unchanged(expected, updated).await }
        async fn load_agent(&self, id: &str) -> Result<Option<AgentArtifact>> { self.inner.load_agent(id).await }
        async fn load_agent_by_name(&self, name: &str) -> Result<Option<AgentArtifact>> { self.inner.load_agent_by_name(name).await }
        async fn list_agents(&self) -> Result<Vec<AgentArtifact>> { self.inner.list_agents().await }
        async fn save_memory(&self, memory: &Memory) -> Result<()> { self.inner.save_memory(memory).await }
        async fn search_memory(&self, agent: Option<&str>, query: &[f32], limit: usize, score: f32) -> Result<Vec<MemoryMatch>> { self.inner.search_memory(agent, query, limit, score).await }
        async fn compare_and_swap_global_presentation_policy(&self, expected: &Value, selection: &ResourceSelection) -> Result<bool> { self.inner.compare_and_swap_global_presentation_policy(expected, selection).await }
        fn supports_durable_run_history(&self) -> bool { self.inner.supports_durable_run_history() }
        async fn upsert_run_history_record(&self, record: &RunHistoryRecord) -> Result<()> { self.inner.upsert_run_history_record(record).await }
        async fn load_run_history_record(&self, id: &str) -> Result<Option<RunHistoryRecord>> { self.inner.load_run_history_record(id).await }
        async fn list_conversation_run_history(&self, owner: &str, conversation: &str) -> Result<Vec<RunHistoryRecord>> { self.inner.list_conversation_run_history(owner, conversation).await }
        async fn append_run_history_event(&self, event: &RunHistoryEvent) -> Result<()> { self.inner.append_run_history_event(event).await }
        async fn load_run_history_events(&self, id: &str) -> Result<Vec<RunHistoryEvent>> { self.inner.load_run_history_events(id).await }
        fn supports_durable_agui_runs(&self) -> bool { self.inner.supports_durable_agui_runs() }
        async fn create_agui_run(&self, row: &AguiRunBinding, previous: Option<&AguiRunBinding>) -> Result<bool> { self.inner.create_agui_run(row, previous).await }
        async fn load_agui_run(&self, owner: &str, agent: &str, thread: &str, run: &str) -> Result<Option<AguiRunBinding>> { self.inner.load_agui_run(owner, agent, thread, run).await }
        async fn list_agui_runs(&self, owner: &str, agent: &str, thread: &str) -> Result<Vec<AguiRunBinding>> { self.inner.list_agui_runs(owner, agent, thread).await }
        async fn compare_and_swap_agui_run(&self, before: &AguiRunBinding, after: &AguiRunBinding) -> Result<bool> { self.inner.compare_and_swap_agui_run(before, after).await }
        fn supports_durable_approvals(&self) -> bool { self.inner.supports_durable_approvals() }
        async fn create_approval_record(&self, record: &ApprovalRecord) -> Result<()> { self.inner.create_approval_record(record).await }
        async fn transition_approval_record(&self, before: &ApprovalRecord, after: &ApprovalRecord) -> Result<bool> { self.inner.transition_approval_record(before, after).await }
        async fn list_approval_records(&self, owner: &str, run: &str) -> Result<Vec<ApprovalRecord>> {
            if self.cleanup_fault.load(Ordering::SeqCst) {
                self.cleanup_failures.fetch_add(1, Ordering::SeqCst);
                anyhow::bail!("R028 injected approval cleanup persistence failure");
            }
            self.inner.list_approval_records(owner, run).await
        }
        async fn get_setting(&self, key: &str) -> Result<Option<Settings>> { self.inner.get_setting(key).await }
        async fn get_settings_type(&self, key: &str) -> Result<Option<SettingsType>> { self.inner.get_settings_type(key).await }
        async fn list_settings(&self, kind: Option<&str>, parent: Option<uuid::Uuid>) -> Result<Vec<Settings>> { self.inner.list_settings(kind, parent).await }
    }
}

struct CompleteWithSaveFault(Arc<terminal_save_fault::Store>);
#[async_trait::async_trait]
impl LlmDriver for CompleteWithSaveFault {
    async fn stream(&self, _request: LlmRequest) -> anyhow::Result<Pin<Box<dyn futures::Stream<Item = anyhow::Result<universal_agent_runtime::normalized::NormalizedEvent>> + Send>>> {
        self.0.armed.store(true, Ordering::SeqCst);
        Ok(Box::pin(futures::stream::iter(vec![Ok(universal_agent_runtime::normalized::NormalizedEvent::Done)])))
    }
}

#[tokio::test]
async fn actual_cancelled_session_save_fault_is_nonterminal_on_public_native_and_standard_transports() {
    use universal_agent_runtime::uar::runtime::turn::RunExecutionRequest;
    let user = external_identity_host::user("a2ui-recovery", false, "a2ui-owner").await;
    let foreign = external_identity_host::user("a2ui-recovery-foreign", false, "a2ui-foreign").await;
    let directory = tempfile::tempdir().unwrap();
    let endpoint = format!("surrealkv://{}", directory.path().join("actual-save-fault.db").display());
    let store = Arc::new(SurrealDbProvider::new(&endpoint, None, None, None, None).await.unwrap());
    let mut historical_rows = Vec::new();
    for cancelled in [true, false] {
        let mut row = binding(&user, if cancelled { "actual-cancel-save-fault" } else { "actual-error-save-fault" });
        row.native_agent_id = Some("agent".into());
        let fault = Arc::new(terminal_save_fault::Store { inner: store.clone(), session: row.session_id.clone(),
            armed: std::sync::atomic::AtomicBool::new(false), failures: AtomicUsize::new(0),
            cleanup_fault: std::sync::atomic::AtomicBool::new(false), cleanup_failures: AtomicUsize::new(0),
            last_session: std::sync::Mutex::new(None) });
        let held = Arc::new(HeldCancellation::default());
        let driver: Arc<dyn LlmDriver> = if cancelled { held.clone() } else { Arc::new(CompleteWithSaveFault(fault.clone())) };
        let manager = manager(fault.clone(), driver).await;
        let mut artifact = universal_agent_runtime::uar::defaults::default_agent();
        artifact.id = "agent".into();
        artifact.policy.skills.max_active = 0;
        artifact.policy.tools.allow.clear();
        artifact.memory.kb.enabled = false;
        let mut request = RunExecutionRequest::new(artifact, "R027 actual terminal save fault".into())
            .with_user_context(&user).unwrap();
        request.session_id = Some(row.session_id.clone());
        row.native_run_id = manager.execute_request(request).await;
        let mut observed = manager.subscribe(&row.native_run_id).await.unwrap();
        if cancelled {
            tokio::time::timeout(std::time::Duration::from_secs(30), held.entered.notified()).await.unwrap();
            fault.armed.store(true, Ordering::SeqCst);
            assert!(manager.cancel_run_for_context(&user, &row.native_run_id).await);
        }
        tokio::time::timeout(std::time::Duration::from_secs(30), async {
            loop {
                if manager.history_since(&row.native_run_id, None).await.unwrap().iter().any(|event|
                    matches!(&event.event, NormalizedEvent::Cancelled { .. } | NormalizedEvent::RunDone { .. }
                        | NormalizedEvent::RunDoneWithUsage { .. })) { break; }
                observed.recv().await.unwrap();
            }
        }).await.unwrap();
        drop(observed);
        let status = manager.get_run(&row.native_run_id).await.unwrap().status;
        assert_eq!(status, if cancelled { RunStatus::Cancelled } else { RunStatus::Error });
        assert_eq!(fault.failures.load(Ordering::SeqCst), 1, "only the terminal save was faulted");
        let history = manager.history_since(&row.native_run_id, None).await.unwrap();
        let diagnostic = history.iter().find(|event| match &event.event {
            NormalizedEvent::PresentationDiagnostic { code, .. } | NormalizedEvent::Error { code, .. } => code == "session_persist_failed",
            _ => false,
        }).unwrap();
        if cancelled {
            assert!(matches!(&diagnostic.event, NormalizedEvent::PresentationDiagnostic { run_id, message, .. }
                if run_id == &row.native_run_id && message == "Conversation terminal could not be saved"));
            assert!(!history.iter().any(|event| matches!(&event.event, NormalizedEvent::Error { .. })));
        } else {
            assert!(matches!(&diagnostic.event, NormalizedEvent::Error { .. }));
        }
        // Actual public native route/projector over manager-produced history.
        let native_app = universal_agent_runtime::uar::api::routes::build_router().with_state(Arc::new(RunApiState {
            manager: manager.clone(), collaboration_catalog: Arc::new(CollaborationCatalogService::in_memory()),
            service_instance: Arc::new(ServiceInstanceAuthority::new(&ServiceInstanceConfig::default(),
                "r027-save-fault", "http://127.0.0.1", false, &[])),
        })).layer(Extension(user.clone()));
        let response = native_app.oneshot(Request::builder().uri(format!(
            "/runs/{}/stream?stream_mode=agui_spec&last_event_id=0", row.native_run_id))
            .body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = tokio::time::timeout(std::time::Duration::from_secs(30), to_bytes(response.into_body(), 128 * 1024)).await.unwrap().unwrap();
        let body = std::str::from_utf8(&bytes).unwrap();
        let frames: Vec<Value> = body.lines().filter_map(|line| line.strip_prefix("data: "))
            .map(|data| serde_json::from_str(data).unwrap()).collect();
        let native_terminals: Vec<_> = frames.iter().filter(|event| matches!(event["type"].as_str(), Some("RUN_ERROR" | "RUN_FINISHED"))).collect();
        assert_eq!(native_terminals.len(), 1, "{body}");
        assert_eq!(native_terminals[0]["type"], "RUN_ERROR");
        assert_eq!(native_terminals[0]["code"], if cancelled { "CANCELLED" } else { "session_persist_failed" });
        if cancelled {
            let custom_index = frames.iter().position(|event| event["name"] == "uar.presentation.diagnostic"
                && event["value"]["code"] == "session_persist_failed").unwrap();
            let terminal_index = frames.iter().position(|event| event["type"] == "RUN_ERROR").unwrap();
            assert!(custom_index < terminal_index);
            assert_eq!(frames[custom_index]["runId"], row.native_run_id);
            assert_eq!(frames[custom_index]["value"]["message"], "Conversation terminal could not be saved");
            assert_eq!(native_terminals[0]["result"]["usage"]["inputTokens"], 7);
            assert_eq!(native_terminals[0]["result"]["usage"]["outputTokens"], 3);
            assert_eq!(native_terminals[0]["result"]["usage"]["totalTokens"], 10);
            let final_event = history.iter().find(|event| matches!(&event.event, NormalizedEvent::Cancelled { .. })).unwrap();
            assert!(diagnostic.id < final_event.id);
            assert!(store.create_agui_run(&row, None).await.unwrap());
            let standard_app = router(manager.clone(), &user);
            let result = reconnect(&standard_app, &row, None).await;
            let sealed = retained(&store, &row).await;
            let terminal = sealed.terminal.as_ref().unwrap();
            assert_eq!(terminal.source_cursor, final_event.id);
            assert_eq!(terminal.event["outcome"]["type"], "cancelled");
            let expected: Vec<_> = history.iter().filter(|event| matches!(&event.event,
                NormalizedEvent::PresentationDiagnostic { .. } | NormalizedEvent::Cancelled { .. }))
                .map(|event| json!({"cursor":event.id,"event":event.event})).collect();
            assert_eq!(terminal.event["metadata"]["uar"]["nativeTerminalEvidence"], json!(expected));
            let projected: Vec<Value> = result.1.lines().filter_map(|line| line.strip_prefix("data: "))
                .map(|data| serde_json::from_str(data).unwrap()).collect();
            let original = projected.iter().find(|event| event["name"] == "uar.presentation.diagnostic"
                && event["value"]["code"] == "session_persist_failed").unwrap();
            assert_eq!(original["metadata"]["uar"]["nativeEvent"], serde_json::to_value(&diagnostic.event).unwrap());
            assert_eq!(original["metadata"]["uar"]["nativeCursor"], diagnostic.id);
            assert_eq!(original["metadata"]["uar"]["nativeRunId"], row.native_run_id);
            assert_eq!(reconnect(&router(manager.clone(), &foreign), &row, None).await.0, StatusCode::NOT_FOUND);
            // Reconstruct only the observer's unsealed binding window. Native
            // status/events above came from the real faulted manager, not seed().
            let mut historical = binding(&user, "actual-fault-unsealed-historical");
            historical.native_agent_id = row.native_agent_id.clone();
            historical.native_run_id = row.native_run_id.clone();
            historical.session_id = row.session_id.clone();
            assert!(store.create_agui_run(&historical, None).await.unwrap());
            historical_rows.push((historical, final_event.id, expected, final_event.event.clone()));
            drop(standard_app);
        }
        manager.shutdown_graph_roots().await.unwrap();
        manager.shutdown_terminals().await.unwrap();
        manager.shutdown_sandboxes().await.unwrap();
        manager.shutdown_run_history().await.unwrap();
        drop(manager);
        drop(fault);
    }
    assert!(store.close_scope().await.engine_close_acknowledged);
    drop(store);
    let store = Arc::new(SurrealDbProvider::new(&endpoint, None, None, None, None).await.unwrap());
    let driver = Arc::new(NoDispatch::default());
    let manager = manager(store.clone(), driver.clone()).await;
    let app = router(manager.clone(), &user);
    for (row, cursor, evidence, cancelled) in historical_rows {
        assert!(manager.get_run(&row.native_run_id).await.is_none());
        let result = reconnect(&app, &row, None).await;
        let sealed = retained(&store, &row).await;
        let terminal = sealed.terminal.as_ref().unwrap();
        assert_eq!(terminal.source_cursor, cursor);
        assert_eq!(terminal.event["code"], "host_cleanup_unconfirmed");
        assert_eq!(terminal.event["metadata"]["uar"]["nativeStatus"], "cancelled");
        assert_eq!(terminal.event["metadata"]["uar"]["nativeTerminal"], serde_json::to_value(cancelled).unwrap());
        assert_eq!(terminal.event["metadata"]["uar"]["nativeTerminalEvidence"], json!(evidence));
        assert_eq!(terminals(&result.1), vec![terminal.event.clone()]);
        assert_eq!(terminals(&reconnect(&app, &row, None).await.1), vec![terminal.event.clone()]);
        assert_eq!(retained(&store, &row).await, sealed);
    }
    assert_eq!(driver.0.load(Ordering::SeqCst), 0);
    assert_eq!(driver.1.load(Ordering::SeqCst), 0);
    drop(app);
    manager.shutdown_graph_roots().await.unwrap();
    manager.shutdown_terminals().await.unwrap();
    manager.shutdown_sandboxes().await.unwrap();
    manager.shutdown_run_history().await.unwrap();
    drop(manager);
    assert!(store.close_scope().await.engine_close_acknowledged);
}


// A graph-owned pending node establishes entry into the actual manager graph
// branch without substituting the ordinary LLM loop or reporting provider usage.
struct HeldGraphNode(Arc<tokio::sync::Notify>);
#[async_trait::async_trait]
impl universal_agent_runtime::uar::runtime::graph::GraphNode for HeldGraphNode {
    fn id(&self) -> &str { "held-graph" }
    async fn execute(&self, _state: universal_agent_runtime::uar::runtime::graph::GraphState,
        _context: &universal_agent_runtime::uar::runtime::graph::GraphContext)
        -> universal_agent_runtime::uar::runtime::graph::NodeResult {
        self.0.notify_one();
        futures::future::pending().await
    }
}

#[tokio::test]
async fn actual_graph_cancel_save_fault_preserves_cancelled_but_cleanup_fault_remains_error() {
    use universal_agent_runtime::{session::SessionMessageStatus,
        uar::runtime::{graph::AgentGraph, turn::RunExecutionRequest}};
    let user = external_identity_host::user("a2ui-recovery", false, "a2ui-owner").await;
    let directory = tempfile::tempdir().unwrap();
    let endpoint = format!("surrealkv://{}", directory.path().join("graph-save-fault.db").display());
    let store = Arc::new(SurrealDbProvider::new(&endpoint, None, None, None, None).await.unwrap());
    for cleanup_failure in [false, true] {
        let mut row = binding(&user, if cleanup_failure { "graph-cleanup-fault" } else { "graph-save-only" });
        row.agent_id = "orchestrator-agent".into();
        row.native_agent_id = Some(row.agent_id.clone());
        let fault = Arc::new(terminal_save_fault::Store { inner: store.clone(), session: row.session_id.clone(),
            armed: std::sync::atomic::AtomicBool::new(false), failures: AtomicUsize::new(0),
            cleanup_fault: std::sync::atomic::AtomicBool::new(false), cleanup_failures: AtomicUsize::new(0),
            last_session: std::sync::Mutex::new(None) });
        let driver = Arc::new(NoDispatch::default());
        let entered = Arc::new(tokio::sync::Notify::new());
        let base = Arc::try_unwrap(manager(fault.clone(), driver.clone()).await).ok().unwrap();
        let manager = Arc::new(base.with_agent_graph(AgentGraph::builder("held-graph")
            .add_node(HeldGraphNode(entered.clone())).build()));
        let mut artifact = universal_agent_runtime::uar::defaults::default_agent();
        artifact.id = row.agent_id.clone();
        artifact.policy.skills.max_active = 0;
        artifact.policy.tools.allow.clear();
        artifact.memory.kb.enabled = false;
        let mut request = RunExecutionRequest::new(artifact, "R028 actual graph cancellation".into())
            .with_user_context(&user).unwrap();
        request.session_id = Some(row.session_id.clone());
        row.native_run_id = manager.execute_request(request).await;
        let mut observed = manager.subscribe(&row.native_run_id).await.unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(30), entered.notified()).await.unwrap();
        fault.armed.store(true, Ordering::SeqCst);
        fault.cleanup_fault.store(cleanup_failure, Ordering::SeqCst);
        assert!(manager.cancel_run_for_context(&user, &row.native_run_id).await);
        tokio::time::timeout(std::time::Duration::from_secs(30), async {
            loop {
                if manager.history_since(&row.native_run_id, None).await.unwrap().iter().any(|event|
                    matches!(&event.event, NormalizedEvent::Cancelled { .. } | NormalizedEvent::RunDone { .. })) { break; }
                observed.recv().await.unwrap();
            }
        }).await.unwrap();
        drop(observed);
        assert_eq!(manager.get_run(&row.native_run_id).await.unwrap().status,
            if cleanup_failure { RunStatus::Error } else { RunStatus::Cancelled });
        assert_eq!(fault.failures.load(Ordering::SeqCst), 1);
        assert_eq!(fault.cleanup_failures.load(Ordering::SeqCst), usize::from(cleanup_failure));
        let session = fault.last_session.lock().unwrap().clone().unwrap();
        let messages: Vec<_> = session.history().into_iter()
            .filter(|message| message.run_id.as_deref() == Some(row.native_run_id.as_str())).collect();
        assert!(!messages.is_empty());
        assert!(messages.iter().all(|message| message.status == if cleanup_failure {
            SessionMessageStatus::Error
        } else { SessionMessageStatus::Cancelled }));
        let history = manager.history_since(&row.native_run_id, None).await.unwrap();
        let diagnostic = history.iter().find(|event| matches!(&event.event,
            NormalizedEvent::PresentationDiagnostic { run_id, code, message }
                if run_id == &row.native_run_id && code == "session_persist_failed"
                    && message == "Cancelled conversation could not be saved")).unwrap();
        assert_eq!(history.iter().filter(|event| matches!(&event.event,
            NormalizedEvent::Cancelled { .. })).count(), usize::from(!cleanup_failure));
        if cleanup_failure {
            assert!(history.iter().any(|event| matches!(&event.event,
                NormalizedEvent::Error { code, .. } if code == "approval_cleanup_unconfirmed")));
        } else {
            assert!(!history.iter().any(|event| matches!(&event.event, NormalizedEvent::Error { .. })));
            let cancelled = history.iter().find(|event| matches!(&event.event,
                NormalizedEvent::Cancelled { usage: None, .. })).unwrap();
            assert!(diagnostic.id < cancelled.id);
        }
        let app = universal_agent_runtime::uar::api::routes::build_router().with_state(Arc::new(RunApiState {
            manager: manager.clone(), collaboration_catalog: Arc::new(CollaborationCatalogService::in_memory()),
            service_instance: Arc::new(ServiceInstanceAuthority::new(&ServiceInstanceConfig::default(),
                "r028-graph-fault", "http://127.0.0.1", false, &[])),
        })).layer(Extension(user.clone()));
        let response = app.oneshot(Request::builder().uri(format!(
            "/runs/{}/stream?stream_mode=agui_spec&last_event_id=0", row.native_run_id))
            .body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = tokio::time::timeout(std::time::Duration::from_secs(30),
            to_bytes(response.into_body(), 128 * 1024)).await.unwrap().unwrap();
        let frames: Vec<Value> = std::str::from_utf8(&bytes).unwrap().lines()
            .filter_map(|line| line.strip_prefix("data: "))
            .map(|data| serde_json::from_str(data).unwrap()).collect();
        let terminals: Vec<_> = frames.iter().filter(|event|
            matches!(event["type"].as_str(), Some("RUN_ERROR" | "RUN_FINISHED"))).collect();
        assert_eq!(terminals.len(), 1);
        assert_eq!(terminals[0]["type"], "RUN_ERROR");
        assert_eq!(terminals[0]["code"], if cleanup_failure { "approval_cleanup_unconfirmed" } else { "CANCELLED" });
        if !cleanup_failure {
            let diagnostic_index = frames.iter().position(|event| event["name"] == "uar.presentation.diagnostic"
                && event["value"]["code"] == "session_persist_failed").unwrap();
            assert!(diagnostic_index < frames.iter().position(|event| event["type"] == "RUN_ERROR").unwrap());
            assert!(terminals[0]["result"].get("usage").is_none_or(Value::is_null));
            assert!(store.create_agui_run(&row, None).await.unwrap());
            let result = reconnect(&router(manager.clone(), &user), &row, None).await;
            assert_eq!(result.0, StatusCode::OK);
            let sealed = retained(&store, &row).await;
            let terminal = sealed.terminal.unwrap();
            assert_eq!(terminal.event["outcome"]["type"], "cancelled");
            assert_eq!(terminal.source_cursor, history.iter().find(|event|
                matches!(&event.event, NormalizedEvent::Cancelled { .. })).unwrap().id);
            assert!(terminal.event["metadata"]["uar"]["nativeTerminalEvidence"].as_array().unwrap().iter()
                .any(|event| event["cursor"] == diagnostic.id && event["event"]["code"] == "session_persist_failed"));
        }
        assert_eq!(driver.0.load(Ordering::SeqCst), 0);
        assert_eq!(driver.1.load(Ordering::SeqCst), 0);
        fault.cleanup_fault.store(false, Ordering::SeqCst);
        manager.shutdown_graph_roots().await.unwrap();
        manager.shutdown_terminals().await.unwrap();
        manager.shutdown_sandboxes().await.unwrap();
        manager.shutdown_run_history().await.unwrap();
    }
    assert!(store.close_scope().await.engine_close_acknowledged);
}
