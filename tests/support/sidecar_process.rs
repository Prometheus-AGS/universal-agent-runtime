//! Process harness for the `uar-sidecar` and standalone `universal-agent-runtime`
//! binaries (sidecar-launch-security, tasks 1.5 and 2.x).
//!
//! Every launch runs in its own temporary workspace: a private `HOME`, working
//! directory, SurrealKV data directory and configuration file, with the parent
//! environment cleared. That keeps the operator's `~/.agents/skills`,
//! `~/.uar/config.yaml`, `mcp.json` and `RUST_LOG` out of the process under
//! test, and lets a test byte-search every file the process wrote.

#![allow(dead_code, reason = "each test target uses a different subset")]

#[path = "approval_events.rs"]
pub mod approval_events;
pub use approval_events::approval_ids;

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use tokio::process::Child;
use std::time::{Duration, Instant};

use serde_json::Value;
use tokio::io::{AsyncBufReadExt as _, AsyncReadExt as _, AsyncWriteExt as _};

tokio::task_local! {
    static CHECK_DIAGNOSTICS: std::sync::Arc<std::sync::Mutex<CheckDiagnostics>>;
}

struct CheckDiagnostics {
    name: &'static str,
    id: String,
    directory: PathBuf,
    outcome: &'static str,
    events: Vec<Value>,
    persistence_error: Option<std::io::ErrorKind>,
}

impl CheckDiagnostics {
    fn publish(&mut self) {
        let receipt = serde_json::json!({"version":1, "check":self.name,
            "receipt_id":self.id, "outcome":self.outcome, "events":self.events});
        let pending = self.directory.join(format!("{}.pending", self.id));
        let destination = self.directory.join(format!("{}.json", self.id));
        let saved = std::fs::create_dir_all(&self.directory)
            .and_then(|()| std::fs::write(&pending, receipt.to_string()))
            .and_then(|()| std::fs::rename(&pending, &destination));
        if let Err(error) = saved {
            self.persistence_error = Some(error.kind());
            eprintln!("sidecar diagnostic receipt persistence failed: {}", error.kind());
        }
        if self.outcome != "in_progress" {
            eprintln!("UAR_CHECK_RECEIPT:{receipt}");
        }
    }
}

fn diagnostic(event: Value) {
    let _ = CHECK_DIAGNOSTICS.try_with(|events| {
        let mut receipt = events.lock().unwrap_or_else(|error| error.into_inner());
        receipt.events.push(event);
        receipt.publish();
    });
}

fn sanitized_body(body: &[u8]) -> Value {
    const CODES: &[&str] = &["unauthorized", "forbidden", "unavailable", "conflict",
        "outcome_unknown", "precondition_failed", "invalid_request", "not_found",
        "sidecar_mode_global_mcp_disabled", "sidecar_mode_feature_disabled",
        "lifecycle_command_failed", "invalid_lifecycle_command"];
    fn retain(value: &Value) -> Value {
        match value {
            Value::String(code) if CODES.contains(&code.as_str()) => value.clone(),
            Value::Number(_) | Value::Bool(_) | Value::Null => value.clone(),
            Value::Object(fields) => Value::Object(fields.iter()
                .filter(|(key, _)| ["error", "code", "kind", "category", "status"].contains(&key.as_str()))
                .map(|(key, value)| (key.clone(), retain(value))).collect()),
            _ => Value::String("[redacted]".into()),
        }
    }
    serde_json::from_slice::<Value>(body).map(|value| retain(&value))
        .unwrap_or_else(|_| serde_json::json!({"format":"non_json", "content":"[redacted]"}))
}

fn record_response(status: u16, body: &[u8]) {
    diagnostic(serde_json::json!({"kind":"http_response", "status":status,
        "body_bytes":body.len(), "body":sanitized_body(body)}));
}

/// One retained receipt per check, including panic and independently observed
/// process cleanup. Raw request/response secrets and panic payloads are excluded.
pub async fn record_check(name: &'static str, body: impl std::future::Future<Output = ()>) {
    use futures::FutureExt as _;
    let directory = std::env::var_os("UAR_SIDECAR_DIAGNOSTICS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("uar-sidecar-check-receipts"));
    let mut receipt = CheckDiagnostics { name, id: uuid::Uuid::new_v4().to_string(),
        directory, outcome: "in_progress", events: Vec::new(), persistence_error: None };
    receipt.publish();
    let events = std::sync::Arc::new(std::sync::Mutex::new(receipt));
    let outcome = CHECK_DIAGNOSTICS.scope(events.clone(),
        std::panic::AssertUnwindSafe(body).catch_unwind()).await;
    let persistence_error = {
        let mut receipt = events.lock().unwrap_or_else(|error| error.into_inner());
        receipt.outcome = if outcome.is_ok() && receipt.persistence_error.is_none() { "passed" } else { "failed" };
        receipt.publish();
        receipt.persistence_error
    };
    if let Err(panic) = outcome { std::panic::resume_unwind(panic); }
    assert!(persistence_error.is_none(), "diagnostic receipt not retained: {persistence_error:?}");
}

/// Buffer ordinary JSON HTTP responses before assertions, retaining sanitized
/// evidence even when a status assertion panics before reading the body.
pub struct ObservedResponse {
    status: reqwest::StatusCode,
    body: Vec<u8>,
}

impl ObservedResponse {
    pub fn status(&self) -> reqwest::StatusCode { self.status }
    pub async fn text(self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body)
    }
    pub async fn json<T: serde::de::DeserializeOwned>(self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }
}

pub trait ObservedRequest {
    fn observed_send(self) -> impl std::future::Future<Output = Result<ObservedResponse, reqwest::Error>> + Send;
}

impl ObservedRequest for reqwest::RequestBuilder {
    async fn observed_send(self) -> Result<ObservedResponse, reqwest::Error> {
        diagnostic(serde_json::json!({"kind":"http_request", "completion":"pending"}));
        let response = self.send().await.map_err(|error| {
            diagnostic(serde_json::json!({"kind":"http_transport_failure", "category":
                if error.is_connect() { "connect" } else if error.is_timeout() { "observation_interrupted" } else { "request" }}));
            error
        })?;
        let status = response.status();
        let body = response.bytes().await?.to_vec();
        record_response(status.as_u16(), &body);
        Ok(ObservedResponse { status, body })
    }
}

/// Model the stub LLM serves. The runtime strips the provider prefix, so the
/// stub fingerprints the bare name.
pub const MODEL: &str = "openai/gpt-5.4-mini";
/// The model name as the stub LLM receives it.
pub const STUB_MODEL: &str = "gpt-5.4-mini";
/// JWT secret written into every generated configuration.
pub const JWT_SECRET: &str = "sidecar-contract-jwt-secret-not-for-production";

const READY_TIMEOUT: Duration = Duration::from_secs(180);

/// A fresh 256-bit launch token as 64 lowercase hexadecimal characters.
pub fn random_token() -> String {
    format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}

/// An ephemeral loopback port that was free at the time of the call.
pub fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("bind ephemeral port")
        .local_addr()
        .expect("ephemeral port address")
        .port()
}

/// Path of a binary built by this package.
pub fn sidecar_binary() -> &'static str {
    env!("CARGO_BIN_EXE_uar-sidecar")
}

/// Path of the standalone server binary.
pub fn standalone_binary() -> &'static str {
    env!("CARGO_BIN_EXE_universal-agent-runtime")
}

/// Path of the test-only environment probe.
pub fn probe_binary() -> &'static str {
    env!("CARGO_BIN_EXE_uar-env-probe")
}

/// Private directories and files for one or more launches.
pub struct Workspace {
    root: tempfile::TempDir,
}

impl Workspace {
    pub fn new() -> Self {
        let root = tempfile::Builder::new()
            .prefix("uar-sidecar-contract-")
            .tempdir()
            .expect("create workspace");
        for directory in ["home", "work", "data", "logs", "no-skills"] {
            std::fs::create_dir_all(root.path().join(directory)).expect("create workspace dir");
        }
        // Standalone and sidecar startup load this committed policy set from
        // their private work cwd; keep governance enabled in every fixture.
        let policies = root.path().join("work/policies");
        std::fs::create_dir_all(&policies).expect("create private policy directory");
        for (name, source) in [
            ("default.cedar", include_str!("../../policies/default.cedar")),
            ("tool-approval.cedar", include_str!("../../policies/tool-approval.cedar")),
            ("skill-mutation.cedar", include_str!("../../policies/skill-mutation.cedar")),
        ] {
            std::fs::write(policies.join(name), source).expect("copy committed governance policy");
        }
        Self { root }
    }

    pub fn path(&self) -> &Path {
        self.root.path()
    }

    pub fn home(&self) -> PathBuf {
        self.path().join("home")
    }

    pub fn work(&self) -> PathBuf {
        self.path().join("work")
    }

    pub fn data(&self) -> PathBuf {
        self.path().join("data")
    }

    pub fn logs(&self) -> PathBuf {
        self.path().join("logs")
    }

    /// Write a configuration file and return its path.
    pub fn write_config(&self, name: &str, yaml: &str) -> PathBuf {
        let path = self.path().join(name);
        std::fs::write(&path, yaml).expect("write config");
        path
    }
}

/// Knobs for the generated configuration file.
#[derive(Clone)]
pub struct ConfigOptions {
    pub llm_base_url: String,
    pub jwt_required: bool,
    pub port: u16,
    pub grpc_port: u16,
    pub extra_yaml: String,
}

impl ConfigOptions {
    pub fn new(llm_base_url: &str) -> Self {
        Self {
            llm_base_url: llm_base_url.to_owned(),
            jwt_required: false,
            port: free_port(),
            grpc_port: free_port(),
            extra_yaml: String::new(),
        }
    }
}

/// Render a complete configuration for `workspace`.
pub fn render_config(workspace: &Workspace, options: &ConfigOptions) -> String {
    format!(
        "skill_source_roots: []\nsecurity:\n  jwt_required: {jwt}\n  jwt_secret: \"{JWT_SECRET}\"\n  settings_mutation_auth_required: false\n\
         resilience:\n  rate_limit_enabled: false\n\
         persistence:\n  provider: \"surreal\"\n  database_url: \"surrealkv://{data}\"\n\
         acp:\n  enabled: true\n  path: \"/acp\"\n  auth_required: true\n\
         llm:\n  model: \"{MODEL}\"\n  base_url: \"{base}\"\n\
         server:\n  host: \"127.0.0.1\"\n  port: {port}\n  grpc_port: {grpc}\n  shutdown_timeout_secs: 10\n\
         native_tools:\n  terminal_exec_enabled: true\n  terminal_use_sandbox: false\n  terminal_timeout_secs: 30\n\
         {extra}",
        jwt = options.jwt_required,
        data = workspace.data().display(),
        base = options.llm_base_url,
        port = options.port,
        grpc = options.grpc_port,
        extra = options.extra_yaml,
    )
}

/// Everything needed to start one process.
#[derive(Clone)]
pub struct LaunchOptions {
    pub config_path: PathBuf,
    pub extra_args: Vec<String>,
    pub extra_env: Vec<(String, String)>,
    /// Bytes written to stdin immediately after spawn. `None` closes stdin.
    pub stdin_bytes: Option<Vec<u8>>,
    /// Keep stdin open after writing `stdin_bytes`.
    pub keep_stdin_open: bool,
    /// Tag used to name the stdout/stderr capture files.
    pub tag: String,
}

impl LaunchOptions {
    pub fn new(config_path: PathBuf, tag: &str) -> Self {
        Self {
            config_path,
            extra_args: Vec::new(),
            extra_env: Vec::new(),
            stdin_bytes: None,
            keep_stdin_open: true,
            tag: tag.to_owned(),
        }
    }

    /// Hand the sidecar its launch token as the first stdin line.
    pub fn with_token(mut self, token: &str) -> Self {
        self.stdin_bytes = Some(format!("{token}\n").into_bytes());
        self
    }

    pub fn env(mut self, key: &str, value: impl Into<String>) -> Self {
        self.extra_env.push((key.to_owned(), value.into()));
        self
    }
}

/// A running (or exited) child process with captured output.
pub struct ServerProcess {
    child: Child,
    pid: u32,
    exit_status: Option<ExitStatus>,
    ready: Option<tokio::sync::oneshot::Receiver<Result<u16, String>>>,
    output_reader: Option<tokio::task::JoinHandle<std::io::Result<()>>>,
    stdin: Option<Box<dyn tokio::io::AsyncWrite + Send + Sync + Unpin>>,
    lifecycle: tokio::sync::mpsc::UnboundedReceiver<Value>,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
    pub port: Option<u16>,
}

fn base_command(program: &str, workspace: &Workspace, options: &LaunchOptions) -> Command {
    let mut command = Command::new(program);
    command
        .arg("--config")
        .arg(&options.config_path)
        .args(&options.extra_args)
        .current_dir(workspace.work())
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .env("HOME", workspace.home())
        .env("UAR_BUILTIN_SKILLS_DIR", workspace.path().join("no-skills"));
    if let Some(tmp) = std::env::var_os("TMPDIR") {
        command.env("TMPDIR", tmp);
    }
    #[cfg(windows)]
    for key in [
        "SYSTEMROOT",
        "WINDIR",
        "COMSPEC",
        "TEMP",
        "TMP",
        "USERPROFILE",
    ] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    for (key, value) in &options.extra_env {
        command.env(key, value);
    }
    command
}

/// Start `program` in `workspace` with captured output.
pub async fn spawn(program: &str, workspace: &Workspace, options: &LaunchOptions) -> ServerProcess {
    let stdout_path = workspace.logs().join(format!("{}.stdout", options.tag));
    let stderr_path = workspace.logs().join(format!("{}.stderr", options.tag));
    let mut stdout_capture = std::fs::File::create(&stdout_path).expect("create stdout capture");
    let stderr = std::fs::File::create(&stderr_path).expect("create stderr capture");
    let mut command = base_command(program, workspace, options);
    command.stdin(Stdio::piped());
    #[cfg(windows)]
    let parent_pipe = if program == sidecar_binary() {
        // Mandatory Windows launch contract: the child reads this inherited
        // named-pipe handle with overlapped I/O and joins canceled reads.
        use std::os::windows::fs::OpenOptionsExt as _;
        use tokio::net::windows::named_pipe::ServerOptions;
        use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OVERLAPPED;
        let name = format!(r"\\.\pipe\uar-sidecar-control-{}", uuid::Uuid::new_v4());
        let parent = ServerOptions::new().first_pipe_instance(true).max_instances(1)
            .reject_remote_clients(true).access_inbound(false).create(&name)
            .expect("create private overlapped input");
        let child_input = std::fs::OpenOptions::new().read(true)
            .custom_flags(FILE_FLAG_OVERLAPPED).open(&name).expect("open child input handle");
        command.stdin(Stdio::from(child_input)).env("UAR_SIDECAR_STDIN_OVERLAPPED", "1");
        Some(parent)
    } else { None };
    let mut command = tokio::process::Command::from(command);
    let mut child = command.kill_on_drop(true)
        .stdout(Stdio::piped()).stderr(Stdio::from(stderr)).spawn()
        .unwrap_or_else(|error| panic!("spawn {program}: {error}"));
    drop(command);
    let mut stdin: Option<Box<dyn tokio::io::AsyncWrite + Send + Sync + Unpin>> = child.stdin.take()
        .map(|pipe| Box::new(pipe) as Box<dyn tokio::io::AsyncWrite + Send + Sync + Unpin>);
    #[cfg(windows)]
    if let Some(parent) = parent_pipe {
        parent.connect().await.expect("connect inherited child input");
        stdin = Some(Box::new(parent));
    }
    if let Some(bytes) = &options.stdin_bytes
        && let Some(pipe) = stdin.as_mut()
    {
        pipe.write_all(bytes).await.expect("write stdin bytes");
        pipe.flush().await.expect("flush stdin");
    }
    if options.stdin_bytes.is_none() || !options.keep_stdin_open {
        stdin = None;
    }
    let pid = child.id().expect("spawned child pid");
    let stdout = child.stdout.take().expect("child stdout pipe");
    let (ready_sender, ready) = tokio::sync::oneshot::channel();
    let (lifecycle_sender, lifecycle) = tokio::sync::mpsc::unbounded_channel();
    let output_reader = tokio::spawn(async move {
        let mut ready_sender = Some(ready_sender);
        let mut reader = tokio::io::BufReader::new(stdout);
        let mut line = Vec::new();
        loop {
            line.clear();
            if reader.read_until(b'\n', &mut line).await? == 0 { break; }
            std::io::Write::write_all(&mut stdout_capture, &line)?;
            std::io::Write::flush(&mut stdout_capture)?;
            if let Some(response) = std::str::from_utf8(&line).ok().and_then(|line| line.strip_prefix("LIFECYCLE:")) {
                if let Ok(response) = serde_json::from_str(response) { let _ = lifecycle_sender.send(response); }
            }
            if let Some(port) = std::str::from_utf8(&line).ok().and_then(|line| line.strip_prefix("READY:")) {
                if let Some(sender) = ready_sender.take() {
                    let _ = sender.send(port.trim().parse::<u16>().map_err(|_| "invalid READY port".to_owned()));
                }
            }
        }
        if let Some(sender) = ready_sender { let _ = sender.send(Err("stdout closed before READY".to_owned())); }
        Ok(())
    });
    ServerProcess {
        child,
        pid,
        exit_status: None,
        ready: Some(ready),
        output_reader: Some(output_reader),
        stdin,
        lifecycle,
        stdout_path,
        stderr_path,
        port: None,
    }
}

/// Start the sidecar and wait for its `READY:{port}` line.
pub async fn launch_sidecar(workspace: &Workspace, options: &LaunchOptions) -> ServerProcess {
    let mut process = spawn(sidecar_binary(), workspace, options).await;
    let port = match process.wait_for_ready(READY_TIMEOUT).await {
        Ok(port) => port,
        Err(error) => {
            process.close_stdin();
            let exit = process.wait_exit(READY_TIMEOUT).await;
            panic!("sidecar did not become ready: {error}; exit={exit:?}");
        }
    };
    process.port = Some(port);
    process
}

/// Start the standalone server on the configured port and wait for `/health`.
pub async fn launch_standalone(
    workspace: &Workspace,
    options: &LaunchOptions,
    port: u16,
) -> ServerProcess {
    let mut options = options.clone();
    options.extra_env.push(("UAR_TEST_READY_STDOUT".to_owned(), "1".to_owned()));
    let mut process = spawn(standalone_binary(), workspace, &options).await;
    let reported = match process.wait_for_ready(READY_TIMEOUT).await {
        Ok(port) => port,
        Err(error) => {
            let exit = process.wait_exit(READY_TIMEOUT).await;
            panic!("standalone did not become ready: {error}; exit={exit:?}");
        }
    };
    assert_eq!(reported, port, "standalone bound port");
    process.port = Some(reported);
    process
}

impl ServerProcess {
    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub fn port(&self) -> u16 {
        self.port.expect("process reported a port")
    }

    pub fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port())
    }

    pub fn stdout(&self) -> String {
        String::from_utf8_lossy(&std::fs::read(&self.stdout_path).unwrap_or_default()).into_owned()
    }

    pub fn stderr(&self) -> String {
        String::from_utf8_lossy(&std::fs::read(&self.stderr_path).unwrap_or_default()).into_owned()
    }

    /// Lines on stdout that are `READY:` signals.
    pub fn ready_lines(&self) -> Vec<String> {
        self.stdout()
            .lines()
            .filter(|line| line.starts_with("READY:"))
            .map(str::to_owned)
            .collect()
    }

    async fn wait_for_ready(&mut self, _timeout: Duration) -> Result<u16, String> {
        diagnostic(serde_json::json!({"kind":"process_readiness", "process_id":self.pid, "completion":"pending"}));
        let ready = self.ready.take().ok_or_else(|| "readiness already consumed".to_owned())?;
        let outcome = tokio::select! {
            result = ready => result.map_err(|_| "readiness reader stopped".to_owned())?,
            status = self.child.wait() => Err(format!("exited before READY: {status:?}\n{}", self.stderr())),
        };
        diagnostic(serde_json::json!({"kind":"process_readiness", "process_id":self.pid,
            "completion":if outcome.is_ok() { "ready" } else { "failed" }}));
        outcome
    }

    /// Issue one launch-authorized management command independently of HTTP/DB auth.
    pub async fn lifecycle(&mut self, operation: Value) -> Value {
        let identity = uuid::Uuid::new_v4().to_string();
        diagnostic(serde_json::json!({"kind":"lifecycle_command", "process_id":self.pid,
            "request_id":identity, "completion":"pending"}));
        let mut command = serde_json::to_vec(&serde_json::json!({
            "version": 1, "request_id": identity, "operation": operation,
        })).expect("serialize control request");
        command.push(b'\n');
        let input = self.stdin.as_mut().expect("launch channel is open");
        input.write_all(&command).await.expect("write lifecycle request");
        input.flush().await.expect("flush lifecycle request");
        loop {
            let response = self.lifecycle.recv().await.expect("child exited before lifecycle receipt");
            if response.get("request_id").and_then(Value::as_str) == Some(identity.as_str()) {
                assert_eq!(response["version"], 1, "lifecycle protocol version");
                diagnostic(serde_json::json!({"kind":"lifecycle_command", "process_id":self.pid,
                    "request_id":identity, "completion":"acknowledged",
                    "error":sanitized_body(&serde_json::to_vec(&serde_json::json!({"error":response.get("error")})).expect("serialize sanitized control category"))}));
                return response;
            }
        }
    }

    /// Close the stdin pipe (the sidecar's shutdown contract).
    pub fn close_stdin(&mut self) {
        self.stdin = None;
    }

    /// Await the owned child and stdout reader. The legacy duration argument
    /// does not establish completion; an outer verification interruption is separate.
    pub async fn wait_exit(&mut self, _timeout: Duration) -> Option<ExitStatus> {
        diagnostic(serde_json::json!({"kind":"process_cleanup", "process_id":self.pid,
            "completion":"pending"}));
        let status = self.child.wait().await.expect("wait for owned child");
        self.exit_status = Some(status);
        if let Some(reader) = self.output_reader.take() {
            reader.await.expect("join child stdout reader").expect("capture child stdout");
        }
        diagnostic(serde_json::json!({"kind":"process_cleanup", "process_id":self.pid,
            "completion":"exited_and_output_joined", "success":status.success(), "exit_code":status.code()}));
        Some(status)
    }

    /// Stop a sidecar through its stdin-EOF contract.
    pub async fn stop_sidecar(&mut self) -> ExitStatus {
        self.close_stdin();
        self.wait_exit(Duration::ZERO).await.expect("owned child exit")
    }

    /// Stop a standalone server with SIGTERM (Unix) and wait for exit.
    pub async fn stop_standalone(&mut self) -> ExitStatus {
        if let Some(status) = self.exit_status { return status; }
        #[cfg(unix)]
        {
            let status = Command::new("/bin/kill")
                .arg("-TERM")
                .arg(self.pid().to_string())
                .status()
                .expect("send SIGTERM");
            assert!(status.success(), "kill -TERM failed: {status}");
        }
        #[cfg(not(unix))]
        {
            let _ = self.child.start_kill();
        }
        self.wait_exit(Duration::ZERO).await.expect("owned child exit")
    }
}

impl Drop for ServerProcess {
    fn drop(&mut self) {
        if self.exit_status.is_none() {
            diagnostic(serde_json::json!({"kind":"process_cleanup", "process_id":self.pid,
                "completion":"cancellation_requested_unconfirmed"}));
        }
        self.stdin = None;
        // Emergency cancellation only. Explicit close paths await child.wait;
        // Tokio also owns platform reaping when a canceled fixture drops Child.
        let _ = self.child.start_kill();
        if let Some(reader) = self.output_reader.take() { reader.abort(); }
    }
}

/// Preserve the test body's original failure after awaited fixture finalization.
pub fn finish_fixture<T>(body: std::thread::Result<T>, cleanup: std::thread::Result<ExitStatus>) -> T {
    diagnostic(serde_json::json!({"kind":"fixture_finalization", "body_panicked":body.is_err(),
        "cleanup_panicked":cleanup.is_err(), "cleanup_success":cleanup.as_ref().ok().map(ExitStatus::success)}));
    match body {
        Err(panic) => {
            if cleanup.is_err() { eprintln!("fixture cleanup also failed; original test failure retained"); }
            std::panic::resume_unwind(panic)
        }
        Ok(value) => {
            let status = cleanup.unwrap_or_else(|panic| std::panic::resume_unwind(panic));
            assert!(status.success(), "fixture process shutdown failed: {status}");
            value
        }
    }
}

// ─── Raw HTTP/1.1 ───────────────────────────────────────────────────────────

/// A parsed HTTP/1.1 response read from a raw socket.
#[derive(Debug, Clone)]
pub struct RawResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl RawResponse {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    pub fn body_text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    /// Everything the client received, for byte searches.
    pub fn all_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for (key, value) in &self.headers {
            bytes.extend_from_slice(key.as_bytes());
            bytes.extend_from_slice(b": ");
            bytes.extend_from_slice(value.as_bytes());
            bytes.extend_from_slice(b"\n");
        }
        bytes.extend_from_slice(&self.body);
        bytes
    }
}

/// Send one request with exactly the given headers. The request line uses
/// `target`; nothing is added except `Connection: close` and, for methods
/// with a body, `Content-Length: 0`.
pub async fn raw_request(
    port: u16,
    method: &str,
    target: &str,
    headers: &[(&str, &str)],
) -> RawResponse {
    diagnostic(serde_json::json!({"kind":"http_request", "completion":"pending"}));
    let mut request = format!("{method} {target} HTTP/1.1\r\n");
    for (name, value) in headers {
        request.push_str(&format!("{name}: {value}\r\n"));
    }
    if matches!(method, "POST" | "PUT" | "PATCH" | "DELETE") {
        request.push_str("Content-Length: 0\r\n");
    }
    request.push_str("Connection: close\r\n\r\n");
    let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .expect("connect to server");
    stream
        .write_all(request.as_bytes())
        .await
        .expect("write request");
    let response = read_response(&mut stream).await;
    record_response(response.status, &response.body);
    response
}

async fn read_response(stream: &mut tokio::net::TcpStream) -> RawResponse {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 8192];
    loop {
        if let Some(parsed) = parse_complete(&buffer) { return parsed; }
        match stream.read(&mut chunk).await {
            Ok(0) => break,
            Ok(count) => buffer.extend_from_slice(&chunk[..count]),
            Err(error) => panic!("HTTP response transport failed before completion: {}", error.kind()),
        }
    }
    parse_partial(&buffer).unwrap_or_else(|| {
        panic!(
            "no HTTP response head received: {:?}",
            String::from_utf8_lossy(&buffer)
        )
    })
}

fn split_head(buffer: &[u8]) -> Option<(u16, Vec<(String, String)>, usize)> {
    let end = buffer.windows(4).position(|window| window == b"\r\n\r\n")?;
    let head = String::from_utf8_lossy(&buffer[..end]).into_owned();
    let mut lines = head.split("\r\n");
    let status = lines
        .next()?
        .split_whitespace()
        .nth(1)?
        .parse::<u16>()
        .ok()?;
    let headers = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
        .collect();
    Some((status, headers, end + 4))
}

/// A response is complete when its framing says so. An event stream is
/// returned as soon as its head arrives: callers only inspect its status and
/// headers, and it would otherwise stay open.
fn parse_complete(buffer: &[u8]) -> Option<RawResponse> {
    let (status, headers, body_start) = split_head(buffer)?;
    let response = |body: Vec<u8>| RawResponse {
        status,
        headers: headers.clone(),
        body,
    };
    let header = |name: &str| {
        headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.to_ascii_lowercase())
    };
    let body = &buffer[body_start..];
    if header("content-type").is_some_and(|value| value.starts_with("text/event-stream")) {
        return Some(response(dechunk(body)));
    }
    if let Some(length) = header("content-length").and_then(|value| value.parse::<usize>().ok()) {
        return (body.len() >= length).then(|| response(body[..length].to_vec()));
    }
    if header("transfer-encoding").is_some_and(|value| value.contains("chunked")) {
        return body
            .windows(5)
            .any(|window| window == b"0\r\n\r\n")
            .then(|| response(dechunk(body)));
    }
    None
}

fn parse_partial(buffer: &[u8]) -> Option<RawResponse> {
    let (status, headers, body_start) = split_head(buffer)?;
    let chunked = headers.iter().any(|(key, value)| {
        key.eq_ignore_ascii_case("transfer-encoding")
            && value.to_ascii_lowercase().contains("chunked")
    });
    let body = &buffer[body_start..];
    Some(RawResponse {
        status,
        headers,
        body: if chunked {
            dechunk(body)
        } else {
            body.to_vec()
        },
    })
}

fn dechunk(mut body: &[u8]) -> Vec<u8> {
    let mut decoded = Vec::new();
    loop {
        let Some(line_end) = body.windows(2).position(|window| window == b"\r\n") else {
            decoded.extend_from_slice(body);
            return decoded;
        };
        let size_text = String::from_utf8_lossy(&body[..line_end]);
        let Ok(size) = usize::from_str_radix(size_text.split(';').next().unwrap_or("").trim(), 16)
        else {
            decoded.extend_from_slice(body);
            return decoded;
        };
        if size == 0 {
            return decoded;
        }
        let start = line_end + 2;
        let end = (start + size).min(body.len());
        decoded.extend_from_slice(&body[start..end]);
        if end + 2 > body.len() {
            return decoded;
        }
        body = &body[end + 2..];
    }
}

// ─── Route inventory ────────────────────────────────────────────────────────

/// Every documented route plus the fixed list from task 2.6, with path
/// parameters filled. Methods come from the OpenAPI document the server
/// serves at `/api/openapi.json` when built with `api-docs`; the document is
/// read from the library so the list is identical in builds without that
/// feature.
pub fn route_inventory() -> Vec<(String, String)> {
    let spec =
        serde_json::to_value(universal_agent_runtime::uar::api::openapi::build_openapi_spec())
            .expect("serialize OpenAPI document");
    let mut routes = Vec::new();
    if let Some(paths) = spec.get("paths").and_then(Value::as_object) {
        for (path, item) in paths {
            let filled = fill_path_parameters(path);
            if let Some(methods) = item.as_object() {
                for method in methods.keys() {
                    let method = method.to_ascii_uppercase();
                    if matches!(method.as_str(), "GET" | "POST" | "PUT" | "DELETE" | "PATCH") {
                        routes.push((method, filled.clone()));
                    }
                }
            }
        }
    }
    let unknown = format!("/no-such-path-{}", uuid::Uuid::new_v4().simple());
    for (method, path) in [
        ("GET", "/"),
        ("GET", "/health"),
        ("GET", "/healthz"),
        ("GET", "/readyz"),
        ("GET", "/metrics"),
        ("GET", "/v1/models"),
        ("POST", "/v1/chat/completions"),
        ("POST", "/v1/messages"),
        ("POST", "/mcp/uar"),
        ("POST", "/mcp/memory"),
        ("GET", "/api/uar/runs/dummy-run/stream"),
        ("GET", "/api/uar/sync/stream"),
        ("POST", "/acp"),
        ("GET", "/api/uar/capabilities"),
        ("GET", "/api/openapi.json"),
        ("GET", unknown.as_str()),
    ] {
        routes.push((method.to_owned(), path.to_owned()));
    }
    routes.sort();
    routes.dedup();
    routes
}

fn fill_path_parameters(path: &str) -> String {
    let mut filled = String::with_capacity(path.len());
    let mut inside = false;
    for character in path.chars() {
        match character {
            '{' => {
                inside = true;
                filled.push_str("dummy");
            }
            '}' => inside = false,
            _ if inside => {}
            _ => filled.push(character),
        }
    }
    filled
}

// ─── Runs ───────────────────────────────────────────────────────────────────

/// A test agent: the default agent under a non-orchestrator id, so a run
/// makes exactly the model calls the stub fixtures describe.
pub fn test_agent(run_policy: Option<Value>) -> Value {
    let mut artifact =
        serde_json::to_value(universal_agent_runtime::uar::defaults::default_agent())
            .expect("serialize default agent");
    artifact["id"] = Value::String("sidecar-contract-agent".to_owned());
    if let Some(policy) = run_policy {
        artifact["extensions"]["uar.run_policy"] = policy;
    }
    artifact
}

/// What a driven run produced.
#[derive(Debug)]
pub struct RunTranscript {
    pub run_id: String,
    pub create_body: String,
    pub stream: String,
    pub approvals: usize,
}

/// Create a run, follow its stream, approve every approval request, and
/// return the stream text once the run ends (or `timeout` elapses).
pub async fn drive_run(
    base_url: &str,
    bearer: Option<&str>,
    artifact: Value,
    input: &str,
    timeout: Duration,
) -> RunTranscript {
    let client = reqwest::Client::new();
    let with_auth = |builder: reqwest::RequestBuilder| match bearer {
        Some(token) => builder.bearer_auth(token),
        None => builder,
    };
    let created = with_auth(client.post(format!("{base_url}/api/uar/runs")))
        .json(&serde_json::json!({ "artifact": artifact, "input": input }))
        .send()
        .await
        .expect("create run");
    let status = created.status();
    let create_body = created.text().await.expect("create run body");
    record_response(status.as_u16(), create_body.as_bytes());
    if !status.is_success() {
        let safe = bearer.map_or_else(|| create_body.clone(), |token| create_body.replace(token, "[redacted]"));
        panic!("create run: {status} {safe}");
    }
    let created: Value = serde_json::from_str(&create_body).expect("create run JSON");
    let run_id = created["run_id"].as_str().expect("run id").to_owned();
    let mut response = with_auth(client.get(format!("{base_url}/api/uar/runs/{run_id}/stream")))
        .send()
        .await
        .expect("open run stream");
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.expect("run stream error body");
        record_response(status.as_u16(), body.as_bytes());
        let body = bearer.map_or_else(|| body.clone(), |token| body.replace(token, "[redacted]"));
        panic!("run stream: {status} {body}");
    }
    let mut stream = String::new();
    let mut approvals = 0;
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        assert!(!remaining.is_zero(), "verification interrupted before run completion: {run_id}");
        let chunk = match tokio::time::timeout(remaining, response.chunk()).await {
            Ok(Ok(Some(chunk))) => chunk,
            Ok(Ok(None)) => panic!("stream ended without terminal event: {run_id} {stream}"),
            Ok(Err(_)) => panic!("run stream transport failed: {run_id} {stream}"),
            Err(_) => panic!("verification interrupted before run completion: {run_id} {stream}"),
        };
        stream.push_str(&String::from_utf8_lossy(&chunk));
        let requested = approval_ids(&stream, &run_id);
        while approvals < requested.len() {
            let decision =
                with_auth(client.post(format!("{base_url}/api/uar/runs/{run_id}/tool-approval")))
                    .json(&serde_json::json!({ "approved": true, "approval_id": requested[approvals] }))
                    .send()
                    .await
                    .expect("approve tool call");
            if !decision.status().is_success() {
                let status = decision.status();
                let body = decision.text().await.expect("approval error body");
                record_response(status.as_u16(), body.as_bytes());
                let body = bearer.map_or_else(|| body.clone(), |token| body.replace(token, "[redacted]"));
                panic!("approval: {status} {body}");
            }
            approvals += 1;
        }
        if [
            "event: agui.done",
            "event: agui.error",
            "event: agui.cancelled",
        ]
        .iter()
        .any(|marker| stream.contains(marker))
        {
            break;
        }
    }
    RunTranscript {
        run_id,
        create_body,
        stream,
        approvals,
    }
}

// ─── Byte search ────────────────────────────────────────────────────────────

/// Every regular file under `root`, recursively.
pub fn files_under(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            match entry.file_type() {
                Ok(kind) if kind.is_dir() => pending.push(path),
                Ok(kind) if kind.is_file() => files.push(path),
                _ => {}
            }
        }
    }
    files
}

pub fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

/// Files under `root` whose bytes contain `needle`.
pub fn files_containing(root: &Path, needle: &[u8]) -> Vec<PathBuf> {
    files_under(root)
        .into_iter()
        .filter(|path| std::fs::read(path).is_ok_and(|bytes| contains_bytes(&bytes, needle)))
        .collect()
}

/// Parse the JSON lines a probe appended to `path`.
pub fn probe_records(path: &Path) -> Vec<Value> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("probe record JSON"))
        .collect()
}
