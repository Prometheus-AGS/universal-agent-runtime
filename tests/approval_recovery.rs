//! Real runtime processes, verified ingress, durable stores, and canonical effects.
#![cfg(all(feature = "server", feature = "surreal-backend", unix))]

#[path = "support/approval_recovery_process.rs"]
mod process;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn durable_approval_recovery_production_boundaries() {
    if process::run_child().await { return; }
    process::qualify().await;
}
