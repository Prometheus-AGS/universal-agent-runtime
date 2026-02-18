//! Built-in skill storage provider.
//!
//! Provides skills that are bundled with the runtime binary.

use super::{SkillStorageProvider, StorageProviderKind};
use crate::uar::domain::skills::Skill;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Provides skills that are bundled with the runtime.
#[derive(Debug)]
pub struct BuiltInStorageProvider {
    id: String,
    name: String,
    skills: Arc<RwLock<Vec<Skill>>>,
    enabled: bool,
}

impl BuiltInStorageProvider {
    /// Create a new built-in provider with pre-registered skills.
    pub fn new(id: impl Into<String>, name: impl Into<String>, skills: Vec<Skill>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            skills: Arc::new(RwLock::new(skills)),
            enabled: true,
        }
    }

    /// Create an empty built-in provider.
    pub fn empty(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self::new(id, name, Vec::new())
    }

    /// Add a skill to the built-in set.
    pub async fn add_skill(&self, skill: Skill) {
        self.skills.write().await.push(skill);
    }
}

#[async_trait]
impl SkillStorageProvider for BuiltInStorageProvider {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> StorageProviderKind {
        StorageProviderKind::BuiltIn
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn list_skills(&self) -> anyhow::Result<Vec<Skill>> {
        Ok(self.skills.read().await.clone())
    }

    async fn refresh(&self) -> anyhow::Result<Vec<Skill>> {
        self.list_skills().await
    }

    async fn save_skill(&self, skill: &Skill) -> anyhow::Result<()> {
        self.add_skill(skill.clone()).await;
        Ok(())
    }

    async fn delete_skill(&self, id: &str) -> anyhow::Result<()> {
        let mut skills = self.skills.write().await;
        skills.retain(|s| s.skill_id != id);
        Ok(())
    }
}
