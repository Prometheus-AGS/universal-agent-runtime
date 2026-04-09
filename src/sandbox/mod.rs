//! Sandboxed code execution for MCP tools and agent-generated code.
//!
//! Provides hardware VM isolation via microsandbox (libkrun) with fallback
//! to Wasmtime or remote HTTP execution.

pub mod mcp_tools;
pub mod platform;
pub mod runner;
pub mod session_manager;
pub mod types;

#[cfg(feature = "sandbox-microsandbox")]
pub mod microsandbox_runner;
pub mod remote_runner;
pub mod wasmtime_runner;

pub use runner::SandboxRunner;
pub use session_manager::SessionManager;
pub use types::*;
