//! The launch token never leaves sidecar memory (tasks 2.19–2.20).

use std::time::Duration;

use futures::FutureExt;
use tokio::io::{AsyncBufReadExt as _, AsyncWriteExt as _};

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

use crate::sidecar_process::{
    RawResponse, contains_bytes, drive_run, files_containing, probe_records, random_token,
    raw_request, test_agent,
};
use crate::stub_llm::FixtureSet;
use crate::{boot_sidecar, probe_command, with_terminal_call};

/// What a same-user inspector can read of one process.
#[derive(Debug, Clone)]
struct ProcessView {
    pid: u32,
    cmd: Vec<String>,
    environ: Vec<String>,
}

/// `root` and every descendant, read through `sysinfo`.
fn process_tree(root: u32) -> Vec<ProcessView> {
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing()
            .with_cmd(UpdateKind::Always)
            .with_environ(UpdateKind::Always),
    );
    let processes = system.processes();
    let mut tree = Vec::new();
    let mut frontier = vec![Pid::from_u32(root)];
    while let Some(pid) = frontier.pop() {
        if let Some(process) = processes.get(&pid) {
            tree.push(ProcessView {
                pid: pid.as_u32(),
                cmd: process
                    .cmd()
                    .iter()
                    .map(|part| part.to_string_lossy().into_owned())
                    .collect(),
                environ: process
                    .environ()
                    .iter()
                    .map(|entry| entry.to_string_lossy().into_owned())
                    .collect(),
            });
        }
        frontier.extend(
            processes
                .iter()
                .filter(|(_, candidate)| candidate.parent() == Some(pid))
                .map(|(child, _)| *child),
        );
    }
    tree
}

fn exposes(view: &ProcessView, needle: &str) -> bool {
    view.cmd
        .iter()
        .chain(view.environ.iter())
        .any(|part| part.contains(needle))
}

const CONFINEMENT_INPUT: &str = "run the environment probe";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn token_absent_from_argv_and_env_of_sidecar_and_children() {
    crate::sidecar_process::record_check(concat!(module_path!(), "::token_absent_from_argv_and_env_of_sidecar_and_children"), async {
    let sentinel = format!("sentinel{}", uuid::Uuid::new_v4().simple());
    let outputs = tempfile::tempdir().expect("probe output dir");
    let probe_output = outputs.path().join("terminal.jsonl");
    let control = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("probe control listener");
    let control_token = random_token();
    let command = format!("{} --control {} --control-token {}", probe_command(&probe_output),
        control.local_addr().expect("probe control address"), control_token);
    let fixtures = with_terminal_call(FixtureSet::new(), CONFINEMENT_INPUT, &command);
    let sentinel_for_launch = sentinel.clone();
    let mut sidecar = boot_sidecar(fixtures, "", move |mut options| {
        let renamed = options
            .config_path
            .with_file_name(format!("config-{sentinel_for_launch}.yaml"));
        std::fs::copy(&options.config_path, &renamed).expect("copy config");
        options.config_path = renamed;
        options.env("UAR_PROBE_SENTINEL", sentinel_for_launch)
    })
    .await;
    let fixture_outcome = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(async {

    let sidecar_pid = sidecar.process.pid();

    // Sentinel control: the inspector must be able to read argv and environ,
    // or an absence assertion below would pass vacuously.
    let own = process_tree(sidecar_pid)
        .into_iter()
        .find(|view| view.pid == sidecar_pid)
        .expect("sidecar visible in the process table");
    assert!(
        own.environ
            .iter()
            .any(|entry| entry == &format!("UAR_PROBE_SENTINEL={sentinel}")),
        "sidecar environ unreadable: {own:?}"
    );
    assert!(
        own.cmd.iter().any(|part| part.contains(&sentinel)),
        "sidecar argv unreadable: {own:?}"
    );

    let base_url = sidecar.base_url();
    let run = std::panic::AssertUnwindSafe(drive_run(
        &base_url, Some(&sidecar.token), test_agent(None), CONFINEMENT_INPUT,
        Duration::from_secs(90),
    )).catch_unwind();
    tokio::pin!(run);
    let (stream, _) = tokio::select! {
        accepted = control.accept() => accepted.expect("accept probe control"),
        outcome = &mut run => match outcome {
            Err(panic) => std::panic::resume_unwind(panic),
            Ok(transcript) => panic!("run ended before probe readiness: {}", transcript.stream),
        },
    };
    let mut control = tokio::io::BufReader::new(stream);
    let mut ready = String::new();
    control.read_line(&mut ready).await.expect("read probe readiness");
    let (nonce, pid) = ready.trim().split_once(' ').expect("probe readiness frame");
    assert_eq!(nonce, control_token, "probe control identity");
    let probe_pid: u32 = pid.parse().expect("probe pid");
    let children: Vec<_> = process_tree(sidecar_pid).into_iter()
        .filter(|view| view.pid != sidecar_pid).collect();
    let saw_probe = children.iter().any(|view| view.pid == probe_pid);
    control.get_mut().write_all(b"RELEASE\n").await.expect("release probe");
    let transcript = match run.await {
        Ok(transcript) => transcript,
        Err(panic) => std::panic::resume_unwind(panic),
    };
    assert!(saw_probe, "ready probe was not a sidecar descendant");

    assert!(
        children
            .iter()
            .any(|view| view.environ.iter().any(|entry| entry.contains(&sentinel))),
        "no child with a readable inherited environment was observed: {children:?}\n{}",
        transcript.stream
    );
    let mut leaks: Vec<u32> = std::iter::once(&own)
        .chain(children.iter())
        .filter(|view| exposes(view, &sidecar.token))
        .map(|view| view.pid)
        .collect();
    leaks.dedup();
    assert!(leaks.is_empty(), "token visible in processes {leaks:?}");

    let records = probe_records(&probe_output);
    assert!(
        !records.is_empty(),
        "terminal probe did not run\n{}",
        transcript.stream
    );
    for record in &records {
        assert!(
            record["env"]["UAR_PROBE_SENTINEL"] == sentinel.as_str(),
            "probe environment unreadable: {record}"
        );
        assert!(
            !record.to_string().contains(&sidecar.token),
            "token visible to the terminal child"
        );
    }
    })).await;
    let fixture_cleanup = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(sidecar.process.stop_sidecar())).await;
    crate::sidecar_process::finish_fixture(fixture_outcome, fixture_cleanup)

    }).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn token_absent_from_output_logs_files_and_responses() {
    crate::sidecar_process::record_check(concat!(module_path!(), "::token_absent_from_output_logs_files_and_responses"), async {
    let mut sidecar = boot_sidecar(FixtureSet::new(), "", |options| {
        let log_file = options
            .config_path
            .parent()
            .expect("workspace root")
            .join("logs")
            .join("uar.log");
        options.env("UAR_LOG_FILE", log_file.to_string_lossy().into_owned())
    })
    .await;
    let fixture_outcome = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(async {

    let port = sidecar.port();
    let host_value = sidecar.host();
    let host = host_value.as_str();
    let authorization_value = sidecar.bearer();
    let authorization = authorization_value.as_str();
    let wrong_value = format!("Bearer {}", random_token());
    let wrong = wrong_value.as_str();
    let requests: Vec<(&str, &str, Vec<(&str, &str)>)> = vec![
        (
            "GET",
            "/readyz",
            vec![("Host", host), ("Authorization", authorization)],
        ),
        ("GET", "/readyz", vec![("Host", host)]),
        (
            "GET",
            "/readyz",
            vec![("Host", host), ("Authorization", wrong)],
        ),
        (
            "GET",
            "/readyz",
            vec![
                ("Host", "attacker.example"),
                ("Authorization", authorization),
            ],
        ),
        (
            "GET",
            "/readyz",
            vec![
                ("Host", host),
                ("Authorization", authorization),
                ("Origin", "https://example.com"),
            ],
        ),
        ("GET", "/api/uar/sync/stream", vec![("Host", host)]),
        (
            "GET",
            "/api/uar/sync/stream",
            vec![("Host", host), ("Authorization", authorization)],
        ),
        (
            "GET",
            "/no-such-path",
            vec![("Host", host), ("Authorization", authorization)],
        ),
        (
            "POST",
            "/api/uar/runs",
            vec![
                ("Host", host),
                ("Authorization", authorization),
                ("Content-Type", "application/json"),
            ],
        ),
    ];
    let mut responses: Vec<RawResponse> = Vec::new();
    for (method, path, headers) in &requests {
        responses.push(raw_request(port, method, path, headers).await);
    }
    let status = sidecar.process.stop_sidecar().await;
    assert!(status.success(), "sidecar exit: {status}");

    let token = sidecar.token.clone();
    for needle in [token.clone(), format!("Bearer {token}")] {
        let files = files_containing(sidecar.workspace.path(), needle.as_bytes());
        assert!(files.is_empty(), "token found in files: {files:?}");
        for (index, response) in responses.iter().enumerate() {
            assert!(
                !contains_bytes(&response.all_bytes(), needle.as_bytes()),
                "token echoed in response {index}"
            );
        }
    }
    })).await;
    let fixture_cleanup = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(sidecar.process.stop_sidecar())).await;
    crate::sidecar_process::finish_fixture(fixture_outcome, fixture_cleanup)

    }).await;
}
