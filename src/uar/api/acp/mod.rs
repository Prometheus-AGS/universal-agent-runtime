//! ACP (Agent Communication Protocol) server module.
//!
//! Exposes a JSON-RPC 2.0 endpoint at the configured path (default: `/acp`)
//! for agent introspection, session management, and streaming run execution.
//!
//! Enable via config: `acp.enabled = true`
//! or CLI: `--acp-enabled` / env: `UAR_ACP__ENABLED=true`

pub mod handler;
pub mod routes;
pub mod types;

pub use handler::AcpSessionStore;
pub use routes::AcpRouter;
