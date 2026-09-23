//! Launch token handoff over stdin (tasks 2.1–2.5).

use std::time::Duration;

use crate::sidecar_process::{
    LaunchOptions, Workspace, contains_bytes, launch_sidecar, random_token, raw_request,
    sidecar_binary, spawn,
};
use crate::stub_llm::{FixtureSet, start_stub_llm};
use crate::{boot_sidecar, host, write_config};

const REJECTION_TIMEOUT: Duration = Duration::from_secs(5);

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn launch_with_valid_token_prints_single_ready() {
    let sidecar = boot_sidecar(FixtureSet::new(), "", |options| options).await;
    // A second READY would arrive right after the first; give it the chance.
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(
        sidecar.process.ready_lines().len(),
        1,
        "stdout: {}",
        sidecar.process.stdout()
    );

    let port = sidecar.port();
    let authorized = raw_request(
        port,
        "GET",
        "/readyz",
        &[("Host", &host(port)), ("Authorization", &sidecar.bearer())],
    )
    .await;
    assert_eq!(authorized.status, 200, "{}", authorized.body_text());

    let anonymous = raw_request(port, "GET", "/readyz", &[("Host", &host(port))]).await;
    assert_eq!(anonymous.status, 401, "{}", anonymous.body_text());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn launch_without_token_line_exits_nonzero_without_ready() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let workspace = Workspace::new();
    let config = write_config(&workspace, &stub.base_url, "");
    // No stdin bytes: the harness closes stdin immediately.
    let mut process = spawn(
        sidecar_binary(),
        &workspace,
        &LaunchOptions::new(config, "no-token"),
    );
    let status = process
        .wait_exit(REJECTION_TIMEOUT)
        .await
        .unwrap_or_else(|| panic!("sidecar still running\n{}", process.stdout()));
    assert!(!status.success(), "exit status: {status}");
    assert!(
        process.ready_lines().is_empty(),
        "stdout: {}",
        process.stdout()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn launch_with_malformed_token_exits_nonzero_without_echo() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let valid = random_token();
    let cases = [
        ("short", valid[..63].to_owned()),
        ("long", format!("{valid}a")),
        ("uppercase", valid.to_ascii_uppercase()),
        ("non-hex", "z".repeat(64)),
        ("empty", String::new()),
    ];
    for (tag, line) in cases {
        let workspace = Workspace::new();
        let config = write_config(&workspace, &stub.base_url, "");
        let mut options = LaunchOptions::new(config, tag);
        options.stdin_bytes = Some(format!("{line}\n").into_bytes());
        let mut process = spawn(sidecar_binary(), &workspace, &options);
        let status = process
            .wait_exit(REJECTION_TIMEOUT)
            .await
            .unwrap_or_else(|| panic!("{tag}: sidecar still running\n{}", process.stdout()));
        assert!(!status.success(), "{tag}: exit status {status}");
        assert!(
            process.ready_lines().is_empty(),
            "{tag}: stdout {}",
            process.stdout()
        );
        if !line.is_empty() {
            for (stream, text) in [("stdout", process.stdout()), ("stderr", process.stderr())] {
                assert!(
                    !contains_bytes(text.as_bytes(), line.as_bytes()),
                    "{tag}: {stream} echoed the received line"
                );
            }
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn launch_ignores_token_shaped_environment_variable() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let workspace = Workspace::new();
    let config = write_config(&workspace, &stub.base_url, "");
    let options = LaunchOptions::new(config, "env-token")
        .env("UAR_SIDECAR_TOKEN", random_token())
        .env("UAR_TOKEN", random_token());
    let mut process = spawn(sidecar_binary(), &workspace, &options);
    let status = process
        .wait_exit(REJECTION_TIMEOUT)
        .await
        .unwrap_or_else(|| panic!("sidecar still running\n{}", process.stdout()));
    assert!(!status.success(), "exit status: {status}");
    assert!(
        process.ready_lines().is_empty(),
        "stdout: {}",
        process.stdout()
    );
}

/// Regression guard: stdin EOF after the token still ends the process.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stdin_eof_after_ready_exits_cleanly() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let workspace = Workspace::new();
    let config = write_config(&workspace, &stub.base_url, "");
    let token = random_token();
    let mut process = launch_sidecar(
        &workspace,
        &LaunchOptions::new(config, "eof").with_token(&token),
    )
    .await;
    process.close_stdin();
    let status = process
        .wait_exit(Duration::from_secs(30))
        .await
        .unwrap_or_else(|| panic!("sidecar ignored stdin EOF\n{}", process.stderr()));
    assert!(status.success(), "exit status: {status}");
}
