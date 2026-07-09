//! Gherkin/BDD test suite (native `cucumber` crate — no JS/Playwright
//! toolchain needed, this is a pure-Rust backend).
//!
//! Reuses the same real-server-boot harness the plain integration tests use
//! (`tests/integration/live/harness.rs` + `stub_llm.rs`) so these scenarios
//! exercise the actual production code path, not a hand-rolled mock. Only
//! these two files are included (not the whole `integration/live/mod.rs`,
//! which also pulls in `baseline_cases.rs`/`librefang_seam_cases.rs`'s own
//! `#[tokio::test]` suites — those need libtest's harness, which this binary
//! doesn't use since it drives its own `main()` via the Cucumber runner).
#[path = "integration/live"]
mod live {
    #[path = "harness.rs"]
    pub mod harness;
    #[path = "stub_llm.rs"]
    pub mod stub_llm;
}

use cucumber::{World as _, given, then, when};
use live::harness::{ServiceNeeds, boot_test_server};
use live::stub_llm::{
    FixtureResponse, FixtureSet, RequestFingerprint, StubLlmServer, start_stub_llm,
};

const MODEL: &str = "gpt-5.4-mini";

/// Cucumber scenario state. `stub` MUST be held for the scenario's lifetime
/// — `StubLlmServer::drop` aborts its task, so dropping it early would kill
/// the LLM backend mid-scenario. The booted UAR server's own handle does NOT
/// need to be held (its OS thread is intentionally detached/leaked for the
/// rest of the test binary's process — see harness.rs's `TestServerHandle`
/// doc comment), so only its `base_url` is kept.
#[derive(cucumber::World)]
struct World {
    pending_fixtures: FixtureSet,
    stub: Option<StubLlmServer>,
    base_url: Option<String>,
    response_status: Option<u16>,
    response_body: String,
}

impl std::fmt::Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
            .field("base_url", &self.base_url)
            .field("response_status", &self.response_status)
            .finish_non_exhaustive()
    }
}

impl Default for World {
    fn default() -> Self {
        Self {
            pending_fixtures: FixtureSet::new(),
            stub: None,
            base_url: None,
            response_status: None,
            response_body: String::new(),
        }
    }
}

fn content_fingerprint(message: &str) -> RequestFingerprint {
    RequestFingerprint {
        model: MODEL.to_string(),
        last_user_message: message.to_string(),
        has_tools: true,
        has_tool_result: false,
    }
}

#[given("a running UAR server with a stub LLM")]
fn given_a_running_server(_world: &mut World) {
    // Actual boot is deferred to the first "When" step, once all fixture-
    // adding Given steps have populated `pending_fixtures` — `stub_llm`'s
    // fixtures are fixed at construction time, so the stub can't be started
    // until the full fixture set for the scenario is known.
}

#[given(regex = r#"^the stub LLM responds to "([^"]+)" with the content "([^"]+)"$"#)]
fn given_content_fixture(world: &mut World, message: String, response: String) {
    let fixtures = std::mem::take(&mut world.pending_fixtures);
    world.pending_fixtures = fixtures.with(
        content_fingerprint(&message),
        FixtureResponse::Content(response),
    );
}

#[given(
    regex = r#"^the stub LLM responds to "([^"]+)" with a call to tool "([^"]+)" then the content "([^"]+)"$"#
)]
fn given_tool_call_fixture(world: &mut World, message: String, tool: String, response: String) {
    let arguments = serde_json::json!({ "message": message }).to_string();
    let fixtures = std::mem::take(&mut world.pending_fixtures)
        .with(
            content_fingerprint(&message),
            FixtureResponse::ToolCall {
                name: tool,
                arguments,
            },
        )
        .with(
            RequestFingerprint {
                model: MODEL.to_string(),
                last_user_message: message,
                has_tools: true,
                has_tool_result: true,
            },
            FixtureResponse::Content(response),
        );
    world.pending_fixtures = fixtures;
}

/// Boots the stub LLM (with whatever fixtures accumulated so far) and a real
/// UAR server pointed at it, storing the server's base URL. Idempotent
/// within a scenario — a second call is a no-op, since only one server per
/// scenario is ever needed even if multiple `When` steps somehow fired.
async fn ensure_server_booted(world: &mut World) {
    if world.base_url.is_some() {
        return;
    }
    let fixtures = std::mem::take(&mut world.pending_fixtures);
    let stub = start_stub_llm(fixtures).await;
    let server = boot_test_server(&stub.base_url, MODEL, ServiceNeeds::default()).await;
    world.base_url = Some(server.base_url);
    world.stub = Some(stub);
}

#[when(
    regex = r#"^I send a bare OpenAI-shaped chat completion request with message "([^"]+)" to "([^"]+)"$"#
)]
async fn when_bare_openai_request(world: &mut World, message: String, path: String) {
    ensure_server_booted(world).await;
    let base_url = world.base_url.clone().expect("server should be booted");
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{base_url}{path}"))
        .json(&serde_json::json!({
            "model": MODEL,
            "messages": [{"role": "user", "content": message}]
        }))
        .send()
        .await
        .expect("request");
    world.response_status = Some(resp.status().as_u16());
    world.response_body = resp.text().await.expect("response body");
}

#[when(
    regex = r#"^I send a streaming chat completion request with stream_mode "([^"]+)" and message "([^"]+)"$"#
)]
async fn when_streaming_request(world: &mut World, stream_mode: String, message: String) {
    ensure_server_booted(world).await;
    let base_url = world.base_url.clone().expect("server should be booted");
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{base_url}/api/chat/completion"))
        .json(&serde_json::json!({
            "model": MODEL,
            "messages": [{"role": "user", "content": message}],
            "stream": true,
            "stream_mode": stream_mode,
        }))
        .send()
        .await
        .expect("request");
    world.response_status = Some(resp.status().as_u16());
    world.response_body = resp.text().await.expect("response body");
}

#[when(
    regex = r#"^I send a chat completion request continuing the conversation: user "([^"]+)", assistant "([^"]+)", then user "([^"]+)"$"#
)]
async fn when_multi_turn_request(
    world: &mut World,
    first_user: String,
    prior_assistant: String,
    latest_user: String,
) {
    ensure_server_booted(world).await;
    let base_url = world.base_url.clone().expect("server should be booted");
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{base_url}/v1/chat/completions"))
        .json(&serde_json::json!({
            "model": MODEL,
            "messages": [
                {"role": "user", "content": first_user},
                {"role": "assistant", "content": prior_assistant},
                {"role": "user", "content": latest_user},
            ]
        }))
        .send()
        .await
        .expect("request");
    world.response_status = Some(resp.status().as_u16());
    world.response_body = resp.text().await.expect("response body");
}

#[when(regex = r#"^I send a chat completion request with no "messages" field$"#)]
async fn when_malformed_request(world: &mut World) {
    ensure_server_booted(world).await;
    let base_url = world.base_url.clone().expect("server should be booted");
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{base_url}/v1/chat/completions"))
        .json(&serde_json::json!({ "model": MODEL }))
        .send()
        .await
        .expect("request");
    world.response_status = Some(resp.status().as_u16());
    world.response_body = resp.text().await.expect("response body");
}

#[then("the response status should be a client error")]
fn then_status_client_error(world: &mut World) {
    let status = world
        .response_status
        .expect("a request should have been sent");
    assert!(
        (400..500).contains(&status),
        "expected 4xx client error, got: {status}"
    );
    assert!(
        !(500..600).contains(&status),
        "expected client error but got server error: {status}"
    );
}

#[then("the response status should be successful")]
fn then_status_successful(world: &mut World) {
    let status = world
        .response_status
        .expect("a request should have been sent");
    assert!((200..300).contains(&status), "status: {status}");
}

#[then(
    regex = r#"^the response body should be an OpenAI chat\.completion with content "([^"]+)"$"#
)]
fn then_openai_completion_content(world: &mut World, expected_content: String) {
    let body: serde_json::Value =
        serde_json::from_str(&world.response_body).expect("JSON response body");
    assert_eq!(body["object"], "chat.completion");
    assert_eq!(body["choices"][0]["message"]["role"], "assistant");
    assert_eq!(body["choices"][0]["message"]["content"], expected_content);
}

#[then(regex = r#"^the response should contain the event "([^"]+)"$"#)]
fn then_contains_event(world: &mut World, event_name: String) {
    assert!(
        world.response_body.contains(&event_name),
        "expected event {event_name:?}, got: {}",
        world.response_body
    );
}

#[then(regex = r#"^the response should not contain the event "([^"]+)"$"#)]
fn then_not_contains_event(world: &mut World, event_name: String) {
    assert!(
        !world.response_body.contains(&event_name),
        "did not expect event {event_name:?}, got: {}",
        world.response_body
    );
}

#[then(regex = r#"^the response should contain the legacy event "([^"]+)"$"#)]
fn then_contains_legacy_event(world: &mut World, event_name: String) {
    assert!(
        world.response_body.contains(&event_name),
        "expected legacy event {event_name:?}, got: {}",
        world.response_body
    );
}

#[then(regex = r#"^the response should not contain the legacy event "([^"]+)"$"#)]
fn then_not_contains_legacy_event(world: &mut World, event_name: String) {
    assert!(
        !world.response_body.contains(&event_name),
        "did not expect legacy event {event_name:?}, got: {}",
        world.response_body
    );
}

#[tokio::main]
async fn main() {
    World::run("tests/features").await;
}
