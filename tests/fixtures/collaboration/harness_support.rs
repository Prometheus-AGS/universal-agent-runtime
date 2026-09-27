//! Helpers for the C03 completed-path integration fixture.

use std::collections::BTreeMap;
use std::time::Duration;

use reqwest::{Method, StatusCode};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::{AGENT, MANIFEST, PROFILE, TEAM, WORKFLOW, WORKSPACE};

pub(super) fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn canonical(value: &Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .iter()
                .map(|(key, value)| (key.clone(), canonical(value)))
                .collect::<BTreeMap<_, _>>()
                .into_iter()
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.iter().map(canonical).collect()),
        value => value.clone(),
    }
}

pub(super) fn finalize(document: &mut Value) {
    document
        .as_object_mut()
        .expect("fixture document object")
        .remove("contentDigest");
    let bytes = serde_json::to_vec(&canonical(document)).expect("canonical fixture JSON");
    document["contentDigest"] = Value::String(sha256(&bytes));
}

pub(super) fn parse_fixture(source: &str) -> Value {
    serde_json::from_str(source).expect("checked-in collaboration fixture JSON")
}

pub(super) fn main_package(command_id: &str) -> Value {
    json!({
        "commandId": command_id,
        "manifest": MANIFEST,
        "files": {
            "agent-definition.json": AGENT,
            "team-definition.json": TEAM,
            "workflow-definition.json": WORKFLOW,
        }
    })
}

pub(super) fn single_agent_package(command_id: &str, package_id: &str, agent: &Value) -> Value {
    let source = format!(
        "{}\n",
        serde_json::to_string_pretty(agent).expect("agent fixture")
    );
    let reference = json!({
        "id": agent["id"], "version": agent["version"], "digest": agent["contentDigest"]
    });
    let mut manifest = json!({
        "profile": PROFILE,
        "kind": "PackageManifest",
        "id": package_id,
        "version": "1.0.0",
        "provenance": {"source": "C03 negative-path fixture", "authors": ["Prometheus-AGS"]},
        "requiredCapabilities": ["collaboration_definition_packages_v2"],
        "extensions": {},
        "entrypoints": [reference.clone()],
        "files": [{
            "path": "agent-definition.json", "kind": "AgentDefinition",
            "definition": reference, "byteDigest": sha256(source.as_bytes())
        }],
        "lock": [],
        "capabilityDeclarations": [{
            "capability": "collaboration_definition_packages_v2", "required": true
        }],
        "resolution": "exact-version-and-digest"
    });
    finalize(&mut manifest);
    json!({
        "commandId": command_id,
        "manifest": serde_json::to_string_pretty(&manifest).expect("manifest fixture"),
        "files": {"agent-definition.json": source}
    })
}

pub(super) struct Api<'a> {
    pub(super) client: &'a reqwest::Client,
    pub(super) base_url: &'a str,
    pub(super) token: &'a str,
}

impl Api<'_> {
    async fn request(
        &self,
        label: &str,
        method: Method,
        path: &str,
        body: Option<Value>,
        expected: StatusCode,
    ) -> Value {
        let mut request = self
            .client
            .request(method, format!("{}{path}", self.base_url))
            .bearer_auth(self.token)
            .header("x-uar-workspace-id", WORKSPACE);
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send().await.expect("C03 real-server request");
        let status = response.status();
        let text = response.text().await.expect("C03 response body");
        let body = serde_json::from_str(&text).unwrap_or_else(|_| Value::String(text));
        assert_eq!(status, expected, "{label}: {body}");
        body
    }

    pub(super) async fn get(&self, label: &str, path: &str) -> Value {
        self.request(label, Method::GET, path, None, StatusCode::OK)
            .await
    }

    pub(super) async fn post(
        &self,
        label: &str,
        path: &str,
        body: Value,
        expected: StatusCode,
    ) -> Value {
        self.request(label, Method::POST, path, Some(body), expected)
            .await
    }
}

pub(super) async fn compile_legacy(api: &Api<'_>, source: &str) -> Value {
    post!(
        api,
        "legacy compiler ingress",
        "/api/uar/compiler/compile",
        json!({"content": source}),
        StatusCode::OK
    )
}

pub(super) async fn wait_for_run(api: &Api<'_>, run_id: &str) -> Value {
    for _ in 0..200 {
        let body = get!(api, "bound run status", &format!("/api/uar/runs/{run_id}"));
        if matches!(
            body["status"].as_str(),
            Some("done" | "error" | "cancelled")
        ) {
            return body;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("bound run did not reach a terminal state");
}

pub(super) async fn replay_run(api: &Api<'_>, run_id: &str) -> String {
    let mut response = api
        .client
        .get(format!(
            "{}/api/uar/runs/{run_id}/stream?last_event_id=0&stream_mode=agui_spec",
            api.base_url
        ))
        .bearer_auth(api.token)
        .header("x-uar-workspace-id", WORKSPACE)
        .header("Last-Event-ID", "0")
        .send()
        .await
        .expect("bound run replay request");
    assert!(
        response.status().is_success(),
        "bound run replay status: {}",
        response.status()
    );
    tokio::time::timeout(Duration::from_secs(30), async {
        let mut body = String::new();
        while let Some(chunk) = response.chunk().await.expect("bound run replay chunk") {
            body.push_str(&String::from_utf8_lossy(&chunk));
            if body.contains("event: RUN_FINISHED") {
                break;
            }
        }
        body
    })
    .await
    .expect("bound run replay emits a terminal frame")
}

pub(super) fn response_text(replay: &str) -> String {
    replay
        .lines()
        .filter_map(|line| line.strip_prefix("data: "))
        .filter_map(|data| serde_json::from_str::<Value>(data).ok())
        .filter(|event| event["type"] == "TEXT_MESSAGE_CONTENT")
        .filter_map(|event| event["delta"].as_str().map(str::to_owned))
        .collect()
}
