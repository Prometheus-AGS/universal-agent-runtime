//! Lifecycle ownership integration acceptance.
//!
//! These cases use real trusted-host scopes and an embedded SurrealKV provider.
//! Every hold/release is an owned one-shot or blocking channel; no deadline,
//! sleep, retry, or lock probe establishes lifecycle completion.
#![cfg(all(feature = "server", feature = "surreal-backend"))]

use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
};

use anyhow::{Context, Result};
use tokio::sync::oneshot;
use universal_agent_runtime::uar::{
    persistence::{
        PersistenceLayer,
        providers::surreal::{HostSurrealBackend, SurrealDbProvider},
        scoped::ScopedPersistence,
    },
    runtime::lifecycle::RuntimeSupervisor,
};

#[tokio::test]
async fn canceled_blocking_waiter_keeps_scope_open_until_the_worker_exits() {
    let supervisor = RuntimeSupervisor::new("lifecycle-owned-resources");
    let worker_scope = supervisor
        .register("blocking-worker", "lifecycle-owned-resources")
        .await
        .expect("register blocking worker scope");
    worker_scope.mark_ready().expect("publish blocking worker readiness");

    let (worker_started, worker_started_receipt) = oneshot::channel();
    let (worker_exited, worker_exited_receipt) = oneshot::channel();
    let (release_worker, wait_for_release) = mpsc::channel();
    let (finalizer_started, mut finalizer_started_receipt) = oneshot::channel();
    worker_scope
        .set_finalizer(async move {
            let _ = finalizer_started.send(());
            Ok(())
        })
        .expect("register blocking worker finalizer");

    let blocking_waiter = worker_scope
        .spawn_blocking("held-worker", move || {
            let _ = worker_started.send(());
            wait_for_release
                .recv()
                .expect("test owns blocking worker release");
            let _ = worker_exited.send(());
        })
        .expect("admit blocking worker");
    worker_started_receipt
        .await
        .expect("blocking worker accepted before its waiter is canceled");
    drop(blocking_waiter);

    let shutdown_waiter = tokio::spawn({
        let supervisor = supervisor.clone();
        async move { supervisor.shutdown().await }
    });

    assert!(
        finalizer_started_receipt.try_recv().is_err(),
        "the resource finalizer cannot begin while the admitted worker remains held"
    );
    release_worker
        .send(())
        .expect("release the owned blocking worker");
    worker_exited_receipt
        .await
        .expect("blocking worker acknowledged exit after its waiter was dropped");
    finalizer_started_receipt
        .await
        .expect("finalizer starts only after the worker exits");

    let report = shutdown_waiter
        .await
        .expect("shutdown waiter joins retained finalization");
    assert!(report.released, "supervisor reports actual closure");
    let worker_report = supervisor
        .resource_report("blocking-worker")
        .await
        .expect("read retained worker report")
        .expect("worker scope closed");
    assert_eq!(worker_report.completed_operations, 1);
}

#[tokio::test]
async fn concurrent_and_canceled_shutdown_waiters_share_one_finalizer_result() {
    let supervisor = RuntimeSupervisor::new("lifecycle-owned-resources");
    let finalizer_calls = Arc::new(AtomicUsize::new(0));
    let (finalizer_started, finalizer_started_receipt) = oneshot::channel();
    let (release_finalizer, wait_for_release) = oneshot::channel();
    let finalizer_calls_for_task = Arc::clone(&finalizer_calls);
    let scope = supervisor
        .register_with_finalizer(
            "retained-finalizer",
            "lifecycle-owned-resources",
            async move {
                finalizer_calls_for_task.fetch_add(1, Ordering::SeqCst);
                let _ = finalizer_started.send(());
                wait_for_release
                    .await
                    .map_err(|_| anyhow::anyhow!("test dropped finalizer release"))?;
                Ok(())
            },
        )
        .await
        .expect("register retained finalizer");
    scope.mark_ready().expect("publish finalizer scope readiness");

    let canceled_waiter = tokio::spawn({
        let supervisor = supervisor.clone();
        async move { supervisor.shutdown().await }
    });
    finalizer_started_receipt
        .await
        .expect("one finalizer begins for the first shutdown caller");

    let surviving_waiter = tokio::spawn({
        let supervisor = supervisor.clone();
        async move { supervisor.shutdown().await }
    });
    canceled_waiter.abort();
    let canceled = canceled_waiter.await.expect_err("first shutdown waiter is canceled");
    assert!(canceled.is_cancelled());

    release_finalizer
        .send(())
        .expect("release the single retained finalizer");
    let surviving_report = surviving_waiter
        .await
        .expect("surviving shutdown waiter observes completion");
    let repeated_report = supervisor.shutdown().await;

    assert_eq!(finalizer_calls.load(Ordering::SeqCst), 1);
    assert!(surviving_report.released);
    assert!(repeated_report.released);
    assert_eq!(
        surviving_report.completed_tasks,
        repeated_report.completed_tasks,
        "repeated shutdown observes the retained result rather than another cleanup"
    );
}

#[tokio::test]
async fn producer_drains_before_owned_embedded_engine_close_and_one_reopen() -> Result<()> {
    let directory = tempfile::tempdir().context("create embedded-store fixture directory")?;
    let endpoint = format!("surrealkv://{}", directory.path().join("lifecycle.db").display());
    let provider = Arc::new(
        SurrealDbProvider::new(
            &endpoint,
            None,
            None,
            Some("lifecycle-acceptance"),
            Some("lifecycle-acceptance"),
        )
        .await
        .context("open owned embedded provider")?,
    );

    let supervisor = RuntimeSupervisor::new("lifecycle-owned-resources");
    let (engine_finalizer_started, mut engine_finalizer_started_receipt) = oneshot::channel();
    let provider_for_finalizer = Arc::clone(&provider);
    let engine_scope = supervisor
        .register_with_finalizer("embedded-engine", "lifecycle-owned-resources", async move {
            let _ = engine_finalizer_started.send(());
            let close = provider_for_finalizer.close_scope().await;
            anyhow::ensure!(close.failure.is_none(), "embedded provider close failed: {:?}", close.failure);
            anyhow::ensure!(close.engine_close_acknowledged, "embedded engine close lacked acknowledgement");
            Ok(())
        })
        .await
        .context("register embedded engine finalizer")?;
    engine_scope.mark_ready().context("publish engine readiness")?;

    let producer_scope = supervisor
        .register("database-producer", "lifecycle-owned-resources")
        .await
        .context("register database producer")?;
    producer_scope.mark_ready().context("publish producer readiness")?;
    supervisor
        .declare_dependencies("database-producer", ["embedded-engine"])
        .await
        .context("retain engine until producer finalization")?;

    let scoped: Arc<dyn PersistenceLayer> = Arc::new(ScopedPersistence::new(
        Arc::clone(&provider) as Arc<dyn PersistenceLayer>,
        producer_scope.clone(),
    ));
    let (producer_started, producer_started_receipt) = oneshot::channel();
    let (release_producer, wait_for_release) = oneshot::channel();
    let (provider_query_finished, provider_query_finished_receipt) = oneshot::channel();
    producer_scope
        .spawn("database-dependent-producer", async move {
            let _ = producer_started.send(());
            wait_for_release
                .await
                .map_err(|_| anyhow::anyhow!("test dropped producer release"))?;
            scoped
                .list_settings_types()
                .await
                .context("provider remains usable until producer joins")?;
            let _ = provider_query_finished.send(());
            Ok(())
        })
        .context("admit database-dependent producer")?;
    producer_started_receipt
        .await
        .context("producer accepted before shutdown")?;

    let shutdown_waiter = tokio::spawn({
        let supervisor = supervisor.clone();
        async move { supervisor.shutdown().await }
    });
    assert!(
        engine_finalizer_started_receipt.try_recv().is_err(),
        "engine finalizer cannot close a dependency while its producer remains admitted"
    );

    release_producer
        .send(())
        .context("release database-dependent producer")?;
    provider_query_finished_receipt
        .await
        .context("producer query completed before engine finalization")?;
    engine_finalizer_started_receipt
        .await
        .context("engine finalizer starts after producer acknowledgement")?;

    let shutdown_report = shutdown_waiter
        .await
        .context("join supervisor shutdown")?;
    anyhow::ensure!(shutdown_report.released, "supervisor did not acknowledge closure");

    let reopened = surrealdb::engine::any::connect(&endpoint)
        .await
        .context("one immediate reopen after acknowledged embedded-engine closure")?;
    reopened
        .shutdown()
        .await
        .context("close immediately reopened embedded engine")?;
    Ok(())
}

#[tokio::test]
async fn host_owned_surreal_client_remains_usable_after_uar_scope_release() -> Result<()> {
    let directory = tempfile::tempdir().context("create borrowed-store fixture directory")?;
    let endpoint = format!("surrealkv://{}", directory.path().join("host.db").display());
    let host_client = surrealdb::engine::any::connect(&endpoint)
        .await
        .context("open host-owned embedded engine")?;
    host_client
        .use_ns("lifecycle-acceptance")
        .use_db("borrowed-host")
        .await
        .context("select host namespace and database")?;

    let provider = Arc::new(
        SurrealDbProvider::from_existing_client(
            host_client.clone(),
            HostSurrealBackend::SurrealKv,
        )
        .await
        .context("construct UAR borrowed persistence facade")?,
    );
    let supervisor = RuntimeSupervisor::new("lifecycle-owned-resources");
    let provider_for_finalizer = Arc::clone(&provider);
    let uar_scope = supervisor
        .register_with_finalizer("borrowed-persistence", "lifecycle-owned-resources", async move {
            let close = provider_for_finalizer.close_scope().await;
            anyhow::ensure!(
                close.ownership == universal_agent_runtime::uar::persistence::providers::surreal::PersistenceOwnership::BorrowedHost,
                "borrowed facade reported a different ownership mode"
            );
            anyhow::ensure!(close.scope_released, "borrowed UAR scope was not released");
            anyhow::ensure!(
                !close.local_connection_released && !close.engine_close_acknowledged,
                "UAR must not release a host-owned engine"
            );
            Ok(())
        })
        .await
        .context("register borrowed persistence scope")?;
    uar_scope.mark_ready().context("publish borrowed facade readiness")?;

    let facade: Arc<dyn PersistenceLayer> = Arc::new(ScopedPersistence::new(
        Arc::clone(&provider) as Arc<dyn PersistenceLayer>,
        uar_scope,
    ));
    facade
        .list_settings_types()
        .await
        .context("use the borrowed facade before its UAR scope is released")?;

    let release = supervisor.shutdown().await;
    anyhow::ensure!(release.released, "borrowed UAR scope did not close");

    host_client
        .query("RETURN 'host client remains usable';")
        .await
        .context("host client is still usable after UAR release")?
        .check()
        .context("host query succeeds after UAR release")?;
    host_client
        .shutdown()
        .await
        .context("host acknowledges its own engine release")?;
    Ok(())
}

#[tokio::test]
async fn owned_provider_close_failure_is_retained_by_the_supervisor() -> Result<()> {
    let directory = tempfile::tempdir().context("create owned-close fixture directory")?;
    let endpoint = format!("surrealkv://{}", directory.path().join("owned-close.db").display());
    let provider = Arc::new(
        SurrealDbProvider::new(
            &endpoint,
            None,
            None,
            Some("lifecycle-acceptance"),
            Some("owned-close"),
        )
        .await
        .context("open owned provider")?,
    );

    provider
        .client()
        .shutdown()
        .await
        .context("acknowledge the externally initiated engine closure")?;

    let supervisor = RuntimeSupervisor::new("lifecycle-owned-resources");
    let provider_for_finalizer = Arc::clone(&provider);
    let engine_scope = supervisor
        .register_with_finalizer("owned-close-failure", "lifecycle-owned-resources", async move {
            let close = provider_for_finalizer.close_scope().await;
            anyhow::ensure!(
                close.failure.is_none(),
                "owned provider propagated close failure: {:?}",
                close.failure
            );
            Ok(())
        })
        .await
        .context("register owned provider finalizer")?;
    engine_scope.mark_ready().context("publish owned provider readiness")?;

    let report = supervisor.shutdown().await;
    anyhow::ensure!(
        !report.released,
        "supervisor fabricated successful release after owned provider close failure"
    );
    anyhow::ensure!(
        report.failures.iter().any(|failure| failure.contains("owned provider propagated close failure")),
        "supervisor report did not retain the provider close failure: {:?}",
        report.failures
    );
    let resource_report = supervisor
        .resource_report("owned-close-failure")
        .await
        .context("read retained owned-close resource report")?
        .context("owned-close scope produced a terminal report")?;
    anyhow::ensure!(
        !resource_report.released && !resource_report.failures.is_empty(),
        "resource report did not retain its failed physical close"
    );
    Ok(())
}
