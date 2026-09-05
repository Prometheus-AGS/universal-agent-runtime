//! Sandboxed code execution for MCP tools and agent-generated code.
//!
//! Provides Wasmtime or remote HTTP execution backends. (A microsandbox/
//! libkrun runner existed as an optional feature but was removed in
//! re-remediate-stale-rustsec: it never compiled — its test called a
//! nonexistent API — and pinned vulnerable hickory-proto into Cargo.lock.)

pub mod bindings;
pub mod execution;
pub mod mcp_tools;
pub mod platform;
pub mod runner;
pub mod session_manager;
pub mod types;

pub mod remote_runner;
pub mod wasmtime_runner;

pub use runner::SandboxRunner;
pub use session_manager::SessionManager;
pub use types::*;
