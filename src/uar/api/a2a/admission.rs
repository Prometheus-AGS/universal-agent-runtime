//! Trusted-host admission for A2A task creation.
//!
//! Transport adapters authenticate the caller, then ask this host-owned
//! boundary whether the selected agent may be admitted. The boundary is
//! intentionally outside client metadata and skill documents: it receives a
//! verified identity and a server-selected agent identifier only.

use std::fmt::Debug;

use crate::uar::security::claims::UserContext;

/// Admission outcome that transports project through their native error model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum A2aAdmissionError {
    /// The authenticated principal does not hold a role required by the target.
    Forbidden,
    /// Host policy or its required source cannot be resolved safely.
    Unavailable,
}

/// Trusted host contract for named A2A task admission.
///
/// Implementations resolve protected explicit skills for `agent_id` from
/// server-owned policy and inspect only verified roles carried by `user`.
/// They must not accept client-provided skill IDs or serialized grants.
#[async_trait::async_trait]
pub trait A2aAdmission: Debug + Send + Sync {
    /// Admit a new durable task before reservation or actor creation.
    async fn admit(
        &self,
        user: &UserContext,
        agent_id: &str,
    ) -> Result<(), A2aAdmissionError>;
}

#[async_trait::async_trait]
impl A2aAdmission for crate::uar::runtime::manager::RunManager {
    async fn admit(
        &self,
        user: &UserContext,
        agent_id: &str,
    ) -> Result<(), A2aAdmissionError> {
        let artifact = self
            .resolve_registered_agent(agent_id)
            .await
            .map_err(|_| A2aAdmissionError::Unavailable)?;
        self.authorize_preferred_skills(user, &artifact)
            .await
            .map_err(|error| match error {
                crate::uar::security::skill_roles::SkillAdmissionError::Forbidden => {
                    A2aAdmissionError::Forbidden
                }
                crate::uar::security::skill_roles::SkillAdmissionError::Unavailable => {
                    A2aAdmissionError::Unavailable
                }
            })
    }
}
