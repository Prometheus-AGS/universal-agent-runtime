//! Skills system module.
//!
//! Provides a pluggable skills engine with:
//! - Multiple storage providers (filesystem, database, built-in)
//! - Configurable matching algorithms (keyword, embedding, LLM, hybrid)
//! - Per-agent skill bindings
//! - SKILL.md parsing with MCP server configuration

pub mod builtin_loader;
pub mod pack_detection;
pub mod provenance;
pub mod registry;
pub mod service;
pub mod storage;
pub mod update_check;
#[cfg(feature = "wasm-runtime")]
pub mod wasm_runtime;
pub mod watcher;

pub use registry::SkillRegistry;
pub use service::{SkillMatchingAlgorithm, SkillMatchingConfig, SkillService};
pub use storage::{SkillStorageProvider, StorageProviderKind};

// ---------------------------------------------------------------------------
// A note on visibility (change-uhe-009, R4)
//
// Everything above is `pub`, and the path to it is `pub` at every level:
// `uar` -> `runtime` -> `skills`. The plan for this change said "keep
// `uar::runtime::skills` internals private". Measured, that is not a change we
// can make here:
//
//   - 16 files under `src/` use these types directly
//   - 6 integration tests in `tests/` import them
//
// Narrowing the visibility would be a breaking change to a surface that
// external code may already depend on, and it would fail the build immediately.
// That is a deliberate deprecation with its own migration, not a task inside an
// SDK change.
//
// What change-uhe-009 delivers instead is the *seam* that makes such a
// narrowing possible later: `crate::skills_api::SkillsApi` is the supported
// embedder surface, reached via `EmbeddedRuntime::skills()`. New embedding code
// should use that. These modules stay public for existing consumers, but they
// are no longer the recommended entry point, and nothing new should be built
// against them.
