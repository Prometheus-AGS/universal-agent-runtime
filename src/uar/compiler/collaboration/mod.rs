//! Compiler and immutable catalog for collaboration packages.

mod bindings;
mod service;
mod storage;
mod validation;

pub use service::{CollaborationCatalogService, CollaborationError};
pub use storage::{CollaborationStorage, InMemoryCollaborationStorage};
#[cfg(feature = "postgres-backend")]
pub use storage::PostgresCollaborationStorage;
#[cfg(feature = "surreal-backend")]
pub use storage::SurrealCollaborationStorage;
