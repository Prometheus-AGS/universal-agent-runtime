//! In-process OpenAI-compatible stub server for the `recorded` backend of the
//! live integration tier (proxy-integration-gate, design.md D1).
//!
//! Both the `live` and `recorded` backends run identical test cases through
//! identical code (a real HTTP client pointed at `UAR_LLM__BASE_URL`) — the
//! only difference is what that URL resolves to. `recorded` points at this
//! stub instead of the real proxy at `127.0.0.1:8181`, so CI (which cannot
//! reach the operator's local proxy) still exercises the full wire format.
//!
//! Wire format matches the OpenAI-compatible `ChatCompletionChunk` /
//! `ChatCompletionResponse` shapes `liter-llm`'s default provider parses
//! (straight JSON per SSE `data:` line, `[DONE]` sentinel, `choices[0].delta`
//! for streaming) — see `liter-llm`'s `Provider::parse_stream_event` default
//! impl and `crate::types::chat`.

use std::collections::HashMap;
use std::net::TcpListener as StdTcpListener;
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde_json::{Value, json};
use tokio::net::TcpListener;

/// A canned response for one fixture key: either plain assistant text, or a
/// single tool call the assistant "decides" to make.
#[derive(Debug, Clone)]
pub enum FixtureResponse {
    Content(String),
    ToolCall { name: String, arguments: String },
}

/// Fingerprint a request the same way regardless of streaming: by model, the
/// last user message's content, and whether any tool schema was offered.
/// This is deliberately coarse — fixtures key on "what is being asked for",
/// not exact request byte-equality.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequestFingerprint {
    pub model: String,
    pub last_user_message: String,
    pub has_tools: bool,
}

impl RequestFingerprint {
    fn from_request_body(body: &Value) -> Self {
        let model = body
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let has_tools = body
            .get("tools")
            .and_then(Value::as_array)
            .is_some_and(|t| !t.is_empty());
        let last_user_message = body
            .get("messages")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter(|m| m.get("role").and_then(Value::as_str) == Some("user"))
            .next_back()
            .and_then(|m| m.get("content"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        Self {
            model,
            last_user_message,
            has_tools,
        }
    }
}

/// The fixture table the stub server serves from. Build one with
/// `FixtureSet::new()` + `.with(...)`, then pass to [`start_stub_llm`].
#[derive(Debug, Clone, Default)]
pub struct FixtureSet {
    responses: HashMap<RequestFingerprint, FixtureResponse>,
}

impl FixtureSet {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with(mut self, fingerprint: RequestFingerprint, response: FixtureResponse) -> Self {
        self.responses.insert(fingerprint, response);
        self
    }
}

/// A running stub server. Drop (or let it fall out of scope) to stop it —
/// the underlying task is aborted when `handle` is dropped.
pub struct StubLlmServer {
    pub base_url: String,
    handle: tokio::task::JoinHandle<()>,
}

impl Drop for StubLlmServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

fn find_free_port() -> u16 {
    StdTcpListener::bind("127.0.0.1:0")
        .expect("bind ephemeral port")
        .local_addr()
        .expect("local_addr")
        .port()
}

/// Start the stub server on a free loopback port and return its base URL
/// (e.g. `http://127.0.0.1:54321/v1`), suitable for `UAR_LLM__BASE_URL`.
pub async fn start_stub_llm(fixtures: FixtureSet) -> StubLlmServer {
    let port = find_free_port();
    let state = Arc::new(fixtures);

    let app = Router::new()
        .route("/v1/models", get(models_handler))
        .route("/v1/chat/completions", post(chat_completions_handler))
        .with_state(state);

    let listener = TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .expect("bind stub llm listener");

    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("stub llm server");
    });

    StubLlmServer {
        base_url: format!("http://127.0.0.1:{port}/v1"),
        handle,
    }
}

/// Minimal `/v1/models` so the same health check
/// (`scripts/live-integration.sh`) used against the real proxy also passes
/// against the stub.
async fn models_handler() -> Json<Value> {
    Json(json!({ "object": "list", "data": [] }))
}

async fn chat_completions_handler(
    State(fixtures): State<Arc<FixtureSet>>,
    Json(body): Json<Value>,
) -> Response {
    let fingerprint = RequestFingerprint::from_request_body(&body);
    let Some(fixture) = fixtures.responses.get(&fingerprint) else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": {
                    "message": format!(
                        "no recorded fixture for model={:?} last_user_message={:?} has_tools={}",
                        fingerprint.model, fingerprint.last_user_message, fingerprint.has_tools
                    ),
                    "type": "stub_llm_missing_fixture",
                }
            })),
        )
            .into_response();
    };

    let stream = body.get("stream").and_then(Value::as_bool).unwrap_or(false);
    let model = fingerprint.model.clone();

    if stream {
        streaming_response(model, fixture.clone()).into_response()
    } else {
        non_streaming_response(model, fixture.clone()).into_response()
    }
}

fn non_streaming_response(model: String, fixture: FixtureResponse) -> Json<Value> {
    let finish_reason = match &fixture {
        FixtureResponse::Content(_) => "stop",
        FixtureResponse::ToolCall { .. } => "tool_calls",
    };
    let message = match fixture {
        FixtureResponse::Content(text) => json!({ "role": "assistant", "content": text }),
        FixtureResponse::ToolCall { name, arguments } => json!({
            "role": "assistant",
            "content": null,
            "tool_calls": [{
                "id": "call_stub_0",
                "type": "function",
                "function": { "name": name, "arguments": arguments },
            }],
        }),
    };
    Json(json!({
        "id": "chatcmpl-stub",
        "object": "chat.completion",
        "created": 0,
        "model": model,
        "choices": [{ "index": 0, "message": message, "finish_reason": finish_reason }],
    }))
}

/// Two-chunk SSE stream: role+content (or role+tool_calls) in chunk 1,
/// finish_reason in chunk 2, then `[DONE]`. Real providers emit far more
/// granular deltas; this is the minimum shape `liter-llm`'s parser needs to
/// reconstruct a complete message.
fn streaming_response(model: String, fixture: FixtureResponse) -> impl IntoResponse {
    let (delta, finish_reason) = match fixture {
        FixtureResponse::Content(text) => (json!({ "role": "assistant", "content": text }), "stop"),
        FixtureResponse::ToolCall { name, arguments } => (
            json!({
                "role": "assistant",
                "tool_calls": [{
                    "index": 0,
                    "id": "call_stub_0",
                    "type": "function",
                    "function": { "name": name, "arguments": arguments },
                }],
            }),
            "tool_calls",
        ),
    };

    let chunk1 = json!({
        "id": "chatcmpl-stub", "object": "chat.completion.chunk", "created": 0,
        "model": model, "choices": [{ "index": 0, "delta": delta, "finish_reason": null }],
    });
    let chunk2 = json!({
        "id": "chatcmpl-stub", "object": "chat.completion.chunk", "created": 0,
        "model": "", "choices": [{ "index": 0, "delta": {}, "finish_reason": finish_reason }],
    });

    let body = format!("data: {}\n\ndata: {}\n\ndata: [DONE]\n\n", chunk1, chunk2);

    (
        [(axum::http::header::CONTENT_TYPE, "text/event-stream")],
        body,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fp(model: &str, msg: &str, has_tools: bool) -> RequestFingerprint {
        RequestFingerprint {
            model: model.to_string(),
            last_user_message: msg.to_string(),
            has_tools,
        }
    }

    #[tokio::test]
    async fn non_streaming_content_round_trip() {
        let server = start_stub_llm(FixtureSet::new().with(
            fp("openai/gpt-5.4-mini", "hello", false),
            FixtureResponse::Content("hi there".to_string()),
        ))
        .await;

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/chat/completions", server.base_url))
            .json(&json!({
                "model": "openai/gpt-5.4-mini",
                "messages": [{"role": "user", "content": "hello"}],
                "stream": false,
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), 200);
        let body: Value = resp.json().await.unwrap();
        assert_eq!(
            body["choices"][0]["message"]["content"].as_str().unwrap(),
            "hi there"
        );
        assert_eq!(
            body["choices"][0]["finish_reason"].as_str().unwrap(),
            "stop"
        );
    }

    #[tokio::test]
    async fn streaming_content_emits_sse_chunks_and_done() {
        let server = start_stub_llm(FixtureSet::new().with(
            fp("openai/gpt-5.4-mini", "hello", false),
            FixtureResponse::Content("hi there".to_string()),
        ))
        .await;

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/chat/completions", server.base_url))
            .json(&json!({
                "model": "openai/gpt-5.4-mini",
                "messages": [{"role": "user", "content": "hello"}],
                "stream": true,
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), 200);
        let text = resp.text().await.unwrap();
        assert!(text.contains("\"content\":\"hi there\""));
        assert!(text.contains("\"finish_reason\":\"stop\""));
        assert!(text.trim_end().ends_with("data: [DONE]"));
    }

    #[tokio::test]
    async fn tool_call_fixture_round_trips_non_streaming() {
        let server = start_stub_llm(FixtureSet::new().with(
            fp("openai/gpt-5.4-mini", "what's the weather", true),
            FixtureResponse::ToolCall {
                name: "get_weather".to_string(),
                arguments: r#"{"city":"SF"}"#.to_string(),
            },
        ))
        .await;

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/chat/completions", server.base_url))
            .json(&json!({
                "model": "openai/gpt-5.4-mini",
                "messages": [{"role": "user", "content": "what's the weather"}],
                "tools": [{"type": "function", "function": {"name": "get_weather"}}],
                "stream": false,
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), 200);
        let body: Value = resp.json().await.unwrap();
        assert_eq!(
            body["choices"][0]["finish_reason"].as_str().unwrap(),
            "tool_calls"
        );
        assert_eq!(
            body["choices"][0]["message"]["tool_calls"][0]["function"]["name"]
                .as_str()
                .unwrap(),
            "get_weather"
        );
    }

    #[tokio::test]
    async fn missing_fixture_returns_clear_404() {
        let server = start_stub_llm(FixtureSet::new()).await;

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/chat/completions", server.base_url))
            .json(&json!({
                "model": "openai/gpt-5.4-mini",
                "messages": [{"role": "user", "content": "unrecorded question"}],
                "stream": false,
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), 404);
        let body: Value = resp.json().await.unwrap();
        assert_eq!(
            body["error"]["type"].as_str().unwrap(),
            "stub_llm_missing_fixture"
        );
    }

    #[tokio::test]
    async fn models_health_check_endpoint_ok() {
        let server = start_stub_llm(FixtureSet::new()).await;
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("{}/models", server.base_url))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
    }
}
