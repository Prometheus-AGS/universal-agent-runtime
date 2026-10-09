//! Synthetic provider/receiver supporting real router and transport scenarios.
use axum::{
    Json, Router,
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
};
use base64::{
    Engine as _,
    engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD},
};
use serde_json::{Value, json};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

pub const PROVIDER_SECRET: &str = "BAUAR-upstream+credential/quoted\"";
pub const MCP_SECRET: &str = "BAUAR-downstream+credential/quoted\"";
pub const ROUTING_SECRET: &str = "BAUAR-routing-constructor-credential";

pub fn variants(secret: &str) -> Vec<String> {
    let quoted = serde_json::to_string(secret).unwrap();
    let percent = secret
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
                char::from(byte).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect::<String>();
    let lower_percent = secret
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
                char::from(byte).to_string()
            } else {
                format!("%{byte:02x}")
            }
        })
        .collect::<String>();
    vec![
        secret.into(),
        quoted[1..quoted.len() - 1].into(),
        percent,
        lower_percent,
        STANDARD.encode(secret),
        STANDARD_NO_PAD.encode(secret),
        URL_SAFE.encode(secret),
        URL_SAFE_NO_PAD.encode(secret),
    ]
}

#[derive(Default)]
pub struct Seen {
    pub requests: Mutex<Vec<Value>>,
    pub effects: Mutex<Vec<Value>>,
    pub embeddings: Mutex<Vec<Value>>,
    pub hold_routing: AtomicBool,
    pub routing_started: tokio::sync::Notify,
    pub routing_release: tokio::sync::Notify,
    ingress: String,
}

pub struct Provider {
    pub url: String,
    pub seen: Arc<Seen>,
    task: tokio::task::JoinHandle<()>,
}

impl Drop for Provider {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl Provider {
    pub async fn start(ingress: &str) -> Self {
        let seen = Arc::new(Seen {
            ingress: ingress.to_owned(),
            ..Seen::default()
        });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let app = Router::new()
            .route("/v1/chat/completions", post(completion))
            .route("/v1/embeddings", post(embedding))
            .route("/chat/completions", post(completion))
            .route("/mcp", post(mcp))
            .with_state(seen.clone());
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        Self { url, seen, task }
    }
}

fn auth(headers: &HeaderMap, expected: &str) -> bool {
    headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        == Some(format!("Bearer {expected}").as_str())
}

async fn completion(
    State(seen): State<Arc<Seen>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    if !auth(&headers, PROVIDER_SECRET) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    seen.requests.lock().unwrap().push(body.clone());
    let has_result = body["messages"]
        .as_array()
        .unwrap()
        .iter()
        .any(|message| message["role"] == "tool");
    let has_tool = body["tools"].as_array().is_some_and(|tools| {
        tools
            .iter()
            .any(|tool| tool["function"]["name"] == "projection__echo")
    });
    let mut frames = Vec::new();
    let chunk = |delta: Value, finish: Value| {
        format!(
            "data: {}\n\n",
            json!({
        "id":"projection-response","object":"chat.completion.chunk","created":1,"model":"gpt-5.4-mini",
        "choices":[{"index":0,"delta":delta,"finish_reason":finish}]})
        )
    };
    if has_tool && !has_result {
        let arguments =
            json!({"control":"legitimate-execution-control","echo":MCP_SECRET}).to_string();
        frames.push(chunk(
            json!({"role":"assistant","tool_calls":[{"index":0,"id":"projection-call",
            "type":"function","function":{"name":"projection__echo","arguments":""}}]}),
            Value::Null,
        ));
        for character in arguments.chars() {
            frames.push(chunk(
                json!({"tool_calls":[{"index":0,"function":{"arguments":character.to_string()}}]}),
                Value::Null,
            ));
        }
        frames.push(chunk(json!({}), json!("tool_calls")));
    } else {
        let text = format!(
            "positive-control:{}",
            [
                variants(PROVIDER_SECRET),
                variants(MCP_SECRET),
                variants(&seen.ingress),
                variants(&format!("Bearer {}", seen.ingress))
            ]
            .concat()
            .join(" | ")
        );
        for character in text.chars() {
            frames.push(chunk(json!({"content":character.to_string()}), Value::Null));
        }
        frames.push(chunk(json!({}), json!("stop")));
    }
    frames.push("data: [DONE]\n\n".into());
    let stream = tokio_stream::iter(frames.into_iter().map(Ok::<_, std::convert::Infallible>));
    (
        [("content-type", "text/event-stream")],
        Body::from_stream(stream),
    )
        .into_response()
}

async fn embedding(
    State(seen): State<Arc<Seen>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    if !auth(&headers, ROUTING_SECRET) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    seen.embeddings.lock().unwrap().push(body.clone());
    if seen.hold_routing.load(Ordering::SeqCst) {
        seen.routing_started.notify_one();
        seen.routing_release.notified().await;
    }
    let data = body["input"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(index, _)| json!({"index":index,"embedding":[1.0,0.0,0.0]}))
        .collect::<Vec<_>>();
    Json(json!({"data":data})).into_response()
}

async fn mcp(
    State(seen): State<Arc<Seen>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    let result = match body["method"].as_str().unwrap_or_default() {
        "initialize" => json!({"protocolVersion":body["params"]["protocolVersion"],
            "capabilities":{"tools":{}},"serverInfo":{"name":"projection-peer","version":"1"}}),
        "tools/list" => json!({"tools":[{"name":"echo","description":"Synthetic credential echo",
            "inputSchema":{"type":"object","properties":{"control":{"type":"string"},"echo":{"type":"string"}},
                "required":["control","echo"]}}]}),
        "tools/call" => {
            if !auth(&headers, MCP_SECRET) {
                return StatusCode::UNAUTHORIZED.into_response();
            }
            seen.effects
                .lock()
                .unwrap()
                .push(body["params"]["arguments"].clone());
            json!({"content":[{"type":"text","text":format!("legitimate-result {MCP_SECRET} {PROVIDER_SECRET}")}],
                "structuredContent":{"secret":MCP_SECRET,"upstream":PROVIDER_SECRET,"control":"legitimate-result"},"isError":false})
        }
        _ => return StatusCode::ACCEPTED.into_response(),
    };
    Json(json!({"jsonrpc":"2.0","id":body["id"],"result":result})).into_response()
}
