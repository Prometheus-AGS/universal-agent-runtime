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
    #[error("follow-up persistence operation failed")] Storage,
}
impl FollowupError {
    pub fn code(&self) -> &'static str {
        match self { Self::Invalid(_) => "followup_invalid", Self::Conflict(_) => "followup_conflict",
            Self::NotFound => "followup_not_found", Self::Denied => "followup_denied",
            Self::Unsupported => "followup_unsupported", Self::Storage => "followup_storage_unavailable" }
    }
}
