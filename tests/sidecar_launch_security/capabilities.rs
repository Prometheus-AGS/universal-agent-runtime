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
const VOCABULARY: [&str; 9] = [
    "agui_stream_fidelity",
    "host_history",
    "ingest_scoped_credentials",
    "reasoning_effort",
    "run_scoped_credentials",
    "run_scoped_mcp_servers",
    "secrets_at_rest",
    "session_principal",
    "working_directory",
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
    let sidecar = boot_sidecar(FixtureSet::new(), "", |options| options).await;
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
}
