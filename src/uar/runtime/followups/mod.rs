//! Owner-controlled follow-ups into a durable conversation inbox.
mod controller;
pub mod quiet_hours;
pub use controller::FollowupController;
#[derive(Debug, thiserror::Error)]
pub enum FollowupError {
    #[error("{0}")] Invalid(&'static str),
    #[error("{0}")] Conflict(&'static str),
    #[error("follow-up not found")] NotFound,
    #[error("follow-up authority denied")] Denied,
    #[error("durable follow-up persistence is unavailable")] Unsupported,
    #[error("follow-up persistence needs unsupported store capability {0}")] CapabilityUnsupported(&'static str),
    #[error("follow-up persistence operation failed")] Storage,
}
impl FollowupError {
    pub fn code(&self) -> &'static str {
        match self { Self::Invalid(_) => "followup_invalid", Self::Conflict(_) => "followup_conflict",
            Self::NotFound => "followup_not_found", Self::Denied => "followup_denied",
            Self::Unsupported | Self::CapabilityUnsupported(_) => crate::uar::persistence::capabilities::CAPABILITY_UNSUPPORTED,
            Self::Storage => "followup_storage_unavailable" }
    }
    /// The store capability a refusal names; `None` for every other failure.
    pub fn capability(&self) -> Option<&'static str> {
        match self { Self::Unsupported => Some(crate::uar::persistence::capabilities::DURABLE_COLLABORATION_CATALOG),
            Self::CapabilityUnsupported(capability) => Some(*capability), _ => None }
    }
}
