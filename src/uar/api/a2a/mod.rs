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
pub mod discovery;
pub mod handler;
pub mod registry;
pub mod registry_postgres;
pub mod task_store;
pub mod types;

use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

pub use discovery::{DiscoveryApiState, build_discovery_router};
pub use handler::A2AState;
pub use registry::{
    AgentInfo, AgentRegistry, ExternalSkill, InMemoryAgentRegistry, SurrealAgentRegistry,
};
pub use registry_postgres::PostgresAgentRegistry;
pub use task_store::TaskStore;

/// Build the A2A router.
///
/// Returns two routers that should be mounted separately:
/// - `rpc_router` → mount at `/a2a/compiler`
/// - `well_known_router` → mount at `/.well-known`
pub fn build_rpc_router() -> Router<Arc<A2AState>> {
    Router::new().route("/", post(handler::handle_rpc))
}

pub fn build_well_known_router() -> Router<Arc<A2AState>> {
    Router::new().route("/agent.json", get(handler::handle_agent_card))
}
