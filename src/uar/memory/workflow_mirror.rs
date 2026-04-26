//! Workflow-state mirror helpers for KBD/OpenSpec records.
//!
//! `.kbd-orchestrator/` remains the authoritative workflow ledger. This module
//! prepares deterministic Surreal Memory records that can be written through the
//! existing memory service or `/mcp/memory` endpoint and queried later as a
//! secondary recovery/audit source.

use std::{cmp::Ordering, path::Path};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use surreal_memory::{Memory, MemoryScope, MemoryType};

/// Metadata category used to find workflow mirror records in memory.
pub const WORKFLOW_MIRROR_CATEGORY: &str = "workflow_mirror";

/// Workflow entity kinds that can be mirrored into Surreal Memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowKind {
    /// Project-level workflow metadata.
    Project,
    /// KBD phase metadata.
    Phase,
    /// OpenSpec change metadata.
    OpenspecChange,
    /// A trackable workflow task.
    Task,
    /// The current tool handoff waypoint.
    Waypoint,
    /// Assessment artifact state.
    Assessment,
    /// Plan artifact state.
    Plan,
    /// Blocking condition state.
    Blocker,
    /// Verification-result state.
    VerificationResult,
}

impl WorkflowKind {
    /// Return the serialized metadata value for this workflow kind.
    ///
    /// # Examples
    ///
    /// ```
    /// use universal_agent_runtime::uar::memory::workflow_mirror::WorkflowKind;
    ///
    /// assert_eq!(WorkflowKind::OpenspecChange.as_str(), "openspec_change");
    /// ```
    ///
    /// # Errors
    ///
    /// This function does not return an error.
    ///
    /// # Panics
    ///
    /// This function does not panic.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Phase => "phase",
            Self::OpenspecChange => "openspec_change",
            Self::Task => "task",
            Self::Waypoint => "waypoint",
            Self::Assessment => "assessment",
            Self::Plan => "plan",
            Self::Blocker => "blocker",
            Self::VerificationResult => "verification_result",
        }
    }

    /// Parse a workflow kind from memory metadata.
    ///
    /// # Examples
    ///
    /// ```
    /// use universal_agent_runtime::uar::memory::workflow_mirror::WorkflowKind;
    ///
    /// assert_eq!(WorkflowKind::parse("task").unwrap(), WorkflowKind::Task);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when the kind is not supported by the workflow mirror
    /// contract.
    ///
    /// # Panics
    ///
    /// This function does not panic.
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "project" => Ok(Self::Project),
            "phase" => Ok(Self::Phase),
            "openspec_change" => Ok(Self::OpenspecChange),
            "task" => Ok(Self::Task),
            "waypoint" => Ok(Self::Waypoint),
            "assessment" => Ok(Self::Assessment),
            "plan" => Ok(Self::Plan),
            "blocker" => Ok(Self::Blocker),
            "verification_result" => Ok(Self::VerificationResult),
            other => bail!("unsupported workflow_kind '{other}'"),
        }
    }
}

/// Memory scope selected for a mirrored workflow record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowMirrorScope {
    /// Project-level workflow state shared across tools.
    Global,
    /// Phase/change-scoped workflow state.
    Task,
}

impl WorkflowMirrorScope {
    fn as_str(self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Task => "task",
        }
    }

    fn memory_scope(self) -> MemoryScope {
        match self {
            Self::Global => MemoryScope::Global,
            Self::Task => MemoryScope::Task,
        }
    }
}

/// KBD/OpenSpec workflow state prepared for Surreal Memory mirroring.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowMirrorRecord {
    /// Type of workflow entity being mirrored.
    pub workflow_kind: WorkflowKind,
    /// Stable identifier for the workflow entity.
    pub workflow_id: String,
    /// KBD phase identifier, when available.
    pub phase: Option<String>,
    /// OpenSpec change identifier, when available.
    pub change: Option<String>,
    /// Tool that produced the mirrored write.
    pub source_tool: String,
    /// Source workflow timestamp used for conflict resolution.
    pub updated_at: DateTime<Utc>,
    /// KBD/OpenSpec path that produced the record.
    pub source_path: String,
    /// Human-readable, sanitized summary of the workflow state.
    pub summary: String,
    /// Memory scope for the mirror record.
    pub scope: WorkflowMirrorScope,
}

impl WorkflowMirrorRecord {
    /// Create a validated workflow mirror record with deterministic defaults.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::Utc;
    /// use universal_agent_runtime::uar::memory::workflow_mirror::{
    ///     WorkflowKind, WorkflowMirrorRecord,
    /// };
    ///
    /// let record = WorkflowMirrorRecord::new(
    ///     WorkflowKind::Task,
    ///     "runtime-console-validation-hardening/surreal-memory",
    ///     "codex",
    ///     Utc::now(),
    ///     ".kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json",
    ///     "mirror task is in progress",
    /// )
    /// .unwrap();
    /// assert_eq!(record.workflow_kind, WorkflowKind::Task);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when required fields are empty, the source path points
    /// at secret-bearing files, or phase/change scoping is inconsistent.
    ///
    /// # Panics
    ///
    /// This function does not panic.
    pub fn new(
        workflow_kind: WorkflowKind,
        workflow_id: impl Into<String>,
        source_tool: impl Into<String>,
        updated_at: DateTime<Utc>,
        source_path: impl Into<String>,
        summary: impl Into<String>,
    ) -> Result<Self> {
        let mut record = Self {
            workflow_kind,
            workflow_id: workflow_id.into(),
            phase: None,
            change: None,
            source_tool: source_tool.into(),
            updated_at,
            source_path: source_path.into(),
            summary: sanitize_summary(&summary.into()),
            scope: WorkflowMirrorScope::Global,
        };
        record.infer_scope();
        record.validate()?;
        Ok(record)
    }

    /// Attach KBD/OpenSpec routing metadata and revalidate the record.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::Utc;
    /// use universal_agent_runtime::uar::memory::workflow_mirror::{
    ///     WorkflowKind, WorkflowMirrorRecord, WorkflowMirrorScope,
    /// };
    ///
    /// let record = WorkflowMirrorRecord::new(
    ///     WorkflowKind::OpenspecChange,
    ///     "runtime-console-validation-hardening/surreal-memory-workflow-mirror-tests",
    ///     "codex",
    ///     Utc::now(),
    ///     "openspec/changes/surreal-memory-workflow-mirror-tests/tasks.md",
    ///     "ready to apply",
    /// )
    /// .unwrap()
    /// .with_routing(Some("runtime-console-validation-hardening"), Some("surreal-memory-workflow-mirror-tests"))
    /// .unwrap();
    /// assert_eq!(record.scope, WorkflowMirrorScope::Task);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the resulting record violates the mirror contract.
    ///
    /// # Panics
    ///
    /// This function does not panic.
    pub fn with_routing(
        mut self,
        phase: Option<impl Into<String>>,
        change: Option<impl Into<String>>,
    ) -> Result<Self> {
        self.phase = phase.map(Into::into);
        self.change = change.map(Into::into);
        self.infer_scope();
        self.validate()?;
        Ok(self)
    }

    /// Return an updated mirror record with new audit metadata and summary.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::{TimeZone, Utc};
    /// use universal_agent_runtime::uar::memory::workflow_mirror::{
    ///     WorkflowKind, WorkflowMirrorRecord,
    /// };
    ///
    /// let record = WorkflowMirrorRecord::new(
    ///     WorkflowKind::Task,
    ///     "phase/task",
    ///     "codex",
    ///     Utc.with_ymd_and_hms(2026, 4, 26, 8, 0, 0).unwrap(),
    ///     ".kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json",
    ///     "old state",
    /// )
    /// .unwrap()
    /// .updated(
    ///     "cursor",
    ///     Utc.with_ymd_and_hms(2026, 4, 26, 9, 0, 0).unwrap(),
    ///     "new state",
    /// )
    /// .unwrap();
    /// assert_eq!(record.source_tool, "cursor");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the updated record violates the mirror contract.
    ///
    /// # Panics
    ///
    /// This function does not panic.
    pub fn updated(
        mut self,
        source_tool: impl Into<String>,
        updated_at: DateTime<Utc>,
        summary: impl Into<String>,
    ) -> Result<Self> {
        self.source_tool = source_tool.into();
        self.updated_at = updated_at;
        self.summary = sanitize_summary(&summary.into());
        self.validate()?;
        Ok(self)
    }

    /// Validate the workflow mirror contract for this record.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::Utc;
    /// use universal_agent_runtime::uar::memory::workflow_mirror::{
    ///     WorkflowKind, WorkflowMirrorRecord,
    /// };
    ///
    /// let record = WorkflowMirrorRecord::new(
    ///     WorkflowKind::Waypoint,
    ///     "current-waypoint",
    ///     "codex",
    ///     Utc::now(),
    ///     ".kbd-orchestrator/current-waypoint.json",
    ///     "handoff points at apply",
    /// )
    /// .unwrap();
    /// record.validate().unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when the record is missing required audit metadata or
    /// references a raw secret-bearing source path.
    ///
    /// # Panics
    ///
    /// This function does not panic.
    pub fn validate(&self) -> Result<()> {
        if self.workflow_id.trim().is_empty() {
            bail!("workflow_id is required");
        }
        if self.source_tool.trim().is_empty() {
            bail!("source_tool is required");
        }
        if self.source_path.trim().is_empty() {
            bail!("source_path is required");
        }
        if is_secret_source_path(&self.source_path) {
            bail!("source_path points at a secret-bearing file");
        }
        if self.summary.trim().is_empty() {
            bail!("summary is required");
        }
        if self.scope == WorkflowMirrorScope::Task && self.phase.is_none() && self.change.is_none()
        {
            bail!("task-scoped workflow mirror records require phase or change metadata");
        }
        Ok(())
    }

    /// Return deterministic memory content for this workflow state.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::Utc;
    /// use universal_agent_runtime::uar::memory::workflow_mirror::{
    ///     WorkflowKind, WorkflowMirrorRecord,
    /// };
    ///
    /// let record = WorkflowMirrorRecord::new(
    ///     WorkflowKind::Project,
    ///     "universal-agent-runtime",
    ///     "codex",
    ///     Utc::now(),
    ///     ".kbd-orchestrator/project.json",
    ///     "project workflow metadata",
    /// )
    /// .unwrap();
    /// assert!(record.content().contains("workflow_kind: project"));
    /// ```
    ///
    /// # Errors
    ///
    /// This function does not return an error.
    ///
    /// # Panics
    ///
    /// This function does not panic.
    pub fn content(&self) -> String {
        [
            format!("workflow_kind: {}", self.workflow_kind.as_str()),
            format!("workflow_id: {}", self.workflow_id),
            format!("phase: {}", self.phase.as_deref().unwrap_or("")),
            format!("change: {}", self.change.as_deref().unwrap_or("")),
            format!("source_tool: {}", self.source_tool),
            format!("updated_at: {}", self.updated_at.to_rfc3339()),
            format!("source_path: {}", self.source_path),
            format!("summary: {}", self.summary),
        ]
        .join("\n")
    }

    /// Return structured metadata for a Surreal Memory record.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::Utc;
    /// use universal_agent_runtime::uar::memory::workflow_mirror::{
    ///     WorkflowKind, WorkflowMirrorRecord,
    /// };
    ///
    /// let record = WorkflowMirrorRecord::new(
    ///     WorkflowKind::Project,
    ///     "universal-agent-runtime",
    ///     "codex",
    ///     Utc::now(),
    ///     ".kbd-orchestrator/project.json",
    ///     "project workflow metadata",
    /// )
    /// .unwrap();
    /// assert_eq!(record.metadata()["workflow_kind"], "project");
    /// ```
    ///
    /// # Errors
    ///
    /// This function does not return an error.
    ///
    /// # Panics
    ///
    /// This function does not panic.
    pub fn metadata(&self) -> Value {
        json!({
            "mirror": WORKFLOW_MIRROR_CATEGORY,
            "workflow_kind": self.workflow_kind.as_str(),
            "workflow_id": self.workflow_id,
            "phase": self.phase,
            "change": self.change,
            "source_tool": self.source_tool,
            "updated_at": self.updated_at.to_rfc3339(),
            "source_path": self.source_path,
            "scope": self.scope.as_str(),
        })
    }

    /// Build a memory-service write payload for this workflow record.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::Utc;
    /// use universal_agent_runtime::uar::memory::workflow_mirror::{
    ///     WorkflowKind, WorkflowMirrorRecord, WORKFLOW_MIRROR_CATEGORY,
    /// };
    ///
    /// let write = WorkflowMirrorRecord::new(
    ///     WorkflowKind::Project,
    ///     "universal-agent-runtime",
    ///     "codex",
    ///     Utc::now(),
    ///     ".kbd-orchestrator/project.json",
    ///     "project workflow metadata",
    /// )
    /// .unwrap()
    /// .to_memory_write();
    /// assert!(write.categories.contains(&WORKFLOW_MIRROR_CATEGORY.to_string()));
    /// ```
    ///
    /// # Errors
    ///
    /// This function does not return an error.
    ///
    /// # Panics
    ///
    /// This function does not panic.
    pub fn to_memory_write(&self) -> WorkflowMirrorMemoryWrite {
        WorkflowMirrorMemoryWrite {
            content: self.content(),
            scope: self.scope.memory_scope(),
            memory_type: MemoryType::Semantic,
            categories: vec![
                WORKFLOW_MIRROR_CATEGORY.to_string(),
                self.workflow_kind.as_str().to_string(),
            ],
            metadata: self.metadata(),
            importance: 0.65,
        }
    }

    fn infer_scope(&mut self) {
        self.scope = if self.phase.is_some() || self.change.is_some() {
            WorkflowMirrorScope::Task
        } else {
            WorkflowMirrorScope::Global
        };
    }
}

/// Write payload accepted by `MemoryService::add` or the `memory_add` MCP tool.
#[derive(Debug, Clone)]
pub struct WorkflowMirrorMemoryWrite {
    /// Deterministic memory content.
    pub content: String,
    /// Memory scope selected for the workflow record.
    pub scope: MemoryScope,
    /// Memory type selected for the workflow record.
    pub memory_type: MemoryType,
    /// Taxonomy tags for memory lookup.
    pub categories: Vec<String>,
    /// Structured workflow metadata.
    pub metadata: Value,
    /// Relative memory importance.
    pub importance: f32,
}

/// Workflow mirror candidate recovered from Surreal Memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowMirrorCandidate {
    /// Optional Surreal Memory record id.
    pub memory_id: Option<String>,
    /// Mirrored workflow record.
    pub record: WorkflowMirrorRecord,
}

/// Result of newest-`updated_at` recovery selection.
#[derive(Debug, Clone)]
pub struct WorkflowMirrorSelection {
    /// Selected winning candidate.
    pub selected: WorkflowMirrorCandidate,
    /// All candidates evaluated, including older audit records.
    pub candidates: Vec<WorkflowMirrorCandidate>,
}

/// Convert an existing memory into a workflow mirror candidate when its metadata
/// matches the mirror contract.
///
/// # Examples
///
/// ```ignore
/// let candidate = workflow_candidate_from_memory(&memory)?;
/// ```
///
/// # Errors
///
/// Returns an error when required workflow metadata is missing or invalid.
///
/// # Panics
///
/// This function does not panic.
pub fn workflow_candidate_from_memory(memory: &Memory) -> Result<WorkflowMirrorCandidate> {
    let metadata = memory
        .metadata
        .as_ref()
        .context("memory has no workflow mirror metadata")?;
    if metadata.get("mirror").and_then(Value::as_str) != Some(WORKFLOW_MIRROR_CATEGORY) {
        bail!("memory is not a workflow mirror record");
    }

    let workflow_kind = WorkflowKind::parse(required_str(metadata, "workflow_kind")?)?;
    let updated_at = DateTime::parse_from_rfc3339(required_str(metadata, "updated_at")?)
        .context("invalid workflow mirror updated_at")?
        .with_timezone(&Utc);
    let scope = match metadata.get("scope").and_then(Value::as_str) {
        Some("task") => WorkflowMirrorScope::Task,
        _ => WorkflowMirrorScope::Global,
    };
    let mut record = WorkflowMirrorRecord {
        workflow_kind,
        workflow_id: required_str(metadata, "workflow_id")?.to_string(),
        phase: optional_str(metadata, "phase"),
        change: optional_str(metadata, "change"),
        source_tool: required_str(metadata, "source_tool")?.to_string(),
        updated_at,
        source_path: required_str(metadata, "source_path")?.to_string(),
        summary: sanitize_summary(summary_from_content(&memory.content)),
        scope,
    };
    record.infer_scope();
    record.validate()?;

    Ok(WorkflowMirrorCandidate {
        memory_id: memory.id.as_ref().map(|id| format!("{id:?}")),
        record,
    })
}

/// Select the newest recovery candidate for a workflow identity while retaining
/// all candidates for audit.
///
/// # Examples
///
/// ```
/// use chrono::{TimeZone, Utc};
/// use universal_agent_runtime::uar::memory::workflow_mirror::{
///     WorkflowKind, WorkflowMirrorCandidate, WorkflowMirrorRecord, select_recovery_candidate,
/// };
///
/// let old = WorkflowMirrorCandidate {
///     memory_id: Some("memory:old".into()),
///     record: WorkflowMirrorRecord::new(
///         WorkflowKind::Task,
///         "phase/task",
///         "codex",
///         Utc.with_ymd_and_hms(2026, 4, 26, 7, 0, 0).unwrap(),
///         ".kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json",
///         "old state",
///     )
///     .unwrap()
///     .with_routing(Some("runtime-console-validation-hardening"), Some("surreal-memory-workflow-mirror-tests"))
///     .unwrap(),
/// };
/// let new = WorkflowMirrorCandidate {
///     memory_id: Some("memory:new".into()),
///     record: WorkflowMirrorRecord::new(
///         WorkflowKind::Task,
///         "phase/task",
///         "claude-code",
///         Utc.with_ymd_and_hms(2026, 4, 26, 8, 0, 0).unwrap(),
///         ".kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json",
///         "new state",
///     )
///     .unwrap()
///     .with_routing(Some("runtime-console-validation-hardening"), Some("surreal-memory-workflow-mirror-tests"))
///     .unwrap(),
/// };
/// let selection = select_recovery_candidate(vec![old, new]).unwrap();
/// assert_eq!(selection.selected.record.source_tool, "claude-code");
/// assert_eq!(selection.candidates.len(), 2);
/// ```
///
/// # Errors
///
/// Returns an error when candidates for different workflow identities are mixed.
///
/// # Panics
///
/// This function does not panic.
pub fn select_recovery_candidate(
    mut candidates: Vec<WorkflowMirrorCandidate>,
) -> Result<Option<WorkflowMirrorSelection>> {
    if candidates.is_empty() {
        return Ok(None);
    }

    let first_kind = candidates[0].record.workflow_kind;
    let first_id = candidates[0].record.workflow_id.clone();
    if candidates.iter().any(|candidate| {
        candidate.record.workflow_kind != first_kind || candidate.record.workflow_id != first_id
    }) {
        bail!("workflow mirror candidates must share workflow_kind and workflow_id");
    }

    candidates.sort_by(compare_candidates);
    let selected = candidates
        .last()
        .cloned()
        .context("workflow mirror candidates unexpectedly empty")?;
    Ok(Some(WorkflowMirrorSelection {
        selected,
        candidates,
    }))
}

/// Build deterministic workflow mirror fixture records for all supported tools.
///
/// # Examples
///
/// ```
/// use universal_agent_runtime::uar::memory::workflow_mirror::cross_tool_fixture_records;
///
/// assert_eq!(cross_tool_fixture_records().unwrap().len(), 4);
/// ```
///
/// # Errors
///
/// Returns an error if the static fixture data violates the mirror contract.
///
/// # Panics
///
/// This function does not panic.
pub fn cross_tool_fixture_records() -> Result<Vec<WorkflowMirrorRecord>> {
    let phase = "runtime-console-validation-hardening";
    let change = "surreal-memory-workflow-mirror-tests";
    let source_path = ".kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json";
    let tools = ["codex", "claude-code", "cursor", "opencode"];

    tools
        .iter()
        .enumerate()
        .map(|(index, tool)| {
            let updated_at = DateTime::parse_from_rfc3339(&format!("2026-04-26T08:0{index}:00Z"))
                .context("fixture timestamp must parse")?
                .with_timezone(&Utc);
            WorkflowMirrorRecord::new(
                WorkflowKind::Task,
                format!("{phase}/{change}/{tool}"),
                *tool,
                updated_at,
                source_path,
                format!("{tool} workflow mirror fixture for {change}"),
            )?
            .with_routing(Some(phase), Some(change))
        })
        .collect()
}

/// Build a workflow mirror record from the repository KBD/OpenSpec paths.
///
/// This adapter reads only explicit workflow metadata and summaries; it does not
/// copy raw file bodies into memory and never mutates workflow files.
///
/// # Examples
///
/// ```ignore
/// let records = workflow_records_from_repo(
///     std::path::Path::new("."),
///     "runtime-console-validation-hardening",
///     "surreal-memory-workflow-mirror-tests",
///     "codex",
/// )?;
/// ```
///
/// # Errors
///
/// Returns an error when the project or waypoint metadata files cannot be read,
/// timestamps are invalid, or generated records violate the mirror contract.
///
/// # Panics
///
/// This function does not panic.
pub fn workflow_records_from_repo(
    root: &Path,
    phase: &str,
    change: &str,
    source_tool: &str,
) -> Result<Vec<WorkflowMirrorRecord>> {
    let project_path = root.join(".kbd-orchestrator/project.json");
    let waypoint_path = root.join(".kbd-orchestrator/current-waypoint.json");

    let project: Value = serde_json::from_str(
        &std::fs::read_to_string(&project_path).context("read .kbd-orchestrator/project.json")?,
    )
    .context("parse .kbd-orchestrator/project.json")?;
    let waypoint: Value = serde_json::from_str(
        &std::fs::read_to_string(&waypoint_path)
            .context("read .kbd-orchestrator/current-waypoint.json")?,
    )
    .context("parse .kbd-orchestrator/current-waypoint.json")?;

    let project_updated_at = parse_workflow_timestamp(&project, "updatedAt")?;
    let waypoint_updated_at = parse_workflow_timestamp(&waypoint, "updatedAt")?;
    let project_name = project
        .get("project")
        .and_then(Value::as_str)
        .unwrap_or("universal-agent-runtime");
    let waypoint_status = waypoint
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");

    let project_record = WorkflowMirrorRecord::new(
        WorkflowKind::Project,
        project_name,
        source_tool,
        project_updated_at,
        ".kbd-orchestrator/project.json",
        format!("project workflow metadata for {project_name}"),
    )?;

    let waypoint_record = WorkflowMirrorRecord::new(
        WorkflowKind::Waypoint,
        "current-waypoint",
        source_tool,
        waypoint_updated_at,
        ".kbd-orchestrator/current-waypoint.json",
        format!("current waypoint status is {waypoint_status} for {change}"),
    )?
    .with_routing(Some(phase), Some(change))?;

    let change_record = WorkflowMirrorRecord::new(
        WorkflowKind::OpenspecChange,
        format!("{phase}/{change}"),
        source_tool,
        waypoint_updated_at,
        format!("openspec/changes/{change}/tasks.md"),
        format!("OpenSpec change {change} is ready for workflow mirror validation"),
    )?
    .with_routing(Some(phase), Some(change))?;

    Ok(vec![project_record, waypoint_record, change_record])
}

fn compare_candidates(a: &WorkflowMirrorCandidate, b: &WorkflowMirrorCandidate) -> Ordering {
    a.record
        .updated_at
        .cmp(&b.record.updated_at)
        .then_with(|| a.memory_id.cmp(&b.memory_id))
}

fn required_str<'a>(metadata: &'a Value, field: &str) -> Result<&'a str> {
    metadata
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .with_context(|| format!("missing workflow mirror metadata field '{field}'"))
}

fn optional_str(metadata: &Value, field: &str) -> Option<String> {
    metadata
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
}

fn parse_workflow_timestamp(metadata: &Value, field: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(required_str(metadata, field)?)
        .with_context(|| format!("invalid workflow timestamp field '{field}'"))
        .map(|timestamp| timestamp.with_timezone(&Utc))
}

fn summary_from_content(content: &str) -> &str {
    content
        .lines()
        .find_map(|line| line.strip_prefix("summary: "))
        .unwrap_or(content)
}

fn sanitize_summary(summary: &str) -> String {
    summary
        .lines()
        .filter(|line| !looks_secret_bearing(line))
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn looks_secret_bearing(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("authorization:")
        || lower.contains("bearer ")
        || lower.contains("password")
        || lower.contains("secret")
        || lower.contains("token")
        || lower.contains("sk-")
        || lower.contains("fw_")
}

fn is_secret_source_path(path: &str) -> bool {
    let normalized = path.replace('\\', "/").to_ascii_lowercase();
    normalized.ends_with(".env")
        || normalized.contains("/.env")
        || normalized.contains("secret")
        || normalized.contains("credential")
        || normalized.contains("token")
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn record(tool: &str, hour: u32) -> WorkflowMirrorRecord {
        WorkflowMirrorRecord::new(
            WorkflowKind::Task,
            "runtime-console-validation-hardening/surreal-memory-workflow-mirror-tests",
            tool,
            Utc.with_ymd_and_hms(2026, 4, 26, hour, 0, 0).unwrap(),
            ".kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json",
            format!("{tool} updated the workflow state"),
        )
        .unwrap()
        .with_routing(
            Some("runtime-console-validation-hardening"),
            Some("surreal-memory-workflow-mirror-tests"),
        )
        .unwrap()
    }

    #[test]
    fn creates_deterministic_memory_write_metadata() {
        let record = record("codex", 8);
        let write = record.to_memory_write();

        assert_eq!(write.scope, MemoryScope::Task);
        assert_eq!(write.memory_type, MemoryType::Semantic);
        assert!(
            write
                .categories
                .contains(&WORKFLOW_MIRROR_CATEGORY.to_string())
        );
        assert_eq!(write.metadata["workflow_kind"], "task");
        assert_eq!(write.metadata["workflow_id"], record.workflow_id);
        assert_eq!(write.metadata["source_tool"], "codex");
        assert!(write.content.contains("workflow_kind: task"));
        assert!(
            write
                .content
                .contains("summary: codex updated the workflow state")
        );
    }

    #[test]
    fn validates_supported_workflow_kinds_and_required_metadata() {
        for kind in [
            WorkflowKind::Project,
            WorkflowKind::Phase,
            WorkflowKind::OpenspecChange,
            WorkflowKind::Task,
            WorkflowKind::Waypoint,
            WorkflowKind::Assessment,
            WorkflowKind::Plan,
            WorkflowKind::Blocker,
            WorkflowKind::VerificationResult,
        ] {
            let record = WorkflowMirrorRecord::new(
                kind,
                format!("id-{}", kind.as_str()),
                "codex",
                Utc.with_ymd_and_hms(2026, 4, 26, 8, 0, 0).unwrap(),
                ".kbd-orchestrator/project.json",
                format!("{} workflow state", kind.as_str()),
            )
            .unwrap();
            assert_eq!(WorkflowKind::parse(kind.as_str()).unwrap(), kind);
            assert!(record.validate().is_ok());
        }

        assert!(WorkflowKind::parse("unsupported").is_err());
    }

    #[test]
    fn redacts_secret_like_summary_lines_and_rejects_secret_paths() {
        let record = WorkflowMirrorRecord::new(
            WorkflowKind::Project,
            "universal-agent-runtime",
            "codex",
            Utc.with_ymd_and_hms(2026, 4, 26, 8, 0, 0).unwrap(),
            ".kbd-orchestrator/project.json",
            "safe summary\nOPENAI_API_KEY=sk-redacted",
        )
        .unwrap();

        assert_eq!(record.summary, "safe summary");
        assert!(
            WorkflowMirrorRecord::new(
                WorkflowKind::Project,
                "universal-agent-runtime",
                "codex",
                Utc.with_ymd_and_hms(2026, 4, 26, 8, 0, 0).unwrap(),
                ".env",
                "unsafe",
            )
            .is_err()
        );
    }

    #[test]
    fn selects_newest_candidate_and_preserves_source_tool_and_audit_candidates() {
        let old = WorkflowMirrorCandidate {
            memory_id: Some("memory:old".into()),
            record: record("codex", 8),
        };
        let new = WorkflowMirrorCandidate {
            memory_id: Some("memory:new".into()),
            record: record("claude-code", 9),
        };

        let selection = select_recovery_candidate(vec![old, new]).unwrap().unwrap();

        assert_eq!(selection.selected.memory_id.as_deref(), Some("memory:new"));
        assert_eq!(selection.selected.record.source_tool, "claude-code");
        assert_eq!(selection.candidates.len(), 2);
        assert_eq!(
            selection.candidates[0].memory_id.as_deref(),
            Some("memory:old")
        );
    }

    #[test]
    fn rejects_mixed_workflow_identity_candidates() {
        let mut other = record("cursor", 10);
        other.workflow_id = "other".into();

        assert!(
            select_recovery_candidate(vec![
                WorkflowMirrorCandidate {
                    memory_id: Some("memory:one".into()),
                    record: record("codex", 8),
                },
                WorkflowMirrorCandidate {
                    memory_id: Some("memory:two".into()),
                    record: other,
                },
            ])
            .is_err()
        );
    }

    #[test]
    fn builds_cross_tool_fixture_records_without_provider_dependencies() {
        let records = cross_tool_fixture_records().unwrap();
        let source_tools = records
            .iter()
            .map(|record| record.source_tool.as_str())
            .collect::<Vec<_>>();

        assert_eq!(source_tools, ["codex", "claude-code", "cursor", "opencode"]);
        assert!(
            records
                .iter()
                .all(|record| record.scope == WorkflowMirrorScope::Task)
        );
        assert!(
            records
                .iter()
                .all(|record| record.to_memory_write().metadata["mirror"]
                    == WORKFLOW_MIRROR_CATEGORY)
        );
    }

    #[test]
    fn updates_mirrored_workflow_content_and_metadata() {
        let updated = record("codex", 8)
            .updated(
                "opencode",
                Utc.with_ymd_and_hms(2026, 4, 26, 11, 0, 0).unwrap(),
                "opencode verified the mirror",
            )
            .unwrap();
        let write = updated.to_memory_write();

        assert_eq!(write.metadata["source_tool"], "opencode");
        assert_eq!(write.metadata["updated_at"], "2026-04-26T11:00:00+00:00");
        assert!(write.content.contains("source_tool: opencode"));
        assert!(
            write
                .content
                .contains("summary: opencode verified the mirror")
        );
    }

    #[test]
    fn repo_adapter_reads_workflow_metadata_without_mutating_files() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let kbd = root.join(".kbd-orchestrator");
        std::fs::create_dir_all(&kbd).unwrap();
        std::fs::write(
            kbd.join("project.json"),
            r#"{
              "project": "universal-agent-runtime",
              "sourceOfTruth": ".kbd-orchestrator",
              "updatedAt": "2026-04-26T00:00:00-05:00"
            }"#,
        )
        .unwrap();
        std::fs::write(
            kbd.join("current-waypoint.json"),
            r#"{
              "phase": "runtime-console-validation-hardening",
              "change": "surreal-memory-workflow-mirror-tests",
              "status": "ready_to_apply",
              "updatedAt": "2026-04-26T03:59:27-05:00"
            }"#,
        )
        .unwrap();

        let before = std::fs::read_to_string(kbd.join("current-waypoint.json")).unwrap();
        let records = workflow_records_from_repo(
            root,
            "runtime-console-validation-hardening",
            "surreal-memory-workflow-mirror-tests",
            "codex",
        )
        .unwrap();
        let after = std::fs::read_to_string(kbd.join("current-waypoint.json")).unwrap();

        assert_eq!(before, after);
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].workflow_kind, WorkflowKind::Project);
        assert_eq!(records[1].workflow_kind, WorkflowKind::Waypoint);
        assert_eq!(records[2].workflow_kind, WorkflowKind::OpenspecChange);
        assert!(records[1].metadata()["change"] == "surreal-memory-workflow-mirror-tests");
    }
}
