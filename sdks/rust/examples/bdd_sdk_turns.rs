//! BDD driver: two chat turns through the shipped Rust SDK, with no credentials.
//! Reads JSON inputs on stdin and writes one JSON receipt to stdout; the
//! Cucumber steps judge the answers.
use futures::StreamExt;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{collections::BTreeSet, error::Error, io::Read};
use universal_agent_runtime_sdk::{ChatCompletionRequest, ChatMessage, Client};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Deserialize)]
struct Input {
    base_url: String,
    workspace: String,
    mode: String,
    model: String,
    agent_id: String,
    session_id: String,
    turns: Vec<String>,
    timeout: f64,
}

async fn turn(client: &Client, input: &Input, text: &str) -> Result<Value> {
    let request = ChatCompletionRequest {
        messages: vec![ChatMessage::text("user", text)],
        model: Some(input.model.clone()), session_id: Some(input.session_id.clone()),
        agent_id: Some(input.agent_id.clone()), ..Default::default()
    };
    if input.mode == "json" {
        let response = client.chat().complete(request).await?;
        let choices = response.extra.get("choices").and_then(Value::as_array).cloned().unwrap_or_default();
        return Ok(json!({"id": response.id, "choices": choices.len(),
            "finish_reason": choices.first().map(|choice| choice["finish_reason"].clone()),
            "content": choices.first().and_then(|choice| choice.pointer("/message/content")).cloned()}));
    }
    let mut stream = client.chat().stream(request).await?;
    let (mut chunks, mut ids, mut finishes, mut events) = (Vec::new(), BTreeSet::new(), Vec::new(), BTreeSet::new());
    while let Some(item) = stream.next().await {
        let event = item?;
        events.insert(event.event.clone().unwrap_or_else(|| "message".into()));
        if event.data.get("object").and_then(Value::as_str) != Some("chat.completion.chunk") { continue; }
        if let Some(id) = event.data.get("id").and_then(Value::as_str) { ids.insert(id.to_owned()); }
        for choice in event.data.get("choices").and_then(Value::as_array).into_iter().flatten() {
            if let Some(fragment) = choice.pointer("/delta/content").and_then(Value::as_str).filter(|text| !text.is_empty()) {
                chunks.push(fragment.to_owned());
            }
            if !choice["finish_reason"].is_null() { finishes.push(choice["finish_reason"].clone()); }
        }
    }
    Ok(json!({"ids": ids, "content_chunks": chunks.len(), "finish_reasons": finishes,
        "events": events, "content": chunks.concat()}))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut raw = String::new();
    std::io::stdin().read_to_string(&mut raw)?;
    let input: Input = serde_json::from_str(&raw)?;
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("x-uar-workspace-id", input.workspace.parse()?);
    let http = reqwest::Client::builder().default_headers(headers)
        .timeout(std::time::Duration::from_secs_f64(input.timeout)).build()?;
    let client = Client::with_client(&input.base_url, http)?;
    let mut turns = Vec::new();
    for text in &input.turns {
        turns.push(turn(&client, &input, text).await?);
    }
    print!("{}", json!({"client": "rust_sdk", "mode": input.mode, "turns": turns}));
    Ok(())
}
