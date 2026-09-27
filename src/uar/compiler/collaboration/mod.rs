//! Compiler and immutable catalog for collaboration packages.

mod bindings;
mod export;
mod grants;
mod runtime_semantics;
mod service;
mod storage;
mod validation;

pub use export::{
    CanonicalPackageExport, DeploymentBindingTemplateExport, PackageExportOutcome,
    PackageExportRequest, PackageExportTarget,
};
pub use grants::{GrantCommandRequest, GrantInstallResponse};
pub use service::{BoundAgentRun, CollaborationCatalogService, CollaborationError};
#[cfg(feature = "postgres-backend")]
pub use storage::PostgresCollaborationStorage;
#[cfg(feature = "surreal-backend")]
pub use storage::SurrealCollaborationStorage;
pub use storage::{CollaborationStorage, InMemoryCollaborationStorage};
