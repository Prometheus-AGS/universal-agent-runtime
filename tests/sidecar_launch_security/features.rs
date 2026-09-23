//! Logging, cross-session shared features and identity (tasks 2.24–2.28).

use std::time::Duration;

use serde_json::{Value, json};
use universal_agent_runtime::uar::persistence::PersistenceLayer as _;
use universal_agent_runtime::uar::persistence::providers::surreal::SurrealDbProvider;

use crate::sidecar_process::{
    ConfigOptions, LaunchOptions, Workspace, drive_run, launch_sidecar, launch_standalone,
    probe_binary, probe_records, random_token, render_config, test_agent,
};
use crate::stub_llm::{FixtureSet, start_stub_llm};
use crate::{boot_sidecar, with_reply, with_terminal_call};

fn log_file_env(options: LaunchOptions) -> LaunchOptions {
    let log_file = options
        .config_path
        .parent()
        .expect("workspace root")
        .join("logs")
        .join("uar.log");
    options.env("UAR_LOG_FILE", log_file.to_string_lossy().into_owned())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sidecar_default_logs_contain_no_prompt_or_tool_argument_canary() {
    let canary = format!("CANARY-{}", uuid::Uuid::new_v4().simple());
    let input = format!("please echo {canary}");
    let fixtures = with_terminal_call(FixtureSet::new(), &input, &format!("echo {canary}"));
    let mut sidecar = boot_sidecar(fixtures, "", log_file_env).await;
    let transcript = drive_run(
        &sidecar.base_url(),
        Some(&sidecar.token),
        test_agent(None),
        &input,
        Duration::from_secs(90),
    )
    .await;
    assert!(
        transcript.stream.contains("event: agui.tool_result")
            && transcript.stream.contains("event: agui.done"),
        "run did not complete a tool call\n{}",
        transcript.stream
    );
    let status = sidecar.process.stop_sidecar().await;
    assert!(status.success(), "sidecar exit: {status}");

    let log_file = sidecar.workspace.logs().join("uar.log");
    for (name, text) in [
        ("stdout", sidecar.process.stdout()),
        ("stderr", sidecar.process.stderr()),
        (
            "UAR_LOG_FILE",
            std::fs::read_to_string(&log_file).unwrap_or_default(),
        ),
    ] {
        let lines: Vec<&str> = text.lines().filter(|line| line.contains(&canary)).collect();
        assert!(
            lines.is_empty(),
            "{name} carries conversation content: {lines:#?}"
        );
    }
}

/// Regression guard: an explicit `RUST_LOG` stays authoritative.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sidecar_honors_explicit_rust_log() {
    let input = "debug logging probe";
    let fixtures = with_reply(FixtureSet::new(), input, "ok");
    let sidecar = boot_sidecar(fixtures, "", |options| {
        options.env("RUST_LOG", "universal_agent_runtime=debug")
    })
    .await;
    let transcript = drive_run(
        &sidecar.base_url(),
        Some(&sidecar.token),
        test_agent(None),
        input,
        Duration::from_secs(60),
    )
    .await;
    assert!(
        transcript.stream.contains("event: agui.done"),
        "{}",
        transcript.stream
    );
    let stdout = sidecar.process.stdout();
    assert!(
        stdout
            .lines()
            .any(|line| line.contains("\"level\":\"DEBUG\"")
                && line.contains("\"target\":\"universal_agent_runtime")),
        "no universal_agent_runtime debug event on stdout"
    );
}

fn feature_yaml(workspace: &Workspace) -> String {
    format!(
        "skill_evolution:\n  enabled: true\n  min_tool_calls: 1\n\
         memory:\n  enabled: true\n  db_path: \"{}\"\n",
        workspace.path().join("memory-db").display()
    )
}

async fn server_names(base_url: &str, bearer: Option<&str>) -> Vec<String> {
    let client = reqwest::Client::new();
    let mut request = client.get(format!("{base_url}/api/uar/mcp/servers"));
    if let Some(token) = bearer {
        request = request.bearer_auth(token);
    }
    let body: Value = request
        .send()
        .await
        .expect("list MCP servers")
        .json()
        .await
        .expect("MCP servers JSON");
    body["servers"]
        .as_array()
        .unwrap_or_else(|| panic!("no servers array: {body}"))
        .iter()
        .filter_map(|server| server["name"].as_str().map(str::to_owned))
        .collect()
}

async fn skill_ids(base_url: &str, token: &str) -> Vec<String> {
    let body: Value = reqwest::Client::new()
        .get(format!("{base_url}/api/uar/skills"))
        .bearer_auth(token)
        .send()
        .await
        .expect("list skills")
        .json()
        .await
        .expect("skills JSON");
    let mut ids: Vec<String> = body
        .as_array()
        .unwrap_or_else(|| panic!("skills list: {body}"))
        .iter()
        .map(Value::to_string)
        .collect();
    ids.sort();
    ids
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sidecar_forces_skill_evolution_memory_and_global_mcp_off() {
    let input = "evolve a skill from this run";
    let fixtures = with_terminal_call(FixtureSet::new(), input, "echo evolution");
    let stub = start_stub_llm(fixtures).await;
    let workspace = Workspace::new();
    let file_probe = workspace.path().join("file-probe.jsonl");
    let seeded_probe = workspace.path().join("seeded-probe.jsonl");
    std::fs::write(
        workspace.work().join("mcp.json"),
        json!({ "mcpServers": { "file-probe": {
            "command": probe_binary(),
            "args": [file_probe.to_string_lossy(), "--mcp"],
        } } })
        .to_string(),
    )
    .expect("write mcp.json");
    let mut options = ConfigOptions::new(&stub.base_url);
    options.extra_yaml = feature_yaml(&workspace);
    let port = options.port;
    let config = workspace.write_config("config.yaml", &render_config(&workspace, &options));

    // Seed a persisted global MCP server through a standalone boot.
    let mut standalone = launch_standalone(
        &workspace,
        &LaunchOptions::new(config.clone(), "seed"),
        port,
    )
    .await;
    let seeded = reqwest::Client::new()
        .put(format!(
            "{}/api/uar/mcp/servers/seeded-probe",
            standalone.base_url()
        ))
        .json(&json!({
            "name": "seeded-probe",
            "transport": "stdio",
            "command": probe_binary(),
            "args": [seeded_probe.to_string_lossy(), "--mcp"],
        }))
        .send()
        .await
        .expect("seed MCP server");
    assert!(seeded.status().is_success(), "seed: {}", seeded.status());
    let standalone_servers = server_names(&standalone.base_url(), None).await;
    assert!(
        standalone_servers.contains(&"seeded-probe".to_owned()),
        "precondition: {standalone_servers:?}"
    );
    let status = standalone.stop_standalone().await;
    assert!(status.success(), "standalone exit: {status}");
    let spawned_before = probe_records(&file_probe).len() + probe_records(&seeded_probe).len();

    let token = random_token();
    let sidecar = launch_sidecar(
        &workspace,
        &LaunchOptions::new(config, "sidecar").with_token(&token),
    )
    .await;
    let base = sidecar.base_url();
    assert_eq!(
        server_names(&base, Some(&token)).await,
        Vec::<String>::new(),
        "global MCP servers listed"
    );
    let memory = reqwest::Client::new()
        .get(format!("{base}/api/memory?q=anything"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("memory search");
    assert_eq!(memory.status(), 503, "memory service constructed");

    let skills_before = skill_ids(&base, &token).await;
    let transcript = drive_run(
        &base,
        Some(&token),
        test_agent(None),
        input,
        Duration::from_secs(90),
    )
    .await;
    assert!(
        transcript.stream.contains("event: agui.tool_result"),
        "run made no tool call\n{}",
        transcript.stream
    );
    tokio::time::sleep(Duration::from_secs(3)).await;
    assert_eq!(
        skill_ids(&base, &token).await,
        skills_before,
        "skill evolved"
    );
    assert!(
        !transcript.stream.contains("skill_evolution"),
        "skill evolution event emitted"
    );
    assert_eq!(
        probe_records(&file_probe).len() + probe_records(&seeded_probe).len(),
        spawned_before,
        "sidecar spawned a global MCP server"
    );
}

async fn settings_value(base: &str, token: &str, key: &str) -> Value {
    let body: Value = reqwest::Client::new()
        .get(format!("{base}/api/uar/settings/{key}"))
        .bearer_auth(token)
        .send()
        .await
        .expect("read setting")
        .json()
        .await
        .expect("setting JSON");
    body["data"].clone()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sidecar_rejects_global_mcp_mutation_and_feature_enable() {
    let sidecar = boot_sidecar(FixtureSet::new(), "", |options| options).await;
    let base = sidecar.base_url();
    let client = reqwest::Client::new();
    let probe_output = sidecar.workspace.path().join("mutation-probe.jsonl");

    let put = client
        .put(format!("{base}/api/uar/mcp/servers/x"))
        .bearer_auth(&sidecar.token)
        .json(&json!({
            "name": "x",
            "transport": "stdio",
            "command": probe_binary(),
            "args": [probe_output.to_string_lossy(), "--mcp"],
        }))
        .send()
        .await
        .expect("PUT MCP server");
    assert_eq!(put.status(), 409, "PUT status");
    let put_body = put.text().await.expect("PUT body");
    assert!(
        put_body.contains("sidecar_mode_global_mcp_disabled"),
        "PUT body: {put_body}"
    );
    let delete = client
        .delete(format!("{base}/api/uar/mcp/servers/x"))
        .bearer_auth(&sidecar.token)
        .send()
        .await
        .expect("DELETE MCP server");
    assert_eq!(delete.status(), 409, "DELETE status");
    let delete_body = delete.text().await.expect("DELETE body");
    assert!(
        delete_body.contains("sidecar_mode_global_mcp_disabled"),
        "DELETE body: {delete_body}"
    );
    assert!(
        probe_records(&probe_output).is_empty(),
        "MCP server was started"
    );
    assert!(
        server_names(&base, Some(&sidecar.token)).await.is_empty(),
        "MCP server registered"
    );

    for (namespace, key) in [
        ("skill-evolution", "skill_evolution.enabled"),
        ("memory", "memory.enabled"),
    ] {
        let saved: Value = client
            .put(format!("{base}/api/uar/settings/{namespace}"))
            .bearer_auth(&sidecar.token)
            .json(&json!({ "data": { "enabled": true } }))
            .send()
            .await
            .expect("save feature setting")
            .json()
            .await
            .expect("feature setting JSON");
        assert_ne!(saved["status"], "updated", "{key} accepted: {saved}");
        assert_eq!(
            settings_value(&base, &sidecar.token, key).await,
            json!(false),
            "{key} effective value"
        );
    }
}

/// Regression guard: the launch token authenticates the host, not a user.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sidecar_authorized_request_is_anonymous_principal() {
    let mut sidecar = boot_sidecar(FixtureSet::new(), "", |options| options).await;
    let name = format!("owner-probe-{}", uuid::Uuid::new_v4().simple());
    let created = reqwest::Client::new()
        .post(format!("{}/api/uar/knowledge-bases", sidecar.base_url()))
        .bearer_auth(&sidecar.token)
        .json(&json!({ "name": name, "description": "owner probe" }))
        .send()
        .await
        .expect("create knowledge base");
    assert!(
        created.status().is_success(),
        "create: {}",
        created.status()
    );
    let status = sidecar.process.stop_sidecar().await;
    assert!(status.success(), "sidecar exit: {status}");

    let provider = SurrealDbProvider::new(
        &format!("surrealkv://{}", sidecar.workspace.data().display()),
        None,
        None,
        None,
        None,
    )
    .await
    .expect("reopen sidecar database");
    let stored = provider
        .get_knowledge_base_by_name("anonymous", &name)
        .await
        .expect("query knowledge base");
    assert!(
        stored.is_some(),
        "record not owned by the anonymous principal"
    );
}
