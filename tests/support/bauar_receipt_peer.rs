//! Synthetic acquisitions and HTTP responses; canonical receipts are made by UAR.
use axum::{
    Json, Router,
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
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};
use universal_agent_runtime::uar::{
    persistence::agent_threads::CanonicalRawSegment,
    runtime::native_skill::{NativeCanonicalReceiptData, NativeSkill},
    tools::descriptor::ToolEffect,
};

pub const SECRET: &str = "BAUAR-receipt+credential/quoted\"";
pub const TOOL: &str = "receipt_fixture";
pub const CONTROL: &str = "legitimate-execution-control";

pub fn digest(bytes: &[u8]) -> String {
    let hex: String = Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    format!("sha256:{hex}")
}

pub fn variants() -> Vec<String> {
    let quoted = serde_json::to_string(SECRET).unwrap();
    let percent = |lower: bool| {
        SECRET
            .bytes()
            .map(|byte| {
                if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
                    char::from(byte).to_string()
                } else if lower {
                    format!("%{byte:02x}")
                } else {
                    format!("%{byte:02X}")
                }
            })
            .collect::<String>()
    };
    vec![
        SECRET.into(),
        quoted[1..quoted.len() - 1].into(),
        percent(false),
        percent(true),
        STANDARD.encode(SECRET),
        STANDARD_NO_PAD.encode(SECRET),
        URL_SAFE.encode(SECRET),
        URL_SAFE_NO_PAD.encode(SECRET),
    ]
}

pub fn raw_text() -> String {
    format!("stream-positive-control:λ🦀 | {}", variants().join(" | "))
}
pub fn expected_text() -> String {
    format!(
        "stream-positive-control:λ🦀 | {}",
        ["[REDACTED]"; 8].join(" | ")
    )
}

pub fn typed_result(mode: &str, echo: &str) -> Value {
    json!({"mode":mode, "echo":echo, "control":CONTROL, "text":raw_text()})
}

pub fn expected_value(mode: &str) -> Value {
    json!({"mode":mode, "echo":"[REDACTED]", "control":CONTROL, "text":expected_text()})
}

pub fn opaque_bytes() -> Vec<u8> {
    [vec![0xff], SECRET.as_bytes().to_vec()].concat()
}

pub fn observed_bytes(mode: &str) -> u64 {
    let typed = serde_json::to_vec(&typed_result(mode, SECRET))
        .unwrap()
        .len();
    let unavailable = if mode == "incomplete" {
        b"unavailable-stream".len()
    } else {
        0
    };
    (typed + raw_text().len() + opaque_bytes().len() + unavailable) as u64
}

#[derive(Default)]
pub struct Seen {
    pub requests: Mutex<Vec<Value>>,
    pub executions: Mutex<Vec<Value>>,
    pub acquisitions: Mutex<Vec<String>>,
}

pub struct ReceiptTool {
    pub seen: Arc<Seen>,
}

#[async_trait::async_trait]
impl NativeSkill for ReceiptTool {
    fn name(&self) -> &str {
        TOOL
    }
    fn description(&self) -> &str {
        "Return synthetic canonical acquisition streams."
    }
    fn effect(&self) -> ToolEffect {
        ToolEffect::ReadOnly
    }
    fn parameters_schema(&self) -> Value {
        json!({"type":"object", "properties":{
            "mode":{"type":"string","enum":["complete","incomplete"]},
            "echo":{"type":"string"}, "control":{"type":"string"}},
            "required":["mode","echo","control"], "additionalProperties":false})
    }
    async fn execute(&self, args: Value) -> anyhow::Result<Value> {
        self.seen.executions.lock().unwrap().push(args.clone());
        Ok(typed_result(
            args["mode"].as_str().unwrap(),
            args["echo"].as_str().unwrap(),
        ))
    }
    fn take_canonical_receipt_data(
        &self,
        result: &mut Value,
    ) -> anyhow::Result<NativeCanonicalReceiptData> {
        let mode = result["mode"].as_str().unwrap();
        self.seen.acquisitions.lock().unwrap().push(mode.to_owned());
        let mut segments = vec![
            CanonicalRawSegment::new(format!("stdout:{SECRET}"), raw_text().as_bytes()),
            CanonicalRawSegment::new("opaque", &opaque_bytes()),
        ];
        if mode == "incomplete" {
            segments.push(CanonicalRawSegment::from_acquisition(
                "unavailable",
                None,
                b"unavailable-stream".len() as u64,
                digest(b"unavailable-stream"),
            )?);
        }
        Ok(NativeCanonicalReceiptData {
            raw_segments: segments,
            acquisition_complete: mode == "complete",
            observed_bytes: observed_bytes(mode),
        })
    }
}

pub struct Peer {
    pub url: String,
    pub seen: Arc<Seen>,
    task: tokio::task::JoinHandle<()>,
}

impl Drop for Peer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl Peer {
    pub async fn start() -> Self {
        let seen = Arc::new(Seen::default());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/v1", listener.local_addr().unwrap());
        let app = Router::new()
            .route("/v1/chat/completions", post(completion))
            .with_state(seen.clone());
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        Self { url, seen, task }
    }
}

async fn completion(
    State(seen): State<Arc<Seen>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    if headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        != Some(format!("Bearer {SECRET}").as_str())
    {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    seen.requests.lock().unwrap().push(body.clone());
    let messages = body["messages"].as_array().unwrap();
    let post_tool = messages.iter().any(|message| message["role"] == "tool");
    let mode = messages
        .iter()
        .rev()
        .find(|message| message["role"] == "user")
        .unwrap()["content"]
        .as_str()
        .unwrap();
    let chunk = |delta: Value, finish: Value| {
        format!(
            "data: {}\n\n",
            json!({
        "id":"receipt-response", "object":"chat.completion.chunk", "created":1,
        "model":"gpt-4o-mini", "choices":[{"index":0,"delta":delta,"finish_reason":finish}]})
        )
    };
    let mut frames = String::new();
    if post_tool {
        frames.push_str(&chunk(
            json!({"role":"assistant","content":"post-tool-positive-control"}),
            Value::Null,
        ));
        frames.push_str(&chunk(json!({}), json!("stop")));
    } else {
        assert!(
            body["tools"]
                .as_array()
                .unwrap()
                .iter()
                .any(|tool| tool["function"]["name"] == TOOL)
        );
        let arguments = json!({"mode":mode,"echo":SECRET,"control":CONTROL}).to_string();
        frames.push_str(&chunk(
            json!({"role":"assistant","tool_calls":[{"index":0,
            "id":format!("receipt-{mode}"), "type":"function",
            "function":{"name":TOOL,"arguments":""}}]}),
            Value::Null,
        ));
        for character in arguments.chars() {
            frames.push_str(&chunk(
                json!({"tool_calls":[{"index":0,
                "function":{"arguments":character.to_string()}}]}),
                Value::Null,
            ));
        }
        frames.push_str(&chunk(json!({}), json!("tool_calls")));
    }
    frames.push_str("data: [DONE]\n\n");
    ([("content-type", "text/event-stream")], frames).into_response()
}
