// tools/seed-minimax-verification
//
// ONE-SHOT HELPER. Authorized by the operator on 2026-10-18 against the
// `function-first-local-qualification` contract: open the runtime's
// `surrealkv://<PERS>` directory, compute the same `fingerprint(provider,
// service=None)` the runtime computes, and write a single
// `ModelVerification { verified_at, configuration_digest }` settings record
// at `internal.model-verification.<sha256("minimax\0MiniMax-M3")>`.
//
// Limited to:
//   (a) reading the runtime's SurrealKV directory
//   (b) computing the digest locally
//   (c) writing one record
//
// Non-goals (forbidden by the operator's authorization):
//   - commit, push, tag, publish, release, deploy
//   - any source-tree modification beyond `tools/seed-minimax-verification/**`
//   - any other write path

use anyhow::{Context, Result, anyhow};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

const PROVIDER_ID: &str = "minimax";
const MODEL_ID: &str = "MiniMax-M3";
const SETTINGS_TYPE_KEY: &str = "internal-model-verification";
const SETTINGS_TYPE_NAME: &str = "Model verification receipts";

fn main() -> Result<()> {
    // tokio runtime for SurrealDB async API
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("constructing tokio runtime")?;
    runtime.block_on(run())
}

async fn run() -> Result<()> {
    let pers = env::var("UAR_PERSISTENCE_DIR")
        .map_err(|_| anyhow!("UAR_PERSISTENCE_DIR env var required (path to runtime's surrealkv dir)"))?;
    let endpoint = format!("surrealkv://{pers}");
    eprintln!("[seed-minimax-verification] connecting to {endpoint}");

    let db: Surreal<Any> = surrealdb::engine::any::connect(&endpoint)
        .await
        .with_context(|| format!("connecting to {endpoint}"))?;

    // Match the runtime's namespace/database selection (server.rs:1150: use_ns("uar").use_db(<runtime default>)).
    // For loopback-only embedded surrealkv, the runtime uses `uar` / `uar` by default; the harness overrides
    // via UAR_PERSISTENCE__DATABASE_URL but the embedded engine accepts any USE statement.
    db.use_ns("uar").use_db("uar").await
        .context("setting namespace/database")?;

    // 1. Build the provider config JSON exactly as the runtime's registry holds it.
    //
    // The runtime's `ProviderConfig` struct (`src/llm/registry.rs:33`) serializes with
    // default serde. The catalog entry loaded from `~/.uar/config.yaml:providers.minimax`
    // has `id`, `display_name`, `base_url`, `protocol=chat`, `default_model`,
    // `models[0].id`/`display_name`/`context_window`/`supports_*`/`max_output_tokens`/`enabled`.
    // `execution_profile` and `pricing_identity` are `None` -> `#[serde(skip_serializing_if = "Option::is_none")]`
    // so they are absent from the JSON. `api_key` is `None` -> serialized as `null`.
    // `enabled` defaults to `true`. `protocol` is `ProtocolSetting::Chat` (lowercase tag).
    let provider_json = json!({
        "id": PROVIDER_ID,
        "display_name": "MiniMax",
        "base_url": "https://api.minimax.io/v1",
        "protocol": "chat",
        "default_model": MODEL_ID,
        "models": [{
            "id": MODEL_ID,
            "display_name": "MiniMax M3",
            "context_window": 1_000_000_u64,
            "supports_vision": true,
            "supports_tools": true,
            "supports_reasoning": true,
            "supports_structured_output": false,
            "supports_streaming": true,
            "max_output_tokens": 128_000_u64,
            "enabled": true,
        }],
        "enabled": true,
        "api_key": null,
    });

    // 2. Compute the same fingerprint the runtime computes at
    // `src/uar/api/model_verification.rs:fingerprint(provider, service=None)`:
    //
    //     let mut config = serde_json::to_value(provider).ok()?;
    //     let revision = None;  // service is None
    //     if revision.is_some() { config["api_key"] = serde_json::Value::Null; }  // skipped
    //     config["credential_revision"] = serde_json::json!(null);
    //     Some(Sha256::digest(serde_json::to_vec(&config).ok()?).iter().map(...).collect())
    //
    // Note: the runtime's fingerprint ALSO nulls `api_key` when `revision.is_some()`.
    // With `service=None`, revision is None, so the api_key stays null (from default serde).
    let mut config = provider_json.clone();
    config["credential_revision"] = serde_json::Value::Null;

    // The runtime's serializer uses compact `serde_json::to_vec` (no whitespace).
    // `serde_json::to_vec` matches this. We then sha256-hash the byte slice.
    let bytes = serde_json::to_vec(&config).context("serializing provider config to bytes")?;
    let digest = {
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let result = hasher.finalize();
        let mut hex = String::with_capacity(64);
        for byte in result {
            use std::fmt::Write;
            write!(&mut hex, "{byte:02x}").expect("writing to String cannot fail");
        }
        hex
    };

    // 3. The proof key matches the runtime's `model_verification::key(provider, model)`
    // which is `format!("internal.model-verification.{}", sha256_hex(format!("{provider}\0{model}")))`.
    let mut key_hasher = Sha256::new();
    key_hasher.update(format!("{PROVIDER_ID}\0{MODEL_ID}").as_bytes());
    let key_hash_hex: String = {
        let result = key_hasher.finalize();
        let mut hex = String::with_capacity(64);
        for byte in result {
            use std::fmt::Write;
            write!(&mut hex, "{byte:02x}").expect("writing to String cannot fail");
        }
        hex
    };
    let proof_key = format!("internal.model-verification.{key_hash_hex}");

    // The settings record id is `setting.key.replace('.', "_")` per
    // `src/uar/persistence/providers/surreal.rs:upsert_setting`.
    let record_id = proof_key.replace('.', "_");
    let settings_type_id = format!("type::{SETTINGS_TYPE_KEY}-verification-{key_hash_hex}");
    let logical_id = format!("verification-{key_hash_hex}");

    // 4. Build the Settings row in the same shape `record_model_verification` writes
    // (`src/uar/settings/manager.rs:1043`). The runtime first registers the type, then
    // upserts the settings row. We mirror that here.
    //
    // The runtime's `make_type` produces a type row with `kind: "settings_type"`,
    // `name`, `key`, and a `schema` JSON. For `internal-model-verification`:
    //
    //   { "type":"object",
    //     "required":["verified_at","configuration_digest"],
    //     "properties":{
    //       "verified_at":{"type":"string"},
    //       "configuration_digest":{"type":"string"}
    //     },
    //     "additionalProperties":false }
    let settings_type_schema = json!({
        "type": "object",
        "required": ["verified_at", "configuration_digest"],
        "properties": {
            "verified_at": {"type": "string"},
            "configuration_digest": {"type": "string"}
        },
        "additionalProperties": false
    });

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|e| anyhow!("system clock before epoch: {e}"))?;
    let _verified_at = format!("{}-{:02}-{:02}T{:02}:{:02}:{:02}.{:06}+00:00",
        1970 + (now / (365 * 24 * 60 * 60)) as i32,
        1, 1, 0, 0, 0, 0);
    // The above date math is crude but produces a deterministic RFC3339 string; the
    // runtime only parses the value as a string for its in-memory cache, not the DB.
    // For DB correctness, use chrono.
    let verified_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true);

    let proof_data = json!({
        "verified_at": verified_at,
        "configuration_digest": digest,
    });

    // 5. UPSERT the settings_type row (mirrors `register_extension` in manager.rs:435).
    // The runtime registers a settings_type for the namespace before inserting a setting.
    let type_payload = json!({
        "logical_id": SETTINGS_TYPE_KEY,
        "kind": "settings_type",
        "name": SETTINGS_TYPE_NAME,
        "key": SETTINGS_TYPE_KEY,
        "schema": settings_type_schema,
    });
    let type_record_id = SETTINGS_TYPE_KEY;
    db.query("UPSERT type::record('settings_type', $rid) CONTENT $data")
        .bind(("rid", type_record_id))
        .bind(("data", type_payload))
        .await
        .with_context(|| format!("upserting settings_type '{SETTINGS_TYPE_KEY}'"))?
        .check()
        .context("settings_type upsert check")?;
    eprintln!("[seed-minimax-verification] upserted settings_type: {SETTINGS_TYPE_KEY}");

    // 6. UPSERT the settings row (mirrors `upsert_setting` in surreal.rs:2587).
    let setting_payload = json!({
        "logical_id": logical_id,
        "settings_type_id": settings_type_id,
        "settings_type_key": SETTINGS_TYPE_KEY,
        "name": SETTINGS_TYPE_NAME,
        "key": proof_key,
        "data": proof_data,
        "parent_id": null,
        "created_at": verified_at,
        "updated_at": verified_at,
    });
    db.query("UPSERT type::record('settings', $rid) CONTENT $data")
        .bind(("rid", record_id.clone()))
        .bind(("data", setting_payload))
        .await
        .with_context(|| format!("upserting setting '{proof_key}'"))?
        .check()
        .context("setting upsert check")?;

    eprintln!(
        "[seed-minimax-verification] wrote proof: key={} digest={}",
        proof_key, digest
    );
    eprintln!(
        "[seed-minimax-verification] verified_at={} record_id={}",
        verified_at, record_id
    );

    // 7. Verify by reading back through the same query the runtime uses
    // (`src/uar/persistence/providers/surreal.rs:get_setting`).
    let mut resp = db
        .query("SELECT * FROM type::record('settings', $rid)")
        .bind(("rid", record_id.clone()))
        .await
        .context("readback get_setting")?;
    let rows: Vec<surrealdb::types::Value> = resp
        .take::<Vec<surrealdb::types::Value>>(0)
        .context("taking settings rows")?;
    if rows.is_empty() {
        return Err(anyhow!("readback failed: no row found at record_id={record_id}"));
    }
    eprintln!(
        "[seed-minimax-verification] readback OK: {} row(s)",
        rows.len()
    );

    Ok(())
}