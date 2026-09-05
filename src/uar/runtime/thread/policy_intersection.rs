//! Host-owned delegation policy. A child receives a frozen subset of its
//! parent's authority, never a fresh resolution against the global universe.
//!
//! `uar.thread_policy` is the versioned, restriction-only artifact extension
//! for credential bindings and sandbox grants. Missing fields inherit; empty
//! collections deny. Binding IDs refer to immutable host-resolved resources,
//! not environment-variable names, filesystem paths, or credential lookups.
//! The executor must use these exact bindings and reject an unenforceable
//! sandbox policy before starting. This module performs no I/O or approvals.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::uar::domain::artifact::{AgentArtifact, ToolExecutionMode};
use crate::uar::domain::policy::{
    ChatMode, EffectiveResourceSelection, EffectiveRunPolicy, PolicyScope, RUN_POLICY_VERSION,
    ResourceSelection, RunPolicy, SelectionMode, ToolApprovalPolicy, policy_from_agent_artifact,
};

use super::AgentThread;

/// Artifact restrictions only; this extension cannot carry user authorization.
pub const THREAD_POLICY_EXTENSION: &str = "uar.thread_policy";

/// The service identity associated with a host-resolved credential binding.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "id",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum CredentialTarget {
    Provider(String),
    McpServer(String),
}

/// A host-resolved credential (or explicitly credential-free service binding).
/// The same target with a different binding is a different grant. Executors
/// must not replace a binding with a child-agent/user/system/env fallback.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CredentialGrant {
    pub target: CredentialTarget,
    pub binding_id: String,
}

/// Trusted mapping from a runtime tool ID to its bound MCP server and upstream
/// name. Never infer server identity by splitting an untrusted tool-name string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpToolBinding {
    pub server_id: String,
    pub tool_name: String,
}

/// Tool identity resolved by the host registry. Missing bindings are errors,
/// not an implicit classification as a native tool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThreadToolBinding {
    Native,
    Mcp(McpToolBinding),
}

/// Access to one immutable host filesystem binding, resolved before delegation.
/// A binding includes the canonical host location and guest mount location;
/// a child cannot redefine either by supplying a path under the same ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FilesystemGrant {
    pub read: bool,
    pub write: bool,
}

/// Concrete sandbox authority. No wildcard, path-prefix, or deny-rule grammar
/// is accepted. Environment entries identify approved host bindings, not values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SandboxPermissions {
    pub execution_mode: ToolExecutionMode,
    pub network_enabled: bool,
    pub filesystem: BTreeMap<String, FilesystemGrant>,
    pub environment: BTreeSet<String>,
}

impl SandboxPermissions {
    pub(crate) fn intersect(&self, requested: &Self) -> Self {
        let filesystem = self
            .filesystem
            .iter()
            .filter_map(|(id, parent)| {
                let child = requested.filesystem.get(id)?;
                let grant = FilesystemGrant {
                    read: parent.read && child.read,
                    write: parent.write && child.write,
                };
                (grant.read || grant.write).then(|| (id.clone(), grant))
            })
            .collect();
        Self {
            execution_mode: intersect_execution_mode(
                &self.execution_mode,
                &requested.execution_mode,
            ),
            network_enabled: self.network_enabled && requested.network_enabled,
            filesystem,
            environment: self
                .environment
                .intersection(&requested.environment)
                .cloned()
                .collect(),
        }
    }

    pub(crate) fn validate(&self) -> Result<(), PolicyIntersectionError> {
        for id in self.filesystem.keys().chain(self.environment.iter()) {
            validate_id(id, "sandbox binding")?;
        }
        Ok(())
    }
}

/// The existing `extensions["budgets"]` shape, decoded strictly at delegation.
/// `None` is unbounded/inherited; zero is a real limit, never a default request.
/// These are ceilings, not fresh balances: usage remains charged to the root.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ThreadBudgets {
    pub max_tokens_per_turn: Option<u64>,
    pub max_tokens_per_session: Option<u64>,
    pub max_tool_calls_per_turn: Option<u32>,
    pub max_cost_per_session_usd: Option<f64>,
    pub timeout_seconds: Option<u64>,
    pub rate_limit: Option<ThreadRateLimit>,
}

/// Root-shared rate ceilings; children do not receive independent rate buckets.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ThreadRateLimit {
    pub requests_per_minute: Option<u32>,
    pub tokens_per_minute: Option<u64>,
}

impl ThreadBudgets {
    pub(crate) fn from_artifact(artifact: &AgentArtifact) -> Result<Self, PolicyIntersectionError> {
        let budgets = match artifact.extensions.get("budgets") {
            // The descriptor compiler writes null for an absent section.
            None | Some(Value::Null) => Self::default(),
            Some(value) => strict_decode::<Self>(value, "budgets")?,
        };
        budgets.validate()?;
        Ok(budgets)
    }

    pub(crate) fn validate(&self) -> Result<(), PolicyIntersectionError> {
        if self
            .max_cost_per_session_usd
            .is_some_and(|cost| !cost.is_finite() || cost < 0.0)
        {
            return Err(PolicyIntersectionError::UnsupportedShape { section: "budgets" });
        }
        Ok(())
    }

    pub(crate) fn intersect(&self, requested: &Self) -> Self {
        let rate_limit = match (&self.rate_limit, &requested.rate_limit) {
            (Some(parent), Some(child)) => Some(ThreadRateLimit {
                requests_per_minute: lower(parent.requests_per_minute, child.requests_per_minute),
                tokens_per_minute: lower(parent.tokens_per_minute, child.tokens_per_minute),
            }),
            (parent, child) => parent.clone().or_else(|| child.clone()),
        };
        Self {
            max_tokens_per_turn: lower(self.max_tokens_per_turn, requested.max_tokens_per_turn),
            max_tokens_per_session: lower(
                self.max_tokens_per_session,
                requested.max_tokens_per_session,
            ),
            max_tool_calls_per_turn: lower(
                self.max_tool_calls_per_turn,
                requested.max_tool_calls_per_turn,
            ),
            max_cost_per_session_usd: lower(
                self.max_cost_per_session_usd,
                requested.max_cost_per_session_usd,
            ),
            timeout_seconds: lower(self.timeout_seconds, requested.timeout_seconds),
            rate_limit,
        }
    }
}

/// Additional root authority supplied by the trusted host, never spawn input.
/// There is deliberately no deserializer or permissive default for this type.
#[derive(Debug, Clone, PartialEq)]
pub struct ThreadPermissions {
    pub credentials: BTreeSet<CredentialGrant>,
    pub tool_bindings: BTreeMap<String, ThreadToolBinding>,
    pub sandbox: SandboxPermissions,
    pub budgets: ThreadBudgets,
    pub max_active_skills: u32,
    pub max_concurrent_tools: u32,
}

/// Restriction-only artifact input. A version is mandatory when present.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactThreadRestrictions {
    version: u32,
    #[serde(default)]
    credentials: Option<BTreeSet<CredentialGrant>>,
    #[serde(default)]
    sandbox: Option<SandboxPermissions>,
}

/// A host-held policy snapshot. Not deserializable, and no API can replace its
/// root approval channel, grant approval, or insert child-supplied authority.
#[derive(Clone)]
pub struct ThreadPolicy {
    owner_id: String,
    approval_root_run_id: String,
    effective: EffectiveRunPolicy,
    permissions: ThreadPermissions,
    artifact: AgentArtifact,
}

impl std::fmt::Debug for ThreadPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThreadPolicy")
            .field("root_run_id", &self.approval_root_run_id)
            .field("artifact_id", &self.artifact.id)
            .field("tool_count", &self.effective.tools.ids.len())
            .finish_non_exhaustive()
    }
}

/// Fail-closed errors contain section names, never policy bodies or secrets.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PolicyIntersectionError {
    #[error("unsupported or malformed thread policy section: {section}")]
    UnsupportedShape { section: &'static str },
    #[error("thread policy identity does not match its host-resolved artifact or root")]
    IdentityMismatch,
    #[error("child model route has no inherited host credential binding")]
    CredentialNotGranted,
}

impl ThreadPolicy {
    /// Capture root authority from a verified host context and resolved policy.
    ///
    /// # Errors
    /// Rejects non-root records, mismatched artifacts, unresolved policy shapes,
    /// malformed restrictions, and model routes without a host binding.
    pub fn for_root(
        root: &AgentThread,
        effective: &EffectiveRunPolicy,
        artifact: &AgentArtifact,
        permissions: ThreadPermissions,
    ) -> Result<Self, PolicyIntersectionError> {
        root.validate()
            .map_err(|_| PolicyIntersectionError::IdentityMismatch)?;
        if root.parent_thread_id.is_some()
            || root.artifact_id != artifact.id
            || effective.agent_id.as_deref() != Some(artifact.id.as_str())
        {
            return Err(PolicyIntersectionError::IdentityMismatch);
        }
        let mut policy = Self {
            owner_id: root.owner_id.clone(),
            approval_root_run_id: root.root_run_id.clone(),
            effective: effective.clone(),
            permissions,
            artifact: artifact.clone(),
        };
        // Preserve the root's already-resolved provider route. Child artifact
        // routes are selected independently, within inherited bindings below.
        policy.narrow(artifact, true)?;
        Ok(policy)
    }

    /// Intersect one child artifact with this exact live parent snapshot.
    ///
    /// # Errors
    /// Unsupported restrictions and unavailable credential routes fail closed.
    /// No global resources, credentials, or approval state are consulted.
    pub fn intersect(&self, artifact: &AgentArtifact) -> Result<Self, PolicyIntersectionError> {
        let mut child = self.clone();
        child.narrow(artifact, false)?;
        Ok(child)
    }

    /// Resolve one exact A2A dependency declared by this execution artifact.
    /// Endpoint data identifies intent only; credentials come from host config.
    pub(crate) fn remote_agent_for_endpoint(
        &self,
        endpoint: &str,
    ) -> Result<String, PolicyIntersectionError> {
        let section = self
            .artifact
            .extensions
            .get("a2a")
            .ok_or(PolicyIntersectionError::UnsupportedShape { section: "a2a" })?;
        let section = strict_decode::<crate::uar::compiler::ir::A2ASection>(section, "a2a")?;
        let matches = section
            .dependencies
            .iter()
            .filter(|dependency| dependency.endpoint == endpoint)
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return Err(PolicyIntersectionError::UnsupportedShape {
                section: "a2a dependency",
            });
        }
        validate_id(&matches[0].agent_id, "a2a agent id")?;
        Ok(matches[0].agent_id.clone())
    }

    /// Build the local persisted proxy policy and secret-free contract ceiling
    /// for a remote child. The proxy starts from the already narrowed parent;
    /// no global resource or credential is re-resolved here.
    pub(crate) fn for_remote_child(
        &self,
        target_agent_id: &str,
    ) -> Result<(Self, RunPolicy, ThreadBudgets, SandboxPermissions), PolicyIntersectionError> {
        validate_id(target_agent_id, "a2a agent id")?;
        let constraint = concrete_scope_for(&self.effective, target_agent_id);
        let mut proxy = self.artifact.clone();
        proxy.id = target_agent_id.to_owned();
        proxy.metadata.title = target_agent_id.to_owned();
        proxy.extensions.insert(
            "uar.run_policy".into(),
            encode(&constraint, "uar.run_policy")?,
        );
        proxy.extensions.insert(
            "budgets".into(),
            encode(&self.permissions.budgets, "budgets")?,
        );
        let child = self.intersect(&proxy)?;
        Ok((
            child,
            constraint,
            self.permissions.budgets.clone(),
            self.permissions.sandbox.clone(),
        ))
    }

    /// Concrete selections passed unchanged to the child's turn kernel.
    pub fn effective(&self) -> &EffectiveRunPolicy {
        &self.effective
    }

    /// Host bindings and root-shared ceilings enforced at execution boundaries.
    pub fn permissions(&self) -> &ThreadPermissions {
        &self.permissions
    }

    /// Captured artifact with narrowed runtime fields; never mutate the source.
    pub fn artifact(&self) -> &AgentArtifact {
        &self.artifact
    }

    /// Verified root owner, inherited unchanged by descendants.
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }

    /// Only this run's authenticated approval channel may satisfy a request.
    pub fn approval_root_run_id(&self) -> &str {
        &self.approval_root_run_id
    }

    fn narrow(
        &mut self,
        artifact: &AgentArtifact,
        is_root: bool,
    ) -> Result<(), PolicyIntersectionError> {
        validate_id(&artifact.id, "artifact id")?;
        if artifact.kind != "agent" || self.effective.version != RUN_POLICY_VERSION {
            return Err(PolicyIntersectionError::UnsupportedShape {
                section: "effective policy",
            });
        }
        for grant in &self.permissions.credentials {
            validate_credential(grant)?;
        }
        for (id, binding) in &self.permissions.tool_bindings {
            validate_id(id, "tool id")?;
            if let ThreadToolBinding::Mcp(binding) = binding {
                validate_id(&binding.server_id, "MCP tool server")?;
                validate_id(&binding.tool_name, "MCP upstream tool")?;
            }
        }
        if self
            .effective
            .tools
            .ids
            .iter()
            .any(|id| !self.permissions.tool_bindings.contains_key(id))
        {
            return Err(PolicyIntersectionError::UnsupportedShape {
                section: "missing host tool binding",
            });
        }
        self.permissions.sandbox.validate()?;
        self.permissions.budgets.validate()?;

        let extension = artifact
            .extensions
            .get("uar.run_policy")
            .map(|value| strict_decode::<RunPolicy>(value, "uar.run_policy"))
            .transpose()?;
        if let Some(extension) = &extension {
            if extension.version != RUN_POLICY_VERSION {
                return Err(PolicyIntersectionError::UnsupportedShape {
                    section: "uar.run_policy.version",
                });
            }
            if extension
                .agent_id
                .as_ref()
                .is_some_and(|id| id != &artifact.id)
            {
                return Err(PolicyIntersectionError::IdentityMismatch);
            }
            if extension
                .chat_mode
                .is_some_and(|mode| mode != ChatMode::Agent && !is_root)
            {
                return Err(PolicyIntersectionError::UnsupportedShape {
                    section: "uar.run_policy.chat_mode",
                });
            }
        }
        let restrictions = artifact
            .extensions
            .get(THREAD_POLICY_EXTENSION)
            .map(|value| {
                strict_decode::<ArtifactThreadRestrictions>(value, THREAD_POLICY_EXTENSION)
            })
            .transpose()?;
        if let Some(restrictions) = restrictions {
            if restrictions.version != 1 {
                return Err(PolicyIntersectionError::UnsupportedShape {
                    section: THREAD_POLICY_EXTENSION,
                });
            }
            if let Some(credentials) = restrictions.credentials {
                for grant in &credentials {
                    validate_credential(grant)?;
                }
                self.permissions.credentials = self
                    .permissions
                    .credentials
                    .intersection(&credentials)
                    .cloned()
                    .collect();
            }
            if let Some(sandbox) = restrictions.sandbox {
                sandbox.validate()?;
                self.permissions.sandbox = self.permissions.sandbox.intersect(&sandbox);
            }
        }
        let budgets = ThreadBudgets::from_artifact(artifact)?;
        self.permissions.budgets = self.permissions.budgets.intersect(&budgets);
        self.permissions.sandbox.execution_mode = intersect_execution_mode(
            &self.permissions.sandbox.execution_mode,
            &artifact.policy.tools.execution_mode,
        );
        self.permissions.max_active_skills = self
            .permissions
            .max_active_skills
            .min(artifact.policy.skills.max_active);
        self.permissions.max_concurrent_tools = self
            .permissions
            .max_concurrent_tools
            .min(artifact.policy.tools.max_concurrent);

        // The legacy converter lets the extension replace base allow/deny sets.
        // Delegation instead intersects both scopes so neither can erase a deny.
        let mut base_artifact = artifact.clone();
        base_artifact.extensions.remove("uar.run_policy");
        let mut base = policy_from_agent_artifact(&base_artifact);
        for id in artifact
            .policy
            .tools
            .allow
            .iter()
            .chain(&artifact.policy.tools.deny)
        {
            if id != "*" {
                validate_id(id, "artifact tools")?;
            }
        }
        if artifact.policy.tools.deny.iter().any(|id| id == "*") {
            base.tools = ResourceSelection {
                mode: SelectionMode::None,
                ..ResourceSelection::default()
            };
        }
        let scopes = [Some(&base), extension.as_ref()];
        for scope in scopes.into_iter().flatten() {
            self.effective.skills = intersect_resources(&self.effective.skills, &scope.skills)?;
            self.effective.tools = intersect_resources(&self.effective.tools, &scope.tools)?;
            self.effective.mcp_servers =
                intersect_resources(&self.effective.mcp_servers, &scope.mcp_servers)?;
            self.effective.knowledge_bases =
                intersect_resources(&self.effective.knowledge_bases, &scope.knowledge_bases)?;
            self.effective.presentations =
                intersect_resources(&self.effective.presentations, &scope.presentations)?;
            self.effective.memory_enabled &= scope.memory_enabled.unwrap_or(true);
            self.effective.tool_approval =
                intersect_approval(self.effective.tool_approval, scope.tool_approval)?;
        }
        // Compiler-declared servers narrow the eligible identities too. Their
        // connection/auth/env data cannot replace inherited live host bindings.
        let declared_servers = match artifact.extensions.get("mcp_servers") {
            None | Some(Value::Null) => None,
            Some(value) => Some(
                strict_decode::<crate::uar::compiler::ir::McpServersSection>(value, "mcp_servers")?,
            ),
        };
        if let Some(servers) = &declared_servers {
            for server in &servers.servers {
                for tool in &server.tools {
                    validate_id(tool, "MCP declared tool")?;
                }
            }
            let selection =
                ResourceSelection::selected(servers.servers.iter().map(|server| server.id.clone()));
            self.effective.mcp_servers =
                intersect_resources(&self.effective.mcp_servers, &selection)?;
        }
        // A server without an inherited binding is unavailable, not a request
        // to resolve fresh credentials or start a differently configured server.
        self.effective.mcp_servers.ids.retain(|server| {
            self.permissions.credentials.iter().any(
                |grant| matches!(&grant.target, CredentialTarget::McpServer(id) if id == server),
            )
        });
        if self.effective.mcp_servers.ids.is_empty() {
            self.effective.mcp_servers = empty_selection();
        }
        self.permissions
            .credentials
            .retain(|grant| match &grant.target {
                CredentialTarget::Provider(_) => true,
                CredentialTarget::McpServer(id) => self.effective.mcp_servers.ids.contains(id),
            });
        self.effective.tools.ids.retain(|tool| {
            let binding = match self.permissions.tool_bindings.get(tool) {
                Some(ThreadToolBinding::Native) => return true,
                Some(ThreadToolBinding::Mcp(binding)) => binding,
                None => return false,
            };
            self.effective.mcp_servers.ids.contains(&binding.server_id)
                && declared_servers.as_ref().is_none_or(|servers| {
                    servers
                        .servers
                        .iter()
                        .filter(|server| server.id == binding.server_id)
                        .all(|server| {
                            server.tools.is_empty() || server.tools.contains(&binding.tool_name)
                        })
                })
        });
        if self.permissions.max_active_skills == 0 {
            self.effective.skills = empty_selection();
        }
        if self.permissions.max_concurrent_tools == 0
            || self.effective.tool_approval == ToolApprovalPolicy::Deny
        {
            self.effective.tools = empty_selection();
        }
        if self.effective.tools.ids.is_empty() {
            self.effective.tools = empty_selection();
        }
        self.permissions
            .tool_bindings
            .retain(|tool, _| self.effective.tools.ids.contains(tool));
        if !is_root {
            let inherited_model = self.effective.model.clone();
            self.effective.chat_mode = ChatMode::Agent;
            self.effective.agent_id = Some(artifact.id.clone());
            self.effective.model = extension
                .as_ref()
                .and_then(|scope| scope.model.clone())
                .or(base.model)
                .or(inherited_model);
            if let Some(strategy) = extension
                .as_ref()
                .and_then(|scope| scope.context_strategy.clone())
            {
                self.effective.context_strategy = strategy;
            }
        }
        let route =
            self.effective
                .model
                .as_ref()
                .ok_or(PolicyIntersectionError::UnsupportedShape {
                    section: "model route",
                })?;
        validate_id(&route.provider_id, "model provider")?;
        validate_id(&route.model_id, "model id")?;
        if !self.permissions.credentials.iter().any(|grant| {
            matches!(&grant.target, CredentialTarget::Provider(id) if id == &route.provider_id)
        }) {
            return Err(PolicyIntersectionError::CredentialNotGranted);
        }
        // Preserve routing restrictions even for callers that inspect the
        // artifact. Effective policy + host permissions remain authoritative.
        let mut narrowed = artifact.clone();
        narrowed
            .policy
            .provider
            .default
            .provider
            .clone_from(&route.provider_id);
        narrowed
            .policy
            .provider
            .default
            .model
            .clone_from(&route.model_id);
        for fallback in &narrowed.policy.provider.fallbacks {
            validate_id(&fallback.provider, "fallback provider")?;
            validate_id(&fallback.model, "fallback model")?;
        }
        narrowed.policy.provider.fallbacks.retain(|fallback| {
            self.permissions.credentials.iter().any(|grant| {
                matches!(&grant.target, CredentialTarget::Provider(id) if id == &fallback.provider)
            })
        });
        narrowed
            .policy
            .tools
            .allow
            .clone_from(&self.effective.tools.ids);
        narrowed.policy.tools.max_concurrent = self.permissions.max_concurrent_tools;
        narrowed.policy.tools.execution_mode = self.permissions.sandbox.execution_mode.clone();
        narrowed
            .policy
            .skills
            .prefer
            .clone_from(&self.effective.skills.ids);
        narrowed.policy.skills.max_active = self.permissions.max_active_skills;
        narrowed.memory.conversation.enabled = self.effective.memory_enabled;
        narrowed.memory.kb.enabled = !self.effective.knowledge_bases.ids.is_empty();
        narrowed
            .memory
            .kb
            .knowledge_bases
            .clone_from(&self.effective.knowledge_bases.ids);
        // Only the selected IDs enter the execution copy. The source artifact
        // remains intact, but its URL/auth/env declarations are never executable
        // delegation authority. The host already owns the connection bindings.
        narrowed.extensions.remove("mcp_servers");
        narrowed.extensions.insert(
            "budgets".into(),
            encode(&self.permissions.budgets, "budgets")?,
        );
        narrowed.extensions.insert(
            "uar.run_policy".into(),
            encode(&concrete_scope(&self.effective), "uar.run_policy")?,
        );
        narrowed.extensions.insert(
            THREAD_POLICY_EXTENSION.into(),
            encode(
                &ArtifactThreadRestrictions {
                    version: 1,
                    credentials: Some(self.permissions.credentials.clone()),
                    sandbox: Some(self.permissions.sandbox.clone()),
                },
                THREAD_POLICY_EXTENSION,
            )?,
        );
        for field in [
            "skills",
            "tools",
            "mcp_servers",
            "knowledge_bases",
            "presentations",
            "memory_enabled",
            "tool_approval",
        ] {
            self.effective
                .provenance
                .insert(field.into(), PolicyScope::Agent);
        }
        if !is_root {
            for field in ["chat_mode", "agent_id", "model"] {
                self.effective
                    .provenance
                    .insert(field.into(), PolicyScope::Agent);
            }
        }
        self.artifact = narrowed;
        Ok(())
    }
}

fn intersect_resources(
    parent: &EffectiveResourceSelection,
    child: &ResourceSelection,
) -> Result<EffectiveResourceSelection, PolicyIntersectionError> {
    if parent.mode == SelectionMode::Inherit
        || (parent.mode == SelectionMode::None && !parent.ids.is_empty())
        || (child.mode != SelectionMode::Selected && !child.ids.is_empty())
    {
        return Err(PolicyIntersectionError::UnsupportedShape {
            section: "resource selection",
        });
    }
    for id in parent.ids.iter().chain(&child.ids).chain(&child.denied_ids) {
        validate_id(id, "resource id")?;
    }
    let mut ids: BTreeSet<_> = parent.ids.iter().cloned().collect();
    if child.mode == SelectionMode::None {
        ids.clear();
    }
    if child.mode == SelectionMode::Selected {
        ids.retain(|id| child.ids.contains(id));
    }
    ids.retain(|id| !child.denied_ids.contains(id));
    Ok(EffectiveResourceSelection {
        mode: if ids.is_empty() {
            SelectionMode::None
        } else {
            SelectionMode::Selected
        },
        ids: ids.into_iter().collect(),
        source: PolicyScope::Agent,
    })
}

fn empty_selection() -> EffectiveResourceSelection {
    EffectiveResourceSelection {
        mode: SelectionMode::None,
        ids: Vec::new(),
        source: PolicyScope::Agent,
    }
}

fn concrete_scope(policy: &EffectiveRunPolicy) -> RunPolicy {
    concrete_scope_with_agent(policy, policy.agent_id.clone())
}

fn concrete_scope_for(policy: &EffectiveRunPolicy, agent_id: &str) -> RunPolicy {
    let mut child = concrete_scope_with_agent(policy, Some(agent_id.to_owned()));
    // A named remote child is an agent execution even when its parent is the
    // UAR router. Retain every inherited resource ceiling, not its routing mode.
    child.chat_mode = Some(ChatMode::Agent);
    child
}

fn concrete_scope_with_agent(policy: &EffectiveRunPolicy, agent_id: Option<String>) -> RunPolicy {
    let selection = |value: &EffectiveResourceSelection| ResourceSelection {
        mode: value.mode,
        ids: value.ids.clone(),
        denied_ids: Vec::new(),
    };
    RunPolicy {
        version: policy.version,
        chat_mode: Some(policy.chat_mode),
        agent_id,
        model: policy.model.clone(),
        skills: selection(&policy.skills),
        tools: selection(&policy.tools),
        mcp_servers: selection(&policy.mcp_servers),
        knowledge_bases: selection(&policy.knowledge_bases),
        presentations: selection(&policy.presentations),
        memory_enabled: Some(policy.memory_enabled),
        prompt_caching_enabled: Some(policy.prompt_caching_enabled),
        context_strategy: Some(policy.context_strategy.clone()),
        tool_approval: policy.tool_approval,
    }
}

fn intersect_approval(
    parent: ToolApprovalPolicy,
    child: ToolApprovalPolicy,
) -> Result<ToolApprovalPolicy, PolicyIntersectionError> {
    if parent == ToolApprovalPolicy::Inherit {
        return Err(PolicyIntersectionError::UnsupportedShape {
            section: "effective approval",
        });
    }
    Ok(match (parent, child) {
        (ToolApprovalPolicy::Deny, _) | (_, ToolApprovalPolicy::Deny) => ToolApprovalPolicy::Deny,
        (ToolApprovalPolicy::Ask, _) | (_, ToolApprovalPolicy::Ask) => ToolApprovalPolicy::Ask,
        _ => ToolApprovalPolicy::Auto,
    })
}

pub(crate) fn intersect_execution_mode(
    parent: &ToolExecutionMode,
    child: &ToolExecutionMode,
) -> ToolExecutionMode {
    match (parent, child) {
        (ToolExecutionMode::Sandboxed, _) | (_, ToolExecutionMode::Sandboxed) => {
            ToolExecutionMode::Sandboxed
        }
        (ToolExecutionMode::Auto, _) | (_, ToolExecutionMode::Auto) => ToolExecutionMode::Auto,
        _ => ToolExecutionMode::Direct,
    }
}

fn lower<T: Copy + PartialOrd>(parent: Option<T>, child: Option<T>) -> Option<T> {
    match (parent, child) {
        (Some(parent), Some(child)) => Some(if parent <= child { parent } else { child }),
        (parent, child) => parent.or(child),
    }
}

fn validate_id(id: &str, section: &'static str) -> Result<(), PolicyIntersectionError> {
    if id.trim().is_empty()
        || id != id.trim()
        || id.contains(['*', '?'])
        || id.chars().any(char::is_control)
    {
        return Err(PolicyIntersectionError::UnsupportedShape { section });
    }
    Ok(())
}

fn validate_credential(grant: &CredentialGrant) -> Result<(), PolicyIntersectionError> {
    let id = match &grant.target {
        CredentialTarget::Provider(id) | CredentialTarget::McpServer(id) => id,
    };
    validate_id(id, "credential target")?;
    validate_id(&grant.binding_id, "credential binding")
}

fn encode<T: Serialize>(
    value: &T,
    section: &'static str,
) -> Result<Value, PolicyIntersectionError> {
    serde_json::to_value(value).map_err(|_| PolicyIntersectionError::UnsupportedShape { section })
}

// Existing RunPolicy and ContextStrategy serde implementations ignore unknown
// fields. Check every supplied field against their canonical representation so
// delegation cannot silently discard an unsupported nested policy restriction.
fn strict_decode<T: DeserializeOwned + Serialize>(
    value: &Value,
    section: &'static str,
) -> Result<T, PolicyIntersectionError> {
    let decoded: T = serde_json::from_value(value.clone())
        .map_err(|_| PolicyIntersectionError::UnsupportedShape { section })?;
    let canonical = encode(&decoded, section)?;
    if !known_fields(value, &canonical) {
        return Err(PolicyIntersectionError::UnsupportedShape { section });
    }
    Ok(decoded)
}

fn known_fields(input: &Value, canonical: &Value) -> bool {
    match (input, canonical) {
        (Value::Object(input), Value::Object(canonical)) => input.iter().all(|(key, value)| {
            canonical
                .get(key)
                .is_some_and(|field| known_fields(value, field))
        }),
        (Value::Array(input), Value::Array(canonical)) => {
            input.len() == canonical.len()
                && input
                    .iter()
                    .zip(canonical)
                    .all(|(value, field)| known_fields(value, field))
        }
        // Null optional fields and scalar values were type-checked by serde.
        (Value::Object(_), _) | (Value::Array(_), _) => false,
        _ => true,
    }
}
