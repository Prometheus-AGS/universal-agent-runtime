//! Idle-session eviction (tasks 1.16–1.19).

use std::time::Duration;

use crate::principal_host::{
    APPROVAL_MARKER, Host, P1, P2, ask_policy, boot, boot_standalone, logged_field, reply,
    terminal_call,
};
use crate::sidecar_process::{LaunchOptions, test_agent};
use crate::stub_llm::FixtureSet;

const SESSIONS: &str = "uar_active_sessions";
const WAIT: Duration = Duration::from_secs(60);
const TERMINAL: [&str; 3] = [
    "event: agui.done",
    "event: agui.error",
    "event: agui.cancelled",
];
const RETENTION_MESSAGE: &str = "Run and session retention configured";
/// A one-second sweep; run retention stays at its default.
const FAST_SWEEP: &str = "runs:\n  sweep_interval_secs: 1\n";

fn idle_timeout(seconds: u64) -> impl FnOnce(LaunchOptions) -> LaunchOptions {
    move |options| options.env("UAR_SESSIONS__IDLE_TIMEOUT_SECS", seconds.to_string())
}

fn cap(sessions: u64) -> impl FnOnce(LaunchOptions) -> LaunchOptions {
    move |options| {
        options
            .env("UAR_SESSIONS__IDLE_TIMEOUT_SECS", "0")
            .env("UAR_SESSIONS__MAX_RETAINED", sessions.to_string())
    }
}

async fn sessions_reach(host: &Host, count: f64, timeout: Duration) -> bool {
    Host::wait_for(timeout, || async move {
        host.gauge(SESSIONS).await == Some(count)
    })
    .await
}

/// Whether the model request for `input` carried `fact` from earlier turns.
async fn request_carries(host: &Host, input: &str, fact: &str) -> bool {
    let requests = host.model_requests_for(input).await;
    assert!(!requests.is_empty(), "no model request for {input}");
    requests.iter().any(|request| request.contains(fact))
}

/// Task 1.16, reduced: the host-history half needs `run-request-host-context`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn idle_session_is_evicted_and_reseeded_from_host_history() {
    let first = "FACT codename=HELIOTROPE";
    let other = "P2 records FACT-SALT";
    let (ask, ask_other) = ("what is the codename", "P2 asks again");
    let fixtures = [first, other, ask, ask_other]
        .into_iter()
        .fold(FixtureSet::new(), |set, input| reply(set, input, "ok"));
    let host = boot(fixtures, FAST_SWEEP, idle_timeout(1)).await;
    host.drive(Some(P1), test_agent(None), first, Some("s1")).await;
    host.drive(Some(P2), test_agent(None), other, Some("s1")).await;
    assert_eq!(host.gauge(SESSIONS).await, Some(2.0), "{SESSIONS} before");

    assert!(
        sessions_reach(&host, 0.0, Duration::from_secs(10)).await,
        "idle sessions were not evicted: {:?}",
        host.gauge(SESSIONS).await
    );
    host.drive(Some(P1), test_agent(None), ask, Some("s1")).await;
    assert!(
        !request_carries(&host, ask, "HELIOTROPE").await,
        "the evicted session's turns reached the next run"
    );
    host.drive(Some(P2), test_agent(None), ask_other, Some("s1")).await;
    assert!(!request_carries(&host, ask_other, "FACT-SALT").await);
    assert!(!request_carries(&host, ask_other, "HELIOTROPE").await);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn session_with_a_live_run_is_not_evicted() {
    let input = "a session waiting on approval";
    let host = boot(
        terminal_call(FixtureSet::new(), input, "true"),
        FAST_SWEEP,
        idle_timeout(1),
    )
    .await;
    let run_id = host
        .start_run(Some(P1), test_agent(Some(ask_policy())), input, Some("s1"))
        .await;
    let mut stream = host
        .open_stream(Some(P1), &run_id)
        .await
        .unwrap_or_else(|status| panic!("stream: {status}"));
    let mut text = String::new();
    assert!(
        Host::read_until(&mut stream, &mut text, &[APPROVAL_MARKER], WAIT).await,
        "run did not pause\n{text}"
    );
    tokio::time::sleep(Duration::from_secs(5)).await;
    assert_eq!(
        host.gauge(SESSIONS).await,
        Some(1.0),
        "a session with a waiting run was evicted"
    );
    assert_eq!(host.approve(Some(P1), &run_id).await, 200);
    assert!(
        Host::read_until(&mut stream, &mut text, &TERMINAL, WAIT).await,
        "run did not finish\n{text}"
    );
    drop(stream);
    assert!(
        sessions_reach(&host, 0.0, Duration::from_secs(10)).await,
        "the session was not evicted once idle"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn session_cap_evicts_least_recently_active_first() {
    let turn = |name: &str| format!("session {name} records FACT-{name}");
    let ask = |name: &str| format!("session {name} asks");
    let names = ["A", "B", "C", "D", "E", "F"];
    let fixtures = names.iter().fold(FixtureSet::new(), |set, name| {
        reply(reply(set, &turn(name), "ok"), &ask(name), "ok")
    });
    let fixtures = terminal_call(fixtures, "session D waits", "true");

    // Three idle sessions, cap 2: the least recently active one goes.
    let host = boot(fixtures.clone(), FAST_SWEEP, cap(2)).await;
    for name in ["A", "B", "C"] {
        host.drive(Some(P1), test_agent(None), &turn(name), Some(name))
            .await;
    }
    assert!(
        sessions_reach(&host, 2.0, Duration::from_secs(10)).await,
        "cap not applied: {:?}",
        host.gauge(SESSIONS).await
    );
    for (name, kept) in [("B", true), ("C", true), ("A", false)] {
        host.drive(Some(P1), test_agent(None), &ask(name), Some(name))
            .await;
        let fact = format!("FACT-{name}");
        assert_eq!(
            request_carries(&host, &ask(name), &fact).await,
            kept,
            "session {name}"
        );
    }

    // A session with a running run is never chosen, even when it is the
    // least recently active.
    let host = boot(fixtures, FAST_SWEEP, cap(2)).await;
    let waiting = host
        .start_run(
            Some(P1),
            test_agent(Some(ask_policy())),
            "session D waits",
            Some("D"),
        )
        .await;
    let mut stream = host
        .open_stream(Some(P1), &waiting)
        .await
        .unwrap_or_else(|status| panic!("stream: {status}"));
    let mut text = String::new();
    assert!(
        Host::read_until(&mut stream, &mut text, &[APPROVAL_MARKER], WAIT).await,
        "run did not pause\n{text}"
    );
    for name in ["E", "F"] {
        host.drive(Some(P1), test_agent(None), &turn(name), Some(name))
            .await;
    }
    assert!(
        sessions_reach(&host, 2.0, Duration::from_secs(10)).await,
        "cap not applied: {:?}",
        host.gauge(SESSIONS).await
    );
    assert_eq!(host.approve(Some(P1), &waiting).await, 200);
    assert!(
        Host::read_until(&mut stream, &mut text, &TERMINAL, WAIT).await,
        "run did not finish\n{text}"
    );
    for (name, fact, kept) in [
        ("D", "session D waits", true),
        ("F", "FACT-F", true),
        ("E", "FACT-E", false),
    ] {
        host.drive(Some(P1), test_agent(None), &ask(name), Some(name))
            .await;
        assert_eq!(
            request_carries(&host, &ask(name), fact).await,
            kept,
            "session {name}"
        );
    }
}

fn logged_u64(output: &str, field: &str) -> Option<u64> {
    logged_field(output, RETENTION_MESSAGE, field).and_then(|value| value.as_u64())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn standalone_default_keeps_idle_sessions() {
    let first = "standalone records FACT-PINE";
    let ask = "standalone asks";
    let fixtures = reply(reply(FixtureSet::new(), first, "ok"), ask, "ok");
    let host = boot_standalone(fixtures, FAST_SWEEP, |options| {
        options.env("UAR_SERVER__LOG_FORMAT", "json")
    })
    .await;
    let stdout = host.process.stdout();
    assert_eq!(logged_u64(&stdout, "session_idle_timeout_secs"), Some(0));
    assert_eq!(logged_u64(&stdout, "session_max_retained"), Some(0));
    host.drive(None, test_agent(None), first, Some("s1")).await;
    tokio::time::sleep(Duration::from_millis(3_500)).await;
    host.drive(None, test_agent(None), ask, Some("s1")).await;
    assert!(
        request_carries(&host, ask, "FACT-PINE").await,
        "standalone evicted an idle session by default"
    );

    let defaults = boot(FixtureSet::new(), "", |options| options).await;
    let stdout = defaults.process.stdout();
    assert_eq!(logged_u64(&stdout, "session_idle_timeout_secs"), Some(1_800));
    assert_eq!(logged_u64(&stdout, "session_max_retained"), Some(1_000));

    let operator = boot(FixtureSet::new(), "", idle_timeout(0)).await;
    let stdout = operator.process.stdout();
    assert_eq!(
        logged_u64(&stdout, "session_idle_timeout_secs"),
        Some(0),
        "the operator's explicit value was replaced"
    );
}
