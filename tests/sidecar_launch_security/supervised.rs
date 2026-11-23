//! Supervised launch from the standalone binary, OS-assigned ports and the
//! build identity route (OpenSpec change `host-supervised-launch`, issue #539).
//!
//! The launch token is the behaviour under test here: each case acts as the
//! parent process, generates a random token for one launch and never stores it.

use std::time::Duration;

use serde_json::Value;

use crate::sidecar_process::{
    ConfigOptions, JWT_SECRET, LaunchOptions, Workspace, contains_bytes, free_port,
    launch_standalone, launch_supervised, random_token, raw_request, render_config,
    spawn_supervised,
};
use crate::stub_llm::{FixtureSet, start_stub_llm};
use crate::surface::listening_sockets;
use crate::{bearer, boot_sidecar, host, write_config};

const VERSION_ROUTE: &str = "/api/uar/version";

const REJECTION_TIMEOUT: Duration = Duration::from_secs(5);

/// Longest a supervised launch or exit may take before the case fails. The
/// harness waits without a limit, and a standalone binary that ignored
/// `--supervised` would start a server that never prints `READY` and never
/// exits on stdin EOF; this turns that hang into a failure.
const SUPERVISED_STEP_LIMIT: Duration = Duration::from_secs(300);

async fn within_limit<T>(step: &str, future: impl std::future::Future<Output = T>) -> T {
    tokio::time::timeout(SUPERVISED_STEP_LIMIT, future)
        .await
        .unwrap_or_else(|_| panic!("{step} did not finish within {SUPERVISED_STEP_LIMIT:?}"))
}

/// Exit status of a launch refused for a missing or malformed token.
const INVALID_TOKEN_EXIT_STATUS: i32 = 2;

/// Lowest port an operating system hands out for a port-0 bind on a supported
/// platform (Linux starts at 32768, macOS and Windows at 49152). The
/// preferred-port walk starts at 1906, so a port up here can only have come
/// from the operating system.
const LOWEST_OS_ASSIGNED_PORT: u16 = 32_768;

fn request_os_assigned_port(mut options: LaunchOptions) -> LaunchOptions {
    options
        .extra_args
        .extend(["--port".to_owned(), "0".to_owned()]);
    options
}

fn assert_version_shape(body: &Value) {
    assert_eq!(body["name"], "universal-agent-runtime", "{body}");
    assert_eq!(body["version"], env!("CARGO_PKG_VERSION"), "{body}");
    assert!(
        matches!(body["build_profile"].as_str(), Some("debug" | "release")),
        "{body}"
    );
    assert!(
        body["target"]
            .as_str()
            .is_some_and(|target| !target.is_empty()),
        "{body}"
    );
    let features = body["features"]
        .as_array()
        .unwrap_or_else(|| panic!("features array: {body}"));
    assert!(features.iter().any(|feature| feature == "server"), "{body}");
    assert!(
        body["git_commit"].is_null() || body["git_commit"].is_string(),
        "{body}"
    );
    assert_eq!(body["protocols"]["ag_ui_profile"], "uar.agui/1", "{body}");
    assert!(body["protocols"]["mcp"].is_string(), "{body}");
    assert!(body["administration_schema"].is_u64(), "{body}");
}

/// R1, R3, R4: the standalone binary run with `--supervised --port 0` serves
/// one token-guarded loopback listener on an OS-assigned port, reports it in
/// a single `READY` line, and exits cleanly when stdin closes.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn standalone_binary_supervised_serves_one_guarded_listener_on_an_os_assigned_port() {
    crate::sidecar_process::record_check(concat!(module_path!(), "::standalone_binary_supervised_serves_one_guarded_listener_on_an_os_assigned_port"), async {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let workspace = Workspace::new();
    let config = write_config(&workspace, &stub.base_url, "");
    let token = random_token();
    let options =
        request_os_assigned_port(LaunchOptions::new(config, "supervised").with_token(&token));
    let mut process =
        within_limit("supervised launch", launch_supervised(&workspace, &options)).await;
    let fixture_outcome = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(async {

    let port = process.port();
    assert!(port >= LOWEST_OS_ASSIGNED_PORT, "READY reported port {port}");
    assert_eq!(
        listening_sockets(process.pid()),
        vec![format!("127.0.0.1:{port}")],
        "a supervised process owns exactly the reported loopback listener"
    );

    let anonymous = raw_request(port, "GET", "/readyz", &[("Host", &host(port))]).await;
    assert_eq!(anonymous.status, 401, "{}", anonymous.body_text());
    let wrong = raw_request(
        port,
        "GET",
        "/readyz",
        &[("Host", &host(port)), ("Authorization", &bearer(&random_token()))],
    )
    .await;
    assert_eq!(wrong.status, 401, "wrong token");
    let authorized = raw_request(
        port,
        "GET",
        "/readyz",
        &[("Host", &host(port)), ("Authorization", &bearer(&token))],
    )
    .await;
    assert_eq!(authorized.status, 200, "{}", authorized.body_text());
    })).await;
    let fixture_cleanup = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(async {
        let status = within_limit("exit after stdin closed", process.stop_sidecar()).await;
        assert_eq!(process.ready_lines().len(), 1, "stdout: {}", process.stdout());
        assert!(
            process
                .stdout()
                .lines()
                .all(|line| line.starts_with("READY:") || line.starts_with("LIFECYCLE:")),
            "stdout carries only control lines: {}",
            process.stdout()
        );
        status
    })).await;
    crate::sidecar_process::finish_fixture(fixture_outcome, fixture_cleanup)

    }).await;
}

/// R2: with no token line the supervised standalone binary exits with the
/// launch-refusal status and never reports readiness.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn standalone_binary_supervised_without_a_token_exits_with_status_two() {
    crate::sidecar_process::record_check(concat!(module_path!(), "::standalone_binary_supervised_without_a_token_exits_with_status_two"), async {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let workspace = Workspace::new();
    let config = write_config(&workspace, &stub.base_url, "");
    // No stdin bytes: the harness closes stdin immediately.
    let mut process = spawn_supervised(
        &workspace,
        &request_os_assigned_port(LaunchOptions::new(config, "supervised-no-token")),
    )
    .await;
    let status = within_limit("launch refusal", process.wait_exit(REJECTION_TIMEOUT))
        .await
        .unwrap_or_else(|| panic!("supervised runtime still running\n{}", process.stdout()));
    assert_eq!(
        status.code(),
        Some(INVALID_TOKEN_EXIT_STATUS),
        "stderr: {}",
        process.stderr()
    );
    // The argument parser also exits with status 2; only the supervised
    // entry point reports a refused launch.
    assert!(
        process.stderr().contains("refused to start"),
        "stderr: {}",
        process.stderr()
    );
    assert!(
        process.ready_lines().is_empty(),
        "stdout: {}",
        process.stdout()
    );
    }).await;
}

/// The standalone binary loads no `.env` file on its own in supervised mode.
/// `PORT` in a `.env` beside the process would otherwise become its `--port`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn standalone_binary_supervised_ignores_a_dotenv_file_in_its_working_directory() {
    crate::sidecar_process::record_check(concat!(module_path!(), "::standalone_binary_supervised_ignores_a_dotenv_file_in_its_working_directory"), async {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let workspace = Workspace::new();
    let config = write_config(&workspace, &stub.base_url, "");
    let dotenv_port = free_port();
    std::fs::write(workspace.work().join(".env"), format!("PORT={dotenv_port}\n"))
        .expect("write .env beside the process");
    let token = random_token();
    let options = LaunchOptions::new(config, "supervised-dotenv").with_token(&token);
    let mut process =
        within_limit("supervised launch", launch_supervised(&workspace, &options)).await;
    let fixture_outcome = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(async {

    assert_ne!(
        process.port(),
        dotenv_port,
        "the supervised standalone binary took its port from ./.env"
    );
    })).await;
    let fixture_cleanup = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(
        within_limit("exit after stdin closed", process.stop_sidecar()),
    )).await;
    crate::sidecar_process::finish_fixture(fixture_outcome, fixture_cleanup)

    }).await;
}

/// R3: `uar-sidecar --port 0` takes an OS-assigned port instead of walking
/// upward from 1906.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sidecar_port_zero_binds_an_os_assigned_port() {
    crate::sidecar_process::record_check(concat!(module_path!(), "::sidecar_port_zero_binds_an_os_assigned_port"), async {
    let mut sidecar = boot_sidecar(FixtureSet::new(), "", request_os_assigned_port).await;
    let fixture_outcome = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(async {

    let port = sidecar.port();
    assert!(port >= LOWEST_OS_ASSIGNED_PORT, "READY reported port {port}");
    let authorized = raw_request(
        port,
        "GET",
        "/readyz",
        &[("Host", &sidecar.host()), ("Authorization", &sidecar.bearer())],
    )
    .await;
    assert_eq!(authorized.status, 200, "{}", authorized.body_text());
    })).await;
    let fixture_cleanup = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(sidecar.process.stop_sidecar())).await;
    crate::sidecar_process::finish_fixture(fixture_outcome, fixture_cleanup)

    }).await;
}

/// R6: on a standalone server that requires a JWT everywhere else, the build
/// identity is readable with no credentials and exposes no configuration.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn version_route_needs_no_credentials_on_a_standalone_server() {
    crate::sidecar_process::record_check(concat!(module_path!(), "::version_route_needs_no_credentials_on_a_standalone_server"), async {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let workspace = Workspace::new();
    let mut options = ConfigOptions::new(&stub.base_url);
    options.jwt_required = true;
    let port = options.port;
    let config = workspace.write_config("config.yaml", &render_config(&workspace, &options));
    let mut standalone =
        launch_standalone(&workspace, &LaunchOptions::new(config, "standalone"), port).await;

    // Control: credentials are enforced on this server.
    let guarded =
        raw_request(port, "GET", "/api/uar/capabilities", &[("Host", &host(port))]).await;
    assert_eq!(guarded.status, 401, "{}", guarded.body_text());

    let anonymous = raw_request(port, "GET", VERSION_ROUTE, &[("Host", &host(port))]).await;
    assert_eq!(anonymous.status, 200, "{}", anonymous.body_text());
    let body: Value = serde_json::from_slice(&anonymous.body).expect("version JSON");
    assert_version_shape(&body);
    for secret in [
        workspace.path().to_string_lossy().into_owned(),
        JWT_SECRET.to_owned(),
    ] {
        assert!(
            !contains_bytes(&anonymous.body, secret.as_bytes()),
            "version body exposes a configuration value"
        );
    }

    // Only `GET` on the exact path is credential-free. Authentication runs
    // before routing, so anything else is refused with 401; a method-agnostic
    // or prefix exemption would reach the router and answer 405 or 404.
    let write = raw_request(port, "POST", VERSION_ROUTE, &[("Host", &host(port))]).await;
    assert_eq!(write.status, 401, "{}", write.body_text());
    let near_miss =
        raw_request(port, "GET", "/api/uar/version/", &[("Host", &host(port))]).await;
    assert_eq!(near_miss.status, 401, "{}", near_miss.body_text());

    let status = standalone.stop_standalone().await;
    assert!(status.success(), "standalone exit: {status}");
    }).await;
}

/// R7: a supervised process requires its launch token for the build identity
/// like every other route.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn version_route_requires_the_launch_token_when_supervised() {
    crate::sidecar_process::record_check(concat!(module_path!(), "::version_route_requires_the_launch_token_when_supervised"), async {
    let mut sidecar = boot_sidecar(FixtureSet::new(), "", |options| options).await;
    let fixture_outcome = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(async {

    let port = sidecar.port();
    let host = sidecar.host();
    let anonymous = raw_request(port, "GET", VERSION_ROUTE, &[("Host", &host)]).await;
    assert_eq!(anonymous.status, 401, "{}", anonymous.body_text());
    assert!(
        !anonymous.body_text().contains(env!("CARGO_PKG_VERSION")),
        "version leaked without the launch token"
    );

    let authorized = raw_request(
        port,
        "GET",
        VERSION_ROUTE,
        &[("Host", &host), ("Authorization", &sidecar.bearer())],
    )
    .await;
    assert_eq!(authorized.status, 200, "{}", authorized.body_text());
    let body: Value = serde_json::from_slice(&authorized.body).expect("version JSON");
    assert_version_shape(&body);
    })).await;
    let fixture_cleanup = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(sidecar.process.stop_sidecar())).await;
    crate::sidecar_process::finish_fixture(fixture_outcome, fixture_cleanup)

    }).await;
}
