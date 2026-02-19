//! Compile report types.
//!
//! The [`CompileReport`] records the outcome of each pipeline stage, providing
//! a full audit trail of the compilation process.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Complete report produced by the compilation pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileReport {
    /// Unique ID of this report.
    pub id: String,
    /// Agent ID being compiled.
    pub agent_id: String,
    /// Agent version string.
    pub version: String,
    /// When the compilation was attempted.
    pub timestamp: DateTime<Utc>,
    /// Per-stage verdicts.
    pub stages: Vec<StageVerdict>,
    /// Overall compilation outcome.
    pub overall: CompileOutcome,
    /// Total compilation time in milliseconds.
    pub total_duration_ms: u64,
}

/// Verdict for a single compilation stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageVerdict {
    /// Stage number (1-8).
    pub stage: u8,
    /// Human-readable stage name.
    pub name: String,
    /// Whether the stage passed, failed, or was skipped.
    pub outcome: CompileOutcome,
    /// Duration of this stage in milliseconds.
    pub duration_ms: u64,
    /// Diagnostic messages produced during this stage.
    pub diagnostics: Vec<Diagnostic>,
}

/// Outcome of a compilation stage or the overall pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompileOutcome {
    Pass,
    Fail,
    Skip,
}

/// A diagnostic message (warning or error) from a pipeline stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Severity level.
    pub level: DiagnosticLevel,
    /// Human-readable message.
    pub message: String,
    /// Optional section reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
}

/// Diagnostic severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
}
