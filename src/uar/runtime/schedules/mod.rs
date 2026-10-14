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
    #[error("schedule persistence operation failed")] Storage,
}

impl ScheduleError {
    pub fn code(&self) -> &'static str {
        match self { Self::Invalid(_) => "schedule_invalid", Self::Conflict(_) => "schedule_conflict",
            Self::NotFound => "schedule_not_found", Self::Denied => "schedule_denied",
            Self::Unsupported => "schedule_unsupported", Self::Storage => "schedule_storage_unavailable" }
    }
}
