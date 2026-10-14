//! SurrealDB's atomic checkpoint/envelope and immutable continuation CAS.

use crate::uar::persistence::checkpoint_resume::{
    CheckpointClaimOutcome, CheckpointContinuationClaim, CheckpointResumeEnvelope,
    CheckpointResumeError,
};
use super::SurrealDbProvider;

fn key(owner: &str, run: &str, checkpoint: &str) -> String {
    crate::uar::persistence::tenant_storage_key(owner,
        &crate::uar::persistence::tenant_storage_key(run, checkpoint))
}

/// Both trusted constructor paths initialize the exact store before its first
/// transactional read. Missing tables are not absent records in SurrealDB.
pub(super) async fn initialize(db: &super::Surreal<super::Any>) -> anyhow::Result<()> {
    check(db.query(
        "DEFINE TABLE IF NOT EXISTS checkpoints SCHEMALESS;
         DEFINE TABLE IF NOT EXISTS checkpoint_resume_envelopes SCHEMALESS;
         DEFINE TABLE IF NOT EXISTS checkpoint_runtime_epochs SCHEMALESS;
         DEFINE TABLE IF NOT EXISTS checkpoint_resume_claims SCHEMALESS;
         DEFINE TABLE IF NOT EXISTS checkpoint_resume_claim_history SCHEMALESS;"
    ).await.map_err(|error| write_error("store_initialization_query", error.into()))?)
        .map_err(|error| write_error("store_initialization_statements", error))
}

fn sdk_reason(error: &surrealdb::Error) -> &'static str {
    use surrealdb::types::{NotFoundError, QueryError, SerializationError, ValidationError};
    match error.not_found_details() {
        Some(NotFoundError::Table { .. }) => return "table_not_found",
        Some(NotFoundError::Record { .. }) => return "record_not_found",
        Some(NotFoundError::Namespace { .. }) => return "namespace_not_found",
        Some(NotFoundError::Database { .. }) => return "database_not_found",
        _ => {}
    }
    match error.query_details() {
        Some(QueryError::NotExecuted) => return "query_not_executed",
        Some(QueryError::Cancelled) => return "query_cancelled",
        Some(QueryError::TimedOut { .. }) => return "query_timed_out",
        Some(QueryError::TransactionConflict) => return "transaction_conflict",
        _ => {}
    }
    match error.validation_details() {
        Some(ValidationError::Parse) => return "query_parse_error",
        Some(ValidationError::InvalidContent { .. }) => return "invalid_content",
        Some(ValidationError::InvalidParameter { .. }) => return "invalid_parameter",
        _ => {}
    }
    match error.serialization_details() {
        Some(SerializationError::Serialization) => return "serialization_error",
        Some(SerializationError::Deserialization) => return "deserialization_error",
        _ => {}
    }
    error.kind_str()
}

fn check(mut response: surrealdb::IndexedResults) -> anyhow::Result<()> {
    let errors = response.take_errors();
    for (index, error) in &errors {
        tracing::warn!(checkpoint_stage = "transaction_statement", checkpoint_statement_index = *index,
            checkpoint_sdk_kind = error.kind_str(), checkpoint_error_class = sdk_reason(error),
            "Checkpoint transaction statement reported an error");
        let message = error.to_string();
        if message.contains("uar_checkpoint_predecessor_uncertain") {
            return Err(CheckpointResumeError::PredecessorUncertain.into());
        }
        if message.contains("uar_checkpoint_executor_closed") {
            return Err(CheckpointResumeError::ExecutorClosed.into());
        }
        if message.contains("uar_checkpoint_conflict") {
            return Err(CheckpointResumeError::Conflict.into());
        }
    }
    if let Some((_, error)) = errors.into_iter().min_by_key(|(index, _)| *index) {
        return Err(error.into());
    }
    Ok(())
}

fn write_error(stage: &'static str, error: anyhow::Error) -> anyhow::Error {
    let class = match error.downcast_ref::<CheckpointResumeError>() {
        Some(CheckpointResumeError::Unsupported) => "unsupported",
        Some(CheckpointResumeError::Invalid) => "protected_metadata_invalid",
        Some(CheckpointResumeError::Conflict) => "checkpoint_conflict",
        Some(CheckpointResumeError::ExecutorClosed) => "executor_closed",
        Some(CheckpointResumeError::PredecessorUncertain) => "predecessor_uncertain",
        Some(CheckpointResumeError::AdmissionDenied) => "admission_denied",
        Some(CheckpointResumeError::ExecutionUnavailable) => "execution_unavailable",
        None => error.downcast_ref::<surrealdb::Error>().map_or("storage_error", sdk_reason),
    };
    // Database Display/Debug can embed a query or protected payload. Emit only
    // fixed host-owned stage/class labels, never the underlying error text.
    tracing::warn!(checkpoint_stage = stage, checkpoint_error_class = class,
        "Checkpoint persistence operation reported an error");
    error
}

pub(super) async fn save(db: &SurrealDbProvider, envelope: &CheckpointResumeEnvelope) -> anyhow::Result<()> {
    supported(db)?;
    envelope.validate()?;
    let checkpoint = envelope.checkpoint()?;
    let mut payload = super::to_db_value(&checkpoint)?;
    let fields = payload.as_object_mut().ok_or(CheckpointResumeError::Invalid)?;
    // The Surreal record owns `id`; the logical checkpoint ID is restored from
    // the existing `_cp_id` field or the explicit load parameter.
    fields.remove("id");
    fields.insert("_cp_id".to_owned(), checkpoint.id.clone().into());
    let response = db.db.query(
        "BEGIN TRANSACTION;
         LET $epoch = (SELECT * FROM type::record('checkpoint_runtime_epochs', $epoch_id))[0];
         IF $epoch != NONE AND $epoch.state != 'active' { THROW 'uar_checkpoint_executor_closed'; };
         IF $epoch = NONE { CREATE type::record('checkpoint_runtime_epochs', $epoch_id)
             CONTENT { state: 'active', epoch: $epoch_id, checkpoint_fence: 0, claim_fence: 0 }; };
         UPDATE type::record('checkpoint_runtime_epochs', $epoch_id) SET checkpoint_fence += 1;
         LET $old = (SELECT * FROM type::record('checkpoint_resume_envelopes', $key))[0];
         LET $cp = (SELECT * FROM type::record('checkpoints', $checkpoint))[0];
         IF $old != NONE OR $cp != NONE { THROW 'uar_checkpoint_conflict'; };
         CREATE type::record('checkpoints', $checkpoint) CONTENT $checkpoint_payload;
         CREATE type::record('checkpoint_resume_envelopes', $key) CONTENT $payload;
         COMMIT TRANSACTION;"
    ).bind(("epoch_id", envelope.source_runtime_epoch.clone()))
        .bind(("key", key(&envelope.owner_key, &envelope.source_run_id, &envelope.checkpoint_id)))
        .bind(("checkpoint", envelope.checkpoint_id.clone()))
        .bind(("checkpoint_payload", payload))
        .bind(("payload", serde_json::json!({
            "owner_key": envelope.owner_key,
            "source_run_id": envelope.source_run_id,
            "checkpoint_id": envelope.checkpoint_id,
            "revision": envelope.checkpoint_revision,
            "source_epoch": envelope.source_runtime_epoch,
            "created_at": checkpoint.created_at,
            "data": serde_json::to_string(envelope)?,
        }))).await.map_err(|error| write_error("save_query", error.into()))?;
    // A repeated identical save is idempotent only after full protected readback.
    match check(response) {
        Ok(()) => {
            let saved = load(db, &envelope.owner_key, &envelope.source_run_id, &envelope.checkpoint_id).await
                .map_err(|error| write_error("save_readback", error))?;
            if saved.as_ref() != Some(envelope) {
                return Err(write_error("save_readback_identity", CheckpointResumeError::Invalid.into()));
            }
            Ok(())
        }
        Err(error) => {
            let error = write_error("save_transaction", error);
            let existing = load(db, &envelope.owner_key, &envelope.source_run_id, &envelope.checkpoint_id).await
                .map_err(|error| write_error("save_conflict_readback", error))?;
            if existing.as_ref() == Some(envelope) { Ok(()) } else { Err(error) }
        }
    }
}

fn supported(db: &SurrealDbProvider) -> anyhow::Result<()> {
    if !db.durable_instances { return Err(CheckpointResumeError::Unsupported.into()); }
    Ok(())
}

pub(super) async fn load(db: &SurrealDbProvider, owner: &str, run: &str, checkpoint: &str) -> anyhow::Result<Option<CheckpointResumeEnvelope>> {
    supported(db)?;
    let mut response = db.db.query(
        "SELECT VALUE data FROM checkpoint_resume_envelopes
         WHERE owner_key = $owner AND source_run_id = $run AND checkpoint_id = $checkpoint"
    ).bind(("owner", owner.to_owned())).bind(("run", run.to_owned()))
        .bind(("checkpoint", checkpoint.to_owned())).await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    let Some(data) = rows.first() else { return Ok(None); };
    let envelope: CheckpointResumeEnvelope = serde_json::from_str(data)?;
    envelope.validate()?;
    if envelope.owner_key != owner || envelope.source_run_id != run || envelope.checkpoint_id != checkpoint || rows.len() != 1 {
        return Err(CheckpointResumeError::Invalid.into());
    }
    Ok(Some(envelope))
}

pub(super) async fn list(db: &SurrealDbProvider, owner: &str, run: &str) -> anyhow::Result<Vec<CheckpointResumeEnvelope>> {
    supported(db)?;
    let mut response = db.db.query(
        "SELECT VALUE data FROM checkpoint_resume_envelopes
         WHERE owner_key = $owner AND source_run_id = $run ORDER BY created_at ASC"
    ).bind(("owner", owner.to_owned())).bind(("run", run.to_owned())).await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    rows.into_iter().map(|data| {
        let envelope: CheckpointResumeEnvelope = serde_json::from_str(&data)?;
        envelope.validate()?;
        if envelope.owner_key != owner || envelope.source_run_id != run {
            return Err(CheckpointResumeError::Invalid.into());
        }
        Ok(envelope)
    }).collect()
}

async fn existing_claim(db: &SurrealDbProvider, claim_key: &str) -> anyhow::Result<Option<CheckpointContinuationClaim>> {
    let mut response = db.db.query(
        "SELECT VALUE data FROM type::record('checkpoint_resume_claims', $key)"
    ).bind(("key", claim_key.to_owned())).await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    rows.first().map(|data| serde_json::from_str(data).map_err(anyhow::Error::from)).transpose()
}

pub(super) async fn claim(db: &SurrealDbProvider, envelope: &CheckpointResumeEnvelope, claim: &CheckpointContinuationClaim) -> anyhow::Result<CheckpointClaimOutcome> {
    supported(db)?;
    envelope.require_claimable_execution()?;
    claim.validate(envelope)?;
    // Stronger than checkpoint-only uniqueness: a different revision cannot
    // launch another concurrent execution from the same source run either.
    let claim_key = crate::uar::persistence::tenant_storage_key(&claim.owner_key, &claim.source_run_id);
    let prior = existing_claim(db, &claim_key).await?;
    if let Some(existing) = &prior {
        if existing.checkpoint_id != envelope.checkpoint_id { return Err(CheckpointResumeError::Conflict.into()); }
        existing.validate(envelope)?;
        if !supervisor_excluded(db,&existing.executor_runtime_epoch).await? {
            return Ok(CheckpointClaimOutcome::Existing(existing.clone()));
        }
        require_resolved_effects(db,&existing.executor_runtime_epoch,&existing.continuation_run_id).await?;
    }
    let source_supervisor_excluded = supervisor_excluded(db,&envelope.source_runtime_epoch).await?;
    if source_supervisor_excluded {
        require_resolved_effects(db,&envelope.source_runtime_epoch,&envelope.source_run_id).await?;
    }
    let response = db.db.query(
        "BEGIN TRANSACTION;
         LET $source = (SELECT * FROM type::record('checkpoint_resume_envelopes', $envelope_key))[0];
         IF $source = NONE OR $source.cancelled = true OR $source.revision != $revision OR $source.data != $envelope_data {
             THROW 'uar_checkpoint_conflict';
         };
         LET $predecessor = (SELECT * FROM type::record('checkpoint_runtime_epochs', $source_epoch))[0];
         IF $predecessor = NONE OR $predecessor.state NOT IN ['orderly_excluded', 'supervisor_excluded'] {
             THROW 'uar_checkpoint_predecessor_uncertain';
         };
         IF $source_supervisor_excluded != ($predecessor.state = 'supervisor_excluded') {
             THROW 'uar_checkpoint_predecessor_uncertain';
         };
         LET $executor = (SELECT * FROM type::record('checkpoint_runtime_epochs', $executor_epoch))[0];
         IF $executor != NONE AND $executor.state != 'active' { THROW 'uar_checkpoint_executor_closed'; };
         IF $executor = NONE { CREATE type::record('checkpoint_runtime_epochs', $executor_epoch)
             CONTENT { state: 'active', epoch: $executor_epoch, checkpoint_fence: 0, claim_fence: 0 }; };
         UPDATE type::record('checkpoint_runtime_epochs', $executor_epoch) SET claim_fence += 1;
         LET $old = (SELECT * FROM type::record('checkpoint_resume_claims', $claim_key))[0];
         IF $has_prior {
             IF $old = NONE OR $old.data != $prior_data { THROW 'uar_checkpoint_conflict'; };
             LET $prior_executor = (SELECT * FROM type::record('checkpoint_runtime_epochs',$prior_epoch))[0];
             IF $prior_executor = NONE OR $prior_executor.state != 'supervisor_excluded' {
                 THROW 'uar_checkpoint_predecessor_uncertain';
             };
             LET $generation = IF $old.generation = NONE { 1 } ELSE { $old.generation };
             CREATE type::record('checkpoint_resume_claim_history',$history_key) SET data=$old.data,generation=$generation,excluded_executor=$prior_epoch;
             UPDATE type::record('checkpoint_resume_claims',$claim_key) CONTENT $payload;
             UPDATE type::record('checkpoint_resume_claims',$claim_key) SET generation=$generation+1;
         } ELSE {
             IF $old != NONE { THROW 'uar_checkpoint_conflict'; };
             CREATE type::record('checkpoint_resume_claims', $claim_key) CONTENT $payload;
         };
         COMMIT TRANSACTION;"
    ).bind(("envelope_key", key(&envelope.owner_key, &envelope.source_run_id, &envelope.checkpoint_id)))
        .bind(("revision", envelope.checkpoint_revision.clone()))
        .bind(("envelope_data", serde_json::to_string(envelope)?))
        .bind(("source_epoch", claim.source_runtime_epoch.clone()))
        .bind(("source_supervisor_excluded", source_supervisor_excluded))
        .bind(("executor_epoch", claim.executor_runtime_epoch.clone()))
        .bind(("claim_key", claim_key.clone()))
        .bind(("has_prior",prior.is_some()))
        .bind(("prior_data",prior.as_ref().map(serde_json::to_string).transpose()?.unwrap_or_default()))
        .bind(("prior_epoch",prior.as_ref().map(|prior| prior.executor_runtime_epoch.clone()).unwrap_or_default()))
        .bind(("history_key",prior.as_ref().map(|prior| crate::uar::persistence::tenant_storage_key(&claim_key,&prior.continuation_run_id)).unwrap_or_default()))
        .bind(("payload", serde_json::json!({
            "owner_key": claim.owner_key,
            "source_run_id": claim.source_run_id,
            "revision": claim.checkpoint_revision,
            "executor_epoch": claim.executor_runtime_epoch,
            "generation": 1,
            "data": serde_json::to_string(claim)?,
        }))).await?;
    match check(response) {
        Ok(()) => {
            let durable = existing_claim(db, &claim_key).await?.ok_or(CheckpointResumeError::Invalid)?;
            durable.validate(envelope)?;
            if &durable != claim { return Err(CheckpointResumeError::Conflict.into()); }
            Ok(CheckpointClaimOutcome::Acquired(durable))
        }
        Err(error) => {
            if let Some(existing) = existing_claim(db, &claim_key).await? {
                if existing.checkpoint_id != envelope.checkpoint_id { return Err(CheckpointResumeError::Conflict.into()); }
                existing.validate(envelope)?;
                return Ok(CheckpointClaimOutcome::Existing(existing));
            }
            Err(error)
        }
    }
}

pub(super) async fn exclude_epoch(db: &SurrealDbProvider, epoch: &str) -> anyhow::Result<()> {
    supported(db)?;
    if epoch.is_empty() { return Err(CheckpointResumeError::Invalid.into()); }
    let response = db.db.query(
        "BEGIN TRANSACTION;
         LET $old = (SELECT * FROM type::record('checkpoint_runtime_epochs', $epoch))[0];
         IF $old = NONE {
             CREATE type::record('checkpoint_runtime_epochs', $epoch)
                 CONTENT { epoch: $epoch, state: 'orderly_excluded', excluded_at: time::now() };
         } ELSE {
             IF $old.state NOT IN ['active','orderly_excluded'] { THROW 'uar_checkpoint_executor_closed'; };
             UPDATE type::record('checkpoint_runtime_epochs', $epoch)
                 SET state = 'orderly_excluded', excluded_at = time::now();
         };
         COMMIT TRANSACTION;"
    ).bind(("epoch", epoch.to_owned())).await?;
    check(response)?;
    let mut readback = db.db.query("SELECT VALUE state FROM type::record('checkpoint_runtime_epochs', $epoch)")
        .bind(("epoch", epoch.to_owned())).await?.check()?;
    let states: Vec<String> = readback.take(0)?;
    if states != ["orderly_excluded"] { return Err(CheckpointResumeError::Invalid.into()); }
    Ok(())
}

/// Exclusion only closes execution. Durable unknown effects and pending money
/// remain a separate barrier; a reboot cannot turn them into safe replay.
async fn supervisor_excluded(db: &SurrealDbProvider, epoch: &str) -> anyhow::Result<bool> {
    let mut response = db.db.query("SELECT VALUE state FROM type::record('checkpoint_runtime_epochs',$epoch)")
        .bind(("epoch",epoch.to_owned())).await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    Ok(rows == ["supervisor_excluded"])
}
pub(super) async fn require_resolved_effects(db: &SurrealDbProvider, epoch: &str, run: &str) -> anyhow::Result<()> {
    use crate::uar::persistence::tool_admission::{ToolAdmissionEvidence,ToolAdmissionEvidenceState};
    // Legacy rows do not have indexed epoch/root fields. Decode exact protected
    // evidence rather than treating missing fields as absence of an invocation.
    let mut response = db.db.query("SELECT VALUE data FROM tool_admission_evidence").await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    let mut pending = std::collections::BTreeMap::new();
    for row in rows {
        let evidence: ToolAdmissionEvidence = serde_json::from_str(&row)?;
        if evidence.runtime_epoch != epoch || (evidence.root_run_id != run && evidence.run_id != run) { continue; }
        if evidence.state == ToolAdmissionEvidenceState::OutcomeUnknown { return Err(CheckpointResumeError::PredecessorUncertain.into()); }
        let states = pending.entry(evidence.invocation_id).or_insert((false,false));
        match evidence.state {
            ToolAdmissionEvidenceState::ClaimIntent => states.0 = true,
            ToolAdmissionEvidenceState::AwaitingApproval => {}, // Dot07 reconstructs current approval authority.
            _ => states.1 = true,
        }
    }
    if pending.values().any(|(claimed,terminal)| *claimed && !terminal) { return Err(CheckpointResumeError::PredecessorUncertain.into()); }
    let mut run_ids = std::collections::BTreeSet::from([run.to_owned()]);
    let mut response = db.db.query("SELECT VALUE data FROM agent_threads WHERE root_run_id=$run")
        .bind(("run",run.to_owned())).await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    for row in rows {
        let thread: crate::uar::persistence::agent_threads::PersistedAgentThread = serde_json::from_str(&row)?;
        if let Some(run_id) = thread.thread.run_id { run_ids.insert(run_id); }
    }
    let mut response = db.db.query("SELECT VALUE data FROM accounting_calls WHERE run_id IN $runs")
        .bind(("runs",run_ids.iter().cloned().collect::<Vec<_>>())).await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    for row in rows {
        let call: crate::uar::persistence::accounting::AccountingCall = serde_json::from_str(&row)?;
        if call.state == crate::uar::persistence::accounting::AccountingState::Pending { return Err(CheckpointResumeError::PredecessorUncertain.into()); }
    }
    let mut response = db.db.query("SELECT VALUE data FROM canonical_tool_receipts WHERE run_id IN $runs")
        .bind(("runs",run_ids.into_iter().collect::<Vec<_>>())).await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    for row in rows {
        let receipt: crate::uar::persistence::agent_threads::CanonicalToolReceipt = serde_json::from_str(&row)?;
        if receipt.outcome.is_none_or(|outcome| outcome == crate::mcp::outcome::ToolOutcome::OutcomeUnknown) {
            return Err(CheckpointResumeError::PredecessorUncertain.into());
        }
    }
    Ok(())
}
