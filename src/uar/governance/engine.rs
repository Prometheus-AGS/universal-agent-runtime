//! Core governance engine wrapping Cedar's [`Authorizer`].
//!
//! [`GovernanceEngine`] loads Cedar policy files from a directory, evaluates
//! authorization requests, and supports runtime policy reload.

use std::path::{Path, PathBuf};

use cedar_policy::{Authorizer, Decision, Entities, PolicySet, Request, Response};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::policy;

// ---------------------------------------------------------------------------
// GovernanceEngine
// ---------------------------------------------------------------------------

/// Authorization engine backed by Cedar policies.
///
/// Thread-safe — all methods take `&self` and the policy set is behind a
/// [`RwLock`] to allow runtime reload without downtime.
///
/// # Example
///
/// ```rust,ignore
/// let engine = GovernanceEngine::load_from_dir("policies").await?;
/// let allowed = engine.is_tool_allowed("agent-1", "web_search").await;
/// assert!(allowed);
/// ```
pub struct GovernanceEngine {
    /// Cedar authorizer (stateless, reusable).
    authorizer: Authorizer,
    /// Currently loaded policy set.
    policies: RwLock<PolicySet>,
    /// Entities for authorization (can be extended at runtime).
    entities: RwLock<Entities>,
    /// Directory from which policies were loaded (for reload).
    policy_dir: Option<PathBuf>,
}

impl std::fmt::Debug for GovernanceEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GovernanceEngine")
            .field("policy_dir", &self.policy_dir)
            .finish()
    }
}

impl GovernanceEngine {
    /// Create a new governance engine with an empty (permissive) policy set.
    ///
    /// With no policies loaded, all requests are implicitly denied by Cedar.
    /// Use [`Self::with_default_permit`] for a permissive default.
    #[must_use]
    pub fn new() -> Self {
        Self {
            authorizer: Authorizer::new(),
            policies: RwLock::new(PolicySet::new()),
            entities: RwLock::new(Entities::empty()),
            policy_dir: None,
        }
    }

    /// Create a governance engine with a default "permit all" policy.
    ///
    /// This is the recommended starting point — agents can execute any tool
    /// until restrictive policies are explicitly added.
    pub fn with_default_permit() -> anyhow::Result<Self> {
        let policy_src = r"
            permit (
                principal,
                action,
                resource
            );
        ";

        let policies: PolicySet = policy_src
            .parse()
            .map_err(|e| anyhow::anyhow!("Failed to parse default policy: {e}"))?;

        Ok(Self {
            authorizer: Authorizer::new(),
            policies: RwLock::new(policies),
            entities: RwLock::new(Entities::empty()),
            policy_dir: None,
        })
    }

    /// Load policies from a directory of `.cedar` files.
    ///
    /// All `*.cedar` files in the directory are concatenated and parsed as a
    /// single policy set.
    pub async fn load_from_dir(dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        let dir = dir.as_ref().to_path_buf();

        let policies = Self::read_policies_from_dir(&dir).await?;
        let policy_count = policies.policies().count();

        info!(
            dir = %dir.display(),
            policy_count = policy_count,
            "Governance policies loaded"
        );

        Ok(Self {
            authorizer: Authorizer::new(),
            policies: RwLock::new(policies),
            entities: RwLock::new(Entities::empty()),
            policy_dir: Some(dir),
        })
    }

    /// Reload policies from the configured directory.
    ///
    /// This is a hot-reload: the new policies take effect immediately for
    /// all subsequent authorization checks, without restarting the server.
    pub async fn reload(&self) -> anyhow::Result<()> {
        let dir = self
            .policy_dir
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No policy directory configured for reload"))?;

        let new_policies = Self::read_policies_from_dir(dir).await?;
        let count = new_policies.policies().count();

        let mut policies = self.policies.write().await;
        *policies = new_policies;

        info!(
            dir = %dir.display(),
            policy_count = count,
            "Governance policies reloaded"
        );

        Ok(())
    }

    /// Check whether a tool execution is allowed.
    ///
    /// Shorthand for building a tool execution request and evaluating it.
    pub async fn is_tool_allowed(&self, agent_id: &str, tool_name: &str) -> bool {
        match policy::tool_execution_request(agent_id, policy::actions::EXECUTE_TOOL, tool_name) {
            Ok(request) => self.evaluate(&request).await == Decision::Allow,
            Err(e) => {
                error!(
                    agent_id = %agent_id,
                    tool = %tool_name,
                    error = %e,
                    "Failed to build governance request — denying by default"
                );
                false
            }
        }
    }

    /// Check whether a generic action is allowed.
    pub async fn is_allowed(&self, agent_id: &str, action: &str, resource: &str) -> bool {
        match policy::tool_execution_request(agent_id, action, resource) {
            Ok(request) => self.evaluate(&request).await == Decision::Allow,
            Err(e) => {
                error!(error = %e, "Failed to build governance request — denying by default");
                false
            }
        }
    }

    /// Check whether a skill mutation is allowed.
    ///
    /// Skill mutations carry rich context (environment, confidence scores,
    /// validation status) that Cedar policies evaluate against. Used by the
    /// self-learning pipeline to gate prompt optimization, skill generation,
    /// and skill promotion.
    ///
    /// # Arguments
    ///
    /// * `agent_id` — The agent/optimizer attempting the mutation.
    /// * `action` — One of: `skill.mutate`, `skill.generate`, `skill.promote`, `trace.capture`.
    /// * `skill_id` — The skill being operated on.
    /// * `context_json` — JSON object with mutation context. Policies evaluate
    ///   fields like `environment`, `confidence_score`, `validation_passed`,
    ///   `human_approved`, `test_pass_rate`, `governance_consent`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let allowed = engine.is_skill_mutation_allowed(
    ///     "pmpo-optimizer",
    ///     "skill.mutate",
    ///     "iterative-evolver",
    ///     r#"{"environment": "staging", "validation_passed": true}"#,
    /// ).await;
    /// ```
    pub async fn is_skill_mutation_allowed(
        &self,
        agent_id: &str,
        action: &str,
        skill_id: &str,
        context_json: &str,
    ) -> bool {
        match policy::skill_mutation_request(agent_id, action, skill_id, context_json) {
            Ok(request) => {
                let response = self.evaluate_full(&request).await;
                let decision = response.decision();

                if decision == Decision::Deny {
                    info!(
                        agent_id = %agent_id,
                        action = %action,
                        skill_id = %skill_id,
                        "Skill mutation DENIED by Cedar policy"
                    );
                }

                decision == Decision::Allow
            }
            Err(e) => {
                error!(
                    agent_id = %agent_id,
                    action = %action,
                    skill_id = %skill_id,
                    error = %e,
                    "Failed to build skill mutation request — denying by default"
                );
                false
            }
        }
    }

    /// Evaluate a Cedar request against the current policy set.
    ///
    /// Returns the full [`Response`] for detailed diagnostics.
    pub async fn evaluate_full(&self, request: &Request) -> Response {
        let policies = self.policies.read().await;
        let entities = self.entities.read().await;
        self.authorizer.is_authorized(request, &policies, &entities)
    }

    /// Evaluate a Cedar request, returning only the decision.
    pub async fn evaluate(&self, request: &Request) -> Decision {
        let response = self.evaluate_full(request).await;
        let decision = response.decision();

        debug!(
            decision = ?decision,
            "Governance policy evaluation"
        );

        // Log any policy errors/warnings
        for diagnostic in response.diagnostics().errors() {
            warn!(error = %diagnostic, "Cedar policy evaluation error");
        }

        decision
    }

    /// Get the number of currently loaded policies.
    pub async fn policy_count(&self) -> usize {
        self.policies.read().await.policies().count()
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Read and parse all `.cedar` files from a directory.
    async fn read_policies_from_dir(dir: &Path) -> anyhow::Result<PolicySet> {
        if !dir.exists() {
            info!(
                dir = %dir.display(),
                "Policy directory does not exist — using empty policy set"
            );
            return Ok(PolicySet::new());
        }

        let mut combined_source = String::new();

        let mut entries = tokio::fs::read_dir(dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "cedar") {
                match tokio::fs::read_to_string(&path).await {
                    Ok(content) => {
                        debug!(file = %path.display(), "Loading Cedar policy");
                        combined_source.push_str(&content);
                        combined_source.push('\n');
                    }
                    Err(e) => {
                        warn!(
                            file = %path.display(),
                            error = %e,
                            "Failed to read policy file — skipping"
                        );
                    }
                }
            }
        }

        if combined_source.is_empty() {
            info!("No Cedar policy files found — using empty policy set");
            return Ok(PolicySet::new());
        }

        let policies: PolicySet = combined_source
            .parse()
            .map_err(|e| anyhow::anyhow!("Failed to parse Cedar policies: {e}"))?;

        Ok(policies)
    }
}

impl Default for GovernanceEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_permit_allows_all() {
        let engine = GovernanceEngine::with_default_permit().unwrap();
        assert!(engine.is_tool_allowed("agent-1", "web_search").await);
        assert!(engine.is_tool_allowed("agent-2", "code_exec").await);
    }

    #[tokio::test]
    async fn test_empty_engine_denies_all() {
        let engine = GovernanceEngine::new();
        assert!(!engine.is_tool_allowed("agent-1", "web_search").await);
    }

    #[tokio::test]
    async fn test_restrictive_policy() {
        let policy_src = r#"
            // Only agent-1 can execute tools
            permit (
                principal == Agent::"agent-1",
                action == Action::"execute_tool",
                resource
            );
        "#;

        let policies: PolicySet = policy_src.parse().unwrap();
        let engine = GovernanceEngine {
            authorizer: Authorizer::new(),
            policies: RwLock::new(policies),
            entities: RwLock::new(Entities::empty()),
            policy_dir: None,
        };

        assert!(engine.is_tool_allowed("agent-1", "web_search").await);
        assert!(!engine.is_tool_allowed("agent-2", "web_search").await);
    }

    #[tokio::test]
    async fn test_deny_specific_tool() {
        let policy_src = r#"
            // Allow everything by default
            permit (principal, action, resource);

            // But deny dangerous_tool
            forbid (
                principal,
                action == Action::"execute_tool",
                resource == Tool::"dangerous_tool"
            );
        "#;

        let policies: PolicySet = policy_src.parse().unwrap();
        let engine = GovernanceEngine {
            authorizer: Authorizer::new(),
            policies: RwLock::new(policies),
            entities: RwLock::new(Entities::empty()),
            policy_dir: None,
        };

        assert!(engine.is_tool_allowed("agent-1", "web_search").await);
        assert!(!engine.is_tool_allowed("agent-1", "dangerous_tool").await);
    }
}
