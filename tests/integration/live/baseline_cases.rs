//! Baseline feature cases for the live integration tier
//! (live-integration-baseline-coverage, task group 2).
//!
//! Each case boots a real server (`harness::boot_test_server`) pointed at a
//! stub LLM (`stub_llm::start_stub_llm`) and makes a real HTTP request,
//! proving the feature works end-to-end through the actual production code
//! path — not a unit-level approximation.

use super::harness::{ServiceNeeds, boot_test_server};
use super::stub_llm::{FixtureResponse, FixtureSet, RequestFingerprint, start_stub_llm};

/// Model name as it appears on the wire to the stub — bare, no `provider/`
/// prefix; see harness.rs's discovery note on why.
const MODEL: &str = "gpt-5.4-mini";

fn content_fixture(last_user_message: &str, response: &str) -> FixtureSet {
    FixtureSet::new().with(
        RequestFingerprint {
            model: MODEL.to_string(),
            last_user_message: last_user_message.to_string(),
            has_tools: true,
            has_tool_result: false,
        },
        FixtureResponse::Content(response.to_string()),
    )
}

/// 2.1 — Streaming chat case for `stream_mode: openai`.
#[tokio::test]
async fn streaming_chat_openai_mode() {
    let stub = start_stub_llm(content_fixture("hello openai mode", "hi there")).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/chat/completion", server.base_url))
        .json(&serde_json::json!({
            "model": MODEL,
            "messages": [{"role": "user", "content": "hello openai mode"}],
            "stream": true,
            "stream_mode": "openai",
        }))
        .send()
        .await
        .expect("request");

    assert!(resp.status().is_success(), "status: {}", resp.status());
    let body = resp.text().await.expect("body");
    assert!(
        body.contains("chat.completion.chunk"),
        "expected an OpenAI-shaped chunk, got: {body}"
    );
    assert!(
        body.contains("hi there"),
        "expected the fixture content to appear, got: {body}"
    );
    // openai mode must not emit agui.* named events.
    assert!(
        !body.contains("agui."),
        "openai mode should not emit agui events, got: {body}"
    );
}

/// 2.2 — Streaming chat case for `stream_mode: agui`.
#[tokio::test]
async fn streaming_chat_agui_mode() {
    let stub = start_stub_llm(content_fixture("hello agui mode", "hi there")).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/chat/completion", server.base_url))
        .json(&serde_json::json!({
            "model": MODEL,
            "messages": [{"role": "user", "content": "hello agui mode"}],
            "stream": true,
            "stream_mode": "agui",
        }))
        .send()
        .await
        .expect("request");

    assert!(resp.status().is_success(), "status: {}", resp.status());
    let body = resp.text().await.expect("body");
    assert!(
        body.contains("agui.message.delta") || body.contains("agui.stream.start"),
        "expected an agui.* named event, got: {body}"
    );
    // agui mode must not emit raw OpenAI chunks.
    assert!(
        !body.contains("chat.completion.chunk"),
        "agui mode should not emit OpenAI chunks, got: {body}"
    );
}

/// 2.3 — Streaming chat case for `stream_mode: dual`.
#[tokio::test]
async fn streaming_chat_dual_mode() {
    let stub = start_stub_llm(content_fixture("hello dual mode", "hi there")).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/chat/completion", server.base_url))
        .json(&serde_json::json!({
            "model": MODEL,
            "messages": [{"role": "user", "content": "hello dual mode"}],
            "stream": true,
            "stream_mode": "dual",
        }))
        .send()
        .await
        .expect("request");

    assert!(resp.status().is_success(), "status: {}", resp.status());
    let body = resp.text().await.expect("body");
    assert!(
        body.contains("chat.completion.chunk"),
        "dual mode should emit OpenAI chunks too, got: {body}"
    );
    assert!(
        body.contains("agui.message.delta") || body.contains("agui.stream.start"),
        "dual mode should emit agui.* events too, got: {body}"
    );
}

/// 2.4 — MCP/native tool-loop round-trip case.
///
/// Uses `native_echo` (`src/uar/runtime/native_skills/echo.rs`) — registered
/// unconditionally by `register_builtins`, so no MCP server or external
/// dependency is needed. Two fixtures: the first request (no tool result in
/// the conversation yet) gets a tool-call fixture; the second request (tool
/// result appended, same user message) gets the final content — see
/// `RequestFingerprint::has_tool_result`'s doc comment for why two fixtures
/// are needed for what looks like "the same" request.
#[tokio::test]
async fn tool_loop_round_trip() {
    let fixtures = FixtureSet::new()
        .with(
            RequestFingerprint {
                model: MODEL.to_string(),
                last_user_message: "echo this back".to_string(),
                has_tools: true,
                has_tool_result: false,
            },
            FixtureResponse::ToolCall {
                name: "native_echo".to_string(),
                arguments: r#"{"message":"echo this back"}"#.to_string(),
            },
        )
        .with(
            RequestFingerprint {
                model: MODEL.to_string(),
                last_user_message: "echo this back".to_string(),
                has_tools: true,
                has_tool_result: true,
            },
            FixtureResponse::Content("the tool echoed it back".to_string()),
        );
    let stub = start_stub_llm(fixtures).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/chat/completion", server.base_url))
        .json(&serde_json::json!({
            "model": MODEL,
            "messages": [{"role": "user", "content": "echo this back"}],
            "stream": false,
        }))
        .send()
        .await
        .expect("request");

    assert!(
        resp.status().is_success(),
        "status: {} body: {}",
        resp.status(),
        resp.text().await.unwrap_or_default()
    );
    let body: serde_json::Value = resp.json().await.expect("json body");
    let content = body["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or_default();
    assert_eq!(
        content, "the tool echoed it back",
        "expected the tool loop to complete and return the post-tool-call content, got: {body}"
    );
}
