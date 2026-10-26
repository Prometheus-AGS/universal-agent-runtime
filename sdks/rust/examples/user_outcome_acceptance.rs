//! Actual Rust SDK acceptance client; never launches a runtime or mock driver.
use futures::StreamExt;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{collections::BTreeSet, error::Error, fs::OpenOptions, io::Write};
use universal_agent_runtime_sdk::{ChatCompletionRequest, ChatMessage, Client};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Deserialize)]
struct Input {
    base_url: String,
    authorization_env: String,
    agent_id: String,
    model: String,
    session_id: String,
    mode: String,
    turns: Vec<String>,
    expected: Vec<Value>,
    timeout_seconds: f64,
    max_bytes: usize,
}

fn require(condition: bool, message: &str) -> Result<()> {
    if condition { Ok(()) } else { Err(message.into()) }
}

async fn acceptance(input: Input) -> Result<Value> {
    require(input.turns.len() == 2 && input.expected.len() == 2, "exact two-turn input required")?;
    require(matches!(input.mode.as_str(), "json" | "stream"), "unknown SDK mode")?;
    require(input.timeout_seconds.is_finite() && input.timeout_seconds > 0.0, "finite timeout required")?;
    let authorization = std::env::var(&input.authorization_env)?;
    let token = authorization.strip_prefix("Bearer ").filter(|value| !value.is_empty())
        .ok_or("bearer credential required")?;
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs_f64(input.timeout_seconds)).build()?;
    let client = Client::with_client(&input.base_url, http)?.with_api_key(token);
    let mut turns = Vec::new();
    for (text, expected) in input.turns.iter().zip(&input.expected) {
        let request = ChatCompletionRequest {
            messages: vec![ChatMessage::text("user", text)],
            model: Some(input.model.clone()), session_id: Some(input.session_id.clone()),
            agent_id: Some(input.agent_id.clone()), ..Default::default()
        };
        let (content, identifier) = if input.mode == "json" {
            let response = client.chat().complete(request).await?;
            let choices = response.extra.get("choices").and_then(Value::as_array).ok_or("choices missing")?;
            require(choices.len() == 1 && choices[0]["finish_reason"] == "stop", "SDK completion failed")?;
            require(response.extra.get("model").and_then(Value::as_str) == Some(input.model.as_str()), "model mismatch")?;
            let content = choices[0].pointer("/message/content").and_then(Value::as_str).ok_or("content missing")?;
            (content.to_owned(), response.id)
        } else {
            let mut stream = client.chat().stream(request).await?;
            let mut content = String::new();
            let mut ids = BTreeSet::new();
            let mut terminal = 0;
            let mut done = 0;
            while let Some(item) = stream.next().await {
                let event = item?;
                if event.data == "[DONE]" { done += 1; continue; }
                require(event.data.get("error").is_none()
                    && !matches!(event.event.as_deref(), Some("error" | "agui.error" | "agui.cancelled")), "SDK stream error")?;
                if event.data.get("object").and_then(Value::as_str) != Some("chat.completion.chunk") { continue; }
                ids.insert(event.data.get("id").and_then(Value::as_str).ok_or("chunk id missing")?.to_owned());
                for choice in event.data.get("choices").and_then(Value::as_array).ok_or("chunk choices missing")? {
                    if let Some(fragment) = choice.pointer("/delta/content").and_then(Value::as_str) { content.push_str(fragment); }
                    if choice.get("finish_reason").is_some_and(|value| !value.is_null()) {
                        require(choice["finish_reason"] == "stop", "SDK stream terminal failed")?;
                        terminal += 1;
                    }
                }
                require(content.len() <= input.max_bytes, "SDK stream output exceeded bound")?;
            }
            require(ids.len() == 1 && terminal == 1 && done == 1, "SDK stream lacks successful terminal")?;
            (content, ids.into_iter().next().ok_or("stream id missing")?)
        };
        require(serde_json::from_str::<Value>(&content)? == *expected, "SDK useful-result predicate failed")?;
        let run_id = identifier.strip_prefix("chatcmpl-").ok_or("native run identity missing")?;
        turns.push(json!({"content":content,"run_id":run_id,"session_id":input.session_id,"protocol_terminal":"success"}));
    }
    Ok(json!({"status":"PASS","client":"rust_sdk","mode":input.mode,"turns":turns}))
}

#[tokio::main]
async fn main() -> Result<()> {
    let output = std::env::var("UAR_ACCEPTANCE_CLIENT_OUTPUT")?;
    let inputs: Result<Input> = std::env::var("UAR_ACCEPTANCE_CLIENT_INPUT").map_err(Into::into)
        .and_then(|value| serde_json::from_str::<Input>(&value).map_err(Into::into));
    let result = match inputs { Ok(input) => acceptance(input).await, Err(error) => Err(error) };
    let receipt = result.as_ref().map_or_else(|error| json!({"status":"FAIL","client":"rust_sdk",
        "error":error.to_string().chars().take(8192).collect::<String>()}), Value::clone);
    let mut file = OpenOptions::new().write(true).create_new(true).open(output)?;
    file.write_all(serde_json::to_string(&receipt)?.as_bytes())?;
    file.sync_all()?;
    result.map(|_| ())
}
