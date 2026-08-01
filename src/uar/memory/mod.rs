//! # Agent Memory System
//!
//! Provides persistent, multi-scoped memory for UAR agents using the
//! [`surreal-memory`](https://github.com/Prometheus-AGS/surreal-memory-server) library.
//!
//! ## Architecture
//!
//! ```text
//!  Request (JWT optional)
//!       │
//!       ▼
//!  auth_middleware → UserContext { user_id, claims }
//!       │
//!       ├──► [pre-call] context_builder::build_context()
//!       │           → hybrid_search over Session→User→Agent→Global
//!       │           → inject ranked block into system prompt
//!       │
//!       ├──► LLM Orchestrator
//!       │
//!       └──► [post-stream] auto_capture::capture_from_stream_end()
//!                   → add_memories_from_conversation()
//! ```
//!
//! ## Scoping
//!
//! | Scope   | Who can write    | Who can read   |
//! |---------|-----------------|----------------|
//! | Global  | System / admin  | Everyone       |
//! | Agent   | This agent      | This agent     |
//! | User    | JWT-auth user   | Same user only |
//! | Session | Any request     | Same session   |
//! | Task    | TaskStream API  | Same task      |
//!
//! ## MCP Exposure
//!
//! All memory capabilities are exposed as MCP tools via [`mcp_server::MemoryMcpServer`].
//! The server is registered as a native tool with [`crate::mcp::registry::McpRegistry`]
//! and optionally as an HTTP endpoint at `/mcp/memory`.

pub mod auto_capture;
pub mod background;
pub mod context_builder;
#[cfg(feature = "server")]
pub mod mcp_server;
pub mod scopes;
pub mod service;
pub mod workflow_mirror;

pub use service::MemoryService;
pub use surreal_memory::{Memory, MemoryHistory, MemoryScope, MemoryType};
