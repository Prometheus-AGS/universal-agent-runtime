//! Setup, stream collection, and retained-copy assertions for projection scenarios.
use super::{
    provider::{MCP_SECRET, PROVIDER_SECRET, variants},
    sidecar_process,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};

pub(super) fn assert_projected(text: &str) {
    for value in [variants(PROVIDER_SECRET), variants(MCP_SECRET)].concat() {
        assert!(
            !text.contains(&value),
            "captured credential variant reached an ordinary sink"
        );
    }
}

pub(super) fn assert_ingress_projected(text: &str, token: &str) {
    for value in [variants(token), variants(&format!("Bearer {token}"))].concat() {
        assert!(
            !text.contains(&value),
            "authenticated ingress variant reached an ordinary sink"
        );
    }
}

pub(super) async fn indexed_kb(client: &reqwest::Client, base: &str, token: &str) -> String {
    let response = client.post(format!("{base}/api/knowledge")).bearer_auth(token)
        .header("X-UAR-Principal", "projection-owner")
        .json(&json!({"name":"projection-routing","description":"Synthetic routing positive control"}))
        .send().await.unwrap();
    assert!(response.status().is_success());
    let kb: Value = response.json().await.unwrap();
    let id = kb["id"].as_str().unwrap().to_owned();
    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(
            b"early-positive-control projection-routing document".to_vec(),
        )
        .file_name("projection.txt")
        .mime_str("text/plain")
        .unwrap(),
    );
    let response = client
        .post(format!("{base}/api/knowledge/{id}/documents"))
        .bearer_auth(token)
        .header("X-UAR-Principal", "projection-owner")
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        let response = client
            .post(format!("{base}/api/knowledge/{id}/search"))
            .bearer_auth(token)
            .header("X-UAR-Principal", "projection-owner")
            .json(&json!({"query":"early-positive-control","min_score":0.0}))
            .send()
            .await
            .unwrap();
        let status = response.status();
        // Report only status and a fixed media-type label, never body or headers
        // that could contain model output or fixture credentials.
        let content_type = match response.headers().get(reqwest::header::CONTENT_TYPE) {
            None => "missing",
            Some(value) => match value
                .to_str()
                .ok()
                .map(|value| value.split(';').next().unwrap_or_default().trim())
            {
                Some("application/json") => "application/json",
                Some("text/plain") => "text/plain",
                Some("text/html") => "text/html",
                _ => "other",
            },
        };
        assert!(
            status.is_success(),
            "KB search failed: status={status}, content_type={content_type}"
        );
        assert_eq!(
            content_type, "application/json",
            "KB search returned non-JSON: status={status}, content_type={content_type}"
        );
        let body: Value = response.json().await.unwrap_or_else(|_| {
            panic!("KB search JSON decode failed: status={status}, content_type={content_type}")
        });
        if body["results"]
            .as_array()
            .is_some_and(|results| !results.is_empty())
        {
            return id;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "KB ingestion did not reach the routing control"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

pub(super) fn copied_strings(value: &Value, output: &mut String) {
    match value {
        Value::String(text) => {
            output.push_str(text);
            output.push('\n');
        }
        Value::Array(values) => {
            for value in values {
                copied_strings(value, output);
            }
        }
        Value::Object(values) => {
            for (key, value) in values {
                output.push_str(key);
                copied_strings(value, output);
            }
        }
        _ => {}
    }
}

pub(super) fn stream_channels(stream: &str) -> BTreeMap<String, String> {
    let mut channels = BTreeMap::new();
    for frame in stream.split("\n\n") {
        let event = frame
            .lines()
            .find_map(|line| line.strip_prefix("event: "))
            .unwrap_or_default();
        let Some(data) = frame.lines().find_map(|line| line.strip_prefix("data: ")) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<Value>(data) else {
            continue;
        };
        let value = value.get("data").unwrap_or(&value);
        // Default AG-UI wraps streamed text and tool arguments in delta objects.
        // Reassemble each wire channel so secrets split across frames stay visible
        // to the assertions, while preserving the existing flat-field handling.
        for path in [
            "/text_delta",
            "/text",
            "/delta",
            "/arguments_delta",
            "/delta/text",
            "/delta/arguments",
        ] {
            if let Some(text) = value.pointer(path).and_then(Value::as_str) {
                channels
                    .entry(format!("{event}:{path}"))
                    .or_insert_with(String::new)
                    .push_str(text);
            }
        }
    }
    channels
}

// Deliberately exclude terminal messages, request IDs, and raw stream content.
pub(super) fn terminal_diagnostic(stream: &str) -> String {
    for frame in stream.split("\n\n") {
        let event = match frame.lines().find_map(|line| line.strip_prefix("event: ")) {
            Some("agui.error") => "agui.error",
            Some("agui.cancelled") => "agui.cancelled",
            Some("agui.done") => "agui.done",
            _ => continue,
        };
        let value = frame
            .lines()
            .find_map(|line| line.strip_prefix("data: "))
            .and_then(|data| serde_json::from_str::<Value>(data).ok());
        let code = value
            .as_ref()
            .and_then(|value| value.get("code"))
            .and_then(Value::as_str)
            .filter(|code| {
                !code.is_empty()
                    && code.len() <= 64
                    && code.as_bytes()[0].is_ascii_alphabetic()
                    && code.bytes().all(|byte| {
                        byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.')
                    })
            })
            .unwrap_or("missing_or_invalid");
        return format!("event={event}, code={code}");
    }
    "event=missing, code=missing_or_invalid".to_owned()
}

pub(super) async fn follow(client: &reqwest::Client, base: &str, token: &str, run: &str) -> String {
    let mut response = client
        .get(format!("{base}/api/uar/runs/{run}/stream"))
        .bearer_auth(token)
        .header("X-UAR-Principal", "projection-owner")
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());
    let mut stream = String::new();
    let mut approved = BTreeSet::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(90);
    loop {
        for id in sidecar_process::approval_ids(&stream, run) {
            if approved.insert(id.clone()) {
                let response = client
                    .post(format!("{base}/api/uar/runs/{run}/tool-approval"))
                    .bearer_auth(token)
                    .header("X-UAR-Principal", "projection-owner")
                    .json(&json!({"approval_id":id,"approved":true}))
                    .send()
                    .await
                    .unwrap();
                assert!(response.status().is_success());
            }
        }
        if [
            "event: agui.done",
            "event: agui.error",
            "event: agui.cancelled",
        ]
        .iter()
        .any(|event| stream.contains(event))
        {
            return stream;
        }
        let chunk = tokio::time::timeout_at(deadline, response.chunk())
            .await
            .unwrap()
            .unwrap()
            .expect("terminal event");
        stream.push_str(&String::from_utf8_lossy(&chunk));
    }
}
