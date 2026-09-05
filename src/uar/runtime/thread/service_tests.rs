//! Host-path regressions: real manager capture, actor root, persistence and jobs.

use super::*;
use crate::config::{A2aConfig, A2aPeerConfig, LlmConfig};
use crate::llm::{ExternalDriverStream, LlmDriver, LlmRequest};
use crate::uar::runtime::thread::actor_host::ActorThreadSession;
use crate::uar::runtime::{
    actor::messages::ActorOwner, manager::RunManager, matching::VectorMatcher,
    skills::SkillRegistry,
};
use crate::uar::security::claims::{UserClaims, UserContext};
use std::time::Duration;

struct HoldingDriver(tokio::sync::Notify);

#[async_trait::async_trait]
impl LlmDriver for HoldingDriver {
    async fn stream(&self, _request: LlmRequest) -> anyhow::Result<ExternalDriverStream> {
        self.0.notify_one();
        Ok(Box::pin(futures::stream::pending()))
    }
}

struct Harness {
    service: Arc<ThreadService>,
    controls: Arc<AgentToolContext>,
    root_job: JoinHandle<anyhow::Result<PersistedAgentThread>>,
    listener: tokio::net::TcpListener,
    endpoint: String,
    cancellation: CancellationToken,
    _database: tempfile::TempDir,
}

impl Harness {
    async fn new() -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!(
            "http://{}/a2a/agents/peer-agent",
            listener.local_addr().unwrap()
        );
        let database = tempfile::tempdir().unwrap();
        let database_endpoint = format!(
            "surrealkv://{}",
            database.path().join("threads.db").display()
        );
        let persistence: Arc<dyn PersistenceLayer> = Arc::new(
            crate::uar::persistence::providers::surreal::SurrealDbProvider::new(
                &database_endpoint,
                None,
                None,
                Some("remote-host"),
                Some("remote-host"),
            )
            .await
            .unwrap(),
        );
        let driver = Arc::new(HoldingDriver(tokio::sync::Notify::new()));
        let manager = Arc::new(
            RunManager::new(
                LlmConfig::default(),
                Arc::new(crate::mcp::registry::McpRegistry::new_empty()),
                crate::session::SessionStore::new(),
                Arc::new(tokio::sync::RwLock::new(SkillRegistry::default())),
                Arc::new(VectorMatcher::new(
                    Arc::new(
                        crate::uar::rag::embeddings::UnavailableEmbeddingBackend::new(
                            384,
                            "host lifecycle fixture",
                        ),
                    ),
                    0.75,
                )),
                Some(persistence.clone()),
            )
            .await
            .with_llm_driver(driver.clone())
            .with_a2a_config(&A2aConfig {
                instance_id: "source-instance".into(),
                trusted_peers: vec![A2aPeerConfig {
                    instance_id: "peer-instance".into(),
                    agent_id: "peer-agent".into(),
                    endpoint: endpoint.clone(),
                    bearer_token: "test-peer-token".to_owned().into(),
                }],
            }),
        );
        let mut artifact = crate::uar::defaults::default_agent();
        artifact.policy.tools.allow = vec!["spawn_agent".into()];
        artifact.extensions.insert(
            "budgets".into(),
            serde_json::json!({"max_tokens_per_turn":1000}),
        );
        artifact.extensions.insert(
            "a2a".into(),
            serde_json::json!({"dependencies":[{"agent_id":"peer-agent","endpoint":endpoint}]}),
        );
        let user = UserContext {
            user_id: "remote-budget-owner".into(),
            tenant_id: None,
            claims: UserClaims {
                sub: "remote-budget-owner".into(),
                name: None,
                roles: None,
                tenant_id: None,
                uar_instance_id: None,
                exp: usize::MAX,
            },
        };
        let owner = ActorOwner::from_verified_context(&user).unwrap();
        let (state, _receiver) = watch::channel(None);
        let owned = Arc::new(Mutex::new(None));
        let cancellation = CancellationToken::new();
        let mut actor = ActorThreadSession::new(
            owner,
            artifact,
            uuid::Uuid::new_v4().to_string(),
            manager,
            persistence,
            cancellation.clone(),
            state,
            owned.clone(),
        );
        let root_job = tokio::spawn(async move {
            actor
                .execute("hold root while testing remote admission".into())
                .await
        });
        tokio::time::timeout(Duration::from_secs(10), driver.0.notified())
            .await
            .expect("real root reaches model after capture");
        let service = owned
            .lock()
            .await
            .as_ref()
            .unwrap()
            .service
            .get()
            .unwrap()
            .clone();
        let controls = service.root_controls().await.unwrap();
        assert!(controls.permits("spawn_agent"));
        Self {
            service,
            controls,
            root_job,
            listener,
            endpoint,
            cancellation,
            _database: database,
        }
    }

    fn request(&self) -> RemoteAgentSpawnRequest {
        RemoteAgentSpawnRequest {
            endpoint: self.endpoint.clone(),
            delegated_prompt: "never send this task".into(),
            task_name: Some("remote-probe".into()),
        }
    }

    fn assert_capacity_restored(&self) {
        let lease = self
            .service
            .inner
            .kernel
            .reserve_remote_budget(&ThreadBudgets::default())
            .expect("root can reserve its full unused allowance again");
        assert_eq!(lease.grant().max_total_tokens, Some(1000));
        lease.release_confirmed().unwrap();
    }

    async fn finish(self) {
        let expected_children = self
            .service
            .inner
            .entries
            .lock()
            .await
            .values()
            .filter(|entry| entry.record.thread.parent_thread_id.is_some())
            .map(|entry| entry.record.thread.thread_id.clone())
            .collect::<Vec<_>>();
        tokio::time::timeout(Duration::from_secs(5), self.service.shutdown())
            .await
            .expect("shutdown is bounded")
            .expect("all host jobs join and pending records settle");
        tokio::time::timeout(Duration::from_secs(5), self.service.shutdown())
            .await
            .expect("repeated shutdown is bounded")
            .expect("joined shutdown remains successful");
        let records = self
            .service
            .inner
            .persistence
            .list_agent_threads(
                &self.service.inner.root.owner_id,
                &self.service.inner.root.root_run_id,
            )
            .await
            .unwrap();
        for child_id in expected_children {
            let child = records
                .iter()
                .find(|record| record.thread.thread_id == child_id)
                .expect("accepted child remains durably present after shutdown");
            assert_eq!(
                child.thread.status,
                AgentThreadStatus::Cancelled,
                "shutdown must settle the durable child, not only its watcher"
            );
        }
        self.cancellation.cancel();
        let record = tokio::time::timeout(Duration::from_secs(5), self.root_job)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert_eq!(record.thread.status, AgentThreadStatus::Cancelled);
        assert!(
            self.service
                .inner
                .jobs
                .lock()
                .unwrap()
                .entries
                .iter()
                .all(|job| job.is_joined())
        );
        assert!(
            tokio::time::timeout(Duration::from_millis(25), self.listener.accept())
                .await
                .is_err(),
            "undispatched child must not open a peer connection"
        );
    }
}

use super::super::policy_intersection::ThreadBudgets;

#[tokio::test]
async fn remote_admission_refusal_does_not_lease_root_budget() {
    let harness = Harness::new().await;
    let mut occupied = Vec::new();
    for index in 0..4 {
        let child = AgentThread::child(
            &harness.service.inner.root,
            "local-slot".into(),
            Some(&format!("slot-{index}")),
        )
        .unwrap();
        occupied.push(
            harness
                .service
                .inner
                .admission
                .reserve_child(&child)
                .unwrap(),
        );
    }
    let error = harness
        .service
        .spawn_remote(harness.controls.scope(), harness.request())
        .await
        .unwrap_err();
    assert!(
        error.to_string().contains("maximum concurrent children"),
        "{error}"
    );
    harness.assert_capacity_restored();
    drop(occupied);
    harness.finish().await;
}

#[tokio::test]
async fn remote_persisted_launch_refusal_releases_capacity_and_shutdown_settles_record() {
    let harness = Harness::new().await;
    // Exercise the exact post-persistence launch refusal without cancelling the
    // root before authorize() has a chance to admit the operation.
    harness.service.inner.jobs.lock().unwrap().closed = true;
    let error = harness
        .service
        .spawn_remote_inner(harness.controls.scope().clone(), harness.request())
        .await
        .unwrap_err();
    assert!(error.to_string().contains("shutting down"), "{error}");
    let entries = harness.service.inner.entries.lock().await;
    let child = entries
        .values()
        .find(|entry| entry.record.thread.parent_thread_id.is_some())
        .unwrap();
    assert!(child.confirmed);
    assert_eq!(child.record.thread.status, AgentThreadStatus::Pending);
    let ChildTarget::Remote(remote) = &child.target else {
        panic!("remote child")
    };
    assert!(!remote.execution_admitted);
    assert!(remote.reservation.is_released());
    drop(entries);
    harness.assert_capacity_restored();
    harness.finish().await;
}

#[tokio::test]
async fn remote_pending_cancellation_releases_capacity_before_any_dispatch() {
    let harness = Harness::new().await;
    // launch() is the last synchronous operation after persistence. On this
    // current-thread runtime, cancel before yielding to its spawned worker.
    let child = harness
        .service
        .spawn_remote_inner(harness.controls.scope().clone(), harness.request())
        .await
        .unwrap();
    let entries = harness.service.inner.entries.lock().await;
    let entry = &entries[&child.thread_id];
    let ChildTarget::Remote(remote) = &entry.target else {
        panic!("remote child")
    };
    assert!(
        !remote.execution_admitted,
        "cancellation point must precede dispatch"
    );
    assert!(!remote.reservation.is_released());
    entry.cancellation.cancel();
    let handle = entry.handle.clone();
    drop(entries);
    let terminal = tokio::time::timeout(Duration::from_secs(5), handle.wait_until_terminal())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(terminal.status, AgentThreadStatus::Cancelled);
    harness.assert_capacity_restored();
    harness.finish().await;
}
