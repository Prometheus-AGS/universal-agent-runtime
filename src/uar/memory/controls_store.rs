//! Transactions over the pinned MemoryStorage implementation's existing database.
use anyhow::{Context, Result};
use std::collections::HashMap;
use surrealdb::{Surreal, engine::any::Any, types::{Datetime, QueryError, RecordId, SurrealValue}};
use surreal_memory::{Memory, SurrealStorage};
use super::{CapturePolicy, CapturePolicyUpdate, Retention, MemoryService};
use crate::uar::memory::owned::record_id;

fn db(service: &MemoryService) -> Result<Surreal<Any>> {
    service.storage.as_any().downcast_ref::<SurrealStorage>()
        .context("Durable memory controls require the production Surreal memory store")?.db()
}
fn memory_id(id: &str) -> Result<RecordId> {
    let record = if id.contains(':') { RecordId::parse_simple(id)? } else { RecordId::new("memory", id) };
    anyhow::ensure!(record.table.as_str() == "memory", "Expected memory record ID");
    Ok(record)
}
fn control_id(record: &RecordId) -> Result<RecordId> {
    Ok(RecordId::new("uar_memory_control", record_id(record)?))
}

pub(super) async fn initialize(service: &MemoryService) -> Result<()> {
    // The pinned V8 migration uses IF NOT EXISTS after V2 already defined this
    // field, leaving it strict. Imported legacy provenance needs nested keys.
    db(service)?.query("DEFINE TABLE IF NOT EXISTS uar_memory_policy SCHEMALESS;
        DEFINE TABLE IF NOT EXISTS uar_memory_control SCHEMALESS;
        DEFINE INDEX IF NOT EXISTS owner ON uar_memory_control FIELDS owner;
        DEFINE INDEX IF NOT EXISTS pending_purge ON uar_memory_control FIELDS deleted, purged;
        DEFINE FIELD OVERWRITE metadata ON memory TYPE option<object> FLEXIBLE;")
        .await?.check()?;
    // The ordinary startup/TTL path re-enters cleanup after an interrupted delete.
    purge(service).await?;
    Ok(())
}

#[derive(serde::Deserialize, SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct PolicyRow {
    enabled: bool,
    days: Option<u16>,
    chosen: bool,
    revision: u64,
    consent_epoch: u64,
}

pub(super) async fn policy(service: &MemoryService, owner: &str) -> Result<CapturePolicy> {
    let rows: Vec<PolicyRow> = db(service)?.query("SELECT * FROM type::record('uar_memory_policy',$owner)")
        .bind(("owner", owner.to_owned())).await?.check()?.take(0)?;
    Ok(match rows.into_iter().next() {
        Some(row) => CapturePolicy { enabled: row.enabled, retention: if row.chosen { Some(match row.days {
            Some(days) => Retention::Days { days }, None => Retention::UntilDeleted,
        }) } else { None }, revision: row.revision, consent_epoch: row.consent_epoch, host_capture_enabled: service.config.auto_capture },
        None => CapturePolicy { enabled: false, retention: None, revision: 0, consent_epoch: 0, host_capture_enabled: service.config.auto_capture },
    })
}

pub(super) async fn set_policy(service: &MemoryService, owner: &str, update: CapturePolicyUpdate) -> Result<CapturePolicy> {
    let days = update.retention.as_ref().and_then(Retention::days);
    // Every revision writes the same owner row as capture. Shortening updates the
    // durable bound, never recomputes it from a later (possibly longer) policy.
    db(service)?.query("BEGIN TRANSACTION;
        LET $pid = type::record('uar_memory_policy',$owner);
        LET $prior = (SELECT * FROM $pid)[0];
        LET $revision = IF $prior = NONE { 1 } ELSE { $prior.revision + 1 };
        LET $epoch = IF $prior = NONE { 0 } ELSE { $prior.consent_epoch + IF $enabled = false { 1 } ELSE { 0 } };
        UPSERT $pid SET enabled=$enabled, days=$days, chosen=$chosen, revision=$revision, consent_epoch=$epoch, fence=rand::uuid();
        LET $duration=IF $days=7 { 7d } ELSE IF $days=30 { 30d } ELSE IF $days=90 { 90d } ELSE { 365d };
        IF $days != NONE {
            UPDATE uar_memory_control SET expires_at = first_capture + $duration
                WHERE owner=$owner AND automatic=true AND deleted=false
                AND (expires_at=NONE OR expires_at > first_capture + $duration);
        };
        COMMIT TRANSACTION;")
        .bind(("owner", owner.to_owned())).bind(("enabled", update.enabled))
        .bind(("chosen", update.retention.is_some())).bind(("days", days))
        .await?.check()?;
    policy(service, owner).await
}

pub(super) async fn visible(service: &MemoryService, mut memory: Memory) -> Result<Option<Memory>> {
    let mid = memory.id.as_ref().context("Memory has no durable identity")?.clone();
    let cid = control_id(&mid)?;
    // Read time comes from the shared database clock. No process cache can
    // survive revocation, shortening or deletion performed by another host.
    let mut response = db(service)?.query("LET $c = (SELECT * FROM $cid)[0];
        SELECT VALUE id FROM $mid WHERE (valid_until=NONE OR valid_until > time::now())
        AND ($c=NONE OR ($c.deleted=false AND ($c.expires_at=NONE OR $c.expires_at > time::now())));")
        .bind(("cid", cid)).bind(("mid", mid.clone())).await?.check()?;
    let live: Vec<RecordId> = response.take(1)?;
    if live.is_empty() { return Ok(None); }
    let control = read_control(service, &mid).await?;
    if let Some(control) = control {
        if control.deleted || control.expired { return Ok(None); }
        // Derived records retain a flattened list of source control identities.
        // A compressed source is hidden itself, but an owner deletion or expiry
        // revokes every descendant, even before index cleanup reaches it.
        for source in control.sources.as_deref().unwrap_or_default() {
            if let Some(source) = read_control(service, source).await? {
                if source.deleted && source.reason.as_deref() != Some("compressed") { return Ok(None); }
                if source.expired { return Ok(None); }
            }
        }
        let metadata = memory.metadata.get_or_insert_with(|| serde_json::json!({}));
        if let Some(metadata) = metadata.as_object_mut() {
            metadata.insert("uar_capture".into(), serde_json::json!({
                "automatic": control.automatic.unwrap_or(false), "first_capture": control.first_capture,
                "policy_revision": control.policy_revision, "retention_days": control.retention_days,
                "effective_expires_at": control.expires_at,
                "retention": if control.automatic.unwrap_or(false) { Some(match control.retention_days { Some(days) => Retention::Days { days }, None => Retention::UntilDeleted }) } else { None },
            }));
        }
    }
    Ok(Some(memory))
}

pub(super) async fn save(service: &MemoryService, mut memory: Memory, key: Option<&str>, epoch: Option<u64>) -> Result<Option<Memory>> {
    let owner = memory.user_id.clone().context("Memory requires owner")?;
    let key = key.map(str::to_owned).unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let mid = RecordId::new("memory", key.clone());
    let cid = control_id(&mid)?;
    let embedding = service.embeddings.embed(&memory.content).await?;
    memory.token_count = Some(u32::try_from(memory.content.len().div_ceil(4)).unwrap_or(u32::MAX));
    memory.embedding = Some(embedding);
    let row = MemoryRow::from(memory);
    // JSON-bound option fields use NONE via the typed storage DTO. Both record
    // and index data plus creation history commit with consent and provenance.
    let response = db(service)?.query("BEGIN TRANSACTION;
        LET $pid=type::record('uar_memory_policy',$owner);
        LET $p=(SELECT * FROM $pid)[0];
        IF $automatic AND ($host=false OR $p=NONE OR $p.enabled=false OR $p.consent_epoch != $epoch) { THROW 'uar_memory_capture_revoked'; };
        IF $automatic { UPDATE $pid SET fence=rand::uuid(); };
        LET $existing=(SELECT * FROM $cid)[0];
        IF $existing != NONE AND ($existing.deleted=true OR ($existing.expires_at != NONE AND $existing.expires_at <= time::now())) { THROW 'uar_memory_removed'; };
        LET $record=(SELECT * FROM $mid)[0];
        IF $record=NONE {
            IF $stream != NONE {
                LET $task=(SELECT * FROM $stream)[0];
                IF $task=NONE OR $task.user_id != $owner OR $task.status != 'active' { THROW 'uar_memory_task_unavailable'; };
            };
            LET $now=time::now();
            LET $expiry=IF $automatic AND $p.days != NONE { $now + IF $p.days=7 { 7d } ELSE IF $p.days=30 { 30d } ELSE IF $p.days=90 { 90d } ELSE { 365d } } ELSE { NONE };
            CREATE $mid CONTENT $memory;
            UPDATE $mid SET created_at=$now, updated_at=$now;
            CREATE $cid SET memory_id=$mid, owner=$owner, automatic=$automatic, first_capture=$now,
                expires_at=$expiry, policy_revision=IF $automatic { $p.revision } ELSE { NONE },
                retention_days=IF $automatic { $p.days } ELSE { NONE }, deleted=false, purged=false;
            CREATE memory_history SET memory_id=$mid, version=1, old_content=NONE, new_content=$content, changed_at=$now, change_type='created';
            IF $stream != NONE { UPDATE $stream SET total_tokens += $tokens, last_active=$now; };
        };
        COMMIT TRANSACTION;")
        .bind(("owner", owner)).bind(("mid", mid)).bind(("cid", cid))
        .bind(("automatic", epoch.is_some())).bind(("epoch", epoch))
        .bind(("host", service.config.auto_capture)).bind(("content", row.content.clone()))
        .bind(("stream", row.task_stream_id.clone())).bind(("tokens", row.token_count))
        .bind(("memory", row)).await?;
    let mut response = response;
    let errors = response.take_errors();
    if errors.values().any(|error| error.to_string().contains("uar_memory_capture_revoked") || error.to_string().contains("uar_memory_removed")) { return Ok(None); }
    if let Some(error) = select_transaction_error(errors) { return Err(error.into()); }
    match service.storage.get_memory(&key).await? {
        Some(memory) => visible(service, memory).await,
        None => Ok(None),
    }
}

fn is_not_executed(error: &surrealdb::Error) -> bool {
    error.query_details() == Some(&QueryError::NotExecuted)
}

fn is_transaction_conflict(error: &surrealdb::Error) -> bool {
    error.query_details() == Some(&QueryError::TransactionConflict)
}

/// A failed SurrealDB transaction has one response row per statement. The SDK
/// reports the earliest row first, but 3.3 uses those rows for `NotExecuted`
/// placeholders after a transaction fails. Select the first substantive error
/// instead, retaining a typed conflict when the entire response is its abort
/// shape and the final placeholder when that is all the server supplied.
fn select_transaction_error(errors: HashMap<usize, surrealdb::Error>) -> Option<surrealdb::Error> {
    let mut errors: Vec<_> = errors.into_iter().collect();
    errors.sort_by_key(|(index, _)| *index);
    let definite_conflict = errors.iter().any(|(_, error)| is_transaction_conflict(error))
        && errors.iter().all(|(_, error)| is_transaction_conflict(error) || is_not_executed(error));
    if definite_conflict {
        return errors.into_iter().map(|(_, error)| error).find(is_transaction_conflict);
    }
    if let Some(position) = errors.iter().position(|(_, error)| !is_not_executed(error) && !is_transaction_conflict(error)) {
        return Some(errors.swap_remove(position).1);
    }
    errors.pop().map(|(_, error)| error)
}

pub(super) async fn update(service: &MemoryService, id: &str, owner: &str, content: String) -> Result<Memory> {
    let mid = memory_id(id)?;
    let cid = control_id(&mid)?;
    let embedding = service.embeddings.embed(&content).await?;
    db(service)?.query("BEGIN TRANSACTION;
        LET $m=(SELECT * FROM $mid)[0]; LET $c=(SELECT * FROM $cid)[0];
        IF $m=NONE OR $m.user_id != $owner OR ($m.valid_until != NONE AND $m.valid_until <= time::now())
            OR ($c != NONE AND ($c.deleted OR ($c.expires_at != NONE AND $c.expires_at <= time::now()))) { THROW 'uar_memory_unavailable'; };
        IF $c != NONE { UPDATE $cid SET fence=rand::uuid(); };
        UPDATE $mid SET content=$content, embedding=$embedding, token_count=$tokens, version=$m.version+1, updated_at=time::now();
        CREATE memory_history SET memory_id=$mid, version=$m.version+1, old_content=$m.content, new_content=$content, changed_at=time::now(), change_type='updated';
        COMMIT TRANSACTION;")
        .bind(("mid", mid)).bind(("cid", cid)).bind(("owner", owner.to_owned()))
        .bind(("tokens", u64::try_from(content.len().div_ceil(4))?))
        .bind(("content", content)).bind(("embedding", embedding)).await?.check()?;
    let memory = service.storage.get_memory(id).await?.context("Memory unavailable after update")?;
    visible(service, memory).await?.context("Memory unavailable after update")
}

pub(super) async fn delete(service: &MemoryService, id: &str, owner: &str) -> Result<()> {
    let mid=memory_id(id)?; let cid=control_id(&mid)?;
    db(service)?.query("BEGIN TRANSACTION;
        LET $m=(SELECT * FROM $mid)[0]; LET $c=(SELECT * FROM $cid)[0];
        IF ($m=NONE AND $c=NONE) OR ($m != NONE AND $m.user_id != $owner) OR ($c != NONE AND $c.owner != $owner) { THROW 'uar_memory_unavailable'; };
        UPSERT $cid SET memory_id=$mid, owner=$owner, deleted=true, purged=false, deleted_at=time::now(), reason='owner_deleted';
        DELETE memory_history WHERE memory_id=$mid;
        IF $m != NONE { UPDATE $mid SET metadata=NONE, updated_at=time::now(); };
        FOR $child IN (SELECT * FROM uar_memory_control WHERE owner=$owner AND $mid IN sources) {
            UPDATE $child.id SET deleted=true, purged=false, reason='source_deleted', deleted_at=time::now();
            DELETE memory_history WHERE memory_id=$child.memory_id;
            LET $descendant=(SELECT id FROM $child.memory_id)[0];
            IF $descendant != NONE { UPDATE $child.memory_id SET metadata=NONE, updated_at=time::now(); };
        };
        COMMIT TRANSACTION;")
        .bind(("mid", mid)).bind(("cid", cid)).bind(("owner", owner.to_owned())).await?.check()?;
    // Do not depend on physical deletion to acknowledge suppression. Startup
    // and the TTL worker finish cleanup even if this request is cancelled now.
    Ok(())
}

pub(super) async fn purge(service: &MemoryService) -> Result<u64> {
    let database = db(service)?;
    // Durable expiry tombstones are committed before physical cleanup.
    database.query("BEGIN TRANSACTION;
        UPDATE uar_memory_control SET deleted=true, reason='expired', deleted_at=time::now()
            WHERE deleted=false AND expires_at != NONE AND expires_at <= time::now();
        COMMIT TRANSACTION;").await?.check()?;
    let stale: Vec<ExpiredRow> = database.query("SELECT id, user_id FROM memory WHERE valid_until != NONE AND valid_until <= time::now()")
        .await?.check()?.take(0)?;
    for memory in stale {
        let owner = memory.user_id.context("Expired memory has no owner")?;
        delete(service, &record_id(&memory.id)?, &owner).await?;
    }
    // Descendant revocation is durable as well as read-time suppression.
    let descendants: Vec<ControlRow> = database.query("SELECT *, (expires_at != NONE AND expires_at <= time::now()) AS expired FROM uar_memory_control WHERE sources != NONE AND deleted=false")
        .await?.check()?.take(0)?;
    for descendant in descendants {
        for source in descendant.sources.as_deref().unwrap_or_default() {
            if let Some(source) = read_control(service, source).await? {
                if source.expired || (source.deleted && source.reason.as_deref() != Some("compressed")) {
                    delete(service, &record_id(&descendant.memory_id)?, &descendant.owner).await?;
                    break;
                }
            }
        }
    }
    let pending: Vec<RecordId> = database.query("SELECT VALUE memory_id FROM uar_memory_control WHERE deleted=true AND purged=false")
        .await?.check()?.take(0)?;
    let mut count=0;
    for mid in pending {
        let cid=control_id(&mid)?;
        database.query("BEGIN TRANSACTION;
            LET $c=(SELECT * FROM $cid)[0];
            IF $c.deleted=true {
                LET $m=(SELECT * FROM $mid)[0];
                IF $m != NONE AND $m.task_stream_id != NONE {
                    UPDATE $m.task_stream_id SET total_tokens=IF total_tokens >= ($m.token_count ?? 0) { total_tokens - ($m.token_count ?? 0) } ELSE { 0 };
                };
                DELETE memory_history WHERE memory_id=$mid;
                DELETE $mid;
                UPDATE $cid SET purged=true, purged_at=time::now();
            };
            COMMIT TRANSACTION;")
            .bind(("mid", mid)).bind(("cid", cid)).await?.check()?;
        count += 1;
    }
    Ok(count)
}

/// Summarization must not turn expiring/revoked content into an immortal copy.
/// Source versions and all lineage controls are fenced in the same transaction.
pub(super) async fn summarize(service: &MemoryService, owner: &str, sources: Vec<Memory>, mut summary: Memory) -> Result<Option<Memory>> {
    if sources.is_empty() { return Ok(None); }
    let mut lineage = Vec::new();
    let mut snapshots = Vec::new();
    let mut automatic = false;
    let mut first_capture = Datetime::default();
    let mut expires_at: Option<Datetime> = None;
    for source in sources {
        let source = visible(service, source).await?.context("Summary source unavailable")?;
        anyhow::ensure!(source.user_id.as_deref() == Some(owner), "Summary source owner mismatch");
        let mid = source.id.context("Summary source has no ID")?;
        lineage.push(mid.clone());
        let mut snapshot = SourceSnapshot { mid: mid.clone(), cid: control_id(&mid)?, version: source.version, expires_at: None };
        if let Some(control) = read_control(service, &mid).await? {
            snapshot.expires_at = control.expires_at.clone();
            automatic |= control.automatic.unwrap_or(false);
            if let Some(anchor) = control.first_capture { if anchor < first_capture { first_capture = anchor; } }
            if let Some(expiry) = control.expires_at { if expires_at.as_ref().is_none_or(|prior| &expiry < prior) { expires_at = Some(expiry); } }
            lineage.extend(control.sources.unwrap_or_default());
        }
        snapshots.push(snapshot);
        if let Some(expiry) = source.valid_until { if expires_at.as_ref().is_none_or(|prior| &expiry < prior) { expires_at = Some(expiry); } }
    }
    summary.embedding = Some(service.embeddings.embed(&summary.content).await?);
    summary.token_count = Some(u32::try_from(summary.content.len().div_ceil(4)).unwrap_or(u32::MAX));
    let row = MemoryRow::from(summary);
    let key = uuid::Uuid::new_v4().to_string();
    let mid = RecordId::new("memory", key.clone());
    let mut lineage_controls = Vec::new();
    for source in &lineage { lineage_controls.push(control_id(source)?); }
    db(service)?.query("BEGIN TRANSACTION;
        LET $pid=type::record('uar_memory_policy',$owner); LET $policy=(SELECT * FROM $pid)[0];
        IF $policy != NONE { UPDATE $pid SET fence=rand::uuid(); };
        FOR $source IN $snapshots {
            LET $m=(SELECT * FROM $source.mid)[0]; LET $c=(SELECT * FROM $source.cid)[0];
            IF $m=NONE OR $m.user_id != $owner OR $m.version != $source.version
                OR ($m.valid_until != NONE AND $m.valid_until <= time::now())
                OR ($c != NONE AND ($c.deleted OR $c.expires_at != $source.expires_at OR ($c.expires_at != NONE AND $c.expires_at <= time::now()))) { THROW 'uar_memory_summary_source_changed'; };
        };
        FOR $cid IN $lineage_controls {
            LET $c=(SELECT * FROM $cid)[0];
            IF $c != NONE {
                IF ($c.deleted AND $c.reason != 'compressed') OR ($c.expires_at != NONE AND $c.expires_at <= time::now()) { THROW 'uar_memory_summary_source_changed'; };
                UPDATE $cid SET fence=rand::uuid();
            };
        };
        LET $duration=IF $policy.days=7 { 7d } ELSE IF $policy.days=30 { 30d } ELSE IF $policy.days=90 { 90d } ELSE { 365d };
        LET $bound=IF $automatic AND $policy.days != NONE AND ($expires=NONE OR $anchor+$duration < $expires) { $anchor+$duration } ELSE { $expires };
        IF $bound != NONE AND $bound <= time::now() { THROW 'uar_memory_summary_source_changed'; };
        CREATE $mid CONTENT $memory;
        IF $stream != NONE { UPDATE $stream SET total_tokens += $tokens, summary_count += 1, last_active=time::now(); };
        CREATE $summary_control SET memory_id=$mid, owner=$owner, automatic=$automatic, first_capture=$anchor,
            expires_at=$bound, policy_revision=$policy.revision, deleted=false, purged=false, sources=$lineage;
        CREATE memory_history SET memory_id=$mid, version=1, old_content=NONE, new_content=$content, changed_at=time::now(), change_type='created';
        FOR $source IN $snapshots {
            UPSERT $source.cid SET memory_id=$source.mid, owner=$owner, deleted=true, purged=false, reason='compressed', deleted_at=time::now();
            UPDATE $source.mid SET updated_at=time::now();
        };
        COMMIT TRANSACTION;")
        .bind(("owner", owner.to_owned())).bind(("snapshots", snapshots))
        .bind(("lineage_controls", lineage_controls)).bind(("lineage", lineage))
        .bind(("automatic", automatic)).bind(("anchor", first_capture)).bind(("expires", expires_at))
        .bind(("mid", mid.clone())).bind(("summary_control", control_id(&mid)?))
        .bind(("stream", row.task_stream_id.clone())).bind(("tokens", row.token_count))
        .bind(("content", row.content.clone())).bind(("memory", row)).await?.check()?;
    match service.storage.get_memory(&key).await? { Some(memory) => visible(service, memory).await, None => Ok(None) }
}

#[derive(serde::Serialize, serde::Deserialize, SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct SourceSnapshot { mid: RecordId, cid: RecordId, version: u32, expires_at: Option<Datetime> }

#[derive(serde::Deserialize, SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct ExpiredRow { id: RecordId, user_id: Option<String> }

#[derive(serde::Deserialize, SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct ControlRow {
    memory_id: RecordId, owner: String, automatic: Option<bool>, first_capture: Option<Datetime>,
    expires_at: Option<Datetime>, policy_revision: Option<u64>, retention_days: Option<u16>,
    expired: bool, deleted: bool, purged: bool, reason: Option<String>, sources: Option<Vec<RecordId>>,
}
pub(super) async fn status(service: &MemoryService, owner: &str, id: &str) -> Result<Option<serde_json::Value>> {
    let mid = memory_id(id)?;
    Ok(read_control(service, &mid).await?.filter(|control| control.owner == owner).map(|control| serde_json::json!({
        "id": id, "deleted": control.deleted, "expired": control.expired, "purged": control.purged,
        "reason": control.reason, "automatic": control.automatic.unwrap_or(false),
        "first_capture": control.first_capture, "effective_expires_at": control.expires_at,
        "policy_revision": control.policy_revision,
    })))
}
async fn read_control(service: &MemoryService, mid: &RecordId) -> Result<Option<ControlRow>> {
    let rows: Vec<ControlRow> = db(service)?.query("SELECT *, (expires_at != NONE AND expires_at <= time::now()) AS expired FROM $cid")
        .bind(("cid", control_id(mid)?)).await?.check()?.take(0)?;
    Ok(rows.into_iter().next())
}

// The pinned SDK's DbMemory is private. Match its documented schema/string
// enums rather than bind Memory's externally tagged SurrealValue enums.
#[derive(serde::Serialize, serde::Deserialize, SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct MemoryRow {
    content: String, embedding: Option<Vec<f32>>, scope: String, memory_type: String,
    user_id: Option<String>, session_id: Option<String>, agent_id: Option<String>,
    task_stream_id: Option<RecordId>, categories: Vec<String>, metadata: Option<serde_json::Value>,
    token_count: Option<u32>, importance: f32, access_count: u32, last_accessed_at: Option<Datetime>,
    valid_until: Option<Datetime>, version: u32, created_at: Datetime, updated_at: Datetime,
}
impl From<Memory> for MemoryRow {
    fn from(memory: Memory) -> Self {
        Self { content:memory.content, embedding:memory.embedding, scope:memory.scope.as_str().into(),
            memory_type:memory.memory_type.as_str().into(), user_id:memory.user_id, session_id:memory.session_id,
            agent_id:memory.agent_id, task_stream_id:memory.task_stream_id, categories:memory.categories,
            metadata:memory.metadata, token_count:memory.token_count, importance:memory.importance,
            access_count:0, last_accessed_at:None, valid_until:memory.valid_until, version:1,
            created_at:Datetime::default(), updated_at:Datetime::default() }
    }
}
