//! Stable observer failures for authenticated API responses.

#[derive(Debug, thiserror::Error)]
pub enum ObserverError {
    #[error("observer request is invalid: {0}")]
    Invalid(&'static str),
    #[error("observer subscription or instance was not found")]
    NotFound,
    #[error("observer state or current authority conflicts with this action: {0}")]
    Conflict(&'static str),
    #[error("observer inbox is full")]
    Capacity,
    #[error("durable observer storage is unavailable: store capability {0} is unsupported")]
    CapabilityUnsupported(&'static str),
    #[error("observer operation failed: {0}")]
    Internal(#[source] anyhow::Error),
}

impl ObserverError {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Invalid(_) => "observer_invalid",
            Self::NotFound => "observer_not_found",
            Self::Conflict(_) => "observer_conflict",
            Self::Capacity => "observer_capacity",
            Self::CapabilityUnsupported(_) => crate::uar::persistence::capabilities::CAPABILITY_UNSUPPORTED,
            Self::Internal(_) => "observer_internal",
        }
    }

    /// The store capability a refusal names; `None` for every other failure.
    #[must_use]
    pub fn capability(&self) -> Option<&'static str> {
        match self {
            Self::CapabilityUnsupported(capability) => Some(*capability),
            _ => None,
        }
    }

    #[must_use]
    pub fn status_code(&self) -> u16 {
        match self {
            Self::Invalid(_) => 400,
            Self::NotFound => 404,
            Self::Conflict(_) => 409,
            Self::Capacity => 429,
            Self::CapabilityUnsupported(_) => 503,
            Self::Internal(_) => 500,
        }
    }
}
