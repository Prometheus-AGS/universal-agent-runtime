//! `GET /api/uar/capabilities` (tasks 2.29–2.30).

use serde_json::Value;

use crate::sidecar_process::{
    ConfigOptions, JWT_SECRET, LaunchOptions, Workspace, contains_bytes, launch_standalone,
    random_token, raw_request, render_config,
};
use crate::stub_llm::{FixtureSet, start_stub_llm};
use crate::{bearer, boot_sidecar, host};

const ROUTE: &str = "/api/uar/capabilities";

/// The closed capability vocabulary of design Decision 12.
const VOCABULARY: [&str; 33] = [
    "collaboration_representation_execution_v1",
    "team_execution_host_workspace_v1",
    "prometheus.workflow-execution/1.0.0",
    "agui_stream_fidelity",
    "approval_lifecycle_v1",
    "collaboration_definition_packages_v1",
    "collaboration_definition_packages_v2",
    "collaboration_deployment_bindings_v1",
    "collaboration_deployment_bindings_v2",
    "collaboration_conversion_reports_v1",
    "collaboration_representation_grant_refs_v1",
    "collaboration_team_planning_v1",
    "collaboration_team_execution_v1",
    "durable_agent_instances_v1",
    "full_harness_delegation_v1",
    "host_history",
    "host_lifecycle_control_v1",
    "ingest_scoped_credentials",
    "local_scoped_observers_v1",
    "reasoning_effort",
    "run_scoped_credentials",
    "run_scoped_mcp_servers",
    "scoped_delegation_grants_v1",
    "secrets_at_rest",
    "session_principal",
    "service_instance_placement_v1",
    "tool_admission_v2",
    "uar_target_catalog_v1",
    "uar_target_model_override_v1",
    "working_directory",
    "team_execution_peer_tools_v1",
    "team_execution_shared_instructions_v1",
    "team_execution_continuations_v1",
];

/// AG-UI profile revision of the runs stream before `agui-runs-stream-fidelity`.
const AGUI_PROFILE_REVISION: u64 = 1;

fn assert_capabilities_shape(body: &Value) {
    assert_eq!(body["uar_version"], env!("CARGO_PKG_VERSION"), "{body}");
    assert_eq!(body["agui"]["profile"], "uar.agui/1", "{body}");
    assert_eq!(
        body["agui"]["profile_revision"], AGUI_PROFILE_REVISION,
        "{body}"
    );
    let names: Vec<&str> = body["capabilities"]
        .as_array()
        .unwrap_or_else(|| panic!("capabilities array: {body}"))
        .iter()
        .map(|name| name.as_str().expect("capability name is a string"))
        .collect();
    let mut canonical = names.clone();
    canonical.sort_unstable();
    canonical.dedup();
    assert_eq!(names, canonical, "capabilities sorted and unique");
    assert!(
        names.iter().all(|name| VOCABULARY.contains(name)),
        "capability outside the vocabulary: {names:?}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn capabilities_route_requires_launch_token_and_reports_version() {
    crate::sidecar_process::record_check(concat!(module_path!(), "::capabilities_route_requires_launch_token_and_reports_version"), async {
    let mut sidecar = boot_sidecar(FixtureSet::new(), "", |options| options).await;
    let fixture_outcome = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(async {

    let port = sidecar.port();
    let host = sidecar.host();

    let anonymous = raw_request(port, "GET", ROUTE, &[("Host", &host)]).await;
    assert_eq!(anonymous.status, 401, "{}", anonymous.body_text());
    let text = anonymous.body_text();
    assert!(!text.contains(env!("CARGO_PKG_VERSION")), "version leaked");
    assert!(
        VOCABULARY.iter().all(|name| !text.contains(name)),
        "capability name leaked"
    );

    let wrong = raw_request(
        port,
        "GET",
        ROUTE,
        &[("Host", &host), ("Authorization", &bearer(&random_token()))],
    )
    .await;
    assert_eq!(wrong.status, 401, "wrong token");

    let authorization = sidecar.bearer();
    let foreign = raw_request(
        port,
        "GET",
        ROUTE,
        &[
            ("Host", "attacker.example"),
            ("Authorization", &authorization),
        ],
    )
    .await;
    assert_eq!(foreign.status, 403, "foreign Host");
    let origin = raw_request(
        port,
        "GET",
        ROUTE,
        &[
            ("Host", &host),
            ("Authorization", &authorization),
            ("Origin", "https://example.com"),
        ],
    )
    .await;
    assert_eq!(origin.status, 403, "Origin");

    let authorized = raw_request(
        port,
        "GET",
        ROUTE,
        &[("Host", &host), ("Authorization", &authorization)],
    )
    .await;
    assert_eq!(authorized.status, 200, "{}", authorized.body_text());
    let body: Value = serde_json::from_slice(&authorized.body).expect("capabilities JSON");
    assert_capabilities_shape(&body);
    assert!(body["capabilities"].as_array().unwrap().iter().any(|name| name == "host_lifecycle_control_v1"));
    let snapshot = sidecar.process.lifecycle(serde_json::json!({"kind":"snapshot"})).await;
    assert_eq!(snapshot["version"], 1);
    assert!(snapshot["error"].is_null(), "launch control failed: {snapshot}");
    assert!(snapshot["result"]["resources"].as_array().is_some_and(|resources| !resources.is_empty()));
    assert!(snapshot["result"]["subscriptions"].is_object());
    let rejected = sidecar.process.lifecycle(serde_json::json!({"kind":"reconnect",
        "resource_id":"unregistered-fixture-resource", "expected_generation":0})).await;
    assert_eq!(rejected["error"], "lifecycle_command_failed");
    })).await;
    let fixture_cleanup = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(sidecar.process.stop_sidecar())).await;
    crate::sidecar_process::finish_fixture(fixture_outcome, fixture_cleanup)

    }).await;
}

fn mint_jwt() -> String {
    let _ = jsonwebtoken::crypto::rust_crypto::DEFAULT_PROVIDER.install_default();
    let claims = universal_agent_runtime::uar::security::claims::UserClaims {
        sub: "capabilities-reader".to_owned(),
        name: None,
        roles: Some(vec!["user".to_owned()]),
        tenant_id: None,
        uar_instance_id: None,
        exp: usize::MAX,
    };
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
    .expect("mint JWT")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn capabilities_route_is_not_auth_exempt_in_standalone() {
    crate::sidecar_process::record_check(concat!(module_path!(), "::capabilities_route_is_not_auth_exempt_in_standalone"), async {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let workspace = Workspace::new();
    let provider_key = format!("sk-capabilities-{}", uuid::Uuid::new_v4().simple());
    let mut options = ConfigOptions::new(&stub.base_url);
    options.jwt_required = true;
    let port = options.port;
    let config = render_config(&workspace, &options)
        .replace("llm:\n", &format!("llm:\n  api_key: \"{provider_key}\"\n"));
    let config = workspace.write_config("config.yaml", &config);
    let mut standalone =
        launch_standalone(&workspace, &LaunchOptions::new(config, "standalone"), port).await;

    let anonymous = raw_request(port, "GET", ROUTE, &[("Host", &host(port))]).await;
    assert_eq!(anonymous.status, 401, "{}", anonymous.body_text());

    let jwt = mint_jwt();
    let authorized = raw_request(
        port,
        "GET",
        ROUTE,
        &[("Host", &host(port)), ("Authorization", &bearer(&jwt))],
    )
    .await;
    assert_eq!(authorized.status, 200, "{}", authorized.body_text());
    let body: Value = serde_json::from_slice(&authorized.body).expect("capabilities JSON");
    assert_capabilities_shape(&body);
    for secret in [
        workspace.data().to_string_lossy().into_owned(),
        JWT_SECRET.to_owned(),
        provider_key,
    ] {
        assert!(
            !contains_bytes(&authorized.body, secret.as_bytes()),
            "capabilities body exposes a configuration value"
        );
    }
    let status = standalone.stop_standalone().await;
    assert!(status.success(), "standalone exit: {status}");
    }).await;
}
