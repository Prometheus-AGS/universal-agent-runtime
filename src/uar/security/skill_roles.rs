//! Operator skill authorization failures, separate from model-selected policy.

#[derive(Debug, thiserror::Error)]
pub enum SkillAdmissionError {
    #[error("Insufficient skill roles")]
    Forbidden,
    #[error("Protected skill is unavailable")]
    Unavailable,
}
