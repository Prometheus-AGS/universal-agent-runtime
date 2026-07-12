//! Capability-disabled governance facade.

use std::path::Path;

/// Runtime outcome retained even when Cedar policy evaluation is not compiled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolGovernanceDecision {
    /// Execute without an approval interrupt.
    Allow,
    /// Pause for human approval because runtime risk requires it.
    RequireApproval,
    /// Reserved for policy-enabled builds; never produced by this facade.
    Deny,
}

/// Explicit permit baseline used when `cedar-governance` is absent.
#[derive(Debug, Default)]
pub struct GovernanceEngine;

impl GovernanceEngine {
    /// Construct the capability-disabled permit baseline.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Construct the capability-disabled permit baseline.
    pub fn with_default_permit() -> anyhow::Result<Self> {
        Ok(Self)
    }

    /// Policy directories cannot be loaded without Cedar.
    pub async fn load_from_dir(_dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        anyhow::bail!("Cedar governance is unavailable: rebuild with `cedar-governance`")
    }

    /// No policy set exists in a capability-disabled build.
    pub async fn policy_count(&self) -> usize {
        0
    }

    /// Tool execution is permitted; runtime risk can still require approval.
    pub async fn is_tool_allowed(&self, _agent_id: &str, _tool_name: &str) -> bool {
        true
    }

    /// Preserve the non-policy risk approval gate.
    pub async fn tool_decision(
        &self,
        _agent_id: &str,
        _tool_name: &str,
        risk_requires_approval: bool,
    ) -> ToolGovernanceDecision {
        if risk_requires_approval {
            ToolGovernanceDecision::RequireApproval
        } else {
            ToolGovernanceDecision::Allow
        }
    }

    /// Generic actions are permitted because no policy engine is compiled.
    pub async fn is_allowed(&self, _agent_id: &str, _action: &str, _resource: &str) -> bool {
        true
    }

    /// Skill mutations are permitted because no policy engine is compiled.
    pub async fn is_skill_mutation_allowed(
        &self,
        _agent_id: &str,
        _action: &str,
        _skill_id: &str,
        _context_json: &str,
    ) -> bool {
        true
    }
}
