//! Dot-05 real HTTP/TLS transports, orchestrators, graph host, and durable stores.
//! This is controlled protocol qualification, not genuine provider inference.
#![cfg(all(feature = "server", feature = "surreal-backend"))]

#[path = "support/stream_terminal_peer.rs"]
mod peer;

use axum::{
    Extension, Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use futures::StreamExt;
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::sync::RwLock;
use tower::ServiceExt;
use universal_agent_runtime::{
    config::{LlmConfig, ServiceInstanceConfig},
    llm::{LiterLlmDriver, LlmDriver, Orchestrator, anthropic_driver::AnthropicDriver},
    mcp::registry::McpRegistry,
    normalized::NormalizedEvent,
    session::SessionStore,
    uar::{
        api::routes::{self, RunApiState},
        compiler::collaboration::CollaborationCatalogService,
        defaults::default_agent,
        domain::{events::NormalizedEvent as RunEvent, runs::RunStatus},
        persistence::{PersistenceLayer, providers::surreal::SurrealDbProvider},
        rag::embeddings::UnavailableEmbeddingBackend,
        runtime::{
            graph::{AgentGraph, CheckpointNode, LlmNode},
            manager::RunManager,
            matching::{VectorMatcher, intent::ClassifierConfig},
            native_skill::{NativeSkill, NativeSkillRegistry},
            skills::SkillRegistry,
        },
        security::claims::{CredentialPrincipal, UserClaims, UserContext},
        service_instance::ServiceInstanceAuthority,
        tools::descriptor::ToolEffect,
    },
};

const OWNER: &str = "dot05-stream-owner";
const WAIT: Duration = Duration::from_secs(60);
const HOSTS: [&str; 4] = [
    "generativelanguage.googleapis.com",
    "api.cohere.com",
    "us-central1-aiplatform.googleapis.com",
    "bedrock-runtime.us-east-1.amazonaws.com",
];

struct Echo(Arc<AtomicUsize>);
#[async_trait::async_trait]
impl NativeSkill for Echo {
    fn name(&self) -> &str {
        peer::TOOL
    }
    fn description(&self) -> &str {
        "Returns exactly the supplied text"
    }
    fn parameters_schema(&self) -> Value {
        json!({"type":"object","properties":{"text":{"type":"string"}},"required":["text"],"additionalProperties":false})
    }
    fn effect(&self) -> ToolEffect {
        ToolEffect::ReadOnly
    }
    fn check_thread_policy(
        &self,
        _policy: &universal_agent_runtime::uar::runtime::thread::policy_intersection::ThreadPolicy,
    ) -> anyhow::Result<()> {
        Ok(())
    }
    async fn execute(&self, args: Value) -> anyhow::Result<Value> {
        assert_eq!(args, json!({"text":"λ🙂"}));
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(args)
    }
}

async fn user() -> UserContext {
    let secret = "dot05-signed-principal-fixture";
    let config = universal_agent_runtime::config::SecurityConfig {
        deployment_profile: Default::default(), jwt_algorithm: None,
        workspace_authorities: Vec::new(),
        api_key_delegable_roles: universal_agent_runtime::config::SecurityConfig::default_api_key_delegable_roles(),
        api_key_admin_principals: Vec::new(), trusted_host_principals: Vec::new(),
        jwt_required: true, jwt_secret: secret.to_owned().into(), jwks_url: None,
        jwt_issuer: None, jwt_audience: None, jwt_validate_nbf: true,
        settings_mutation_auth_required: true, settings_admin_key: None,
    };
    // The production verifier owns process-provider initialization. Raw fixture
    // signing must follow it, while malformed credentials remain unauthorized.
    assert!(matches!(universal_agent_runtime::uar::security::middleware::authenticate_bearer_context(
        &config, "dot05-malformed-synthetic-token").await, Err(status) if status == StatusCode::UNAUTHORIZED),
        "production verifier must reject malformed fixture credentials before signing");
    let token = jsonwebtoken::encode(&jsonwebtoken::Header::default(), &UserClaims {
        sub: OWNER.into(), name: None, roles: Some(vec!["user".into()]),
        tenant_id: None, uar_instance_id: None,
        exp: (chrono::Utc::now().timestamp() + 3600) as usize,
    }, &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes())).unwrap();
    universal_agent_runtime::uar::security::middleware::authenticate_bearer_context(&config, &token)
        .await.expect("actual signed fixture principal verification")
}

fn openssl(root: &Path, args: &[&str]) {
    let output = Command::new("openssl")
        .args(args)
        .current_dir(root)
        .env_clear()
        .env("PATH", "/usr/bin:/bin:/usr/local/bin")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "synthetic certificate generation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
fn certificates(root: &Path) {
    openssl(
        root,
        &[
            "req",
            "-x509",
            "-newkey",
            "rsa:2048",
            "-nodes",
            "-keyout",
            "ca.key",
            "-out",
            "ca.pem",
            "-days",
            "1",
            "-subj",
            "/CN=Dot05 Synthetic Fixture CA",
            "-addext",
            "basicConstraints=critical,CA:TRUE",
            "-addext",
            "keyUsage=critical,keyCertSign,cRLSign",
        ],
    );
    openssl(
        root,
        &[
            "req",
            "-newkey",
            "rsa:2048",
            "-nodes",
            "-keyout",
            "server.key",
            "-out",
            "server.csr",
            "-subj",
            "/CN=Dot05 Synthetic Fixture Server",
        ],
    );
    std::fs::write(root.join("extensions.cnf"), format!("basicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\nsubjectAltName={}\n", HOSTS.iter().map(|host| format!("DNS:{host}")).collect::<Vec<_>>().join(","))).unwrap();
    openssl(
        root,
        &[
            "x509",
            "-req",
            "-in",
            "server.csr",
            "-CA",
            "ca.pem",
            "-CAkey",
            "ca.key",
            "-CAcreateserial",
            "-out",
            "server.pem",
            "-days",
            "1",
            "-extfile",
            "extensions.cnf",
        ],
    );
}

struct HttpsPeer {
    child: Child,
    root: PathBuf,
    address: std::net::SocketAddr,
}
impl HttpsPeer {
    async fn start(
        root: &Path,
        certs: &Path,
        protocol: peer::Protocol,
        case: &'static str,
    ) -> Self {
        std::fs::create_dir_all(root).unwrap();
        for file in ["ca.pem", "server.pem", "server.key"] {
            std::fs::copy(certs.join(file), root.join(file)).unwrap();
        }
        std::fs::write(root.join("first.bin"), peer::wire(protocol, case, false)).unwrap();
        std::fs::write(
            root.join("content-type"),
            if matches!(protocol, peer::Protocol::Bedrock) {
                "bedrock"
            } else {
                "sse"
            },
        )
        .unwrap();
        std::fs::write(root.join("second.bin"), peer::wire(protocol, case, true)).unwrap();
        let child = Command::new("/usr/bin/python3")
            .arg(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/fixtures/stream_terminal_https.py"),
            )
            .arg(root)
            .arg(case)
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .stdout(Stdio::null())
            .stderr(Stdio::from(
                std::fs::File::create(root.join("peer.stderr")).unwrap(),
            ))
            .spawn()
            .unwrap();
        tokio::time::timeout(Duration::from_secs(10), async {
            while !root.join("port").exists() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("controlled HTTPS peer ready");
        let port = std::fs::read_to_string(root.join("port"))
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        Self {
            child,
            root: root.to_path_buf(),
            address: std::net::SocketAddr::from(([127, 0, 0, 1], port)),
        }
    }
    fn client(&self) -> reqwest_mcp::Client {
        liter_llm::ensure_crypto_provider();
        // Concrete trusted-host test transport: TLS verification stays enabled.
        let certificate =
            reqwest_mcp::Certificate::from_pem(&std::fs::read(self.root.join("ca.pem")).unwrap())
                .unwrap();
        let mut builder = reqwest_mcp::Client::builder()
            .no_proxy()
            .redirect(reqwest_mcp::redirect::Policy::none())
            .add_root_certificate(certificate)
            .timeout(WAIT);
        for host in HOSTS {
            builder = builder.resolve(host, self.address);
        }
        builder.build().unwrap()
    }
    fn requests(&self) -> Vec<Value> {
        std::fs::read_to_string(self.root.join("requests.jsonl"))
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
}
impl Drop for HttpsPeer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

enum Endpoint {
    Http(peer::Peer),
    Https(HttpsPeer),
}
impl Endpoint {
    async fn start(
        root: &Path,
        certs: &Path,
        protocol: peer::Protocol,
        case: &'static str,
    ) -> Self {
        std::fs::create_dir_all(root).unwrap();
        std::fs::write(root.join("first.bin"), peer::wire(protocol, case, false)).unwrap();
        std::fs::write(root.join("second.bin"), peer::wire(protocol, case, true)).unwrap();
        if matches!(
            protocol,
            peer::Protocol::GoogleAi
                | peer::Protocol::Vertex
                | peer::Protocol::Cohere
                | peer::Protocol::Bedrock
        ) {
            Self::Https(HttpsPeer::start(root, certs, protocol, case).await)
        } else {
            Self::Http(peer::Peer::start(root, protocol, case).await)
        }
    }
    fn driver(&self, protocol: peer::Protocol) -> Arc<dyn LlmDriver> {
        match self {
            Self::Http(peer) if matches!(protocol, peer::Protocol::NativeAnthropic) => {
                Arc::new(AnthropicDriver::new(
                    "dot05-synthetic-key".into(),
                    protocol.model().split_once('/').unwrap().1.into(),
                    Some(peer.url.clone()),
                    None,
                    None,
                    None,
                ))
            }
            Self::Http(peer) => {
                let mut config = liter_llm::ClientConfig::new("dot05-synthetic-key");
                config.base_url = Some(format!("{}/v1", peer.url));
                config.load_env = false;
                config.max_retries = 0;
                config.transport = config.transport.with_proxies_disabled(true);
                Arc::new(LiterLlmDriver::new(config, protocol.model().into(), None).unwrap())
            }
            Self::Https(peer) => {
                let mut config = liter_llm::ClientConfig::new("dot05-synthetic-key");
                config.load_env = false;
                config.max_retries = 0;
                config.bedrock_region = Some("us-east-1".into());
                config.bedrock_access_key_id = Some("dot05-synthetic-access".into());
                config.bedrock_secret_access_key = Some("dot05-synthetic-secret".into());
                config.bedrock_session_token = Some("dot05-synthetic-session".into());
                let client = liter_llm::DefaultClient::with_http_client(
                    config,
                    Some(protocol.model()),
                    peer.client(),
                )
                .unwrap();
                Arc::new(LiterLlmDriver::from_client(
                    Arc::new(client),
                    protocol.model().into(),
                    None,
                    None,
                ))
            }
        }
    }
    fn assert_selected_wire(&self, protocol: peer::Protocol) {
        if let Self::Http(peer) = self {
            for trace in peer.requests.lock().unwrap().iter() {
                let path = trace["path"].as_str().unwrap();
                match protocol {
                    peer::Protocol::Azure => {
                        assert!(path.contains("/openai/deployments/gpt-4o-mini/chat/completions"))
                    }
                    peer::Protocol::OpenAi => assert_eq!(path, "/v1/chat/completions"),
                    peer::Protocol::LiterAnthropic | peer::Protocol::NativeAnthropic => {
                        assert_eq!(path, "/v1/messages")
                    }
                    _ => unreachable!("fixed HTTPS protocols use the controlled TLS endpoint"),
                }
                assert!(trace["request"].get("messages").is_some());
            }
        }
        if let Self::Https(peer) = self {
            for trace in peer.requests() {
                let path = trace["path"].as_str().unwrap();
                let body = &trace["request"];
                match protocol {
                    peer::Protocol::GoogleAi => {
                        assert!(path.starts_with("/v1beta/models/"));
                        assert!(path.ends_with(":streamGenerateContent?alt=sse"));
                        assert!(body.get("contents").is_some());
                    }
                    peer::Protocol::Vertex => {
                        assert!(
                            path.starts_with("/v1/projects/dot05-fixture/locations/us-central1/")
                        );
                        assert!(path.ends_with(":streamGenerateContent?alt=sse"));
                        assert!(body.get("contents").is_some());
                    }
                    peer::Protocol::Cohere => {
                        assert_eq!(path, "/v2/chat");
                        assert!(body.get("messages").is_some());
                    }
                    peer::Protocol::Bedrock => {
                        assert!(path.starts_with("/model/"));
                        assert!(path.ends_with("/converse-stream"));
                        assert!(body.get("messages").is_some());
                    }
                    _ => unreachable!("HTTP protocols do not use this fixture"),
                }
            }
        }
    }
    fn count(&self) -> usize {
        match self {
            Self::Http(peer) => peer.requests.lock().unwrap().len(),
            Self::Https(peer) => peer.requests().len(),
        }
    }
}

fn config(protocol: peer::Protocol) -> LlmConfig {
    LlmConfig {
        model: protocol.model().into(),
        api_key: Some("dot05-synthetic-key".into()),
        host_supplied_connection: true,
        ..LlmConfig::default()
    }
}
fn succeeds(case: &str) -> bool {
    matches!(case, "valid" | "split" | "tool")
}

async fn legacy(root: &Path, certs: &Path, protocol: peer::Protocol, case: &'static str) {
    let endpoint = Endpoint::start(root, certs, protocol, case).await;
    let calls = Arc::new(AtomicUsize::new(0));
    let skills = Arc::new(NativeSkillRegistry::new());
    skills.register(Echo(calls.clone())).await.unwrap();
    let orchestrator = Orchestrator::from_driver(
        config(protocol),
        Arc::new(McpRegistry::new_empty()),
        skills,
        endpoint.driver(protocol),
    )
    .with_resilience_policy(
        universal_agent_runtime::uar::settings::resilience_policy::ResiliencePolicy {
            retries_enabled: false,
            ..Default::default()
        },
    );
    let stream = orchestrator.chat("protocol fixture").await.unwrap();
    futures::pin_mut!(stream);
    let mut events = Vec::new();
    tokio::time::timeout(WAIT, async {
        while let Some(event) = stream.next().await {
            let stop = case == "cancel" && matches!(&event, NormalizedEvent::MessageDelta { .. });
            events.push(event);
            if stop {
                break;
            }
        }
    })
    .await
    .unwrap();
    let done = events
        .iter()
        .filter(|event| matches!(event, NormalizedEvent::Done))
        .count();
    let failed = events
        .iter()
        .any(|event| matches!(event, NormalizedEvent::Error { .. }));
    std::fs::write(
        root.join("boundary-diagnostics.json"),
        serde_json::to_vec_pretty(&json!({
            "route": "legacy_orchestrator",
            "protocol": format!("{protocol:?}"),
            "case": case,
            "events": &events,
            "terminal_state": {"done_count": done, "failed": failed, "cancelled_by_fixture": case == "cancel"},
            "persisted_messages": null,
            "persistence_scope": "legacy orchestrator boundary has no persistence collaborator",
            "tool_effect_count": calls.load(Ordering::SeqCst),
            "provider_request_count": endpoint.count(),
        })).unwrap(),
    ).unwrap();
    assert_eq!(
        done,
        usize::from(succeeds(case)),
        "legacy {protocol:?}/{case}"
    );
    assert_eq!(
        failed,
        !succeeds(case) && case != "cancel",
        "legacy {protocol:?}/{case}"
    );
    assert_eq!(calls.load(Ordering::SeqCst), usize::from(case == "tool"));
    assert_eq!(
        endpoint.count(),
        if case == "tool" { 2 } else { 1 },
        "no retry after semantic stream data"
    );
    endpoint.assert_selected_wire(protocol);
    if matches!(protocol, peer::Protocol::NativeAnthropic) && matches!(case, "valid" | "split") {
        let usage = events
            .iter()
            .filter_map(|event| match event {
                NormalizedEvent::Usage {
                    prompt_tokens,
                    completion_tokens,
                    total_tokens,
                    cached_tokens,
                    cache_creation_tokens,
                } => Some((
                    *prompt_tokens,
                    *completion_tokens,
                    *total_tokens,
                    *cached_tokens,
                    *cache_creation_tokens,
                )),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            usage,
            vec![(2, 3, 5, None, None)],
            "native output-only delta must retain message_start input/cache usage"
        );
    }
    if succeeds(case) {
        let text = events
            .iter()
            .filter_map(|event| {
                if let NormalizedEvent::MessageDelta { text } = event {
                    Some(text.as_str())
                } else {
                    None
                }
            })
            .collect::<String>();
        assert_eq!(text, peer::TEXT);
    }
}

async fn graph(
    root: &Path,
    certs: &Path,
    store: Arc<dyn PersistenceLayer>,
    protocol: peer::Protocol,
    case: &'static str,
) {
    let endpoint = Endpoint::start(root, certs, protocol, case).await;
    let calls = Arc::new(AtomicUsize::new(0));
    let skills = Arc::new(NativeSkillRegistry::new());
    skills.register(Echo(calls.clone())).await.unwrap();
    let manager = Arc::new(
        RunManager::with_classifier_config(
            config(protocol),
            Arc::new(McpRegistry::new_empty()),
            SessionStore::new(),
            Arc::new(RwLock::new(SkillRegistry::default())),
            Arc::new(VectorMatcher::new(
                Arc::new(UnavailableEmbeddingBackend::new(
                    384,
                    "stream fixture does not use embeddings",
                )),
                0.75,
            )),
            Some(store.clone()),
            ClassifierConfig::default(),
            skills,
        )
        .await
        .with_resilience_policy(
            universal_agent_runtime::uar::settings::resilience_policy::ResiliencePolicy {
                retries_enabled: false,
                ..Default::default()
            },
        )
        .with_llm_driver(endpoint.driver(protocol))
        .with_agent_graph(
            AgentGraph::builder("admission")
                .add_node(CheckpointNode::new("admission", "before-provider"))
                .add_node(LlmNode::new("stream"))
                .add_node(CheckpointNode::new("terminal", "after-provider"))
                .add_edge("admission", "stream")
                .add_edge("stream", "terminal")
                .build(),
        ),
    );
    let state = Arc::new(RunApiState {
        manager: manager.clone(),
        collaboration_catalog: Arc::new(CollaborationCatalogService::in_memory()),
        service_instance: Arc::new(ServiceInstanceAuthority::new(
            &ServiceInstanceConfig::default(),
            "dot05-instance",
            "http://127.0.0.1",
            false,
            &[],
        )),
    });
    let verified_user = user().await;
    let session_owner = CredentialPrincipal::from_verified_context(&verified_user)
        .expect("verified graph session principal")
        .storage_key();
    let app = Router::new()
        .nest("/api/uar", routes::build_router().with_state(state))
        .layer(Extension(verified_user));
    let mut artifact = default_agent();
    artifact.id = "orchestrator-agent".into();
    artifact.memory.kb.enabled = false;
    let (provider, model) = protocol.model().split_once('/').unwrap();
    artifact.policy.provider.default.provider = provider.into();
    artifact.policy.provider.default.model = model.into();
    artifact.policy.tools.allow = vec![peer::TOOL.into()];
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/uar/runs")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"artifact":artifact,"input":"protocol fixture"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let admitted: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 1024 * 1024).await.unwrap())
            .unwrap();
    let run = admitted["run_id"].as_str().unwrap();
    tokio::time::timeout(WAIT, async {
        let mut cancelled = false;
        loop {
            let events = manager.history_since(run, None).await.unwrap();
            if case == "cancel"
                && !cancelled
                && events
                    .iter()
                    .any(|event| matches!(&event.event, RunEvent::ChatDelta { .. }))
            {
                assert!(manager.cancel_run(run).await);
                cancelled = true;
            }
            if events.iter().any(|event| {
                matches!(
                    &event.event,
                    RunEvent::RunDone { .. }
                        | RunEvent::RunDoneWithUsage { .. }
                        | RunEvent::Error { .. }
                        | RunEvent::Cancelled { .. }
                )
            }) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("production graph terminal outcome");
    let events = manager.history_since(run, None).await.unwrap();
    let done = events.iter().any(|event| {
        matches!(
            &event.event,
            RunEvent::RunDone { .. } | RunEvent::RunDoneWithUsage { .. }
        )
    });
    let failed = events
        .iter()
        .any(|event| matches!(&event.event, RunEvent::Error { .. }));
    let cancelled = events
        .iter()
        .any(|event| matches!(&event.event, RunEvent::Cancelled { .. }));
    // Wait for the host's final state write after terminal event publication.
    tokio::time::timeout(WAIT, async {
        loop {
            let state = manager.get_run(run).await.unwrap();
            if matches!(
                state.status,
                RunStatus::Done | RunStatus::Error | RunStatus::Cancelled
            ) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    let state = manager.get_run(run).await.unwrap();
    let checkpoints = store.list_checkpoints(run).await.unwrap();
    let session = store
        .load_session(
            &session_owner,
            state.conversation_id.as_deref().expect("real session"),
        )
        .await
        .unwrap()
        .expect("graph history persisted");
    let persisted_messages = session.messages();
    let history = serde_json::to_string(&persisted_messages).unwrap();
    let partial_text_exposed = events.iter().any(|event| {
        matches!(&event.event, RunEvent::ChatDelta { text_delta, .. } if !text_delta.is_empty())
    });
    std::fs::write(
        root.join("boundary-diagnostics.json"),
        serde_json::to_vec_pretty(&json!({
            "route": "graph",
            "protocol": format!("{protocol:?}"),
            "case": case,
            "events": events.iter().map(|event| json!({"id":event.id, "event": &event.event})).collect::<Vec<_>>(),
            "terminal_state": &state,
            "persisted_messages": &persisted_messages,
            "checkpoints": &checkpoints,
            "partial_text_exposed": partial_text_exposed,
            "tool_effect_count": calls.load(Ordering::SeqCst),
            "provider_request_count": endpoint.count(),
        })).unwrap(),
    ).unwrap();
    if succeeds(case) {
        assert!(done, "graph {protocol:?}/{case}");
    }
    if case == "cancel" {
        assert!(!done);
    }
    assert_eq!(
        failed,
        !succeeds(case) && case != "cancel",
        "graph {protocol:?}/{case}"
    );
    assert_eq!(cancelled, case == "cancel");
    assert_eq!(calls.load(Ordering::SeqCst), usize::from(case == "tool"));
    assert_eq!(endpoint.count(), if case == "tool" { 2 } else { 1 });
    endpoint.assert_selected_wire(protocol);
    assert_eq!(
        state.status,
        if succeeds(case) {
            RunStatus::Done
        } else if case == "cancel" {
            RunStatus::Cancelled
        } else {
            RunStatus::Error
        }
    );
    assert!(
        checkpoints
            .iter()
            .any(|checkpoint| checkpoint.node_id == "admission"),
        "real graph admission path absent"
    );
    assert_eq!(
        checkpoints
            .iter()
            .any(|checkpoint| checkpoint.node_id == "terminal"),
        succeeds(case),
        "unfinished provider advanced graph success edge"
    );

    if succeeds(case) {
        assert!(history.contains(peer::TEXT));
    } else if partial_text_exposed {
        assert!(
            history.contains("TurnInterrupted"),
            "partial stream lost interruption marker"
        );
    }
    if case == "tool" {
        let receipts = store
            .list_canonical_tool_receipts(OWNER, run)
            .await
            .unwrap();
        assert_eq!(receipts.len(), 1);
        assert_eq!(
            receipts[0].outcome,
            Some(universal_agent_runtime::mcp::outcome::ToolOutcome::Succeeded)
        );
    } else {
        assert!(
            store
                .list_canonical_tool_receipts(OWNER, run)
                .await
                .unwrap()
                .is_empty(),
            "unfinished stream executed tools"
        );
    }
    manager.shutdown_graph_roots().await.unwrap();
}

async fn suite(root: &Path) {
    assert_eq!(
        std::env::var("VERTEXAI_PROJECT").as_deref(),
        Ok("dot05-fixture")
    );
    assert_eq!(
        std::env::var("VERTEXAI_LOCATION").as_deref(),
        Ok("us-central1")
    );
    assert!(
        std::env::var_os("BEDROCK_BASE_URL").is_none(),
        "controlled native Bedrock route cannot inherit an endpoint override"
    );
    assert!(
        std::env::var_os("BEDROCK_CROSS_REGION").is_none(),
        "controlled native Bedrock route cannot inherit a model prefix"
    );
    let certs = root.join("certs");
    std::fs::create_dir_all(&certs).unwrap();
    certificates(&certs);
    let local: Arc<dyn PersistenceLayer> = Arc::new(
        SurrealDbProvider::new(
            &format!("surrealkv://{}", root.join("local-db").display()),
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap(),
    );
    let infrastructure: Value = serde_json::from_slice(
        &std::fs::read(
            std::env::var("UAR_DOT05_INFRA_FILE")
                .expect("private shared infrastructure manifest required"),
        )
        .unwrap(),
    )
    .unwrap();
    let input = &infrastructure["surreal"];
    let namespace =
        std::env::var("UAR_DOT05_NAMESPACE").expect("isolated shared fixture namespace");
    let shared: Arc<dyn PersistenceLayer> = Arc::new(
        SurrealDbProvider::new(
            input["url"].as_str().unwrap(),
            input["username"].as_str(),
            input["password"].as_str(),
            Some(&namespace),
            Some("stream_terminal"),
        )
        .await
        .unwrap(),
    );
    let mut index = 0;
    for protocol in [
        peer::Protocol::OpenAi,
        peer::Protocol::Azure,
        peer::Protocol::LiterAnthropic,
        peer::Protocol::NativeAnthropic,
        peer::Protocol::GoogleAi,
        peer::Protocol::Vertex,
        peer::Protocol::Cohere,
        peer::Protocol::Bedrock,
    ] {
        for case in [
            "valid",
            "split",
            "empty",
            "missing_terminal",
            "partial_after_terminal",
            "partial_event",
            "malformed",
            "invalid_utf8",
            "malformed_after_terminal",
            "tool",
            "tool_missing_terminal",
            "cancel",
        ] {
            if matches!(protocol, peer::Protocol::Azure)
                && !matches!(case, "valid" | "missing_terminal")
            {
                continue;
            }
            if matches!(protocol, peer::Protocol::Bedrock)
                && matches!(
                    case,
                    "partial_event" | "invalid_utf8" | "malformed_after_terminal"
                )
            {
                continue;
            }
            index += 1;
            let directory = root.join(format!("case-{index}"));
            legacy(&directory.join("legacy"), &certs, protocol, case).await;
            for (name, store) in [("local", local.clone()), ("shared", shared.clone())] {
                graph(&directory.join(name), &certs, store, protocol, case).await;
            }
            println!("dot05 {protocol:?}/{case}: legacy + local/shared graph accepted");
        }
    }
    println!("dot05 controlled scenario matrix: {index} protocol/scenario rows");
}

#[test]
fn stream_terminal_contract_production_boundaries() {
    let root = tempfile::Builder::new()
        .prefix("uar-dot05-integration-")
        .tempdir()
        .unwrap()
        .keep();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    let thread = std::thread::Builder::new()
        .name("dot05-production-gate".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            runtime.block_on(suite(&root));
            println!("dot05 private fixture evidence: {}", root.display());
        })
        .unwrap();
    thread.join().unwrap();
}
