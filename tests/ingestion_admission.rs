//! Issue #426 production ingestion acceptance. Authored with verification CLOSED.
//! Controlled embeddings hold the actual scope-owned blocking worker; extraction,
//! admission, durable claims, generation fencing and publication remain production code.
#![cfg(all(feature = "server", feature = "surreal-backend"))]

use std::{sync::Arc, time::Duration};

use tokio::sync::{mpsc, oneshot};
use universal_agent_runtime::{
    config::AppConfig,
    uar::{
        domain::knowledge::{DocumentStatus, KbConfig, KnowledgeBase, KnowledgeDocument},
        persistence::{PersistenceLayer, ingestion_records::IngestionState,
            providers::surreal::SurrealDbProvider},
        rag::{chunking::ChunkingStrategy,
            embeddings::{EmbeddingBackend, EmbeddingError, swappable::SwappableEmbeddingBackend},
            ingest::IngestService, ingestion_worker::IngestionWorkerPool},
        runtime::lifecycle::{LifecycleState, ResourceScope, RuntimeSupervisor},
    },
};

const OWNER: &str = "anonymous";
const KB: &str = "issue-426";
const WAIT: Duration = Duration::from_secs(30);

#[derive(Debug)]
enum Outcome {
    Publish,
    Fail,
    Panic,
}

#[derive(Debug)]
struct EmbeddingCall {
    text: String,
    finish: oneshot::Sender<Outcome>,
}

#[derive(Debug)]
struct HeldEmbedding {
    calls: mpsc::UnboundedSender<EmbeddingCall>,
}

#[async_trait::async_trait]
impl EmbeddingBackend for HeldEmbedding {
    fn backend_name(&self) -> &str { "openai" }
    fn model_id(&self) -> &str { "issue-426-controlled" }
    fn vector_dimension(&self) -> usize { 384 }

    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        let (finish, finished) = oneshot::channel();
        self.calls.send(EmbeddingCall { text: texts.join("\n"), finish })
            .expect("test retains the admitted embedding call receiver");
        match finished.await.expect("test owns worker completion") {
            Outcome::Publish => Ok(vec![vec![1.0; 384]; texts.len()]),
            Outcome::Fail => Err(EmbeddingError::RequestFailed("issue-426 controlled failure".into())),
            Outcome::Panic => panic!("issue-426 controlled executor panic"),
        }
    }
}

struct Fixture {
    // Keep the physical store directory alive until explicit close acknowledgement.
    _directory: tempfile::TempDir,
    provider: Arc<SurrealDbProvider>,
    persistence: Arc<dyn PersistenceLayer>,
    supervisor: RuntimeSupervisor,
    scope: ResourceScope,
    pool: Arc<IngestionWorkerPool>,
    calls: mpsc::UnboundedReceiver<EmbeddingCall>,
}

impl Fixture {
    async fn new(workers: usize, waiting: usize) -> Self {
        let directory = tempfile::tempdir().unwrap();
        let endpoint = format!("surrealkv://{}", directory.path().join("issue-426.db").display());
        let provider = Arc::new(SurrealDbProvider::new(&endpoint, None, None,
            Some("issue-426"), Some("issue-426")).await.unwrap());
        let persistence: Arc<dyn PersistenceLayer> = provider.clone();
        let now = chrono::Utc::now().to_rfc3339();
        persistence.save_knowledge_base(&KnowledgeBase {
            id: KB.into(), owner_id: OWNER.into(), name: KB.into(), description: None,
            config: KbConfig { file_processor: "local".into(),
                chunk_strategy: ChunkingStrategy::Document, ..KbConfig::default() },
            created_at: now.clone(), updated_at: now,
        }).await.unwrap();
        // Deserialize a closed fixture config; do not load operator files or ambient secrets.
        let config: AppConfig = serde_json::from_value(serde_json::json!({
            "server": {"host":"127.0.0.1", "port":0},
            "security": {"jwt_required":false, "jwt_secret":"issue-426-fixture-only",
                "settings_mutation_auth_required":false},
            "resilience": {"rate_limit_enabled":false, "requests_per_second":10.0, "burst_size":20.0},
            "persistence": {"provider":"surreal", "database_url":endpoint,
                "vector_dimension":384, "external_cache_enabled":false}
        })).unwrap();
        let (calls, receiver) = mpsc::unbounded_channel();
        let embeddings = Arc::new(SwappableEmbeddingBackend::new(Arc::new(HeldEmbedding { calls })));
        let ingest = Arc::new(IngestService::new(Arc::clone(&persistence), embeddings,
            ChunkingStrategy::Document));
        let supervisor = RuntimeSupervisor::new("issue-426-acceptance");
        let scope = supervisor.register("ingestion", "issue-426-acceptance").await.unwrap();
        scope.mark_ready().unwrap();
        let pool = Arc::new(IngestionWorkerPool::new(workers, waiting, ingest,
            Arc::clone(&persistence), Arc::new(config), scope.clone()).unwrap());
        Self { _directory: directory, provider, persistence, supervisor, scope, pool, calls: receiver }
    }

    async fn accept(&self, id: &str) {
        let now = chrono::Utc::now().to_rfc3339();
        let document = KnowledgeDocument {
            id: id.into(), owner_id: OWNER.into(), kb_id: KB.into(), filename: format!("{id}.txt"),
            file_path: None, mime_type: Some("text/plain".into()), chunk_count: 0,
            status: DocumentStatus::Pending, created_at: now.clone(), updated_at: now,
        };
        assert_eq!(self.pool.submit(document, id.as_bytes().to_vec()).await.unwrap(), id);
    }

    async fn next(&mut self) -> EmbeddingCall {
        tokio::time::timeout(WAIT, self.calls.recv()).await
            .expect("finite admission wait expired").expect("embedding channel remains owned")
    }

    async fn close_store(&self) {
        let close = self.provider.close_scope().await;
        assert!(close.failure.is_none(), "embedded engine release failed: {:?}", close.failure);
        assert!(close.engine_close_acknowledged);
    }
}

/// Observe a real worker join, not a wall-clock delay. The dispatcher itself
/// stays admitted until shutdown; every preparation task must have released its
/// lease before this fixture can inspect final durable state.
async fn workers_joined(scope: &ResourceScope) {
    let mut changes = scope.subscribe();
    tokio::time::timeout(WAIT, async {
        loop {
            let state = changes.borrow_and_update().clone();
            if state.outstanding_operation_ids.iter()
                .all(|id| id.contains(":task:ingestion-dispatch:")) {
                return;
            }
            changes.changed().await.expect("scope observation remains retained");
        }
    }).await.expect("actual ingestion worker joins complete within fixture budget");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn durable_backlog_outlives_local_saturation_and_publishes_each_generation_once() {
    let mut fixture = Fixture::new(1, 1).await;
    let ids = ["job-00", "job-01", "job-02", "job-03", "job-04", "job-05"];
    for id in ids { fixture.accept(id).await; }
    fixture.pool.start_durable_dispatch();
    let first = fixture.next().await;
    assert_eq!(first.text, ids[0]);
    admission_counts(&fixture.pool, 1, 1).await;
    // Only the admitted worker may claim a generation or load its source;
    // jobs beyond both the running slot and bounded local queue stay durable.
    for id in &ids[1..] {
        let job = fixture.persistence.get_ingestion_job(OWNER, id).await.unwrap().unwrap();
        assert_eq!(job.state, IngestionState::Pending);
        assert_eq!(job.attempts, 0);
        assert_eq!(fixture.persistence.get_ingestion_source(&job).await.unwrap().unwrap(), id.as_bytes());
    }
    // Repeated notifications must not create duplicate attempts or workers.
    for _ in 0..8 { fixture.pool.notify_committed(); }
    let admission = fixture.pool.admission_snapshot();
    assert_eq!(admission.active_jobs, 1);
    assert_eq!(admission.waiting_jobs, 1, "descriptor bound is independent of durable backlog");
    assert_eq!(admission.active_mailboxes.len(), 1, "duplicate wakeups retain one active receipt");
    first.finish.send(Outcome::Publish).unwrap();
    for id in &ids[1..] {
        let call = fixture.next().await;
        assert_eq!(call.text, *id, "equal-priority admitted descriptors retain discovery order");
        call.finish.send(Outcome::Publish).unwrap();
    }
    workers_joined(&fixture.scope).await;
    admission_counts(&fixture.pool, 0, 0).await;
    for id in ids {
        let job = fixture.persistence.get_ingestion_job(OWNER, id).await.unwrap().unwrap();
        assert_eq!(job.state, IngestionState::Indexed);
        assert_eq!(job.attempts, 1, "repeated discovery did not duplicate a generation");
        let document = fixture.persistence.get_document(OWNER, id).await.unwrap().unwrap();
        assert_eq!(document.status, DocumentStatus::Indexed);
        assert_eq!(document.chunk_count, 1);
    }
    let matches = fixture.persistence.search_knowledge_scoped(OWNER, &[KB],
        &vec![1.0; 384], 100, 0.0).await.unwrap();
    assert_eq!(matches.len(), ids.len(), "exactly one chunk per published document");
    assert!(fixture.calls.try_recv().is_err(), "no additional embedding invocation after joins");
    fixture.pool.shutdown().await.unwrap();
    fixture.close_store().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn two_slots_admit_two_actual_workers_while_the_third_stays_unclaimed() {
    let mut fixture = Fixture::new(2, 1).await;
    for id in ["parallel-00", "parallel-01", "parallel-02"] { fixture.accept(id).await; }
    fixture.pool.start_durable_dispatch();
    let first = fixture.next().await;
    let second = fixture.next().await;
    let mut started = [first.text.as_str(), second.text.as_str()];
    started.sort_unstable();
    assert_eq!(started, ["parallel-00", "parallel-01"]);
    let third = fixture.persistence.get_ingestion_job(OWNER, "parallel-02").await.unwrap().unwrap();
    assert_eq!(third.state, IngestionState::Pending);
    first.finish.send(Outcome::Publish).unwrap();
    let third = fixture.next().await;
    assert_eq!(third.text, "parallel-02");
    second.finish.send(Outcome::Publish).unwrap();
    third.finish.send(Outcome::Publish).unwrap();
    workers_joined(&fixture.scope).await;
    admission_counts(&fixture.pool, 0, 0).await;
    fixture.pool.shutdown().await.unwrap();
    fixture.close_store().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn canceled_shutdown_caller_keeps_the_real_ingestion_worker_and_dependency_owned() {
    let mut fixture = Fixture::new(1, 1).await;
    let (dependency_released, mut dependency_receipt) = oneshot::channel();
    let provider_for_close = Arc::clone(&fixture.provider);
    let persistence_for_close = Arc::clone(&fixture.persistence);
    let dependency = fixture.supervisor.register_with_finalizer("dependent-engine", "issue-426", async move {
        // Inspect durable settlement at the last real storage boundary, then
        // release the actual embedded engine rather than a stand-in finalizer.
        let held = persistence_for_close.get_ingestion_job(OWNER, "held-worker").await?
            .expect("accepted held generation remains stored");
        let queued = persistence_for_close.get_ingestion_job(OWNER, "queued-worker").await?
            .expect("queued generation remains stored");
        let source = persistence_for_close.get_ingestion_source(&queued).await?;
        let close = provider_for_close.close_scope().await;
        anyhow::ensure!(close.failure.is_none(), "physical engine close failed: {:?}", close.failure);
        anyhow::ensure!(close.engine_close_acknowledged, "physical engine close lacked acknowledgement");
        let _ = dependency_released.send((held, queued, source));
        Ok(())
    }).await.unwrap();
    dependency.mark_ready().unwrap();
    fixture.supervisor.declare_dependencies("ingestion", ["dependent-engine"]).await.unwrap();
    fixture.accept("held-worker").await;
    fixture.accept("queued-worker").await;
    fixture.pool.start_durable_dispatch();
    let held = fixture.next().await;
    assert_eq!(held.text, "held-worker");
    let mut states = fixture.scope.subscribe();
    let first_waiter = tokio::spawn({
        let supervisor = fixture.supervisor.clone();
        async move { supervisor.shutdown().await }
    });
    tokio::time::timeout(WAIT, async {
        loop {
            let state = states.borrow_and_update().state.clone();
            if !matches!(state, LifecycleState::Ready | LifecycleState::Registered) { break; }
            states.changed().await.unwrap();
        }
    }).await.expect("shutdown closes ingestion admission");
    assert!(dependency_receipt.try_recv().is_err(), "dependency cannot release a held actual worker");
    assert!(fixture.scope.close_report().is_none(), "cancellation is not worker completion");
    let second_waiter = tokio::spawn({
        let supervisor = fixture.supervisor.clone();
        async move { supervisor.shutdown().await }
    });
    first_waiter.abort();
    assert!(first_waiter.await.unwrap_err().is_cancelled());
    assert!(dependency_receipt.try_recv().is_err(), "dropping shutdown waiter cannot release dependency");
    held.finish.send(Outcome::Publish).unwrap();
    let report = tokio::time::timeout(WAIT, second_waiter).await.unwrap().unwrap();
    assert!(fixture.scope.close_report().is_some(), "shutdown retains an actual worker-join acknowledgement");
    assert!(report.released || !report.failures.is_empty(), "unsuccessful closure retains its failure");
    let (held, queued, source) = dependency_receipt.await.unwrap();
    assert_ne!(held.state, IngestionState::Indexed, "cooperative stop fences late publication");
    assert!(held.last_error.as_deref().is_some_and(|message| message.contains("interrupted")));
    assert_eq!(queued.state, IngestionState::Pending);
    assert_eq!(queued.attempts, 0);
    assert!(source.is_some());
    assert!(fixture.calls.try_recv().is_err(), "shutdown did not start queued preparation");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn failed_and_panicked_executors_return_capacity_after_durable_failure_settles() {
    for failure in [Outcome::Fail, Outcome::Panic] {
        let mut fixture = Fixture::new(1, 1).await;
        fixture.accept("failure-00").await;
        fixture.accept("failure-01").await;
        fixture.pool.start_durable_dispatch();
        let failed = fixture.next().await;
        assert_eq!(failed.text, "failure-00");
        failed.finish.send(failure).unwrap();
        let next = fixture.next().await;
        assert_eq!(next.text, "failure-01", "failure returns a slot and wakes durable dispatch");
        let job = fixture.persistence.get_ingestion_job(OWNER, "failure-00").await.unwrap().unwrap();
        assert_eq!(job.state, IngestionState::Retryable);
        assert!(job.last_error.is_some(), "actual failure persists before replacement worker starts");
        next.finish.send(Outcome::Publish).unwrap();
        workers_joined(&fixture.scope).await;
        admission_counts(&fixture.pool, 0, 0).await;
        let published = fixture.persistence.get_ingestion_job(OWNER, "failure-01").await.unwrap().unwrap();
        assert_eq!(published.state, IngestionState::Indexed);
        // Durable per-job failure is already asserted above. A scope can close
        // cleanly after settled failures; an unsuccessful close must retain its
        // own lifecycle evidence rather than lose the completion receipt.
        let shutdown = fixture.pool.shutdown().await;
        let close = fixture.scope.close_report().unwrap();
        if shutdown.is_err() {
            assert!(!close.released);
            assert!(!close.failures.is_empty() || !close.unresolved_effects.is_empty());
        }
        fixture.close_store().await;
    }
}

/// Watch the operational production snapshot; no polling interval or scheduler
/// sleep stands in for a real admission transition.
async fn admission_counts(pool: &IngestionWorkerPool, active: usize, waiting: usize) {
    let mut changes = pool.subscribe_admission();
    let _observed = tokio::time::timeout(WAIT, changes.wait_for(|state|
        state.active_jobs == active && state.waiting_jobs == waiting))
        .await.expect("admission transition exceeded fixture wait budget")
        .expect("admission observation remains retained");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn canceled_and_timed_out_retrieval_waiters_cannot_return_a_running_slot() {
    let mut fixture = Fixture::new(1, 1).await;
    fixture.accept("retrieval-00").await;
    fixture.accept("retrieval-01").await;
    fixture.pool.start_durable_dispatch();
    let held = fixture.next().await;
    assert_eq!(held.text, "retrieval-00");
    admission_counts(&fixture.pool, 1, 1).await;
    let job = fixture.persistence.get_ingestion_job(OWNER, "retrieval-00").await.unwrap().unwrap();
    let mailbox = fixture.pool.admission_snapshot().active_mailboxes
        .get(&job.storage_key()).cloned().expect("active worker exposes its actual mailbox receipt");

    // Poll the legacy zero-timeout call once to prove it is a live waiter,
    // then cancel that caller by dropping its future while the worker is held.
    let mut retrieval = Box::pin(fixture.pool.retrieve(&mailbox, Duration::ZERO));
    std::future::poll_fn(|context| {
        assert!(std::future::Future::poll(retrieval.as_mut(), context).is_pending(),
            "legacy retrieval timeout cannot manufacture worker completion");
        std::task::Poll::Ready(())
    }).await;
    drop(retrieval);
    assert!(tokio::time::timeout(Duration::ZERO,
        fixture.pool.retrieve(&mailbox, Duration::ZERO)).await.is_err(),
        "external timeout ends only the retrieval caller");

    let admission = fixture.pool.admission_snapshot();
    assert_eq!(admission.active_jobs, 1, "neither canceled observer returned the running grant");
    assert_eq!(admission.waiting_jobs, 1);
    assert_eq!(admission.active_mailboxes.get(&job.storage_key()), Some(&mailbox));
    let queued = fixture.persistence.get_ingestion_job(OWNER, "retrieval-01").await.unwrap().unwrap();
    assert_eq!(queued.state, IngestionState::Pending);
    assert_eq!(queued.attempts, 0);

    held.finish.send(Outcome::Publish).unwrap();
    let next = fixture.next().await;
    assert_eq!(next.text, "retrieval-01", "actual worker completion returns capacity and wakes its successor");
    let first = fixture.persistence.get_ingestion_job(OWNER, "retrieval-00").await.unwrap().unwrap();
    assert_eq!(first.state, IngestionState::Indexed);
    next.finish.send(Outcome::Publish).unwrap();
    workers_joined(&fixture.scope).await;
    admission_counts(&fixture.pool, 0, 0).await;
    fixture.pool.shutdown().await.unwrap();
    fixture.close_store().await;
}

#[tokio::test]
async fn production_admission_rejects_oversized_work_and_wakes_fifo_after_a_grant_drop() {
    use prometheus_parking_lot_core::{
        ScheduledTask, SchedulerError, TaskMetadata,
        Priority, ResourceCost, ResourceKind,
    };
    use universal_agent_runtime::uar::rag::ingestion_admission::IngestionAdmission;

    fn descriptor(id: u64, units: u32) -> ScheduledTask<u64> {
        ScheduledTask {
            meta: TaskMetadata {
                id, priority: Priority::Normal,
                cost: ResourceCost::new(ResourceKind::Cpu, units),
                created_at_ms: 0, deadline_ms: None, mailbox: None,
                idempotency_key: None, sequence: 0,
            },
            payload: id,
        }
    }

    // This is the same UAR admission wrapper used by IngestionWorkerPool,
    // not an upstream-only dispatcher assertion or a fabricated document cost.
    let admission = IngestionAdmission::new(1, 2).unwrap();
    let wake = Arc::new(tokio::sync::Notify::new());
    admission.install_release_wake(Arc::clone(&wake));
    assert!(matches!(admission.submit(descriptor(0, 2), 0),
        Err(SchedulerError::CapacityExceeded { requested: 2, available: 1, .. })));
    assert_eq!(admission.waiting_jobs(), 0, "oversized request cannot become a blocking queue head");
    assert_eq!(admission.active_jobs(), 0);

    let mut first = admission.submit(descriptor(1, 1), 0).unwrap();
    assert!(first.started, "eligible work follows oversized rejection immediately");
    let first = first.dispatched.started.pop().unwrap();
    assert_eq!(first.item.payload, 1);
    assert!(!admission.submit(descriptor(2, 1), 0).unwrap().started);
    assert!(!admission.submit(descriptor(3, 1), 0).unwrap().started);
    assert!(matches!(admission.submit(descriptor(4, 1), 0), Err(SchedulerError::QueueFull { .. })));
    assert_eq!(admission.waiting_jobs(), 2);

    let mut released = admission.complete(first.permit, 0);
    assert_eq!(released.started.len(), 1);
    let second = released.started.pop().unwrap();
    assert_eq!(second.item.payload, 2, "normal priority uses strict FIFO without head skips");
    assert_eq!(admission.waiting_jobs(), 1);

    // An exceptional grant drop must supply a host wake. Poke then admits the
    // successor without another submission, so returned capacity cannot sleep.
    // Consume any notification from the earlier normal completion before
    // requiring a new notification from this exceptional permit drop.
    let mut prior_wake = Box::pin(wake.notified());
    std::future::poll_fn(|context| {
        let _ = std::future::Future::poll(prior_wake.as_mut(), context);
        std::task::Poll::Ready(())
    }).await;
    drop(prior_wake);
    drop(second.permit);
    tokio::time::timeout(WAIT, wake.notified()).await.expect("grant release wakes host dispatch");
    let mut released = admission.poke(0);
    assert_eq!(released.started.len(), 1);
    let third = released.started.pop().unwrap();
    assert_eq!(third.item.payload, 3);
    assert!(!admission.submit(descriptor(5, 1), 0).unwrap().started);
    let parked = admission.close();
    assert_eq!(parked.len(), 1);
    assert_eq!(parked[0].payload, 5, "close returns the unexecuted descriptor for durable recovery");
    assert!(matches!(admission.submit(descriptor(6, 1), 0), Err(SchedulerError::PoolShutdown { .. })));
    assert!(admission.complete(third.permit, 0).started.is_empty());
    assert_eq!(admission.waiting_jobs(), 0);
    assert_eq!(admission.active_jobs(), 0);
}
