//! Skill service — central coordinator for skills.
//!
//! Mirrors cherry-studio's `SkillService` pattern:
//! - Aggregates skills from multiple storage providers
//! - Configurable matching algorithms
//! - Per-agent skill bindings
//! - Script execution (sandboxed)

use super::registry::SkillRegistry;
use super::storage::{SkillStorageProvider, StorageProviderKind};
use crate::uar::domain::skills::{Skill, SkillScope};
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

impl Default for SkillMatchingConfig {
    fn default() -> Self {
        Self {
            algorithm: SkillMatchingAlgorithm::default(),
            threshold: default_threshold(),
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
                    all_skills.extend(skills.clone());
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

        if updated.origin == crate::uar::domain::skills::SkillOrigin::User {
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

        self.persist_to_filesystem(&skill).await;

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

    /// Match skills to a query for a specific agent.
    pub async fn match_skills(&self, query: &str, agent_id: Option<&str>) -> Vec<Skill> {
        self.match_skills_scoped(query, agent_id, None).await
    }

    /// Match skills after resolving conversation > agent > global state.
    pub async fn match_skills_scoped(
        &self,
        query: &str,
        agent_id: Option<&str>,
        conversation_id: Option<&str>,
    ) -> Vec<Skill> {
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
        let config = self.matching_config.read().await;

        let candidates = registry
            .list()
            .into_iter()
            .filter(|skill| {
                let agent_fallback = legacy_bindings.as_ref().map(|skill_ids| {
                    skill_ids.iter().any(|id| id == &skill.skill_id)
                        && skill.enabled_for(None, None)
                });
                skill.enabled_for_with_agent_fallback(agent_id, conversation_id, agent_fallback)
            })
            .collect::<Vec<_>>();

        if candidates.is_empty() {
            return Vec::new();
        }

        // Match using configured algorithm
        let matched = match config.algorithm {
            SkillMatchingAlgorithm::Keyword => {
                self.keyword_match(query, &candidates, config.top_k, config.threshold)
            }
            SkillMatchingAlgorithm::Embedding | SkillMatchingAlgorithm::LocalEmbedding => {
                // Use vector matching through the registry
                let results = registry.find_matches(query).await;
                results
                    .into_iter()
                    .filter(|s| candidates.iter().any(|c| c.skill_id == s.skill_id))
                    .take(config.top_k)
                    .collect()
            }
            SkillMatchingAlgorithm::Llm => {
                // LLM matching — fallback to keyword for now
                // TODO: implement LLM-based classification
                warn!("LLM matching not yet implemented, falling back to keyword");
                self.keyword_match(query, &candidates, config.top_k, config.threshold)
            }
            SkillMatchingAlgorithm::Hybrid => {
                // Combine keyword + embedding results
                let keyword_results =
                    self.keyword_match(query, &candidates, config.top_k * 2, config.threshold);
                let vector_results: Vec<Skill> = registry
                    .find_matches(query)
                    .await
                    .into_iter()
                    .filter(|s| candidates.iter().any(|c| c.skill_id == s.skill_id))
                    .collect();

                // Deduplicate and merge
                let mut seen = std::collections::HashSet::new();
                let mut merged = Vec::new();
                for s in vector_results.iter().chain(keyword_results.iter()) {
                    if seen.insert(s.skill_id.clone()) {
                        merged.push(s.clone());
                    }
                }
                merged.truncate(config.top_k);
                merged
            }
        };

        // CH-08: record an activation decision (accepted=true — everything
        // `match_skills` returns was selected) per skill, labeled by the
        // matching backend actually used. Per-skill/per-backend counters give
        // the activation-recall half of the precision/recall pair; whether
        // the model actually *used* an activated skill's tools (the outcome
        // half, `record_skill_activation_outcome`) requires correlating this
        // decision against the run's later tool-call stream, which is a
        // separate, harder problem (candidate-vs-considered-but-rejected
        // visibility doesn't exist at this layer either) — deliberately
        // scope-cut for this pass, consistent with this phase's other
        // documented scope cuts (plan.md D-A..D-D).
        let backend = match config.algorithm {
            SkillMatchingAlgorithm::Keyword => "keyword",
            SkillMatchingAlgorithm::Embedding => "embedding",
            SkillMatchingAlgorithm::LocalEmbedding => "local_embedding",
            SkillMatchingAlgorithm::Llm => "llm",
            SkillMatchingAlgorithm::Hybrid => "hybrid",
        };
        for skill in &matched {
            crate::uar::telemetry::metrics::record_skill_activation(&skill.skill_id, backend, true);
        }
        matched
    }

    /// Simple keyword matching.
    fn keyword_match(
        &self,
        query: &str,
        candidates: &[Skill],
        top_k: usize,
        _threshold: f32,
    ) -> Vec<Skill> {
        let q = query.to_lowercase();
        let mut scored: Vec<(&Skill, f32)> = candidates
            .iter()
            .filter_map(|s| {
                let mut score = 0.0_f32;

                // Check keyword triggers
                for kw in &s.triggers.keywords {
                    if q.contains(&kw.to_lowercase()) {
                        score += 1.0;
                    }
                }

                // Check title/description
                if s.title.to_lowercase().contains(&q) {
                    score += 0.5;
                }
                if s.description.to_lowercase().contains(&q) {
                    score += 0.3;
                }

                if score > 0.0 { Some((s, score)) } else { None }
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored
            .into_iter()
            .take(top_k)
            .map(|(s, _)| s.clone())
            .collect()
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
    use crate::uar::runtime::skills::storage::DatabaseStorageProvider;

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
                .is_empty()
        );
        assert_eq!(
            service
                .match_skills_scoped("initial", Some("agent-a"), Some("conversation-b"))
                .await
                .len(),
            1
        );
        assert!(
            service
                .match_skills_scoped("initial", Some("agent-b"), Some("conversation-b"))
                .await
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
                .len(),
            1
        );
        assert!(
            service
                .match_skills_scoped("initial", Some("agent-a"), Some("conversation-b"))
                .await
                .is_empty()
        );
        assert_eq!(
            service
                .match_skills_scoped("initial", Some("agent-b"), Some("conversation-b"))
                .await
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
        assert_eq!(in_flight_binding.len(), 1);

        assert!(
            service
                .set_scoped_enabled(
                    "test-skill",
                    SkillScope::Conversation("conversation-a".to_string()),
                    false,
                )
                .await
        );

        assert_eq!(in_flight_binding.len(), 1);
        assert!(
            service
                .match_skills_scoped("initial", Some("agent-a"), Some("conversation-a"))
                .await
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
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].skill_id, "future-skill");

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
        assert_eq!(conversation_match.len(), 2);
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
                .is_empty()
        );
        assert_eq!(
            restarted
                .match_skills_scoped("initial", Some("agent-b"), Some("conversation-b"))
                .await
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
}
