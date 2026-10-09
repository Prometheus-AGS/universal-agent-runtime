//! Metadata-only skill selection and read-through reconciliation.

use super::{SkillMatchingSnapshot, SkillService};
use crate::uar::domain::skills::{Skill, SkillScope};
use crate::uar::runtime::skills::storage::StorageProviderKind;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Fixed errors for selection operations, which may have a committed prefix.
#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum SkillSelectionError {
    /// Governance refused the mutation before its write lock was acquired.
    #[error("skill_selection_mutation_denied")]
    Denied,
    /// The requested single skill is not registered.
    #[error("skill_selection_not_found")]
    NotFound,
    /// Registered metadata has no corresponding durable record.
    #[error("skill_selection_storage_missing; partial outcome possible; read to reconcile")]
    StorageMissing,
    /// Storage did not establish a known outcome.
    #[error("skill_selection_outcome_unknown; read to reconcile; do not replay automatically")]
    StoreUnavailable,
    /// Selection was published but its separate filesystem write failed.
    #[error("skill_selection_filesystem_partial; registry metadata published")]
    FilesystemPartial,
}

impl SkillService {
    /// Return registered skills after reconciling only their durable selection fields.
    pub async fn get_skills_checked(&self) -> Result<Vec<Skill>, SkillSelectionError> {
        let mut registry = self.registry.write().await;
        registry.reconcile_selection().await?;
        Ok(registry.list())
    }

    /// Return selected IDs; explicit registered overrides take priority over legacy IDs.
    pub async fn get_agent_skill_ids_checked(
        &self,
        agent_id: &str,
    ) -> Result<Vec<String>, SkillSelectionError> {
        let mut registry = self.registry.write().await;
        registry.reconcile_selection().await?;
        let mut ids = self
            .agent_skills
            .read()
            .await
            .get(agent_id)
            .cloned()
            .unwrap_or_default();
        for skill in registry.list() {
            if let Some(config) = skill
                .scoped_config
                .iter()
                .find(|config| matches!(&config.scope, SkillScope::Agent(id) if id == agent_id))
            {
                ids.retain(|id| id != &skill.skill_id);
                if config.enabled {
                    ids.push(skill.skill_id);
                }
            }
        }
        Ok(ids)
    }

    /// Change one scope without re-embedding its unchanged document.
    pub async fn set_scoped_enabled_checked(
        &self,
        id: &str,
        scope: SkillScope,
        enabled: bool,
    ) -> Result<(), SkillSelectionError> {
        self.ensure_mutation_allowed(id)
            .await
            .map_err(|_| SkillSelectionError::Denied)?;
        let mut registry = self.registry.write().await;
        registry.reconcile_selection().await?;
        let updated = registry.update_selection(id, scope, enabled).await?;
        self.persist_selection_to_filesystem(&updated).await
    }

    /// Replace selection sequentially; a failure does not roll back its confirmed prefix.
    pub async fn set_agent_skills_checked(
        &self,
        agent_id: &str,
        skill_ids: Vec<String>,
    ) -> Result<(), SkillSelectionError> {
        let ids = self
            .registry
            .read()
            .await
            .list()
            .into_iter()
            .map(|skill| skill.skill_id)
            .collect::<Vec<_>>();
        for id in &ids {
            self.ensure_mutation_allowed(id)
                .await
                .map_err(|_| SkillSelectionError::Denied)?;
        }
        let selected = skill_ids.iter().cloned().collect::<HashSet<_>>();
        let mut registry = self.registry.write().await;
        registry.reconcile_selection().await?;
        for id in ids {
            let updated = registry
                .update_selection(
                    &id,
                    SkillScope::Agent(agent_id.to_string()),
                    selected.contains(&id),
                )
                .await?;
            self.persist_selection_to_filesystem(&updated).await?;
        }
        // This is only the legacy fallback for IDs without an explicit loaded override.
        self.agent_skills
            .write()
            .await
            .insert(agent_id.to_string(), skill_ids);
        Ok(())
    }

    /// Add a loaded override or retain a deferred ID for a skill loaded later.
    pub async fn add_skill_to_agent_checked(
        &self,
        agent_id: &str,
        skill_id: &str,
    ) -> Result<(), SkillSelectionError> {
        self.change_agent_skill_checked(agent_id, skill_id, true)
            .await
    }

    /// Remove a loaded override or a deferred ID.
    pub async fn remove_skill_from_agent_checked(
        &self,
        agent_id: &str,
        skill_id: &str,
    ) -> Result<(), SkillSelectionError> {
        self.change_agent_skill_checked(agent_id, skill_id, false)
            .await
    }

    async fn change_agent_skill_checked(
        &self,
        agent_id: &str,
        skill_id: &str,
        enabled: bool,
    ) -> Result<(), SkillSelectionError> {
        self.ensure_mutation_allowed(skill_id)
            .await
            .map_err(|_| SkillSelectionError::Denied)?;
        let mut registry = self.registry.write().await;
        registry.reconcile_selection().await?;
        if registry.get(skill_id).is_some() {
            let updated = registry
                .update_selection(skill_id, SkillScope::Agent(agent_id.to_string()), enabled)
                .await?;
            self.persist_selection_to_filesystem(&updated).await?;
        }
        let mut bindings = self.agent_skills.write().await;
        let ids = bindings.entry(agent_id.to_string()).or_default();
        ids.retain(|id| id != skill_id);
        if enabled {
            ids.push(skill_id.to_string());
        }
        Ok(())
    }

    async fn persist_selection_to_filesystem(
        &self,
        skill: &Skill,
    ) -> Result<(), SkillSelectionError> {
        if skill.provider_id != "api" {
            return Ok(());
        }
        for provider in &self.providers {
            if provider.kind() == StorageProviderKind::Filesystem {
                provider
                    .save_skill(skill)
                    .await
                    .map_err(|_| SkillSelectionError::FilesystemPartial)?;
                break;
            }
        }
        Ok(())
    }

    /// Capture only reconciled selection state for a fresh run.
    pub(crate) async fn matching_snapshot(
        &self,
    ) -> Result<SkillMatchingSnapshot, SkillSelectionError> {
        let mut registry = self.registry.write().await;
        registry.reconcile_selection().await?;
        let agent_skills = self.agent_skills.read().await.clone();
        let config = self.matching_config.read().await.clone();
        Ok(SkillMatchingSnapshot {
            registry: Arc::new(RwLock::new(registry.clone())),
            config,
            agent_skills,
        })
    }
}
