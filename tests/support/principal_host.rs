//! A host client for the token-authenticated sidecar that can assert a
//! per-request principal (`X-UAR-Principal`), for the OpenSpec change
//! `sidecar-session-principal`. Builds on the process harness in
//! `sidecar_process.rs`: every host is a real `uar-sidecar` process.

#![allow(dead_code, reason = "each test target uses a different subset")]

use std::time::Duration;

use reqwest::Method;
use serde_json::{Value, json};

use crate::sidecar_process::{
    ConfigOptions, LaunchOptions, STUB_MODEL, ServerProcess, Workspace, launch_sidecar,
    launch_standalone, random_token, render_config,
};
use crate::stub_llm::{
    FixtureResponse, FixtureSet, RequestFingerprint, StubLlmServer, start_stub_llm,
};

/// The request header the host uses to name the session principal.
pub const PRINCIPAL_HEADER: &str = "X-UAR-Principal";
/// First the-boss session principal.
pub const P1: &str = "boss-session-1";
/// Second the-boss session principal.
pub const P2: &str = "boss-session-2";

const TERMINAL_MARKERS: [&str; 3] = [
    "event: agui.done",
    "event: agui.error",
    "event: agui.cancelled",
];
/// Marker of a pending tool approval on the runs stream.
pub const APPROVAL_MARKER: &str = "event: agui.tool_call.approval_required";

fn fingerprint(input: &str, has_tool_result: bool) -> RequestFingerprint {
    RequestFingerprint {
        model: STUB_MODEL.to_owned(),
        last_user_message: input.to_owned(),
        has_tools: true,
        has_tool_result,
    }
}

/// Add a one-step conversation for `input` that answers `reply`.
pub fn reply(set: FixtureSet, input: &str, reply: &str) -> FixtureSet {
    set.with(
        fingerprint(input, false),
        FixtureResponse::Content(reply.to_owned()),
    )
}

/// Add a two-step conversation for `input`: the model calls `tool` with
/// `arguments`, then answers `done` after the tool result.
pub fn tool_call(set: FixtureSet, input: &str, tool: &str, arguments: &Value) -> FixtureSet {
    set.with(
        fingerprint(input, false),
        FixtureResponse::ToolCall {
            name: tool.to_owned(),
            arguments: arguments.to_string(),
        },
    )
    .with(
        fingerprint(input, true),
        FixtureResponse::Content("done".to_owned()),
    )
}

/// Add a two-step conversation for `input` that calls `terminal_exec`.
pub fn terminal_call(set: FixtureSet, input: &str, command: &str) -> FixtureSet {
    tool_call(set, input, "terminal_exec", &json!({ "command": command }))
}

/// A run policy that asks the host before every tool call.
pub fn ask_policy() -> Value {
    json!({ "tool_approval": "ask" })
}

/// A running sidecar plus the stub LLM and workspace it depends on.
pub struct Host {
    pub process: ServerProcess,
    pub token: String,
    pub workspace: Workspace,
    pub stub: StubLlmServer,
    http: reqwest::Client,
}

/// Boot a sidecar with a fresh launch token. `extra_yaml` is appended to the
/// generated configuration; `customize` may add environment variables.
pub async fn boot(
    fixtures: FixtureSet,
    extra_yaml: &str,
    customize: impl FnOnce(LaunchOptions) -> LaunchOptions,
) -> Host {
    let stub = start_stub_llm(fixtures).await;
    let workspace = Workspace::new();
    let mut options = ConfigOptions::new(&stub.base_url);
    options.extra_yaml = extra_yaml.to_owned();
    let config = workspace.write_config("config.yaml", &render_config(&workspace, &options));
    let token = random_token();
    let launch = customize(LaunchOptions::new(config, "sidecar").with_token(&token));
    let process = launch_sidecar(&workspace, &launch).await;
    Host {
        process,
        token,
        workspace,
        stub,
        http: reqwest::Client::new(),
    }
}

/// Boot the standalone server (JWT off). Its requests carry an unverifiable
/// bearer, which a JWT-off server treats as anonymous, and no principal.
pub async fn boot_standalone(
    fixtures: FixtureSet,
    extra_yaml: &str,
    customize: impl FnOnce(LaunchOptions) -> LaunchOptions,
) -> Host {
    let stub = start_stub_llm(fixtures).await;
    let workspace = Workspace::new();
    let mut options = ConfigOptions::new(&stub.base_url);
    options.extra_yaml = extra_yaml.to_owned();
    let port = options.port;
    let config = workspace.write_config("config.yaml", &render_config(&workspace, &options));
    let launch = customize(LaunchOptions::new(config, "standalone"));
    let process = launch_standalone(&workspace, &launch, port).await;
    Host {
        process,
        token: random_token(),
        workspace,
        stub,
        http: reqwest::Client::new(),
    }
}

/// The value of `field` on the first JSON log line whose message is
/// `message`, from a process that logs in JSON.
pub fn logged_field(output: &str, message: &str, field: &str) -> Option<Value> {
    output
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|record| record["fields"]["message"] == message)
        .map(|record| record["fields"][field].clone())
}

/// Everything a finished (or timed-out) stream read produced.
#[derive(Debug)]
pub struct Driven {
    pub run_id: String,
    pub stream: String,
}

impl Host {
    pub fn base_url(&self) -> String {
        self.process.base_url()
    }

    pub fn port(&self) -> u16 {
        self.process.port()
    }

    /// A request carrying the launch token and, when given, a principal.
    pub fn request(
        &self,
        method: Method,
        path: &str,
        principal: Option<&str>,
    ) -> reqwest::RequestBuilder {
        let builder = self
            .http
            .request(method, format!("{}{path}", self.base_url()))
            .bearer_auth(&self.token);
        match principal {
            Some(value) => builder.header(PRINCIPAL_HEADER, value),
            None => builder,
        }
    }

    /// Send a request and return its status and body text.
    pub async fn call(
        &self,
        method: Method,
        path: &str,
        principal: Option<&str>,
        body: Option<Value>,
    ) -> (u16, String) {
        let mut builder = self.request(method, path, principal);
        if let Some(body) = body {
            builder = builder.json(&body);
        }
        let response = builder.send().await.expect("send request");
        let status = response.status().as_u16();
        (status, response.text().await.unwrap_or_default())
    }

    /// Create a run and return its id.
    pub async fn start_run(
        &self,
        principal: Option<&str>,
        artifact: Value,
        input: &str,
        session_id: Option<&str>,
    ) -> String {
        let mut body = json!({ "artifact": artifact, "input": input });
        if let Some(session) = session_id {
            body["session_id"] = Value::String(session.to_owned());
        }
        let (status, text) = self
            .call(Method::POST, "/api/uar/runs", principal, Some(body))
            .await;
        assert_eq!(status, 200, "create run: {text}");
        let created: Value = serde_json::from_str(&text).expect("create run JSON");
        created["run_id"].as_str().expect("run id").to_owned()
    }

    /// Open a run's stream, or return the status when it is not 2xx.
    pub async fn open_stream(
        &self,
        principal: Option<&str>,
        run_id: &str,
    ) -> Result<reqwest::Response, u16> {
        let response = self
            .request(
                Method::GET,
                &format!("/api/uar/runs/{run_id}/stream"),
                principal,
            )
            .send()
            .await
            .expect("open run stream");
        if response.status().is_success() {
            Ok(response)
        } else {
            Err(response.status().as_u16())
        }
    }

    /// Read `response` into `text` until any of `markers` appears, the
    /// stream ends, or `timeout` elapses. Returns whether a marker appeared.
    pub async fn read_until(
        response: &mut reqwest::Response,
        text: &mut String,
        markers: &[&str],
        timeout: Duration,
    ) -> bool {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if markers.iter().any(|marker| text.contains(marker)) {
                return true;
            }
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                return false;
            }
            match tokio::time::timeout(remaining, response.chunk()).await {
                Ok(Ok(Some(chunk))) => text.push_str(&String::from_utf8_lossy(&chunk)),
                _ => return markers.iter().any(|marker| text.contains(marker)),
            }
        }
    }

    /// Approve the pending tool call of `run_id` as `principal`.
    pub async fn approve(&self, principal: Option<&str>, run_id: &str) -> u16 {
        self.call(
            Method::POST,
            &format!("/api/uar/runs/{run_id}/tool-approval"),
            principal,
            Some(json!({ "approved": true })),
        )
        .await
        .0
    }

    /// Follow an existing run to its end, approving every approval request
    /// as the same principal.
    pub async fn follow(&self, principal: Option<&str>, run_id: &str, timeout: Duration) -> String {
        let mut response = self
            .open_stream(principal, run_id)
            .await
            .unwrap_or_else(|status| panic!("run stream: {status}"));
        let mut text = String::new();
        let mut approvals = 0;
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if text.matches(APPROVAL_MARKER).count() > approvals {
                assert_eq!(self.approve(principal, run_id).await, 200, "approval");
                approvals += 1;
                continue;
            }
            if TERMINAL_MARKERS.iter().any(|marker| text.contains(marker)) {
                break;
            }
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                break;
            }
            match tokio::time::timeout(remaining, response.chunk()).await {
                Ok(Ok(Some(chunk))) => text.push_str(&String::from_utf8_lossy(&chunk)),
                _ => break,
            }
        }
        text
    }

    /// Create a run and follow it to its end.
    pub async fn drive(
        &self,
        principal: Option<&str>,
        artifact: Value,
        input: &str,
        session_id: Option<&str>,
    ) -> Driven {
        let run_id = self.start_run(principal, artifact, input, session_id).await;
        let stream = self.follow(principal, &run_id, Duration::from_secs(90)).await;
        assert!(
            TERMINAL_MARKERS.iter().any(|marker| stream.contains(marker)),
            "run {run_id} did not end\n{stream}"
        );
        Driven { run_id, stream }
    }

    /// Whether `run_id` is still addressable by `principal`. Uses the
    /// checkpoint listing, which neither subscribes nor changes the run.
    pub async fn run_exists(&self, principal: Option<&str>, run_id: &str) -> bool {
        let (status, body) = self
            .call(
                Method::GET,
                &format!("/api/uar/runs/{run_id}/checkpoints"),
                principal,
                None,
            )
            .await;
        match status {
            200 => true,
            404 => false,
            other => panic!("checkpoint listing for {run_id}: {other} {body}"),
        }
    }

    /// Every chat-completion request body the stub LLM received.
    pub async fn stub_requests(&self) -> Vec<Value> {
        let root = self.stub.base_url.trim_end_matches("/v1");
        let body: Value = self
            .http
            .get(format!("{root}/_stub/requests"))
            .send()
            .await
            .expect("stub request log")
            .json()
            .await
            .expect("stub request log JSON");
        body["requests"].as_array().cloned().unwrap_or_default()
    }

    /// Serialized model requests whose last user message is `input`.
    pub async fn model_requests_for(&self, input: &str) -> Vec<String> {
        self.stub_requests()
            .await
            .into_iter()
            .filter(|request| last_user_message(request) == input)
            .map(|request| request.to_string())
            .collect()
    }

    /// Current value of an unlabelled Prometheus gauge from `/metrics`.
    pub async fn gauge(&self, name: &str) -> Option<f64> {
        let (status, text) = self.call(Method::GET, "/metrics", None, None).await;
        assert_eq!(status, 200, "metrics: {text}");
        text.lines()
            .filter(|line| !line.starts_with('#'))
            .find_map(|line| {
                let (metric, value) = line.split_once(' ')?;
                (metric == name).then(|| value.trim().parse::<f64>().ok())?
            })
    }

    /// Poll `probe` every 250 ms until it holds or `timeout` elapses.
    pub async fn wait_for<F, Fut>(timeout: Duration, mut probe: F) -> bool
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if probe().await {
                return true;
            }
            if tokio::time::Instant::now() >= deadline {
                return false;
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    }
}

/// The text of the last user message in a chat-completion request body.
pub fn last_user_message(request: &Value) -> String {
    request["messages"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|message| message["role"] == "user")
        .next_back()
        .and_then(|message| message["content"].as_str())
        .unwrap_or_default()
        .to_owned()
}
