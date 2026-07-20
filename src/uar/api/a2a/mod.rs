//! A2A Protocol RC v1.0 endpoint for the UAR Compiler Agent.
//!
//! Mounts at:
//! - `POST /a2a/compiler` — JSON-RPC 2.0 dispatcher
//! - `GET /.well-known/agent.json` — AgentCard
//!
//! ## A2A ↔ Compiler Session Mapping
//!
//! | A2A Concept | UAR Mapping |
//! |-------------|-------------|
//! | `contextId` | `CompilerSession.id` |
//! | `message/send` (first) | Creates session + task |
//! | `message/send` (subsequent) | Adds turn to session |
//! | `tasks/get` | Returns session status + completeness |
//! | `tasks/cancel` | Cancels session |
//! | `Artifact` in response | Compiled descriptor.json when `Completed` |

pub mod agent_card;
pub mod client;
#[cfg(feature = "server")]
pub mod discovery;
// gRPC transport requires proto compilation via tonic-build.
// Enable once tonic-build prost integration is configured.
#[cfg(feature = "a2a-transport")]
pub mod grpc;
#[cfg(feature = "server")]
pub mod handler;
pub mod registry;
#[cfg(feature = "postgres-backend")]
pub mod registry_postgres;
pub mod task_store;
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
