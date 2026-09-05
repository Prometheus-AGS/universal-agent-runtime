//! UAR-to-UAR delegation contract carried in standard A2A metadata.
//! Authentication remains transport-owned; these values contain no secrets.

use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::uar::domain::policy::{
    ChatMode, RUN_POLICY_VERSION, RunPolicy, SelectionMode, ToolApprovalPolicy,
};
use crate::uar::runtime::thread::policy_intersection::{SandboxPermissions, ThreadBudgets};

#[cfg(test)]
#[path = "contract_presentation_tests.rs"]
mod presentation_tests;

pub const UAR_DELEGATION_CONTRACT_VERSION: u32 = 2;
pub const UAR_DELEGATION_CONTRACT_METADATA: &str = "uar.delegation_contract";
pub const UAR_DELEGATION_ACK_METADATA: &str = "uar.delegation_ack";
pub const UAR_USAGE_METADATA: &str = "uar.usage";
pub const UAR_CLEANUP_CLOSED_METADATA: &str = "uar.cleanup_closed";

/// Presence-preserving policy wire form. Older contracts did not contain a
/// Presentation ceiling; their typed serialization must retain its old digest.
#[derive(Debug, Clone, PartialEq)]
pub struct UarDelegationPolicy {
    policy: RunPolicy,
    presentation_ceiling_present: bool,
}

impl From<RunPolicy> for UarDelegationPolicy {
    fn from(policy: RunPolicy) -> Self {
        Self {
            policy,
            presentation_ceiling_present: true,
        }
    }
}

impl UarDelegationPolicy {
    /// Use the historical wire form only when no new authority or output
    /// restriction needs to cross the peer boundary. Never downgrade on retry.
    pub(crate) fn for_peer(
        mut policy: RunPolicy,
        negotiation: &Option<crate::uar::a2ui::presentation_selection::PresentationNegotiation>,
    ) -> Self {
        let legacy = negotiation.is_none()
            && policy.presentations.mode == SelectionMode::None
            && policy.presentations.ids.is_empty()
            && policy.presentations.denied_ids.is_empty();
        if legacy {
            // Match typed deserialization of the omitted field for subsequent
            // contract equality. execution_policy applies the target-local None.
            policy.presentations = Default::default();
        }
        Self {
            policy,
            presentation_ceiling_present: !legacy,
        }
    }
}

impl std::ops::Deref for UarDelegationPolicy {
    type Target = RunPolicy;
    fn deref(&self) -> &Self::Target {
        &self.policy
    }
}

impl<'de> Deserialize<'de> for UarDelegationPolicy {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        let presentation_ceiling_present = value.get("presentations").is_some();
        let policy = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
        Ok(Self {
            policy,
            presentation_ceiling_present,
        })
    }
}

impl Serialize for UarDelegationPolicy {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Preserve the historical RunPolicy field order. Hashing a JSON Value
        // instead would sort keys and change existing contract acknowledgements.
        let policy = &self.policy;
        let mut state = serializer.serialize_struct(
            "RunPolicy",
            if self.presentation_ceiling_present {
                13
            } else {
                12
            },
        )?;
        state.serialize_field("version", &policy.version)?;
        state.serialize_field("chat_mode", &policy.chat_mode)?;
        state.serialize_field("agent_id", &policy.agent_id)?;
        state.serialize_field("model", &policy.model)?;
        state.serialize_field("skills", &policy.skills)?;
        state.serialize_field("tools", &policy.tools)?;
        state.serialize_field("mcp_servers", &policy.mcp_servers)?;
        state.serialize_field("knowledge_bases", &policy.knowledge_bases)?;
        if self.presentation_ceiling_present {
            state.serialize_field("presentations", &policy.presentations)?;
        }
        state.serialize_field("memory_enabled", &policy.memory_enabled)?;
        state.serialize_field("prompt_caching_enabled", &policy.prompt_caching_enabled)?;
        state.serialize_field("context_strategy", &policy.context_strategy)?;
        state.serialize_field("tool_approval", &policy.tool_approval)?;
        state.end()
    }
}

fn deserialize_negotiation<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<crate::uar::a2ui::presentation_selection::PresentationNegotiation>, D::Error> {
    // Only wire omission qualifies as legacy. Explicit null is not an older
    // contract and must not erase the requirement for a concrete ceiling.
    crate::uar::a2ui::presentation_selection::PresentationNegotiation::deserialize(deserializer)
        .map(Some)
}

/// Secret-free authority ceiling supplied by an authenticated source UAR.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UarDelegationContract {
    pub version: u32,
    pub source_instance_id: String,
    pub target_instance_id: String,
    pub owner_id: String,
    pub root_run_id: String,
    pub parent_thread_id: String,
    pub child_thread_id: String,
    pub target_agent_id: String,
    pub policy: UarDelegationPolicy,
    /// Target-local per-turn and rolling-window limits.
    pub budgets: ThreadBudgets,
    /// Source-reserved cumulative capacity for this remote child lifetime.
    pub usage_grant: UarUsageGrant,
    pub sandbox: SandboxPermissions,
    /// Source-captured output restriction, never a model-controlled spawn field.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_negotiation"
    )]
    pub presentation_negotiation:
        Option<crate::uar::a2ui::presentation_selection::PresentationNegotiation>,
}

impl UarDelegationContract {
    pub fn validate(&self) -> anyhow::Result<()> {
        for (name, value) in [
            ("source instance", self.source_instance_id.as_str()),
            ("target instance", self.target_instance_id.as_str()),
            ("owner", self.owner_id.as_str()),
            ("root run", self.root_run_id.as_str()),
            ("parent thread", self.parent_thread_id.as_str()),
            ("child thread", self.child_thread_id.as_str()),
            ("target agent", self.target_agent_id.as_str()),
        ] {
            anyhow::ensure!(
                !value.trim().is_empty()
                    && value == value.trim()
                    && !value.chars().any(char::is_control),
                "UAR delegation {name} identity is invalid"
            );
        }
        anyhow::ensure!(
            self.version == UAR_DELEGATION_CONTRACT_VERSION
                && self.policy.version == RUN_POLICY_VERSION
                && self.policy.chat_mode == Some(ChatMode::Agent)
                && self.policy.agent_id.as_deref() == Some(self.target_agent_id.as_str()),
            "UAR delegation contract version or target policy is incompatible"
        );
        anyhow::ensure!(
            self.policy.model.is_some()
                && self.policy.memory_enabled.is_some()
                && self.policy.prompt_caching_enabled.is_some()
                && self.policy.context_strategy.is_some()
                && self.policy.tool_approval != ToolApprovalPolicy::Inherit,
            "UAR delegation policy is not a concrete authority ceiling"
        );
        for selection in [
            &self.policy.skills,
            &self.policy.tools,
            &self.policy.mcp_servers,
            &self.policy.knowledge_bases,
        ] {
            anyhow::ensure!(
                matches!(
                    selection.mode,
                    SelectionMode::Selected | SelectionMode::None
                ) && selection.denied_ids.is_empty()
                    && (selection.mode == SelectionMode::Selected) == !selection.ids.is_empty(),
                "UAR delegation resource policy is not concrete"
            );
        }
        if self.policy.presentation_ceiling_present {
            let selection = &self.policy.presentations;
            anyhow::ensure!(
                matches!(
                    selection.mode,
                    SelectionMode::Selected | SelectionMode::None
                ) && selection.denied_ids.is_empty()
                    && (selection.mode == SelectionMode::Selected) == !selection.ids.is_empty(),
                "UAR delegation Presentation ceiling is not concrete"
            );
        } else {
            anyhow::ensure!(
                self.presentation_negotiation.is_none(),
                "Negotiated UAR delegation requires a concrete Presentation ceiling"
            );
        }
        self.budgets.validate()?;
        self.usage_grant.validate()?;
        self.sandbox.validate()?;
        Ok(())
    }

    /// Apply a stricter target-local restriction for an older contract without
    /// changing the received policy used for equality and acknowledgement.
    pub(crate) fn execution_policy(&self) -> RunPolicy {
        let mut policy = self.policy.policy.clone();
        if !self.policy.presentation_ceiling_present {
            policy.presentations = crate::uar::domain::policy::ResourceSelection {
                mode: SelectionMode::None,
                ids: Vec::new(),
                denied_ids: Vec::new(),
            };
        }
        policy
    }

    pub fn digest(&self) -> anyhow::Result<String> {
        self.validate()?;
        let encoded = serde_json::to_vec(self)?;
        let mut digest = Sha256::new();
        digest.update(encoded);
        Ok(digest
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect())
    }
}

/// Cumulative source-owned capacity leased to one governed remote child.
/// These fields deliberately do not reuse per-turn or per-minute names.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct UarUsageGrant {
    pub max_total_tokens: Option<u64>,
    pub max_total_cost_usd: Option<f64>,
    pub max_total_model_requests: Option<u64>,
    pub max_total_tool_calls: Option<u64>,
    pub expires_after_seconds: Option<u64>,
}

impl UarUsageGrant {
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.max_total_cost_usd
                .is_none_or(|cost| cost.is_finite() && cost >= 0.0),
            "A2A cumulative cost grant is invalid"
        );
        anyhow::ensure!(
            self.expires_after_seconds.is_none_or(|seconds| seconds > 0),
            "A2A cumulative usage grant expiry is invalid"
        );
        Ok(())
    }
}

/// Contractual acknowledgement returned on every task receipt. This confirms
/// what the peer agreed to enforce; it is not remote attestation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UarDelegationAcknowledgement {
    pub version: u32,
    pub target_instance_id: String,
    pub child_thread_id: String,
    pub contract_digest: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_thread_id: Option<String>,
}

impl UarDelegationAcknowledgement {
    pub fn for_contract(
        contract: &UarDelegationContract,
        remote_thread_id: Option<String>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            version: UAR_DELEGATION_CONTRACT_VERSION,
            target_instance_id: contract.target_instance_id.clone(),
            child_thread_id: contract.child_thread_id.clone(),
            contract_digest: contract.digest()?,
            remote_thread_id,
        })
    }

    pub fn validate_for(&self, contract: &UarDelegationContract) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.version == UAR_DELEGATION_CONTRACT_VERSION
                && self.target_instance_id == contract.target_instance_id
                && self.child_thread_id == contract.child_thread_id
                && self.contract_digest == contract.digest()?,
            "A2A peer did not acknowledge the exact UAR delegation contract"
        );
        Ok(())
    }
}

/// Cumulative usage for one remote run. The source charges this receipt once
/// against its root ledger after terminal settlement.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UarUsageReceipt {
    pub total_tokens: u64,
    pub cost_usd: f64,
    pub model_requests: u64,
    pub tool_calls: u64,
}

impl UarUsageReceipt {
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.cost_usd.is_finite() && self.cost_usd >= 0.0,
            "A2A usage cost is invalid"
        );
        Ok(())
    }
}
