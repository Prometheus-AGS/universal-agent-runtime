//! Skills system module.
//!
//! Provides a pluggable skills engine with:
//! - Multiple storage providers (filesystem, database, built-in)
//! - Configurable matching algorithms (keyword, embedding, LLM, hybrid)
//! - Per-agent skill bindings
//! - SKILL.md parsing with MCP server configuration

pub mod registry;
pub mod service;
pub mod storage;

pub use registry::SkillRegistry;
pub use service::{SkillMatchingAlgorithm, SkillMatchingConfig, SkillService};
pub use storage::{SkillStorageProvider, StorageProviderKind};
