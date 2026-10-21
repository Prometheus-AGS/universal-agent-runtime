//! Phase-only source seams: real host manager, SurrealKV and HTTP provider.
//! These supplement installed-image observations; they are not image evidence.
use axum::{Extension, Json, Router, response::IntoResponse, routing::post};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use universal_agent_runtime::{
    config::LlmConfig,
    mcp::{outcome::ToolOutcome, registry::McpRegistry},
    session::SessionStore,
    uar::{defaults::default_agent, domain::{events::NormalizedEvent, runs::RunStatus},
        persistence::{PersistenceLayer, providers::surreal::SurrealDbProvider},
        rag::embeddings::UnavailableEmbeddingBackend,
        runtime::{manager::RunManager, matching::{ClassifierConfig, VectorMatcher}, native_skill::{NativeSkill, NativeSkillRegistry}, skills::SkillRegistry, turn::RunExecutionRequest},
        security::{claims::UserContext, middleware::authenticate_bearer_context},
        tools::descriptor::ToolEffect},
};
#[cfg(feature = "in-memory-backend")]
use universal_agent_runtime::uar::persistence::providers::memory::InMemoryProvider;

#[derive(Default)]
struct Seen { requests: Mutex<Vec<Value>> }
async fn model(Extension(seen): Extension<Arc<Seen>>, Json(request): Json<Value>) -> impl IntoResponse {
    seen.requests.lock().unwrap().push(request.clone());
    let has_result = request["messages"].as_array().unwrap().iter().any(|row| row["role"] == "tool");
    let native = request["tools"].as_array().is_some_and(|tools| tools.iter().any(|tool| tool["function"]["name"] == "phase_native"));
    let delta = if native && !has_result {
        json!({"tool_calls":[{"index":0,"id":"phase-native-call","type":"function","function":{"name":"phase_native","arguments":"{}"}}]})
    } else { json!({"content":"phase-native-success"}) };
    let first = json!({"id":"phase","object":"chat.completion.chunk","created":0,"model":"gpt-4o-mini","choices":[{"index":0,"delta":delta,"finish_reason":null}]});
    let last = json!({"id":"phase","object":"chat.completion.chunk","created":0,"model":"gpt-4o-mini","choices":[{"index":0,"delta":{},"finish_reason":if native && !has_result {"tool_calls"} else {"stop"}}]});
    ([("content-type","text/event-stream")],format!("data: {first}\n\ndata: {last}\n\ndata: [DONE]\n\n"))
}

struct Peer { base: String, seen: Arc<Seen>, task: tokio::task::JoinHandle<()> }
impl Peer {
    async fn start() -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}/v1",listener.local_addr().unwrap());
        let seen = Arc::new(Seen::default());
        let app = Router::new().route("/v1/chat/completions",post(model)).layer(Extension(seen.clone()));
        let task = tokio::spawn(async move { axum::serve(listener,app).await.unwrap(); });
        Self {base,seen,task}
    }
    fn count(&self) -> usize { self.seen.requests.lock().unwrap().len() }
    async fn close(self) { self.task.abort(); assert!(self.task.await.unwrap_err().is_cancelled()); }
}

async fn user() -> UserContext {
    let config = serde_json::from_value(json!({"jwt_required":true,"jwt_secret":"phase-synthetic-signing-key",
        "jwt_issuer":"phase-fixture","jwt_audience":"phase-runtime","jwt_validate_nbf":true})).unwrap();
    assert!(authenticate_bearer_context(&config,"not-a-token").await.is_err());
    let claims = json!({"sub":"phase-native-owner","iss":"phase-fixture","aud":"phase-runtime","tenant_id":"phase-native",
        "roles":["user"],"uar_credential_kind":"issuer","exp":chrono::Utc::now().timestamp()+3600,"nbf":chrono::Utc::now().timestamp()-5});
    let token = jsonwebtoken::encode(&jsonwebtoken::Header::default(),&claims,
        &jsonwebtoken::EncodingKey::from_secret(b"phase-synthetic-signing-key")).unwrap();
    authenticate_bearer_context(&config,&token).await.unwrap()
}

async fn manager(peer: &Peer, store: Arc<dyn PersistenceLayer>, native: Arc<NativeSkillRegistry>, endpoint: Option<String>) -> RunManager {
    RunManager::with_classifier_config(LlmConfig {model:"openai/gpt-4o-mini".into(),base_url:Some(endpoint.unwrap_or_else(|| peer.base.clone())),
        api_key:Some("phase-synthetic-provider-key".into()),host_supplied_connection:true,max_retries:0,..LlmConfig::default()},
        Arc::new(McpRegistry::new_empty()),SessionStore::new(),Arc::new(RwLock::new(SkillRegistry::default())),
        Arc::new(VectorMatcher::new(Arc::new(UnavailableEmbeddingBackend::new(384,"phase fixture has no embeddings")),0.75)),
        Some(store),ClassifierConfig::default(),native).await
}

async fn settled(manager: &RunManager, id: &str) -> RunStatus {
    tokio::time::timeout(std::time::Duration::from_secs(60),async {
        loop {
            let run = manager.get_run(id).await.unwrap();
            if matches!(run.status,RunStatus::Done|RunStatus::Error|RunStatus::Cancelled) { return run.status; }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    }).await.unwrap()
}

async fn close(manager: &RunManager) {
    manager.shutdown_graph_roots().await.unwrap();
    manager.shutdown_terminals().await.unwrap();
    manager.shutdown_sandboxes().await.unwrap();
}

pub async fn binding_refusals() {
    assert!(cfg!(feature = "in-memory-backend"), "source unsupported-storage fixture requires in-memory-backend");
    assert!(universal_agent_runtime::llm::local_only::enabled(), "source binding fixture requires UAR_LOCAL_ONLY=1 for secret-bearing constructor refusal");
    let root = tempfile::tempdir().unwrap();
    let store = Arc::new(SurrealDbProvider::new(&format!("surrealkv://{}",root.path().join("store").display()),None,None,Some("phase111"),Some("binding")).await.unwrap());
    let peer = Peer::start().await;
    let identity = user().await;
    for case in ["unsupported-storage","missing-principal","invalid-budget","constructor","valid"] {
        let endpoint = (case == "constructor").then(|| "http://[phase-constructor-secret".to_owned());
        let case_store: Arc<dyn PersistenceLayer> = if case == "unsupported-storage" {
            #[cfg(feature = "in-memory-backend")]
            { Arc::new(InMemoryProvider::new()) }
            #[cfg(not(feature = "in-memory-backend"))]
            { panic!("source unsupported-storage fixture requires in-memory-backend") }
        } else { store.clone() };
        if case == "unsupported-storage" {
            assert!(!case_store.supports_durable_accounting());
            assert_eq!(case_store.durable_store_eligibility(),universal_agent_runtime::uar::persistence::DurableStoreEligibility::UnsupportedBackend);
        }
        let host = manager(&peer,case_store,Arc::new(NativeSkillRegistry::new()),endpoint).await;
        let mut artifact = default_agent();
        artifact.id = format!("phase111-{case}");
        artifact.memory.kb.enabled = false;
        artifact.memory.conversation.enabled = false;
        artifact.policy.tools.allow.clear();
        artifact.extensions.insert("budgets".into(),if case == "invalid-budget" {json!({"max_cost_per_session_usd":-1})} else {json!({"max_tokens_per_turn":1000000})});
        let request = RunExecutionRequest::new(artifact,case.into());
        let request = if case == "missing-principal" { request } else { request.with_user_context(&identity).unwrap() };
        let before = peer.count();
        let id = host.execute_request(request).await;
        assert_eq!(settled(&host,&id).await,if case == "valid" {RunStatus::Done} else {RunStatus::Error});
        let events = host.history_since(&id,None).await.unwrap();
        if case != "valid" {
            assert_eq!(peer.count(),before,"refused binding performed provider I/O: {case}");
            let message = events.iter().find_map(|event| match &event.event {
                NormalizedEvent::Error {code,message,..} if code == "orchestrator_start_failed" => Some(message),_ => None,
            }).expect("safe binding failure");
            assert!(message.contains("diagnostic reference "));
            let expected_stage = if matches!(case,"missing-principal"|"unsupported-storage") {"durable_accounting"} else if case == "invalid-budget" {"budget_admission"} else {"client_construction"};
            assert!(message.contains(expected_stage),"wrong safe binding stage: {message}");
            assert!(events.iter().all(|event| matches!(
                &event.event,
                NormalizedEvent::Error { .. }
                    | NormalizedEvent::RunDone { .. }
                    | NormalizedEvent::RunDoneWithUsage { .. }
                    | NormalizedEvent::Cancelled { .. }
            )), "binding refusal published nonterminal admission content: {case}");
            let serialized = serde_json::to_string(&events.iter().map(|event| &event.event).collect::<Vec<_>>()).unwrap();
            assert!(!serialized.contains("phase-constructor-secret") && !serialized.contains("phase-synthetic-provider-key"));
        } else {
            assert_eq!(peer.count(),before+1);
        }
        close(&host).await;
    }
    peer.close().await;
}

struct NativePayload;
#[async_trait::async_trait]
impl NativeSkill for NativePayload {
    fn name(&self) -> &str {"phase_native"}
    fn description(&self) -> &str {"Returns fixture data through the actual native host path"}
    fn parameters_schema(&self) -> Value {json!({"type":"object","properties":{},"additionalProperties":false})}
    fn effect(&self) -> ToolEffect {ToolEffect::ReadOnly}
    async fn execute(&self,_args:Value) -> anyhow::Result<Value> {Ok(json!({"isError":true,"native_payload":42}))}
}

pub async fn native_incidental_error() {
    let root = tempfile::tempdir().unwrap();
    let store = Arc::new(SurrealDbProvider::new(&format!("surrealkv://{}",root.path().join("store").display()),None,None,Some("phase111"),Some("native")).await.unwrap());
    let peer = Peer::start().await;
    let native = Arc::new(NativeSkillRegistry::new());
    native.register(NativePayload).await.unwrap();
    let host = manager(&peer,store.clone(),native,None).await;
    let identity = user().await;
    let mut artifact = default_agent();
    artifact.id = "phase-native-control".into();
    artifact.policy.tools.allow = vec!["phase_native".into()];
    artifact.memory.kb.enabled = false;
    artifact.memory.conversation.enabled = false;
    let request = RunExecutionRequest::new(artifact,"native payload control".into()).with_user_context(&identity).unwrap();
    let id = host.execute_request(request).await;
    assert_eq!(settled(&host,&id).await,RunStatus::Done);
    assert_eq!(peer.count(),2,"native model tool loop must execute once and finish");
    let events = host.history_since(&id,None).await.unwrap();
    assert!(events.iter().any(|event| matches!(&event.event,NormalizedEvent::ToolEnd{output,ok:true,..} if output["isError"] == true)));
    let receipts = store.list_canonical_tool_receipts(&identity.user_id,&id).await.unwrap();
    assert_eq!(receipts.len(),1);
    assert_eq!(receipts[0].outcome,Some(ToolOutcome::Succeeded));
    assert_eq!(receipts[0].typed_value.as_ref().unwrap()["isError"],true);
    close(&host).await;
    peer.close().await;
}
