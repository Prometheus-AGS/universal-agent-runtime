//! Offline storage setup/inspection for the genuine Boss/UAR acceptance gate.
//! This libtest entry is a utility, not a substitute runtime or acceptance test.

#![cfg(feature = "surreal-backend")]

use std::{collections::BTreeMap, path::PathBuf};

use serde_json::{Value, json};
use universal_agent_runtime::uar::persistence::providers::surreal::value_to_json;

const MARKER: &str = ".bauar-native-storage-fixture.json";
const STATES: [&str; 9] = [
    "AwaitingApproval", "ClaimIntent", "Succeeded", "Failed", "Denied",
    "Cancelled", "Invalidated", "Interrupted", "OutcomeUnknown",
];
type FixtureResult<T> = Result<T, &'static str>;

fn setting(name: &str) -> FixtureResult<String> {
    std::env::var(name).map_err(|_| "missing_setting")
}

fn owned_store(seed: bool) -> FixtureResult<PathBuf> {
    let root = PathBuf::from(setting("BAUAR_STORAGE_FIXTURE_ROOT")?);
    if !root.is_absolute() || !root.is_dir() {
        return Err("invalid_root");
    }
    let canonical = root.canonicalize().map_err(|_| "invalid_root")?;
    if canonical != root {
        return Err("root_must_be_canonical");
    }
    let marker = root.join(MARKER);
    let metadata = std::fs::symlink_metadata(&marker).map_err(|_| "ownership_required")?;
    if !metadata.is_file() || metadata.len() > 256 {
        return Err("ownership_required");
    }
    let marker: Value = serde_json::from_slice(
        &std::fs::read(marker).map_err(|_| "ownership_required")?,
    ).map_err(|_| "ownership_required")?;
    if marker != json!({"version": 1, "purpose": "bauar-native-admission-storage-fixture"}) {
        return Err("ownership_required");
    }
    let store = root.join("runtime.db");
    match std::fs::symlink_metadata(&store) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && seed => {}
        Ok(metadata) if !seed && metadata.is_dir() => {}
        _ => return Err("store_freshness_required"),
    }
    Ok(store)
}

async fn run_fixture() -> FixtureResult<Value> {
    let mode = setting("BAUAR_STORAGE_FIXTURE_MODE")?;
    let assertion = match mode.as_str() {
        "seed-claim-intent" => Some("$value != 'ClaimIntent'"),
        "seed-terminal" => Some("$value != 'Succeeded' AND $value != 'Failed'"),
        "inspect" => None,
        _ => return Err("unsupported_mode"),
    };
    let store = owned_store(assertion.is_some())?;
    let namespace = setting("BAUAR_STORAGE_FIXTURE_NS")?;
    let database = setting("BAUAR_STORAGE_FIXTURE_DB")?;
    if namespace != "uar" || database != "uar" {
        return Err("unexpected_database_scope");
    }
    let endpoint = format!("surrealkv://{}", store.display());
    let db = surrealdb::engine::any::connect(&endpoint)
        .await.map_err(|_| "store_open_failed")?;
    db.use_ns(namespace).use_db(database)
        .await.map_err(|_| "database_selection_failed")?;
    if let Some(assertion) = assertion {
        // Only these fixed expressions enter SQL; no gate-provided SQL is accepted.
        db.query(format!(
            "DEFINE TABLE IF NOT EXISTS tool_admission_evidence SCHEMALESS; \
             DEFINE FIELD state ON TABLE tool_admission_evidence TYPE string ASSERT {assertion};"
        ))
            .await.map_err(|_| "schema_seed_failed")?
            .check().map_err(|_| "schema_seed_failed")?;
    }
    let mut info = db.query("INFO FOR TABLE tool_admission_evidence")
        .await.map_err(|_| "schema_inspection_failed")?
        .check().map_err(|_| "schema_inspection_failed")?;
    let info: Option<surrealdb::types::Value> = info.take(0)
        .map_err(|_| "schema_decode_failed")?;
    let info = value_to_json(info.ok_or("schema_missing")?)
        .map_err(|_| "schema_decode_failed")?;
    let definition = info.get("fields").and_then(|fields| fields.get("state"))
        .and_then(Value::as_str).ok_or("assertion_missing")?;
    let assertion_kind = if !definition.contains("ASSERT") || !definition.contains("!=") {
        return Err("assertion_unrecognized");
    } else if definition.contains("ClaimIntent")
        && !definition.contains("Succeeded") && !definition.contains("Failed") {
        "claim-intent"
    } else if definition.contains("Succeeded") && definition.contains("Failed")
        && !definition.contains("ClaimIntent") {
        "terminal"
    } else {
        return Err("assertion_unrecognized");
    };
    if (mode == "seed-claim-intent" && assertion_kind != "claim-intent")
        || (mode == "seed-terminal" && assertion_kind != "terminal") {
        return Err("assertion_mismatch");
    }
    let mut rows = db.query(
        "SELECT state, count() AS count FROM tool_admission_evidence GROUP BY state"
    ).await.map_err(|_| "count_query_failed")?
        .check().map_err(|_| "count_query_failed")?;
    let rows: Vec<surrealdb::types::Value> = rows.take(0)
        .map_err(|_| "count_decode_failed")?;
    let mut counts: BTreeMap<&str, u64> = STATES.into_iter().map(|state| (state, 0)).collect();
    counts.insert("Other", 0);
    let mut total = 0_u64;
    for row in rows {
        let row = value_to_json(row).map_err(|_| "count_decode_failed")?;
        let state = row.get("state").and_then(Value::as_str).ok_or("count_decode_failed")?;
        let count = row.get("count").and_then(Value::as_u64).ok_or("count_decode_failed")?;
        let key = STATES.iter().copied().find(|known| *known == state).unwrap_or("Other");
        let entry = counts.get_mut(key).ok_or("count_decode_failed")?;
        *entry = entry.checked_add(count).ok_or("count_overflow")?;
        total = total.checked_add(count).ok_or("count_overflow")?;
    }
    if assertion.is_some() && total != 0 {
        return Err("seed_store_not_empty");
    }
    // The caller must await this process's successful exit before opening UAR.
    drop(db);
    Ok(json!({
        "version": 1, "ok": true, "mode": mode,
        "assertion": assertion_kind, "stateCounts": counts, "totalRows": total,
    }))
}

#[tokio::test]
async fn storage_fixture() {
    match run_fixture().await {
        Ok(summary) => println!("BAUAR_STORAGE_FIXTURE_JSON={summary}"),
        Err(code) => {
            println!("BAUAR_STORAGE_FIXTURE_JSON={}", json!({"version": 1, "ok": false, "code": code}));
            panic!("Offline admission storage fixture refused or failed");
        }
    }
}
