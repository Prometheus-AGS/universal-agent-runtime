//! Skill registry — in-memory index of loaded skills.
//!
//! The registry aggregates skills from all enabled storage providers
//! and provides fast lookups.

use crate::uar::domain::skills::Skill;
use crate::uar::persistence::PersistenceLayer;
use crate::uar::runtime::matching::vector::VectorMatcher;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info};

/// In-memory skill index.
#[derive(Clone)]
pub struct SkillRegistry {
    skills: HashMap<String, Skill>,
    persistence: Option<Arc<dyn PersistenceLayer>>,
    vector_matcher: Option<Arc<VectorMatcher>>,
}

impl std::fmt::Debug for SkillRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkillRegistry")
            .field("skills_count", &self.skills.len())
            .field("persistence", &self.persistence.is_some())
            .field("vector_matcher", &self.vector_matcher.is_some())
            .finish()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new(None, None)
    }
}

impl SkillRegistry {
    pub fn new(
        persistence: Option<Arc<dyn PersistenceLayer>>,
        vector_matcher: Option<Arc<VectorMatcher>>,
    ) -> Self {
        Self {
            skills: HashMap::new(),
            persistence,
            vector_matcher,
        }
    }

    /// Register a skill in the in-memory index.
    pub async fn register(&mut self, skill: Skill) {
        // Persist + embed if backends available
        if let (Some(db), Some(vm)) = (&self.persistence, &self.vector_matcher) {
            let text = format!("{}: {}", skill.title, skill.description);
            match vm.embed_batch(vec![text]).await {
                Ok(embeddings) => {
                    if let Some(emb) = embeddings.first()
                        && let Err(e) = db.save_skill(&skill, emb).await
                    {
                        error!("Failed to persist skill {}: {:?}", skill.skill_id, e);
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to generate embedding for skill {}: {:?}",
                        skill.skill_id, e
                    );
                }
            }
        }

        self.skills.insert(skill.skill_id.clone(), skill);
    }

    /// Register a skill that has already been loaded from durable storage.
    ///
    /// Startup hydration must not re-persist or eagerly embed every skill. In
    /// addition to duplicating storage writes, doing so makes the UAR listener
    /// depend on optional ONNX initialization. Embeddings are created only by
    /// explicit mutations or when an embedding-backed operation is requested.
    pub fn register_loaded(&mut self, skill: Skill) {
        self.skills.insert(skill.skill_id.clone(), skill);
    }

    /// Bulk register skills.
    pub async fn register_all(&mut self, skills: Vec<Skill>) {
        for skill in skills {
            self.register(skill).await;
        }
    }

    /// Hydrate a batch that already belongs to a configured storage provider.
    pub fn register_all_loaded(&mut self, skills: Vec<Skill>) {
        for skill in skills {
            self.register_loaded(skill);
        }
    }

    /// Look up a skill by ID.
    pub fn get(&self, id: &str) -> Option<&Skill> {
        self.skills.get(id)
    }

    /// List all registered skills.
    pub fn list(&self) -> Vec<Skill> {
        self.skills.values().cloned().collect()
    }

    /// List only enabled skills.
    pub fn list_enabled(&self) -> Vec<Skill> {
        self.skills
            .values()
            .filter(|s| s.enabled)
            .cloned()
            .collect()
    }

    /// Toggle a skill's enabled state.
    pub fn toggle(&mut self, id: &str, enabled: bool) -> bool {
        if let Some(skill) = self.skills.get_mut(id) {
            skill.enabled = enabled;
            info!("Skill '{}' enabled={}", id, enabled);
            true
        } else {
            false
        }
    }

    /// Remove a skill from the registry.
    pub fn remove(&mut self, id: &str) -> Option<Skill> {
        self.skills.remove(id)
    }

    /// Clear all skills.
    pub fn clear(&mut self) {
        self.skills.clear();
    }

    /// Get the count of registered skills.
    pub fn len(&self) -> usize {
        self.skills.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }

    /// Find skills matching a query (fallback keyword search).
    pub fn find_by_keyword(&self, query: &str) -> Vec<Skill> {
        let q = query.to_lowercase();
        self.skills
            .values()
            .filter(|s| {
                s.title.to_lowercase().contains(&q)
                    || s.description.to_lowercase().contains(&q)
                    || s.triggers
                        .keywords
                        .iter()
                        .any(|k| k.to_lowercase().contains(&q))
            })
            .cloned()
            .collect()
    }

    /// Vector-based skill matching (uses persistence + VectorMatcher).
    pub async fn find_matches(&self, query: &str) -> Vec<Skill> {
        if let (Some(db), Some(vm)) = (&self.persistence, &self.vector_matcher) {
            match vm.embed_batch(vec![query.to_string()]).await {
                Ok(embeddings) => {
                    if let Some(q_vec) = embeddings.first() {
                        match db.search_skills(q_vec, 5).await {
                            Ok(matches) => {
                                return matches.into_iter().map(|m| m.skill).collect();
                            }
                            Err(e) => {
                                error!("Skill search failed: {:?}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Query embedding failed: {:?}", e);
                }
            }
        }

        // Fallback to keyword match
        self.find_by_keyword(query)
    }
}
