//! The embedder-facing skill API (R4).
//!
//! # Why this module exists
//!
//! Before this, an embedder wanting to list or install a skill had to reach
//! into `uar::runtime::skills::service::SkillService` — an internal type whose
//! signature is free to change. Every path was `pub` all the way down, so the
//! crate had no seam between "what we support" and "what happens to be
//! reachable".
//!
//! This module is that seam. It is deliberately **narrow**: five verbs over
//! [`Skill`], nothing else. An embedded host — a mobile app, a desktop shell, a
//! test — should not need to know that a `SkillRegistry` exists, that
//! persistence is optional, or that matching has a configurable algorithm.
//!
//! # What this is NOT
//!
//! It is not a re-export of `SkillService`. Handing back the internal type
//! would name it in the public API and re-create the coupling this module
//! removes. The facade owns an `Arc<SkillService>` privately and exposes only
//! the operations R4 names.
//!
//! # Example
//!
//! ```no_run
//! use universal_agent_runtime::skills_api::SkillsApi;
//! # async fn example(api: SkillsApi) -> anyhow::Result<()> {
//! // Everything an embedder needs, without touching runtime internals.
//! let all = api.list().await;
//! if let Some(skill) = api.get("some-skill").await {
//!     api.toggle(&skill.skill_id, false).await;
//! }
//! let matches = api.query("summarise a document").await;
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;

use crate::uar::domain::skills::Skill;
use crate::uar::runtime::skills::SkillService;

/// The public skill surface for embedders.
///
/// Obtain one from [`crate::embedded::EmbeddedRuntime::skills`]. Cloning is
/// cheap — it is an `Arc` internally — so a host may hand copies to whatever
/// subsystem needs skill access.
#[derive(Clone)]
pub struct SkillsApi {
    inner: Arc<SkillService>,
}

impl std::fmt::Debug for SkillsApi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Deliberately opaque: the point of the facade is that the internal
        // service is not part of the contract, so it does not appear here.
        f.debug_struct("SkillsApi").finish_non_exhaustive()
    }
}

impl SkillsApi {
    /// Wrap a service. Crate-internal: embedders go through
    /// [`crate::embedded::EmbeddedRuntime::skills`], which guarantees the
    /// service was built and initialised correctly.
    pub(crate) fn new(inner: Arc<SkillService>) -> Self {
        Self { inner }
    }

    /// Every skill known to the runtime, enabled or not.
    ///
    /// Includes pack builtins and anything installed at runtime. Use
    /// [`Self::list_enabled`] when presenting choices to a user.
    pub async fn list(&self) -> Vec<Skill> {
        self.inner.get_skills().await
    }

    /// Only skills currently enabled.
    ///
    /// A builtin that has been switched off is absent here but still present in
    /// [`Self::list`] — pack skills are never deleted, only disabled, so the two
    /// lists diverge rather than one shrinking permanently.
    pub async fn list_enabled(&self) -> Vec<Skill> {
        self.inner.get_enabled_skills().await
    }

    /// One skill by id, or `None`.
    pub async fn get(&self, skill_id: &str) -> Option<Skill> {
        self.inner
            .get_skills()
            .await
            .into_iter()
            .find(|s| s.skill_id == skill_id)
    }

    /// Install a skill, registering it in the database when persistence is
    /// configured.
    ///
    /// This is the path R4 calls out for *dynamically created* skills: a host
    /// that generates a skill at runtime can persist it here and have it appear
    /// in the admin UI and REST API like any other.
    ///
    /// # Errors
    ///
    /// Propagates validation and storage failures from the underlying service.
    pub async fn install(&self, skill: Skill) -> anyhow::Result<Skill> {
        self.inner.create_skill(skill).await
    }

    /// Enable or disable a skill. Returns `false` when no such skill exists.
    ///
    /// This is the supported way to "remove" a pack builtin. Deletion is
    /// refused at the database by a trigger, so disabling is not a soft
    /// alternative to deleting — it is the only option.
    pub async fn toggle(&self, skill_id: &str, enabled: bool) -> bool {
        self.inner.toggle_skill(skill_id, enabled).await
    }

    /// Skills matching a natural-language query, best first.
    ///
    /// Uses vector matching when an embedding backend is configured and falls
    /// back to keyword matching otherwise — an embedded host with no embedder
    /// still gets useful results rather than an empty list.
    pub async fn query(&self, query: &str) -> Vec<Skill> {
        self.inner.match_skills(query, None).await
    }
}
