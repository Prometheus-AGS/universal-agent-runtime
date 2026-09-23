//! Terminal-run eviction (tasks 1.10–1.14).

use std::time::Duration;

use reqwest::Method;
use serde_json::json;

use crate::principal_host::{
    APPROVAL_MARKER, Host, P1, ask_policy, boot, reply, terminal_call,
};
use crate::sidecar_process::test_agent;
use crate::stub_llm::FixtureSet;

const RETAINED: &str = "uar_runs_retained";
const REPLAY_RUNS: &str = "uar_a2ui_replay_runs";
const WAIT: Duration = Duration::from_secs(60);
const TERMINAL: [&str; 3] = [
    "event: agui.done",
    "event: agui.error",
    "event: agui.cancelled",
];

/// Run settings with a one-second sweep.
fn runs_yaml(retention_secs: u64, max_retained: u64) -> String {
    format!(
        "runs:\n  retention_after_terminal_secs: {retention_secs}\n  \
         max_retained_terminal: {max_retained}\n  sweep_interval_secs: 1\n"
    )
}

async fn trigger_surface(host: &Host, run_id: &str) -> u16 {
    host.call(
        Method::POST,
        &format!("/api/uar/runs/{run_id}/a2ui/surface-test-trigger"),
        Some(P1),
        Some(json!({
            "surface_id": "retention-surface",
            "kind": "createSurface",
            "payload": { "catalogId": "urn:uar:a2ui:catalog:1" },
        })),
    )
    .await
    .0
}

async fn evicted_within(host: &Host, run_id: &str, timeout: Duration) -> bool {
    Host::wait_for(timeout, || async move { !host.run_exists(Some(P1), run_id).await }).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn terminal_run_is_evicted_after_retention() {
    let first = "retention turn one FACT-LILAC";
    let next = "retention turn two";
    let fixtures = reply(reply(FixtureSet::new(), first, "noted"), next, "recalled");
    let host = boot(fixtures, &runs_yaml(3, 1000), |options| options).await;
    let run = host.drive(Some(P1), test_agent(None), first, Some("s1")).await;
    assert_eq!(trigger_surface(&host, &run.run_id).await, 200, "surface");
    tokio::time::sleep(Duration::from_millis(1_200)).await;
    let retained_before = host.gauge(RETAINED).await.expect("retained-runs gauge");
    assert!(retained_before >= 1.0, "{RETAINED} = {retained_before}");

    assert!(
        evicted_within(&host, &run.run_id, Duration::from_secs(10)).await,
        "run retained past retention plus two sweeps"
    );
    let base = format!("/api/uar/runs/{}", run.run_id);
    assert_eq!(host.open_stream(Some(P1), &run.run_id).await.err(), Some(404));
    for (method, path, body) in [
        (Method::POST, format!("{base}/cancel"), None),
        (Method::GET, format!("{base}/checkpoints"), None),
        (
            Method::POST,
            format!("{base}/resume"),
            Some(json!({ "artifact": test_agent(None), "input": next })),
        ),
        (
            Method::POST,
            format!("{base}/tool-approval"),
            Some(json!({ "approved": true })),
        ),
        (Method::GET, format!("{base}/a2ui/surface-replay"), None),
        (
            Method::POST,
            format!("{base}/a2ui/actions"),
            Some(json!({
                "surfaceId": "retention-surface",
                "name": "submit",
                "sourceComponentId": "button",
            })),
        ),
    ] {
        let (status, text) = host.call(method.clone(), &path, Some(P1), body).await;
        assert_eq!(status, 404, "{method} {path} after eviction: {text}");
    }
    let retained_after = host.gauge(RETAINED).await.expect("retained-runs gauge");
    assert_eq!(retained_after, retained_before - 1.0, "{RETAINED}");

    // The session outlives its evicted run.
    host.drive(Some(P1), test_agent(None), next, Some("s1")).await;
    let requests = host.model_requests_for(next).await;
    assert!(
        !requests.is_empty() && requests.iter().all(|request| request.contains("FACT-LILAC")),
        "the next run lost the session's prior turns"
    );
}

/// Task 1.11, reduced: a running (approval-paused) run without subscribers
/// outlives the retention period.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn attached_subscriber_and_live_runs_are_not_evicted() {
    let input = "a run that waits past retention";
    let host = boot(
        terminal_call(FixtureSet::new(), input, "true"),
        &runs_yaml(1, 1000),
        |options| options,
    )
    .await;
    // No stream is opened, so no disconnect guard can cancel the run.
    let run_id = host
        .start_run(Some(P1), test_agent(Some(ask_policy())), input, None)
        .await;
    tokio::time::sleep(Duration::from_secs(5)).await;
    assert!(
        host.run_exists(Some(P1), &run_id).await,
        "a running run was evicted"
    );
    let stream = host.follow(Some(P1), &run_id, WAIT).await;
    assert!(
        stream.contains(APPROVAL_MARKER) && stream.contains("event: agui.done"),
        "the run did not wait and then finish\n{stream}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cap_evicts_oldest_terminal_runs_first() {
    let inputs: Vec<String> = (1..=5).map(|index| format!("capped run {index}")).collect();
    let fixtures = inputs
        .iter()
        .fold(FixtureSet::new(), |set, input| reply(set, input, "ok"));
    let host = boot(fixtures, &runs_yaml(3_600, 3), |options| options).await;
    let mut run_ids = Vec::new();
    for input in &inputs {
        run_ids.push(host.drive(Some(P1), test_agent(None), input, None).await.run_id);
    }
    let (host_ref, oldest) = (&host, &run_ids[..2]);
    let settled = Host::wait_for(Duration::from_secs(10), || async move {
        !host_ref.run_exists(Some(P1), &oldest[0]).await
            && !host_ref.run_exists(Some(P1), &oldest[1]).await
    })
    .await;
    assert!(settled, "the two oldest runs are still retained");
    for run_id in &run_ids[2..] {
        assert!(
            host.run_exists(Some(P1), run_id).await,
            "a run within the cap was evicted"
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn retained_state_is_bounded_under_sustained_load() {
    let input = "sustained load run";
    let host = boot(
        terminal_call(FixtureSet::new(), input, "true"),
        &runs_yaml(0, 50),
        |options| options,
    )
    .await;
    let artifact = test_agent(Some(ask_policy()));
    for index in 0..500 {
        let run_id = host.start_run(Some(P1), artifact.clone(), input, None).await;
        let mut stream = host
            .open_stream(Some(P1), &run_id)
            .await
            .unwrap_or_else(|status| panic!("run {index} stream: {status}"));
        let mut text = String::new();
        assert!(
            Host::read_until(&mut stream, &mut text, &[APPROVAL_MARKER], WAIT).await,
            "run {index} did not pause\n{text}"
        );
        assert_eq!(trigger_surface(&host, &run_id).await, 200, "run {index} surface");
        assert_eq!(host.approve(Some(P1), &run_id).await, 200, "run {index} approval");
        assert!(
            Host::read_until(&mut stream, &mut text, &TERMINAL, WAIT).await,
            "run {index} did not finish\n{text}"
        );
    }
    tokio::time::sleep(Duration::from_millis(2_500)).await;
    let retained = host.gauge(RETAINED).await.expect("retained-runs gauge");
    let replays = host.gauge(REPLAY_RUNS).await.expect("replay-runs gauge");
    assert!(retained <= 50.0, "{RETAINED} = {retained}");
    assert!(replays <= 50.0, "{REPLAY_RUNS} = {replays}");
}

/// Task 1.14, reduced: `agui-runs-stream-fidelity` has not landed, so no
/// lag-ended stream exists. A host that reconnects with its last event id
/// after the run finished still receives the remainder.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn lagged_host_can_resync_a_just_finished_run() {
    let input = "resync after finish";
    let host = boot(reply(FixtureSet::new(), input, "ok"), "", |options| options).await;
    let run = host.drive(Some(P1), test_agent(None), input, None).await;
    let first_id = *event_ids(&run.stream)
        .first()
        .unwrap_or_else(|| panic!("no event id on the stream\n{}", run.stream));
    tokio::time::sleep(Duration::from_secs(1)).await;
    let mut response = host
        .request(
            Method::GET,
            &format!("/api/uar/runs/{}/stream", run.run_id),
            Some(P1),
        )
        .header("Last-Event-ID", first_id.to_string())
        .send()
        .await
        .expect("reconnect");
    assert_eq!(response.status(), 200, "reconnect status");
    let mut text = String::new();
    assert!(
        Host::read_until(&mut response, &mut text, &TERMINAL, WAIT).await,
        "the remainder did not arrive\n{text}"
    );
    let replayed = event_ids(&text);
    assert!(
        !replayed.is_empty() && replayed.iter().all(|id| *id > first_id),
        "the replay is not the remainder after {first_id}: {replayed:?}"
    );
}

/// The `id:` fields of an SSE body, in order.
fn event_ids(stream: &str) -> Vec<u64> {
    stream
        .lines()
        .filter_map(|line| line.strip_prefix("id:"))
        .filter_map(|id| id.trim().parse().ok())
        .collect()
}
