//! Controlled wire endpoints; all requests reach the actual selected driver.
use axum::{
    Json, Router,
    body::Body,
    extract::{OriginalUri, State},
    http::header,
    response::Response,
};
use serde_json::{Value, json};
use std::{
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tokio::task::JoinHandle;

pub const TEXT: &str = "stream-control:λ🙂";
pub const TOOL: &str = "stream_echo";

#[derive(Clone, Copy, Debug)]
pub enum Protocol {
    OpenAi,
    Azure,
    LiterAnthropic,
    NativeAnthropic,
    Bedrock,
    GoogleAi,
    Vertex,
    Cohere,
}
impl Protocol {
    pub fn model(self) -> &'static str {
        match self {
            Self::OpenAi => "openai/gpt-4o-mini",
            Self::Azure => "azure/gpt-4o-mini",
            Self::LiterAnthropic | Self::NativeAnthropic => "anthropic/claude-sonnet-4-20250514",
            Self::GoogleAi => "google_ai/gemini-2.5-flash",
            Self::Vertex => "vertex_ai/gemini-2.5-flash",
            Self::Cohere => "cohere/command-r",
            Self::Bedrock => "bedrock/anthropic.claude-3-5-sonnet-20241022-v2:0",
        }
    }
}

#[derive(Clone)]
struct Settings {
    protocol: Protocol,
    case: &'static str,
    root: PathBuf,
    requests: Arc<Mutex<Vec<Value>>>,
}

pub struct Peer {
    pub url: String,
    pub requests: Arc<Mutex<Vec<Value>>>,
    server: JoinHandle<()>,
}
impl Peer {
    pub async fn start(root: &Path, protocol: Protocol, case: &'static str) -> Self {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let router = Router::new().fallback(reply).with_state(Settings {
            protocol,
            case,
            root: root.to_path_buf(),
            requests: requests.clone(),
        });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        Self {
            url,
            requests,
            server,
        }
    }
}
impl Drop for Peer {
    fn drop(&mut self) {
        self.server.abort();
    }
}

fn sse(value: Value, named: bool) -> Vec<u8> {
    let event = if named {
        format!("event: {}\r\n", value["type"].as_str().unwrap())
    } else {
        String::new()
    };
    // Multiline SSE JSON requires joining data lines before JSON decoding.
    let payload = serde_json::to_string_pretty(&value).unwrap();
    format!(
        "{event}{}\r\n",
        payload
            .lines()
            .map(|line| format!("data: {line}\r\n"))
            .collect::<String>()
    )
    .into_bytes()
}

fn crc(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb88320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}
fn frame(kind: &str, value: Value) -> Vec<u8> {
    let payload = value.to_string();
    let mut headers = Vec::new();
    for (name, value) in [(":message-type", "event"), (":event-type", kind)] {
        headers.push(u8::try_from(name.len()).unwrap());
        headers.extend_from_slice(name.as_bytes());
        headers.push(7);
        headers.extend_from_slice(&u16::try_from(value.len()).unwrap().to_be_bytes());
        headers.extend_from_slice(value.as_bytes());
    }
    let mut bytes = Vec::new();
    bytes.extend_from_slice(
        &u32::try_from(16 + headers.len() + payload.len())
            .unwrap()
            .to_be_bytes(),
    );
    bytes.extend_from_slice(&u32::try_from(headers.len()).unwrap().to_be_bytes());
    bytes.extend_from_slice(&crc(&bytes).to_be_bytes());
    bytes.extend_from_slice(&headers);
    bytes.extend_from_slice(payload.as_bytes());
    bytes.extend_from_slice(&crc(&bytes).to_be_bytes());
    bytes
}

pub fn wire(protocol: Protocol, case: &str, second_turn: bool) -> Vec<u8> {
    if case == "empty" {
        return Vec::new();
    }
    let tool = matches!(case, "tool" | "tool_missing_terminal") && !second_turn;
    let terminal = !matches!(
        case,
        "missing_terminal" | "tool_missing_terminal" | "cancel"
    );
    let mut bytes = Vec::new();
    match protocol {
        Protocol::OpenAi | Protocol::Azure => {
            let chunk = |delta: Value, reason: Value| {
                sse(
                    json!({
                        "id":"stream-fixture","object":"chat.completion.chunk","created":0,"model":"gpt-4o-mini",
                        "choices":[{"index":0,"delta":delta,"finish_reason":reason}]
                    }),
                    false,
                )
            };
            if tool {
                bytes.extend(chunk(json!({"tool_calls":[{"index":0,"id":"call-stream","type":"function","function":{"name":TOOL,"arguments":"{\"text\":"}}]}), Value::Null));
                bytes.extend(chunk(
                    json!({"tool_calls":[{"index":0,"function":{"arguments":"\"λ🙂\"}"}}]}),
                    Value::Null,
                ));
            } else {
                bytes.extend(chunk(json!({"content":TEXT}), Value::Null));
            }
            bytes.extend(chunk(
                json!({}),
                json!(if tool { "tool_calls" } else { "stop" }),
            ));
            if case == "malformed" {
                bytes.extend_from_slice(b"data: {bad}\r\n\r\n");
            }
            if terminal {
                bytes.extend_from_slice(b"data: [DONE]\r\n\r\n");
            }
        }
        Protocol::LiterAnthropic | Protocol::NativeAnthropic => {
            bytes.extend(sse(json!({"type":"message_start","message":{"id":"stream-fixture","model":"claude-sonnet-4-20250514","role":"assistant","usage":{"input_tokens":2,"output_tokens":0}}}), true));
            bytes.extend(sse(json!({"type":"content_block_start","index":0,"content_block":if tool { json!({"type":"tool_use","id":"call-stream","name":TOOL}) } else { json!({"type":"text","text":""}) }}), true));
            if tool {
                for part in ["{\"text\":", "\"λ🙂\"}"] {
                    bytes.extend(sse(json!({"type":"content_block_delta","index":0,"delta":{"type":"input_json_delta","partial_json":part}}), true));
                }
            } else {
                bytes.extend(sse(json!({"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":TEXT}}), true));
            }
            bytes.extend(sse(json!({"type":"content_block_stop","index":0}), true));
            bytes.extend(sse(json!({"type":"message_delta","delta":{"stop_reason":if tool {"tool_use"} else {"end_turn"},"stop_sequence":null},"usage":{"output_tokens":3}}), true));
            if case == "malformed" {
                bytes.extend_from_slice(b"event: content_block_delta\r\ndata: {bad}\r\n\r\n");
            }
            if terminal {
                bytes.extend(sse(json!({"type":"message_stop"}), true));
            }
        }
        Protocol::GoogleAi | Protocol::Vertex => {
            let parts = if tool {
                json!([{"functionCall":{"name":TOOL,"args":{"text":"λ🙂"}}}])
            } else {
                json!([{"text":TEXT}])
            };
            bytes.extend(sse(
                json!({"candidates":[{"content":{"role":"model","parts":parts}}]}),
                false,
            ));
            if case == "malformed" {
                bytes.extend_from_slice(b"data: {bad}\r\n\r\n");
            }
            if terminal {
                bytes.extend(sse(json!({"candidates":[{"content":{"role":"model","parts":[]},"finishReason":"STOP"}]}), false));
            }
        }
        Protocol::Cohere => {
            bytes.extend(sse(json!({"type":"message-start","id":"stream-fixture","delta":{"message":{"role":"assistant"}}}), true));
            if tool {
                bytes.extend(sse(json!({"type":"tool-call-start","index":0,"delta":{"message":{"tool_calls":{"id":"call-stream","type":"function","function":{"name":TOOL,"arguments":""}}}}}), true));
                for part in ["{\"text\":", "\"λ🙂\"}"] {
                    bytes.extend(sse(json!({"type":"tool-call-delta","index":0,"delta":{"message":{"tool_calls":{"function":{"arguments":part}}}}}), true));
                }
                bytes.extend(sse(json!({"type":"tool-call-end","index":0}), true));
            } else {
                bytes.extend(sse(
                    json!({"type":"content-delta","delta":{"message":{"content":{"text":TEXT}}}}),
                    true,
                ));
            }
            if case == "malformed" {
                bytes.extend_from_slice(b"data: {bad}\r\n\r\n");
            }
            if terminal {
                bytes.extend(sse(json!({"type":"message-end","delta":{"finish_reason":if tool {"TOOL_CALL"} else {"COMPLETE"},"usage":{"billed_units":{"input_tokens":2,"output_tokens":3}}}}), true));
            }
        }
        Protocol::Bedrock => {
            bytes.extend(frame("messageStart", json!({"role":"assistant"})));
            if tool {
                bytes.extend(frame("contentBlockStart", json!({"contentBlockIndex":0,"start":{"toolUse":{"toolUseId":"call-stream","name":TOOL}}})));
                for part in ["{\"text\":", "\"λ🙂\"}"] {
                    bytes.extend(frame(
                        "contentBlockDelta",
                        json!({"contentBlockIndex":0,"delta":{"toolUse":{"input":part}}}),
                    ));
                }
            } else {
                bytes.extend(frame(
                    "contentBlockDelta",
                    json!({"contentBlockIndex":0,"delta":{"text":TEXT}}),
                ));
            }
            if case == "malformed" {
                let mut bad = frame("contentBlockStop", json!({"contentBlockIndex":0}));
                let last = bad.len() - 1;
                bad[last] ^= 1;
                bytes.extend(bad);
            }
            if terminal {
                bytes.extend(frame(
                    "messageStop",
                    json!({"stopReason":if tool {"tool_use"} else {"end_turn"}}),
                ));
            }
        }
    }
    match case {
        "partial_after_terminal" => {
            bytes.extend_from_slice(if matches!(protocol, Protocol::Bedrock) {
                b"\0\0"
            } else {
                b"data: {"
            })
        }
        "partial_event" if !matches!(protocol, Protocol::Bedrock) => {
            bytes.truncate(bytes.len() - 2);
        }
        "invalid_utf8" if !matches!(protocol, Protocol::Bedrock) => {
            bytes.extend_from_slice(b"data: \xff\r\n\r\n")
        }
        "malformed_after_terminal" if !matches!(protocol, Protocol::Bedrock) => {
            bytes.extend_from_slice(b"data: {bad}\r\n\r\n")
        }
        _ => {}
    }
    bytes
}

async fn reply(
    State(settings): State<Settings>,
    OriginalUri(uri): OriginalUri,
    Json(request): Json<Value>,
) -> Response {
    let second_turn = {
        let mut requests = settings.requests.lock().unwrap();
        let trace = json!({"path":uri.path_and_query().unwrap().as_str(),"request":request});
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(settings.root.join("requests.jsonl"))
            .unwrap();
        writeln!(file, "{trace}").unwrap();
        requests.push(trace);
        requests.len() > 1
    };
    let bytes = wire(settings.protocol, settings.case, second_turn);
    let split = settings.case == "split" || settings.case == "tool";
    let cancel = settings.case == "cancel";
    let chunks = async_stream::stream! {
        if split {
            for byte in bytes {
                yield Ok::<_, std::io::Error>(vec![byte]);
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            }
        } else { yield Ok::<_, std::io::Error>(bytes); }
        if cancel { std::future::pending::<()>().await; }
    };
    Response::builder()
        .header(
            header::CONTENT_TYPE,
            if matches!(settings.protocol, Protocol::Bedrock) {
                "application/vnd.amazon.eventstream"
            } else {
                "text/event-stream"
            },
        )
        .body(Body::from_stream(chunks))
        .unwrap()
}
