//! A2A transport adapters over the persisted agent-thread host.
//!
//! Mounts at:
//! - `POST /a2a/compiler` — JSON-RPC 2.0 dispatcher
//! - `POST /a2a/agents/{agent_id}` — dispatcher for a registered artifact
//! - `GET /.well-known/agent.json` — AgentCard
//!
//! ## Task mapping
//!
//! | A2A Concept | UAR Mapping |
//! |-------------|-------------|
//! | `context_id` | Owner/artifact-qualified conversation correlation |
//! | `message/send` (first) | Creates an exact actor binding and submits a root turn |
//! | `message/send` (subsequent) | Submits another turn after the previous receipt |
//! | `tasks/get` | Projects the persisted thread and exact invocation receipt |
//! | `tasks/cancel` | Cancels and joins the bound actor, then settles persistence |
//!
//! `metadata.cleanup_unconfirmed` distinguishes failed execution from confirmed
//! resource cleanup. Neither transport may treat that flag as successful stop.

pub mod agent_card;
pub mod client;
pub mod contract;
#[cfg(feature = "server")]
pub mod discovery;
// gRPC transport requires proto compilation via tonic-build.
// Enable once tonic-build prost integration is configured.
#[cfg(feature = "a2a-transport")]
pub mod grpc;
#[cfg(feature = "server")]
pub mod handler;
pub mod peer;
pub mod registry;
#[cfg(feature = "postgres-backend")]
pub mod registry_postgres;
pub mod task_execution;
pub mod task_store;
#[cfg(feature = "server")]
pub mod thread_service;
pub mod types;

#[cfg(feature = "server")]
use std::sync::Arc;

#[cfg(feature = "server")]
use axum::{
    Router,
    routing::{get, post},
};

pub use client::A2AClient;
#[cfg(feature = "server")]
pub use discovery::{DiscoveryApiState, build_discovery_router};
#[cfg(feature = "server")]
pub use handler::A2AState;
#[cfg(feature = "surreal-backend")]
pub use registry::SurrealAgentRegistry;
pub use registry::{AgentInfo, AgentRegistry, ExternalSkill, InMemoryAgentRegistry};
#[cfg(feature = "postgres-backend")]
pub use registry_postgres::PostgresAgentRegistry;
pub use task_store::TaskStore;

/// Build the A2A router.
///
/// Returns two routers that should be mounted separately:
/// - `rpc_router` → mount at `/a2a/compiler`
/// - `well_known_router` → mount at `/.well-known`
#[cfg(feature = "server")]
pub fn build_rpc_router() -> Router<Arc<A2AState>> {
    Router::new().route("/", post(handler::handle_rpc))
}

#[cfg(feature = "server")]
pub fn build_well_known_router() -> Router<Arc<A2AState>> {
    Router::new().route("/agent.json", get(handler::handle_agent_card))
}
