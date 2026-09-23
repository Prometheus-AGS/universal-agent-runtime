//! Process harness for the `uar-sidecar` and standalone `universal-agent-runtime`
//! binaries (sidecar-launch-security, tasks 1.5 and 2.x).
//!
//! Every launch runs in its own temporary workspace: a private `HOME`, working
//! directory, SurrealKV data directory and configuration file, with the parent
//! environment cleared. That keeps the operator's `~/.agents/skills`,
//! `~/.uar/config.yaml`, `mcp.json` and `RUST_LOG` out of the process under
//! test, and lets a test byte-search every file the process wrote.

#![allow(dead_code, reason = "each test target uses a different subset")]

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

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
        "security:\n  jwt_required: {jwt}\n  jwt_secret: \"{JWT_SECRET}\"\n  settings_mutation_auth_required: false\n\
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
    stdin: Option<ChildStdin>,
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
pub fn spawn(program: &str, workspace: &Workspace, options: &LaunchOptions) -> ServerProcess {
    let stdout_path = workspace.logs().join(format!("{}.stdout", options.tag));
    let stderr_path = workspace.logs().join(format!("{}.stderr", options.tag));
    let stdout = std::fs::File::create(&stdout_path).expect("create stdout capture");
    let stderr = std::fs::File::create(&stderr_path).expect("create stderr capture");
    let mut child = base_command(program, workspace, options)
        .stdin(Stdio::piped())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .unwrap_or_else(|error| panic!("spawn {program}: {error}"));
    let mut stdin = child.stdin.take();
    if let Some(bytes) = &options.stdin_bytes
        && let Some(pipe) = stdin.as_mut()
    {
        pipe.write_all(bytes).expect("write stdin bytes");
        pipe.flush().expect("flush stdin");
    }
    if options.stdin_bytes.is_none() || !options.keep_stdin_open {
        stdin = None;
    }
    ServerProcess {
        child,
        stdin,
        stdout_path,
        stderr_path,
        port: None,
    }
}

/// Start the sidecar and wait for its `READY:{port}` line.
pub async fn launch_sidecar(workspace: &Workspace, options: &LaunchOptions) -> ServerProcess {
    let mut process = spawn(sidecar_binary(), workspace, options);
    let port = process
        .wait_for_ready(READY_TIMEOUT)
        .await
        .unwrap_or_else(|error| panic!("sidecar did not become ready: {error}"));
    process.port = Some(port);
    process
}

/// Start the standalone server on the configured port and wait for `/health`.
pub async fn launch_standalone(
    workspace: &Workspace,
    options: &LaunchOptions,
    port: u16,
) -> ServerProcess {
    let mut process = spawn(standalone_binary(), workspace, options);
    let client = reqwest::Client::new();
    let deadline = Instant::now() + READY_TIMEOUT;
    loop {
        if let Ok(response) = client
            .get(format!("http://127.0.0.1:{port}/health"))
            .send()
            .await
            && response.status().is_success()
        {
            process.port = Some(port);
            return process;
        }
        if let Some(status) = process.child.try_wait().expect("poll standalone") {
            panic!(
                "standalone server exited before readiness: {status}\n{}",
                process.stderr()
            );
        }
        assert!(
            Instant::now() < deadline,
            "standalone server not healthy in time\n{}",
            process.stderr()
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

impl ServerProcess {
    pub fn pid(&self) -> u32 {
        self.child.id()
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

    async fn wait_for_ready(&mut self, timeout: Duration) -> Result<u16, String> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(line) = self.ready_lines().first() {
                return line
                    .trim_start_matches("READY:")
                    .trim()
                    .parse::<u16>()
                    .map_err(|error| format!("bad READY line {line:?}: {error}"));
            }
            if let Some(status) = self.child.try_wait().map_err(|error| error.to_string())? {
                return Err(format!("exited with {status}\n{}", self.stderr()));
            }
            if Instant::now() >= deadline {
                return Err(format!("no READY within {timeout:?}\n{}", self.stderr()));
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    /// Close the stdin pipe (the sidecar's shutdown contract).
    pub fn close_stdin(&mut self) {
        self.stdin = None;
    }

    /// Wait up to `timeout` for exit.
    pub async fn wait_exit(&mut self, timeout: Duration) -> Option<ExitStatus> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(status) = self.child.try_wait().expect("poll child") {
                return Some(status);
            }
            if Instant::now() >= deadline {
                return None;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    /// Stop a sidecar through its stdin-EOF contract.
    pub async fn stop_sidecar(&mut self) -> ExitStatus {
        self.close_stdin();
        match self.wait_exit(Duration::from_secs(30)).await {
            Some(status) => status,
            None => {
                let _ = self.child.kill();
                panic!("sidecar did not exit after stdin EOF\n{}", self.stderr());
            }
        }
    }

    /// Stop a standalone server with SIGTERM (Unix) and wait for exit.
    pub async fn stop_standalone(&mut self) -> ExitStatus {
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
            let _ = self.child.kill();
        }
        match self.wait_exit(Duration::from_secs(30)).await {
            Some(status) => status,
            None => {
                let _ = self.child.kill();
                panic!("standalone server did not exit\n{}", self.stderr());
            }
        }
    }
}

impl Drop for ServerProcess {
    fn drop(&mut self) {
        self.stdin = None;
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
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
    read_response(&mut stream).await
}

async fn read_response(stream: &mut tokio::net::TcpStream) -> RawResponse {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 8192];
    loop {
        if let Some(parsed) = parse_complete(&buffer) {
            return parsed;
        }
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        match tokio::time::timeout(remaining, stream.read(&mut chunk)).await {
            Ok(Ok(0)) | Err(_) => break,
            Ok(Ok(count)) => buffer.extend_from_slice(&chunk[..count]),
            Ok(Err(_)) => break,
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
    assert!(status.is_success(), "create run: {status} {create_body}");
    let created: Value = serde_json::from_str(&create_body).expect("create run JSON");
    let run_id = created["run_id"].as_str().expect("run id").to_owned();
    let mut response = with_auth(client.get(format!("{base_url}/api/uar/runs/{run_id}/stream")))
        .send()
        .await
        .expect("open run stream");
    assert!(
        response.status().is_success(),
        "run stream: {}",
        response.status()
    );
    let mut stream = String::new();
    let mut approvals = 0;
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        let Ok(Ok(Some(chunk))) = tokio::time::timeout(remaining, response.chunk()).await else {
            break;
        };
        stream.push_str(&String::from_utf8_lossy(&chunk));
        let requested = stream
            .matches("event: agui.tool_call.approval_required")
            .count();
        while approvals < requested {
            let decision =
                with_auth(client.post(format!("{base_url}/api/uar/runs/{run_id}/tool-approval")))
                    .json(&serde_json::json!({ "approved": true }))
                    .send()
                    .await
                    .expect("approve tool call");
            assert!(
                decision.status().is_success(),
                "approval: {}",
                decision.status()
            );
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
