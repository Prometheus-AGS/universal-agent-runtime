//! Actual server processes, authenticated API, protocol peers and durable databases.
//! Controlled protocol success does not qualify genuine provider inference.
#![cfg(all(feature = "server", feature = "surreal-backend", unix))]

#[path = "support/fresh_process_resume_process.rs"]
mod process;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn durable_fresh_process_resume_production_boundaries() {
    if process::run_readback_child().await { return; }
    process::qualify().await;
}
