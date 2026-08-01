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

    /// Register a skill in the in-memory index, and persist it when a database
    /// is configured.
    ///
    /// # Persistence is NOT gated on the embedder
    ///
    /// This previously read `if let (Some(db), Some(vm)) = …`, so a host with a
    /// database but no [`VectorMatcher`] persisted **nothing** — silently, with
    /// no error, because the tuple pattern simply did not match. Measured on the
    /// embedded path: 3 skills discovered, **0 rows written**.
    ///
    /// That is the embedded case, which has no embedding service by definition,
    /// so the platform R1 exists for was exactly the one that never reached the
    /// database. Any consumer reading skills from persistence — the admin UI,
    /// the REST API, a mobile host — saw an empty catalogue from a process that
    /// looked healthy.
    ///
    /// An embedding is an enrichment for vector search. Failing to compute one
    /// is not a reason to discard the skill, so the two concerns are now
    /// independent: persist whenever there is a database, and attach an
    /// embedding when one can be produced.
    pub async fn register(&mut self, skill: Skill) {
        if let Some(db) = &self.persistence {
            // Best-effort embedding. `None` (no matcher) and `Err` (matcher
            // failed) both degrade to persisting without one, rather than to
            // dropping the skill.
            let embedding: Vec<f32> = match &self.vector_matcher {
                Some(vm) => {
                    let text = format!("{}: {}", skill.title, skill.description);
                    match vm.embed_batch(vec![text]).await {
                        Ok(embeddings) => embeddings.into_iter().next().unwrap_or_default(),
                        Err(e) => {
                            error!(
                                "Failed to generate embedding for skill {}: {:?} — \
                                 persisting without one; vector search will not match it \
                                 until it is re-embedded",
                                skill.skill_id, e
                            );
                            Vec::new()
                        }
                    }
                }
                None => Vec::new(),
            };

            if let Err(e) = db.save_skill(&skill, &embedding).await {
                error!("Failed to persist skill {}: {:?}", skill.skill_id, e);
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
    ///
    /// There is a second reason this must not route through [`Self::register`]:
    /// `save_skill` upserts `embedding = EXCLUDED.embedding`, so a host with no
    /// [`VectorMatcher`] would overwrite a good stored embedding with an empty
    /// one on **every restart** — silently degrading vector search on a
    /// database that was previously fine. Load and save are separate
    /// operations, not one operation that happens to be idempotent when an
    /// embedder is configured.
    pub fn register_loaded(&mut self, skill: Skill) {
        self.skills.insert(skill.skill_id.clone(), skill);
    }

    /// Bulk register skills, persisting each one.
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
