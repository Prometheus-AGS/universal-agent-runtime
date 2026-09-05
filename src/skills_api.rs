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
    /// `None` = defer to `UAR_REGISTER_GENERATED_SKILLS`; `Some` = explicit
    /// override. Never defaults to enabled — see [`SkillsApi::install_generated`].
    register_generated: Option<bool>,
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
        Self {
            inner,
            register_generated: None,
        }
    }

    /// Build a facade directly from a service.
    ///
    /// Production hosts go through [`crate::embedded::EmbeddedRuntime::skills`],
    /// which guarantees the service was built and initialised correctly. This
    /// exists so an **integration test** — which is a separate crate and cannot
    /// reach `pub(crate)` — can exercise the facade without standing up a whole
    /// runtime (and, notably, without supplying an LLM driver it has no use for).
    #[must_use]
    pub fn for_test(inner: Arc<SkillService>) -> Self {
        Self::new(inner)
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
    /// This is the explicit path: the caller asked for an install, so it
    /// happens. For skills a tool *generates* on the fly, see
    /// [`Self::install_generated`], which is gated.
    ///
    /// # Errors
    ///
    /// Propagates validation and storage failures from the underlying service.
    pub async fn install(&self, skill: Skill) -> anyhow::Result<Skill> {
        self.inner.create_skill(skill).await
    }

    /// Register a skill that was **generated at runtime** — only when the host
    /// has opted in.
    ///
    /// # Why this is separate from [`Self::install`], and off by default
    ///
    /// R4 says dynamic skill creation should register in the database
    /// **optionally**. "Optionally" has to live in the default, not just in the
    /// documentation: a generator that writes to the database by default
    /// silently grows a user's skill catalogue with artifacts they never asked
    /// to keep, and a `skills` table that fills up on its own is far harder to
    /// diagnose than one that stays empty.
    ///
    /// So the default is **off**. Opt in with either:
    ///
    /// - `UAR_REGISTER_GENERATED_SKILLS=true` (or `1`), or
    /// - [`Self::with_generated_registration`] for programmatic control, which
    ///   an embedded host can set without touching process environment.
    ///
    /// # Returns
    ///
    /// `Ok(None)` when registration is **not** enabled — this is a normal,
    /// successful outcome, not a failure. `Ok(Some(skill))` when it is.
    ///
    /// # Errors
    ///
    /// Propagates validation and storage failures, but only on the enabled
    /// path — the disabled path cannot fail because it does nothing.
    pub async fn install_generated(&self, skill: Skill) -> anyhow::Result<Option<Skill>> {
        if !self.generated_registration_enabled() {
            return Ok(None);
        }
        self.inner.create_skill(skill).await.map(Some)
    }

    /// Is registration of generated skills currently enabled?
    ///
    /// Explicit override wins; otherwise the environment decides; otherwise
    /// **false**.
    #[must_use]
    pub fn generated_registration_enabled(&self) -> bool {
        if let Some(explicit) = self.register_generated {
            return explicit;
        }
        std::env::var("UAR_REGISTER_GENERATED_SKILLS")
            .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
            .unwrap_or(false)
    }

    /// Set generated-skill registration explicitly, overriding the environment.
    ///
    /// For embedded hosts that decide this per-session rather than per-process
    /// — a mobile app cannot usefully set an env var on itself.
    #[must_use]
    pub fn with_generated_registration(mut self, enabled: bool) -> Self {
        self.register_generated = Some(enabled);
        self
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
        self.inner.match_skills(query, None).await.accepted_skills()
    }
}
