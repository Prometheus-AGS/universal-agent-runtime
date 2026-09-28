//! Stable controller failures for transport-specific status mapping.

#[derive(Debug, thiserror::Error)]
pub enum AgentInstanceError {
    #[error("invalid instance request: {0}")]
    Invalid(&'static str),
    #[error("logical instance not found")]
    NotFound,
    #[error("logical instance state conflicts with this action: {0}")]
    Conflict(&'static str),
    #[error("logical instance inbox is full")]
    Capacity,
    #[error("durable instance storage is unavailable")]
    Unavailable,
    #[error("logical instance operation failed: {0}")]
    Internal(#[source] anyhow::Error),
}

impl AgentInstanceError {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Invalid(_) => "instance_invalid",
            Self::NotFound => "instance_not_found",
            Self::Conflict(_) => "instance_conflict",
            Self::Capacity => "instance_capacity",
            Self::Unavailable => "instance_unavailable",
            Self::Internal(_) => "instance_internal",
        }
    }

    #[must_use]
    pub fn status_code(&self) -> u16 {
        match self {
            Self::Invalid(_) => 400,
            Self::NotFound => 404,
            Self::Conflict(_) => 409,
            Self::Capacity => 429,
            Self::Unavailable => 503,
            Self::Internal(_) => 500,
        }
    }
}
