//! Shared validation for the three host-owned agent-thread stores.
//! Storage revisions cover every status write; history revisions do not.

use std::fmt::Write as _;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::uar::runtime::thread::{AgentEdge, AgentThread, AgentThreadStatus, ThreadRecordError};

/// Maximum retained payload for one canonical tool receipt.
pub const MAX_CANONICAL_RECEIPT_BYTES: u64 = 16 * 1024 * 1024;
/// Maximum retained canonical tool payload across one run.
pub const MAX_CANONICAL_RUN_BYTES: u64 = 64 * 1024 * 1024;

/// Trusted-host execution path that produced a canonical receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanonicalReceiptSource {
    Native,
    Mcp,
    Graph,
    Terminal,
    Sandbox,
}

/// Why a canonical payload is unavailable. Missing bytes are never fabricated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CanonicalReceiptCompleteness {
    Complete,
    AcquisitionLimitExceeded { observed_bytes: u64 },
    AcquisitionIncomplete { observed_bytes: u64 },
    RunStorageLimitExceeded { retained_before: u64 },
    LegacyUnknown,
}

/// One named byte stream retained exactly as supplied by the trusted host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalRawSegment {
    pub name: String,
    pub bytes_base64: Option<String>,
    pub byte_len: u64,
    pub sha256: String,
}

impl CanonicalRawSegment {
    #[must_use]
    pub fn new(name: impl Into<String>, bytes: &[u8]) -> Self {
        Self {
            name: name.into(),
            bytes_base64: Some(BASE64.encode(bytes)),
            byte_len: bytes.len() as u64,
            sha256: sha256(bytes),
        }
    }

    pub fn from_acquisition(
        name: impl Into<String>,
        bytes: Option<Vec<u8>>,
        byte_len: u64,
        sha256: String,
    ) -> Result<Self, CanonicalReceiptStoreError> {
        let segment = Self {
            name: name.into(),
            bytes_base64: bytes.as_deref().map(|bytes| BASE64.encode(bytes)),
            byte_len,
            sha256,
        };
        if segment.name.is_empty()
            || !valid_sha256(&segment.sha256)
            || segment
                .verified_bytes()?
                .is_some_and(|bytes| bytes.len() as u64 != byte_len)
        {
            return Err(CanonicalReceiptStoreError::InvalidRecord);
        }
        Ok(segment)
    }

    /// Decode the retained bytes and verify their identity.
    pub fn verified_bytes(&self) -> Result<Option<Vec<u8>>, CanonicalReceiptStoreError> {
        let Some(encoded) = &self.bytes_base64 else {
            return Ok(None);
        };
        let bytes = BASE64
            .decode(encoded)
            .map_err(|_| CanonicalReceiptStoreError::InvalidRecord)?;
        if bytes.len() as u64 != self.byte_len || sha256(&bytes) != self.sha256 {
            return Err(CanonicalReceiptStoreError::InvalidRecord);
        }
        Ok(Some(bytes))
    }

    fn discard_bytes(&mut self) {
        self.bytes_base64 = None;
    }
}

/// Immutable pre-format tool result retained by the trusted host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalToolReceipt {
    pub schema_version: u32,
    pub owner_id: String,
    pub run_id: String,
    pub sequence: u64,
    pub call_id: String,
    pub tool: String,
    pub source: CanonicalReceiptSource,
    pub typed_value: Option<serde_json::Value>,
    pub typed_value_bytes: u64,
    pub typed_value_sha256: String,
    pub raw_segments: Vec<CanonicalRawSegment>,
    pub retained_bytes: u64,
    pub completeness: CanonicalReceiptCompleteness,
}

impl CanonicalToolReceipt {
    /// Acquire one result before any display formatting. Oversized values retain
    /// identity metadata and an explicit incomplete status, never a fake suffix.
    pub fn acquire(
        owner_id: impl Into<String>,
        run_id: impl Into<String>,
        sequence: u64,
        call_id: impl Into<String>,
        tool: impl Into<String>,
        source: CanonicalReceiptSource,
        typed_value: serde_json::Value,
        raw_segments: impl IntoIterator<Item = (String, Vec<u8>)>,
        acquisition_complete: bool,
        observed_bytes: u64,
    ) -> Result<Self, CanonicalReceiptStoreError> {
        let raw_segments = raw_segments
            .into_iter()
            .map(|(name, bytes)| CanonicalRawSegment::new(name, &bytes))
            .collect::<Vec<_>>();
        Self::acquire_with_segments(
            owner_id,
            run_id,
            sequence,
            call_id,
            tool,
            source,
            typed_value,
            raw_segments,
            acquisition_complete,
            observed_bytes,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn acquire_with_segments(
        owner_id: impl Into<String>,
        run_id: impl Into<String>,
        sequence: u64,
        call_id: impl Into<String>,
        tool: impl Into<String>,
        source: CanonicalReceiptSource,
        typed_value: serde_json::Value,
        raw_segments: Vec<CanonicalRawSegment>,
        acquisition_complete: bool,
        observed_bytes: u64,
    ) -> Result<Self, CanonicalReceiptStoreError> {
        let owner_id = owner_id.into();
        let run_id = run_id.into();
        let call_id = call_id.into();
        let tool = tool.into();
        if owner_id.is_empty() || run_id.is_empty() || call_id.is_empty() || tool.is_empty() {
            return Err(CanonicalReceiptStoreError::InvalidRecord);
        }
        let serialized = serde_json::to_vec(&typed_value)
            .map_err(|_| CanonicalReceiptStoreError::InvalidRecord)?;
        let typed_value_bytes = serialized.len() as u64;
        let typed_value_sha256 = sha256(&serialized);
        let raw_complete = raw_segments.iter().try_fold(true, |complete, segment| {
            Ok::<_, CanonicalReceiptStoreError>(complete && segment.verified_bytes()?.is_some())
        })?;
        let acquisition_complete = acquisition_complete && raw_complete;
        let retained_bytes = raw_segments
            .iter()
            .try_fold(typed_value_bytes, |total, segment| {
                total.checked_add(segment.byte_len)
            });
        let can_retain = acquisition_complete
            && retained_bytes.is_some_and(|bytes| bytes <= MAX_CANONICAL_RECEIPT_BYTES);
        let observed_bytes = observed_bytes.max(retained_bytes.unwrap_or(u64::MAX));
        let completeness = if can_retain {
            CanonicalReceiptCompleteness::Complete
        } else if acquisition_complete {
            CanonicalReceiptCompleteness::AcquisitionLimitExceeded { observed_bytes }
        } else {
            CanonicalReceiptCompleteness::AcquisitionIncomplete { observed_bytes }
        };
        let mut raw_segments = raw_segments;
        if !can_retain {
            for segment in &mut raw_segments {
                segment.discard_bytes();
            }
        }
        Ok(Self {
            schema_version: 1,
            owner_id,
            run_id,
            sequence,
            call_id,
            tool,
            source,
            typed_value: can_retain.then_some(typed_value),
            typed_value_bytes,
            typed_value_sha256,
            raw_segments,
            retained_bytes: if can_retain {
                retained_bytes.unwrap_or_default()
            } else {
                0
            },
            completeness,
        })
    }

    pub fn validate(&self) -> Result<(), CanonicalReceiptStoreError> {
        if self.schema_version != 1
            || self.owner_id.is_empty()
            || self.run_id.is_empty()
            || self.call_id.is_empty()
            || self.tool.is_empty()
            || self.sequence > i64::MAX as u64
            || self.retained_bytes > MAX_CANONICAL_RECEIPT_BYTES
            || !valid_sha256(&self.typed_value_sha256)
            || self
                .raw_segments
                .iter()
                .any(|segment| segment.name.is_empty() || !valid_sha256(&segment.sha256))
        {
            return Err(CanonicalReceiptStoreError::InvalidRecord);
        }
        let payload_bytes =
            self.raw_segments
                .iter()
                .try_fold(self.typed_value_bytes, |total, segment| {
                    total
                        .checked_add(segment.byte_len)
                        .ok_or(CanonicalReceiptStoreError::InvalidRecord)
                })?;
        match (&self.completeness, &self.typed_value) {
            (CanonicalReceiptCompleteness::Complete, Some(value)) => {
                let serialized = serde_json::to_vec(value)
                    .map_err(|_| CanonicalReceiptStoreError::InvalidRecord)?;
                let retained = self.raw_segments.iter().try_fold(
                    serialized.len() as u64,
                    |total, segment| {
                        if segment.verified_bytes()?.is_none() {
                            return Err(CanonicalReceiptStoreError::InvalidRecord);
                        }
                        total
                            .checked_add(segment.byte_len)
                            .ok_or(CanonicalReceiptStoreError::InvalidRecord)
                    },
                )?;
                if serialized.len() as u64 != self.typed_value_bytes
                    || sha256(&serialized) != self.typed_value_sha256
                    || retained != self.retained_bytes
                    || retained != payload_bytes
                {
                    return Err(CanonicalReceiptStoreError::InvalidRecord);
                }
            }
            (CanonicalReceiptCompleteness::Complete, None) => {
                return Err(CanonicalReceiptStoreError::InvalidRecord);
            }
            (_, Some(_)) => {
                return Err(CanonicalReceiptStoreError::InvalidRecord);
            }
            (_, None) => {
                if self.retained_bytes != 0
                    || self
                        .raw_segments
                        .iter()
                        .any(|segment| segment.bytes_base64.is_some())
                {
                    return Err(CanonicalReceiptStoreError::InvalidRecord);
                }
            }
        }
        match self.completeness {
            CanonicalReceiptCompleteness::Complete => {}
            CanonicalReceiptCompleteness::AcquisitionLimitExceeded { observed_bytes }
                if observed_bytes > MAX_CANONICAL_RECEIPT_BYTES
                    && observed_bytes >= payload_bytes => {}
            CanonicalReceiptCompleteness::AcquisitionIncomplete { observed_bytes }
                if observed_bytes >= payload_bytes => {}
            CanonicalReceiptCompleteness::RunStorageLimitExceeded { retained_before }
                if retained_before <= MAX_CANONICAL_RUN_BYTES
                    && retained_before
                        .checked_add(payload_bytes)
                        .is_none_or(|total| total > MAX_CANONICAL_RUN_BYTES) => {}
            CanonicalReceiptCompleteness::LegacyUnknown => {}
            _ => return Err(CanonicalReceiptStoreError::InvalidRecord),
        }
        Ok(())
    }

    /// Apply the per-run storage ceiling. A receipt that cannot fit retains its
    /// identity and digest but no partial payload.
    pub fn admit_to_run(
        mut self,
        retained_before: u64,
    ) -> Result<Self, CanonicalReceiptStoreError> {
        self.validate()?;
        if retained_before > MAX_CANONICAL_RUN_BYTES {
            return Err(CanonicalReceiptStoreError::InvalidRecord);
        }
        if retained_before
            .checked_add(self.retained_bytes)
            .is_some_and(|total| total <= MAX_CANONICAL_RUN_BYTES)
        {
            return Ok(self);
        }
        self.typed_value = None;
        for segment in &mut self.raw_segments {
            segment.discard_bytes();
        }
        self.retained_bytes = 0;
        self.completeness =
            CanonicalReceiptCompleteness::RunStorageLimitExceeded { retained_before };
        Ok(self)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CanonicalReceiptStoreError {
    #[error("canonical tool receipt is invalid")]
    InvalidRecord,
    #[error("canonical tool receipt is outside the verified owner or run")]
    ScopeMismatch,
    #[error("canonical tool receipt identity conflicts with stored data")]
    Conflict,
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(71);
    encoded.push_str("sha256:");
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").expect("writing to a String cannot fail");
    }
    encoded
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn same_canonical_payload(left: &CanonicalToolReceipt, right: &CanonicalToolReceipt) -> bool {
    left.schema_version == right.schema_version
        && left.owner_id == right.owner_id
        && left.run_id == right.run_id
        && left.sequence == right.sequence
        && left.call_id == right.call_id
        && left.tool == right.tool
        && left.source == right.source
        && left.typed_value_bytes == right.typed_value_bytes
        && left.typed_value_sha256 == right.typed_value_sha256
        && left.raw_segments.len() == right.raw_segments.len()
        && left
            .raw_segments
            .iter()
            .zip(&right.raw_segments)
            .all(|(left_segment, right_segment)| {
                left_segment.name == right_segment.name
                    && left_segment.byte_len == right_segment.byte_len
                    && left_segment.sha256 == right_segment.sha256
            })
}

pub(crate) fn reconcile_canonical_receipt(
    stored: &CanonicalToolReceipt,
    receipt: &CanonicalToolReceipt,
) -> Result<CanonicalToolReceipt, CanonicalReceiptStoreError> {
    stored.validate()?;
    receipt.validate()?;
    if stored.owner_id != receipt.owner_id || stored.run_id != receipt.run_id {
        return Err(CanonicalReceiptStoreError::ScopeMismatch);
    }
    if stored == receipt
        || (matches!(
            stored.completeness,
            CanonicalReceiptCompleteness::RunStorageLimitExceeded { .. }
        ) && same_canonical_payload(stored, receipt))
    {
        Ok(stored.clone())
    } else {
        Err(CanonicalReceiptStoreError::Conflict)
    }
}

#[cfg(any(feature = "in-memory-backend", test))]
pub(crate) fn prepare_canonical_receipt(
    existing: &[CanonicalToolReceipt],
    receipt: &CanonicalToolReceipt,
) -> Result<CanonicalToolReceipt, CanonicalReceiptStoreError> {
    receipt.validate()?;
    if let Some(stored) = existing
        .iter()
        .find(|stored| stored.call_id == receipt.call_id)
    {
        return reconcile_canonical_receipt(stored, receipt);
    }
    if existing
        .iter()
        .any(|stored| stored.owner_id != receipt.owner_id || stored.run_id != receipt.run_id)
    {
        return Err(CanonicalReceiptStoreError::ScopeMismatch);
    }
    let retained_before = existing.iter().try_fold(0_u64, |total, stored| {
        stored.validate()?;
        total
            .checked_add(stored.retained_bytes)
            .ok_or(CanonicalReceiptStoreError::InvalidRecord)
    })?;
    receipt.clone().admit_to_run(retained_before)
}

pub(crate) fn ordered_canonical_receipts(
    mut receipts: Vec<CanonicalToolReceipt>,
    owner_id: &str,
    run_id: &str,
) -> Result<Vec<CanonicalToolReceipt>, CanonicalReceiptStoreError> {
    for receipt in &receipts {
        receipt.validate()?;
        if receipt.owner_id != owner_id || receipt.run_id != run_id {
            return Err(CanonicalReceiptStoreError::ScopeMismatch);
        }
    }
    receipts.sort_by(|left, right| {
        left.sequence
            .cmp(&right.sequence)
            .then_with(|| left.call_id.cmp(&right.call_id))
    });
    Ok(receipts)
}

/// A thread plus its optimistic-concurrency token. Revisions start at zero.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PersistedAgentThread {
    pub thread: AgentThread,
    pub revision: u64,
}

/// Portable store failures. Database transport/transaction errors remain errors
/// too, and never imply that an interrupted write did not commit.
#[derive(Debug, thiserror::Error)]
pub enum AgentThreadStoreError {
    #[error(transparent)]
    InvalidRecord(#[from] ThreadRecordError),
    #[error("agent thread is outside the verified owner or requested tree")]
    ScopeMismatch,
    #[error("agent thread or required parent/root does not exist")]
    NotFound,
    #[error("agent thread ID or canonical path already exists")]
    AlreadyExists,
    #[error("agent thread changed since the supplied storage revision")]
    Conflict,
    #[error("agent thread lineage, artifact, or creation time cannot be changed")]
    ImmutableField,
    #[error("invalid persisted agent thread transition")]
    InvalidTransition,
    #[error("agent thread storage revision is exhausted")]
    RevisionExhausted,
}

impl PersistedAgentThread {
    /// Validate a record returned by storage, including its exact requested owner.
    ///
    /// # Errors
    /// Malformed state, scope mismatches, and unsupported revisions fail closed.
    pub fn validate(&self, owner_id: &str) -> Result<(), AgentThreadStoreError> {
        self.thread.validate()?;
        if owner_id != self.thread.owner_id {
            return Err(AgentThreadStoreError::ScopeMismatch);
        }
        // Both durable stores have signed 64-bit integers. Keep the same bound
        // in memory, rather than allowing a value another provider cannot store.
        if self.revision > i64::MAX as u64 {
            return Err(AgentThreadStoreError::RevisionExhausted);
        }
        if self.thread.status == AgentThreadStatus::Pending && self.thread.run_id.is_some() {
            return Err(AgentThreadStoreError::InvalidTransition);
        }
        Ok(())
    }
}

pub(crate) fn new_root(
    owner_id: &str,
    thread: &AgentThread,
) -> Result<PersistedAgentThread, AgentThreadStoreError> {
    let record = PersistedAgentThread {
        thread: thread.clone(),
        revision: 0,
    };
    record.validate(owner_id)?;
    if thread.parent_thread_id.is_some()
        || thread.run_id.as_ref() != Some(&thread.root_run_id)
        || thread.history_revision != 0
        || thread.status != AgentThreadStatus::Running
    {
        return Err(AgentThreadStoreError::InvalidTransition);
    }
    Ok(record)
}

pub(crate) fn new_child(
    owner_id: &str,
    thread: &AgentThread,
    edge: &AgentEdge,
    parent: &PersistedAgentThread,
    root: &PersistedAgentThread,
) -> Result<PersistedAgentThread, AgentThreadStoreError> {
    let record = PersistedAgentThread {
        thread: thread.clone(),
        revision: 0,
    };
    record.validate(owner_id)?;
    parent.validate(owner_id)?;
    root.validate(owner_id)?;
    if root.thread.parent_thread_id.is_some()
        || root.thread.thread_id != thread.root_thread_id
        || root.thread.root_run_id != thread.root_run_id
    {
        return Err(AgentThreadStoreError::ScopeMismatch);
    }
    if thread.status != AgentThreadStatus::Pending
        || thread.history_revision != 0
        || !matches!(
            parent.thread.status,
            AgentThreadStatus::Running | AgentThreadStatus::Waiting
        )
        || !matches!(
            root.thread.status,
            AgentThreadStatus::Running | AgentThreadStatus::Waiting
        )
    {
        return Err(AgentThreadStoreError::InvalidTransition);
    }
    if &AgentEdge::between(&parent.thread, thread)? != edge {
        return Err(AgentThreadStoreError::ScopeMismatch);
    }
    Ok(record)
}

pub(crate) fn next_record(
    owner_id: &str,
    current: &PersistedAgentThread,
    expected_revision: u64,
    next: &AgentThread,
) -> Result<PersistedAgentThread, AgentThreadStoreError> {
    current.validate(owner_id)?;
    next.validate()?;
    if next.owner_id != owner_id {
        return Err(AgentThreadStoreError::ScopeMismatch);
    }
    if current.revision != expected_revision {
        return Err(AgentThreadStoreError::Conflict);
    }
    let before = &current.thread;
    if before.thread_id != next.thread_id
        || before.root_thread_id != next.root_thread_id
        || before.root_run_id != next.root_run_id
        || before.parent_thread_id != next.parent_thread_id
        || before.canonical_path != next.canonical_path
        || before.artifact_id != next.artifact_id
        || before.created_at != next.created_at
    {
        return Err(AgentThreadStoreError::ImmutableField);
    }
    let remote_transition = before.remote == next.remote
        || (before.remote.is_none()
            && next.remote.is_some()
            && before.parent_thread_id.is_some()
            && matches!(
                before.status,
                AgentThreadStatus::Running | AgentThreadStatus::Waiting
            )
            && next.status == before.status
            && next.run_id == before.run_id
            && next.history_revision == before.history_revision);
    if !remote_transition {
        return Err(AgentThreadStoreError::ImmutableField);
    }
    let valid_transition = if before == next {
        true
    } else {
        match before.status {
            AgentThreadStatus::Pending => match next.status {
                AgentThreadStatus::Pending => next.run_id.is_none(),
                AgentThreadStatus::Running => {
                    next.run_id.is_some() && next.history_revision > before.history_revision
                }
                AgentThreadStatus::Failed | AgentThreadStatus::Cancelled => next.run_id.is_none(),
                _ => false,
            },
            AgentThreadStatus::Running | AgentThreadStatus::Waiting => {
                next.status != AgentThreadStatus::Pending && next.run_id == before.run_id
            }
            AgentThreadStatus::Completed
            | AgentThreadStatus::Failed
            | AgentThreadStatus::Cancelled => {
                next.status == AgentThreadStatus::Running
                    && next.run_id.is_some()
                    && next.run_id != before.run_id
                    && next.history_revision > before.history_revision
            }
        }
    };
    if !valid_transition || next.history_revision < before.history_revision {
        return Err(AgentThreadStoreError::InvalidTransition);
    }
    let revision = current
        .revision
        .checked_add(1)
        .filter(|value| *value <= i64::MAX as u64)
        .ok_or(AgentThreadStoreError::RevisionExhausted)?;
    let record = PersistedAgentThread {
        thread: next.clone(),
        revision,
    };
    record.validate(owner_id)?;
    Ok(record)
}

pub(crate) fn validate_lookup(
    record: &PersistedAgentThread,
    owner_id: &str,
    thread_id: &str,
) -> Result<(), AgentThreadStoreError> {
    record.validate(owner_id)?;
    if record.thread.thread_id != thread_id {
        return Err(AgentThreadStoreError::ScopeMismatch);
    }
    Ok(())
}

pub(crate) fn ordered_threads(
    mut records: Vec<PersistedAgentThread>,
    owner_id: &str,
    root_run_id: &str,
) -> Result<Vec<PersistedAgentThread>, AgentThreadStoreError> {
    for record in &records {
        record.validate(owner_id)?;
        if record.thread.root_run_id != root_run_id {
            return Err(AgentThreadStoreError::ScopeMismatch);
        }
    }
    records.sort_by(|left, right| left.thread.order_key().cmp(&right.thread.order_key()));
    Ok(records)
}

pub(crate) fn ordered_edges(
    mut edges: Vec<AgentEdge>,
    threads: &[PersistedAgentThread],
    owner_id: &str,
    root_run_id: &str,
) -> Result<Vec<AgentEdge>, AgentThreadStoreError> {
    for edge in &edges {
        if edge.owner_id != owner_id || edge.root_run_id != root_run_id {
            return Err(AgentThreadStoreError::ScopeMismatch);
        }
        let parent = threads
            .iter()
            .find(|record| record.thread.thread_id == edge.parent_thread_id)
            .ok_or(AgentThreadStoreError::NotFound)?;
        let child = threads
            .iter()
            .find(|record| record.thread.thread_id == edge.child_thread_id)
            .ok_or(AgentThreadStoreError::NotFound)?;
        if &AgentEdge::between(&parent.thread, &child.thread)? != edge {
            return Err(AgentThreadStoreError::ScopeMismatch);
        }
    }
    edges.sort_by(|left, right| left.order_key().cmp(&right.order_key()));
    Ok(edges)
}

#[cfg(test)]
mod canonical_receipt_tests {
    use serde_json::json;

    use super::*;

    fn receipt(raw: Vec<u8>) -> CanonicalToolReceipt {
        CanonicalToolReceipt::acquire(
            "owner-1",
            "run-1",
            3,
            "call-1",
            "terminal.exec",
            CanonicalReceiptSource::Terminal,
            json!({"text": "snowman ☃\r\nnext"}),
            [("stdout".to_owned(), raw)],
            true,
            0,
        )
        .expect("receipt should be acquired")
    }

    #[test]
    fn complete_receipt_preserves_typed_value_and_raw_bytes_exactly() {
        let raw = b"snowman \xe2\x98\x83\r\nnext\0".to_vec();
        let receipt = receipt(raw.clone());

        receipt.validate().expect("receipt should validate");
        assert_eq!(receipt.completeness, CanonicalReceiptCompleteness::Complete);
        assert_eq!(
            receipt.typed_value,
            Some(json!({"text": "snowman ☃\r\nnext"}))
        );
        assert_eq!(
            receipt.raw_segments[0]
                .verified_bytes()
                .expect("raw segment should validate"),
            Some(raw)
        );
    }

    #[test]
    fn oversized_receipt_keeps_identity_and_digests_without_partial_payload() {
        let raw = vec![b'x'; MAX_CANONICAL_RECEIPT_BYTES as usize];
        let receipt = receipt(raw);

        receipt.validate().expect("receipt should validate");
        assert!(matches!(
            receipt.completeness,
            CanonicalReceiptCompleteness::AcquisitionLimitExceeded { observed_bytes }
                if observed_bytes > MAX_CANONICAL_RECEIPT_BYTES
        ));
        assert!(receipt.typed_value.is_none());
        assert_eq!(receipt.retained_bytes, 0);
        assert!(receipt.raw_segments[0].bytes_base64.is_none());
        assert_eq!(
            receipt.raw_segments[0].byte_len,
            MAX_CANONICAL_RECEIPT_BYTES
        );
        assert!(valid_sha256(&receipt.raw_segments[0].sha256));
    }

    #[test]
    fn incomplete_acquisition_is_explicit_and_never_retains_partial_payload() {
        let receipt = CanonicalToolReceipt::acquire(
            "owner-1",
            "run-1",
            4,
            "call-2",
            "terminal.exec",
            CanonicalReceiptSource::Terminal,
            json!({"partial": true}),
            [("stdout".to_owned(), b"partial".to_vec())],
            false,
            100,
        )
        .expect("receipt should be acquired");

        receipt.validate().expect("receipt should validate");
        assert_eq!(
            receipt.completeness,
            CanonicalReceiptCompleteness::AcquisitionIncomplete {
                observed_bytes: 100
            }
        );
        assert!(receipt.typed_value.is_none());
        assert!(receipt.raw_segments[0].bytes_base64.is_none());
    }

    #[test]
    fn metadata_only_raw_segment_cannot_be_labeled_complete() {
        let segment = CanonicalRawSegment::from_acquisition("stdout", None, 7, sha256(b"partial"))
            .expect("metadata-only segment should be valid");
        let receipt = CanonicalToolReceipt::acquire_with_segments(
            "owner-1",
            "run-1",
            5,
            "call-3",
            "terminal.exec",
            CanonicalReceiptSource::Terminal,
            json!({"partial": true}),
            vec![segment],
            true,
            7,
        )
        .expect("receipt should record incomplete acquisition");

        assert!(matches!(
            receipt.completeness,
            CanonicalReceiptCompleteness::AcquisitionIncomplete { .. }
        ));
        receipt.validate().expect("receipt should validate");
    }

    #[test]
    fn run_limit_records_metadata_and_same_payload_retry_is_idempotent() {
        let receipt = receipt(b"exact".to_vec());
        let retained_before = MAX_CANONICAL_RUN_BYTES - receipt.retained_bytes + 1;
        let stored = receipt
            .clone()
            .admit_to_run(retained_before)
            .expect("receipt admission should be recorded");

        stored.validate().expect("stored receipt should validate");
        assert_eq!(
            stored.completeness,
            CanonicalReceiptCompleteness::RunStorageLimitExceeded { retained_before }
        );
        assert!(stored.typed_value.is_none());
        assert!(stored.raw_segments[0].bytes_base64.is_none());
        assert_eq!(
            prepare_canonical_receipt(std::slice::from_ref(&stored), &receipt)
                .expect("same payload retry should be idempotent"),
            stored
        );
    }

    #[test]
    fn repeated_call_id_with_different_payload_conflicts() {
        let stored = receipt(b"first".to_vec());
        let changed = receipt(b"second".to_vec());

        assert!(matches!(
            prepare_canonical_receipt(&[stored], &changed),
            Err(CanonicalReceiptStoreError::Conflict)
        ));
    }
}
