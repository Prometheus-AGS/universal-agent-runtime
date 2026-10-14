//! Full binary qualification; no crate-private adapter or same-process manager proof.

use axum::{Json, Router, extract::State, response::{IntoResponse, Response}, routing::post};
use reqwest::Method;
use serde_json::{Value, json};
use std::{collections::BTreeSet, io::Write, path::{Path, PathBuf}, process::{Child, Command, Stdio},
    sync::{Arc, Mutex}, time::Duration};
use universal_agent_runtime::uar::{defaults::default_agent, persistence::{PersistenceLayer,
    checkpoint_resume::CheckpointResumeEnvelope, providers::surreal::SurrealDbProvider}};

pub(super) const OWNER: &str = "dot06-owner";
pub(super) const TENANT: &str = "dot06-tenant";
pub(super) const ISSUER: &str = "dot06-fixture-issuer";
const JWT_SECRET: &str = "dot06-signed-jwt-fixture";
const PROVIDER_SECRET: &str = "dot06-synthetic-provider-key";
const ADMIN: &str = "dot06-private-admin-key";
const WAIT: Duration = Duration::from_secs(90);

pub(super) fn private_dir(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::create_dir_all(path).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
}

pub(super) fn private_write(path: &Path, bytes: &[u8]) {
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = std::fs::OpenOptions::new().write(true).create(true).truncate(true)
        .mode(0o600).open(path).unwrap();
    file.write_all(bytes).unwrap();
}

pub(super) fn private_file(path: &Path) -> std::fs::File {
    private_write(path, b"");
    std::fs::OpenOptions::new().append(true).open(path).unwrap()
}

pub(super) fn token(subject: &str, tenant: &str, issuer: &str) -> String {
    jsonwebtoken::encode(&jsonwebtoken::Header::default(),
        &json!({"sub":subject,"tenant_id":tenant,"iss":issuer,"aud":"dot06-runtime",
            "roles":["user"],"exp":chrono::Utc::now().timestamp()+3600}),
        &jsonwebtoken::EncodingKey::from_secret(JWT_SECRET.as_bytes())).unwrap()
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct Database {
    provider: String,
    url: String,
    namespace: String,
    name: String,
    user: Option<String>,
    password: Option<String>,
    pub(super) remote: bool,
}

impl Database {
    pub(super) fn new(root: &Path, profile: &str, case: &str) -> Self {
        if profile == "local" {
            Self { provider: "surreal".into(), url: format!("surrealkv://{}", root.join("database").display()),
                namespace: "dot06".into(), name: "resume".into(), user: None, password: None, remote: false }
        } else if profile == "memory" {
            Self { provider: "surreal".into(), url: "memory".into(), namespace: "dot06".into(),
                name: "unsupported".into(), user: None, password: None, remote: false }
        } else {
            let path = std::env::var("UAR_DOT06_INFRA_FILE").expect("private disposable infrastructure manifest");
            let manifest: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
            if profile == "postgres" {
                Self { provider: "postgres".into(), url: manifest["postgres"]["url"].as_str().unwrap().into(),
                    namespace: String::new(), name: String::new(), user: None, password: None, remote: false }
            } else {
                let config = &manifest["surreal"];
                Self { provider: "surreal".into(), url: config["url"].as_str().unwrap().into(),
                    namespace: std::env::var("UAR_DOT06_NAMESPACE").expect("isolated namespace"),
                    name: format!("resume_{}", case.replace('-', "_")),
                    user: config["username"].as_str().map(str::to_owned),
                    password: config["password"].as_str().map(str::to_owned), remote: true }
            }
        }
    }

    pub(super) async fn connect(&self) -> SurrealDbProvider {
        SurrealDbProvider::new(&self.url, self.user.as_deref(), self.password.as_deref(),
            Some(&self.namespace), Some(&self.name)).await.expect("qualified durable readback")
    }
}

#[derive(Default)]
pub(super) struct Seen { pub(super) requests: Mutex<Vec<Value>>, pub(super) log: PathBuf }

pub(super) struct Peer { pub(super) url: String, pub(super) seen: Arc<Seen>, pub(super) task: tokio::task::JoinHandle<()> }

impl Peer {
    pub(super) async fn start(root: &Path) -> Self {
        let log = root.join("provider-requests.jsonl");
        private_write(&log, b"");
        let seen = Arc::new(Seen { log, ..Seen::default() });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/v1", listener.local_addr().unwrap());
        let app = Router::new().route("/v1/chat/completions", post(completion)).with_state(seen.clone());
        let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap(); });
        Self { url, seen, task }
    }
    pub(super) fn count(&self) -> usize { self.seen.requests.lock().unwrap().len() }
    fn saw(&self, mode: &str) -> bool {
        self.seen.requests.lock().unwrap().iter().any(|request|
            request["messages"].as_array().is_some_and(|messages|
                messages.iter().any(|message| message["role"] == "user" && message["content"] == mode)))
    }
}

impl Drop for Peer { fn drop(&mut self) { self.task.abort(); } }

async fn completion(State(seen): State<Arc<Seen>>, Json(body): Json<Value>) -> Response {
    {
        seen.requests.lock().unwrap().push(body.clone());
        let mut log = std::fs::OpenOptions::new().append(true).open(&seen.log).unwrap();
        serde_json::to_writer(&mut log, &body).unwrap(); log.write_all(b"\n").unwrap(); log.sync_all().unwrap();
    }
    let messages = body["messages"].as_array().unwrap();
    let index = messages.iter().rposition(|message| message["role"] == "user").unwrap();
    let mode = messages[index]["content"].as_str().expect("controlled user input");
    if mode == "kill_before" { std::future::pending::<()>().await; }
    let chunk = |delta: Value, finish: Value| format!("data: {}\n\n", json!({
        "id":"dot06-response","object":"chat.completion.chunk","created":1,"model":"gpt-4o-mini",
        "choices":[{"index":0,"delta":delta,"finish_reason":finish}]}));
    let mut frames = String::new();
    if mode == "Reply with OK." || messages[index + 1..].iter().any(|message| message["role"] == "tool") {
        frames.push_str(&chunk(json!({"role":"assistant","content":"resume-complete"}), Value::Null));
        frames.push_str(&chunk(json!({}), json!("stop")));
    } else {
        let tool = body["tools"].as_array().unwrap().iter().find_map(|tool|
            tool["function"]["name"].as_str().filter(|name| name.ends_with("__effect"))).unwrap();
        let arguments = json!({"mode":mode}).to_string();
        frames.push_str(&chunk(json!({"role":"assistant","tool_calls":[{"index":0,
            "id":format!("dot06-{mode}-{index}"),"type":"function","function":{"name":tool,"arguments":arguments}}]}), Value::Null));
        frames.push_str(&chunk(json!({}), json!("tool_calls")));
    }
    frames.push_str("data: [DONE]\n\n");
    ([("content-type", "text/event-stream")], frames).into_response()
}

fn unused_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

fn trace(path: &Path) -> Vec<Value> {
    std::fs::read_to_string(path).unwrap_or_default().lines()
        .map(|line| serde_json::from_str(line).unwrap()).collect()
}

pub(super) fn effects(path: &Path) -> usize { trace(path).iter().filter(|row| row["event"] == "effect").count() }

pub(super) struct Node { pub(super) child: Child, pub(super) base: String, pub(super) directory: PathBuf, trace: PathBuf, client: reqwest::Client }

impl Node {
    fn launch(root: &Path, label: &str, database: &Database, peer: &Peer, effect_trace: &Path, attested: bool) -> Self {
        let directory = root.join(label); private_dir(&directory);
        for suffix in ["home", "work", "work/policies", "no-builtins"] { private_dir(&directory.join(suffix)); }
        for name in ["default.cedar", "tool-approval.cedar", "skill-mutation.cedar"] {
            private_write(&directory.join("work/policies").join(name), &std::fs::read(
                Path::new(env!("CARGO_MANIFEST_DIR")).join("policies").join(name)).unwrap());
        }
        let mcp = json!({"mcpServers":{"resume":{"command":"/usr/bin/python3",
            "args":[Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fresh_process_resume_mcp.py"),effect_trace],
            "env":{},"sandboxed":false}}});
        private_write(&directory.join("mcp.json"), &serde_json::to_vec_pretty(&mcp).unwrap());
        let port = unused_port();
        let config = json!({
            "service_instance":{"instance_id":"urn:uar:dot06:fixture"},
            "security":{"jwt_required":true,"jwt_secret":JWT_SECRET,"jwt_issuer":ISSUER,"jwt_audience":"dot06-runtime",
                "settings_mutation_auth_required":false,"settings_admin_key":ADMIN},
            "persistence":{"provider":database.provider,"database_url":database.url,"surreal_ns":database.namespace,
                "surreal_db":database.name,"surreal_user":database.user,"surreal_pass":database.password},
            "resilience":{"rate_limit_enabled":false},
            "llm":{"model":"openai/gpt-4o-mini","base_url":peer.url,"api_key":PROVIDER_SECRET},
            "providers":[{"id":"openai","base_url":peer.url,"api_key":PROVIDER_SECRET,"protocol":"chat",
                "default_model":"gpt-4o-mini","enabled":true,"models":[{"id":"gpt-4o-mini","enabled":true,
                "supports_tools":true,"supports_streaming":true,"execution_profile":{"profile":{
                    "id":"uar.openai-compatible-chat.settings-v1","revision":1},"settingsRevision":1,"reasoning":{"mode":"off"}}}]}],
            "server":{"host":"127.0.0.1","port":port,"grpc_port":unused_port(),"shutdown_timeout_secs":10},
            "memory":{"enabled":false}});
        private_write(&directory.join("config.yaml"), &serde_json::to_vec_pretty(&config).unwrap());
        let mut command = Command::new(env!("CARGO_BIN_EXE_universal-agent-runtime"));
        command.arg("--config").arg(directory.join("config.yaml")).current_dir(directory.join("work"))
            .env_clear().env("PATH", std::env::var_os("PATH").unwrap_or_default()).env("HOME", directory.join("home"))
            .env("MCP_CONFIG_PATH", directory.join("mcp.json")).env("UAR_BUILTIN_SKILLS_DIR", directory.join("no-builtins"))
            .env("CREDENTIAL_ENCRYPTION_KEY", "0123456789abcdef0123456789abcdef")
            .stdin(Stdio::null()).stdout(private_file(&directory.join("stdout.log"))).stderr(private_file(&directory.join("stderr.log")));
        if database.remote && attested { command.env("UAR_REMOTE_SURREAL_DURABILITY_ATTESTED", "1"); }
        let child = command.spawn().expect("actual production runtime binary");
        let node = Self { child, base: format!("http://127.0.0.1:{port}"), directory,
            trace: effect_trace.to_path_buf(), client: reqwest::Client::builder().timeout(WAIT).build().unwrap() };
        private_write(&node.directory.join("process.json"), &serde_json::to_vec(&json!({"pid":node.child.id()})).unwrap());
        node
    }

    pub(super) async fn boot(root: &Path, label: &str, database: &Database, peer: &Peer, effect_trace: &Path, attested: bool) -> Self {
        let mut node = Self::launch(root, label, database, peer, effect_trace, attested);
        let deadline = tokio::time::Instant::now() + Duration::from_secs(180);
        loop {
            if node.client.get(format!("{}/health", node.base)).send().await.is_ok_and(|response| response.status().is_success()) { break; }
            assert!(node.child.try_wait().unwrap().is_none(), "runtime exited; private logs retained");
            assert!(tokio::time::Instant::now() < deadline, "runtime readiness deadline");
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        node
    }

    pub(super) async fn request(&self, credential: &str, method: Method, path: &str, body: Option<Value>) -> (u16, Value) {
        let mut request = self.client.request(method, format!("{}{path}", self.base)).bearer_auth(credential)
            .header("x-uar-admin-key", ADMIN).header("x-uar-workspace-id", "dot06-workspace");
        if let Some(body) = body { request = request.json(&body); }
        let response = request.send().await.expect("actual authenticated API request");
        let status = response.status().as_u16(); let bytes = response.bytes().await.unwrap();
        private_write(&self.directory.join("last-response.json"), &bytes);
        (status, if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap() })
    }

    pub(super) async fn api(&self, method: Method, path: &str, body: Option<Value>, expected: u16) -> Value {
        let (status, value) = self.request(&token(OWNER, TENANT, ISSUER), method, path, body).await;
        assert_eq!(status, expected, "API status; private response retained"); value
    }

    async fn approve(&self, run: &str) {
        let pending = self.api(Method::GET, &format!("/api/uar/runs/{run}/tool-approval/pending"), None, 200).await;
        if let Some(id) = pending["pending"]["approval_id"].as_str().or_else(|| pending["pending"]["approvalId"].as_str()) {
            self.api(Method::POST, &format!("/api/uar/runs/{run}/tool-approval"),
                Some(json!({"approval_id":id,"approved":true})), 200).await;
        }
    }

    pub(super) async fn finish(&self, run: &str, success: bool) {
        let deadline = tokio::time::Instant::now() + WAIT;
        loop {
            self.approve(run).await;
            let state = self.api(Method::GET, &format!("/api/uar/runs/{run}"), None, 200).await;
            if ["done", "error", "cancelled"].contains(&state["status"].as_str().unwrap()) {
                let events = self.api(Method::GET, &format!("/api/uar/runs/{run}/events?after=0"), None, 200).await;
                private_write(&self.directory.join(format!("events-{run}.json")), &serde_json::to_vec_pretty(&events).unwrap());
                assert_eq!(state["status"], if success { "done" } else { "error" });
                let errors = events["events"].as_array().unwrap().iter().filter(|event| event["eventName"] == "agui.error").count();
                assert_eq!(errors == 0, success, "event and RunStatus agreement"); return;
            }
            assert!(tokio::time::Instant::now() < deadline, "run completion deadline");
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }

    pub(super) async fn stop(&mut self) {
        assert!(self.stop_status().await.success(), "orderly runtime shutdown failed");
    }

    pub(super) async fn stop_status(&mut self) -> std::process::ExitStatus {
        assert!(Command::new("/bin/kill").arg("-TERM").arg(self.child.id().to_string()).status().unwrap().success());
        let deadline = tokio::time::Instant::now() + Duration::from_secs(45);
        loop {
            if let Some(status) = self.child.try_wait().unwrap() { return status; }
            assert!(tokio::time::Instant::now() < deadline, "orderly shutdown deadline");
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    pub(super) fn kill(&mut self) { self.child.kill().unwrap(); self.child.wait().unwrap(); }
}

impl Drop for Node {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() { let _ = self.child.kill(); let _ = self.child.wait(); }
        cleanup_peers(&self.trace);
    }
}

pub(super) fn artifact() -> Value {
    let mut artifact = default_agent(); artifact.id = "dot06-artifact".into(); artifact.memory.kb.enabled = false;
    artifact.policy.tools.allow = vec!["resume__effect".into()];
    artifact.policy.provider.default.provider = "openai".into(); artifact.policy.provider.default.model = "gpt-4o-mini".into();
    serde_json::to_value(artifact).unwrap()
}

pub(super) fn credentials(peer: &Peer) -> Value { json!([{"provider_id":"openai","provider_kind":"openai_compatible",
    "base_url":peer.url,"api_key":PROVIDER_SECRET,"default_model":"gpt-4o-mini","context_window":128000}]) }

async fn source(node: &Node, peer: &Peer, mode: &str, binding: bool, selected_model: bool) -> (String, String) {
    let mut body = json!({"artifact":artifact(),"input":mode,"run_credentials":credentials(peer)});
    if selected_model {
        let verified = node.api(Method::POST, "/api/uar/providers/openai/test", Some(json!({"model":"gpt-4o-mini"})), 200).await;
        assert_eq!(verified["ok"], true); assert_eq!(verified["received_text"], true);
        body["model"] = json!("openai/gpt-4o-mini");
    }
    if binding {
        body["service_placement"] = json!({"intent":"new","expectedInstanceId":"urn:uar:dot06:fixture",
            "expectedProfile":universal_agent_runtime::uar::service_instance::SERVICE_PROFILE});
    }
    let created = node.api(Method::POST, "/api/uar/runs", Some(body), 200).await;
    let run = created["run_id"].as_str().unwrap().to_owned();
    node.finish(&run, mode != "unknown").await;
    let checkpoints = node.api(Method::GET, &format!("/api/uar/runs/{run}/checkpoints"), None, 200).await;
    assert_eq!(checkpoints["execution_restorable"], true);
    let checkpoint = checkpoints["checkpoints"].as_array().unwrap().last().expect("real tool loop saved checkpoint")["id"].as_str().unwrap().to_owned();
    (run, checkpoint)
}

async fn envelopes(db: &SurrealDbProvider, run: &str) -> Vec<CheckpointResumeEnvelope> {
    let owner = format!("v1:k:issuer:i:{}:{}:t:{}:{}:s:{}:{}", ISSUER.len(), ISSUER, TENANT.len(), TENANT, OWNER.len(), OWNER);
    let rows = db.list_checkpoint_resumes(&owner, run).await.unwrap();
    assert!(!rows.is_empty());
    for row in &rows {
        row.validate().unwrap(); assert!(!row.source_json.contains(PROVIDER_SECRET)); assert!(!row.checkpoint_json.contains(PROVIDER_SECRET));
        let checkpoint = db.load_checkpoint(&row.checkpoint_id).await.unwrap().expect("atomically saved checkpoint");
        assert_eq!(serde_json::to_value(checkpoint).unwrap(), serde_json::to_value(row.checkpoint().unwrap()).unwrap());
    }
    rows
}

async fn proof(db: &SurrealDbProvider, rows: &[CheckpointResumeEnvelope], run: &str,
    expected_epoch_state: &str, expected_claims: usize) {
    let mut response = db.client().query("SELECT VALUE state FROM checkpoint_runtime_epochs WHERE epoch = $epoch; SELECT VALUE data FROM checkpoint_resume_claims WHERE source_run_id = $run;")
        .bind(("epoch", rows[0].source_runtime_epoch.clone())).bind(("run", run.to_owned())).await.unwrap().check().unwrap();
    let states: Vec<String> = response.take(0).unwrap(); assert_eq!(states, vec![expected_epoch_state]);
    let claims: Vec<String> = response.take(1).unwrap(); assert_eq!(claims.len(), expected_claims);
    for data in claims { let claim: universal_agent_runtime::uar::persistence::checkpoint_resume::CheckpointContinuationClaim = serde_json::from_str(&data).unwrap();
        let envelope = rows.iter().find(|row| row.checkpoint_id == claim.checkpoint_id).unwrap(); claim.validate(envelope).unwrap(); }
}

#[derive(serde::Serialize, serde::Deserialize)]
enum ReadbackOperation {
    Inspect { expected_epoch_state: String, expected_claims: usize, corrupt_checkpoint: Option<String> },
    AssertNoClaims,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ReadbackJob {
    database: Database,
    run: String,
    operation: ReadbackOperation,
    result: PathBuf,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ReadbackResult { envelopes: Vec<CheckpointResumeEnvelope> }

/// Fixture orchestration only: the parent never opens the embedded KV. One
/// child owns one production provider for the whole readback/mutation phase.
pub async fn run_readback_child() -> bool {
    let Some(path) = std::env::var_os("UAR_DOT06_READBACK_JOB") else { return false; };
    let job: ReadbackJob = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let db = job.database.connect().await;
    let rows = match job.operation {
        ReadbackOperation::Inspect { expected_epoch_state, expected_claims, corrupt_checkpoint } => {
            let rows = envelopes(&db, &job.run).await;
            proof(&db, &rows, &job.run, &expected_epoch_state, expected_claims).await;
            if let Some(checkpoint) = corrupt_checkpoint {
                let mut altered = rows.iter().find(|row| row.checkpoint_id == checkpoint).unwrap().clone();
                let mut source: Value = serde_json::from_str(&altered.source_json).unwrap();
                source["context"]["agent_snapshot"]["artifact"]["prompt"]["system"] = json!("tampered stored definition");
                altered.source_json = serde_json::to_string(&source).unwrap();
                db.client().query("UPDATE checkpoint_resume_envelopes SET data = $data WHERE source_run_id = $run AND checkpoint_id = $checkpoint;")
                    .bind(("data", serde_json::to_string(&altered).unwrap())).bind(("run", job.run.clone()))
                    .bind(("checkpoint", checkpoint)).await.unwrap().check().unwrap();
            }
            rows
        }
        ReadbackOperation::AssertNoClaims => {
            let mut response = db.client().query("SELECT VALUE data FROM checkpoint_resume_claims WHERE source_run_id = $run;")
                .bind(("run", job.run)).await.unwrap().check().unwrap();
            let claims: Vec<String> = response.take(0).unwrap(); assert!(claims.is_empty());
            Vec::new()
        }
    };
    private_write(&job.result, &serde_json::to_vec(&ReadbackResult { envelopes: rows }).unwrap());
    true
}

struct ReadbackChild(Child);

impl Drop for ReadbackChild {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() { let _ = self.0.kill(); let _ = self.0.wait(); }
    }
}

async fn readback(database: &Database, root: &Path, label: &str, run: &str,
    operation: ReadbackOperation) -> Vec<CheckpointResumeEnvelope> {
    let directory = root.join(label); private_dir(&directory);
    let result = directory.join("readback-result.json");
    let job = ReadbackJob { database: database.clone(), run: run.to_owned(), operation, result: result.clone() };
    let job_path = directory.join("readback-job.json");
    private_write(&job_path, &serde_json::to_vec(&job).unwrap());
    let executable = std::env::current_exe().unwrap();
    let mut command = Command::new(&executable);
    // This invokes the unchanged full integration entrypoint in its private
    // fixture-child mode. There is no filter or independent acceptance gate.
    command.args(["--nocapture", "--test-threads=1"]).env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default()).env("HOME", &directory)
        .env("UAR_DOT06_READBACK_JOB", &job_path).stdin(Stdio::null())
        .stdout(private_file(&directory.join("stdout.log"))).stderr(private_file(&directory.join("stderr.log")));
    for name in ["DYLD_LIBRARY_PATH", "LD_LIBRARY_PATH"] {
        if let Some(value) = std::env::var_os(name) { command.env(name, value); }
    }
    if database.remote && std::env::var("UAR_REMOTE_SURREAL_DURABILITY_ATTESTED").is_ok_and(|value| value == "1") {
        command.env("UAR_REMOTE_SURREAL_DURABILITY_ATTESTED", "1");
    }
    let mut child = ReadbackChild(command.spawn().expect("owned durable readback child"));
    let pid = child.0.id();
    let process = directory.join("readback-process.json");
    private_write(&process, &serde_json::to_vec(&json!({"pid":pid,"role":"durable-readback","executable":executable})).unwrap());
    let deadline = tokio::time::Instant::now() + WAIT;
    let status = loop {
        if let Some(status) = child.0.try_wait().unwrap() { break status; }
        assert!(tokio::time::Instant::now() < deadline, "durable readback child deadline; private diagnostics retained");
        tokio::time::sleep(Duration::from_millis(25)).await;
    };
    private_write(&process, &serde_json::to_vec(&json!({"pid":pid,"role":"durable-readback","executable":executable,
        "joined":true,"exit_code":status.code()})).unwrap());
    assert!(status.success(), "durable readback child failed; private diagnostics retained");
    // A joined OS exit releases this phase's KV ownership before the caller can
    // launch its next readback or production runtime. Lexical Drop is not proof.
    let result: ReadbackResult = serde_json::from_slice(&std::fs::read(result).unwrap()).unwrap();
    result.envelopes
}

async fn wait_for(node: &Node, run: &str, condition: impl Fn() -> bool) {
    let deadline = tokio::time::Instant::now() + WAIT;
    while !condition() { node.approve(run).await; assert!(tokio::time::Instant::now() < deadline, "controlled boundary deadline");
        tokio::time::sleep(Duration::from_millis(25)).await; }
}

fn cleanup_peers(effect_trace: &Path) {
    for pid in trace(effect_trace).iter().filter_map(|row| row["pid"].as_u64()).collect::<BTreeSet<_>>() {
        let _ = Command::new("/bin/kill").arg("-TERM").arg(pid.to_string())
            .stdout(Stdio::null()).stderr(Stdio::null()).status();
    }
}

async fn corrupted_envelope(root: &Path, profile: &str) {
    private_dir(root); let effect_trace = root.join("effects.jsonl"); private_write(&effect_trace, b"");
    let database = Database::new(root, profile, "corrupt"); let peer = Peer::start(root).await;
    let mut predecessor = Node::boot(root, "predecessor", &database, &peer, &effect_trace, true).await;
    let (run, checkpoint) = source(&predecessor, &peer, "seed", false, false).await;
    predecessor.stop().await;
    readback(&database, root, "source-readback", &run, ReadbackOperation::Inspect {
        expected_epoch_state: "orderly_excluded".into(), expected_claims: 0, corrupt_checkpoint: Some(checkpoint.clone()),
    }).await;
    let mut current = Node::boot(root, "current", &database, &peer, &effect_trace, true).await;
    let requests = peer.count(); let effect_count = effects(&effect_trace);
    for (method, path, body) in [
        (Method::GET, format!("/api/uar/runs/{run}/checkpoints"), None),
        (Method::POST, format!("/api/uar/runs/{run}/resume/{checkpoint}"), Some(json!({"run_credentials":credentials(&peer)}))),
    ] {
        let response = current.api(method, &path, body, 409).await;
        assert_eq!(response["code"], "checkpoint_resume_invalid");
    }
    assert_eq!(requests, peer.count()); assert_eq!(effect_count, effects(&effect_trace));
    current.stop().await;
    readback(&database, root, "final-readback", &run, ReadbackOperation::AssertNoClaims).await;
    cleanup_peers(&effect_trace);
    private_write(&root.join("acceptance.json"), &serde_json::to_vec_pretty(&json!({"profile":profile,"case":"corrupt",
        "status":409,"newExecutions":0,"verified":true,"genuineInference":false})).unwrap());
}

async fn scenario(root: &Path, profile: &str, case: &str) {
    private_dir(root); let effect_trace = root.join("effects.jsonl"); private_write(&effect_trace, b"");
    let database = Database::new(root, profile, case); let peer = Peer::start(root).await;
    let mut predecessor = Node::boot(root, "predecessor", &database, &peer, &effect_trace, true).await;
    let (run, checkpoint) = source(&predecessor, &peer, if case == "uncertain" { "unknown" } else { "seed" },
        case == "binding", case == "target").await;
    let predecessor_pid = predecessor.child.id();
    if case == "abrupt" { predecessor.kill(); }
    else if case == "uncertain" {
        // Even after a real orderly signal and joined shutdown work, unknown
        // effects prevent an exclusion certificate. Exit failure is retained.
        assert!(!predecessor.stop_status().await.success(), "unknown effects must fail orderly certification");
    } else { predecessor.stop().await; }
    let rows = readback(&database, root, "source-readback", &run, ReadbackOperation::Inspect {
        expected_epoch_state: if case == "abrupt" || case == "uncertain" { "active" } else { "orderly_excluded" }.into(),
        expected_claims: 0, corrupt_checkpoint: None,
    }).await;
    let finals = rows.iter().filter(|row| row.checkpoint().unwrap().node_id == "tool_loop_1").collect::<Vec<_>>();
    assert_eq!(finals.len(), 1, "one committed post-ToolEnd checkpoint");
    assert_eq!(finals[0].checkpoint_id, checkpoint);
    assert_eq!(rows.iter().filter(|row| row.checkpoint().unwrap().node_id == "pending_tool_approval").count(), 1,
        "one protected pre-effect approval checkpoint");
    let mut current = Node::boot(root, "current", &database, &peer, &effect_trace, true).await;
    assert_ne!(predecessor_pid, current.child.id(), "fresh OS process");
    if case == "target" {
        assert_eq!(finals[0].source_run().unwrap().context["effective_target"]["kind"], "model");
        let verified = current.api(Method::POST, "/api/uar/providers/openai/test", Some(json!({"model":"gpt-4o-mini"})), 200).await;
        assert_eq!(verified["ok"], true);
    }
    current.api(Method::GET, &format!("/api/uar/runs/{run}"), None, 404).await;
    current.api(Method::GET, &format!("/api/uar/runs/{run}/checkpoints"), None, 200).await;
    let path = format!("/api/uar/runs/{run}/resume/{checkpoint}"); let before = effects(&effect_trace); let requests = peer.count();
    let input = match case { "kill-before" => "kill_before", "kill-after" => "hold_after_effect", _ => "resume" };
    let body = json!({"input":input,"run_credentials":credentials(&peer)});
    if case == "abrupt" || case == "uncertain" {
        let rejected = current.api(Method::POST, &path, Some(body), 409).await;
        assert_eq!(rejected["code"], "checkpoint_predecessor_uncertain"); assert_eq!(peer.count(), requests); assert_eq!(effects(&effect_trace), before);
        current.kill();
    } else if case == "binding" {
        // A fresh process answers at a different actual endpoint. Revalidation
        // must reject the original identity-bearing endpoint roles.
        let rejected = current.api(Method::POST, &path, Some(body), 409).await;
        assert_eq!(rejected["code"], "service_instance_incompatible");
        assert_eq!(peer.count(), requests); assert_eq!(effects(&effect_trace), before);
        current.stop().await;
    } else if case == "policy" {
        let conversation = finals[0].source_run().unwrap().conversation_id.unwrap();
        current.api(Method::PUT, &format!("/api/uar/conversations/{conversation}/policy"),
            Some(json!({"tool_approval":"deny"})), 200).await;
        let rejected = current.api(Method::POST, &path, Some(body), 409).await;
        assert_eq!(rejected["code"], "checkpoint_admission_denied");
        assert_eq!(peer.count(), requests); assert_eq!(effects(&effect_trace), before);
        let runs = current.api(Method::GET, "/api/uar/runs", None, 200).await;
        assert!(runs.as_array().unwrap().iter().any(|run| run["status"] == "error"));
        current.stop().await;
    } else if case == "negative" {
        for (subject, tenant, issuer, expected) in [("other", TENANT, ISSUER, 404), (OWNER, "other", ISSUER, 404), (OWNER, TENANT, "other", 401)] {
            let (status, _) = current.request(&token(subject, tenant, issuer), Method::POST, &path, Some(body.clone())).await; assert_eq!(status, expected);
            let (status, _) = current.request(&token(subject, tenant, issuer), Method::GET, &format!("/api/uar/runs/{run}/checkpoints"), None).await; assert_eq!(status, expected);
        }
        let missing = current.api(Method::POST, &path, Some(json!({"input":"resume"})), 422).await; assert_eq!(missing["code"], "run_credential_required");
        let mut altered = artifact(); altered["prompt"]["system"] = json!("altered");
        let rejected = current.api(Method::POST, &path, Some(json!({"artifact":altered,"run_credentials":credentials(&peer)})), 409).await;
        assert_eq!(rejected["code"], "run_artifact_mismatch");
        current.api(Method::POST, &path, Some(json!({"session_id":"different","run_credentials":credentials(&peer)})), 409).await;
        current.api(Method::POST, &path, Some(json!({"history":{"session_id":"caller-history","messages":[]},"run_credentials":credentials(&peer)})), 422).await;
        let binding = current.api(Method::POST, &path, Some(json!({"run_credentials":credentials(&peer),"service_placement":{
            "intent":"reattach","expectedInstanceId":"altered","expectedProfile":"local"}})), 409).await;
        assert_eq!(binding["code"], "service_binding_unavailable");
        assert_eq!(peer.count(), requests); assert_eq!(effects(&effect_trace), before);
        let result = current.api(Method::POST, &format!("/api/uar/runs/{run}/resume"), Some(body), 200).await;
        current.finish(result["run_id"].as_str().unwrap(), true).await;
        current.api(Method::POST, &path, Some(json!({"run_credentials":credentials(&peer)})), 409).await;
        assert_eq!(effects(&effect_trace), before + 1); current.stop().await;
    } else if case == "race" {
        assert_eq!(profile, "shared");
        let mut second = Node::boot(root, "second", &database, &peer, &effect_trace, true).await; assert_ne!(current.child.id(), second.child.id());
        let principal = token(OWNER, TENANT, ISSUER);
        let (a, b) = tokio::join!(current.request(&principal, Method::POST, &path, Some(body.clone())), second.request(&principal, Method::POST, &path, Some(body)));
        assert_eq!(BTreeSet::from([a.0, b.0]), BTreeSet::from([200, 409]));
        if a.0 == 200 { current.finish(a.1["run_id"].as_str().unwrap(), true).await; } else { second.finish(b.1["run_id"].as_str().unwrap(), true).await; }
        assert_eq!(effects(&effect_trace), before + 1); current.stop().await; second.stop().await;
    } else {
        let result = current.api(Method::POST, &path, Some(body.clone()), 200).await;
        let continuation = result["run_id"].as_str().unwrap(); assert_ne!(continuation, run);
        if case == "kill-before" || case == "kill-after" {
            wait_for(&current, continuation, || if case == "kill-before" { peer.saw("kill_before") } else { effects(&effect_trace) == before + 1 }).await;
            current.kill();
            let mut second = Node::boot(root, "second", &database, &peer, &effect_trace, true).await;
            let count = peer.count(); let effect_count = effects(&effect_trace);
            let rejected = second.api(Method::POST, &path, Some(body), 409).await; assert_eq!(rejected["code"], "checkpoint_resume_conflict");
            assert_eq!(peer.count(), count); assert_eq!(effects(&effect_trace), effect_count);
            if case == "kill-before" { assert_eq!(effect_count, before); }
            second.stop().await;
        } else {
            current.finish(continuation, true).await;
            current.api(Method::POST, &path, Some(body), 409).await; assert_eq!(effects(&effect_trace), before + 1);
            current.stop().await;
        }
    }
    readback(&database, root, "final-readback", &run, ReadbackOperation::Inspect {
        expected_epoch_state: if case == "abrupt" || case == "uncertain" { "active" } else { "orderly_excluded" }.into(),
        expected_claims: if case == "abrupt" || case == "uncertain" || case == "binding" { 0 } else { 1 },
        corrupt_checkpoint: None,
    }).await;
    // Retained peers identify themselves in this private protocol trace. Only
    // fixture children are signalled; no process inventory or argv is inspected.
    cleanup_peers(&effect_trace);
    private_write(&root.join("acceptance.json"), &serde_json::to_vec_pretty(&json!({"profile":profile,"case":case,
        "sourceProcess":predecessor_pid,"sourceRun":run,"checkpoint":checkpoint,"effects":effects(&effect_trace),
        "providerRequests":peer.count(),"verified":true,"genuineInference":false})).unwrap());
}

async fn unsupported(root: &Path, profile: &str, attested: bool) {
    private_dir(root); let effect_trace = root.join("effects.jsonl"); private_write(&effect_trace, b"");
    let peer = Peer::start(root).await; let database = Database::new(root, profile, "unattested");
    if profile == "memory" {
        // The release feature set excludes the development-only kv-mem engine.
        // Qualify its actual explicit startup refusal, not an unentered HTTP API.
        let mut node = Node::launch(root, "unsupported", &database, &peer, &effect_trace, attested);
        let deadline = tokio::time::Instant::now() + WAIT;
        let status = loop {
            assert!(!node.client.get(format!("{}/health", node.base)).send().await
                .is_ok_and(|response| response.status().is_success()), "memory runtime unexpectedly accepted startup");
            if let Some(status) = node.child.try_wait().unwrap() { break status; }
            assert!(tokio::time::Instant::now() < deadline, "memory startup refusal deadline; private logs retained");
            tokio::time::sleep(Duration::from_millis(100)).await;
        };
        assert!(!status.success(), "memory configuration must be explicitly rejected by the release profile");
        let stdout = std::fs::read_to_string(node.directory.join("stdout.log")).unwrap();
        let stderr = std::fs::read_to_string(node.directory.join("stderr.log")).unwrap();
        assert!(stdout.contains("Unsupported scheme: memory") || stderr.contains("Unsupported scheme: memory"),
            "memory startup did not report the observed unsupported transport reason; private logs retained");
        assert_eq!(peer.count(), 0); assert_eq!(effects(&effect_trace), 0);
        cleanup_peers(&effect_trace);
        private_write(&root.join("acceptance.json"), &serde_json::to_vec_pretty(&json!({"profile":profile,"attested":attested,
            "status":"startup_rejected","refusalBoundary":"startup","reason":"unsupported_memory_transport",
            "httpResumeStatus":Value::Null,"runRequests":0,"resumeRequests":0,"acceptedCheckpoints":0,
            "executionCount":0,"providerRequests":0,"effects":0,"startupAccepted":false,
            "runtimeExitCode":status.code(),"verified":true,"durabilityCertified":false,"checkpointApiCertified":false})).unwrap());
        return;
    }
    let mut node = Node::boot(root, "unsupported", &database, &peer, &effect_trace, attested).await;
    for (method, path, body) in [(Method::GET,"/api/uar/runs/unsupported/checkpoints", None),
        (Method::POST,"/api/uar/runs/unsupported/resume", Some(json!({}))),
        (Method::POST,"/api/uar/runs/unsupported/resume/checkpoint", Some(json!({})))] {
        let response = node.api(method, path, body, 503).await; assert_eq!(response["code"], "checkpoint_resume_unsupported");
    }
    assert_eq!(peer.count(), 0); assert_eq!(effects(&effect_trace), 0); node.stop().await;
    cleanup_peers(&effect_trace);
    private_write(&root.join("acceptance.json"), &serde_json::to_vec_pretty(&json!({"profile":profile,"attested":attested,
        "status":503,"executionCount":0,"verified":true})).unwrap());
}

pub async fn qualify() {
    assert!(cfg!(feature = "postgres-backend"), "complete dot06 gate requires the postgres-backend capability for its unsupported-profile proof");
    let root = PathBuf::from(std::env::var("UAR_DOT06_EVIDENCE_ROOT").expect("private preserved evidence directory"));
    private_dir(&root);
    for profile in ["local", "shared"] {
        for case in ["resume", "target", "negative", "policy", "binding", "abrupt", "uncertain", "kill-before", "kill-after"] {
            scenario(&root.join(format!("{profile}-{case}")), profile, case).await;
            eprintln!("dot06 profile={profile} case={case} status=passed");
        }
        corrupted_envelope(&root.join(format!("{profile}-corrupt")), profile).await;
        eprintln!("dot06 profile={profile} case=corrupt status=passed");
    }
    scenario(&root.join("shared-race"), "shared", "race").await;
    unsupported(&root.join("memory"), "memory", false).await;
    unsupported(&root.join("unattested"), "shared", false).await;
    #[cfg(feature = "postgres-backend")]
    unsupported(&root.join("postgres"), "postgres", false).await;
}
