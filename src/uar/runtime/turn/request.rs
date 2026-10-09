use crate::llm::Message;
use crate::uar::domain::{
    artifact::AgentArtifact,
    collaboration::EffectiveBindingReceipt,
    events::MemoryItem,
    policy::{EffectiveRunPolicy, RunPolicy},
};
use crate::uar::runtime::{graph::GraphState, manager::SeedMessage};

#[derive(Debug, Clone)]
pub struct CollaborationRunBinding {
    pub owner_id: String,
    pub workspace_id: String,
    pub receipt: EffectiveBindingReceipt,
    pub(crate) team_attempt: Option<crate::uar::domain::team_execution::TeamExecutionAttempt>,
    pub(crate) team_instructions: Option<crate::uar::domain::team_context::TeamInstructions>,
    pub(crate) team_yield:
        std::sync::Arc<std::sync::Mutex<Option<crate::uar::domain::team_wait::KernelTeamYield>>>,
    pub(crate) service:
        std::sync::Arc<crate::uar::compiler::collaboration::CollaborationCatalogService>,
}

impl CollaborationRunBinding {
    pub async fn revalidate(&self, artifact: &AgentArtifact) -> anyhow::Result<()> {
        super::bindings::validate_effective_binding_artifact(&self.receipt, artifact)?;
        self.revalidate_authority().await
    }

    async fn revalidate_authority(&self) -> anyhow::Result<()> {
        match &self.team_attempt {
            Some(attempt) => {
                crate::uar::runtime::team_execution::revalidate_member_binding(
                    &self.service,
                    attempt,
                    &self.receipt,
                )
                .await
            }
            None => self
                .service
                .revalidate_effective_binding(&self.owner_id, &self.workspace_id, &self.receipt)
                .await
                .map_err(anyhow::Error::from),
        }
    }
}

#[async_trait::async_trait]
impl crate::uar::runtime::tool_admission::ClaimRevalidator for CollaborationRunBinding {
    async fn revalidate(&self) -> anyhow::Result<()> {
        self.revalidate_authority().await
    }
}

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
    /// Request-scoped paired-host admission. This capability is constructed by
    /// the authenticated host adapter and never deserialized into core state.
    pub(crate) host_tool_admission:
        Option<std::sync::Arc<dyn crate::uar::runtime::tool_admission::HostToolAdmissionPort>>,
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
    /// Private effective binding captured by the authenticated collaboration
    /// adapter. It is revalidated before execution and again before tool claim.
    pub(crate) collaboration_binding: Option<CollaborationRunBinding>,
    /// Exact host-owned logical activation, checked again before protected effects.
    pub(crate) instance_binding: Option<crate::uar::runtime::instance::InstanceEpochBinding>,
    /// Host-selected service identity verified before executable admission.
    pub(crate) service_binding: Option<crate::uar::service_instance::EffectiveServiceBinding>,
    pub skill_attachments: Vec<String>,
    /// Host-selected cwd. This never grants workspace trust or file permissions.
    pub working_directory: Option<std::path::PathBuf>,
}

impl RunExecutionRequest {
    /// Merge redaction-only ingress data without changing run authority.
    pub(crate) fn with_credential_capture(
        mut self,
        capture: Option<crate::uar::security::credential_capture::AuthenticatedCredentialCapture>,
    ) -> Self {
        if let Some(capture) = capture {
            self.host_secret_scrubber.extend(capture.into_scrubber());
        }
        self
    }

    pub fn new(artifact: AgentArtifact, input: String) -> Self {
        Self {
            artifact,
            input: Some(input),
            session_id: None,
            user_id: None,
            verified_owner: None,
            mcp_resources: None,
            host_tool_admission: None,
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
            collaboration_binding: None,
            instance_binding: None,
            service_binding: None,
            skill_attachments: Vec::new(),
            working_directory: None,
        }
    }

    #[must_use]
    pub fn from_bound_agent(
        bound: crate::uar::compiler::collaboration::BoundAgentRun,
        input: String,
        owner_id: String,
        workspace_id: String,
        service: std::sync::Arc<crate::uar::compiler::collaboration::CollaborationCatalogService>,
    ) -> Self {
        let service_binding = bound.effective_binding_receipt.service_binding.clone();
        let mut artifact = bound.artifact;
        let primary_index = bound
            .effective_binding_receipt
            .resolved_models
            .iter()
            .position(|model| {
                model.get("role").and_then(serde_json::Value::as_str) == Some("primary")
            })
            .unwrap_or(0);
        if let Some(model) = bound
            .effective_binding_receipt
            .resolved_models
            .get(primary_index)
        {
            if let Some(provider_id) = model.get("providerId").and_then(serde_json::Value::as_str) {
                artifact.policy.provider.default.provider = provider_id.to_owned();
            }
            if let Some(model_id) = model.get("modelId").and_then(serde_json::Value::as_str) {
                artifact.policy.provider.default.model = model_id.to_owned();
            }
        }
        artifact.policy.provider.fallbacks = bound
            .effective_binding_receipt
            .resolved_models
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != primary_index)
            .filter_map(|(_, model)| {
                Some(crate::uar::domain::artifact::ProviderSelection {
                    provider: model.get("providerId")?.as_str()?.to_owned(),
                    model: model.get("modelId")?.as_str()?.to_owned(),
                })
            })
            .collect();
        let skill_attachments = bound
            .effective_binding_receipt
            .resolved_skills
            .iter()
            .map(|resolved| resolved.skill.id.clone())
            .collect();
        let collaboration_binding = CollaborationRunBinding {
            owner_id,
            workspace_id,
            receipt: bound.effective_binding_receipt,
            team_attempt: None,
            team_yield: Default::default(),
            team_instructions: None,
            service,
        };
        let mut request = Self::new(artifact, input);
        request.host_policy_constraint = collaboration_binding
            .receipt
            .effective
            .get("contextStrategy")
            .cloned()
            .and_then(|value| serde_json::from_value(value).ok())
            .map(|context_strategy| RunPolicy {
                context_strategy: Some(context_strategy),
                ..RunPolicy::default()
            });
        request.skill_attachments = skill_attachments;
        request.collaboration_binding = Some(collaboration_binding);
        request.service_binding = service_binding;
        request
    }

    pub(crate) fn with_team_attempt(
        mut self,
        attempt: crate::uar::domain::team_execution::TeamExecutionAttempt,
    ) -> anyhow::Result<Self> {
        let binding = self
            .collaboration_binding
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Team attempt has no bound member receipt"))?;
        let policy = binding
            .receipt
            .effective
            .get("teamRunPolicy")
            .ok_or_else(|| anyhow::anyhow!("Team member receipt has no resource policy"))?;
        self.host_policy_constraint = Some(serde_json::from_value(policy.clone())?);
        binding.team_attempt = Some(attempt);
        Ok(self)
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
