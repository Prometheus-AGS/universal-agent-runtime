//! Durable schedule host. Reuses logical-instance admission and observer wakeups.
pub mod calendar;
mod controller;
pub use controller::ScheduleController;

#[derive(Debug, thiserror::Error)]
pub enum ScheduleError {
    #[error("{0}")] Invalid(&'static str),
    #[error("{0}")] Conflict(&'static str),
    #[error("schedule not found")] NotFound,
    #[error("schedule authority denied")] Denied,
    #[error("durable schedule persistence is unavailable")] Unsupported,
    #[error("schedule persistence needs unsupported store capability {0}")] CapabilityUnsupported(&'static str),
    #[error("schedule persistence operation failed")] Storage,
}

impl ScheduleError {
    pub fn code(&self) -> &'static str {
        match self { Self::Invalid(_) => "schedule_invalid", Self::Conflict(_) => "schedule_conflict",
            Self::NotFound => "schedule_not_found", Self::Denied => "schedule_denied",
            Self::Unsupported | Self::CapabilityUnsupported(_) => crate::uar::persistence::capabilities::CAPABILITY_UNSUPPORTED,
            Self::Storage => "schedule_storage_unavailable" }
    }

    /// The store capability a refusal names; `None` for every other failure.
    pub fn capability(&self) -> Option<&'static str> {
        match self { Self::Unsupported => Some(crate::uar::persistence::capabilities::DURABLE_COLLABORATION_CATALOG),
            Self::CapabilityUnsupported(capability) => Some(*capability), _ => None }
    }
}
