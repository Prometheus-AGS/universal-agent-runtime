//! Skill storage provider trait and implementations.
//!
//! Storage providers abstract where skills are loaded from:
//! - Filesystem (SKILL.md files in directories)
//! - Database (SQLite-backed)
//! - Built-in (bundled with the runtime)

pub mod builtin;
pub mod database;
pub mod filesystem;

use crate::uar::domain::skills::Skill;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Identifies the kind of storage provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageProviderKind {
    /// Local filesystem directories containing SKILL.md files
    Filesystem,
    /// SQLite-backed persistent storage
    Database,
    /// Skills bundled with the runtime binary
    BuiltIn,
}

/// Abstract trait for skill storage/discovery.
///
/// Implementations discover skills from different sources.
/// All methods are async to support both I/O-bound and compute-bound providers.
#[async_trait]
pub trait SkillStorageProvider: Send + Sync + std::fmt::Debug {
    /// Unique identifier for this provider instance.
    fn id(&self) -> &str;

    /// Human-readable name of the provider.
    fn name(&self) -> &str;

    /// The kind of storage this provider uses.
    fn kind(&self) -> StorageProviderKind;

    /// Whether this provider is enabled.
    fn is_enabled(&self) -> bool;

    /// List all skills available from this provider.
    async fn list_skills(&self) -> anyhow::Result<Vec<Skill>>;

    /// Refresh the skill list (re-scan directories, re-query database, etc.).
    async fn refresh(&self) -> anyhow::Result<Vec<Skill>>;

    /// Save or update a skill in this provider.
    async fn save_skill(&self, skill: &Skill) -> anyhow::Result<()>;

    /// Delete a skill by ID.
    async fn delete_skill(&self, id: &str) -> anyhow::Result<()>;
}

pub use builtin::BuiltInStorageProvider;
pub use database::DatabaseStorageProvider;
pub use filesystem::FilesystemStorageProvider;
