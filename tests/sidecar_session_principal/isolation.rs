//! What a principal scopes: runs, approvals, memory tools, knowledge bases,
//! sessions and A2UI surfaces (tasks 1.3–1.9).

use std::time::Duration;

use reqwest::Method;
use serde_json::{Value, json};

use crate::principal_host::{
    APPROVAL_MARKER, Host, P1, P2, ask_policy, boot, reply, terminal_call, tool_call,
};
use crate::sidecar_process::test_agent;
use crate::stub_llm::FixtureSet;

const WAIT: Duration = Duration::from_secs(60);
const TERMINAL: [&str; 3] = [
    "event: agui.done",
    "event: agui.error",
    "event: agui.cancelled",
];

/// Open `run_id` as its owner and read until it waits for an approval.
async fn paused_stream(host: &Host, principal: &str, run_id: &str) -> (reqwest::Response, String) {
    let mut stream = host
        .open_stream(Some(principal), run_id)
        .await
        .unwrap_or_else(|status| panic!("owner stream: {status}"));
    let mut text = String::new();
    assert!(
        Host::read_until(&mut stream, &mut text, &[APPROVAL_MARKER], WAIT).await,
        "run did not pause on approval\n{text}"
    );
    (stream, text)
}

fn action_body() -> Value {
    json!({
        "surfaceId": "isolation-surface",
        "name": "submit",
        "sourceComponentId": "button",
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn run_routes_refuse_another_principal() {
    let input = "paused run for route isolation";
    let resume_input = "resume probe for route isolation";
    let fixtures = reply(
        terminal_call(FixtureSet::new(), input, "echo route-isolation"),
        resume_input,
        "resumed",
    );
    let host = boot(fixtures, "", |options| options).await;
    let artifact = test_agent(Some(ask_policy()));
    let run_id = host
        .start_run(Some(P1), artifact.clone(), input, None)
        .await;
    let (mut stream, mut text) = paused_stream(&host, P1, &run_id).await;

    assert_eq!(
        host.open_stream(Some(P2), &run_id).await.err(),
        Some(404),
        "stream opened by another principal"
    );
    let base = format!("/api/uar/runs/{run_id}");
    let foreign = [
        (Method::POST, format!("{base}/cancel"), None),
        (Method::GET, format!("{base}/checkpoints"), None),
        (
            Method::POST,
            format!("{base}/resume"),
            Some(json!({ "artifact": artifact, "input": resume_input })),
        ),
        (
            Method::POST,
            format!("{base}/tool-approval"),
            Some(json!({ "approved": true })),
        ),
        (
            Method::POST,
            format!("{base}/approval"),
            Some(json!({ "approved": true })),
        ),
        (Method::POST, format!("{base}/a2ui/actions"), Some(action_body())),
        (Method::GET, format!("{base}/a2ui/surface-replay"), None),
    ];
    for (method, path, body) in foreign {
        let (status, text) = host.call(method.clone(), &path, Some(P2), body).await;
        assert_eq!(status, 404, "{method} {path} as another principal: {text}");
    }

    // The owner still controls the run, which is still waiting.
    assert!(host.run_exists(Some(P1), &run_id).await, "owner checkpoints");
    let (status, replay) = host
        .call(
            Method::GET,
            &format!("{base}/a2ui/surface-replay"),
            Some(P1),
            None,
        )
        .await;
    assert_eq!(status, 200, "owner surface replay: {replay}");
    let (status, action) = host
        .call(
            Method::POST,
            &format!("{base}/a2ui/actions"),
            Some(P1),
            Some(action_body()),
        )
        .await;
    assert_ne!(status, 404, "owner A2UI action not admitted: {action}");
    let second = host.open_stream(Some(P1), &run_id).await;
    assert!(second.is_ok(), "owner second stream: {:?}", second.err());
    drop(second);
    let (status, resumed) = host
        .call(
            Method::POST,
            &format!("{base}/resume"),
            Some(P1),
            Some(json!({ "artifact": test_agent(None), "input": resume_input })),
        )
        .await;
    assert_eq!(status, 200, "owner resume: {resumed}");
    assert_eq!(host.approve(Some(P1), &run_id).await, 200, "owner approval");
    assert!(
        Host::read_until(&mut stream, &mut text, &TERMINAL, WAIT).await,
        "run did not finish\n{text}"
    );
    assert!(
        text.contains("event: agui.tool_result") && text.contains("event: agui.done"),
        "run was changed by another principal\n{text}"
    );
    let (status, cancelled) = host
        .call(Method::POST, &format!("{base}/cancel"), Some(P1), None)
        .await;
    assert_eq!(status, 200, "owner cancel: {cancelled}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn approval_from_another_principal_is_refused() {
    let outputs = tempfile::tempdir().expect("output dir");
    let executions = outputs.path().join("executions.log");
    let input = "run a tool that needs approval";
    let command = format!("echo executed >> '{}'", executions.display());
    let host = boot(
        terminal_call(FixtureSet::new(), input, &command),
        "",
        |options| options,
    )
    .await;
    let run_id = host
        .start_run(Some(P1), test_agent(Some(ask_policy())), input, None)
        .await;
    let (mut stream, mut text) = paused_stream(&host, P1, &run_id).await;

    for route in ["tool-approval", "approval"] {
        let (status, body) = host
            .call(
                Method::POST,
                &format!("/api/uar/runs/{run_id}/{route}"),
                Some(P2),
                Some(json!({ "approved": true })),
            )
            .await;
        assert_eq!(status, 404, "{route} from another principal: {body}");
    }
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert!(
        !executions.exists(),
        "the tool ran on another principal's approval"
    );

    assert_eq!(host.approve(Some(P1), &run_id).await, 200, "owner approval");
    assert!(
        Host::read_until(&mut stream, &mut text, &TERMINAL, WAIT).await,
        "run did not finish\n{text}"
    );
    let log = std::fs::read_to_string(&executions).unwrap_or_default();
    assert_eq!(log.lines().count(), 1, "tool executions: {log:?}\n{text}");
}

/// Task 1.5, reduced: the sidecar has no memory backend (memory is forced off
/// by `sidecar-launch-security`), so what is observable is the owner
/// confinement that runs before any store access.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn memory_is_scoped_to_the_principal() {
    let cases = [
        ("memory_list", json!({ "user_id": P1 })),
        ("memory_recall", json!({ "query": "P1-SECRET", "user_id": P1 })),
        ("memory_save", json!({ "content": "P2 note", "user_id": P1 })),
    ];
    let mut fixtures = FixtureSet::new();
    for (tool, arguments) in &cases {
        fixtures = tool_call(fixtures, &format!("principal memory {tool}"), tool, arguments);
    }
    let host = boot(fixtures, "", |options| options).await;
    for (tool, _) in &cases {
        let run = host
            .drive(Some(P2), test_agent(None), &format!("principal memory {tool}"), None)
            .await;
        assert!(
            run.stream.contains("event: agui.tool_result"),
            "{tool}: no tool result\n{}",
            run.stream
        );
        assert!(
            run.stream
                .contains("Memory owner cannot be replaced by tool arguments"),
            "{tool}: a model-supplied owner was accepted\n{}",
            run.stream
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn anonymous_runs_refuse_unscoped_memory_tools() {
    let refused = [
        ("memory_list", json!({})),
        (
            "memory_update_by_id",
            json!({ "memory_id": "memory:probe", "content": "changed" }),
        ),
        ("memory_delete_by_id", json!({ "memory_id": "memory:probe" })),
        ("memory_history", json!({ "memory_id": "memory:probe" })),
    ];
    let recall = ("memory_recall", json!({ "query": "anything" }));
    let mut fixtures = FixtureSet::new();
    for (tool, arguments) in refused.iter().chain(std::iter::once(&recall)) {
        fixtures = tool_call(fixtures, &format!("anonymous memory {tool}"), tool, arguments);
    }
    let host = boot(fixtures, "", |options| options).await;
    for (tool, _) in &refused {
        let run = host
            .drive(None, test_agent(None), &format!("anonymous memory {tool}"), None)
            .await;
        assert!(
            run.stream.contains("memory_requires_verified_owner"),
            "{tool}: not refused without a verified owner\n{}",
            run.stream
        );
    }
    // Recall keeps its behaviour for anonymous runs.
    let run = host
        .drive(None, test_agent(None), "anonymous memory memory_recall", None)
        .await;
    assert!(
        run.stream.contains("event: agui.tool_result")
            && !run.stream.contains("memory_requires_verified_owner"),
        "memory_recall changed for anonymous runs\n{}",
        run.stream
    );
}

async fn knowledge_base_names(host: &Host, principal: Option<&str>) -> Vec<String> {
    let (status, body) = host
        .call(Method::GET, "/api/uar/knowledge-bases", principal, None)
        .await;
    assert_eq!(status, 200, "list knowledge bases: {body}");
    let list: Value = serde_json::from_str(&body).expect("knowledge bases JSON");
    list.as_array()
        .unwrap_or_else(|| panic!("knowledge base list: {list}"))
        .iter()
        .filter_map(|kb| kb["name"].as_str().map(str::to_owned))
        .collect()
}

/// Task 1.7, reduced to name resolution: this build has no local embedding
/// backend, so retrieval cannot run.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn knowledge_bases_resolve_only_for_their_principal() {
    let host = boot(FixtureSet::new(), "", |options| options).await;
    let body = json!({ "name": "handbook", "description": "principal isolation" });
    let (status, created) = host
        .call(
            Method::POST,
            "/api/uar/knowledge-bases",
            Some(P1),
            Some(body.clone()),
        )
        .await;
    assert!((200..300).contains(&status), "create as P1: {status} {created}");

    let count = |names: &[String]| names.iter().filter(|name| *name == "handbook").count();
    assert_eq!(count(&knowledge_base_names(&host, Some(P1)).await), 1);
    assert_eq!(
        count(&knowledge_base_names(&host, Some(P2)).await),
        0,
        "another principal resolves the knowledge base"
    );
    assert_eq!(
        count(&knowledge_base_names(&host, None).await),
        0,
        "the anonymous scope resolves the knowledge base"
    );
    let (status, created) = host
        .call(Method::POST, "/api/uar/knowledge-bases", Some(P2), Some(body))
        .await;
    assert!((200..300).contains(&status), "create as P2: {status} {created}");
    assert_eq!(count(&knowledge_base_names(&host, Some(P1)).await), 1);
    assert_eq!(count(&knowledge_base_names(&host, Some(P2)).await), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn same_session_id_under_two_principals_is_two_sessions() {
    let turns = [
        (P1, "P1 records FACT-AMBER", "P1 asks for its facts", "FACT-AMBER"),
        (P2, "P2 records FACT-COBALT", "P2 asks for its facts", "FACT-COBALT"),
    ];
    let mut fixtures = FixtureSet::new();
    for (_, first, second, _) in &turns {
        fixtures = reply(reply(fixtures, first, "noted"), second, "answered");
    }
    let host = boot(fixtures, "", |options| options).await;
    for (principal, first, _, _) in turns {
        host.drive(Some(principal), test_agent(None), first, Some("s1"))
            .await;
    }
    for (principal, _, second, _) in turns {
        host.drive(Some(principal), test_agent(None), second, Some("s1"))
            .await;
    }
    for (index, (_, _, second, own)) in turns.iter().enumerate() {
        let other = turns[1 - index].3;
        let requests = host.model_requests_for(second).await;
        assert!(!requests.is_empty(), "no model request for {second}");
        for request in requests {
            assert!(request.contains(own), "{second} lost its own fact");
            assert!(
                !request.contains(other),
                "{second} received the other principal's fact"
            );
        }
    }
}

async fn trigger_surface(host: &Host, principal: &str, run_id: &str, fact: &str) {
    let (status, body) = host
        .call(
            Method::POST,
            &format!("/api/uar/runs/{run_id}/a2ui/surface-test-trigger"),
            Some(principal),
            Some(json!({
                "surface_id": format!("surface-{principal}"),
                "kind": "createSurface",
                "payload": { "catalogId": "urn:uar:a2ui:catalog:1", "fact": fact },
            })),
        )
        .await;
    assert_eq!(status, 200, "surface trigger: {body}");
}

/// Task 1.9, reduced: concurrent sessions, their streams, A2UI surfaces and
/// model requests. Compaction, memory and MCP bindings are not exercised
/// here (see tasks.md).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_sessions_share_nothing_in_one_process() {
    let facts = [(P1, "ORCHID-7431"), (P2, "TUNDRA-2986")];
    let first = |principal: &str, fact: &str| format!("{principal} private {fact}");
    let second = |principal: &str| format!("{principal} follow-up");
    let mut fixtures = FixtureSet::new();
    for (principal, fact) in facts {
        fixtures = reply(fixtures, &first(principal, fact), "kept");
        fixtures = reply(fixtures, &second(principal), "recalled");
    }
    let host = boot(fixtures, "", |options| options).await;
    let (first_one, first_two) = (first(P1, facts[0].1), first(P2, facts[1].1));
    let (one, two) = tokio::join!(
        host.drive(Some(P1), test_agent(None), &first_one, Some("shared")),
        host.drive(Some(P2), test_agent(None), &first_two, Some("shared")),
    );
    trigger_surface(&host, P1, &one.run_id, facts[0].1).await;
    trigger_surface(&host, P2, &two.run_id, facts[1].1).await;
    let (second_one, second_two) = (second(P1), second(P2));
    let (one_again, two_again) = tokio::join!(
        host.drive(Some(P1), test_agent(None), &second_one, Some("shared")),
        host.drive(Some(P2), test_agent(None), &second_two, Some("shared")),
    );

    let runs = [
        (P1, facts[0].1, facts[1].1, [&one, &one_again]),
        (P2, facts[1].1, facts[0].1, [&two, &two_again]),
    ];
    for (principal, own, other, driven) in runs {
        for run in driven {
            assert!(
                !run.stream.contains(other),
                "{principal} stream carries the other session's fact"
            );
        }
        let (status, replay) = host
            .call(
                Method::GET,
                &format!("/api/uar/runs/{}/a2ui/surface-replay", driven[0].run_id),
                Some(principal),
                None,
            )
            .await;
        assert_eq!(status, 200, "{principal} surface replay: {replay}");
        assert!(replay.contains(own), "{principal} surface lost: {replay}");
        assert!(!replay.contains(other), "{principal} surface carries the other fact");
        for input in [first(principal, own), second(principal)] {
            for request in host.model_requests_for(&input).await {
                assert!(
                    !request.contains(other),
                    "{principal} model request carries the other session's fact"
                );
            }
        }
        let follow_up = host.model_requests_for(&second(principal)).await;
        assert!(
            follow_up.iter().all(|request| request.contains(own)),
            "{principal} follow-up lost its own session history"
        );
    }
    let (status, _) = host
        .call(
            Method::GET,
            &format!("/api/uar/runs/{}/a2ui/surface-replay", one.run_id),
            Some(P2),
            None,
        )
        .await;
    assert_eq!(status, 404, "another principal read a surface");
}
