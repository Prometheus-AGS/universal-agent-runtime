//! Message types for inter-agent communication.
//!
//! Defines the typed message envelope that agent actors exchange,
//! plus reply types for request-response patterns.

use crate::uar::security::credential_capture::AuthenticatedCredentialCapture;
use serde::{Deserialize, Serialize};

/// Host-created actor namespace. Never deserialize identity from actor payloads.
#[derive(Debug, Clone)]
pub struct ActorOwner {
    // Fresh host-captured authority is separate from stable ownership identity.
    verified_roles: std::collections::BTreeSet<String>,
    user_id: String,
    tenant_id: Option<String>,
    credential_principal: Option<crate::uar::security::claims::CredentialPrincipal>,
}

impl PartialEq for ActorOwner {
    fn eq(&self, other: &Self) -> bool {
        self.user_id == other.user_id && self.tenant_id == other.tenant_id
            && self.credential_principal == other.credential_principal
    }
}
impl Eq for ActorOwner {}
impl std::hash::Hash for ActorOwner {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&self.user_id, state);
        std::hash::Hash::hash(&self.tenant_id, state);
        std::hash::Hash::hash(&self.credential_principal, state);
    }
}

impl ActorOwner {
    pub(crate) fn verified_roles(&self) -> &std::collections::BTreeSet<String> {
        &self.verified_roles
    }

    /// Capture identity established by the authentication middleware.
    ///
    /// # Errors
    /// Rejects absent/anonymous or inconsistent principal identity. This is not
    /// a credential verifier: the context must already come from the trusted host.
    pub fn from_verified_context(
        user: &crate::uar::security::claims::UserContext,
    ) -> anyhow::Result<Self> {
        if user.user_id.trim().is_empty()
            || user.user_id == "anonymous"
            || user.claims.sub != user.user_id
        {
            anyhow::bail!("Actor operations require authenticated user context");
        }
        Ok(Self {
            verified_roles: user.verified_identity().map(|identity| identity.roles().iter().cloned().collect()).unwrap_or_default(),
            credential_principal: crate::uar::security::claims::CredentialPrincipal::from_verified_context(user).ok(),
            user_id: user.user_id.clone(),
            tenant_id: user
                .tenant_id
                .as_ref()
                .map(|tenant| tenant.as_str().to_string()),
        })
    }

    /// Verified user identity supplied to the shared run kernel.
    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    /// Rehydrate an identity captured at authenticated subscription creation.
    /// This only reconstructs the namespace; callers must recheck live authority.
    pub(crate) fn from_persisted_identity(
        user_id: &str,
        tenant_id: Option<&str>,
        expected_owner_key: &str,
    ) -> anyhow::Result<Self> {
        if user_id.trim().is_empty() || user_id == "anonymous" {
            anyhow::bail!("Persisted observer principal is invalid");
        }
        let owner = Self {
            verified_roles: Default::default(),
            // Subject/tenant persistence is not enough to reconstruct issuer proof.
            credential_principal: None,
            user_id: user_id.to_owned(),
            tenant_id: tenant_id.map(str::to_owned),
        };
        if owner.presentation_owner_key() != expected_owner_key {
            anyhow::bail!("Persisted observer principal does not match its owner scope");
        }
        Ok(owner)
    }

    /// The tenant part of a host-verified owner, if present.
    pub(crate) fn tenant_id(&self) -> Option<&str> {
        self.tenant_id.as_deref()
    }

    /// Full identity captured at ingress, retained for secret resolution and cache
    /// partitioning. Legacy persisted observers have no credential capability.
    pub fn credential_principal(&self) -> Option<&crate::uar::security::claims::CredentialPrincipal> {
        self.credential_principal.as_ref()
    }

    /// Reconstruct only the authority delegated to a protected durable schedule.
    /// The schedule host must load and recheck the exact record before dispatch.
    pub(crate) fn from_schedule_grant(
        grant: &crate::uar::security::claims::ScheduleOwnerGrant,
        owner_key: &str, schedule_id: &str, workspace: &str, instance: &str,
    ) -> anyhow::Result<Self> {
        let principal = grant.restore(owner_key, schedule_id, workspace, instance)?;
        Ok(Self { verified_roles: Default::default(), user_id: principal.subject().to_owned(),
            tenant_id: principal.tenant_id().map(str::to_owned),
            credential_principal: Some(principal) })
    }

    /// Scope-bound restoration for the trusted durable follow-up host only.
    pub(crate) fn from_followup_grant(
        grant: &crate::uar::security::claims::FollowupOwnerGrant,
        owner_key: &str, rule_id: &str, workspace: &str, instance: &str,
    ) -> anyhow::Result<Self> {
        let principal = grant.restore(owner_key, rule_id, workspace, instance)?;
        Ok(Self { verified_roles: Default::default(), user_id: principal.subject().to_owned(), tenant_id: principal.tenant_id().map(str::to_owned),
            credential_principal: Some(principal) })
    }

    /// History and conversation policies require the full verified principal.
    pub(crate) fn session_owner_key(&self) -> anyhow::Result<String> {
        self.credential_principal.as_ref().map(|principal| principal.storage_key())
            .ok_or_else(|| anyhow::anyhow!("Verified conversation owner required"))
    }

    /// Catalog partition for this host-verified tenant and subject.
    /// Matches the existing principal storage-key encoding, including lengths.
    pub(crate) fn presentation_owner_key(&self) -> String {
        match &self.tenant_id {
            Some(tenant) => format!(
                "v1:t:{}:{}:s:{}:{}",
                tenant.len(),
                tenant,
                self.user_id.len(),
                self.user_id
            ),
            None => format!("v1:s:{}:{}", self.user_id.len(), self.user_id),
        }
    }
}

/// Messages that can be sent to an [`AgentActor`](super::agent_actor::AgentActor).
#[derive(Debug)]
pub enum AgentMessage {
    /// A trusted transport's exact run submission. The host allocates run_id;
    /// wire payloads cannot create this envelope or replace its identity.
    UserRun {
        owner: ActorOwner,
        run_id: String,
        content: String,
        credential_capture: AuthenticatedCredentialCapture,
        artifacts: crate::uar::runtime::thread::artifacts::RunArtifactCollector,
        reply: tokio::sync::oneshot::Sender<
            Result<crate::uar::persistence::agent_threads::PersistedAgentThread, ActorRunError>,
        >,
    },
    /// A user or system prompt to process.
    UserPrompt {
        owner: ActorOwner,
        /// The text prompt to handle.
        content: String,
        /// Optional reply channel for the response.
        reply: Option<tokio::sync::oneshot::Sender<AgentReply>>,
    },

    /// Request collaboration from this actor.
    /// Another agent is asking this actor to help with a sub-task.
    Collaborate {
        /// ID of the requesting agent.
        from_agent_id: String,
        /// The task description or query.
        task: String,
        /// Reply channel for the collaboration result.
        reply: tokio::sync::oneshot::Sender<AgentReply>,
    },

    /// A tool execution result being returned to the actor.
    ToolResult {
        /// Tool call ID.
        tool_call_id: String,
        /// Serialized result content.
        content: serde_json::Value,
        /// Whether the tool call succeeded.
        success: bool,
    },

    /// Administrative: request the actor to stop gracefully.
    Shutdown,
}

/// Host failure is separate from a persisted failed/cancelled turn outcome.
#[derive(Debug, thiserror::Error)]
pub enum ActorRunError {
    #[error("actor stopped before kernel entry")]
    Stopped,
    #[error("actor host failed: {0}")]
    Host(#[source] anyhow::Error),
}

/// Reply sent back through oneshot channels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentReply {
    /// The agent's response content.
    pub content: String,
    /// Whether the operation succeeded.
    pub success: bool,
    /// Optional metadata about the response.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Summary information about a running actor (for API responses).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorInfo {
    /// Actor name / ID.
    pub id: String,
    /// Agent artifact ID this actor is running.
    pub agent_id: String,
    /// Current status.
    pub status: ActorStatus,
    /// Owner-scoped conversation used by the shared run kernel.
    pub session_id: String,
    /// Present after the first root thread has been committed.
    pub thread_id: Option<String>,
    /// Latest committed run identity, usable for streaming and approvals.
    pub run_id: Option<String>,
}

/// Actor lifecycle status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActorStatus {
    /// Actor is starting up.
    Starting,
    /// Actor is running and ready to process messages.
    Running,
    /// Actor is shutting down.
    Stopping,
    /// Actor has stopped.
    Stopped,
}
