//! Round-5 regression coverage for the SurrealDB value conversion.
//!
//! The missing-value sentinel (Value::None) must convert to JSON null so that
//! typed Option fields (e.g. MigrationRowMetadata.row_revision) deserialize as
//! None; Value::Null keeps null behavior; legitimate strings "None"/"Null"
//! remain strings; real revision strings are unchanged.

use serde::Deserialize;
use serde_json::json;
use surrealdb::types::Value;
use universal_agent_runtime::uar::persistence::providers::surreal::value_to_json;
use universal_agent_runtime::uar::security::credentials::migration::MigrationRowMetadata;

#[test]
fn serde_shape_sentinels_are_plain_strings_and_strings_are_tagged() {
    // Documents the serialization contract the conversion relies on: derived
    // Serialize emits unit variants as their names and newtype String content
    // as {"String": s}.
    assert_eq!(serde_json::to_value(&Value::None).unwrap(), json!("None"));
    assert_eq!(serde_json::to_value(&Value::Null).unwrap(), json!("Null"));
    assert_eq!(
        serde_json::to_value(&Value::String("None".to_string())).unwrap(),
        json!({"String": "None"})
    );
}

#[test]
fn missing_value_sentinel_becomes_absent() {
    assert_eq!(value_to_json(Value::None).unwrap(), serde_json::Value::Null);
}

#[test]
fn null_retains_null_behavior() {
    assert_eq!(value_to_json(Value::Null).unwrap(), serde_json::Value::Null);
}

#[test]
fn legitimate_sentinel_looking_strings_remain_strings() {
    assert_eq!(value_to_json(Value::String("None".to_string())).unwrap(), json!("None"));
    assert_eq!(value_to_json(Value::String("Null".to_string())).unwrap(), json!("Null"));
}

#[test]
fn real_revision_string_unchanged() {
    assert_eq!(
        value_to_json(Value::String("27eb0d20-d61f-4113-83b7-c5d28cc625e1".to_string())).unwrap(),
        json!("27eb0d20-d61f-4113-83b7-c5d28cc625e1")
    );
}

#[test]
fn nested_collections_keep_sentinel_semantics() {
    // Wire form of Value::Array is externally tagged.
    let value = Value::deserialize(json!({"Array": [
        "Null", "None", {"String": "None"}, {"String": "real"}
    ]}))
    .expect("typed value");
    assert_eq!(value_to_json(value).unwrap(), json!([null, null, "None", "real"]));
}

fn row_with_revision(revision: Value) -> Value {
    Value::deserialize(json!({"Object": {
        "scope": {"String": "user"},
        "scope_id": {"String": "source"},
        "provider_id": {"String": "upgrade"},
        "row_revision": serde_json::to_value(&revision).unwrap(),
        "created_at": {"String": "2026-10-18T00:00:00Z"},
        "updated_at": {"String": "2026-10-18T00:00:00Z"}
    }}))
    .expect("typed row")
}

#[test]
fn extraction_boundary_sentinel_yields_absent_revision() {
    let json = value_to_json(row_with_revision(Value::None)).unwrap();
    let row: MigrationRowMetadata = serde_json::from_value(json).expect("row");
    assert!(row.row_revision.is_none());
}

#[test]
fn extraction_boundary_null_yields_absent_revision() {
    let json = value_to_json(row_with_revision(Value::Null)).unwrap();
    let row: MigrationRowMetadata = serde_json::from_value(json).expect("row");
    assert!(row.row_revision.is_none());
}

#[test]
fn extraction_boundary_real_revision_preserved() {
    let json = value_to_json(row_with_revision(Value::String(
        "27eb0d20-d61f-4113-83b7-c5d28cc625e1".to_string(),
    )))
    .unwrap();
    let row: MigrationRowMetadata = serde_json::from_value(json).expect("row");
    assert_eq!(row.row_revision.as_deref(), Some("27eb0d20-d61f-4113-83b7-c5d28cc625e1"));
}

#[test]
fn extraction_boundary_legit_string_stays_some() {
    // A legitimate string that looks like the sentinel must stay a string.
    let json = value_to_json(row_with_revision(Value::String("None".to_string()))).unwrap();
    let row: MigrationRowMetadata = serde_json::from_value(json).expect("row");
    assert_eq!(row.row_revision.as_deref(), Some("None"));
}
