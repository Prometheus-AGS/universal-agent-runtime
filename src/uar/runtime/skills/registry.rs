//! Skill registry — in-memory index of loaded skills.
//!
//! The registry aggregates skills from all enabled storage providers
//! and provides fast lookups.

use crate::uar::domain::skills::Skill;
#[cfg(test)]
use crate::uar::domain::skills::SkillMatch;
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
    const VECTOR_MATCH_LIMIT: usize = 5;

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
        let embedding = self.embedding_for(&skill).await;
        if let Some(db) = &self.persistence
            && let Err(e) = db.save_skill(&skill, &embedding).await
        {
            error!("Failed to persist skill {}: {:?}", skill.skill_id, e);
        }

        self.skills.insert(skill.skill_id.clone(), skill);
    }

    /// Register and persist a skill, returning persistence failures to callers
    /// that must not claim reconciliation succeeded after a failed write.
    pub(crate) async fn register_checked(&mut self, skill: Skill) -> anyhow::Result<()> {
        let embedding = self.embedding_for(&skill).await;
        if let Some(db) = &self.persistence {
            db.save_skill(&skill, &embedding).await?;
        }
        self.skills.insert(skill.skill_id.clone(), skill);
        Ok(())
    }

    /// Register and persist changed skills without waiting for optional vector
    /// enrichment. Startup reconciliation must finish before readiness even
    /// when the standard directory contains hundreds of skills and the local
    /// embedding model needs a cold start.
    pub(crate) async fn register_checked_batch_without_embeddings(
        &mut self,
        skills: Vec<Skill>,
    ) -> anyhow::Result<()> {
        for skill in skills {
            if let Some(db) = &self.persistence {
                db.save_skill(&skill, &[]).await?;
            }
            self.skills.insert(skill.skill_id.clone(), skill);
        }
        Ok(())
    }

    /// Read every durable skill, including tombstoned records.
    pub(crate) async fn list_persisted(&self) -> anyhow::Result<Vec<Skill>> {
        match &self.persistence {
            Some(db) => db.list_skills().await,
            None => Ok(Vec::new()),
        }
    }

    async fn embedding_for(&self, skill: &Skill) -> Vec<f32> {
        // Best-effort embedding. `None` (no matcher) and `Err` (matcher failed)
        // both degrade to persisting without one, rather than dropping the skill.
        let Some(vm) = &self.vector_matcher else {
            return Vec::new();
        };
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

    /// Register built-ins while preserving durable enabled-state configuration.
    ///
    /// Pack metadata is refreshed from `skills`, but a persisted row remains
    /// authoritative for its global and scoped enabled values. The durable
    /// catalogue is read once for the batch so startup does not perform one
    /// full-table read per built-in.
    pub async fn register_builtins(&mut self, mut skills: Vec<Skill>) {
        if let Some(db) = &self.persistence {
            let stored = match db.list_skills().await {
                Ok(stored) => stored
                    .into_iter()
                    .map(|skill| (skill.skill_id.clone(), skill))
                    .collect::<HashMap<_, _>>(),
                Err(error) => {
                    error!(
                        ?error,
                        "Failed to load stored skill configuration; refusing to overwrite built-ins"
                    );
                    return;
                }
            };
            for skill in &mut skills {
                if let Some(existing) = stored.get(&skill.skill_id) {
                    skill.enabled = existing.enabled;
                    skill.scoped_config.clone_from(&existing.scoped_config);
                }
            }
        }

        self.register_all(skills).await;
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
        let mut skills = self
            .skills
            .values()
            .filter(|skill| !skill.tombstoned)
            .cloned()
            .collect::<Vec<_>>();
        skills.sort_by(|left, right| left.skill_id.cmp(&right.skill_id));
        skills
    }

    /// List only enabled skills.
    pub fn list_enabled(&self) -> Vec<Skill> {
        self.skills
            .values()
            .filter(|skill| !skill.tombstoned && skill.enabled)
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
        self.skills
            .values()
            .filter(|skill| !skill.tombstoned)
            .count()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.skills.values().all(|skill| skill.tombstoned)
    }

    /// Find skills matching a query (fallback keyword search).
    pub fn find_by_keyword(&self, query: &str) -> Vec<Skill> {
        let q = query.to_lowercase();
        self.skills
            .values()
            .filter(|s| {
                !s.tombstoned
                    && (s.title.to_lowercase().contains(&q)
                        || s.description.to_lowercase().contains(&q)
                        || s.triggers
                            .keywords
                            .iter()
                            .any(|k| k.to_lowercase().contains(&q)))
            })
            .cloned()
            .collect()
    }

    /// Vector-based skill matching (uses persistence + VectorMatcher).
    pub async fn find_matches(&self, query: &str) -> Vec<Skill> {
        self.find_candidates(query)
            .await
            .into_iter()
            .map(|candidate| candidate.skill)
            .collect()
    }

    /// Preserve real similarity scores for thresholding and shadow telemetry.
    pub async fn find_candidates(
        &self,
        query: &str,
    ) -> Vec<crate::uar::domain::skills::SkillCandidate> {
        if let (Some(db), Some(vm)) = (&self.persistence, &self.vector_matcher) {
            match vm.embed_batch(vec![query.to_string()]).await {
                Ok(embeddings) => {
                    if let Some(q_vec) = embeddings.first() {
                        match db.search_skills(q_vec, self.vector_candidate_limit()).await {
                            Ok(matches) => {
                                return matches
                                    .into_iter()
                                    .filter(|candidate| !candidate.skill.tombstoned)
                                    .map(|candidate| crate::uar::domain::skills::SkillCandidate {
                                        skill: candidate.skill,
                                        score: candidate.score,
                                    })
                                    .collect();
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
        self.list()
            .iter()
            .map(|skill| crate::uar::domain::skills::SkillCandidate::keyword(skill, query))
            .collect()
    }

    fn vector_candidate_limit(&self) -> usize {
        self.skills.len().max(Self::VECTOR_MATCH_LIMIT)
    }

    #[cfg(test)]
    fn visible_vector_matches(matches: Vec<SkillMatch>) -> Vec<Skill> {
        matches
            .into_iter()
            .map(|candidate| candidate.skill)
            .filter(|skill| !skill.tombstoned)
            .take(Self::VECTOR_MATCH_LIMIT)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tombstoned_skill_is_hidden_but_retrievable_for_restore() {
        let mut registry = SkillRegistry::default();
        let active = Skill {
            skill_id: "active".to_string(),
            title: "Shared Keyword".to_string(),
            description: "active".to_string(),
            enabled: true,
            ..Skill::default()
        };
        let tombstoned = Skill {
            skill_id: "removed".to_string(),
            title: "Shared Keyword".to_string(),
            description: "removed".to_string(),
            tombstoned: true,
            ..Skill::default()
        };
        registry.register_loaded(active);
        registry.register_loaded(tombstoned);

        assert_eq!(registry.list().len(), 1);
        assert_eq!(registry.list_enabled().len(), 1);
        assert_eq!(registry.find_by_keyword("shared").len(), 1);
        assert_eq!(registry.len(), 1);
        assert!(registry.get("removed").is_some());
    }

    #[test]
    fn vector_candidates_include_tombstones_before_visibility_filtering() {
        let mut registry = SkillRegistry::default();
        let mut matches = Vec::new();
        for index in 0..SkillRegistry::VECTOR_MATCH_LIMIT {
            let skill = Skill {
                skill_id: format!("removed-{index}"),
                tombstoned: true,
                ..Skill::default()
            };
            registry.register_loaded(skill.clone());
            matches.push(SkillMatch { skill, score: 1.0 });
        }
        let active = Skill {
            skill_id: "active".to_string(),
            ..Skill::default()
        };
        registry.register_loaded(active.clone());
        matches.push(SkillMatch {
            skill: active,
            score: 0.5,
        });

        assert_eq!(registry.vector_candidate_limit(), 6);
        assert_eq!(
            SkillRegistry::visible_vector_matches(matches)
                .into_iter()
                .map(|skill| skill.skill_id)
                .collect::<Vec<_>>(),
            vec!["active"]
        );
    }
}
