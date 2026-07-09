//! `upgrade-a2ui-testing-live-round-trip` — live verification that
//! `POST /api/uar/runs/{run_id}/a2ui/test-trigger` emits a real
//! `ArtifactInputRequest` using the exact same `RunManager::emit_to_run`
//! path a live agent tool call uses, and that the resulting artifact_id can
//! be submitted through the real, unmodified `/artifact-response` endpoint —
//! proving the full round-trip contract the reworked `A2uiTestingPage`
//! depends on, against a real booted server (not a mock of this project's
//! own code).

use super::harness::{ServiceNeeds, boot_test_server};
use super::stub_llm::{FixtureResponse, FixtureSet, RequestFingerprint, start_stub_llm};
use serial_test::serial;

/// Passed to `boot_test_server` / sent as the request body's `model` field —
/// the orchestrator strips the `provider/` prefix before the wire request
/// reaches the stub, so fixtures below fingerprint on the bare model name.
const MODEL: &str = "openai/gpt-5.4-mini";
const WIRE_MODEL: &str = "gpt-5.4-mini";

fn content_fixture(last_user_message: &str, response: &str) -> FixtureSet {
    FixtureSet::new().with(
        RequestFingerprint {
            model: WIRE_MODEL.to_string(),
            last_user_message: last_user_message.to_string(),
            has_tools: true,
            has_tool_result: false,
        },
        FixtureResponse::Content(response.to_string()),
    )
}

/// Extracts `request_id` from the first `agui.stream.start` event's JSON
/// payload in a raw SSE response body — the same field the real frontend
/// (`chat-stream-store.ts`) reads to learn the server-assigned run_id.
fn extract_run_id(sse_body: &str) -> String {
    let marker = "\"request_id\":\"";
    let start = sse_body
        .find(marker)
        .expect("agui.stream.start event with request_id in SSE body")
        + marker.len();
    let end = sse_body[start..].find('"').expect("closing quote");
    sse_body[start..start + end].to_string()
}

/// Full real round-trip: start a real chat run against a stub LLM, extract
/// its run_id from the real SSE stream, trigger a synthetic artifact input
/// request against that (now-completed but still-tracked) run, and confirm
/// the returned artifact_id can be submitted through the real,
/// already-existing `/artifact-response` endpoint.
#[tokio::test]
#[serial]
async fn test_trigger_round_trips_through_real_artifact_response_endpoint() {
    let stub = start_stub_llm(content_fixture(
        "hello a2ui test trigger",
        "hi there",
    ))
    .await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;
    let client = reqwest::Client::new();

    // 1. Complete a real chat turn to obtain a real, server-tracked run_id.
    let resp = client
        .post(format!("{}/api/chat/completion", server.base_url))
        .json(&serde_json::json!({
            "model": MODEL,
            "messages": [{"role": "user", "content": "hello a2ui test trigger"}],
            "stream": true,
            "stream_mode": "dual",
        }))
        .send()
        .await
        .expect("chat completion request");
    assert!(resp.status().is_success(), "status: {}", resp.status());
    let sse_body = resp.text().await.expect("sse body");
    let run_id = extract_run_id(&sse_body);

    // 2. Trigger a synthetic ArtifactInputRequest against that run — the
    //    exact mechanism a live agent tool call would use, per
    //    NormalizedEvent::ArtifactInputRequest / RunManager::emit_to_run.
    let trigger_resp = client
        .post(format!(
            "{}/api/uar/runs/{}/a2ui/test-trigger",
            server.base_url, run_id
        ))
        .json(&serde_json::json!({
            "artifact_type": "confirm",
            "title": "Integration test confirm",
            "content": "{\"message\":\"Proceed?\"}",
            "metadata": {}
        }))
        .send()
        .await
        .expect("test-trigger request");
    assert!(
        trigger_resp.status().is_success(),
        "test-trigger status: {}",
        trigger_resp.status()
    );
    let trigger_body: serde_json::Value = trigger_resp.json().await.expect("trigger ack JSON");
    assert_eq!(trigger_body["run_id"], run_id);
    assert_eq!(trigger_body["status"], "triggered");
    let artifact_id = trigger_body["artifact_id"]
        .as_str()
        .expect("artifact_id string")
        .to_string();
    assert!(!artifact_id.is_empty());

    // 3. Complete the round-trip through the real, unmodified
    //    /artifact-response endpoint the production A2uiInputBlock uses.
    let submit_resp = client
        .post(format!(
            "{}/api/uar/runs/{}/artifact-response",
            server.base_url, run_id
        ))
        .json(&serde_json::json!({
            "artifact_id": artifact_id,
            "response": {"accepted": true}
        }))
        .send()
        .await
        .expect("artifact-response request");
    assert!(
        submit_resp.status().is_success(),
        "artifact-response status: {}",
        submit_resp.status()
    );
    let submit_body: serde_json::Value = submit_resp.json().await.expect("submit ack JSON");
    assert_eq!(submit_body["run_id"], run_id);
    assert_eq!(submit_body["artifact_id"], artifact_id);
    assert_eq!(submit_body["status"], "accepted");
}

/// A test-trigger against a run that was never created MUST fail clearly,
/// not silently succeed or fabricate a run — per this change's own spec
/// requirement ("Triggering against a nonexistent or inactive run").
#[tokio::test]
#[serial]
async fn test_trigger_rejects_nonexistent_run() {
    let stub = start_stub_llm(content_fixture("unused", "unused")).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!(
            "{}/api/uar/runs/nonexistent-run-id/a2ui/test-trigger",
            server.base_url
        ))
        .json(&serde_json::json!({
            "artifact_type": "confirm",
            "title": "Should not trigger",
            "content": "{}",
            "metadata": {}
        }))
        .send()
        .await
        .expect("test-trigger request");

    assert_eq!(resp.status(), reqwest::StatusCode::NOT_FOUND);
}
