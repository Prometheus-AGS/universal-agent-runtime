//! A2UI design-system bridge — component catalog, design-token import, and
//! semantic-search embeddings for A2UI surfaces.
//!
//! Migrated from `flint-forge`:
//! - `flint-forge/crates/fdb-app/src/a2ui/` (domain types + the `DESIGN.md`
//!   parser) → [`types`] + [`design_md_parser`], ported near-verbatim (pure
//!   Rust, no database dependency).
//! - `flint-forge/migrations/0009_flint_a2ui_design_systems.sql` (design
//!   system provenance columns + `component_overrides` table) → the
//!   Postgres migration under `migrations/`, and the SurrealDB schema in
//!   `migrations/surrealdb/schema.surql`, both consumed via [`store`].
//! - `flint-forge/crates/fdb-gateway/src/a2ui_embedder.rs` (Postgres
//!   `LISTEN`/`NOTIFY` embedding worker) → [`embedder`], re-architected
//!   around UAR's `EmbeddingBackend` trait and `DesignSystemStore` instead
//!   of a Postgres-specific channel listener (see `embedder`'s module docs
//!   for the full rationale).
//!
//! [`import`] is new UAR-side glue connecting the parser to the store,
//! since flint-forge's own "apply a parsed DESIGN.md" use case lives in its
//! interface layer (out of scope for this migration).
//!
//! See `openspec/changes/a2ui-migrate-design-systems-embedder-from-flint-forge/`
//! for the full audit of what was and wasn't migrated, and why.

pub mod design_md_parser;
pub mod embedder;
pub mod import;
pub mod store;
pub mod types;

pub use import::{ImportError, ImportReport, import_design_md, import_w3c_tokens};
#[cfg(feature = "sqlx")]
pub use store::PostgresDesignSystemStore;
#[cfg(feature = "surreal-backend")]
pub use store::SurrealDesignSystemStore;
pub use store::{DesignSystemStore, InMemoryDesignSystemStore, SharedDesignSystemStore};
pub use types::{
    Component, ComponentOverrideRecord, DesignSystem, DesignToken, DesignTokenMap, Renderers,
    ResolvedComponent, SourceFormat,
};
