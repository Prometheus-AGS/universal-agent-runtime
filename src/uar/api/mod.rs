#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod a2a;
#[cfg(feature = "server")]
pub mod acp;
#[cfg(feature = "server")]
pub mod actors;
pub mod adapters;
#[cfg(feature = "server")]
pub mod auth;
#[cfg(feature = "server")]
pub mod compiler;
#[cfg(feature = "server")]
pub mod credentials;
#[cfg(feature = "server")]
pub mod discovery;
#[cfg(feature = "server")]
pub mod ingest;
#[cfg(feature = "server")]
pub mod knowledge;
#[cfg(feature = "server")]
pub mod live;
#[cfg(feature = "server")]
pub mod mcp_admin;
#[cfg(feature = "server")]
pub mod memory;
#[cfg(feature = "server")]
pub mod memory_admin;
#[cfg(feature = "server")]
pub mod openai;
#[cfg(feature = "server")]
pub mod openapi;
#[cfg(feature = "server")]
pub mod providers;
#[cfg(feature = "server")]
pub mod routes;
#[cfg(feature = "server")]
pub mod settings;
#[cfg(feature = "server")]
pub mod skills;
#[cfg(feature = "server")]
pub mod sse;
#[cfg(feature = "server")]
pub mod upload;
#[cfg(feature = "server")]
pub mod user_settings;

#[cfg(feature = "server")]
use axum::Router;

#[cfg(feature = "server")]
use crate::uar::runtime::manager::RunManager;
#[cfg(feature = "server")]
use std::sync::Arc;

#[cfg(feature = "server")]
pub fn router() -> Router<Arc<RunManager>> {
    // In M3 we will build the router in routes.rs and just return it here
    routes::build_router()
}
