//! Database-backed skill storage provider.
//!
//! Persists skills in SQLite via the existing persistence layer.

use super::{SkillStorageProvider, StorageProviderKind};
use crate::uar::domain::skills::Skill;
use crate::uar::persistence::PersistenceLayer;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::info;

/// SQLite-backed skill storage using the persistence layer.
#[derive(Debug)]
pub struct DatabaseStorageProvider {
    id: String,
    name: String,
    persistence: Arc<dyn PersistenceLayer>,
    enabled: bool,
}

impl DatabaseStorageProvider {
    /// Create a new database provider.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        persistence: Arc<dyn PersistenceLayer>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            persistence,
            enabled: true,
        }
    }
}

#[async_trait]
impl SkillStorageProvider for DatabaseStorageProvider {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> StorageProviderKind {
        StorageProviderKind::Database
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn list_skills(&self) -> anyhow::Result<Vec<Skill>> {
        // Delegate to persistence layer's skill listing
        self.persistence.list_skills().await
    }

    async fn refresh(&self) -> anyhow::Result<Vec<Skill>> {
        // For database, refresh is the same as listing
        self.list_skills().await
    }

    async fn save_skill(&self, skill: &Skill) -> anyhow::Result<()> {
        // Save skill without embedding (storage only)
        self.persistence.save_skill(skill, &Vec::new()).await?;
        info!("Saved skill to database: {}", skill.skill_id);
        Ok(())
    }

    async fn delete_skill(&self, id: &str) -> anyhow::Result<()> {
        self.persistence.delete_skill(id).await
    }
}
