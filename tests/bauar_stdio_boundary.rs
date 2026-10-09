//! Active configured and captured stdio boundaries, with a Node LTS receiver.
//! These scenarios exercise process ownership; they do not claim an OS sandbox exists.
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashMap},
    ffi::OsString,
    io::Write as _,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Arc,
    time::Duration,
};
use universal_agent_runtime::mcp::{
    binding_cache::{McpBindingCache, McpBindingEnvironment, McpBindingRequest},
    catalog::{McpCatalog, ServerAuthentication, ServerDefinition, ServerSource},
    config::{McpConfig, McpServerEntry},
    projection::{McpProjectionScope, McpServerProjection},
    registry::McpRegistry,
    runtime::{ConfiguredMcpConnector, McpRuntimeManager},
};
use universal_agent_runtime::uar::{
    domain::policy::{PolicyResolutionInput, PolicyUniverse, resolve_run_policy},
    runtime::actor::messages::ActorOwner,
    security::claims::{UserClaims, UserContext},
};

const NODE: &str = "/opt/homebrew/opt/node@24/bin/node";
const HELPER: &str = "BAUAR_STDIO_HELPER";
const UNDECLARED: &str = "BAUAR_UNDECLARED_PROVIDER_CREDENTIAL";
const DECLARED: &str = "BAUAR_DECLARED_CREDENTIAL";
const STDERR: &str = "BAUAR-RAW-CHILD-STDERR-CANARY";

fn entry(directory: &Path, mode: &str) -> McpServerEntry {
    McpServerEntry::Stdio {
        command: NODE.to_owned(),
        args: vec![
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/bauar_mcp_stdio.mjs")
                .display()
                .to_string(),
            directory.display().to_string(),
            mode.to_owned(),
        ],
        env: HashMap::from([(
            DECLARED.to_owned(),
            "${BAUAR_DECLARED_CREDENTIAL}".to_owned(),
        )]),
        sandboxed: false,
    }
}

fn owner() -> ActorOwner {
    ActorOwner::from_verified_context(&UserContext {
        host_authority: None,
        authority: None,
        user_id: "stdio-owner".to_owned(),
        tenant_id: None,
        claims: UserClaims {
            sub: "stdio-owner".to_owned(),
            name: None,
            roles: Some(vec!["user".to_owned()]),
            tenant_id: None,
            uar_instance_id: None,
            exp: usize::MAX,
        },
    })
    .unwrap()
}

fn receipts(directory: &Path) -> Vec<Value> {
    std::fs::read_to_string(directory.join("receipts.jsonl"))
        .unwrap_or_default()
        .lines()
        .map(|line| serde_json::from_str(line).expect("complete receipt"))
        .collect()
}

fn count(directory: &Path, event: &str) -> usize {
    receipts(directory)
        .iter()
        .filter(|receipt| receipt["event"] == event)
        .count()
}

async fn wait_for(directory: &Path, event: &str, target: usize) {
    tokio::time::timeout(Duration::from_secs(10), async {
        while count(directory, event) < target {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("fixture did not reach controlled boundary");
}

fn captured(directory: &Path, mode: &str) -> (McpRuntimeManager, Arc<McpBindingRequest>) {
    let definition = Arc::new(
        ServerDefinition::new(
            "stdio".to_owned(),
            ServerSource::Skill {
                skill_id: "fixture".to_owned(),
            },
            entry(directory, mode),
            true,
            ServerAuthentication::NotRequired,
        )
        .unwrap(),
    );
    let environment = Arc::new(
        McpBindingEnvironment::new(
            directory.to_path_buf(),
            BTreeMap::from([
                (OsString::from("PATH"), OsString::from("/usr/bin:/bin")),
                (
                    OsString::from(DECLARED),
                    OsString::from("captured-declared-value"),
                ),
                (
                    OsString::from(UNDECLARED),
                    OsString::from("captured-undeclared-value"),
                ),
            ]),
        )
        .unwrap(),
    );
    let request = Arc::new(McpBindingRequest::new(owner(), definition, environment));
    let runtime = McpRuntimeManager::new(
        McpBindingCache::default(),
        Arc::new(ConfiguredMcpConnector::default()),
        Duration::from_secs(15),
        Duration::from_secs(15),
    )
    .unwrap();
    (runtime, request)
}

async fn projected(
    runtime: &McpRuntimeManager,
    request: &McpBindingRequest,
) -> universal_agent_runtime::mcp::preflight::McpPreflight {
    let catalog = McpCatalog::from_definitions([request.definition().as_ref().clone()]).unwrap();
    let policy = resolve_run_policy(PolicyResolutionInput {
        universe: PolicyUniverse {
            skills: ["fixture".to_owned()].into(),
            tools: ["stdio__effect".to_owned()].into(),
            mcp_servers: ["stdio".to_owned()].into(),
            ..PolicyUniverse::default()
        },
        ..PolicyResolutionInput::default()
    });
    let projection = McpServerProjection::resolve(
        &catalog,
        &policy,
        &McpProjectionScope {
            active_skills: ["fixture".to_owned()].into(),
        },
    )
    .unwrap();
    runtime
        .preflight(&projection, request.owner(), request.environment())
        .await
        .unwrap()
}

async fn snapshot_lifetime(directory: &Path) {
    let (runtime, request) = captured(directory, "normal");
    let first = runtime.prepare(request.clone()).await.unwrap();
    assert!(first.catalog().tools().contains_key("stdio__effect"));
    first.retire_connection().await.unwrap();
    wait_for(directory, "eof", 1).await;
    let lazy = projected(&runtime, &request).await;
    assert_eq!(
        count(directory, "started"),
        1,
        "cached preparation launched a process"
    );
    let result = lazy
        .call_tool("stdio__effect", json!({"action":"once"}), None)
        .await
        .unwrap();
    assert!(result.to_string().contains("effect:once"));
    runtime.shutdown().await.unwrap();
    assert_eq!(count(directory, "started"), 2);
    assert_eq!(count(directory, "eof"), 2);
    assert_eq!(count(directory, "effect"), 1);
    assert!(
        runtime.prepare(request).await.is_err(),
        "closed supervisor accepted new work"
    );
    for receipt in receipts(directory)
        .into_iter()
        .filter(|value| value["event"] == "started")
    {
        assert_eq!(receipt["environment"][DECLARED], "captured-declared-value");
        assert!(receipt["environment"].get(UNDECLARED).is_none());
        assert!(receipt["environment"].get(HELPER).is_none());
        assert!(receipt["environment"].get("HOME").is_none());
    }
    assert_eq!(
        count(directory, "invalid-stdin"),
        0,
        "child inherited host stdin"
    );
}

async fn cancelled_lifetime(directory: &Path, during_call: bool) {
    let (runtime, request) = captured(
        directory,
        if during_call {
            "ignore-eof"
        } else {
            "hold-initialize"
        },
    );
    let pending = if during_call {
        let projection = projected(&runtime, &request).await;
        tokio::spawn(async move {
            let _ = projection
                .call_tool("stdio__effect", json!({"action":"hold"}), None)
                .await;
        })
    } else {
        let runtime = runtime.clone();
        tokio::spawn(async move {
            let _ = runtime.prepare(request).await;
        })
    };
    wait_for(
        directory,
        if during_call { "effect" } else { "initialize" },
        1,
    )
    .await;
    pending.abort();
    assert!(pending.await.unwrap_err().is_cancelled());
    // The receiver ignores EOF. Success therefore depends on kill plus reap, including
    // a partial handshake that never yielded a registry-owned RunningService.
    runtime.shutdown().await.unwrap();
    let after_join = count(directory, "heartbeat");
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert_eq!(
        count(directory, "heartbeat"),
        after_join,
        "child survived cleanup join"
    );
    assert_eq!(
        count(directory, "started"),
        1,
        "cancelled work silently retried"
    );
    assert_eq!(count(directory, "effect"), usize::from(during_call));
}

async fn configured_constructor(directory: &Path) {
    let registry = McpRegistry::from_config(&McpConfig {
        mcp_servers: HashMap::from([("stdio".to_owned(), entry(directory, "normal"))]),
    })
    .await
    .unwrap();
    let result = registry
        .call_namespaced_tool("stdio__effect", json!({"action":"exit"}))
        .await
        .unwrap();
    assert!(result.to_string().contains("effect:exit"));
    assert_eq!(count(directory, "effect"), 1);
    let started = receipts(directory)
        .into_iter()
        .find(|receipt| receipt["event"] == "started")
        .unwrap();
    assert_eq!(started["environment"][DECLARED], "ambient-declared-value");
    assert!(started["environment"].get(UNDECLARED).is_none());
    assert_eq!(count(directory, "invalid-stdin"), 0);
}

async fn required_sandbox(directory: &Path) {
    let mut configuration = entry(directory, "normal");
    if let McpServerEntry::Stdio { sandboxed, .. } = &mut configuration {
        *sandboxed = true;
    }
    let definition = ServerDefinition::new(
        "stdio".to_owned(),
        ServerSource::Global,
        configuration.clone(),
        true,
        ServerAuthentication::NotRequired,
    );
    assert!(
        definition
            .unwrap_err()
            .to_string()
            .contains("sandbox backend is unavailable")
    );
    let configured = McpRegistry::from_config(&McpConfig {
        mcp_servers: HashMap::from([("stdio".to_owned(), configuration)]),
    })
    .await;
    assert!(configured.is_err(), "required sandbox was silently ignored");
    assert_eq!(
        count(directory, "started"),
        0,
        "sandbox rejection happened after spawn"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn child_host_helper() {
    let Ok(directory) = std::env::var(HELPER) else {
        return;
    };
    let directory = PathBuf::from(directory);
    for name in [
        "snapshot",
        "initializing",
        "calling",
        "configured",
        "sandbox",
    ] {
        std::fs::create_dir_all(directory.join(name)).unwrap();
    }
    snapshot_lifetime(&directory.join("snapshot")).await;
    cancelled_lifetime(&directory.join("initializing"), false).await;
    cancelled_lifetime(&directory.join("calling"), true).await;
    configured_constructor(&directory.join("configured")).await;
    required_sandbox(&directory.join("sandbox")).await;
}

#[test]
fn stdio_boundary_captures_environment_owns_children_and_discards_raw_stderr() {
    let directory = tempfile::tempdir().unwrap();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", "child_host_helper", "--nocapture"])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env(HELPER, directory.path())
        .env(DECLARED, "ambient-declared-value")
        .env(UNDECLARED, "ambient-provider-value")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"HOST-LOGIN-TOKEN-MUST-NOT-BECOME-MCP-STDIN\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "helper failed\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !String::from_utf8_lossy(&output.stderr).contains(STDERR),
        "raw MCP stderr escaped"
    );
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains(STDERR),
        "raw MCP stderr entered stdout"
    );
    assert_eq!(count(&directory.path().join("snapshot"), "started"), 2);
    assert_eq!(count(&directory.path().join("configured"), "started"), 1);
}
