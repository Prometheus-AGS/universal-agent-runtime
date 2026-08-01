//! Skill service — central coordinator for skills.
//!
//! Mirrors cherry-studio's `SkillService` pattern:
//! - Aggregates skills from multiple storage providers
//! - Configurable matching algorithms
//! - Per-agent skill bindings
//! - Script execution (sandboxed)

use super::registry::SkillRegistry;
use super::storage::{SkillStorageProvider, StorageProviderKind};
use crate::uar::domain::skills::Skill;
use crate::uar::persistence::PersistenceLayer;
use crate::uar::runtime::matching::vector::VectorMatcher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    /// Per-agent skill bindings: agent_id -> Vec<skill_id>
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
    /// the in-memory registry. Used at startup by the builtin-loader; these
    /// skills are not persisted via storage providers.
    pub async fn register_builtins(&self, skills: Vec<crate::uar::domain::skills::Skill>) {
        let mut registry = self.registry.write().await;
        let count = skills.len();
        for s in skills {
            registry.register_loaded(s);
        }
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
        match self
            .update_skill(
                id,
                SkillUpdate {
                    enabled: Some(enabled),
                    ..SkillUpdate::default()
                },
            )
            .await
        {
            Ok(Some(_)) => true,
            Ok(None) => false,
            Err(e) => {
                warn!("Failed to toggle skill '{}': {:?}", id, e);
                false
            }
        }
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
        // Cedar governance gate: check if this mutation is allowed
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
            skill.enabled = enabled;
        }
        if let Some(execution_config) = update.execution_config {
            skill.execution_config = execution_config;
        }

        self.registry.write().await.register(skill.clone()).await;

        // Persist updates to writable filesystem skills so changes survive restarts.
        for provider in &self.providers {
            if provider.kind() == StorageProviderKind::Filesystem {
                if let Err(e) = provider.save_skill(&skill).await {
                    warn!(
                        "Could not write updated skill '{}' to filesystem provider '{}': {:?}",
                        skill.skill_id,
                        provider.name(),
                        e
                    );
                }
                break;
            }
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

    /// Match skills to a query for a specific agent.
    pub async fn match_skills(&self, query: &str, agent_id: Option<&str>) -> Vec<Skill> {
        let registry = self.registry.read().await;
        let config = self.matching_config.read().await;

        // Get candidate skills (agent-specific if agent_id provided)
        let candidates = if let Some(aid) = agent_id {
            self.get_enabled_skills_for_agent_inner(&registry, aid)
                .await
        } else {
            registry.list_enabled()
        };

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

    // --- Per-agent skill bindings ---

    /// Get skill IDs associated with an agent.
    pub async fn get_agent_skill_ids(&self, agent_id: &str) -> Vec<String> {
        self.agent_skills
            .read()
            .await
            .get(agent_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Set skill bindings for an agent.
    pub async fn set_agent_skills(&self, agent_id: &str, skill_ids: Vec<String>) {
        self.agent_skills
            .write()
            .await
            .insert(agent_id.to_string(), skill_ids);
    }

    /// Add a single skill to an agent's bindings.
    pub async fn add_skill_to_agent(&self, agent_id: &str, skill_id: &str) {
        let mut map = self.agent_skills.write().await;
        let entry = map.entry(agent_id.to_string()).or_default();
        if !entry.contains(&skill_id.to_string()) {
            entry.push(skill_id.to_string());
        }
    }

    /// Remove a single skill from an agent's bindings.
    pub async fn remove_skill_from_agent(&self, agent_id: &str, skill_id: &str) {
        let mut map = self.agent_skills.write().await;
        if let Some(entry) = map.get_mut(agent_id) {
            entry.retain(|id| id != skill_id);
        }
    }

    /// Get enabled skills for an agent (intersection of agent bindings and globally enabled).
    pub async fn get_enabled_skills_for_agent(&self, agent_id: &str) -> Vec<Skill> {
        let registry = self.registry.read().await;
        self.get_enabled_skills_for_agent_inner(&registry, agent_id)
            .await
    }

    async fn get_enabled_skills_for_agent_inner(
        &self,
        registry: &SkillRegistry,
        agent_id: &str,
    ) -> Vec<Skill> {
        let agent_skill_ids = self.get_agent_skill_ids(agent_id).await;
        if agent_skill_ids.is_empty() {
            // If no specific bindings, return all enabled skills
            return registry.list_enabled();
        }

        registry
            .list_enabled()
            .into_iter()
            .filter(|s| agent_skill_ids.contains(&s.skill_id))
            .collect()
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
    use crate::uar::domain::skills::{SkillConstraints, SkillTriggers};

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
}
