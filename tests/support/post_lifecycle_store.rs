//! Real-store collaborators for the reviewed post-lifecycle main packet.
//! Authoring this module does not authorize store access or verification.
#![cfg(feature = "surreal-backend")]

use std::{fs::OpenOptions, io::Write, path::Path, sync::Arc, time::Duration};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use surrealdb::{Surreal, engine::any::{self, Any}, opt::auth::Root};
use tokio::sync::Barrier;
use universal_agent_runtime::uar::persistence::{PersistenceLayer,
    providers::surreal::SurrealDbProvider, agui_runs::{AguiRunBinding, AguiTerminal}};

type Result<T> = std::result::Result<T, String>;
const DEADLINE: Duration = Duration::from_secs(20);

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value[key].as_str().filter(|value| !value.is_empty()).ok_or_else(|| format!("missing:{key}"))
}

fn receipt(path: &Path, value: &Value) -> Result<()> {
    let mut output = OpenOptions::new().create_new(true).write(true).open(path)
        .map_err(|_| "store_receipt_create_failed".to_owned())?;
    output.write_all(value.to_string().as_bytes()).map_err(|_| "store_receipt_write_failed".to_owned())
}

fn settings(manifest: &Value) -> Result<(&str, &str, &str, String, String)> {
    text(manifest, "storeFixtureApprovalRef")?;
    let store = &manifest["store"];
    let endpoint = text(store, "endpoint")?;
    if !endpoint.starts_with("ws://127.0.0.1:") || endpoint.contains('@') {
        return Err("unchanged_loopback_store_required".into());
    }
    let namespace = text(store, "namespace")?;
    let database = text(store, "database")?;
    if !namespace.starts_with("uar_post_lifecycle_") || !database.starts_with("uar_post_lifecycle_") {
        return Err("dedicated_fixture_namespace_required".into());
    }
    let user = std::env::var(text(store, "userEnv")?).map_err(|_| "existing_store_user_unavailable")?;
    let password = std::env::var(text(store, "passwordEnv")?).map_err(|_| "existing_store_password_unavailable")?;
    Ok((endpoint, namespace, database, user, password))
}

fn error_kind(error: &anyhow::Error) -> Value {
    for cause in error.chain() {
        if let Some(error) = cause.downcast_ref::<surrealdb::Error>() {
            return json!({"storageError":true,"kind":error.kind_str(),
                "transactionConflict":matches!(error.query_details(),Some(surrealdb::types::QueryError::TransactionConflict))});
        }
    }
    json!({"storageError":false,"kind":"application_error","transactionConflict":false})
}

async fn raw_client(manifest: &Value) -> Result<Surreal<Any>> {
    let (endpoint, namespace, database, user, password) = settings(manifest)?;
    let db = tokio::time::timeout(DEADLINE, any::connect(endpoint)).await
        .map_err(|_| "raw_store_connect_timeout")?.map_err(|_| "raw_store_connect_failed")?;
    let initialize = async {
        db.signin(Root { username:user, password }).await.map_err(|_| "existing_store_auth_failed")?;
        db.use_ns(namespace).use_db(database).await.map_err(|_| "fixture_scope_selection_failed")?;
        Ok::<(),String>(())
    };
    match tokio::time::timeout(DEADLINE, initialize).await {
        Ok(Ok(())) => Ok(db),
        result => {
            let _ = db.shutdown().await;
            match result { Ok(Err(error)) => Err(error), _ => Err("raw_store_initialize_timeout".into()) }
        }
    }
}

async fn read_row(db: &Surreal<Any>, table: &str, key: &str) -> Result<Option<surrealdb::types::Value>> {
    let mut response = db.query("SELECT * FROM type::record($table,$key)")
        .bind(("table", table.to_owned())).bind(("key", key.to_owned()))
        .await.map_err(|_| "exact_row_read_transport_failed")?.check().map_err(|_| "exact_row_read_query_failed")?;
    let rows: Vec<surrealdb::types::Value> = response.take(0).map_err(|_| "exact_row_decode_failed")?;
    if rows.len() > 1 { return Err("exact_row_read_multiple_records".into()); }
    Ok(rows.into_iter().next())
}

/// A fixture-only synchronous schema event widens the real transaction window
/// by touching a different owned context row before a bounded SLEEP. A bounded
/// competing writer targets only that other row. Scheduling is observed, never
/// assumed: no conflict seen is BLOCKED, not PASS.
async fn unrelated_conflict_schedule(provider: &SurrealDbProvider, db: &Surreal<Any>, seed: &AguiRunBinding,
    events: &mut Vec<Value>, owned: &mut Vec<Value>) -> Result<()> {
    let mut before = seed.clone(); before.thread_id.push_str("-unrelated");
    if !provider.create_agui_run(&before,None).await.map_err(|_| "unrelated_cas_seed_failed")? {
        return Err("unrelated_cas_seed_not_fresh".into());
    }
    owned.push(json!({"table":"agui_runs","key":before.storage_key(),"match":{
        "owner_key":before.owner,"agent_id":before.agent_id,"thread_id":before.thread_id}}));
    owned.push(json!({"table":"agui_contexts","key":before.context_key(),"match":{"run_key":before.storage_key()}}));
    let suffix = format!("{:x}",Sha256::digest(before.storage_key().as_bytes()));
    let contention = format!("uar_post_lifecycle_contention_{suffix}");
    let event_name = format!("uar_post_lifecycle_{}",&suffix[..16]);
    db.query("CREATE type::record('agui_contexts',$key) SET run_key='fixture-initial'")
        .bind(("key",contention.clone())).await.map_err(|_| "contention_seed_transport_failed")?
        .check().map_err(|_| "contention_seed_query_failed")?;
    // Identifier consists only of a constant and SHA hex. JSON string quoting
    // supplies literal record-key strings; no retained query binding is assumed.
    let row_literal = serde_json::to_string(&before.storage_key()).map_err(|_| "fixture_literal_failed")?;
    let contention_literal = serde_json::to_string(&contention).map_err(|_| "fixture_literal_failed")?;
    let define = format!("DEFINE EVENT {event_name} ON TABLE agui_runs WHEN $event = 'UPDATE' AND $after.id = type::record('agui_runs',{row_literal}) THEN {{ UPDATE type::record('agui_contexts',{contention_literal}) SET run_key='fixture-event'; SLEEP 100ms; }};");
    events.push(json!({"kind":"fixture_schema_event_intent","event":event_name,"table":"agui_runs",
        "ownedTarget":before.storage_key(),"otherOwnedRow":contention}));
    db.query(define).await.map_err(|_| "contention_event_define_transport_failed")?
        .check().map_err(|_| "contention_event_define_query_failed")?;
    events.push(json!({"kind":"fixture_schema_event_defined","event":event_name,"table":"agui_runs",
        "ownedTarget":before.storage_key(),"otherOwnedRow":contention,"sleepMilliseconds":100}));
    let mut after = before.clone(); after.revision = 2;
    after.terminal = Some(AguiTerminal {source_cursor:3,event:json!({"fixture":"unrelated-conflict"}),interrupt_ids:Vec::new()});
    let work = async {
        let barrier = Barrier::new(2);
        let transition = async { barrier.wait().await; provider.compare_and_swap_agui_run(&before,&after).await };
        let competitor = async {
            barrier.wait().await;
            let mut observations = Vec::new();
            for attempt in 0..12 {
                let result = db.query("UPDATE type::record('agui_contexts',$key) SET run_key=$marker")
                    .bind(("key",contention.clone())).bind(("marker",format!("fixture-competitor-{attempt}"))).await;
                observations.push(match result {
                    Ok(response) => match response.check() {
                        Ok(_) => json!({"attempt":attempt,"ok":true}),
                        Err(error) => json!({"attempt":attempt,"ok":false,"kind":error.kind_str(),
                            "transactionConflict":matches!(error.query_details(),Some(surrealdb::types::QueryError::TransactionConflict))}),
                    },
                    Err(error) => json!({"attempt":attempt,"ok":false,"kind":error.kind_str()}),
                });
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            observations
        };
        let (transition,competitor) = tokio::join!(transition,competitor);
        let observed = provider.load_agui_run(&before.owner,&before.agent_id,&before.thread_id,&before.wire_run_id)
            .await.map_err(|_| "unrelated_cas_readback_failed")?;
        let unchanged = observed.as_ref()==Some(&before);
        let result = match &transition { Ok(value) => json!({"ok":value}),Err(error) => error_kind(error) };
        let classification = match &transition {
            Ok(false) if unchanged => "FAIL_UNRELATED_CONFLICT_REPORTED_AS_CAS_MISS",
            Err(error) if unchanged && error_kind(error)["transactionConflict"]==true => "PASS_UNRELATED_TRANSACTION_CONFLICT_RETAINED",
            _ => "BLOCKED_UNRELATED_TRANSACTION_CONFLICT_NOT_OBSERVED",
        };
        events.push(json!({"kind":"unrelated_adapter_contention","transition":result,"exactTargetUnchanged":unchanged,
            "competitor":competitor,"classification":classification,"internalInterleavingGuaranteed":false,
            "scope":"actual adapter plus distinct real-store row; no synthetic provider error"}));
        Ok::<(),String>(())
    };
    let result = tokio::time::timeout(Duration::from_secs(30),work).await
        .unwrap_or_else(|_| Err("unrelated_contention_timeout_outcome_unknown".into()));
    let remove = match db.query(format!("REMOVE EVENT {event_name} ON TABLE agui_runs")).await {
        Ok(response) => response.check().map(|_| ()).map_err(|_| "contention_event_remove_query_failed"),
        Err(_) => Err("contention_event_remove_transport_failed"),
    };
    events.push(json!({"kind":"fixture_schema_event_remove","event":event_name,"removed":remove.is_ok()}));
    // Ownership of a failed removal is retained explicitly; never remove a
    // differently named event or treat a transport failure as successful cleanup.
    if remove.is_err() { return Err("fixture_schema_event_cleanup_unconfirmed".into()); }
    if let Some(row) = read_row(db,"agui_contexts",&contention).await? {
        let value = universal_agent_runtime::uar::persistence::providers::surreal::value_to_json(row)
            .map_err(|_| "contention_cleanup_snapshot_failed")?;
        owned.push(json!({"table":"agui_contexts","key":contention,"match":{"run_key":value["run_key"]}}));
    }
    result
}

/// Exercise concurrent public AG-UI CAS calls and retain actual outcomes. The
/// shared barrier proves concurrent attempts, not an internal DB interleaving.
/// Without fault instrumentation the forced-interleaving claim stays BLOCKED.
pub async fn cas_discriminator(manifest: &Value, evidence: &Path) -> Result<Value> {
    let (endpoint, namespace, database, user, password) = settings(manifest)?;
    let spec = &manifest["casFixture"];
    let owner = text(spec, "owner")?;
    let prefix = text(spec, "fixtureId")?;
    if spec["unrelatedConflictSchedule"]==true {
        let declared = manifest["fixtureSchemaEffects"].as_array().is_some_and(|effects| effects.iter().any(|effect|
            effect["kind"]=="synchronous_event" && effect["table"]=="agui_runs"
            && effect["purpose"]=="unrelated_transaction_conflict"
            && effect["sleepMilliseconds"]==100 && effect["competitorWritesMax"]==12
            && effect["cleanup"]=="remove_exact_owned_event"));
        if !declared { return Err("reviewed_fixture_schema_effects_not_declared".into()); }
    }
    let provider = tokio::time::timeout(DEADLINE,
        SurrealDbProvider::new(endpoint,Some(&user),Some(&password),Some(namespace),Some(database))).await
        .map_err(|_| "cas_provider_open_timeout")?.map_err(|_| "cas_provider_open_failed")?;
    let mut events = Vec::new();
    let mut owned = Vec::new();
    let body = async {
        if !provider.durable_store_eligibility().permits_durable_work() {
            return Err("store_durability_unproven_no_substitution".into());
        }
        let db = raw_client(manifest).await?;
        let operations = async {
            for ordering in ["a-first", "b-first"] {
                let before = AguiRunBinding { owner:owner.into(),agent_id:format!("{prefix}-agent"),
                    thread_id:format!("{prefix}-{ordering}"),wire_run_id:format!("{prefix}-run"),
                    native_run_id:format!("{prefix}-native"),session_id:format!("{prefix}-session"),
                    workspace_id:None,input_digest:format!("{:x}",Sha256::digest(prefix.as_bytes())),
                    ordinal:1,revision:1,start_cursor:0,previous_wire_run_id:None,resumed_tool_ids:Vec::new(),terminal:None };
                if !provider.create_agui_run(&before,None).await.map_err(|_| "cas_seed_failed")? {
                    return Err("cas_seed_not_fresh".into());
                }
                owned.push(json!({"table":"agui_runs","key":before.storage_key(),"match":{
                    "owner_key":owner,"agent_id":before.agent_id,"thread_id":before.thread_id}}));
                owned.push(json!({"table":"agui_contexts","key":before.context_key(),"match":{"run_key":before.storage_key()}}));
                let mut first = before.clone(); first.revision = 2;
                first.terminal = Some(AguiTerminal {source_cursor:1,event:json!({"fixtureWinner":"a"}),interrupt_ids:Vec::new()});
                let mut second = first.clone();
                second.terminal = Some(AguiTerminal {source_cursor:2,event:json!({"fixtureWinner":"b"}),interrupt_ids:Vec::new()});
                let barrier = Arc::new(Barrier::new(2));
                let a = async { barrier.wait().await; provider.compare_and_swap_agui_run(&before,&first).await };
                let b = async { barrier.wait().await; provider.compare_and_swap_agui_run(&before,&second).await };
                let (a,b) = if ordering == "a-first" { tokio::join!(a,b) } else {
                    let (b,a) = tokio::join!(b,a); (a,b)
                };
                let row = provider.load_agui_run(owner,&before.agent_id,&before.thread_id,&before.wire_run_id)
                    .await.map_err(|_| "cas_exact_readback_failed")?;
                let a_result = match &a { Ok(value) => json!({"ok":value}), Err(error) => error_kind(error) };
                let b_result = match &b { Ok(value) => json!({"ok":value}), Err(error) => error_kind(error) };
                let matched_winner = (matches!(a,Ok(true)) && matches!(b,Ok(false)) && row.as_ref()==Some(&first))
                    || (matches!(b,Ok(true)) && matches!(a,Ok(false)) && row.as_ref()==Some(&second));
                events.push(json!({"kind":"concurrent_cas","launchOrdering":ordering,"a":a_result,"b":b_result,
                    "exactReadbackMatched":matched_winner,"rowKey":before.storage_key(),
                    "expectedOrderingObserved":if ordering=="a-first" {matches!(a,Ok(true))} else {matches!(b,Ok(true))},
                    "winnerRevision":row.as_ref().map(|row|row.revision),"internalInterleavingGuaranteed":false}));
                if !matched_winner { return Err("concurrent_cas_outcome_unresolved".into()); }
                if provider.load_agui_run("different-owner",&before.agent_id,&before.thread_id,&before.wire_run_id)
                    .await.map_err(|_| "cas_foreign_read_failed")?.is_some() { return Err("cas_foreign_row_visible".into()); }
                // This reaches the real DB schema, not validate_transition().
                // CANCEL prevents mutation even if a changed schema accepts it.
                let response = db.query("BEGIN TRANSACTION; UPDATE type::record('agui_runs',$key) SET owner_key=NONE; CANCEL TRANSACTION;")
                    .bind(("key",before.storage_key())).await.map_err(|_| "storage_control_transport_failed")?;
                let observed = match response.check() {
                    Ok(_) => json!({"storageError":false,"reason":"schema_control_not_rejected"}),
                    Err(error) => json!({"storageError":true,"kind":error.kind_str(),
                        "transactionConflict":matches!(error.query_details(),Some(surrealdb::types::QueryError::TransactionConflict))}),
                };
                let unchanged = provider.load_agui_run(owner,&before.agent_id,&before.thread_id,&before.wire_run_id)
                    .await.map_err(|_| "storage_control_readback_failed")? == row;
                events.push(json!({"kind":"unrelated_database_schema_error_control","observed":observed,
                    "rowUnchanged":unchanged,"scope":"actual raw backend query; not adapter transaction-conflict injection"}));
                if !unchanged { return Err("storage_error_control_changed_owned_row".into()); }
                // This ancillary control cannot substitute for, or prevent the
                // separate actual adapter transaction-contention experiment.
                if observed["storageError"]!=true || observed["transactionConflict"]==true {
                    events.push(json!({"kind":"schema_control_disposition","outcome":"BLOCKED",
                        "reason":"unrelated_schema_error_not_observed"}));
                }
            }
            if spec["unrelatedConflictSchedule"]==true {
                let before = AguiRunBinding { owner:owner.into(),agent_id:format!("{prefix}-agent"),
                    thread_id:format!("{prefix}-schedule"),wire_run_id:format!("{prefix}-run"),
                    native_run_id:format!("{prefix}-native"),session_id:format!("{prefix}-session"),
                    workspace_id:None,input_digest:format!("{:x}",Sha256::digest(prefix.as_bytes())),
                    ordinal:1,revision:1,start_cursor:0,previous_wire_run_id:None,resumed_tool_ids:Vec::new(),terminal:None };
                unrelated_conflict_schedule(&provider,&db,&before,&mut events,&mut owned).await?;
            }
            Ok::<(),String>(())
        };
        let result = tokio::time::timeout(Duration::from_secs(100),operations).await
            .unwrap_or_else(|_| Err("cas_operations_timeout_outcome_unknown".into()));
        let closed = db.shutdown().await.is_ok();
        events.push(json!({"kind":"raw_client_close","scopeReleased":closed}));
        if !closed { return Err("raw_store_close_unconfirmed".into()); }
        result
    };
    let result: Result<()> = body.await;
    let closed = provider.close_scope().await;
    let unrelated_failed = events.iter().any(|event| event["classification"]=="FAIL_UNRELATED_CONFLICT_REPORTED_AS_CAS_MISS");
    let retained_schema: Vec<Value> = events.iter().filter(|event| event["kind"]=="fixture_schema_event_intent")
        .filter(|intent| !events.iter().any(|event| event["kind"]=="fixture_schema_event_remove"
            && event["event"]==intent["event"] && event["removed"]==true))
        .map(|event| json!({"table":"agui_runs","name":event["event"],"outcome":"retained_or_uncertain"})).collect();
    let exact_orderings = events.iter().filter(|event| event["kind"]=="concurrent_cas"
        && event["exactReadbackMatched"]==true && event["expectedOrderingObserved"]==true).count()==2;
    let unrelated_preserved = events.iter().any(|event|
        event["classification"]=="PASS_UNRELATED_TRANSACTION_CONFLICT_RETAINED");
    let raw_closed = events.iter().any(|event| event["kind"]=="raw_client_close" && event["scopeReleased"]==true);
    let passed = result.is_ok() && exact_orderings && unrelated_preserved && raw_closed
        && closed.scope_released && retained_schema.is_empty();
    let reason = result.err().unwrap_or_else(|| if passed {"observed_exact_orderings_and_unrelated_transaction_conflict_preserved".into()}
        else {"required_fault_or_ordering_not_observed_or_cleanup_unconfirmed_see_events".into()});
    let report = json!({"outcome":if unrelated_failed {"FAIL"} else if passed {"PASS"} else {"BLOCKED"},"reason":reason,
        "events":events,"ownedRows":owned,"retainedSchemaEvents":retained_schema,"providerScopeReleased":closed.scope_released,
        "claims":{"publicConcurrentAttempts":true,"forcedInternalInterleaving":false,
            "actualRequiredOrderingsObserved":exact_orderings,"actualUnrelatedConflictPreserved":unrelated_preserved,
            "unrelatedRawSchemaErrorIsNotAdapterConflictProof":true}});
    receipt(&evidence.join("cas-store-collaborator.json"),&report)?;
    Ok(report)
}

fn match_owned(row: &Value, entry: &Value) -> Result<()> {
    let table = text(entry,"table")?;
    let expected = entry["match"].as_object().filter(|value| !value.is_empty()).ok_or("cleanup_ownership_match_missing")?;
    let required: &[&str] = match table {
        "agui_runs" => &["owner_key","agent_id","thread_id"],
        "agui_contexts" => &["run_key"],
        "accounting_calls" => &["owner_key","run_id"],
        "accounting_scopes" => &["data"],
        "durable_run_records" => &["owner_key","run_id"],
        "durable_run_events" => &["run_id","event_id"],
        "knowledge_bases" | "knowledge_documents" | "knowledge_chunks"
        | "principal_knowledge_bases" | "principal_knowledge_documents" | "principal_knowledge_chunks" => &["owner_id"],
        "knowledge_ingestion" | "principal_knowledge_ingestion" => &["data"],
        _ => return Err("cleanup_table_not_in_owned_fixture_allowlist".into()),
    };
    if required.iter().any(|name| !expected.contains_key(*name)) {
        return Err("cleanup_required_owner_fields_missing".into());
    }
    for (name,value) in expected {
        if value.is_null() || value.as_str()==Some("") || row.get(name)!=Some(value) {
            return Err("cleanup_exact_ownership_mismatch".into());
        }
    }
    Ok(())
}

/// Remove only individually reviewed table/key records after main is stopped.
/// Every row's ownership fields and full current snapshot are rechecked inside
/// the delete transaction; a changed row is retained with BLOCKED evidence.
pub async fn cleanup_rows(manifest: &Value, evidence: &Path) -> Result<Value> {
    if manifest["cleanupAuthorized"]!=true { return Err("exact_row_cleanup_not_authorized".into()); }
    if manifest["ownedMainStopped"]!=true { return Err("owned_main_exit_not_confirmed".into()); }
    let entries = manifest["cleanupOwnedRows"].as_array().ok_or("cleanup_owned_rows_missing")?;
    let db = raw_client(manifest).await?;
    let mut events = Vec::new();
    let work = async {
        for entry in entries {
            let table = text(entry,"table")?;
            let key = text(entry,"key")?;
            // Validate table and required ownership fields even for absent rows.
            match_owned(&entry["match"],entry)?;
            let Some(before) = read_row(&db,table,key).await? else {
                events.push(json!({"table":table,"key":key,"outcome":"already_absent"})); continue;
            };
            let decoded = universal_agent_runtime::uar::persistence::providers::surreal::value_to_json(before.clone())
                .map_err(|_| "cleanup_snapshot_decode_failed")?;
            match_owned(&decoded,entry)?;
            db.query("BEGIN TRANSACTION; LET $current=(SELECT * FROM type::record($table,$key))[0]; IF $current != $before { THROW 'post_lifecycle_owned_row_changed'; }; DELETE type::record($table,$key); COMMIT TRANSACTION;")
                .bind(("table",table.to_owned())).bind(("key",key.to_owned())).bind(("before",before))
                .await.map_err(|_| "cleanup_delete_transport_failed")?.check().map_err(|_| "cleanup_delete_query_failed_row_retained_or_unknown")?;
            if read_row(&db,table,key).await?.is_some() { return Err("cleanup_row_still_present".into()); }
            events.push(json!({"table":table,"key":key,"outcome":"deleted_exact_owned_row","absenceObserved":true}));
        }
        Ok::<(),String>(())
    };
    let result = tokio::time::timeout(Duration::from_secs(120),work).await
        .unwrap_or_else(|_| Err("cleanup_timeout_remaining_ownership_retained".into()));
    let closed = db.shutdown().await.is_ok();
    let success = result.is_ok() && closed;
    let report = json!({"outcome":if success {"PASS"} else {"BLOCKED"},"events":events,
        "reason":result.err(),"rawScopeReleased":closed,"namespaceOrDatabaseDeleted":false,
        "retainedOwnership":!success,"declaredRowCount":entries.len()});
    receipt(&evidence.join("store-exact-row-cleanup.json"),&report)?;
    Ok(report)
}

/// Enumerate exact history records for actual main-run IDs already captured by
/// the caller. The durable record owner must match the existing manifest owner
/// before any related event row can enter the cleanup ownership inventory.
pub async fn history_cleanup_rows(manifest: &Value, run_ids: &[String]) -> Result<Vec<Value>> {
    let expected_owner = text(&manifest["fixtureVariables"],"ownerStorageKey")?;
    let db = raw_client(manifest).await?;
    let work = async {
        let mut owned = Vec::new();
        for run in run_ids {
            if run.is_empty() { return Err("history_cleanup_run_id_missing".into()); }
            let record_key = format!("{}:{run}",run.len());
            let record = read_row(&db,"durable_run_records",&record_key).await?
                .ok_or("history_cleanup_record_owner_proof_missing")?;
            let record = universal_agent_runtime::uar::persistence::providers::surreal::value_to_json(record)
                .map_err(|_| "history_cleanup_record_decode_failed")?;
            if record["owner_key"].as_str()!=Some(expected_owner) || record["run_id"].as_str()!=Some(run.as_str()) {
                return Err("history_cleanup_record_owner_mismatch".into());
            }
            let mut response = db.query("SELECT * FROM durable_run_events WHERE run_id=$run ORDER BY event_id ASC")
                .bind(("run",run.clone())).await.map_err(|_| "history_cleanup_events_transport_failed")?
                .check().map_err(|_| "history_cleanup_events_query_failed")?;
            let events: Vec<surrealdb::types::Value> = response.take(0).map_err(|_| "history_cleanup_events_decode_failed")?;
            for event in events {
                let event = universal_agent_runtime::uar::persistence::providers::surreal::value_to_json(event)
                    .map_err(|_| "history_cleanup_event_decode_failed")?;
                if event["run_id"].as_str()!=Some(run.as_str()) { return Err("history_cleanup_event_scope_mismatch".into()); }
                let id = event["event_id"].as_u64().ok_or("history_cleanup_event_id_invalid")?;
                owned.push(json!({"table":"durable_run_events","key":format!("{record_key}:{id}"),
                    "match":{"run_id":run,"event_id":id}}));
            }
            owned.push(json!({"table":"durable_run_records","key":record_key,
                "match":{"owner_key":expected_owner,"run_id":run}}));
        }
        Ok::<Vec<Value>,String>(owned)
    };
    let result = tokio::time::timeout(Duration::from_secs(60),work).await
        .unwrap_or_else(|_| Err("history_cleanup_enumeration_timeout".into()));
    if db.shutdown().await.is_err() { return Err("history_cleanup_client_close_unconfirmed".into()); }
    result
}
