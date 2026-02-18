//! Skill service — central coordinator for skills.
//!
//! Mirrors cherry-studio's `SkillService` pattern:
//! - Aggregates skills from multiple storage providers
//! - Configurable matching algorithms
//! - Per-agent skill bindings
//! - Script execution (sandboxed)

use super::registry::SkillRegistry;
use super::storage::SkillStorageProvider;
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

/// Central skill service coordinating storage + matching.
pub struct SkillService {
    /// In-memory skill index
    registry: Arc<RwLock<SkillRegistry>>,
    /// Registered storage providers
    providers: Vec<Arc<dyn SkillStorageProvider>>,
    /// Current matching configuration
    matching_config: RwLock<SkillMatchingConfig>,
    /// Per-agent skill bindings: agent_id -> Vec<skill_id>
    agent_skills: RwLock<HashMap<String, Vec<String>>>,
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
        }
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
                    registry.register_all(skills).await;
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
                    registry.register_all(skills).await;
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
        self.registry.write().await.toggle(id, enabled)
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
        match config.algorithm {
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
        }
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
}
