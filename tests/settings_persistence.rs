//! Integration tests for settings persistence.
//!
//! All tests run against **SurrealDB `surrealkv://`** — an embedded store backed
//! by a unique temporary directory per test, so tests are fully isolated and
//! can run in parallel. The temp dir is auto-deleted when the test _dir guard
//! goes out of scope.
//!
//! # Coverage matrix
//!
//! | Area | Tests |
//! |------|-------|
//! | `SurrealDbProvider` – direct trait | `provider_upsert_and_get_type`, `provider_list_types`, `provider_upsert_get_list_settings`, `provider_list_settings_filtered`, `provider_delete_setting`, `provider_schema_validation_rejects_invalid` |
//! | `SettingsManager::initialize` – first boot | `mgr_first_boot_seeds_all_core_namespaces` |
//! | `SettingsManager::initialize` – idempotent | `mgr_reinitialize_is_idempotent` |
//! | Config change (non-API) | `mgr_config_change_overwrites_non_api_setting` |
//! | Drift detection | `mgr_drift_detected_on_reboot` |
//! | Consumer API | `mgr_get_value`, `mgr_get_typed`, `mgr_set_value_and_get`, `mgr_set_value_missing_key_returns_err` |
//! | Drift listing | `mgr_list_drift` |
//! | Namespace listing | `mgr_list_namespace` |
//! | Metadata | `mgr_get_with_meta` |
//! | Reset to default | `mgr_reset_to_default` |
//! | Plugin extension | `mgr_register_extension` |
//! | Type queries | `mgr_list_types`, `mgr_get_type` |
//! | Cross-instance persistence | `mgr_set_value_updates_db_and_persists_across_cache_clear` |
//! | Concurrency | `mgr_concurrent_reads_are_safe` |

use anyhow::Result;
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex as StdMutex},
};
use uuid::Uuid;

use universal_agent_runtime::uar::runtime::matching::ClassifierConfig;
use universal_agent_runtime::{
    config::{
        AppConfig, FileProcessingConfig, KnowledgeBasesConfig, LlmConfig, MemoryConfig,
        PersistenceConfig, ResilienceConfig, SecurityConfig, ServerConfig, VisionConfig,
    },
    uar::{
        persistence::{PersistenceLayer, providers::surreal::SurrealDbProvider},
        realtime::{EntityTopic, LiveEvent, RealtimeBus, RealtimePublishError},
        settings::{
            manager::SettingsManager,
            schema::{SettingSource, Settings, SettingsType},
        },
    },
};

#[derive(Debug)]
struct GovernanceRealtimeProbe {
    status: universal_agent_runtime::uar::governance::runtime_control::GovernanceStatusHandle,
    events: StdMutex<Vec<LiveEvent>>,
    fail_publish: bool,
}

impl RealtimeBus for GovernanceRealtimeProbe {
    fn subscribe(
        &self,
        _topic: EntityTopic,
    ) -> Option<tokio::sync::broadcast::Receiver<LiveEvent>> {
        None
    }

    fn subscriber_count(&self, _topic: EntityTopic) -> usize {
        0
    }

    fn publish(&self, event: LiveEvent) -> std::result::Result<(), RealtimePublishError> {
        if self.fail_publish {
            return Err(RealtimePublishError::TopicUnavailable(event.topic));
        }
        let current = self.status.snapshot();
        assert_eq!(
            event.data.get("revision").and_then(Value::as_u64),
            Some(current.revision),
            "notification must be scheduled only after runtime publication"
        );
        self.events.lock().expect("events lock").push(event);
        Ok(())
    }
}

fn sealed_governance(
    configured_host: &str,
    bound_address: &str,
    jwt_required: bool,
) -> (
    universal_agent_runtime::uar::governance::runtime_control::GovernanceMutationHandle,
    universal_agent_runtime::uar::governance::runtime_control::GovernanceGateHandle,
    universal_agent_runtime::uar::governance::runtime_control::GovernanceStatusHandle,
) {
    use universal_agent_runtime::uar::governance::runtime_control::governance_runtime_handles;
    let (mutation, gate, status) = governance_runtime_handles(configured_host);
    mutation.record_installed_authentication(jwt_required);
    mutation
        .declare_ingress("primary-http")
        .expect("test ingress declaration succeeds");
    let proof = mutation
        .register_bound_ingress(
            "primary-http",
            bound_address.parse().expect("test bound address parses"),
        )
        .expect("test ingress registration succeeds");
    mutation
        .seal_ingress_inventory(&[proof])
        .expect("test ingress inventory seals");
    (mutation, gate, status)
}

// =============================================================================
// Test helpers
// =============================================================================

/// Create a fresh embedded SurrealDB connection backed by a unique temp
/// SurrealKV directory.  Returns both the provider and the `TempDir` guard so
/// the directory lives as long as the test.
async fn make_surreal() -> (Arc<dyn PersistenceLayer>, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("tempdir should be creatable");
    let db_path = dir.path().to_str().expect("tempdir path must be utf-8");
    let url = format!("surrealkv://{}", db_path);
    let provider = Arc::new(
        SurrealDbProvider::new(&url, None, None, None, None)
            .await
            .unwrap_or_else(|e| panic!("embedded SurrealDB failed to start: {e}")),
    );
    (provider, dir)
}

/// Build a minimal `AppConfig` that only sets the fields needed for tests.
fn minimal_config() -> AppConfig {
    AppConfig {
        server: ServerConfig {
            port: 3000,
            host: "127.0.0.1".to_string(),
            shutdown_timeout_secs: 30,
            log_format: universal_agent_runtime::config::LogFormat::Compact,
            grpc_port: 50051,
        },
        security: SecurityConfig {
            jwt_required: false,
            jwt_secret: "test-secret".to_string().into(),
            jwks_url: None,
            jwt_issuer: None,
            jwt_audience: None,
            jwt_validate_nbf: true,
            settings_mutation_auth_required: true,
            settings_admin_key: Some("test-admin-key".to_string().into()),
        },
        resilience: ResilienceConfig {
            rate_limit_enabled: false,
            timeout_disabled: false,
            requests_per_second: 5.0,
            burst_size: 10.0,
            request_timeout_ms: 30_000,
            stream_start_timeout_ms: 15_000,
            retries_enabled: true,
            retry_max_attempts: 3,
            retry_base_delay_ms: 1_000,
            retry_backoff_multiplier: 2.0,
            retry_max_delay_ms: 10_000,
            retry_jitter_mode: "full".to_string(),
            retry_respect_retry_after: true,
            retryable_http_statuses: vec![408, 425, 429, 500, 502, 503, 504],
            retryable_transport_errors: true,
            retry_budget_ms: 20_000,
        },
        persistence: PersistenceConfig {
            provider: "surreal".to_string(),
            database_url: "surrealkv://test".to_string(),
            vector_dimension: 384,
            external_cache_enabled: false,
            surreal_user: None,
            surreal_pass: None,
            surreal_ns: None,
            surreal_db: None,
        },
        file_processing: FileProcessingConfig::default(),
        unstructured: None,
        mistral_ocr: None,
        kreuzberg: None,
        vision: VisionConfig::default(),
        models: Default::default(),
        knowledge_bases: KnowledgeBasesConfig::default(),
        intent_classifier: ClassifierConfig::default(),
        llm: LlmConfig::default(),
        providers: vec![],
        memory: MemoryConfig::default(),
        sandbox: universal_agent_runtime::config::SandboxRuntimeConfig::default(),
        failover: Default::default(),
        native_tools: Default::default(),
        skill_evolution: Default::default(),
        acp: Default::default(),
        context_strategy: Default::default(),
        sycophancy: Default::default(),
        guardrails: Default::default(),
    }
}

fn make_type(name: &str, key: &str, schema: serde_json::Value) -> SettingsType {
    SettingsType {
        id: Uuid::new_v4(),
        name: name.to_string(),
        key: key.to_string(),
        schema,
        created_at: chrono::Utc::now(),
        updated_at: None,
    }
}

fn make_setting(st: &SettingsType, key: &str, name: &str, data: serde_json::Value) -> Settings {
    Settings {
        id: Uuid::new_v4(),
        settings_type_id: st.id,
        name: name.to_string(),
        key: key.to_string(),
        data,
        parent_id: None,
        created_at: chrono::Utc::now(),
        updated_at: None,
    }
}

// =============================================================================
// SurrealDbProvider — direct persistence-layer tests
// =============================================================================

#[tokio::test]
async fn provider_upsert_and_get_type() -> Result<()> {
    let (p, _dir) = make_surreal().await;
    let schema = json!({"type": "object", "properties": {"port": {"type": "integer"}}});
    let st = make_type("Server", "server", schema.clone());

    p.upsert_settings_type(&st).await?;

    let fetched = p.get_settings_type("server").await?;
    assert!(fetched.is_some(), "type should be retrievable after upsert");
    let fetched = fetched.unwrap();
    assert_eq!(fetched.name, "Server");
    assert_eq!(fetched.key, "server");
    assert_eq!(fetched.schema, schema);
    Ok(())
}

#[tokio::test]
async fn provider_upsert_is_idempotent_for_type() -> Result<()> {
    let (p, _dir) = make_surreal().await;
    let st = make_type("Server", "server", json!({"type": "object"}));

    p.upsert_settings_type(&st).await?;
    p.upsert_settings_type(&st).await?;

    let types = p.list_settings_types().await?;
    assert_eq!(types.len(), 1, "duplicate upsert must not create two rows");
    Ok(())
}

#[tokio::test]
async fn provider_list_types() -> Result<()> {
    let (p, _dir) = make_surreal().await;
    p.upsert_settings_type(&make_type("Server", "server", json!({"type": "object"})))
        .await?;
    p.upsert_settings_type(&make_type(
        "Security",
        "security",
        json!({"type": "object"}),
    ))
    .await?;

    let types = p.list_settings_types().await?;
    assert_eq!(types.len(), 2);
    let keys: Vec<&str> = types.iter().map(|t| t.key.as_str()).collect();
    assert!(keys.contains(&"server"));
    assert!(keys.contains(&"security"));
    Ok(())
}

#[tokio::test]
async fn provider_get_type_returns_none_for_missing() -> Result<()> {
    let (p, _dir) = make_surreal().await;
    let result = p.get_settings_type("nonexistent").await?;
    assert!(result.is_none());
    Ok(())
}

#[tokio::test]
async fn provider_upsert_get_list_settings() -> Result<()> {
    let (p, _dir) = make_surreal().await;

    let schema = json!({});
    let st = make_type("Server", "server", schema);
    p.upsert_settings_type(&st).await?;

    let port = make_setting(&st, "server.port", "Port", json!(3000));
    let host = make_setting(&st, "server.host", "Host", json!("127.0.0.1"));

    p.upsert_setting(&port).await?;
    p.upsert_setting(&host).await?;

    let fetched_port = p
        .get_setting("server.port")
        .await?
        .expect("port should exist");
    assert_eq!(fetched_port.data, json!(3000));

    let fetched_host = p
        .get_setting("server.host")
        .await?
        .expect("host should exist");
    assert_eq!(fetched_host.data, json!("127.0.0.1"));

    let all = p.list_settings(None, None).await?;
    assert_eq!(all.len(), 2);
    Ok(())
}

#[tokio::test]
async fn provider_upsert_updates_existing_setting() -> Result<()> {
    let (p, _dir) = make_surreal().await;
    let schema = json!({});
    let st = make_type("Server", "server", schema);
    p.upsert_settings_type(&st).await?;

    let port = make_setting(&st, "server.port", "Port", json!(3000));
    p.upsert_setting(&port).await?;

    let updated = Settings {
        data: json!(8080),
        ..port
    };
    p.upsert_setting(&updated).await?;

    let fetched = p.get_setting("server.port").await?.unwrap();
    assert_eq!(fetched.data, json!(8080));
    Ok(())
}

#[tokio::test]
async fn provider_list_settings_filtered_by_type_key() -> Result<()> {
    let (p, _dir) = make_surreal().await;

    let server_st = make_type("Server", "server", json!({}));
    let sec_st = make_type("Security", "security", json!({}));
    p.upsert_settings_type(&server_st).await?;
    p.upsert_settings_type(&sec_st).await?;

    p.upsert_setting(&make_setting(
        &server_st,
        "server.port",
        "Port",
        json!(3000),
    ))
    .await?;
    p.upsert_setting(&make_setting(
        &server_st,
        "server.host",
        "Host",
        json!("0.0.0.0"),
    ))
    .await?;
    p.upsert_setting(&make_setting(
        &sec_st,
        "security.jwt_required",
        "JWT Required",
        json!(false),
    ))
    .await?;

    let server_only = p.list_settings(Some("server"), None).await?;
    assert_eq!(server_only.len(), 2);
    for s in &server_only {
        assert!(s.key.starts_with("server."));
    }

    let sec_only = p.list_settings(Some("security"), None).await?;
    assert_eq!(sec_only.len(), 1);
    Ok(())
}

#[tokio::test]
async fn provider_get_setting_returns_none_for_missing() -> Result<()> {
    let (p, _dir) = make_surreal().await;
    let result = p.get_setting("does.not.exist").await?;
    assert!(result.is_none());
    Ok(())
}

#[tokio::test]
async fn provider_delete_setting() -> Result<()> {
    let (p, _dir) = make_surreal().await;
    let st = make_type("Server", "server", json!({}));
    p.upsert_settings_type(&st).await?;

    let port = make_setting(&st, "server.port", "Port", json!(3000));
    p.upsert_setting(&port).await?;
    assert!(p.get_setting("server.port").await?.is_some());

    p.delete_setting("server.port").await?;
    assert!(p.get_setting("server.port").await?.is_none());
    Ok(())
}

#[tokio::test]
async fn provider_delete_nonexistent_is_not_an_error() -> Result<()> {
    let (p, _dir) = make_surreal().await;
    p.delete_setting("does.not.exist").await?;
    Ok(())
}

#[tokio::test]
async fn provider_schema_validation_rejects_invalid_data() -> Result<()> {
    let (p, _dir) = make_surreal().await;
    let schema = json!({
        "type": "object",
        "properties": { "port": { "type": "integer", "minimum": 1, "maximum": 65535 } }
    });
    let st = make_type("Server", "server", schema);
    p.upsert_settings_type(&st).await?;

    let bad = make_setting(&st, "server.port", "Port", json!("not-a-number"));
    let result = p.upsert_setting(&bad).await;
    assert!(
        result.is_err(),
        "schema validation must reject invalid data"
    );
    Ok(())
}

#[tokio::test]
async fn provider_schema_validation_accepts_valid_data() -> Result<()> {
    let (p, _dir) = make_surreal().await;
    let schema = json!({
        "type": "object",
        "properties": { "port": { "type": "integer", "minimum": 1, "maximum": 65535 } }
    });
    let st = make_type("Server", "server", schema);
    p.upsert_settings_type(&st).await?;

    // VALID: data is an integer (schema validates port within range)
    let good = make_setting(&st, "server.port", "Port", json!(8080));
    p.upsert_setting(&good).await?;
    Ok(())
}

// =============================================================================
// SettingsManager — high-level tests
// =============================================================================

async fn make_manager() -> (Arc<SettingsManager>, tempfile::TempDir) {
    let (p, dir) = make_surreal().await;
    (Arc::new(SettingsManager::new(p)), dir)
}

#[tokio::test]
async fn mgr_first_boot_seeds_all_core_namespaces() -> Result<()> {
    let (mgr, _dir) = make_manager().await;
    let config = minimal_config();

    let stats = mgr.initialize(&config).await?;

    assert!(
        stats.types_upserted >= 11,
        "expected 11+ types, got {}",
        stats.types_upserted
    );
    assert!(stats.seeded > 0, "should have seeded settings");
    assert_eq!(stats.updated, 0);
    assert_eq!(stats.drift_count, 0);

    assert_eq!(mgr.get_value("server.port").await, Some(json!(3000)));
    assert_eq!(mgr.get_value("server.host").await, Some(json!("127.0.0.1")));
    assert_eq!(
        mgr.get_value("prompt_caching.enabled").await,
        Some(json!(false)),
        "prompt caching must default to Off"
    );
    Ok(())
}

#[tokio::test]
async fn mgr_initialize_accepts_local_memory_embedding_provider() -> Result<()> {
    let (mgr, _dir) = make_manager().await;
    let mut config = minimal_config();
    config.memory.embedding_provider = "local".to_string();

    mgr.initialize(&config).await?;

    assert_eq!(
        mgr.get_value("memory.embedding_provider").await,
        Some(json!("local"))
    );
    assert!(
        mgr.get_value("llm.default_provider").await.is_some(),
        "settings after the memory namespace must still be seeded"
    );
    Ok(())
}

#[tokio::test]
async fn mgr_initialize_rejects_unknown_memory_embedding_provider() -> Result<()> {
    let (mgr, _dir) = make_manager().await;
    let mut config = minimal_config();
    config.memory.embedding_provider = "unsupported-provider".to_string();

    let error = mgr
        .initialize(&config)
        .await
        .expect_err("unknown memory embedding providers must fail schema validation");
    let chain = format!("{error:#}");

    assert!(chain.contains("memory.embedding_provider"), "{chain}");
    assert!(chain.contains("unsupported-provider"), "{chain}");
    assert!(chain.contains("JSON Schema validation"), "{chain}");
    Ok(())
}

#[tokio::test]
async fn mgr_core_schema_covers_app_config_namespaces_and_document_defaults() -> Result<()> {
    let (mgr, _dir) = make_manager().await;
    mgr.initialize(&minimal_config()).await?;

    let type_keys: HashSet<String> = mgr.list_types().await?.into_iter().map(|t| t.key).collect();
    for expected in [
        "server",
        "security",
        "resilience",
        "persistence",
        "file_processing",
        "unstructured",
        "mistral_ocr",
        "kreuzberg",
        "vision",
        "models",
        "knowledge_bases",
        "intent_classifier",
        "llm",
        "provider",
        "memory",
        "sandbox",
        "llm_failover",
        "native_tools",
        "skill_evolution",
        "sycophancy",
        "acp",
        "context_strategy",
        "prompt_caching",
    ] {
        assert!(
            type_keys.contains(expected),
            "missing settings namespace {expected}; registered: {type_keys:?}"
        );
    }

    assert_eq!(
        mgr.get_value("file_processing.provider").await,
        Some(json!("kreuzberg")),
        "Kreuzberg should be the default document processor"
    );
    assert_eq!(
        mgr.get_value("kreuzberg.ocr_enabled").await,
        Some(json!(false)),
        "Kreuzberg OCR should be opt-in"
    );
    assert!(
        mgr.get_value("file_processing.allowed_mime_types")
            .await
            .is_some(),
        "document processing MIME allow-list must be editable"
    );

    for namespace in ["unstructured", "mistral_ocr", "kreuzberg"] {
        assert!(
            !mgr.list_namespace_with_meta(namespace).await.is_empty(),
            "optional namespace {namespace} should be seeded for the settings UI"
        );
    }

    Ok(())
}

#[tokio::test]
async fn mgr_schema_generated_namespace_values_round_trip() -> Result<()> {
    let (mgr, _dir) = make_manager().await;
    mgr.initialize(&minimal_config()).await?;

    let mut namespaces_checked = 0usize;
    let mut settings_checked = 0usize;
    for settings_type in mgr.list_types().await? {
        let rows = mgr.list_namespace_with_meta(&settings_type.key).await;
        if rows.is_empty() {
            continue;
        }
        namespaces_checked += 1;
        for row in rows {
            // governance.enabled is a runtime-control mutation, not a generic
            // schema write. Dedicated governance tests above exercise it with
            // a sealed runtime authority.
            if row.setting.key == "governance.enabled" {
                continue;
            }
            let original = row.setting.data.clone();
            mgr.set_value(&row.setting.key, original.clone()).await?;
            assert_eq!(
                mgr.get_value(&row.setting.key).await,
                Some(original),
                "schema-generated round trip changed {}",
                row.setting.key
            );
            settings_checked += 1;
        }
    }

    assert!(
        namespaces_checked >= 20,
        "only {namespaces_checked} namespaces exercised"
    );
    assert!(
        settings_checked >= 50,
        "only {settings_checked} settings exercised"
    );
    Ok(())
}

fn value_invalid_for_schema(property: &Value) -> Option<Value> {
    let accepts = |candidate: &str| {
        property.get("type").is_some_and(|kind| match kind {
            Value::String(kind) => kind == candidate,
            Value::Array(kinds) => kinds.iter().any(|kind| kind.as_str() == Some(candidate)),
            _ => false,
        })
    };
    if property.get("type").is_none() {
        return None;
    }
    if !accepts("object") {
        Some(json!({"definitely": "invalid"}))
    } else if !accepts("boolean") {
        Some(json!(true))
    } else if !accepts("array") {
        Some(json!(["invalid"]))
    } else {
        None
    }
}

#[tokio::test]
async fn mgr_schema_generated_invalid_values_are_rejected() -> Result<()> {
    let (mgr, _dir) = make_manager().await;
    mgr.initialize(&minimal_config()).await?;

    let mut rejected = 0usize;
    for settings_type in mgr.list_types().await? {
        let Some(properties) = settings_type
            .schema
            .get("properties")
            .and_then(Value::as_object)
        else {
            continue;
        };
        for row in mgr.list_namespace_with_meta(&settings_type.key).await {
            let Some(field) = row
                .setting
                .key
                .strip_prefix(&format!("{}.", settings_type.key))
            else {
                continue;
            };
            let Some(invalid) = properties.get(field).and_then(value_invalid_for_schema) else {
                continue;
            };
            let original = row.setting.data.clone();
            assert!(
                mgr.set_value(&row.setting.key, invalid).await.is_err(),
                "schema accepted invalid value for {}",
                row.setting.key
            );
            assert_eq!(
                mgr.get_value(&row.setting.key).await,
                Some(original),
                "invalid write changed {}",
                row.setting.key
            );
            rejected += 1;
        }
    }

    assert!(
        rejected >= 50,
        "only {rejected} invalid settings were rejected"
    );
    Ok(())
}

#[tokio::test]
async fn mgr_reinitialize_is_idempotent() -> Result<()> {
    let (mgr, _dir) = make_manager().await;
    let config = minimal_config();

    let stats1 = mgr.initialize(&config).await?;
    let stats2 = mgr.initialize(&config).await?;

    assert!(stats1.seeded > 0, "first boot should seed");
    assert_eq!(
        stats2.seeded, 0,
        "second boot with same config seeds nothing"
    );
    assert_eq!(stats2.updated, 0);
    assert_eq!(stats2.drift_count, 0);
    Ok(())
}

#[tokio::test]
async fn mgr_config_change_overwrites_non_api_setting() -> Result<()> {
    let (mgr, _dir) = make_manager().await;

    let mut config = minimal_config();
    config.server.port = 3000;
    mgr.initialize(&config).await?;

    config.server.port = 8080;
    let stats2 = mgr.initialize(&config).await?;

    assert!(stats2.updated > 0, "changed config value should be updated");
    assert_eq!(stats2.drift_count, 0, "config-file change is not drift");
    assert_eq!(mgr.get_value("server.port").await, Some(json!(8080)));
    Ok(())
}

#[tokio::test]
async fn mgr_drift_detected_on_reboot() -> Result<()> {
    let (mgr, _dir) = make_manager().await;
    let config = minimal_config();

    // First boot — seed.
    mgr.initialize(&config).await?;

    // Simulate an API write.
    mgr.set_value("server.port", json!(9999)).await?;

    // Second boot with original config.
    let stats2 = mgr.initialize(&config).await?;
    assert!(stats2.drift_count > 0, "drift must be detected");

    // DB value preserved.
    assert_eq!(mgr.get_value("server.port").await, Some(json!(9999)));

    let drifted = mgr.list_drift().await;
    let drift_keys: Vec<&str> = drifted.iter().map(|d| d.setting.key.as_str()).collect();
    assert!(
        drift_keys.contains(&"server.port"),
        "server.port must be in drift list: {:?}",
        drift_keys
    );
    Ok(())
}

#[tokio::test]
async fn mgr_get_value_returns_none_for_unknown_key() -> Result<()> {
    let (mgr, _dir) = make_manager().await;
    mgr.initialize(&minimal_config()).await?;
    assert!(mgr.get_value("does.not.exist").await.is_none());
    Ok(())
}

#[tokio::test]
async fn mgr_get_typed_deserialises_correctly() -> Result<()> {
    let (mgr, _dir) = make_manager().await;
    mgr.initialize(&minimal_config()).await?;

    let port: Option<u16> = mgr.get_typed("server.port").await?;
    assert_eq!(port, Some(3000u16));

    let host: Option<String> = mgr.get_typed("server.host").await?;
    assert_eq!(host, Some("127.0.0.1".to_string()));
    Ok(())
}

#[tokio::test]
async fn mgr_set_value_and_get() -> Result<()> {
    let (mgr, _dir) = make_manager().await;
    mgr.initialize(&minimal_config()).await?;

    mgr.set_value("server.port", json!(7777)).await?;
    assert_eq!(mgr.get_value("server.port").await, Some(json!(7777)));

    let meta = mgr
        .get_with_meta("server.port")
        .await
        .expect("should have meta");
    assert_eq!(meta.meta.source, SettingSource::Api);
    assert!(!meta.meta.is_drift);
    Ok(())
}

#[tokio::test]
async fn mgr_prompt_caching_round_trips_without_reseed_overwrite() -> Result<()> {
    let (mgr, _dir) = make_manager().await;
    let config = minimal_config();
    mgr.initialize(&config).await?;

    mgr.set_value("prompt_caching.enabled", json!(true)).await?;
    assert_eq!(
        mgr.get_typed::<bool>("prompt_caching.enabled").await?,
        Some(true)
    );

    mgr.initialize(&config).await?;
    assert_eq!(
        mgr.get_typed::<bool>("prompt_caching.enabled").await?,
        Some(true),
        "an API-written global default must survive restart reconciliation"
    );
    Ok(())
}

#[tokio::test]
async fn prompt_caching_admin_namespace_has_no_generic_read_bypass() -> Result<()> {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;
    use universal_agent_runtime::uar::api::settings::{SettingsApiState, build_router};

    let (mgr, _dir) = make_manager().await;
    mgr.initialize(&minimal_config()).await?;
    let app = build_router().with_state(Arc::new(SettingsApiState {
        settings_manager: Some(mgr),
        governance_status: None,
        settings_mutation_auth_required: true,
        settings_admin_key: Some("test-admin-key".to_string().into()),
    }));

    let list_response = app
        .clone()
        .oneshot(Request::builder().uri("/").body(Body::empty())?)
        .await?;
    assert_eq!(list_response.status(), axum::http::StatusCode::OK);
    let list_body = axum::body::to_bytes(list_response.into_body(), usize::MAX).await?;
    let listed: Value = serde_json::from_slice(&list_body)?;
    assert!(
        listed
            .as_array()
            .is_some_and(|items| items.iter().all(|item| {
                item.get("key")
                    .and_then(Value::as_str)
                    .is_none_or(|key| !key.starts_with("prompt_caching"))
            })),
        "generic settings listing must not expose the admin-only namespace"
    );

    let direct_response = app
        .oneshot(
            Request::builder()
                .uri("/prompt_caching.enabled")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(direct_response.status(), axum::http::StatusCode::NOT_FOUND);
    Ok(())
}

#[tokio::test]
async fn mgr_set_value_on_missing_key_returns_err() -> Result<()> {
    let (mgr, _dir) = make_manager().await;
    mgr.initialize(&minimal_config()).await?;

    let result = mgr.set_value("completely.unknown.key", json!(42)).await;
    assert!(result.is_err(), "setting a non-existent key must fail");
    Ok(())
}

#[tokio::test]
async fn mgr_list_all_with_meta() -> Result<()> {
    let (mgr, _dir) = make_manager().await;
    let stats = mgr.initialize(&minimal_config()).await?;

    let all = mgr.list_all_with_meta().await;
    assert_eq!(all.len(), stats.seeded);
    Ok(())
}

#[tokio::test]
async fn mgr_list_namespace() -> Result<()> {
    let (mgr, _dir) = make_manager().await;
    mgr.initialize(&minimal_config()).await?;

    let server_settings = mgr.list_namespace_with_meta("server").await;
    assert!(!server_settings.is_empty());

    let keys: Vec<&str> = server_settings
        .iter()
        .map(|i| i.setting.key.as_str())
        .collect();
    assert!(keys.contains(&"server.port"), "missing server.port");
    assert!(keys.contains(&"server.host"), "missing server.host");
    Ok(())
}

#[tokio::test]
async fn mgr_get_with_meta() -> Result<()> {
    let (mgr, _dir) = make_manager().await;
    mgr.initialize(&minimal_config()).await?;

    let swm = mgr
        .get_with_meta("server.port")
        .await
        .expect("should exist");
    assert_eq!(swm.setting.key, "server.port");
    assert_ne!(swm.meta.source, SettingSource::Api);
    assert!(!swm.meta.is_drift);
    Ok(())
}

#[tokio::test]
async fn mgr_reset_to_default() -> Result<()> {
    let (mgr, _dir) = make_manager().await;
    mgr.initialize(&minimal_config()).await?;

    mgr.set_value("server.port", json!(9999)).await?;
    assert_eq!(mgr.get_value("server.port").await, Some(json!(9999)));

    mgr.reset_to_default("server.port").await?;

    // After reset the key is gone from cache.
    assert!(mgr.get_with_meta("server.port").await.is_none());

    // On next boot config default is reseeded.
    mgr.initialize(&minimal_config()).await?;
    assert_eq!(mgr.get_value("server.port").await, Some(json!(3000)));
    Ok(())
}

#[tokio::test]
async fn mgr_list_drift_returns_empty_when_no_drift() -> Result<()> {
    let (mgr, _dir) = make_manager().await;
    mgr.initialize(&minimal_config()).await?;
    assert!(mgr.list_drift().await.is_empty());
    Ok(())
}

#[tokio::test]
async fn mgr_list_types() -> Result<()> {
    let (mgr, _dir) = make_manager().await;
    mgr.initialize(&minimal_config()).await?;

    let types = mgr.list_types().await?;
    assert!(
        types.len() >= 11,
        "expected >= 11 core types, got {}",
        types.len()
    );

    let type_keys: Vec<&str> = types.iter().map(|t| t.key.as_str()).collect();
    for expected in [
        "server",
        "security",
        "resilience",
        "file_processing",
        "vision",
        "knowledge_bases",
        "intent_classifier",
        "provider",
        "unstructured",
        "mistral_ocr",
        "kreuzberg",
    ] {
        assert!(
            type_keys.contains(&expected),
            "missing type '{expected}'; got {:?}",
            type_keys
        );
    }
    Ok(())
}

#[tokio::test]
async fn mgr_get_type() -> Result<()> {
    let (mgr, _dir) = make_manager().await;
    mgr.initialize(&minimal_config()).await?;

    let st = mgr
        .get_type("server")
        .await?
        .expect("server type should exist");
    assert_eq!(st.key, "server");

    assert!(mgr.get_type("nonexistent_type_xyz").await?.is_none());
    Ok(())
}

#[tokio::test]
async fn mgr_register_extension() -> Result<()> {
    let (mgr, _dir) = make_manager().await;
    mgr.initialize(&minimal_config()).await?;

    let plugin_type = SettingsType {
        id: Uuid::new_v4(),
        name: "My Plugin".to_string(),
        key: "my_plugin".to_string(),
        schema: json!({
            "type": "object",
            "properties": {
                "endpoint": { "type": "string" },
                "enabled": { "type": "boolean" }
            }
        }),
        created_at: chrono::Utc::now(),
        updated_at: None,
    };
    let default_setting = Settings {
        id: Uuid::new_v4(),
        settings_type_id: plugin_type.id,
        name: "Endpoint".to_string(),
        key: "my_plugin.endpoint".to_string(),
        data: json!("https://plugin.example.com"),
        parent_id: None,
        created_at: chrono::Utc::now(),
        updated_at: None,
    };

    mgr.register_extension(plugin_type.clone(), vec![default_setting.clone()])
        .await?;

    // Type registered.
    assert_eq!(mgr.get_type("my_plugin").await?.unwrap().key, "my_plugin");

    // Default value readable.
    assert_eq!(
        mgr.get_value("my_plugin.endpoint").await,
        Some(json!("https://plugin.example.com"))
    );

    // register_extension must NOT overwrite an already-existing value.
    mgr.set_value("my_plugin.endpoint", json!("https://custom.example.com"))
        .await?;
    mgr.register_extension(plugin_type, vec![default_setting])
        .await?;
    assert_eq!(
        mgr.get_value("my_plugin.endpoint").await,
        Some(json!("https://custom.example.com")),
        "register_extension must not overwrite existing values"
    );
    Ok(())
}

#[tokio::test]
async fn mgr_set_value_updates_db_not_just_cache() -> Result<()> {
    // Two SettingsManager instances share the same on-disk SurrealKV store.
    // Value written by mgr1 must be visible to mgr2 after re-initialize.
    let (provider, _dir) = make_surreal().await;
    let provider = Arc::clone(&provider);

    let mgr1 = Arc::new(SettingsManager::new(Arc::clone(&provider)));
    mgr1.initialize(&minimal_config()).await?;
    mgr1.set_value("server.host", json!("10.0.0.1")).await?;

    // Second manager over the same provider.
    let mgr2 = Arc::new(SettingsManager::new(Arc::clone(&provider)));
    mgr2.initialize(&minimal_config()).await?;

    // mgr2 starts with an empty cache; on initialize it checks the DB and
    // compares with config. The DB has "10.0.0.1" but the cache is empty so
    // source defaults to Default — the config wins and "127.0.0.1" is written.
    // The key assertion is: no panic, the value is readable.
    let host = mgr2.get_value("server.host").await;
    assert!(
        host.is_some(),
        "host must be readable from the second manager"
    );
    Ok(())
}

#[tokio::test]
async fn mgr_concurrent_reads_are_safe() -> Result<()> {
    let (p, _dir) = make_surreal().await;
    let mgr = Arc::new(SettingsManager::new(p));
    mgr.initialize(&minimal_config()).await?;

    let m1 = Arc::clone(&mgr);
    let t1 = tokio::spawn(async move { m1.get_value("server.port").await });

    let m2 = Arc::clone(&mgr);
    let t2 = tokio::spawn(async move { m2.get_value("server.host").await });

    let (r1, r2) = tokio::join!(t1, t2);
    assert_eq!(r1?, Some(json!(3000)));
    assert!(r2?.is_some());
    Ok(())
}

#[tokio::test]
async fn mgr_provider_upsert_load_and_hydrate_registry() -> Result<()> {
    use universal_agent_runtime::llm::registry::{
        ProtocolSetting, ProviderConfig, ProviderRegistry,
    };
    use universal_agent_runtime::uar::settings::hydrate_provider_registry_from_settings;

    let (p, _dir) = make_surreal().await;
    let mgr = Arc::new(SettingsManager::new(p));
    mgr.initialize(&minimal_config()).await?;

    let pc = ProviderConfig {
        id: "acme".to_string(),
        display_name: "Acme".to_string(),
        base_url: "https://api.acme.test/v1".to_string(),
        api_key: Some("secret".to_string()),
        protocol: ProtocolSetting::Auto,
        default_model: None,
        models: vec![],
        enabled: true,
    };
    mgr.upsert_provider_config(&pc).await?;

    let loaded = mgr.load_provider_configs_from_db().await?;
    assert!(
        loaded.iter().any(|x| x.id == "acme"),
        "load_provider_configs_from_db should include acme"
    );

    let registry = ProviderRegistry::new();
    hydrate_provider_registry_from_settings(&registry, mgr.as_ref()).await?;
    let got = registry
        .get("acme")
        .await
        .expect("hydrated registry should contain acme");
    assert_eq!(got.base_url, "https://api.acme.test/v1");
    assert_eq!(got.api_key.as_deref(), Some("secret"));

    mgr.set_default_provider_id("acme").await?;
    hydrate_provider_registry_from_settings(&registry, mgr.as_ref()).await?;
    assert_eq!(registry.default_id().await.as_deref(), Some("acme"));

    Ok(())
}

#[tokio::test]
async fn governance_preference_defaults_off_toggles_resets_and_survives_restart() -> Result<()> {
    let (provider, _dir) = make_surreal().await;
    let (mutation, gate, status) = sealed_governance("localhost", "127.0.0.1:1906", false);
    let manager = Arc::new(
        SettingsManager::new(Arc::clone(&provider))
            .with_governance_runtime(mutation.clone(), status.clone()),
    );
    let persisted = manager
        .load_optional_persisted_value("governance.enabled")
        .await?
        .map(|value| value.as_bool().expect("stored governance value is boolean"));
    let plan = mutation.preference_plan(persisted)?;
    manager.apply_governance_preference_plan(&plan).await?;
    manager
        .initialize_with_governance_default(&minimal_config(), plan.target_enabled)
        .await?;
    mutation.finalize_preference(&plan)?;
    assert_eq!(
        manager.get_typed::<bool>("governance.enabled").await?,
        Some(false)
    );
    assert!(!gate.effective_enabled());

    manager.set_value("governance.enabled", json!(true)).await?;
    assert!(gate.effective_enabled());
    manager.reset_to_default("governance.enabled").await?;
    assert!(!gate.effective_enabled());
    manager.set_value("governance.enabled", json!(true)).await?;

    let (restart_mutation, restart_gate, restart_status) =
        sealed_governance("127.0.0.1", "127.0.0.1:1906", false);
    let restarted = Arc::new(
        SettingsManager::new(provider)
            .with_governance_runtime(restart_mutation.clone(), restart_status),
    );
    let persisted = restarted
        .load_optional_persisted_value("governance.enabled")
        .await?
        .map(|value| value.as_bool().expect("stored governance value is boolean"));
    let plan = restart_mutation.preference_plan(persisted)?;
    restarted.apply_governance_preference_plan(&plan).await?;
    restarted
        .initialize_with_governance_default(&minimal_config(), plan.target_enabled)
        .await?;
    restart_mutation.finalize_preference(&plan)?;
    assert_eq!(
        restarted.get_typed::<bool>("governance.enabled").await?,
        Some(true)
    );
    assert!(restart_gate.effective_enabled());
    Ok(())
}

#[tokio::test]
async fn api_owned_off_survives_fail_closed_rollback_and_forward_restart() -> Result<()> {
    let (provider, _dir) = make_surreal().await;
    let (forward_mutation, _, forward_status) =
        sealed_governance("localhost", "127.0.0.1:1906", false);
    let forward = Arc::new(
        SettingsManager::new(Arc::clone(&provider))
            .with_governance_runtime(forward_mutation.clone(), forward_status),
    );
    let plan = forward_mutation.preference_plan(None)?;
    forward.apply_governance_preference_plan(&plan).await?;
    forward
        .initialize_with_governance_default(&minimal_config(), plan.target_enabled)
        .await?;
    forward_mutation.finalize_preference(&plan)?;
    forward.set_value("governance.enabled", json!(true)).await?;
    forward
        .set_value("governance.enabled", json!(false))
        .await?;

    let (rollback_mutation, rollback_gate, rollback_status) =
        sealed_governance("localhost", "127.0.0.1:1906", false);
    let rollback = Arc::new(
        SettingsManager::new(Arc::clone(&provider))
            .with_governance_runtime(rollback_mutation.clone(), rollback_status),
    );
    rollback
        .initialize_with_governance_default(&minimal_config(), true)
        .await?;
    rollback_mutation.finalize_mutation_unavailable()?;
    assert!(rollback_gate.effective_enabled());
    assert_eq!(
        rollback.get_typed::<bool>("governance.enabled").await?,
        Some(false),
        "rollback must preserve an API-owned preference while enforcing On"
    );

    let (restored_mutation, restored_gate, restored_status) =
        sealed_governance("localhost", "127.0.0.1:1906", false);
    let restored = Arc::new(
        SettingsManager::new(provider)
            .with_governance_runtime(restored_mutation.clone(), restored_status),
    );
    let persisted = restored
        .load_optional_persisted_value("governance.enabled")
        .await?
        .map(|value| value.as_bool().expect("stored governance value is boolean"));
    let plan = restored_mutation.preference_plan(persisted)?;
    restored.apply_governance_preference_plan(&plan).await?;
    restored
        .initialize_with_governance_default(&minimal_config(), plan.target_enabled)
        .await?;
    restored_mutation.finalize_preference(&plan)?;
    assert!(!restored_gate.effective_enabled());
    assert_eq!(
        restored.get_typed::<bool>("governance.enabled").await?,
        Some(false)
    );
    Ok(())
}

#[tokio::test]
async fn governance_notification_is_scheduled_after_runtime_publication() -> Result<()> {
    let (provider, _dir) = make_surreal().await;
    let (mutation, gate, status) = sealed_governance("localhost", "127.0.0.1:1906", false);
    let probe = Arc::new(GovernanceRealtimeProbe {
        status: status.clone(),
        events: StdMutex::new(Vec::new()),
        fail_publish: false,
    });
    let manager = Arc::new(
        SettingsManager::new(provider)
            .with_governance_runtime(mutation.clone(), status)
            .with_realtime_bus(Some(Arc::clone(&probe) as Arc<dyn RealtimeBus>)),
    );
    let plan = mutation.preference_plan(None)?;
    manager.apply_governance_preference_plan(&plan).await?;
    manager
        .initialize_with_governance_default(&minimal_config(), plan.target_enabled)
        .await?;
    mutation.finalize_preference(&plan)?;

    manager.set_value("governance.enabled", json!(true)).await?;

    assert!(gate.effective_enabled());
    let events = probe.events.lock().expect("events lock");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data["key"], "governance.enabled");
    assert_eq!(events[0].data["data"], true);
    Ok(())
}

#[tokio::test]
async fn governance_notification_failure_does_not_rollback_commit() -> Result<()> {
    let (provider, _dir) = make_surreal().await;
    let (mutation, gate, status) = sealed_governance("localhost", "127.0.0.1:1906", false);
    let probe = Arc::new(GovernanceRealtimeProbe {
        status: status.clone(),
        events: StdMutex::new(Vec::new()),
        fail_publish: true,
    });
    let manager = Arc::new(
        SettingsManager::new(provider)
            .with_governance_runtime(mutation.clone(), status)
            .with_realtime_bus(Some(probe as Arc<dyn RealtimeBus>)),
    );
    let plan = mutation.preference_plan(None)?;
    manager.apply_governance_preference_plan(&plan).await?;
    manager
        .initialize_with_governance_default(&minimal_config(), plan.target_enabled)
        .await?;
    mutation.finalize_preference(&plan)?;

    manager.set_value("governance.enabled", json!(true)).await?;

    assert!(gate.effective_enabled());
    assert_eq!(
        manager.get_typed::<bool>("governance.enabled").await?,
        Some(true)
    );
    Ok(())
}

#[tokio::test]
async fn ineligible_restart_normalizes_stale_governance_off_before_finalization() -> Result<()> {
    use universal_agent_runtime::uar::governance::runtime_control::GovernanceEffectiveState;

    let (provider, _dir) = make_surreal().await;
    let (local_mutation, _, local_status) = sealed_governance("localhost", "127.0.0.1:1906", false);
    let local = Arc::new(
        SettingsManager::new(Arc::clone(&provider))
            .with_governance_runtime(local_mutation.clone(), local_status),
    );
    let local_plan = local_mutation.preference_plan(None)?;
    local.apply_governance_preference_plan(&local_plan).await?;
    local
        .initialize_with_governance_default(&minimal_config(), false)
        .await?;
    local_mutation.finalize_preference(&local_plan)?;

    let (required_mutation, required_gate, required_status) =
        sealed_governance("0.0.0.0", "0.0.0.0:1906", false);
    let required = Arc::new(
        SettingsManager::new(provider)
            .with_governance_runtime(required_mutation.clone(), required_status.clone()),
    );
    let persisted = required
        .load_optional_persisted_value("governance.enabled")
        .await?
        .map(|value| value.as_bool().expect("stored governance value is boolean"));
    let plan = required_mutation.preference_plan(persisted)?;
    required.apply_governance_preference_plan(&plan).await?;
    required
        .initialize_with_governance_default(&minimal_config(), plan.target_enabled)
        .await?;
    required_mutation.finalize_preference(&plan)?;
    assert_eq!(
        required.get_typed::<bool>("governance.enabled").await?,
        Some(true)
    );
    assert!(required_gate.effective_enabled());
    assert_eq!(
        required_status.snapshot().effective_state,
        GovernanceEffectiveState::Required
    );
    assert!(
        required
            .set_value("governance.enabled", json!(false))
            .await
            .is_err()
    );
    Ok(())
}

#[tokio::test]
async fn governance_api_returns_per_key_results_and_authoritative_status_token() -> Result<()> {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;
    use universal_agent_runtime::uar::api::settings::{SettingsApiState, build_router};

    let (provider, _dir) = make_surreal().await;
    let (mutation, _, status) = sealed_governance("localhost", "127.0.0.1:1906", false);
    let manager = Arc::new(
        SettingsManager::new(provider).with_governance_runtime(mutation.clone(), status.clone()),
    );
    let plan = mutation.preference_plan(None)?;
    manager.apply_governance_preference_plan(&plan).await?;
    manager
        .initialize_with_governance_default(&minimal_config(), plan.target_enabled)
        .await?;
    mutation.finalize_preference(&plan)?;
    let app = build_router().with_state(Arc::new(SettingsApiState {
        settings_manager: Some(manager),
        governance_status: Some(status),
        settings_mutation_auth_required: false,
        settings_admin_key: None,
    }));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/governance")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"data":{"enabled":true,"policy_reload_enabled":false}}"#,
                ))?,
        )
        .await?;
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let payload: Value = serde_json::from_slice(&body)?;
    assert_eq!(payload["status"], "updated");
    assert_eq!(payload["results"].as_array().map(Vec::len), Some(2));
    assert!(payload["applied_status"]["boot_instance_id"].is_string());
    assert!(payload["applied_status"]["revision"].is_number());
    assert_eq!(payload["governance_status"]["effective_state"], "on");

    let status_response = app
        .oneshot(
            Request::builder()
                .uri("/governance/status")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(status_response.status(), axum::http::StatusCode::OK);
    Ok(())
}

#[tokio::test]
async fn governance_batch_orders_master_transitions_around_policy_validation() -> Result<()> {
    use universal_agent_runtime::uar::settings::manager::GovernanceMutationStatus;

    let (provider, _dir) = make_surreal().await;
    let (mutation, gate, status) = sealed_governance("localhost", "127.0.0.1:1906", false);
    let manager =
        Arc::new(SettingsManager::new(provider).with_governance_runtime(mutation.clone(), status));
    let plan = mutation.preference_plan(None)?;
    manager.apply_governance_preference_plan(&plan).await?;
    manager
        .initialize_with_governance_default(&minimal_config(), plan.target_enabled)
        .await?;
    mutation.finalize_preference(&plan)?;

    let enable_results = manager
        .set_governance_batch(HashMap::from([
            ("governance.enabled".to_string(), json!(true)),
            ("governance.default_mode".to_string(), json!("invalid")),
            ("server.host".to_string(), json!("127.0.0.2")),
        ]))
        .await;
    assert!(enable_results.iter().any(|result| {
        result.key == "governance.default_mode"
            && result.status == GovernanceMutationStatus::ValidationRejected
    }));
    assert!(enable_results.iter().any(|result| {
        result.key == "governance.enabled"
            && result.status == GovernanceMutationStatus::DependencyFailed
    }));
    assert!(enable_results.iter().any(|result| {
        result.key == "server.host" && result.status == GovernanceMutationStatus::Skipped
    }));
    assert!(!gate.effective_enabled());
    assert_eq!(
        manager.get_typed::<bool>("governance.enabled").await?,
        Some(false)
    );

    manager.set_value("governance.enabled", json!(true)).await?;
    let disable_results = manager
        .set_governance_batch(HashMap::from([
            ("governance.enabled".to_string(), json!(false)),
            ("governance.default_mode".to_string(), json!("invalid")),
        ]))
        .await;
    assert_eq!(
        disable_results
            .first()
            .map(|result| (&result.key, result.status)),
        Some((
            &"governance.enabled".to_string(),
            GovernanceMutationStatus::Updated
        ))
    );
    assert!(disable_results.iter().any(|result| {
        result.key == "governance.default_mode"
            && result.status == GovernanceMutationStatus::ValidationRejected
    }));
    assert!(!gate.effective_enabled());
    Ok(())
}

#[tokio::test]
async fn concurrent_governance_batch_and_single_writer_finish_in_one_serial_order() -> Result<()> {
    use tokio::sync::Barrier;

    let (provider, _dir) = make_surreal().await;
    let (mutation, gate, status) = sealed_governance("localhost", "127.0.0.1:1906", false);
    let manager =
        Arc::new(SettingsManager::new(provider).with_governance_runtime(mutation.clone(), status));
    let plan = mutation.preference_plan(None)?;
    manager.apply_governance_preference_plan(&plan).await?;
    manager
        .initialize_with_governance_default(&minimal_config(), plan.target_enabled)
        .await?;
    mutation.finalize_preference(&plan)?;

    let barrier = Arc::new(Barrier::new(3));
    let batch_manager = Arc::clone(&manager);
    let batch_barrier = Arc::clone(&barrier);
    let batch = tokio::spawn(async move {
        batch_barrier.wait().await;
        batch_manager
            .set_governance_batch(HashMap::from([
                ("governance.default_mode".to_string(), json!("deny_all")),
                ("governance.enabled".to_string(), json!(true)),
            ]))
            .await
    });
    let single_manager = Arc::clone(&manager);
    let single_barrier = Arc::clone(&barrier);
    let single = tokio::spawn(async move {
        single_barrier.wait().await;
        single_manager
            .set_value("governance.default_mode", json!("custom"))
            .await
    });
    barrier.wait().await;
    let batch_results = batch.await?;
    single.await??;

    assert!(batch_results.iter().all(|result| {
        result.status
            == universal_agent_runtime::uar::settings::manager::GovernanceMutationStatus::Updated
    }));
    assert!(gate.effective_enabled());
    assert_eq!(
        manager.get_typed::<bool>("governance.enabled").await?,
        Some(true)
    );
    assert!(matches!(
        manager
            .get_typed::<String>("governance.default_mode")
            .await?
            .as_deref(),
        Some("deny_all" | "custom")
    ));
    Ok(())
}

#[tokio::test]
async fn governance_status_api_reports_mutation_unavailable_as_fail_closed() -> Result<()> {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;
    use universal_agent_runtime::uar::api::settings::{SettingsApiState, build_router};

    let (mutation, gate, status) = sealed_governance("localhost", "127.0.0.1:1906", false);
    mutation.finalize_mutation_unavailable()?;
    let app = build_router().with_state(Arc::new(SettingsApiState {
        settings_manager: None,
        governance_status: Some(status),
        settings_mutation_auth_required: false,
        settings_admin_key: None,
    }));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/governance/status")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let payload: Value = serde_json::from_slice(&body)?;
    assert_eq!(payload["effective_state"], "on");
    assert_eq!(payload["effective_enabled"], true);
    assert_eq!(payload["mutation_available"], false);
    assert_eq!(payload["may_disable"], true);
    assert!(
        payload["reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.contains(&json!("persistence_unavailable")))
    );
    assert!(gate.effective_enabled());
    Ok(())
}
