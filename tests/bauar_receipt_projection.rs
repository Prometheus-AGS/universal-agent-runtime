//! SOURCE ONLY: real router/Liter/native/store acceptance scenarios, not run here.
//! The fixture supplies identity; it does not certify authentication or live providers.
#![cfg(feature = "surreal-backend")]

#[path = "support/bauar_receipt_peer.rs"]
mod peer;

use axum::{
    Extension, Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tower::ServiceExt;
use universal_agent_runtime::{
    config::{LlmConfig, ServiceInstanceConfig},
    mcp::registry::McpRegistry,
    session::SessionStore,
    uar::{
        api::routes::{self, RunApiState},
        compiler::collaboration::CollaborationCatalogService,
        defaults::default_agent,
        domain::{events::NormalizedEvent, runs::RunStatus},
        persistence::{
            PersistenceLayer,
            agent_threads::{
                CanonicalReceiptCompleteness, CanonicalReceiptSource, CanonicalReceiptStoreError,
                CanonicalToolReceipt,
            },
            providers::surreal::SurrealDbProvider,
        },
        rag::embeddings::UnavailableEmbeddingBackend,
        runtime::{
            manager::RunManager,
            matching::{VectorMatcher, intent::ClassifierConfig},
            native_skill::NativeSkillRegistry,
            skills::SkillRegistry,
        },
        security::claims::{UserClaims, UserContext},
        service_instance::ServiceInstanceAuthority,
    },
};

const OWNER: &str = "receipt-owner";
const WAIT: Duration = Duration::from_secs(45);

struct Fixture {
    app: Router,
    manager: Arc<RunManager>,
    store: Option<Arc<dyn PersistenceLayer>>,
    peer: peer::Peer,
    // The private embedded database path stays alive as long as the injected store.
    _directory: tempfile::TempDir,
}

impl Fixture {
    async fn new(with_store: bool) -> Self {
        let directory = tempfile::tempdir().unwrap();
        let store: Option<Arc<dyn PersistenceLayer>> = if with_store {
            Some(Arc::new(
                SurrealDbProvider::new(
                    &format!("surrealkv://{}", directory.path().display()),
                    None,
                    None,
                    None,
                    None,
                )
                .await
                .unwrap(),
            ))
        } else {
            None
        };
        let peer = peer::Peer::start().await;
        let native = Arc::new(NativeSkillRegistry::new());
        native
            .register(peer::ReceiptTool {
                seen: peer.seen.clone(),
            })
            .await
            .unwrap();
        let manager = Arc::new(
            RunManager::with_classifier_config(
                LlmConfig {
                    model: "openai/gpt-4o-mini".to_owned(),
                    base_url: Some(peer.url.clone()),
                    api_key: Some(peer::SECRET.to_owned()),
                    host_supplied_connection: true,
                    ..LlmConfig::default()
                },
                Arc::new(McpRegistry::new_empty()),
                SessionStore::new(),
                Arc::new(RwLock::new(SkillRegistry::default())),
                Arc::new(VectorMatcher::new(
                    Arc::new(UnavailableEmbeddingBackend::new(
                        384,
                        "receipt fixture does not use embeddings",
                    )),
                    0.75,
                )),
                store.clone(),
                ClassifierConfig::default(),
                native,
            )
            .await,
        );
        let runs = Arc::new(RunApiState {
            manager: manager.clone(),
            collaboration_catalog: Arc::new(CollaborationCatalogService::in_memory()),
            service_instance: Arc::new(ServiceInstanceAuthority::new(
                &ServiceInstanceConfig::default(),
                "receipt-instance",
                "http://127.0.0.1",
                false,
                &[],
            )),
        });
        let user = UserContext {
            user_id: OWNER.to_owned(),
            tenant_id: None,
            authority: None,
            host_authority: None,
            claims: UserClaims {
                sub: OWNER.to_owned(),
                name: None,
                roles: None,
                tenant_id: None,
                uar_instance_id: None,
                exp: usize::MAX,
            },
        };
        let app = Router::new()
            .nest("/api/uar", routes::build_router().with_state(runs))
            .layer(Extension(user));
        Self {
            app,
            manager,
            store,
            peer,
            _directory: directory,
        }
    }

    async fn run(&self, mode: &str) -> String {
        let mut artifact = default_agent();
        artifact.id = format!("receipt-{mode}");
        artifact.memory.kb.enabled = false;
        artifact.policy.tools.allow = vec![peer::TOOL.to_owned()];
        let request = Request::builder()
            .uri("/api/uar/runs")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({"artifact":artifact,"input":mode}).to_string(),
            ))
            .unwrap();
        let response = tokio::time::timeout(WAIT, self.app.clone().oneshot(request))
            .await
            .expect("router admission deadline")
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let admitted: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 1024 * 1024).await.unwrap())
                .unwrap();
        let run = admitted["run_id"].as_str().unwrap().to_owned();
        tokio::time::timeout(WAIT, async {
            loop {
                let terminal = self.manager.get_run(&run).await.is_some_and(|run| {
                    matches!(
                        run.status,
                        RunStatus::Done | RunStatus::Error | RunStatus::Cancelled
                    )
                });
                if terminal
                    && self
                        .manager
                        .history_since(&run, None)
                        .await
                        .is_some_and(|history| {
                            history.iter().any(|event| {
                                matches!(
                                    event.event,
                                    NormalizedEvent::RunDone { .. }
                                        | NormalizedEvent::RunDoneWithUsage { .. }
                                        | NormalizedEvent::Error { .. }
                                        | NormalizedEvent::Cancelled { .. }
                                )
                            })
                        })
                {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("real native/store execution terminal deadline");
        run
    }

    fn original_execution(&self, mode: &str) {
        assert_eq!(
            *self.peer.seen.executions.lock().unwrap(),
            vec![json!({"mode":mode,"echo":peer::SECRET,"control":peer::CONTROL})]
        );
        assert_eq!(
            *self.peer.seen.acquisitions.lock().unwrap(),
            vec![mode.to_owned()]
        );
    }

    async fn produced_receipt(&self, run: &str, mode: &str) -> CanonicalToolReceipt {
        let receipts = self
            .store
            .as_ref()
            .unwrap()
            .list_canonical_tool_receipts(OWNER, run)
            .await
            .unwrap();
        assert_eq!(
            receipts.len(),
            1,
            "must retrieve the receipt produced by actual execution"
        );
        let receipt = receipts.into_iter().next().unwrap();
        assert_eq!(receipt.owner_id, OWNER);
        assert_eq!(receipt.run_id, run);
        assert_eq!(receipt.call_id, format!("receipt-{mode}"));
        assert_eq!(receipt.tool, peer::TOOL);
        assert_eq!(receipt.source, CanonicalReceiptSource::Native);
        receipt
    }

    async fn projected_output(&self, run: &str, mode: &str) {
        let history = self.manager.history_since(run, None).await.unwrap();
        let output = history
            .iter()
            .find_map(|event| match &event.event {
                NormalizedEvent::ToolEnd {
                    tool, output, ok, ..
                } if tool == peer::TOOL => {
                    assert!(*ok);
                    Some(output)
                }
                _ => None,
            })
            .expect("actual projected ToolEnd must be retained");
        // Run events wrap the native format_result JSON as a text content block.
        let event_text = serde_json::to_string(output).unwrap();
        assert!(event_text.contains(peer::CONTROL));
        assert!(event_text.contains("[REDACTED]"));
        assert_no_variants(&event_text);
        let requests = self.peer.seen.requests.lock().unwrap();
        let content = requests
            .iter()
            .flat_map(|request| request["messages"].as_array().unwrap())
            .find(|message| message["role"] == "tool")
            .expect("positive control requires actual post-tool Liter request")["content"]
            .as_str()
            .unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(content).unwrap(),
            peer::expected_value(mode)
        );
        assert_no_variants(content);
        let chat = history
            .iter()
            .filter_map(|event| match &event.event {
                NormalizedEvent::ChatDelta { text_delta, .. } => Some(text_delta.as_str()),
                _ => None,
            })
            .collect::<String>();
        assert!(chat.contains("post-tool-positive-control"));
    }

    async fn exact_replay(&self, receipt: &CanonicalToolReceipt) {
        let store = self.store.as_ref().unwrap();
        assert_eq!(
            store.save_canonical_tool_receipt(receipt).await.unwrap(),
            *receipt
        );
        let mut changed = receipt.clone();
        changed
            .secret_projection
            .as_mut()
            .unwrap()
            .omitted_raw_segments += 1;
        let error = store
            .save_canonical_tool_receipt(&changed)
            .await
            .unwrap_err();
        assert!(matches!(
            error.downcast_ref::<CanonicalReceiptStoreError>(),
            Some(CanonicalReceiptStoreError::Conflict)
        ));
        if let CanonicalReceiptCompleteness::AcquisitionIncomplete { observed_bytes } =
            &receipt.completeness
        {
            let mut changed = receipt.clone();
            changed.completeness = CanonicalReceiptCompleteness::AcquisitionIncomplete {
                observed_bytes: *observed_bytes + 1,
            };
            let error = store
                .save_canonical_tool_receipt(&changed)
                .await
                .unwrap_err();
            assert!(matches!(
                error.downcast_ref::<CanonicalReceiptStoreError>(),
                Some(CanonicalReceiptStoreError::Conflict)
            ));
        }
        assert_eq!(
            store
                .list_canonical_tool_receipts(OWNER, &receipt.run_id)
                .await
                .unwrap(),
            vec![receipt.clone()]
        );
    }
}

fn assert_no_variants(text: &str) {
    for variant in peer::variants() {
        assert!(!text.contains(&variant), "captured variant retained");
    }
}

fn projected_identity(receipt: &CanonicalToolReceipt, mode: &str, omissions: u64) {
    let expected = serde_json::to_vec(&peer::expected_value(mode)).unwrap();
    assert_eq!(receipt.typed_value_bytes, expected.len() as u64);
    assert_eq!(receipt.typed_value_sha256, peer::digest(&expected));
    assert_ne!(
        receipt.typed_value_bytes,
        serde_json::to_vec(&peer::typed_result(mode, peer::SECRET))
            .unwrap()
            .len() as u64
    );
    assert_ne!(
        receipt.typed_value_sha256,
        peer::digest(&serde_json::to_vec(&peer::typed_result(mode, peer::SECRET)).unwrap())
    );
    let metadata = receipt.secret_projection.as_ref().unwrap();
    assert_eq!(metadata.version, 1);
    assert!(metadata.redacted);
    assert_eq!(metadata.omitted_raw_segments, omissions);
    assert_eq!(
        receipt.raw_segments.len(),
        1,
        "opaque and unavailable streams are explicitly omitted"
    );
    let stream = &receipt.raw_segments[0];
    assert_eq!(stream.name, "stdout:[REDACTED]");
    assert_eq!(stream.byte_len, peer::expected_text().len() as u64);
    assert_ne!(
        stream.byte_len,
        peer::expected_text().chars().count() as u64
    );
    assert_eq!(
        stream.sha256,
        peer::digest(peer::expected_text().as_bytes())
    );
    assert_ne!(stream.sha256, peer::digest(peer::raw_text().as_bytes()));
    assert_ne!(stream.byte_len, peer::raw_text().len() as u64);
    let serialized = serde_json::to_string(receipt).unwrap();
    assert_no_variants(&serialized);
    assert!(!serialized.contains(&peer::digest(&peer::opaque_bytes())));
    assert!(!serialized.contains(&STANDARD.encode(peer::opaque_bytes())));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn actual_complete_acquisition_stores_projected_receipt_and_preserves_replay() {
    let fixture = Fixture::new(true).await;
    let run = fixture.run("complete").await;
    fixture.original_execution("complete");
    let receipt = fixture.produced_receipt(&run, "complete").await;
    projected_identity(&receipt, "complete", 1);
    assert_eq!(receipt.completeness, CanonicalReceiptCompleteness::Complete);
    assert_eq!(receipt.typed_value, Some(peer::expected_value("complete")));
    let expected_raw = peer::expected_text().into_bytes();
    assert_eq!(
        receipt.raw_segments[0].verified_bytes().unwrap(),
        Some(expected_raw.clone())
    );
    assert_eq!(
        receipt.raw_segments[0].bytes_base64,
        Some(STANDARD.encode(&expected_raw))
    );
    assert_eq!(
        receipt.retained_bytes,
        receipt.typed_value_bytes + expected_raw.len() as u64
    );
    fixture.projected_output(&run, "complete").await;
    fixture.exact_replay(&receipt).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn actual_incomplete_acquisition_keeps_disposition_and_projected_identity() {
    let fixture = Fixture::new(true).await;
    let run = fixture.run("incomplete").await;
    fixture.original_execution("incomplete");
    let receipt = fixture.produced_receipt(&run, "incomplete").await;
    projected_identity(&receipt, "incomplete", 2);
    assert_eq!(
        receipt.completeness,
        CanonicalReceiptCompleteness::AcquisitionIncomplete {
            observed_bytes: peer::observed_bytes("incomplete"),
        }
    );
    assert!(receipt.typed_value.is_none());
    assert_eq!(receipt.retained_bytes, 0);
    assert!(receipt.raw_segments[0].bytes_base64.is_none());
    assert!(receipt.raw_segments[0].verified_bytes().unwrap().is_none());
    assert!(
        !fixture
            .peer
            .seen
            .requests
            .lock()
            .unwrap()
            .iter()
            .any(|request| request["messages"]
                .as_array()
                .unwrap()
                .iter()
                .any(|message| message["role"] == "tool"))
    );
    let history = fixture.manager.history_since(&run, None).await.unwrap();
    assert!(history.iter().any(|event| matches!(&event.event,
        NormalizedEvent::Error {code, ..} if code == "TERMINAL_RESULT_PERSISTENCE_FAILED")));
    fixture.exact_replay(&receipt).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn actual_no_store_early_return_projects_both_acquisition_modes_before_history() {
    for mode in ["complete", "incomplete"] {
        let fixture = Fixture::new(false).await;
        assert!(
            fixture.store.is_none(),
            "manager receives the actual no-store configuration"
        );
        let run = fixture.run(mode).await;
        fixture.original_execution(mode);
        fixture.projected_output(&run, mode).await;
        assert_eq!(
            fixture.manager.get_run(&run).await.unwrap().status,
            RunStatus::Done
        );
    }
}
