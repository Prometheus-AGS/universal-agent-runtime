//! A2A v1 wire projection and an owned SSE subscription. No reconnect policy,
//! mutation replay, timers or detached readers live at this boundary.

use anyhow::{Context, Result, anyhow, ensure};
use serde_json::{Value, json};

use super::{validate_binding, validate_governed_receipt};
use super::super::{contract::UarDelegationContract, types::{Artifact, Message, Part, Role, Task, TaskState, TaskStatus}};

// The remote peer controls framing. Bound a single unfinished event rather
// than permitting an untrusted stream to grow the host's memory indefinitely.
const MAX_EVENT_BYTES: usize = 8 * 1024 * 1024;

pub(super) fn message(value: &Message, task_id: Option<&str>, context_id: Option<&str>) -> Result<Value> {
    let parts = value.parts.iter().map(|part| match part {
        Part::Text { text } => json!({"text": text}),
        Part::Data { data } => json!({"data": data}),
        Part::File { name, mime_type, uri, bytes } => {
            let mut value = json!({});
            if let Some(bytes) = bytes { value["raw"] = json!(bytes); }
            else if let Some(uri) = uri { value["url"] = json!(uri); }
            if let Some(name) = name { value["filename"] = json!(name); }
            if let Some(mime) = mime_type { value["mediaType"] = json!(mime); }
            value
        }
    }).collect::<Vec<_>>();
    ensure!(!parts.is_empty(), "A2A message has no parts");
    let mut result = json!({"messageId": uuid::Uuid::new_v4().to_string(),
        "role": match value.role { Role::User => "ROLE_USER", Role::Agent => "ROLE_AGENT" },
        "parts": parts, "metadata": value.metadata});
    if let Some(id) = task_id { result["taskId"] = json!(id); }
    if let Some(id) = context_id { result["contextId"] = json!(id); }
    Ok(result)
}

fn parts(value: &Value) -> Result<Vec<Part>> {
    value.as_array().ok_or_else(|| anyhow!("A2A parts are missing"))?.iter().map(|value| {
        let count = ["text", "data", "raw", "url"].iter().filter(|key| value.get(**key).is_some()).count();
        ensure!(count == 1, "A2A Part requires exactly one content field");
        if let Some(text) = value.get("text").and_then(Value::as_str) { return Ok(Part::text(text)); }
        if let Some(data) = value.get("data") { return Ok(Part::Data { data: data.clone() }); }
        let bytes = value.get("raw").and_then(Value::as_str).map(str::to_owned);
        let uri = value.get("url").and_then(Value::as_str).map(str::to_owned);
        ensure!(bytes.is_some() || uri.is_some(), "invalid A2A file content");
        Ok(Part::File { name: value.get("filename").and_then(Value::as_str).map(str::to_owned),
            mime_type: value.get("mediaType").and_then(Value::as_str).map(str::to_owned), bytes, uri })
    }).collect()
}

fn decode_message(value: &Value) -> Result<Message> {
    ensure!(value.get("messageId").and_then(Value::as_str).is_some_and(|id| !id.trim().is_empty()), "A2A message has no identity");
    Ok(Message {
        role: match value.get("role").and_then(Value::as_str) {
            Some("ROLE_USER") => Role::User, Some("ROLE_AGENT") => Role::Agent,
            _ => return Err(anyhow!("unknown A2A message role")),
        },
        parts: parts(&value["parts"])?,
        metadata: value.get("metadata").cloned().map(serde_json::from_value).transpose()?.unwrap_or_default(),
    })
}

fn status(value: &Value) -> Result<TaskStatus> {
    Ok(TaskStatus {
        state: match value.get("state").and_then(Value::as_str) {
            Some("TASK_STATE_SUBMITTED") => TaskState::Submitted,
            Some("TASK_STATE_WORKING") => TaskState::Working,
            Some("TASK_STATE_COMPLETED") => TaskState::Completed,
            Some("TASK_STATE_CANCELED") => TaskState::Canceled,
            Some("TASK_STATE_FAILED" | "TASK_STATE_REJECTED") => TaskState::Failed,
            Some("TASK_STATE_INPUT_REQUIRED" | "TASK_STATE_AUTH_REQUIRED") => TaskState::InputRequired,
            _ => return Err(anyhow!("A2A task execution is unknown")),
        },
        message: value.get("message").filter(|v| !v.is_null()).map(decode_message).transpose()?,
        timestamp: value.get("timestamp").and_then(Value::as_str).map(str::to_owned),
    })
}

fn artifact(value: &Value) -> Result<Artifact> {
    let id = value.get("artifactId").and_then(Value::as_str).filter(|id| !id.is_empty())
        .ok_or_else(|| anyhow!("A2A artifact has no identity"))?;
    Ok(Artifact { artifact_id: id.to_owned(),
        name: value.get("name").and_then(Value::as_str).map(str::to_owned),
        description: value.get("description").and_then(Value::as_str).map(str::to_owned),
        parts: parts(&value["parts"])?,
        metadata: value.get("metadata").cloned().map(serde_json::from_value).transpose()?.unwrap_or_default(),
    })
}

pub(super) fn task(value: &Value) -> Result<Task> {
    let task = Task {
        id: value.get("id").and_then(Value::as_str).ok_or_else(|| anyhow!("A2A task has no identity"))?.to_owned(),
        context_id: value.get("contextId").and_then(Value::as_str).filter(|id| !id.is_empty()).map(str::to_owned),
        status: status(&value["status"])?,
        history: value.get("history").and_then(Value::as_array).map(|values| values.iter().map(decode_message).collect()).transpose()?.unwrap_or_default(),
        artifacts: value.get("artifacts").and_then(Value::as_array).map(|values| values.iter().map(artifact).collect()).transpose()?.unwrap_or_default(),
        metadata: value.get("metadata").cloned().map(serde_json::from_value).transpose()?.unwrap_or_default(),
    };
    validate_binding(&task, None, None)?;
    Ok(task)
}

pub(in crate::uar::api::a2a) struct A2ATaskStream {
    response: reqwest::Response,
    request_id: String,
    task: Task,
    contract: Option<UarDelegationContract>,
    first: bool,
    buffer: Vec<u8>,
    data: Vec<String>,
    event_bytes: usize,
    skip_lf: bool,
    first_line: bool,
}

impl A2ATaskStream {
    pub(super) fn new(response: reqwest::Response, request_id: String, task: Task, contract: Option<UarDelegationContract>) -> Self {
        Self { response, request_id, task, contract, first: true, buffer: Vec::new(), data: Vec::new(), event_bytes: 0, skip_lf: false, first_line: true }
    }

    pub(in crate::uar::api::a2a) async fn next_task(&mut self) -> Result<Task> {
        loop {
            if let Some(index) = self.buffer.iter().position(|byte| matches!(*byte, b'\r' | b'\n')) {
                let delimiter = self.buffer[index];
                let mut bytes = self.buffer.drain(..=index).collect::<Vec<_>>();
                bytes.pop();
                if self.skip_lf && delimiter == b'\n' && bytes.is_empty() { self.skip_lf = false; continue; }
                self.skip_lf = delimiter == b'\r';
                self.event_bytes += bytes.len();
                ensure!(self.event_bytes <= MAX_EVENT_BYTES, "A2A stream event exceeds host frame limit");
                let line = std::str::from_utf8(&bytes).context("invalid UTF-8 in A2A stream")?;
                let line = if self.first_line {
                    self.first_line = false;
                    line.strip_prefix('\u{feff}').unwrap_or(line)
                } else { line };
                if line.is_empty() {
                    self.event_bytes = 0;
                    if self.data.is_empty() { continue; }
                    let payload = self.data.join("\n");
                    self.data.clear();
                    return self.apply(serde_json::from_str(&payload).context("invalid A2A SSE JSON")?);
                }
                if let Some(value) = line.strip_prefix("data:") {
                    self.data.push(value.strip_prefix(' ').unwrap_or(value).to_owned());
                }
                continue;
            }
            ensure!(self.buffer.len() + self.event_bytes <= MAX_EVENT_BYTES, "A2A stream event exceeds host frame limit");
            let chunk = self.response.chunk().await.context("A2A stream connection lost")?
                .ok_or_else(|| anyhow!("A2A stream ended without a terminal receipt"))?;
            self.buffer.extend_from_slice(&chunk);
        }
    }

    fn apply(&mut self, envelope: Value) -> Result<Task> {
        ensure!(envelope.get("jsonrpc").and_then(Value::as_str) == Some("2.0")
            && envelope.get("id").and_then(Value::as_str) == Some(self.request_id.as_str()), "A2A event belongs to another request");
        ensure!(envelope.get("error").is_none(), "A2A event stream returned an error");
        let result = envelope.get("result").ok_or_else(|| anyhow!("A2A event has no result"))?;
        ensure!(["task", "message", "statusUpdate", "artifactUpdate"].iter().filter(|key| result.get(**key).is_some()).count() == 1,
            "A2A stream requires exactly one payload");
        if let Some(value) = result.get("task") {
            ensure!(self.first, "A2A stream repeated its initial task snapshot");
            let next = task(value)?;
            validate_binding(&next, Some(&self.task.id), self.task.context_id.as_deref())?;
            ensure!(next.context_id == self.task.context_id, "A2A snapshot changed its context binding");
            if let Some(contract) = &self.contract { validate_governed_receipt(&next, contract)?; }
            self.task = next;
            self.first = false;
            return Ok(self.task.clone());
        }
        ensure!(!self.first, "A2A subscription did not start with a task snapshot");
        let update = result.get("statusUpdate").or_else(|| result.get("artifactUpdate"))
            .ok_or_else(|| anyhow!("unexpected standalone message in task subscription"))?;
        ensure!(update.get("taskId").and_then(Value::as_str) == Some(self.task.id.as_str())
            && update.get("contextId").and_then(Value::as_str) == self.task.context_id.as_deref(),
            "A2A event belongs to another task or context");
        // A governed delta must acknowledge this contract itself; cached
        // acknowledgement metadata cannot turn an unrelated delta into proof.
        if let Some(contract) = &self.contract {
            let mut receipt = self.task.clone();
            receipt.metadata = update.get("metadata").cloned().map(serde_json::from_value).transpose()?.unwrap_or_default();
            validate_governed_receipt(&receipt, contract)?;
        }
        if let Some(value) = result.get("statusUpdate") {
            self.task.status = status(&value["status"])?;
            if let Some(metadata) = value.get("metadata") {
                if self.contract.is_some() {
                    // UAR status events carry the complete receipt metadata.
                    // Replacement also permits clearing prior uncertainty.
                    self.task.metadata = serde_json::from_value(metadata.clone())?;
                } else {
                    self.task.metadata.extend(serde_json::from_value::<std::collections::HashMap<String, Value>>(metadata.clone())?);
                }
            }
        } else {
            let next = artifact(&update["artifact"])?;
            if let Some(index) = self.task.artifacts.iter().position(|value| value.artifact_id == next.artifact_id) {
                if update.get("append").and_then(Value::as_bool) == Some(true) {
                    self.task.artifacts[index].parts.extend(next.parts);
                } else { self.task.artifacts[index] = next; }
            } else {
                ensure!(update.get("append").and_then(Value::as_bool) != Some(true), "A2A artifact append has no initial artifact");
                self.task.artifacts.push(next);
            }
        }
        Ok(self.task.clone())
    }
}
