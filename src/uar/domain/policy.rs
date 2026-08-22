//! Scoped chat policy contracts and deterministic resolution.
//!
//! UAR owns policy resolution. Hosts submit intent at global, agent,
//! conversation, or turn scope; the runtime resolves one immutable policy before
//! it assembles context or exposes tools to a model.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::uar::context::ContextStrategy;
use crate::uar::domain::artifact::AgentArtifact;
use crate::uar::persistence::PersistenceLayer;
use crate::uar::settings::manager::SettingsManager;

/// Current serialized policy schema version.
pub const RUN_POLICY_VERSION: u32 = 1;

fn default_policy_owner() -> String {
    crate::session::ANONYMOUS_SESSION_OWNER.to_string()
}

/// Durable conversation-scoped policy record owned by UAR persistence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ConversationPolicyRecord {
    /// Authenticated user that owns the conversation.
    #[serde(default = "default_policy_owner")]
    pub owner_id: String,
    /// Stable conversation/session identifier.
    pub conversation_id: String,
    /// Versioned requested policy.
    pub policy: RunPolicy,
    /// Last update timestamp.
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl ConversationPolicyRecord {
    /// Create a record using the current UTC timestamp.
    #[must_use]
    pub fn new(conversation_id: impl Into<String>, policy: RunPolicy) -> Self {
        Self::new_for_user(
            crate::session::ANONYMOUS_SESSION_OWNER,
            conversation_id,
            policy,
        )
    }

    /// Create an owner-scoped record using the current UTC timestamp.
    #[must_use]
    pub fn new_for_user(
        owner_id: impl Into<String>,
        conversation_id: impl Into<String>,
        policy: RunPolicy,
    ) -> Self {
        Self {
            owner_id: owner_id.into(),
            conversation_id: conversation_id.into(),
            policy,
            updated_at: chrono::Utc::now(),
        }
    }
}

/// User-visible execution mode for a chat run.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChatMode {
    /// Use a registered host-owned local model through UAR governance.
    Local,
    /// Use UAR's protected orchestrator with its configured provider route.
    #[default]
    Uar,
    /// Execute a selected persisted UAR agent artifact.
    Agent,
}

/// How a scope selects resources from the set allowed by its parent scope.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SelectionMode {
    /// Preserve the parent scope's selection.
    #[default]
    Inherit,
    /// Let UAR choose a subset from the eligible resources at run time.
    Auto,
    /// Make every eligible resource available.
    All,
    /// Make no resource of this kind available.
    None,
    /// Make only the named eligible resources available.
    Selected,
}

/// Resource-selection intent at one policy scope.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ResourceSelection {
    /// Selection behavior.
    #[serde(default)]
    pub mode: SelectionMode,
    /// Stable resource identifiers used when `mode` is `selected`.
    #[serde(default)]
    pub ids: Vec<String>,
    /// Resource identifiers explicitly denied at this scope.
    #[serde(default)]
    pub denied_ids: Vec<String>,
}

impl ResourceSelection {
    /// Construct a selected-resource policy with deterministic identifiers.
    #[must_use]
    pub fn selected(ids: impl IntoIterator<Item = String>) -> Self {
        let ids = ids
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        Self {
            mode: SelectionMode::Selected,
            ids,
            denied_ids: Vec::new(),
        }
    }
}

/// Explicit provider/model route requested by a policy scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ModelRoute {
    /// Provider registry identifier.
    pub provider_id: String,
    /// Provider-local model identifier.
    pub model_id: String,
}

/// Tool-call approval constraint.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ToolApprovalPolicy {
    /// Preserve the parent policy.
    #[default]
    Inherit,
    /// Let UAR governance decide whether approval is necessary.
    Auto,
    /// Require user approval for tool calls.
    Ask,
    /// Deny tool calls.
    Deny,
}

/// Versioned policy supplied at global, agent, conversation, or turn scope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RunPolicy {
    /// Serialized schema version.
    #[serde(default = "default_policy_version")]
    pub version: u32,
    /// Optional execution-mode override.
    #[serde(default)]
    pub chat_mode: Option<ChatMode>,
    /// Optional selected agent identifier.
    #[serde(default)]
    pub agent_id: Option<String>,
    /// Optional explicit provider/model route.
    #[serde(default)]
    pub model: Option<ModelRoute>,
    /// Eligible or selected skills.
    #[serde(default)]
    pub skills: ResourceSelection,
    /// Eligible or selected tool identifiers.
    #[serde(default)]
    pub tools: ResourceSelection,
    /// Eligible or selected MCP servers.
    #[serde(default)]
    pub mcp_servers: ResourceSelection,
    /// Eligible or selected knowledge bases.
    #[serde(default)]
    pub knowledge_bases: ResourceSelection,
    /// Optional memory enablement override.
    #[serde(default)]
    pub memory_enabled: Option<bool>,
    /// Optional message/context management strategy.
    #[serde(default)]
    pub context_strategy: Option<ContextStrategy>,
    /// Tool approval constraint.
    #[serde(default)]
    pub tool_approval: ToolApprovalPolicy,
}

const fn default_policy_version() -> u32 {
    RUN_POLICY_VERSION
}

impl Default for RunPolicy {
    fn default() -> Self {
        Self {
            version: RUN_POLICY_VERSION,
            chat_mode: None,
            agent_id: None,
            model: None,
            skills: ResourceSelection::default(),
            tools: ResourceSelection::default(),
            mcp_servers: ResourceSelection::default(),
            knowledge_bases: ResourceSelection::default(),
            memory_enabled: None,
            context_strategy: None,
            tool_approval: ToolApprovalPolicy::Inherit,
        }
    }
}

/// Convert a persisted agent artifact into its policy scope.
#[must_use]
pub fn policy_from_agent_artifact(agent: &AgentArtifact) -> RunPolicy {
    let chat_mode = if matches!(agent.id.as_str(), "default-agent" | "orchestrator-agent") {
        ChatMode::Uar
    } else {
        ChatMode::Agent
    };
    let skills = if agent.policy.skills.prefer.is_empty() {
        ResourceSelection {
            mode: SelectionMode::Auto,
            ..ResourceSelection::default()
        }
    } else {
        ResourceSelection::selected(agent.policy.skills.prefer.clone())
    };
    let tools = if agent.policy.tools.allow.is_empty() {
        ResourceSelection {
            mode: SelectionMode::Auto,
            denied_ids: agent.policy.tools.deny.clone(),
            ..ResourceSelection::default()
        }
    } else if agent.policy.tools.allow.iter().any(|tool| tool == "*") {
        // Agent artifacts historically use `*` as a wildcard.  The scoped
        // policy contract represents that intent explicitly as `all`; treating
        // it as a literal selected tool empties the eligible set and prevents
        // every native/MCP tool from reaching the model.
        ResourceSelection {
            mode: SelectionMode::All,
            denied_ids: agent.policy.tools.deny.clone(),
            ..ResourceSelection::default()
        }
    } else {
        ResourceSelection {
            denied_ids: agent.policy.tools.deny.clone(),
            ..ResourceSelection::selected(agent.policy.tools.allow.clone())
        }
    };
    let knowledge_bases = if agent.memory.kb.enabled {
        if agent.memory.kb.knowledge_bases.is_empty() {
            ResourceSelection {
                mode: SelectionMode::Auto,
                ..ResourceSelection::default()
            }
        } else {
            ResourceSelection::selected(agent.memory.kb.knowledge_bases.clone())
        }
    } else {
        ResourceSelection {
            mode: SelectionMode::None,
            ..ResourceSelection::default()
        }
    };

    let default_provider = &agent.policy.provider.default;
    let model = (!default_provider.provider.trim().is_empty()
        && !default_provider.model.trim().is_empty())
    .then(|| ModelRoute {
        provider_id: default_provider.provider.clone(),
        model_id: default_provider.model.clone(),
    });
    let mut policy = RunPolicy {
        chat_mode: Some(chat_mode),
        agent_id: Some(agent.id.clone()),
        model,
        skills,
        tools,
        knowledge_bases,
        memory_enabled: Some(agent.memory.conversation.enabled),
        ..RunPolicy::default()
    };

    // Agent artifacts predate the shared scoped RunPolicy contract. Preserve
    // their stable schema while allowing KnowMe and other UAR clients to store
    // the complete agent-level policy (MCP servers, context strategy, approval,
    // and explicit deny lists) in the standard extension map.
    if let Some(extension) = agent.extensions.get("uar.run_policy")
        && let Ok(extended) = serde_json::from_value::<RunPolicy>(extension.clone())
    {
        if extended.chat_mode.is_some() {
            policy.chat_mode = extended.chat_mode;
        }
        if extended.agent_id.is_some() {
            policy.agent_id = extended.agent_id;
        }
        if extended.model.is_some() {
            policy.model = extended.model;
        }
        merge_resource_selection(&mut policy.skills, extended.skills);
        merge_resource_selection(&mut policy.tools, extended.tools);
        merge_resource_selection(&mut policy.mcp_servers, extended.mcp_servers);
        merge_resource_selection(&mut policy.knowledge_bases, extended.knowledge_bases);
        if extended.memory_enabled.is_some() {
            policy.memory_enabled = extended.memory_enabled;
        }
        if extended.context_strategy.is_some() {
            policy.context_strategy = extended.context_strategy;
        }
        if extended.tool_approval != ToolApprovalPolicy::Inherit {
            policy.tool_approval = extended.tool_approval;
        }
    }

    policy
}

fn merge_resource_selection(target: &mut ResourceSelection, value: ResourceSelection) {
    if value.mode != SelectionMode::Inherit || !value.ids.is_empty() || !value.denied_ids.is_empty()
    {
        *target = value;
    }
}

/// Scope that supplied an effective policy value.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum PolicyScope {
    /// Runtime-wide policy.
    Global,
    /// Selected agent policy.
    Agent,
    /// Persisted conversation policy.
    Conversation,
    /// One-run override.
    Turn,
    /// Compatibility behavior derived from existing runtime configuration.
    Legacy,
}

/// Resources enabled by configuration and available on the current runtime.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PolicyUniverse {
    /// Enabled skill identifiers.
    pub skills: BTreeSet<String>,
    /// Enabled tool identifiers.
    pub tools: BTreeSet<String>,
    /// Enabled and platform-compatible MCP server identifiers.
    pub mcp_servers: BTreeSet<String>,
    /// Enabled knowledge-base identifiers.
    pub knowledge_bases: BTreeSet<String>,
}

/// Resolved selection for one resource kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EffectiveResourceSelection {
    /// Runtime behavior after resolution.
    pub mode: SelectionMode,
    /// Eligible resource identifiers in deterministic order.
    pub ids: Vec<String>,
    /// Last non-inherited scope that affected the selection.
    pub source: PolicyScope,
}

/// Secret-free, immutable policy used to execute one run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EffectiveRunPolicy {
    /// Serialized schema version.
    pub version: u32,
    /// Effective chat mode.
    pub chat_mode: ChatMode,
    /// Effective agent identifier, when applicable.
    pub agent_id: Option<String>,
    /// Effective provider/model route, when explicitly selected.
    pub model: Option<ModelRoute>,
    /// Effective skill eligibility.
    pub skills: EffectiveResourceSelection,
    /// Effective tool eligibility.
    pub tools: EffectiveResourceSelection,
    /// Effective MCP server eligibility.
    pub mcp_servers: EffectiveResourceSelection,
    /// Effective knowledge-base eligibility.
    pub knowledge_bases: EffectiveResourceSelection,
    /// Whether governed memory participates in the run.
    pub memory_enabled: bool,
    /// Effective context strategy.
    pub context_strategy: ContextStrategy,
    /// Effective tool approval constraint.
    pub tool_approval: ToolApprovalPolicy,
    /// Source scope for scalar policy fields.
    pub provenance: BTreeMap<String, PolicyScope>,
    /// Missing, disabled, or conflicting selections that were safely excluded.
    pub warnings: Vec<String>,
}

/// Input required to resolve one immutable effective policy.
#[derive(Debug, Clone, Default)]
pub struct PolicyResolutionInput {
    /// Runtime-wide policy.
    pub global: Option<RunPolicy>,
    /// Selected agent policy.
    pub agent: Option<RunPolicy>,
    /// Persisted conversation policy.
    pub conversation: Option<RunPolicy>,
    /// One-run policy override.
    pub turn: Option<RunPolicy>,
    /// Enabled and runtime-compatible resources.
    pub universe: PolicyUniverse,
    /// Compatibility mode used when no scope selects one.
    pub default_chat_mode: ChatMode,
    /// Compatibility context strategy.
    pub default_context_strategy: ContextStrategy,
    /// Compatibility agent identifier.
    pub default_agent_id: Option<String>,
    /// Compatibility provider/model route.
    pub default_model: Option<ModelRoute>,
}

/// Resolve one run policy using deny-safe, non-widening scope precedence.
#[must_use]
pub fn resolve_run_policy(input: PolicyResolutionInput) -> EffectiveRunPolicy {
    let scopes = [
        (PolicyScope::Global, input.global.as_ref()),
        (PolicyScope::Agent, input.agent.as_ref()),
        (PolicyScope::Conversation, input.conversation.as_ref()),
        (PolicyScope::Turn, input.turn.as_ref()),
    ];

    let mut provenance = BTreeMap::new();
    let mut chat_mode = input.default_chat_mode;
    let mut agent_id = input.default_agent_id;
    let mut model = input.default_model;
    let mut context_strategy = input.default_context_strategy;
    let mut memory_enabled = true;
    let mut memory_locked_off = false;
    let mut tool_approval = ToolApprovalPolicy::Auto;

    for (scope, policy) in scopes {
        let Some(policy) = policy else { continue };
        if let Some(value) = policy.chat_mode {
            chat_mode = value;
            provenance.insert("chat_mode".into(), scope);
        }
        if let Some(value) = &policy.agent_id {
            agent_id = Some(value.clone());
            provenance.insert("agent_id".into(), scope);
        }
        if let Some(value) = &policy.model {
            model = Some(value.clone());
            provenance.insert("model".into(), scope);
        }
        if let Some(value) = &policy.context_strategy {
            context_strategy = value.clone();
            provenance.insert("context_strategy".into(), scope);
        }
        if let Some(value) = policy.memory_enabled {
            if !value {
                memory_enabled = false;
                memory_locked_off = true;
                provenance.insert("memory_enabled".into(), scope);
            } else if !memory_locked_off {
                memory_enabled = true;
                provenance.insert("memory_enabled".into(), scope);
            }
        }
        if approval_rank(policy.tool_approval) > approval_rank(tool_approval) {
            tool_approval = policy.tool_approval;
            provenance.insert("tool_approval".into(), scope);
        }
    }

    provenance
        .entry("chat_mode".into())
        .or_insert(PolicyScope::Legacy);
    provenance
        .entry("agent_id".into())
        .or_insert(PolicyScope::Legacy);
    provenance
        .entry("model".into())
        .or_insert(PolicyScope::Legacy);
    provenance
        .entry("context_strategy".into())
        .or_insert(PolicyScope::Legacy);
    provenance
        .entry("memory_enabled".into())
        .or_insert(PolicyScope::Legacy);
    provenance
        .entry("tool_approval".into())
        .or_insert(PolicyScope::Legacy);

    let mut warnings = Vec::new();
    let skills = resolve_resources(
        &input.universe.skills,
        &scopes,
        |policy| &policy.skills,
        "skill",
        &mut warnings,
    );
    let tools = resolve_resources(
        &input.universe.tools,
        &scopes,
        |policy| &policy.tools,
        "tool",
        &mut warnings,
    );
    let mcp_servers = resolve_resources(
        &input.universe.mcp_servers,
        &scopes,
        |policy| &policy.mcp_servers,
        "MCP server",
        &mut warnings,
    );
    let knowledge_bases = resolve_resources(
        &input.universe.knowledge_bases,
        &scopes,
        |policy| &policy.knowledge_bases,
        "knowledge base",
        &mut warnings,
    );

    EffectiveRunPolicy {
        version: RUN_POLICY_VERSION,
        chat_mode,
        agent_id,
        model,
        skills,
        tools,
        mcp_servers,
        knowledge_bases,
        memory_enabled,
        context_strategy,
        tool_approval,
        provenance,
        warnings,
    }
}

/// Transport-free inputs shared by the service and embedded resolvers.
///
/// Both the Axum `AppState` path and the in-process `RunManager` path build this
/// context from the components they already own (a settings manager, a resolved
/// [`PolicyUniverse`], and a default context strategy) so that policy resolution
/// runs through one shared core with no dependency on a web framework.
#[derive(Debug)]
pub struct PolicyResolutionContext<'a> {
    /// Runtime-wide settings source for the Global scope (`run_policy.global`).
    pub settings_manager: Option<&'a SettingsManager>,
    /// Resources enabled and available on the current runtime.
    pub universe: PolicyUniverse,
    /// Compatibility context strategy used when no scope selects one.
    pub default_context_strategy: ContextStrategy,
}

/// Resolve the effective policy for a conversation and turn using the full
/// Global → Agent → Conversation → Turn precedence.
///
/// This is the transport-free core shared by the HTTP service path and the
/// embedded runtime. It reads the Global scope from `run_policy.global` via the
/// supplied [`SettingsManager`], derives the Agent scope from `agent`, applies the
/// caller-resolved `conversation` scope, and applies `turn` last. When no settings
/// manager is available the Global scope is simply absent, which is the
/// backward-compatible agent+conversation behavior.
///
/// The conversation scope is passed in (not loaded here) because callers differ in
/// where a conversation policy lives: the service path also consults an in-memory
/// session map, while the embedded path uses [`load_persisted_conversation_policy`].
/// The resolution itself delegates to [`resolve_run_policy`], so identical inputs
/// always yield an identical [`EffectiveRunPolicy`] regardless of caller.
pub async fn resolve_effective_run_policy_core(
    ctx: PolicyResolutionContext<'_>,
    agent: &AgentArtifact,
    conversation: Option<RunPolicy>,
    turn: Option<RunPolicy>,
) -> EffectiveRunPolicy {
    let agent_default = &agent.policy.provider.default;
    let default_model = (!agent_default.provider.trim().is_empty()
        && !agent_default.model.trim().is_empty())
    .then(|| ModelRoute {
        provider_id: agent_default.provider.clone(),
        model_id: agent_default.model.clone(),
    });
    let global = match ctx.settings_manager {
        Some(manager) => manager
            .get_typed::<RunPolicy>("run_policy.global")
            .await
            .map_err(|error| {
                tracing::warn!(%error, "failed to load global run policy; using inherited defaults");
                error
            })
            .ok()
            .flatten(),
        None => None,
    };
    let agent_policy = policy_from_agent_artifact(agent);
    resolve_run_policy(PolicyResolutionInput {
        global,
        agent: Some(agent_policy),
        conversation,
        turn,
        universe: ctx.universe,
        default_chat_mode: ChatMode::Agent,
        default_context_strategy: ctx.default_context_strategy,
        default_agent_id: Some(agent.id.clone()),
        default_model,
        ..PolicyResolutionInput::default()
    })
}

/// Load the durable conversation-scoped policy from persistence, if present.
///
/// Returns `None` when persistence is absent, the record is missing, or the load
/// fails (a load error is logged and treated as "inherit"). Callers that also
/// maintain an in-memory conversation policy (e.g. the service path) resolve that
/// fallback themselves and pass the result as the conversation scope.
pub async fn load_persisted_conversation_policy(
    persistence: Option<&Arc<dyn PersistenceLayer>>,
    owner_id: &str,
    conversation_id: &str,
) -> Option<RunPolicy> {
    let persistence = persistence?;
    match persistence
        .load_conversation_policy(owner_id, conversation_id)
        .await
    {
        Ok(Some(record)) => Some(record.policy),
        Ok(None) => None,
        Err(error) => {
            tracing::warn!(%error, %conversation_id, "failed to load conversation policy");
            None
        }
    }
}

fn approval_rank(value: ToolApprovalPolicy) -> u8 {
    match value {
        ToolApprovalPolicy::Inherit | ToolApprovalPolicy::Auto => 0,
        ToolApprovalPolicy::Ask => 1,
        ToolApprovalPolicy::Deny => 2,
    }
}

fn resolve_resources(
    universe: &BTreeSet<String>,
    scopes: &[(PolicyScope, Option<&RunPolicy>); 4],
    field: impl Fn(&RunPolicy) -> &ResourceSelection,
    label: &str,
    warnings: &mut Vec<String>,
) -> EffectiveResourceSelection {
    let mut eligible = universe.clone();
    let mut mode = SelectionMode::Auto;
    let mut source = PolicyScope::Legacy;

    for (scope, policy) in scopes {
        let Some(policy) = policy else { continue };
        let selection = field(policy);
        let denied = selection
            .denied_ids
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        eligible = eligible
            .difference(&denied)
            .cloned()
            .collect::<BTreeSet<_>>();
        match selection.mode {
            SelectionMode::Inherit => {}
            SelectionMode::None => {
                eligible.clear();
                mode = SelectionMode::None;
                source = *scope;
            }
            SelectionMode::All | SelectionMode::Auto => {
                // Lower scopes cannot widen a set narrowed or denied above them.
                mode = selection.mode;
                source = *scope;
            }
            SelectionMode::Selected => {
                let requested = selection.ids.iter().cloned().collect::<BTreeSet<_>>();
                for missing in requested.difference(universe) {
                    warnings.push(format!("{label} '{missing}' is unavailable"));
                }
                eligible = eligible
                    .intersection(&requested)
                    .cloned()
                    .collect::<BTreeSet<_>>();
                mode = SelectionMode::Selected;
                source = *scope;
            }
        }
    }

    EffectiveResourceSelection {
        mode,
        ids: eligible.into_iter().collect(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn universe() -> PolicyUniverse {
        PolicyUniverse {
            skills: ["alpha", "beta", "gamma"].map(str::to_string).into(),
            tools: ["read", "search"].map(str::to_string).into(),
            mcp_servers: ["files", "search"].map(str::to_string).into(),
            knowledge_bases: ["personal", "work"].map(str::to_string).into(),
        }
    }

    fn input() -> PolicyResolutionInput {
        PolicyResolutionInput {
            universe: universe(),
            default_chat_mode: ChatMode::Uar,
            default_context_strategy: ContextStrategy::Auto,
            default_agent_id: Some("orchestrator-agent".into()),
            default_model: Some(ModelRoute {
                provider_id: "openai".into(),
                model_id: "gpt-test".into(),
            }),
            ..PolicyResolutionInput::default()
        }
    }

    #[test]
    fn conversation_selection_narrows_agent_resources() {
        let agent = RunPolicy {
            skills: ResourceSelection::selected(["alpha".into(), "beta".into()]),
            ..RunPolicy::default()
        };
        let conversation = RunPolicy {
            skills: ResourceSelection::selected(["beta".into(), "gamma".into()]),
            ..RunPolicy::default()
        };
        let effective = resolve_run_policy(PolicyResolutionInput {
            agent: Some(agent),
            conversation: Some(conversation),
            ..input()
        });

        assert_eq!(effective.skills.ids, vec!["beta"]);
        assert_eq!(effective.skills.source, PolicyScope::Conversation);
    }

    #[test]
    fn lower_scope_cannot_reenable_parent_none() {
        let global = RunPolicy {
            mcp_servers: ResourceSelection {
                mode: SelectionMode::None,
                ids: vec![],
                denied_ids: vec![],
            },
            ..RunPolicy::default()
        };
        let turn = RunPolicy {
            mcp_servers: ResourceSelection {
                mode: SelectionMode::All,
                ids: vec![],
                denied_ids: vec![],
            },
            ..RunPolicy::default()
        };
        let effective = resolve_run_policy(PolicyResolutionInput {
            global: Some(global),
            turn: Some(turn),
            ..input()
        });

        assert!(effective.mcp_servers.ids.is_empty());
    }

    #[test]
    fn unavailable_resources_warn_without_broadening() {
        let conversation = RunPolicy {
            knowledge_bases: ResourceSelection::selected(["missing".into()]),
            ..RunPolicy::default()
        };
        let effective = resolve_run_policy(PolicyResolutionInput {
            conversation: Some(conversation),
            ..input()
        });

        assert!(effective.knowledge_bases.ids.is_empty());
        assert_eq!(
            effective.warnings,
            vec!["knowledge base 'missing' is unavailable"]
        );
    }

    #[test]
    fn memory_and_approval_constraints_only_tighten() {
        let global = RunPolicy {
            memory_enabled: Some(false),
            tool_approval: ToolApprovalPolicy::Ask,
            ..RunPolicy::default()
        };
        let turn = RunPolicy {
            memory_enabled: Some(true),
            tool_approval: ToolApprovalPolicy::Auto,
            ..RunPolicy::default()
        };
        let effective = resolve_run_policy(PolicyResolutionInput {
            global: Some(global),
            turn: Some(turn),
            ..input()
        });

        assert!(!effective.memory_enabled);
        assert_eq!(effective.tool_approval, ToolApprovalPolicy::Ask);
    }

    #[test]
    fn legacy_defaults_are_backward_compatible() {
        let effective = resolve_run_policy(input());

        assert_eq!(effective.chat_mode, ChatMode::Uar);
        assert_eq!(effective.skills.ids, vec!["alpha", "beta", "gamma"]);
        assert_eq!(effective.context_strategy, ContextStrategy::Auto);
        assert!(effective.warnings.is_empty());
    }

    #[test]
    fn agent_artifact_tool_wildcard_selects_every_available_tool() {
        let policy = policy_from_agent_artifact(&crate::uar::defaults::default_agent());
        assert_eq!(policy.tools.mode, SelectionMode::All);
        assert!(policy.tools.ids.is_empty());

        let effective = resolve_run_policy(PolicyResolutionInput {
            agent: Some(policy),
            ..input()
        });

        assert_eq!(effective.tools.mode, SelectionMode::All);
        assert_eq!(effective.tools.ids, vec!["read", "search"]);
        assert!(effective.warnings.is_empty());
    }

    #[test]
    fn agent_without_explicit_provider_inherits_the_runtime_model() {
        let mut agent = crate::uar::defaults::default_agent();
        agent.policy.provider.default.provider.clear();
        agent.policy.provider.default.model.clear();

        let policy = policy_from_agent_artifact(&agent);

        assert_eq!(policy.model, None);
    }

    #[test]
    fn agent_extension_supplies_complete_scoped_policy() {
        let mut agent = crate::uar::defaults::default_agent();
        agent.extensions.insert(
            "uar.run_policy".into(),
            serde_json::json!({
                "version": 1,
                "skills": { "mode": "selected", "ids": ["alpha"], "denied_ids": [] },
                "tools": { "mode": "inherit", "ids": [], "denied_ids": ["search"] },
                "mcp_servers": { "mode": "selected", "ids": ["files"], "denied_ids": [] },
                "knowledge_bases": { "mode": "none", "ids": [], "denied_ids": [] },
                "memory_enabled": false,
                "context_strategy": { "type": "sliding_window", "max_messages": 7 },
                "tool_approval": "ask"
            }),
        );

        let policy = policy_from_agent_artifact(&agent);
        assert_eq!(policy.skills.mode, SelectionMode::Selected);
        assert_eq!(policy.skills.ids, vec!["alpha"]);
        assert_eq!(policy.tools.denied_ids, vec!["search"]);
        assert_eq!(policy.mcp_servers.ids, vec!["files"]);
        assert_eq!(policy.knowledge_bases.mode, SelectionMode::None);
        assert_eq!(policy.memory_enabled, Some(false));
        assert_eq!(policy.tool_approval, ToolApprovalPolicy::Ask);
        assert!(matches!(
            policy.context_strategy,
            Some(ContextStrategy::SlidingWindow { max_messages: 7 })
        ));
    }
}
