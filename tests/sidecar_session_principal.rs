//! Contract tests for the OpenSpec change `sidecar-session-principal`
//! (tasks.md 1.1–1.9 and 1.20). Every test enters through a real process: the
//! `uar-sidecar` binary launched with a launch token, or the standalone
//! `universal-agent-runtime` binary.

#[path = "support/principal_host.rs"]
mod principal_host;
#[path = "support/sidecar_process.rs"]
mod sidecar_process;
#[path = "integration/live/stub_llm.rs"]
mod stub_llm;

#[path = "sidecar_session_principal/admission.rs"]
mod admission;
#[path = "sidecar_session_principal/isolation.rs"]
mod isolation;
