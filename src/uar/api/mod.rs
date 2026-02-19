pub mod a2a;
pub mod actors;
pub mod adapters;
pub mod auth;
pub mod compiler;
pub mod discovery;
pub mod ingest;
pub mod knowledge;
pub mod memory;
pub mod openai;
pub mod providers;
pub mod routes;
pub mod settings;
pub mod skills;
pub mod sse;
pub mod upload;

use axum::Router;

use crate::uar::runtime::manager::RunManager;
use std::sync::Arc;

pub fn router() -> Router<Arc<RunManager>> {
    // In M3 we will build the router in routes.rs and just return it here
    routes::build_router()
}
