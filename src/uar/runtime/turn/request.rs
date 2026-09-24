use crate::llm::Message;
use crate::uar::domain::{
    artifact::AgentArtifact,
    events::MemoryItem,
    policy::{EffectiveRunPolicy, RunPolicy},
};
use crate::uar::runtime::{graph::GraphState, manager::SeedMessage};

/// Complete host-verified material required to resume one persisted checkpoint.
/// Keeping these values together makes partial or state-only restoration
/// unrepresentable at the shared request boundary.
#[derive(Debug, Clone)]
pub struct CheckpointResume {
    pub state: GraphState,
    pub history: Vec<Message>,
    pub authorization_digest: String,
}

/// Owned execution input shared by HTTP, embedded, and checkpoint adapters.
#[derive(Debug, Clone)]
pub struct RunExecutionRequest {
    pub artifact: AgentArtifact,
    pub input: Option<String>,
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    /// Host-verified user and tenant identity for executable resource bindings.
    /// A legacy user ID alone is not sufficient to populate this field.
    pub verified_owner: Option<crate::uar::runtime::actor::messages::ActorOwner>,
    /// Root-host MCP capture, never accepted from JSON or model arguments.
    /// Requires a matching verified owner. When policy is not already resolved,
    /// the manager resolves it against this capture's immutable catalog.
    pub mcp_resources: Option<crate::mcp::runtime::McpRunResources>,
    /// Request-scoped provider definitions. No member implements Serialize.
    pub(crate) run_credentials: Option<super::host::RunCredentials>,
    /// Safe names-only marker retained after secret resources are dropped.
    pub(crate) host_resources_marker: super::host::HostResourcesMarker,
    pub(crate) host_secret_scrubber: super::host::RunSecretScrubber,
    pub memory_hits: Vec<MemoryItem>,
    pub resolved_policy: Option<EffectiveRunPolicy>,
    /// Client output restrictions; these never grant template/resource access.
    pub presentation_negotiation: crate::uar::a2ui::presentation_selection::PresentationNegotiation,
    /// Restriction-only turn scope accepted from a verified host adapter.
    /// This is resolved with local Global/Agent/Conversation policy; it never
    /// replaces the target runtime's resource universe.
    pub(crate) host_policy_constraint: Option<RunPolicy>,
    /// Root budget ceiling from a verified host adapter. Local artifact limits
    /// intersect with it before any executable binding is captured.
    pub(crate) host_budget_constraint:
        Option<crate::uar::runtime::thread::policy_intersection::ThreadBudgets>,
    /// Stable host-only cumulative usage grant shared across governed peer turns.
    pub(crate) host_usage_grant: Option<crate::uar::runtime::cost_budget::RemoteUsageGrantBinding>,
    pub(crate) host_sandbox_constraint:
        Option<crate::uar::runtime::thread::policy_intersection::SandboxPermissions>,
    pub seed_history: Vec<SeedMessage>,
    /// Validated host history for a cold sidecar session.
    pub(crate) host_history: Option<Vec<Message>>,
    /// Host-selected reasoning override for this run and local children.
    pub(crate) reasoning_effort: Option<crate::config::ReasoningEffort>,
    /// Persisted checkpoint restoration admitted by the trusted host. The
    /// manager always resolves current policy again before accepting it.
    pub(crate) checkpoint_resume: Option<CheckpointResume>,
    /// Canonical child history captured with inherited host policy. This is not
    /// persisted checkpoint material and does not use checkpoint authorization.
    pub(crate) inherited_history: Option<Vec<Message>>,
    pub skill_attachments: Vec<String>,
    /// Host-selected cwd. This never grants workspace trust or file permissions.
    pub working_directory: Option<std::path::PathBuf>,
}

impl RunExecutionRequest {
    pub fn new(artifact: AgentArtifact, input: String) -> Self {
        Self {
            artifact,
            input: Some(input),
            session_id: None,
            user_id: None,
            verified_owner: None,
            mcp_resources: None,
            run_credentials: None,
            host_resources_marker: Default::default(),
            host_secret_scrubber: Default::default(),
            memory_hits: Vec::new(),
            resolved_policy: None,
            presentation_negotiation: Default::default(),
            host_policy_constraint: None,
            host_budget_constraint: None,
            host_usage_grant: None,
            host_sandbox_constraint: None,
            seed_history: Vec::new(),
            host_history: None,
            reasoning_effort: None,
            checkpoint_resume: None,
            inherited_history: None,
            skill_attachments: Vec::new(),
            working_directory: None,
        }
    }

    /// Retain the identity verified by the ingress host, without decoding a
    /// credential or taking tenant identity from a run/model payload.
    pub fn with_verified_owner(
        mut self,
        owner: crate::uar::runtime::actor::messages::ActorOwner,
    ) -> Self {
        self.user_id = Some(owner.user_id().to_owned());
        self.verified_owner = Some(owner);
        self
    }

    /// Capture middleware-established identity. Anonymous middleware context
    /// retains its existing behavior but cannot acquire a verified cache owner.
    ///
    /// # Errors
    /// Rejects an inconsistent principal or an anonymous tenant assertion.
    pub fn with_user_context(
        mut self,
        user: &crate::uar::security::claims::UserContext,
    ) -> anyhow::Result<Self> {
        if user.user_id == crate::session::ANONYMOUS_SESSION_OWNER {
            anyhow::ensure!(
                user.claims.sub == user.user_id && user.tenant_id.is_none(),
                "Anonymous run context cannot carry another principal or tenant"
            );
            self.user_id = Some(user.user_id.clone());
            self.verified_owner = None;
            return Ok(self);
        }
        Ok(self.with_verified_owner(
            crate::uar::runtime::actor::messages::ActorOwner::from_verified_context(user)?,
        ))
    }
}
