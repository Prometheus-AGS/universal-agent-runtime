//! Governance stays mandatory for a token-authenticated sidecar
//! (tasks 2.16–2.18).

use std::time::Duration;

use serde_json::{Value, json};

use crate::sidecar_process::{
    ConfigOptions, LaunchOptions, Workspace, drive_run, launch_sidecar, launch_standalone,
    probe_records, random_token, render_config, test_agent,
};
use crate::stub_llm::{FixtureSet, start_stub_llm};
use crate::{Sidecar, boot_sidecar, probe_command, with_terminal_call};

async fn get_json(base_url: &str, bearer: Option<&str>, path: &str) -> Value {
    let client = reqwest::Client::new();
    let mut request = client.get(format!("{base_url}{path}"));
    if let Some(token) = bearer {
        request = request.bearer_auth(token);
    }
    let response = request.send().await.expect("GET request");
    let status = response.status();
    let body = response.text().await.expect("GET body");
    assert!(status.is_success(), "GET {path}: {status} {body}");
    serde_json::from_str(&body).expect("GET JSON")
}

async fn persisted_governance(base_url: &str, bearer: Option<&str>) -> Value {
    get_json(base_url, bearer, "/api/uar/settings/governance.enabled").await["data"].clone()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sidecar_governance_required_with_host_token_reason() {
    let sidecar = boot_sidecar(FixtureSet::new(), "", |options| options).await;
    let base = sidecar.base_url();
    let status = get_json(
        &base,
        Some(&sidecar.token),
        "/api/uar/settings/governance/status",
    )
    .await;
    assert_eq!(status["effective_state"], "required", "{status}");
    assert!(
        status["reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| reason == "host_token_required")),
        "{status}"
    );
    assert_eq!(status["jwt_required"], false, "{status}");
    assert_eq!(
        persisted_governance(&base, Some(&sidecar.token)).await,
        json!(true)
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sidecar_normalizes_persisted_governance_off_and_refuses_off() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let workspace = Workspace::new();
    let options = ConfigOptions::new(&stub.base_url);
    let port = options.port;
    let config = workspace.write_config("config.yaml", &render_config(&workspace, &options));

    // A standalone loopback boot without JWT seeds governance Off.
    let mut standalone = launch_standalone(
        &workspace,
        &LaunchOptions::new(config.clone(), "seed"),
        port,
    )
    .await;
    let seeded = persisted_governance(&standalone.base_url(), None).await;
    assert_eq!(seeded, json!(false), "standalone seed");
    let status = standalone.stop_standalone().await;
    assert!(status.success(), "standalone exit: {status}");

    let token = random_token();
    let sidecar = launch_sidecar(
        &workspace,
        &LaunchOptions::new(config, "sidecar").with_token(&token),
    )
    .await;
    let base = sidecar.base_url();
    assert_eq!(persisted_governance(&base, Some(&token)).await, json!(true));

    let response = reqwest::Client::new()
        .put(format!("{base}/api/uar/settings/governance"))
        .bearer_auth(&token)
        .json(&json!({ "data": { "enabled": false } }))
        .send()
        .await
        .expect("save governance off");
    let body: Value = response.json().await.expect("governance save JSON");
    let result = body["results"]
        .as_array()
        .and_then(|results| {
            results
                .iter()
                .find(|result| result["key"] == "governance.enabled")
        })
        .cloned()
        .unwrap_or_else(|| panic!("no governance.enabled result: {body}"));
    assert_eq!(result["status"], "validation_rejected", "{body}");
    assert_eq!(persisted_governance(&base, Some(&token)).await, json!(true));
}

const DENIED_INPUT: &str = "call the denied tool";
const DENY_ALL_INPUT: &str = "call a tool under a deny-all policy";

async fn run_denied(sidecar: &Sidecar, policy: Value, input: &str) -> String {
    drive_run(
        &sidecar.base_url(),
        Some(&sidecar.token),
        test_agent(Some(policy)),
        input,
        Duration::from_secs(60),
    )
    .await
    .stream
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sidecar_run_policy_denied_tool_is_not_executed() {
    let outputs = tempfile::tempdir().expect("probe output dir");
    let listed_output = outputs.path().join("listed.jsonl");
    let deny_all_output = outputs.path().join("deny-all.jsonl");
    let fixtures = with_terminal_call(
        with_terminal_call(
            FixtureSet::new(),
            DENIED_INPUT,
            &probe_command(&listed_output),
        ),
        DENY_ALL_INPUT,
        &probe_command(&deny_all_output),
    );
    let sidecar = boot_sidecar(fixtures, "", |options| options).await;

    // `tools.deny` removes the tool from the run's visible tool set.
    let listed = run_denied(
        &sidecar,
        json!({ "tools": { "mode": "inherit", "ids": [], "denied_ids": ["terminal_exec"] } }),
        DENIED_INPUT,
    )
    .await;
    assert!(
        probe_records(&listed_output).is_empty(),
        "tool listed in tools.deny executed\n{listed}"
    );

    // A deny approval policy is the run-policy denial the governance gate
    // enforces; with governance bypassed the call would run.
    let deny_all = run_denied(&sidecar, json!({ "tool_approval": "deny" }), DENY_ALL_INPUT).await;
    assert!(
        probe_records(&deny_all_output).is_empty(),
        "tool executed under a deny policy\n{deny_all}"
    );
    assert!(
        deny_all.contains("event: agui.tool_call.denied"),
        "denial not recorded\n{deny_all}"
    );
}

/// Regression guard: standalone loopback without JWT stays operator-optional.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn standalone_loopback_without_jwt_remains_governance_optional() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let workspace = Workspace::new();
    let options = ConfigOptions::new(&stub.base_url);
    let port = options.port;
    let config = workspace.write_config("config.yaml", &render_config(&workspace, &options));
    let mut standalone =
        launch_standalone(&workspace, &LaunchOptions::new(config, "standalone"), port).await;
    let status = get_json(
        &standalone.base_url(),
        None,
        "/api/uar/settings/governance/status",
    )
    .await;
    assert_eq!(status["may_disable"], true, "{status}");
    assert_ne!(status["effective_state"], "required", "{status}");
    assert!(
        status["reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().all(|reason| reason != "host_token_required")),
        "{status}"
    );
    let exit = standalone.stop_standalone().await;
    assert!(exit.success(), "standalone exit: {exit}");
}
