//! Contract tests for run and session retention in the OpenSpec change
//! `sidecar-session-principal` (tasks.md 1.10–1.14 and 1.16–1.19). Every test
//! enters through a real `uar-sidecar` or standalone process; retained state
//! is observed through run routes, model requests and the `/metrics` gauges.

#[path = "support/principal_host.rs"]
mod principal_host;
#[path = "support/sidecar_process.rs"]
mod sidecar_process;
#[path = "integration/live/stub_llm.rs"]
mod stub_llm;

#[path = "run_retention/runs.rs"]
mod runs;
#[path = "run_retention/sessions.rs"]
mod sessions;
