//! Skill service — central coordinator for skills.
//!
//! Mirrors cherry-studio's `SkillService` pattern:
//! - Aggregates skills from multiple storage providers
//! - Configurable matching algorithms
//! - Per-agent skill bindings
//! - Script execution (sandboxed)

use super::registry::SkillRegistry;
use super::storage::{SkillStorageProvider, StorageProviderKind};
use crate::uar::domain::skills::{Skill, SkillCandidate, SkillMatchResult, SkillScope};
use crate::uar::persistence::PersistenceLayer;
use crate::uar::runtime::matching::vector::VectorMatcher;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Supported matching algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillMatchingAlgorithm {
    /// Simple keyword/regex matching (fast, no model needed)
    Keyword,
    /// Embedding-based similarity using Burn-rs + USearch
    Embedding,
    /// LLM-based intent classification
    Llm,
    /// Weighted combination of keyword + embedding
    Hybrid,
    /// On-device embedding without API calls
    LocalEmbedding,
}

impl Default for SkillMatchingAlgorithm {
    fn default() -> Self {
        Self::Keyword
    }
}

/// Configuration for skill matching behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMatchingConfig {
    /// Which algorithm to use for matching
    #[serde(default)]
    pub algorithm: SkillMatchingAlgorithm,
    /// Minimum score threshold for a match (0.0–1.0)
    #[serde(default = "default_threshold")]
    pub threshold: f32,
    /// Required separation between the two strongest candidates.
    #[serde(default = "default_margin_threshold")]
    pub margin_threshold: f32,
    /// Maximum number of matched skills to return
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    /// Model name for LLM/embedding matchers (if applicable)
    #[serde(default)]
    pub model_name: Option<String>,
}

fn default_threshold() -> f32 {
    0.5
}

fn default_top_k() -> usize {
    3
}

fn default_margin_threshold() -> f32 {
    0.05
}

impl Default for SkillMatchingConfig {
    fn default() -> Self {
        Self {
            algorithm: SkillMatchingAlgorithm::default(),
            threshold: default_threshold(),
            margin_threshold: default_margin_threshold(),
            top_k: default_top_k(),
            model_name: None,
        }
    }
}

/// Patch fields for updating an existing skill.
#[derive(Debug, Clone, Default)]
pub struct SkillUpdate {
    pub version: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub triggers: Option<crate::uar::domain::skills::SkillTriggers>,
    pub prompt_overlay: Option<String>,
    pub preferred_tools: Option<Vec<String>>,
    pub enabled: Option<bool>,
    pub execution_config: Option<crate::uar::domain::skills::SkillExecutionConfig>,
}

/// Outcome of one standard-directory startup reconciliation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct AgentSkillReconciliationReport {
    pub discovered: usize,
    pub added: usize,
    pub updated: usize,
    pub unchanged: usize,
}

/// Central skill service coordinating storage + matching.
///
/// When a [`GovernanceEngine`] is attached, all skill mutations (updates,
/// prompt overlays) are gated through Cedar policy evaluation before
/// being applied. This enforces the Skill Mutation PEP for the
/// self-learning pipeline.
pub struct SkillService {
    /// In-memory skill index
    registry: Arc<RwLock<SkillRegistry>>,
    /// Registered storage providers
    providers: Vec<Arc<dyn SkillStorageProvider>>,
    /// Current matching configuration
    matching_config: RwLock<SkillMatchingConfig>,
    /// Compatibility view for agent bindings whose skill is not loaded yet.
    /// Loaded skills also receive durable [`SkillScope::Agent`] overrides.
    agent_skills: RwLock<HashMap<String, Vec<String>>>,
    /// Optional governance engine for Cedar policy enforcement on skill mutations.
    governance: Option<Arc<crate::uar::governance::engine::GovernanceEngine>>,
}

/// Read-only matching inputs captured for one run. No storage-provider or
/// skill-mutation API is carried into this view.
pub(crate) struct SkillMatchingSnapshot {
    pub(crate) registry: Arc<RwLock<SkillRegistry>>,
    pub(crate) config: SkillMatchingConfig,
    agent_skills: HashMap<String, Vec<String>>,
}

impl SkillMatchingSnapshot {
    pub(crate) async fn match_skills_scoped(
        &self,
        query: &str,
        agent_id: Option<&str>,
        conversation_id: Option<&str>,
    ) -> SkillMatchResult {
        let legacy_bindings = agent_id
            .and_then(|id| self.agent_skills.get(id))
            .filter(|ids| !ids.is_empty())
            .cloned();
        let registry = self.registry.read().await;
        SkillService::match_in_registry(
            query,
            agent_id,
            conversation_id,
            legacy_bindings.as_ref(),
            &registry,
            &self.config,
        )
        .await
    }
}

impl std::fmt::Debug for SkillService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkillService")
            .field("providers", &self.providers.len())
            .finish()
    }
}

impl SkillService {
    /// Create a new skill service.
    pub fn new(
        persistence: Option<Arc<dyn PersistenceLayer>>,
        vector_matcher: Option<Arc<VectorMatcher>>,
    ) -> Self {
        Self {
            registry: Arc::new(RwLock::new(SkillRegistry::new(persistence, vector_matcher))),
            providers: Vec::new(),
            matching_config: RwLock::new(SkillMatchingConfig::default()),
            agent_skills: RwLock::new(HashMap::new()),
            governance: None,
        }
    }

    /// Attach a governance engine for Cedar policy enforcement on skill mutations.
    ///
    /// When attached, `update_skill` checks `is_skill_mutation_allowed` before
    /// applying changes. Without governance, all mutations are permitted.
    pub fn with_governance(
        mut self,
        engine: Arc<crate::uar::governance::engine::GovernanceEngine>,
    ) -> Self {
        self.governance = Some(engine);
        self
    }

    /// Add a storage provider.
    pub fn add_provider(&mut self, provider: Arc<dyn SkillStorageProvider>) {
        info!(
            "Added skill storage provider: {} ({})",
            provider.name(),
            provider.id()
        );
        self.providers.push(provider);
    }

    /// Register a batch of `Skill { kind = Manifest, origin = Builtin }` into
    /// the registry, persisting each one when a database is configured.
    ///
    /// Used at startup by the builtin-loader. Builtins are **not** written back
    /// through the storage providers in [`Self::initialize`] — that would make
    /// the pack a provider — but they DO reach the persistence layer, which is
    /// what the admin UI, the REST API, and embedded hosts read.
    ///
    /// This doc comment previously claimed "these skills are not persisted via
    /// storage providers", which read as *not persisted at all* and matched the
    /// behaviour: nothing reached the database. See
    /// [`super::registry::SkillRegistry::register`] for the defect.
    pub async fn register_builtins(&self, skills: Vec<crate::uar::domain::skills::Skill>) {
        let mut registry = self.registry.write().await;
        let count = skills.len();
        registry.register_builtins(skills).await;
        info!(count, "registered builtin skills");
    }

    /// Initialize the service by loading skills from all providers.
    pub async fn initialize(&self) -> anyhow::Result<()> {
        let mut registry = self.registry.write().await;
        registry.clear();

        for provider in &self.providers {
            if !provider.is_enabled() {
                info!("Skipping disabled provider: {}", provider.name());
                continue;
            }

            match provider.list_skills().await {
                Ok(skills) => {
                    info!(
                        "Loaded {} skills from provider '{}'",
                        skills.len(),
                        provider.name()
                    );
                    registry.register_all_loaded(skills);
                }
                Err(e) => {
                    error!(
                        "Failed to load skills from provider '{}': {:?}",
                        provider.name(),
                        e
                    );
                }
            }
        }

        info!("SkillService initialized: {} total skills", registry.len());
        Ok(())
    }

    /// Merge configuration-managed filesystem skills into durable storage.
    ///
    /// API-created files beneath the reserved `dynamic/` directory reload with
    /// `provider_id = "api"` and are therefore outside this operation.
    pub(crate) async fn reconcile_config_skills(&self) -> anyhow::Result<()> {
        let Some(config_provider) = self
            .providers
            .iter()
            .find(|provider| provider.id() == "fs-skills")
        else {
            return Ok(());
        };
        let config_skills = config_provider
            .list_skills()
            .await?
            .into_iter()
            .filter(|skill| skill.provider_id == "fs-skills")
            .collect::<Vec<_>>();
        let stored_skills = self.registry.read().await.list_persisted().await?;
        let stored_by_id = stored_skills
            .iter()
            .map(|skill| (skill.skill_id.clone(), skill))
            .collect::<HashMap<_, _>>();
        let stored_config_skill_count = stored_skills
            .iter()
            .filter(|skill| skill.provider_id == "fs-skills")
            .count();

        if config_skills.is_empty() && stored_config_skill_count > 0 {
            error!(
                stored_config_skills = stored_config_skill_count,
                "configuration skill source is empty; refusing to tombstone stored skills"
            );
            return Ok(());
        }

        let configured_ids = config_skills
            .iter()
            .map(|skill| skill.skill_id.clone())
            .collect::<HashSet<_>>();
        for mut configured in config_skills {
            if let Some(existing) = stored_by_id.get(&configured.skill_id) {
                if existing.provider_id != "fs-skills" {
                    warn!(
                        skill_id = %configured.skill_id,
                        stored_provider_id = %existing.provider_id,
                        "configuration skill conflicts with a non-configuration skill; preserving stored skill"
                    );
                    continue;
                }
                configured.enabled = existing.enabled;
                configured.scoped_config.clone_from(&existing.scoped_config);
            }
            configured.tombstoned = false;

            let changed = match stored_by_id.get(&configured.skill_id) {
                Some(existing) => {
                    serde_json::to_value(existing)? != serde_json::to_value(&configured)?
                }
                None => true,
            };
            if changed {
                self.registry
                    .write()
                    .await
                    .register_checked(configured)
                    .await?;
            }
        }

        for stored in stored_skills
            .iter()
            .filter(|skill| skill.provider_id == "fs-skills")
        {
            if configured_ids.contains(&stored.skill_id) || stored.tombstoned {
                continue;
            }
            let mut tombstoned = stored.clone();
            tombstoned.tombstoned = true;
            self.registry
                .write()
                .await
                .register_checked(tombstoned)
                .await?;
            info!(
                skill_id = %stored.skill_id,
                reason = "absent_from_configuration",
                "tombstoned configuration-managed skill"
            );
        }

        Ok(())
    }

    /// Upsert new or changed skills from the standard `~/.agents/skills`
    /// provider. Source absence never implies deletion for this provider.
    /// Reconciliation never waits for embedding inference.
    pub(crate) async fn reconcile_standard_agent_skills(
        &self,
    ) -> anyhow::Result<AgentSkillReconciliationReport> {
        let Some(agent_provider) = self
            .providers
            .iter()
            .find(|provider| provider.id() == "agent-skills")
        else {
            return Ok(AgentSkillReconciliationReport::default());
        };
        let discovered_skills = agent_provider
            .list_skills()
            .await?
            .into_iter()
            .filter(|skill| skill.provider_id == "agent-skills")
            .collect::<Vec<_>>();
        let stored_skills = self.registry.read().await.list_persisted().await?;
        let stored_by_id = stored_skills
            .iter()
            .map(|skill| (skill.skill_id.clone(), skill))
            .collect::<HashMap<_, _>>();

        let mut report = AgentSkillReconciliationReport {
            discovered: discovered_skills.len(),
            ..Default::default()
        };
        let mut changed_skills = Vec::new();
        for mut discovered in discovered_skills {
            if let Some(existing) = stored_by_id.get(&discovered.skill_id) {
                if existing.provider_id != "agent-skills" {
                    warn!(
                        skill_id = %discovered.skill_id,
                        stored_provider_id = %existing.provider_id,
                        "standard agent skill identity conflicts with another source; preserving stored skill"
                    );
                    report.unchanged += 1;
                    continue;
                }
                discovered.enabled = existing.enabled;
                discovered.scoped_config.clone_from(&existing.scoped_config);
            }
            discovered.tombstoned = false;

            match stored_by_id.get(&discovered.skill_id) {
                Some(existing)
                    if serde_json::to_value(existing)? == serde_json::to_value(&discovered)? =>
                {
                    report.unchanged += 1;
                }
                Some(_) => {
                    report.updated += 1;
                    changed_skills.push(discovered);
                }
                None => {
                    report.added += 1;
                    changed_skills.push(discovered);
                }
            }
        }

        self.registry
            .write()
            .await
            .register_checked_batch_without_embeddings(changed_skills)
            .await?;
        info!(
            name: "skills.standard.reconciled",
            provider = "agent-skills",
            discovered = report.discovered,
            added = report.added,
            updated = report.updated,
            unchanged = report.unchanged,
            "reconciled standard agent skills"
        );
        Ok(report)
    }

    /// Get all skills from all providers.
    pub async fn get_skills(&self) -> Vec<Skill> {
        self.registry.read().await.list()
    }

    /// Get only enabled skills.
    pub async fn get_enabled_skills(&self) -> Vec<Skill> {
        self.registry.read().await.list_enabled()
    }

    /// Refresh skills from all providers.
    pub async fn refresh(&self) -> anyhow::Result<Vec<Skill>> {
        let mut registry = self.registry.write().await;
        registry.clear();

        let mut all_skills = Vec::new();
        for provider in &self.providers {
            if !provider.is_enabled() {
                continue;
            }
            match provider.refresh().await {
                Ok(skills) => {
                    all_skills.extend(skills.iter().filter(|skill| !skill.tombstoned).cloned());
                    registry.register_all_loaded(skills);
                }
                Err(e) => {
                    error!("Failed to refresh provider '{}': {:?}", provider.name(), e);
                }
            }
        }

        Ok(all_skills)
    }

    /// Toggle a skill's enabled state.
    pub async fn toggle_skill(&self, id: &str, enabled: bool) -> bool {
        self.set_scoped_enabled(id, SkillScope::Global, enabled)
            .await
    }

    /// Set a durable enabled-state override at one scope.
    pub async fn set_scoped_enabled(&self, id: &str, scope: SkillScope, enabled: bool) -> bool {
        if let Err(error) = self.ensure_mutation_allowed(id).await {
            warn!("Failed to configure skill '{}': {:?}", id, error);
            return false;
        }

        let updated = {
            let mut registry = self.registry.write().await;
            let Some(mut skill) = registry.get(id).cloned() else {
                return false;
            };
            skill.set_enabled_for(scope, enabled);
            registry.register(skill.clone()).await;
            skill
        };

        if updated.provider_id == "api" {
            self.persist_to_filesystem(&updated).await;
        }
        info!(skill_id = id, enabled, "updated scoped skill configuration");
        true
    }

    /// Create a new skill dynamically via API.
    ///
    /// Persists to the database (via the registry's auto-persist path) and
    /// writes a SKILL.md file to the writable filesystem provider so the
    /// skill survives pod restarts.
    pub async fn create_skill(&self, mut skill: Skill) -> anyhow::Result<Skill> {
        skill.provider_id = "api".to_string();

        // Persist to DB + embed via registry (requires persistence + VectorMatcher)
        self.registry.write().await.register(skill.clone()).await;

        // Also write to the filesystem provider (skills/dynamic/) for restart durability
        for provider in &self.providers {
            if provider.kind() == StorageProviderKind::Filesystem {
                if let Err(e) = provider.save_skill(&skill).await {
                    warn!(
                        "Could not write skill '{}' to filesystem provider '{}': {:?}",
                        skill.skill_id,
                        provider.name(),
                        e
                    );
                }
                break;
            }
        }

        info!("Created skill via API: {}", skill.skill_id);
        Ok(skill)
    }

    /// Update an existing skill by ID.
    ///
    /// Returns `Ok(None)` if the skill does not exist.
    ///
    /// When a [`GovernanceEngine`] is attached, the mutation is checked against
    /// Cedar policies before being applied. The default environment is
    /// `"development"` (all mutations permitted). Set `PROMETHEUS_ENVIRONMENT`
    /// to `"staging"` or `"production"` for stricter enforcement.
    pub async fn update_skill(
        &self,
        id: &str,
        update: SkillUpdate,
    ) -> anyhow::Result<Option<Skill>> {
        self.ensure_mutation_allowed(id).await?;

        let existing = { self.registry.read().await.get(id).cloned() };
        let Some(mut skill) = existing else {
            return Ok(None);
        };
        if matches!(
            skill.origin,
            crate::uar::domain::skills::SkillOrigin::Builtin
        ) {
            anyhow::bail!("system_skill_immutable");
        }

        if let Some(version) = update.version {
            skill.version = version;
        }
        if let Some(title) = update.title {
            skill.title = title;
        }
        if let Some(description) = update.description {
            skill.description = description;
        }
        if let Some(triggers) = update.triggers {
            skill.triggers = triggers;
        }
        if let Some(prompt_overlay) = update.prompt_overlay {
            skill.prompt_overlay = prompt_overlay;
        }
        if let Some(preferred_tools) = update.preferred_tools {
            skill.preferred_tools = preferred_tools;
        }
        if let Some(enabled) = update.enabled {
            skill.set_enabled_for(SkillScope::Global, enabled);
        }
        if let Some(execution_config) = update.execution_config {
            skill.execution_config = execution_config;
        }

        self.registry.write().await.register(skill.clone()).await;

        if skill.provider_id == "api" {
            self.persist_to_filesystem(&skill).await;
        }

        info!("Updated skill via API: {}", skill.skill_id);
        Ok(Some(skill))
    }

    /// Permanently delete a skill from the registry, database, and filesystem.
    ///
    /// Skills with `origin = Builtin` are immutable; this method returns
    /// `Err(SystemSkillImmutable)` for them so the API layer can map to 409.
    pub async fn delete_skill_permanent(&self, id: &str) -> anyhow::Result<bool> {
        // Block deletion of Builtin skills (system-shipped, immutable).
        {
            let registry = self.registry.read().await;
            if let Some(skill) = registry.get(id) {
                if matches!(
                    skill.origin,
                    crate::uar::domain::skills::SkillOrigin::Builtin
                ) {
                    anyhow::bail!("system_skill_immutable");
                }
            }
        }

        let removed = self.registry.write().await.remove(id).is_some();

        // Delete from all providers that support deletion
        for provider in &self.providers {
            if let Err(e) = provider.delete_skill(id).await {
                warn!(
                    "Provider '{}' delete_skill('{}') failed (non-fatal): {:?}",
                    provider.name(),
                    id,
                    e
                );
            }
        }

        info!("Deleted skill permanently: {}", id);
        Ok(removed)
    }

    /// Match skills to a query without treating candidates as activations.
    pub async fn match_skills(&self, query: &str, agent_id: Option<&str>) -> SkillMatchResult {
        self.match_skills_scoped(query, agent_id, None).await
    }

    /// Match after resolving conversation > agent > global enabled state.
    pub async fn match_skills_scoped(
        &self,
        query: &str,
        agent_id: Option<&str>,
        conversation_id: Option<&str>,
    ) -> SkillMatchResult {
        let legacy_bindings = if let Some(agent_id) = agent_id {
            self.agent_skills
                .read()
                .await
                .get(agent_id)
                .filter(|skill_ids| !skill_ids.is_empty())
                .cloned()
        } else {
            None
        };
        let registry = self.registry.read().await;
        let config = self.matching_config.read().await.clone();
        Self::match_in_registry(
            query,
            agent_id,
            conversation_id,
            legacy_bindings.as_ref(),
            &registry,
            &config,
        )
        .await
    }

    /// Capture bodies, scoped enablement, bindings, and matching configuration
    /// before run assembly. Vector retrieval may supply scores, but never
    /// replace a captured skill body or introduce an uncaptured skill ID.
    pub(crate) async fn matching_snapshot(&self) -> SkillMatchingSnapshot {
        let agent_skills = self.agent_skills.read().await.clone();
        let registry = self.registry.read().await;
        let config = self.matching_config.read().await.clone();
        SkillMatchingSnapshot {
            registry: Arc::new(RwLock::new(registry.clone())),
            config,
            agent_skills,
        }
    }

    async fn match_in_registry(
        query: &str,
        agent_id: Option<&str>,
        conversation_id: Option<&str>,
        legacy_bindings: Option<&Vec<String>>,
        registry: &SkillRegistry,
        config: &SkillMatchingConfig,
    ) -> SkillMatchResult {
        let eligible = registry
            .list()
            .into_iter()
            .filter(|skill| {
                let fallback = legacy_bindings
                    .as_ref()
                    .map(|ids| ids.contains(&skill.skill_id) && skill.enabled_for(None, None));
                skill.enabled_for_with_agent_fallback(agent_id, conversation_id, fallback)
            })
            .collect::<Vec<_>>();
        let keyword = || {
            eligible
                .iter()
                .map(|skill| SkillCandidate::keyword(skill, query))
                .collect::<Vec<_>>()
        };
        let candidates = match config.algorithm {
            SkillMatchingAlgorithm::Keyword | SkillMatchingAlgorithm::Llm => {
                if config.algorithm == SkillMatchingAlgorithm::Llm {
                    warn!("LLM matching not yet implemented, falling back to keyword");
                }
                Self::keyword_match(query, &eligible, config.top_k, config.threshold).candidates
            }
            SkillMatchingAlgorithm::Embedding | SkillMatchingAlgorithm::LocalEmbedding => registry
                .find_candidates(query)
                .await
                .into_iter()
                .filter_map(|candidate| {
                    eligible
                        .iter()
                        .find(|skill| skill.skill_id == candidate.skill.skill_id)
                        .map(|skill| SkillCandidate {
                            skill: skill.clone(),
                            score: candidate.score,
                        })
                })
                .collect(),
            SkillMatchingAlgorithm::Hybrid => {
                let mut merged = keyword()
                    .into_iter()
                    .map(|candidate| (candidate.skill.skill_id.clone(), candidate))
                    .collect::<HashMap<_, _>>();
                for candidate in registry.find_candidates(query).await {
                    if let Some(existing) = merged.get_mut(&candidate.skill.skill_id) {
                        // Both scores are confidence values; retain the stronger signal.
                        existing.score = existing.score.max(candidate.score);
                    }
                }
                merged.into_values().collect()
            }
        };
        SkillMatchResult::resolve(
            candidates,
            config.threshold,
            config.margin_threshold,
            config.top_k,
        )
    }

    /// Keyword matching with the same scored-result contract as other backends.
    fn keyword_match(
        query: &str,
        candidates: &[Skill],
        top_k: usize,
        threshold: f32,
    ) -> SkillMatchResult {
        SkillMatchResult::resolve(
            candidates
                .iter()
                .map(|skill| SkillCandidate::keyword(skill, query))
                .collect(),
            threshold,
            0.0,
            top_k,
        )
    }

    // --- Per-agent skill configuration ---

    /// Get skill IDs explicitly enabled for an agent.
    pub async fn get_agent_skill_ids(&self, agent_id: &str) -> Vec<String> {
        let mut skill_ids = self
            .agent_skills
            .read()
            .await
            .get(agent_id)
            .cloned()
            .unwrap_or_default();
        for skill_id in self
            .registry
            .read()
            .await
            .list()
            .into_iter()
            .filter(|skill| {
                skill.scoped_config.iter().any(|config| {
                    config.enabled
                        && matches!(&config.scope, SkillScope::Agent(id) if id == agent_id)
                })
            })
            .map(|skill| skill.skill_id)
        {
            if !skill_ids.contains(&skill_id) {
                skill_ids.push(skill_id);
            }
        }
        skill_ids
    }

    /// Replace an agent's durable skill overrides using allowlist semantics.
    pub async fn set_agent_skills(&self, agent_id: &str, skill_ids: Vec<String>) {
        let selected = skill_ids.iter().cloned().collect::<HashSet<_>>();
        self.agent_skills
            .write()
            .await
            .insert(agent_id.to_string(), skill_ids);
        let all_ids = self
            .registry
            .read()
            .await
            .list()
            .into_iter()
            .map(|skill| skill.skill_id)
            .collect::<Vec<_>>();
        for skill_id in all_ids {
            self.set_scoped_enabled(
                &skill_id,
                SkillScope::Agent(agent_id.to_string()),
                selected.contains(&skill_id),
            )
            .await;
        }
    }

    /// Enable one skill for an agent.
    pub async fn add_skill_to_agent(&self, agent_id: &str, skill_id: &str) {
        let mut bindings = self.agent_skills.write().await;
        let entry = bindings.entry(agent_id.to_string()).or_default();
        if !entry.iter().any(|id| id == skill_id) {
            entry.push(skill_id.to_string());
        }
        drop(bindings);
        self.set_scoped_enabled(skill_id, SkillScope::Agent(agent_id.to_string()), true)
            .await;
    }

    /// Disable one skill for an agent.
    pub async fn remove_skill_from_agent(&self, agent_id: &str, skill_id: &str) {
        if let Some(bindings) = self.agent_skills.write().await.get_mut(agent_id) {
            bindings.retain(|id| id != skill_id);
        }
        self.set_scoped_enabled(skill_id, SkillScope::Agent(agent_id.to_string()), false)
            .await;
    }

    /// Get skills enabled after resolving agent state over global state.
    pub async fn get_enabled_skills_for_agent(&self, agent_id: &str) -> Vec<Skill> {
        let registry = self.registry.read().await;
        registry
            .list()
            .into_iter()
            .filter(|skill| skill.enabled_for(Some(agent_id), None))
            .collect()
    }

    async fn ensure_mutation_allowed(&self, id: &str) -> anyhow::Result<()> {
        if let Some(ref engine) = self.governance {
            let environment = std::env::var("PROMETHEUS_ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string());
            let context = format!(
                r#"{{"environment": "{}", "validation_passed": true}}"#,
                environment
            );
            let allowed = engine
                .is_skill_mutation_allowed("uar-runtime", "skill.mutate", id, &context)
                .await;
            if !allowed {
                anyhow::bail!(
                    "Cedar policy denied skill mutation for '{}' in {} environment",
                    id,
                    environment
                );
            }
        }
        Ok(())
    }

    async fn persist_to_filesystem(&self, skill: &Skill) {
        for provider in &self.providers {
            if provider.kind() == StorageProviderKind::Filesystem {
                if let Err(error) = provider.save_skill(skill).await {
                    warn!(
                        "Could not write updated skill '{}' to filesystem provider '{}': {:?}",
                        skill.skill_id,
                        provider.name(),
                        error
                    );
                }
                break;
            }
        }
    }

    // --- Config ---

    /// Get the current matching configuration.
    pub async fn get_matching_config(&self) -> SkillMatchingConfig {
        self.matching_config.read().await.clone()
    }

    /// Update the matching configuration.
    pub async fn set_matching_config(&self, config: SkillMatchingConfig) {
        *self.matching_config.write().await = config;
    }

    /// Get access to the internal registry.
    pub fn registry(&self) -> &Arc<RwLock<SkillRegistry>> {
        &self.registry
    }

    /// Get the list of registered providers.
    pub fn providers(&self) -> &[Arc<dyn SkillStorageProvider>] {
        &self.providers
    }

    /// Post-run skill evolution hook (Hermes learning cycle).
    ///
    /// Called after a run completes when `SkillEvolutionConfig::enabled` is true
    /// and the run performed at least `min_tool_calls` tool completions.
    ///
    /// Currently this is a no-op stub — a future implementation will:
    /// 1. Retrieve the run transcript / tool call log.
    /// 2. Fire a reflection prompt via the configured LLM.
    /// 3. Parse suggested skill CRUD operations from the response.
    /// 4. Apply them subject to `allow_update` / `allow_deletion` guards.
    pub async fn evolve_from_run(
        &self,
        run_id: &str,
        tool_call_count: usize,
        cfg: &crate::config::SkillEvolutionConfig,
    ) -> anyhow::Result<()> {
        tracing::debug!(
            run_id = %run_id,
            tool_calls = tool_call_count,
            max_skills = cfg.max_skills_per_run,
            allow_update = cfg.allow_update,
            allow_deletion = cfg.allow_deletion,
            "skill evolution hook invoked (stub — reflection not yet implemented)"
        );
        // TODO: implement reflection prompt → skill CRUD pipeline.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::uar::domain::skills::{SkillConstraints, SkillOrigin, SkillTriggers};
    use crate::uar::persistence::providers::surreal::SurrealDbProvider;
    use crate::uar::runtime::skills::storage::{
        DatabaseStorageProvider, FilesystemStorageProvider, filesystem::serialize_skill_to_md,
    };

    const B5_CHILD_MODE: &str = "UAR_B5_RECONCILIATION_CHILD_MODE";
    const B5_CHILD_ENDPOINT: &str = "UAR_B5_RECONCILIATION_ENDPOINT";
    const B5_CHILD_SKILLS_DIR: &str = "UAR_B5_RECONCILIATION_SKILLS_DIR";
    const STANDARD_CHILD_MODE: &str = "UAR_STANDARD_SKILL_CHILD_MODE";
    const STANDARD_CHILD_ENDPOINT: &str = "UAR_STANDARD_SKILL_CHILD_ENDPOINT";
    const STANDARD_CHILD_SKILLS_DIR: &str = "UAR_STANDARD_SKILL_CHILD_SKILLS_DIR";

    #[cfg(feature = "local-models")]
    #[derive(Debug)]
    struct RecordingEmbeddingBackend {
        batch_sizes: Arc<std::sync::Mutex<Vec<usize>>>,
    }

    #[cfg(feature = "local-models")]
    #[async_trait::async_trait]
    impl crate::uar::rag::embeddings::EmbeddingBackend for RecordingEmbeddingBackend {
        fn backend_name(&self) -> &str {
            "recording"
        }

        fn vector_dimension(&self) -> usize {
            1
        }

        async fn embed(
            &self,
            texts: &[&str],
        ) -> Result<Vec<Vec<f32>>, crate::uar::rag::embeddings::EmbeddingError> {
            self.batch_sizes.lock().unwrap().push(texts.len());
            Ok(vec![vec![1.0]; texts.len()])
        }
    }

    fn test_skill() -> Skill {
        Skill {
            skill_id: "test-skill".to_string(),
            version: "1.0.0".to_string(),
            title: "Test Skill".to_string(),
            description: "Initial description".to_string(),
            triggers: SkillTriggers {
                keywords: vec!["initial".to_string()],
                semantic: None,
            },
            prompt_overlay: "# Initial Prompt".to_string(),
            preferred_tools: vec!["search".to_string()],
            mcp_config: None,
            constraints: SkillConstraints::default(),
            enabled: true,
            provider_id: "api".to_string(),
            execution_config: Default::default(),
            kind: Default::default(),
            origin: Default::default(),
            ..Default::default()
        }
    }

    async fn write_config_skill(root: &std::path::Path, skill: &Skill) {
        let skill_directory = root.join(&skill.skill_id);
        tokio::fs::create_dir_all(&skill_directory).await.unwrap();
        tokio::fs::write(
            skill_directory.join("SKILL.md"),
            serialize_skill_to_md(skill).unwrap(),
        )
        .await
        .unwrap();
    }

    fn reconciliation_service(
        persistence: Arc<dyn PersistenceLayer>,
        skills_root: &std::path::Path,
    ) -> SkillService {
        let mut service = SkillService::new(Some(Arc::clone(&persistence)), None);
        service.add_provider(Arc::new(FilesystemStorageProvider::new(
            "fs-skills",
            "Configuration skills",
            skills_root,
        )));
        service.add_provider(Arc::new(DatabaseStorageProvider::new(
            "db-skills",
            "Database skills",
            persistence,
        )));
        service
    }

    fn standard_reconciliation_service(
        persistence: Arc<dyn PersistenceLayer>,
        skills_root: &std::path::Path,
    ) -> SkillService {
        let mut service = SkillService::new(Some(Arc::clone(&persistence)), None);
        service.add_provider(Arc::new(
            FilesystemStorageProvider::standard_agent_directory(skills_root),
        ));
        service.add_provider(Arc::new(DatabaseStorageProvider::new(
            "db-skills",
            "Database skills",
            persistence,
        )));
        service
    }

    async fn write_standard_skill(
        root: &std::path::Path,
        relative_directory: &str,
        name: &str,
        description: &str,
    ) {
        let skill_directory = root.join(relative_directory);
        tokio::fs::create_dir_all(&skill_directory).await.unwrap();
        tokio::fs::write(
            skill_directory.join("SKILL.md"),
            format!("---\nname: {name}\ndescription: {description}\n---\n\n# {name}\n"),
        )
        .await
        .unwrap();
    }

    #[cfg(feature = "local-models")]
    #[tokio::test]
    async fn standard_skill_reconciliation_does_not_invoke_embeddings() {
        let batch_sizes = Arc::new(std::sync::Mutex::new(Vec::new()));
        let backend = Arc::new(RecordingEmbeddingBackend {
            batch_sizes: Arc::clone(&batch_sizes),
        });
        let matcher = Arc::new(VectorMatcher::new(backend, 0.75));
        let database_directory = tempfile::tempdir().unwrap();
        let endpoint = format!(
            "surrealkv://{}",
            database_directory.path().join("skills.db").display()
        );
        let persistence: Arc<dyn PersistenceLayer> = Arc::new(
            SurrealDbProvider::new(
                &endpoint,
                None,
                None,
                Some("agent-skills"),
                Some("no-embedding"),
            )
            .await
            .unwrap(),
        );
        let skills_directory = tempfile::tempdir().unwrap();
        write_standard_skill(
            skills_directory.path(),
            "alpha",
            "alpha-skill",
            "first alpha",
        )
        .await;
        let mut service = SkillService::new(Some(Arc::clone(&persistence)), Some(matcher));
        service.add_provider(Arc::new(
            FilesystemStorageProvider::standard_agent_directory(skills_directory.path()),
        ));
        service.initialize().await.unwrap();

        let report = service.reconcile_standard_agent_skills().await.unwrap();

        assert_eq!(
            report,
            AgentSkillReconciliationReport {
                discovered: 1,
                added: 1,
                updated: 0,
                unchanged: 0,
            }
        );
        assert!(batch_sizes.lock().unwrap().is_empty());
        assert!(
            persistence
                .list_skills()
                .await
                .unwrap()
                .iter()
                .any(|skill| skill.skill_id == "agents::alpha")
        );
    }

    #[tokio::test]
    async fn startup_standard_reconciliation_upserts_changes_preserves_scope_and_never_removes() {
        let database_directory = tempfile::tempdir().unwrap();
        let skills_directory = tempfile::tempdir().unwrap();
        let endpoint = format!(
            "surrealkv://{}",
            database_directory.path().join("skills.db").display()
        );
        let persistence: Arc<dyn PersistenceLayer> = Arc::new(
            SurrealDbProvider::new(
                &endpoint,
                None,
                None,
                Some("agent-skills"),
                Some("reconcile"),
            )
            .await
            .unwrap(),
        );
        write_standard_skill(
            skills_directory.path(),
            "alpha",
            "alpha-skill",
            "first alpha",
        )
        .await;
        write_standard_skill(
            skills_directory.path(),
            "removed",
            "removed-skill",
            "retained after source removal",
        )
        .await;

        let first =
            standard_reconciliation_service(Arc::clone(&persistence), skills_directory.path());
        first.initialize().await.unwrap();
        let first_report = first.reconcile_standard_agent_skills().await.unwrap();
        assert_eq!(
            first_report,
            AgentSkillReconciliationReport {
                discovered: 2,
                added: 2,
                updated: 0,
                unchanged: 0,
            }
        );
        assert!(
            first
                .set_scoped_enabled(
                    "agents::alpha",
                    SkillScope::Agent("agent-a".to_string()),
                    false,
                )
                .await
        );
        drop(first);

        write_standard_skill(
            skills_directory.path(),
            "alpha",
            "alpha-skill",
            "changed alpha",
        )
        .await;
        tokio::fs::remove_dir_all(skills_directory.path().join("removed"))
            .await
            .unwrap();
        write_standard_skill(skills_directory.path(), "beta", "beta-skill", "new beta").await;

        let changed =
            standard_reconciliation_service(Arc::clone(&persistence), skills_directory.path());
        changed.initialize().await.unwrap();
        let changed_report = changed.reconcile_standard_agent_skills().await.unwrap();
        assert_eq!(
            changed_report,
            AgentSkillReconciliationReport {
                discovered: 2,
                added: 1,
                updated: 1,
                unchanged: 0,
            }
        );
        let stored = persistence.list_skills().await.unwrap();
        let alpha = stored
            .iter()
            .find(|skill| skill.skill_id == "agents::alpha")
            .unwrap();
        assert_eq!(alpha.description, "changed alpha");
        assert!(alpha.scoped_config.iter().any(|config| {
            matches!(&config.scope, SkillScope::Agent(id) if id == "agent-a") && !config.enabled
        }));
        assert!(
            stored
                .iter()
                .any(|skill| { skill.skill_id == "agents::removed" && !skill.tombstoned })
        );
        assert!(stored.iter().any(|skill| skill.skill_id == "agents::beta"));

        let unchanged =
            standard_reconciliation_service(Arc::clone(&persistence), skills_directory.path());
        unchanged.initialize().await.unwrap();
        let unchanged_report = unchanged.reconcile_standard_agent_skills().await.unwrap();
        assert_eq!(
            unchanged_report,
            AgentSkillReconciliationReport {
                discovered: 2,
                added: 0,
                updated: 0,
                unchanged: 2,
            }
        );
    }

    #[tokio::test]
    async fn standard_reconciliation_survives_cold_process_restarts() {
        if let Ok(mode) = std::env::var(STANDARD_CHILD_MODE) {
            let endpoint = std::env::var(STANDARD_CHILD_ENDPOINT).unwrap();
            let skills_directory =
                std::path::PathBuf::from(std::env::var(STANDARD_CHILD_SKILLS_DIR).unwrap());
            let persistence: Arc<dyn PersistenceLayer> = Arc::new(
                SurrealDbProvider::new(
                    &endpoint,
                    None,
                    None,
                    Some("agent-skills-cold"),
                    Some("agent-skills-cold"),
                )
                .await
                .unwrap(),
            );
            let service =
                standard_reconciliation_service(Arc::clone(&persistence), &skills_directory);
            service.initialize().await.unwrap();
            let report = service.reconcile_standard_agent_skills().await.unwrap();

            match mode.as_str() {
                "seed" => {
                    assert_eq!(report.added, 2);
                    assert!(
                        service
                            .set_scoped_enabled(
                                "agents::alpha",
                                SkillScope::Agent("agent-a".to_string()),
                                false,
                            )
                            .await
                    );
                }
                "change" => {
                    assert_eq!(report.added, 1);
                    assert_eq!(report.updated, 1);
                    let stored = persistence.list_skills().await.unwrap();
                    let alpha = stored
                        .iter()
                        .find(|skill| skill.skill_id == "agents::alpha")
                        .unwrap();
                    assert_eq!(alpha.description, "changed alpha after restart");
                    assert!(alpha.scoped_config.iter().any(|config| {
                        matches!(&config.scope, SkillScope::Agent(id) if id == "agent-a")
                            && !config.enabled
                    }));
                    assert!(
                        stored.iter().any(|skill| {
                            skill.skill_id == "agents::removed" && !skill.tombstoned
                        })
                    );
                }
                "unchanged" => {
                    assert_eq!(report.unchanged, 2);
                }
                _ => panic!("unknown standard skill child mode: {mode}"),
            }
            return;
        }

        let directory = tempfile::tempdir().unwrap();
        let endpoint = format!(
            "surrealkv://{}",
            directory.path().join("standard-skills.db").display()
        );
        let skills_directory = directory.path().join("skills");
        write_standard_skill(&skills_directory, "alpha", "alpha-skill", "first alpha").await;
        write_standard_skill(
            &skills_directory,
            "removed",
            "removed-skill",
            "retained after source removal",
        )
        .await;
        run_standard_skill_child("seed", &endpoint, &skills_directory);

        write_standard_skill(
            &skills_directory,
            "alpha",
            "alpha-skill",
            "changed alpha after restart",
        )
        .await;
        tokio::fs::remove_dir_all(skills_directory.join("removed"))
            .await
            .unwrap();
        write_standard_skill(
            &skills_directory,
            "beta",
            "beta-skill",
            "new beta after restart",
        )
        .await;
        run_standard_skill_child("change", &endpoint, &skills_directory);
        run_standard_skill_child("unchanged", &endpoint, &skills_directory);
    }

    fn run_standard_skill_child(mode: &str, endpoint: &str, skills_directory: &std::path::Path) {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "uar::runtime::skills::service::tests::standard_reconciliation_survives_cold_process_restarts",
                "--test-threads=1",
            ])
            .env(STANDARD_CHILD_MODE, mode)
            .env(STANDARD_CHILD_ENDPOINT, endpoint)
            .env(STANDARD_CHILD_SKILLS_DIR, skills_directory)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "standard skill {mode} child failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[tokio::test]
    async fn update_skill_modifies_selected_fields() {
        let service = SkillService::new(None, None);
        let created = service
            .create_skill(test_skill())
            .await
            .expect("skill should be created");

        let updated = service
            .update_skill(
                &created.skill_id,
                SkillUpdate {
                    title: Some("Updated Skill".to_string()),
                    description: Some("Updated description".to_string()),
                    prompt_overlay: Some("## Updated markdown prompt".to_string()),
                    preferred_tools: Some(vec!["memory".to_string(), "search".to_string()]),
                    triggers: Some(SkillTriggers {
                        keywords: vec!["updated".to_string()],
                        semantic: Some("updated semantic".to_string()),
                    }),
                    enabled: Some(false),
                    ..SkillUpdate::default()
                },
            )
            .await
            .expect("update should succeed")
            .expect("skill should exist");

        assert_eq!(updated.skill_id, "test-skill");
        assert_eq!(updated.title, "Updated Skill");
        assert_eq!(updated.description, "Updated description");
        assert_eq!(updated.prompt_overlay, "## Updated markdown prompt");
        assert_eq!(updated.preferred_tools, vec!["memory", "search"]);
        assert_eq!(updated.triggers.keywords, vec!["updated"]);
        assert_eq!(updated.enabled, false);
    }

    #[tokio::test]
    async fn update_skill_returns_none_for_missing_skill() {
        let service = SkillService::new(None, None);
        let result = service
            .update_skill(
                "missing-skill",
                SkillUpdate {
                    title: Some("No-op".to_string()),
                    ..SkillUpdate::default()
                },
            )
            .await
            .expect("missing update should not error");

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn refresh_hides_tombstones_but_keeps_them_available_for_restore() {
        let database_directory = tempfile::tempdir().unwrap();
        let endpoint = format!(
            "surrealkv://{}",
            database_directory.path().join("skills.db").display()
        );
        let persistence: Arc<dyn PersistenceLayer> = Arc::new(
            SurrealDbProvider::new(&endpoint, None, None, Some("b5"), Some("visibility"))
                .await
                .unwrap(),
        );
        let active = Skill {
            skill_id: "active".to_string(),
            provider_id: "fs-skills".to_string(),
            ..Skill::default()
        };
        let tombstoned = Skill {
            skill_id: "removed".to_string(),
            provider_id: "fs-skills".to_string(),
            tombstoned: true,
            ..Skill::default()
        };
        persistence.save_skill(&active, &[]).await.unwrap();
        persistence.save_skill(&tombstoned, &[]).await.unwrap();

        let mut service = SkillService::new(Some(Arc::clone(&persistence)), None);
        service.add_provider(Arc::new(DatabaseStorageProvider::new(
            "db-skills",
            "Database skills",
            persistence,
        )));

        let visible = service.refresh().await.unwrap();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].skill_id, "active");
        assert!(service.registry.read().await.get("removed").is_some());
    }

    #[tokio::test]
    async fn scoped_state_resolves_conversation_then_agent_then_global_both_directions() {
        let service = SkillService::new(None, None);
        service.create_skill(test_skill()).await.unwrap();

        assert!(
            service
                .set_scoped_enabled("test-skill", SkillScope::Global, false)
                .await
        );
        assert!(
            service
                .set_scoped_enabled("test-skill", SkillScope::Agent("agent-a".to_string()), true,)
                .await
        );
        assert!(
            service
                .set_scoped_enabled(
                    "test-skill",
                    SkillScope::Conversation("conversation-a".to_string()),
                    false,
                )
                .await
        );

        assert!(
            service
                .match_skills_scoped("initial", Some("agent-a"), Some("conversation-a"))
                .await
                .accepted
                .is_empty()
        );
        assert_eq!(
            service
                .match_skills_scoped("initial", Some("agent-a"), Some("conversation-b"))
                .await
                .accepted
                .len(),
            1
        );
        assert!(
            service
                .match_skills_scoped("initial", Some("agent-b"), Some("conversation-b"))
                .await
                .accepted
                .is_empty()
        );

        assert!(
            service
                .set_scoped_enabled("test-skill", SkillScope::Global, true)
                .await
        );
        assert!(
            service
                .set_scoped_enabled(
                    "test-skill",
                    SkillScope::Agent("agent-a".to_string()),
                    false,
                )
                .await
        );
        assert!(
            service
                .set_scoped_enabled(
                    "test-skill",
                    SkillScope::Conversation("conversation-a".to_string()),
                    true,
                )
                .await
        );

        assert_eq!(
            service
                .match_skills_scoped("initial", Some("agent-a"), Some("conversation-a"))
                .await
                .accepted
                .len(),
            1
        );
        assert!(
            service
                .match_skills_scoped("initial", Some("agent-a"), Some("conversation-b"))
                .await
                .accepted
                .is_empty()
        );
        assert_eq!(
            service
                .match_skills_scoped("initial", Some("agent-b"), Some("conversation-b"))
                .await
                .accepted
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn scoped_change_affects_next_match_without_mutating_existing_binding() {
        let service = SkillService::new(None, None);
        service.create_skill(test_skill()).await.unwrap();

        let in_flight_binding = service
            .match_skills_scoped("initial", Some("agent-a"), Some("conversation-a"))
            .await;
        assert_eq!(in_flight_binding.accepted.len(), 1);

        assert!(
            service
                .set_scoped_enabled(
                    "test-skill",
                    SkillScope::Conversation("conversation-a".to_string()),
                    false,
                )
                .await
        );

        assert_eq!(in_flight_binding.accepted.len(), 1);
        assert!(
            service
                .match_skills_scoped("initial", Some("agent-a"), Some("conversation-a"))
                .await
                .accepted
                .is_empty()
        );
    }

    #[tokio::test]
    async fn agent_binding_set_before_load_filters_future_skills() {
        let service = SkillService::new(None, None);
        service
            .set_agent_skills("agent-a", vec!["future-skill".to_string()])
            .await;

        let mut future = test_skill();
        future.skill_id = "future-skill".to_string();
        let mut unbound = test_skill();
        unbound.skill_id = "unbound-skill".to_string();
        service.create_skill(future).await.unwrap();
        service.create_skill(unbound).await.unwrap();

        let matched = service.match_skills("initial", Some("agent-a")).await;
        assert_eq!(matched.accepted.len(), 1);
        assert_eq!(matched.accepted[0], "future-skill");

        assert!(
            service
                .set_scoped_enabled(
                    "unbound-skill",
                    SkillScope::Conversation("conversation-a".to_string()),
                    true,
                )
                .await
        );
        let conversation_match = service
            .match_skills_scoped("initial", Some("agent-a"), Some("conversation-a"))
            .await;
        assert_eq!(conversation_match.candidates.len(), 2);
        assert!(conversation_match.accepted.is_empty());
    }

    #[tokio::test]
    async fn builtin_scoped_state_survives_restart_reregistration() {
        let directory = tempfile::tempdir().unwrap();
        let endpoint = format!(
            "surrealkv://{}",
            directory.path().join("skills.db").display()
        );
        let persistence: Arc<dyn PersistenceLayer> = Arc::new(
            SurrealDbProvider::new(&endpoint, None, None, Some("b4"), Some("b4"))
                .await
                .unwrap(),
        );
        let mut global_builtin = test_skill();
        global_builtin.skill_id = "global-builtin".to_string();
        global_builtin.origin = SkillOrigin::Builtin;
        global_builtin.provider_id = "builtin".to_string();
        let mut agent_builtin = global_builtin.clone();
        agent_builtin.skill_id = "agent-builtin".to_string();

        let first = SkillService::new(Some(Arc::clone(&persistence)), None);
        first
            .register_builtins(vec![global_builtin.clone(), agent_builtin.clone()])
            .await;
        assert!(
            first
                .set_scoped_enabled("global-builtin", SkillScope::Global, false)
                .await
        );
        assert!(
            first
                .set_scoped_enabled(
                    "agent-builtin",
                    SkillScope::Agent("agent-a".to_string()),
                    false,
                )
                .await
        );

        let mut restarted = SkillService::new(Some(Arc::clone(&persistence)), None);
        restarted.add_provider(Arc::new(DatabaseStorageProvider::new(
            "test-db",
            "Test database",
            Arc::clone(&persistence),
        )));
        restarted
            .register_builtins(vec![global_builtin, agent_builtin])
            .await;
        restarted.initialize().await.unwrap();

        assert!(
            restarted
                .match_skills_scoped("initial", Some("agent-a"), Some("conversation-b"))
                .await
                .accepted
                .is_empty()
        );
        assert_eq!(
            restarted
                .match_skills_scoped("initial", Some("agent-b"), Some("conversation-b"))
                .await
                .accepted
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn builtin_delete_is_refused_and_user_delete_succeeds() {
        let service = SkillService::new(None, None);
        let mut builtin = test_skill();
        builtin.skill_id = "builtin-skill".to_string();
        builtin.origin = SkillOrigin::Builtin;
        service.register_builtins(vec![builtin]).await;
        service.create_skill(test_skill()).await.unwrap();

        let error = service
            .delete_skill_permanent("builtin-skill")
            .await
            .expect_err("built-in deletion must fail");
        assert!(error.to_string().contains("system_skill_immutable"));
        assert!(service.delete_skill_permanent("test-skill").await.unwrap());
        assert!(
            service
                .get_skills()
                .await
                .iter()
                .any(|skill| skill.skill_id == "builtin-skill")
        );
        assert!(
            service
                .get_skills()
                .await
                .iter()
                .all(|skill| skill.skill_id != "test-skill")
        );
    }

    #[tokio::test]
    async fn builtin_update_is_refused_while_disable_remains_available() {
        let service = SkillService::new(None, None);
        let mut builtin = test_skill();
        builtin.skill_id = "builtin-update-proof".to_string();
        builtin.title = "Original built-in".to_string();
        builtin.origin = SkillOrigin::Builtin;
        service.register_builtins(vec![builtin]).await;

        let error = service
            .update_skill(
                "builtin-update-proof",
                SkillUpdate {
                    title: Some("Mutated built-in".to_string()),
                    ..SkillUpdate::default()
                },
            )
            .await
            .expect_err("built-in edits must fail");
        assert!(error.to_string().contains("system_skill_immutable"));
        assert!(
            service
                .set_scoped_enabled("builtin-update-proof", SkillScope::Global, false)
                .await,
            "built-ins remain disableable"
        );

        let stored = service
            .get_skills()
            .await
            .into_iter()
            .find(|skill| skill.skill_id == "builtin-update-proof")
            .expect("built-in remains present");
        assert_eq!(stored.title, "Original built-in");
        assert!(!stored.enabled_for(None, None));
    }

    #[tokio::test]
    async fn reconciliation_adds_changes_tombstones_and_restores_scoped_config() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();
        let database_directory = tempfile::tempdir().unwrap();
        let skills_directory = tempfile::tempdir().unwrap();
        let endpoint = format!(
            "surrealkv://{}",
            database_directory.path().join("skills.db").display()
        );
        let persistence: Arc<dyn PersistenceLayer> = Arc::new(
            SurrealDbProvider::new(&endpoint, None, None, Some("b5"), Some("roundtrip"))
                .await
                .unwrap(),
        );
        let mut removed = test_skill();
        removed.skill_id = "config-removed".to_string();
        removed.title = "Config Removed".to_string();
        removed.provider_id = "fs-skills".to_string();
        removed.triggers.keywords = vec!["removed".to_string()];
        let mut retained = test_skill();
        retained.skill_id = "config-retained".to_string();
        retained.title = "Config Retained".to_string();
        retained.provider_id = "fs-skills".to_string();
        await_config_pair(skills_directory.path(), &removed, &retained).await;

        let first = reconciliation_service(Arc::clone(&persistence), skills_directory.path());
        first.initialize().await.unwrap();
        first.reconcile_config_skills().await.unwrap();
        assert!(
            first
                .set_scoped_enabled(
                    "config-removed",
                    SkillScope::Agent("agent-a".to_string()),
                    false,
                )
                .await
        );
        assert!(!skills_directory.path().join("dynamic").exists());

        removed.description = "Changed configuration definition".to_string();
        write_config_skill(skills_directory.path(), &removed).await;
        let changed = reconciliation_service(Arc::clone(&persistence), skills_directory.path());
        changed.initialize().await.unwrap();
        changed.reconcile_config_skills().await.unwrap();
        let changed_record = persistence
            .list_skills()
            .await
            .unwrap()
            .into_iter()
            .find(|skill| skill.skill_id == "config-removed")
            .unwrap();
        assert_eq!(
            changed_record.description,
            "Changed configuration definition"
        );

        tokio::fs::remove_dir_all(skills_directory.path().join("config-removed"))
            .await
            .unwrap();
        let removed_service =
            reconciliation_service(Arc::clone(&persistence), skills_directory.path());
        removed_service.initialize().await.unwrap();
        removed_service.reconcile_config_skills().await.unwrap();
        assert!(
            removed_service
                .get_skills()
                .await
                .iter()
                .all(|skill| skill.skill_id != "config-removed")
        );
        assert!(
            removed_service
                .match_skills("removed", Some("agent-b"))
                .await
                .accepted
                .is_empty()
        );
        let tombstoned = persistence
            .list_skills()
            .await
            .unwrap()
            .into_iter()
            .find(|skill| skill.skill_id == "config-removed")
            .unwrap();
        assert!(tombstoned.tombstoned);

        write_config_skill(skills_directory.path(), &removed).await;
        let restored = reconciliation_service(Arc::clone(&persistence), skills_directory.path());
        restored.initialize().await.unwrap();
        restored.reconcile_config_skills().await.unwrap();
        let restored_record = persistence
            .list_skills()
            .await
            .unwrap()
            .into_iter()
            .find(|skill| skill.skill_id == "config-removed")
            .unwrap();
        assert!(!restored_record.tombstoned);
        assert!(restored_record.scoped_config.iter().any(|config| {
            matches!(&config.scope, SkillScope::Agent(id) if id == "agent-a") && !config.enabled
        }));
        assert!(
            restored
                .match_skills("removed", Some("agent-a"))
                .await
                .accepted
                .is_empty()
        );
        assert_eq!(
            restored
                .match_skills("removed", Some("agent-b"))
                .await
                .accepted
                .len(),
            1
        );
    }

    async fn await_config_pair(root: &std::path::Path, first: &Skill, second: &Skill) {
        write_config_skill(root, first).await;
        write_config_skill(root, second).await;
    }

    #[tokio::test]
    async fn empty_source_fail_safe_preserves_every_skill_origin() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();
        let database_directory = tempfile::tempdir().unwrap();
        let skills_directory = tempfile::tempdir().unwrap();
        let endpoint = format!(
            "surrealkv://{}",
            database_directory.path().join("skills.db").display()
        );
        let persistence: Arc<dyn PersistenceLayer> = Arc::new(
            SurrealDbProvider::new(&endpoint, None, None, Some("b5"), Some("failsafe"))
                .await
                .unwrap(),
        );
        for (id, provider_id, origin) in [
            ("config-skill", "fs-skills", SkillOrigin::User),
            ("api-skill", "api", SkillOrigin::User),
            ("builtin-skill", "builtin", SkillOrigin::Builtin),
        ] {
            let mut skill = test_skill();
            skill.skill_id = id.to_string();
            skill.title = id.to_string();
            skill.provider_id = provider_id.to_string();
            skill.origin = origin;
            persistence.save_skill(&skill, &[]).await.unwrap();
        }

        let service = reconciliation_service(Arc::clone(&persistence), skills_directory.path());
        service.initialize().await.unwrap();
        service.reconcile_config_skills().await.unwrap();

        let stored = persistence.list_skills().await.unwrap();
        assert_eq!(stored.len(), 3);
        assert!(stored.iter().all(|skill| !skill.tombstoned));
    }

    #[tokio::test]
    async fn api_skill_survives_empty_configuration_by_provider_id() {
        let database_directory = tempfile::tempdir().unwrap();
        let skills_directory = tempfile::tempdir().unwrap();
        let endpoint = format!(
            "surrealkv://{}",
            database_directory.path().join("skills.db").display()
        );
        let persistence: Arc<dyn PersistenceLayer> = Arc::new(
            SurrealDbProvider::new(&endpoint, None, None, Some("b5"), Some("api-origin"))
                .await
                .unwrap(),
        );
        let first = reconciliation_service(Arc::clone(&persistence), skills_directory.path());
        first.initialize().await.unwrap();
        let mut api_skill = test_skill();
        api_skill.skill_id = "api-survivor".to_string();
        api_skill.provider_id = "api".to_string();
        first.create_skill(api_skill).await.unwrap();
        let mut builtin = test_skill();
        builtin.skill_id = "builtin-survivor".to_string();
        builtin.provider_id = "builtin".to_string();
        builtin.origin = SkillOrigin::Builtin;
        first.register_builtins(vec![builtin]).await;
        drop(first);

        let service = reconciliation_service(Arc::clone(&persistence), skills_directory.path());
        service.initialize().await.unwrap();
        service.reconcile_config_skills().await.unwrap();

        let stored = persistence.list_skills().await.unwrap();
        let api_skill = stored
            .iter()
            .find(|skill| skill.skill_id == "api-survivor")
            .unwrap();
        assert!(!api_skill.tombstoned);
        let builtin = stored
            .iter()
            .find(|skill| skill.skill_id == "builtin-survivor")
            .unwrap();
        assert!(!builtin.tombstoned);
    }

    #[tokio::test]
    async fn reconciliation_survives_cold_process_restarts() {
        if let Ok(mode) = std::env::var(B5_CHILD_MODE) {
            let endpoint = std::env::var(B5_CHILD_ENDPOINT).unwrap();
            let skills_directory =
                std::path::PathBuf::from(std::env::var(B5_CHILD_SKILLS_DIR).unwrap());
            let persistence: Arc<dyn PersistenceLayer> = Arc::new(
                SurrealDbProvider::new(&endpoint, None, None, Some("b5-cold"), Some("b5-cold"))
                    .await
                    .unwrap(),
            );
            let service = reconciliation_service(Arc::clone(&persistence), &skills_directory);
            service.initialize().await.unwrap();
            service.reconcile_config_skills().await.unwrap();

            match mode.as_str() {
                "seed" => {
                    assert_eq!(persistence.list_skills().await.unwrap().len(), 2);
                    assert!(
                        service
                            .set_scoped_enabled(
                                "cold-removed",
                                SkillScope::Agent("agent-a".to_string()),
                                false,
                            )
                            .await
                    );
                    assert!(!skills_directory.join("dynamic").exists());
                }
                "change" => {
                    let changed = persistence
                        .list_skills()
                        .await
                        .unwrap()
                        .into_iter()
                        .find(|skill| skill.skill_id == "cold-removed")
                        .unwrap();
                    assert_eq!(changed.description, "Changed across cold restart");
                }
                "remove" => {
                    let tombstoned = persistence
                        .list_skills()
                        .await
                        .unwrap()
                        .into_iter()
                        .find(|skill| skill.skill_id == "cold-removed")
                        .unwrap();
                    assert!(tombstoned.tombstoned);
                    assert!(
                        service
                            .get_skills()
                            .await
                            .iter()
                            .all(|skill| skill.skill_id != "cold-removed")
                    );
                    assert!(
                        service
                            .match_skills("cold-removed", Some("agent-b"))
                            .await
                            .accepted
                            .is_empty()
                    );
                }
                "restore" => {
                    let restored = persistence
                        .list_skills()
                        .await
                        .unwrap()
                        .into_iter()
                        .find(|skill| skill.skill_id == "cold-removed")
                        .unwrap();
                    assert!(!restored.tombstoned);
                    assert!(restored.scoped_config.iter().any(|config| {
                        matches!(&config.scope, SkillScope::Agent(id) if id == "agent-a")
                            && !config.enabled
                    }));
                    assert!(
                        service
                            .match_skills("cold-removed", Some("agent-a"))
                            .await
                            .accepted
                            .is_empty()
                    );
                    assert_eq!(
                        service
                            .match_skills("cold-removed", Some("agent-b"))
                            .await
                            .accepted
                            .len(),
                        1
                    );
                }
                _ => panic!("unknown B5 child mode: {mode}"),
            }
            return;
        }

        let directory = tempfile::tempdir().unwrap();
        let endpoint = format!(
            "surrealkv://{}",
            directory.path().join("reconciliation.db").display()
        );
        let skills_directory = directory.path().join("skills");
        let mut removed = test_skill();
        removed.skill_id = "cold-removed".to_string();
        removed.title = "Cold Removed".to_string();
        removed.provider_id = "fs-skills".to_string();
        removed.triggers.keywords = vec!["cold-removed".to_string()];
        let mut retained = test_skill();
        retained.skill_id = "cold-retained".to_string();
        retained.title = "Cold Retained".to_string();
        retained.provider_id = "fs-skills".to_string();
        await_config_pair(&skills_directory, &removed, &retained).await;
        run_b5_child("seed", &endpoint, &skills_directory);

        removed.description = "Changed across cold restart".to_string();
        write_config_skill(&skills_directory, &removed).await;
        run_b5_child("change", &endpoint, &skills_directory);

        tokio::fs::remove_dir_all(skills_directory.join("cold-removed"))
            .await
            .unwrap();
        run_b5_child("remove", &endpoint, &skills_directory);

        write_config_skill(&skills_directory, &removed).await;
        run_b5_child("restore", &endpoint, &skills_directory);
    }

    fn run_b5_child(mode: &str, endpoint: &str, skills_directory: &std::path::Path) {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "uar::runtime::skills::service::tests::reconciliation_survives_cold_process_restarts",
                "--test-threads=1",
            ])
            .env(B5_CHILD_MODE, mode)
            .env(B5_CHILD_ENDPOINT, endpoint)
            .env(B5_CHILD_SKILLS_DIR, skills_directory)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "B5 {mode} child failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
