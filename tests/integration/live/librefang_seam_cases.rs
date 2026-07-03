//! CH-18 librefang-a2a-agui-bridge (D-C scope) — UAR-side integration surface.
//!
//! Per plan.md A1.5, deliverable #1 is the "zero-code seam": prove a bossfang
//! provider pointed at UAR's OpenAI-compatible `/v1/chat/completions` just
//! works, with NO librefang/bossfang code involved (cross-repo work is
//! explicitly out of scope — see plan.md CH-18). This also finishes wiring
//! CH-21's `to_agui_spec_event` (defined but never actually reachable via any
//! stream_mode until this change — see `agui_spec` in `src/server.rs`
//! `StreamMode`), since CH-18's AG-UI stream consumption contract needs a
//! real, reachable stream to document.

use super::harness::{ServiceNeeds, boot_test_server};
use super::stub_llm::{FixtureResponse, FixtureSet, RequestFingerprint, start_stub_llm};
use serial_test::serial;

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

/// A1.5's zero-code seam: a bare OpenAI-client-shaped request (no UAR
/// extension fields at all — no `stream_mode`, no `stream`) against
/// `/v1/chat/completions`, the exact path a bossfang `provider_urls` entry
/// would target. Proves UAR is a drop-in OpenAI-compatible provider.
#[tokio::test]
#[serial]
async fn zero_code_seam_v1_chat_completions_is_openai_compatible() {
    let stub = start_stub_llm(content_fixture(
        "hello from a bossfang-shaped client",
        "hi, this is UAR",
    ))
    .await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v1/chat/completions", server.base_url))
        .json(&serde_json::json!({
            "model": MODEL,
            "messages": [{"role": "user", "content": "hello from a bossfang-shaped client"}]
        }))
        .send()
        .await
        .expect("request");

    assert!(resp.status().is_success(), "status: {}", resp.status());
    let body: serde_json::Value = resp.json().await.expect("OpenAI-shaped JSON body");

    assert_eq!(body["object"], "chat.completion");
    assert_eq!(body["choices"][0]["message"]["role"], "assistant");
    assert_eq!(body["choices"][0]["message"]["content"], "hi, this is UAR");
    assert_eq!(body["choices"][0]["finish_reason"], "stop");
}

/// CH-18's AG-UI stream consumption contract needs `stream_mode: "agui_spec"`
/// to actually emit the official AG-UI event vocabulary (RUN_STARTED,
/// TEXT_MESSAGE_CONTENT, RUN_FINISHED) — finishing CH-21's wiring gap.
#[tokio::test]
#[serial]
async fn agui_spec_mode_emits_official_event_vocabulary() {
    let stub = start_stub_llm(content_fixture("hello agui spec mode", "hi there")).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/chat/completion", server.base_url))
        .json(&serde_json::json!({
            "model": MODEL,
            "messages": [{"role": "user", "content": "hello agui spec mode"}],
            "stream": true,
            "stream_mode": "agui_spec",
        }))
        .send()
        .await
        .expect("request");

    assert!(resp.status().is_success(), "status: {}", resp.status());
    let body = resp.text().await.expect("body");

    assert!(
        body.contains("RUN_STARTED"),
        "expected RUN_STARTED, got: {body}"
    );
    assert!(
        body.contains("TEXT_MESSAGE_CONTENT"),
        "expected TEXT_MESSAGE_CONTENT, got: {body}"
    );
    assert!(
        body.contains("RUN_FINISHED"),
        "expected RUN_FINISHED, got: {body}"
    );
    // agui_spec mode must not emit the legacy agui.* names or raw OpenAI
    // chunks — it's an independent mode from `agui`/`dual`/`openai`.
    assert!(
        !body.contains("agui.stream.start") && !body.contains("agui.message.delta"),
        "agui_spec mode should not emit legacy agui.* events, got: {body}"
    );
    assert!(
        !body.contains("chat.completion.chunk"),
        "agui_spec mode should not emit OpenAI chunks, got: {body}"
    );
}
