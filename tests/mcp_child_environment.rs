//! Child-process environment and stdin rules in every mode
//! (sidecar-launch-security tasks 2.21, 2.22 and 2.31).
//!
//! 2.21 and 2.22 need the *test process's* environment to hold a sentinel and
//! its stdin to be a pipe holding bytes. Each test therefore re-runs this test
//! binary as a child (`child_process_helper`) with exactly that environment
//! and stdin, and the helper drives the real spawn paths in-process.

#[path = "support/sidecar_process.rs"]
mod sidecar_process;
#[path = "integration/live/stub_llm.rs"]
mod stub_llm;

use std::collections::{BTreeMap, HashMap};
use std::ffi::OsString;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value, json};
use universal_agent_runtime::mcp::binding_cache::{McpBindingEnvironment, McpBindingRequest};
use universal_agent_runtime::mcp::catalog::{ServerAuthentication, ServerDefinition, ServerSource};
use universal_agent_runtime::mcp::config::{McpConfig, McpServerEntry};
use universal_agent_runtime::mcp::registry::McpRegistry;
use universal_agent_runtime::mcp::runtime::{ConfiguredMcpConnector, McpConnector as _};
use universal_agent_runtime::uar::orchestrator::provisioning::{
    GitInstallSpec, PerOsPackageName, ProvisionOptions, ToolProvisioner, ToolSpec,
};
use universal_agent_runtime::uar::runtime::actor::messages::ActorOwner;
use universal_agent_runtime::uar::runtime::native_skill::NativeSkill as _;
use universal_agent_runtime::uar::security::claims::{UserClaims, UserContext};
use universal_agent_runtime::uar::tools::terminal_exec::TerminalExecTool;

use sidecar_process::{
    ConfigOptions, LaunchOptions, STUB_MODEL, Workspace, drive_run, launch_standalone,
    probe_binary, probe_records, render_config, test_agent,
};
use stub_llm::{FixtureResponse, FixtureSet, RequestFingerprint, start_stub_llm};

const HELPER_ENV: &str = "UAR_MCP_CHILD_HELPER";
const OUTPUT_ENV: &str = "UAR_MCP_CHILD_OUTPUT";
const SENTINEL_KEY: &str = "UAR_UNDECLARED_SENTINEL";
const DECLARED_KEY: &str = "UAR_DECLARED_VAR";
const DECLARED_VALUE: &str = "declared-value";
const HOST_STDIN: &[u8] = b"host-bytes-that-a-child-must-never-read\n";
/// The stdio MCP launch allowlist of design Decision 6.
const LAUNCH_ALLOWLIST: [&str; 8] = [
    "PATH",
    "PATHEXT",
    "SYSTEMROOT",
    "WINDIR",
    "COMSPEC",
    "TMPDIR",
    "TEMP",
    "TMP",
];

// ─── Child helper ───────────────────────────────────────────────────────────

/// Runs only as the child process the tests below start; a no-op otherwise.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn child_process_helper() {
    let Ok(scenario) = std::env::var(HELPER_ENV) else {
        return;
    };
    let output = PathBuf::from(std::env::var(OUTPUT_ENV).expect("helper output directory"));
    match scenario.as_str() {
        "spawn-paths" => drive_stdio_spawn_paths(&output).await,
        "stdin" => drive_non_protocol_children(&output).await,
        other => panic!("unknown helper scenario {other}"),
    }
}

fn probe_entry(output: &Path) -> McpServerEntry {
    McpServerEntry::Stdio {
        command: probe_binary().to_owned(),
        args: vec![output.to_string_lossy().into_owned(), "--mcp".to_owned()],
        env: HashMap::from([(DECLARED_KEY.to_owned(), DECLARED_VALUE.to_owned())]),
        sandboxed: false,
    }
}

fn verified_user() -> UserContext {
    UserContext {
        user_id: "child-environment-user".to_owned(),
        tenant_id: None,
        claims: UserClaims {
            sub: "child-environment-user".to_owned(),
            name: None,
            roles: Some(vec!["user".to_owned()]),
            tenant_id: None,
            uar_instance_id: None,
            exp: usize::MAX,
        },
    }
}

async fn drive_stdio_spawn_paths(output: &Path) {
    // Configured-server path (`from_config` → `connect_configured_server`).
    let config = McpConfig {
        mcp_servers: HashMap::from([(
            "probe".to_owned(),
            probe_entry(&output.join("legacy.jsonl")),
        )]),
    };
    let registry = McpRegistry::from_config(&config)
        .await
        .expect("configured stdio server connects");
    assert!(
        registry
            .tools()
            .iter()
            .any(|(name, _)| name == "probe__probe"),
        "probe tool not listed"
    );
    // The probe exits on `tools/call`; the transport loss makes the registry
    // reconnect through `connect_server`, which spawns a second probe.
    let _ = tokio::time::timeout(
        Duration::from_secs(30),
        registry.call_namespaced_tool("probe__probe", json!({})),
    )
    .await;

    // Projected snapshot path (`connect_stdio_snapshot`).
    let captured = BTreeMap::from([
        (
            OsString::from("PATH"),
            std::env::var_os("PATH").unwrap_or_default(),
        ),
        (
            OsString::from(SENTINEL_KEY),
            std::env::var_os(SENTINEL_KEY).unwrap_or_default(),
        ),
        (OsString::from(DECLARED_KEY), OsString::from(DECLARED_VALUE)),
    ]);
    let environment = Arc::new(
        McpBindingEnvironment::new(output.to_path_buf(), captured).expect("binding environment"),
    );
    let definition = Arc::new(
        ServerDefinition::new(
            "snapshot-probe".to_owned(),
            ServerSource::Skill {
                skill_id: "probe-skill".to_owned(),
            },
            probe_entry(&output.join("snapshot.jsonl")),
            true,
            ServerAuthentication::NotRequired,
        )
        .expect("snapshot definition"),
    );
    let owner = ActorOwner::from_verified_context(&verified_user()).expect("verified owner");
    let request = Arc::new(McpBindingRequest::new(owner, definition, environment));
    let connector = ConfiguredMcpConnector::default();
    let _ = tokio::time::timeout(Duration::from_secs(30), connector.connect(request)).await;
    let _ = connector.shutdown().await;
}

fn leak(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

fn git(repository: &Path, arguments: &[&str]) {
    let status = Command::new("git")
        .args([
            "-c",
            "user.email=probe@example.com",
            "-c",
            "user.name=probe",
        ])
        .args(arguments)
        .current_dir(repository)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("run git");
    assert!(status.success(), "git {arguments:?}: {status}");
}

async fn drive_non_protocol_children(output: &Path) {
    let tool = TerminalExecTool {
        shell: "sh".to_owned(),
        timeout_secs: 10,
        use_sandbox: false,
    };
    let command = format!(
        "'{}' '{}'",
        probe_binary(),
        output.join("terminal.jsonl").display()
    );
    let _ = tool.execute(json!({ "command": command })).await;

    let repository = output.join("fixture-repository");
    std::fs::create_dir_all(&repository).expect("create fixture repository");
    std::fs::write(repository.join("README"), "fixture").expect("write fixture file");
    git(&repository, &["init", "-q"]);
    git(&repository, &["add", "."]);
    git(&repository, &["commit", "-q", "-m", "fixture"]);
    let build_command: &'static [&'static str] = Box::leak(
        vec![
            leak(probe_binary().to_owned()),
            leak(
                output
                    .join("provision.jsonl")
                    .to_string_lossy()
                    .into_owned(),
            ),
        ]
        .into_boxed_slice(),
    );
    let spec = ToolSpec {
        name: leak(format!("uar-probe-tool-{}", uuid::Uuid::new_v4().simple())),
        native_pkg: PerOsPackageName::default(),
        git_install: Some(GitInstallSpec {
            url: leak(repository.to_string_lossy().into_owned()),
            build_cmd: build_command,
            binary_relpath: "binary-that-the-probe-never-builds",
        }),
        prebuilt: None,
    };
    let options = ProvisionOptions {
        allow_install: true,
        cache_dir: output.join("provision-cache"),
    };
    // The build step is the probe; resolution then fails on the missing binary.
    let _ = ToolProvisioner::resolve(&spec, &options).await;
}

/// Re-run this test binary as `child_process_helper` for `scenario`.
fn run_helper(scenario: &str, output: &Path, sentinel: &str, stdin: Option<&[u8]>) {
    let mut child = Command::new(std::env::current_exe().expect("test executable"))
        .args([
            "--exact",
            "child_process_helper",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(HELPER_ENV, scenario)
        .env(OUTPUT_ENV, output)
        .env(SENTINEL_KEY, sentinel)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn helper");
    let mut pipe = child.stdin.take().expect("helper stdin");
    if let Some(bytes) = stdin {
        pipe.write_all(bytes).expect("write helper stdin");
        pipe.flush().expect("flush helper stdin");
    }
    // Keep the pipe open (and its bytes unread) until the helper exits.
    let output = child.wait_with_output().expect("wait for helper");
    drop(pipe);
    assert!(
        output.status.success(),
        "helper {scenario} failed: {}\n{}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

// ─── Assertions ─────────────────────────────────────────────────────────────

fn assert_launch_environment(record: &Value, label: &str) {
    let environment = record["env"]
        .as_object()
        .unwrap_or_else(|| panic!("{label}: no env in {record}"));
    assert_eq!(
        environment.get(DECLARED_KEY).and_then(Value::as_str),
        Some(DECLARED_VALUE),
        "{label}: declared variable missing"
    );
    assert!(
        !environment.contains_key(SENTINEL_KEY),
        "{label}: undeclared parent variable inherited"
    );
    let unexpected: Vec<&String> = environment
        .keys()
        .filter(|key| key.as_str() != DECLARED_KEY && !LAUNCH_ALLOWLIST.contains(&key.as_str()))
        .collect();
    assert!(
        unexpected.is_empty(),
        "{label}: keys outside the allowlist: {unexpected:?}"
    );
}

fn sentinel() -> String {
    format!("sentinel-{}", uuid::Uuid::new_v4().simple())
}

#[test]
fn stdio_mcp_children_get_allowlisted_env_on_every_spawn_path() {
    let output = tempfile::tempdir().expect("probe output directory");
    run_helper("spawn-paths", output.path(), &sentinel(), None);

    let legacy = probe_records(&output.path().join("legacy.jsonl"));
    assert!(
        legacy.len() >= 2,
        "expected the configured spawn and the reconnect spawn, saw {}",
        legacy.len()
    );
    for (index, record) in legacy.iter().enumerate() {
        let label = if index == 0 {
            "connect_configured_server"
        } else {
            "connect_server (reconnect)"
        };
        assert_launch_environment(record, label);
    }
    let snapshot = probe_records(&output.path().join("snapshot.jsonl"));
    assert!(!snapshot.is_empty(), "snapshot path did not spawn");
    for record in &snapshot {
        assert_launch_environment(record, "connect_stdio_snapshot");
    }
}

#[test]
fn child_processes_read_eof_from_stdin() {
    let output = tempfile::tempdir().expect("probe output directory");
    run_helper("stdin", output.path(), &sentinel(), Some(HOST_STDIN));
    for (file, label) in [
        ("terminal.jsonl", "terminal tool"),
        ("provision.jsonl", "provisioning build command"),
    ] {
        let records = probe_records(&output.path().join(file));
        assert!(!records.is_empty(), "{label} did not run the probe");
        for record in &records {
            assert_eq!(record["stdin"], "eof", "{label} read the host's stdin");
        }
    }
}

// ─── Standalone server (2.31) ───────────────────────────────────────────────

const STANDALONE_INPUT: &str = "run the probe from the terminal tool";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn standalone_stdio_mcp_child_env_and_terminal_stdin() {
    let workspace = Workspace::new();
    let mcp_output = workspace.path().join("mcp.jsonl");
    let terminal_output = workspace.path().join("terminal.jsonl");
    let command = format!("'{}' '{}'", probe_binary(), terminal_output.display());
    let fingerprint = |has_tool_result| RequestFingerprint {
        model: STUB_MODEL.to_owned(),
        last_user_message: STANDALONE_INPUT.to_owned(),
        has_tools: true,
        has_tool_result,
    };
    let fixtures = FixtureSet::new()
        .with(
            fingerprint(false),
            FixtureResponse::ToolCall {
                name: "terminal_exec".to_owned(),
                arguments: json!({ "command": command }).to_string(),
            },
        )
        .with(
            fingerprint(true),
            FixtureResponse::Content("done".to_owned()),
        );
    let stub = start_stub_llm(fixtures).await;
    std::fs::write(
        workspace.work().join("mcp.json"),
        json!({ "mcpServers": { "probe": {
            "command": probe_binary(),
            "args": [mcp_output.to_string_lossy(), "--mcp"],
            "env": { DECLARED_KEY: DECLARED_VALUE },
        } } })
        .to_string(),
    )
    .expect("write mcp.json");
    let options = ConfigOptions::new(&stub.base_url);
    let port = options.port;
    let config = workspace.write_config("config.yaml", &render_config(&workspace, &options));
    let mut launch = LaunchOptions::new(config, "standalone").env(SENTINEL_KEY, sentinel());
    launch.stdin_bytes = Some(HOST_STDIN.to_vec());
    let mut standalone = launch_standalone(&workspace, &launch, port).await;

    let transcript = drive_run(
        &standalone.base_url(),
        None,
        test_agent(None),
        STANDALONE_INPUT,
        Duration::from_secs(90),
    )
    .await;
    let status = standalone.stop_standalone().await;
    assert!(status.success(), "standalone exit: {status}");

    let mcp_records = probe_records(&mcp_output);
    assert!(
        !mcp_records.is_empty(),
        "configured stdio MCP server did not start"
    );
    for record in &mcp_records {
        assert_launch_environment(record, "standalone stdio MCP server");
    }
    let terminal_records = probe_records(&terminal_output);
    assert!(
        !terminal_records.is_empty(),
        "terminal tool did not run the probe\n{}",
        transcript.stream
    );
    for record in &terminal_records {
        assert_eq!(
            record["stdin"], "eof",
            "terminal command read the host's stdin"
        );
    }
}
