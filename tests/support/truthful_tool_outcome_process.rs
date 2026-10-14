//! Actual server composition and team-consumer acceptance. Logs remain local.

use super::peer;
use reqwest::Method;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::Duration,
};
use universal_agent_runtime::{
    uar::{
        compiler::collaboration::{CollaborationStorage, SurrealCollaborationStorage},
        persistence::{
            PersistenceLayer,
            agent_threads::{CanonicalReceiptSource, CanonicalToolReceipt},
            providers::surreal::SurrealDbProvider,
            tool_admission::ToolAdmissionEvidenceState,
        },
    },
};

const JWT_SECRET: &str = "dot04-process-jwt-fixture";
const ADMIN_KEY: &str = "dot04-process-admin-fixture";
const WORKSPACE: &str = "dot04-process-workspace";
const TEAM_TOOL: &str = "dot04__outcomes__effect";
const WAIT: Duration = Duration::from_secs(120);

struct Database {
    url: String,
    namespace: String,
    name: String,
    user: Option<String>,
    password: Option<String>,
}

impl Database {
    fn configuration(root: &Path, shared: bool, label: &str) -> Self {
        if shared {
            let path =
                std::env::var("UAR_DOT04_INFRA_FILE").expect("private infrastructure manifest");
            let input: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
            let infrastructure = &input["surreal"];
            Self {
                url: infrastructure["url"].as_str().unwrap().into(),
                namespace: std::env::var("UAR_DOT04_NAMESPACE").unwrap(),
                name: format!("truthful_outcomes_binary_{}", label.replace('-', "_")),
                user: infrastructure["username"].as_str().map(str::to_owned),
                password: infrastructure["password"].as_str().map(str::to_owned),
            }
        } else {
            std::fs::create_dir_all(root.join("binary-database")).unwrap();
            Self {
                url: format!(
                    "surrealkv://{}",
                    root.join("binary-database").join(label).display()
                ),
                namespace: "dot04_process".into(),
                name: "outcomes".into(),
                user: None,
                password: None,
            }
        }
    }

    async fn connect(&self) -> SurrealDbProvider {
        SurrealDbProvider::new(
            &self.url,
            self.user.as_deref(),
            self.password.as_deref(),
            Some(&self.namespace),
            Some(&self.name),
        )
        .await
        .expect("binary durable store")
    }
}

struct Node {
    child: Child,
    base: String,
    directory: PathBuf,
    trace: PathBuf,
    client: reqwest::Client,
    token: String,
    started_at: String,
}

fn token() -> String {
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &json!({"sub":peer::OWNER,"roles":["user"],"exp":chrono::Utc::now().timestamp()+3600}),
        &jsonwebtoken::EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
    .unwrap()
}

fn unused_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

impl Node {
    async fn boot(root: &Path, label: &str, database: &Database, model: &peer::Peer) -> Self {
        let directory = root.join(format!("binary-{label}"));
        for suffix in ["home", "work/policies", "work/skills/dot04", "no-builtins"] {
            std::fs::create_dir_all(directory.join(suffix)).unwrap();
        }
        for name in [
            "default.cedar",
            "tool-approval.cedar",
            "skill-mutation.cedar",
        ] {
            std::fs::copy(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("policies")
                    .join(name),
                directory.join("work/policies").join(name),
            )
            .unwrap();
        }
        let trace = directory.join("effects.jsonl");
        let configuration = serde_json::to_vec_pretty(&peer::configuration(&trace)).unwrap();
        std::fs::write(directory.join("mcp.json"), &configuration).unwrap();
        std::fs::write(directory.join("work/skills/dot04/mcp.json"), &configuration).unwrap();
        std::fs::write(directory.join("work/skills/dot04/SKILL.md"),
            "---\nname: dot04\nversion: 1.0.0\ndescription: Controlled tool outcome production acceptance\ntools:\n  - dot04__outcomes__effect\n---\nUse the selected effect tool exactly once.\n").unwrap();
        let port = unused_port();
        // A/B and restarts share this logical backend identity, while their
        // process IDs and catalog execution-owner claims remain independent.
        let service_instance_id = format!(
            "urn:uar:dot04:service:{}",
            sha256(
                &serde_json::to_vec(&(&database.url, &database.namespace, &database.name)).unwrap()
            )
        );
        let config = json!({
            "service_instance":{"instance_id":service_instance_id},
            "security":{"jwt_required":true,"jwt_secret":JWT_SECRET,
                "settings_mutation_auth_required":false,"settings_admin_key":ADMIN_KEY},
            "resilience":{"rate_limit_enabled":false},
            "persistence":{"provider":"surreal","database_url":database.url,
                "surreal_ns":database.namespace,"surreal_db":database.name,
                "surreal_user":database.user,"surreal_pass":database.password},
            "llm":{"model":"openai/gpt-4o-mini","base_url":model.url,"api_key":peer::SECRET},
            "providers":[{"id":"openai","base_url":model.url,"api_key":peer::SECRET,
                "protocol":"chat","default_model":"gpt-4o-mini","enabled":true,
                "models":[{"id":"gpt-4o-mini","enabled":true,"supports_tools":true,
                    "supports_streaming":true,"execution_profile":{
                        "profile":{"id":"uar.openai-compatible-chat.settings-v1","revision":1},
                        "settingsRevision":1,"reasoning":{"mode":"off"}}}]}],
            "server":{"host":"127.0.0.1","port":port,"grpc_port":unused_port(),
                "shutdown_timeout_secs":5},
            "memory":{"enabled":false}
        });
        std::fs::write(
            directory.join("config.yaml"),
            serde_json::to_vec_pretty(&config).unwrap(),
        )
        .unwrap();
        let started_at = chrono::Utc::now().to_rfc3339();
        let child = Command::new(env!("CARGO_BIN_EXE_universal-agent-runtime"))
            .arg("--config")
            .arg(directory.join("config.yaml"))
            .current_dir(directory.join("work"))
            .env_clear()
            .env("PATH", std::env::var_os("PATH").unwrap_or_default())
            .env("HOME", directory.join("home"))
            .env("MCP_CONFIG_PATH", directory.join("mcp.json"))
            .env("UAR_BUILTIN_SKILLS_DIR", directory.join("no-builtins"))
            .env(
                "CREDENTIAL_ENCRYPTION_KEY",
                "0123456789abcdef0123456789abcdef",
            )
            .env("UAR_REMOTE_SURREAL_DURABILITY_ATTESTED", "1")
            .stdin(Stdio::null())
            .stdout(std::fs::File::create(directory.join("stdout.log")).unwrap())
            .stderr(std::fs::File::create(directory.join("stderr.log")).unwrap())
            .spawn()
            .expect("actual server binary");
        let mut node = Self {
            child,
            base: format!("http://127.0.0.1:{port}"),
            directory,
            trace,
            client: reqwest::Client::new(),
            token: token(),
            started_at,
        };
        let deadline = tokio::time::Instant::now() + Duration::from_secs(180);
        loop {
            if node
                .client
                .get(format!("{}/health", node.base))
                .send()
                .await
                .is_ok_and(|response| response.status().is_success())
            {
                break;
            }
            assert!(
                node.child.try_wait().unwrap().is_none(),
                "server exited; logs at {}",
                node.directory.display()
            );
            assert!(
                tokio::time::Instant::now() < deadline,
                "server readiness timeout; logs at {}",
                node.directory.display()
            );
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        node
    }

    async fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
        expected: u16,
    ) -> Value {
        let mut request = self
            .client
            .request(method, format!("{}{path}", self.base))
            .bearer_auth(&self.token)
            .header("x-uar-workspace-id", WORKSPACE)
            .header("x-uar-admin-key", ADMIN_KEY);
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send().await.expect("binary API request");
        let status = response.status().as_u16();
        let bytes = response.bytes().await.unwrap();
        if status != expected {
            std::fs::write(self.directory.join("api-failure.json"), &bytes).unwrap();
        }
        assert_eq!(
            status,
            expected,
            "binary API {path}; evidence at {}",
            self.directory.display()
        );
        if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap()
        }
    }

    async fn get(&self, path: &str) -> Value {
        self.request(Method::GET, path, None, 200).await
    }
    async fn post(&self, path: &str, body: Value, status: u16) -> Value {
        self.request(Method::POST, path, Some(body), status).await
    }

    fn peer_pids(&self) -> Vec<u32> {
        let output = Command::new("/bin/ps")
            .args(["-axo", "pid=,args="])
            .output()
            .unwrap();
        assert!(output.status.success());
        let trace = self.trace.to_str().unwrap();
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .filter(|line| line.contains("truthful_tool_outcome_mcp.py") && line.contains(trace))
            .filter_map(|line| line.split_whitespace().next()?.parse().ok())
            .collect()
    }

    async fn close_peer_before_dispatch(&self) {
        let pids = self.peer_pids();
        assert!(!pids.is_empty(), "real discovered MCP peer must exist");
        for pid in &pids {
            assert!(
                Command::new("/bin/kill")
                    .arg("-TERM")
                    .arg(pid.to_string())
                    .status()
                    .unwrap()
                    .success()
            );
        }
        let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
        while self.peer_pids().iter().any(|pid| pids.contains(pid)) {
            assert!(
                tokio::time::Instant::now() < deadline,
                "MCP peer did not close"
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        // Allow the already established rmcp reader to publish its closed queue
        // before approving dispatch. The outcome assertion proves the ordering.
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    async fn run_events(&self, run: &str, mode: &str) -> Vec<Value> {
        let deadline = tokio::time::Instant::now() + WAIT;
        // Team dispatch returns its durable attempt before the actor has
        // registered the run. Await that public registration boundary.
        let mut response = loop {
            let response = self
                .client
                .get(format!(
                    "{}/api/uar/runs/{run}/stream?last_event_id=0",
                    self.base
                ))
                .bearer_auth(&self.token)
                .header("x-uar-workspace-id", WORKSPACE)
                .header("Last-Event-ID", "0")
                .send()
                .await
                .unwrap();
            if response.status().is_success() {
                break response;
            }
            assert_eq!(
                response.status().as_u16(),
                404,
                "binary stream registration; logs at {}",
                self.directory.display()
            );
            assert!(
                tokio::time::Instant::now() < deadline,
                "team run registration deadline; logs at {}",
                self.directory.display()
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        };
        let mut buffer = String::new();
        let mut events = Vec::new();
        let mut closed = false;
        loop {
            let chunk = tokio::time::timeout_at(deadline, response.chunk())
                .await
                .expect("binary run terminal deadline")
                .unwrap()
                .expect("terminal stream frame");
            buffer.push_str(&String::from_utf8_lossy(&chunk));
            while let Some(end) = buffer.find("\n\n") {
                let frame = buffer[..end].to_owned();
                buffer.drain(..end + 2);
                let event = frame
                    .lines()
                    .find_map(|line| line.strip_prefix("event: "))
                    .unwrap_or_default();
                let Some(data) = frame.lines().find_map(|line| line.strip_prefix("data: ")) else {
                    continue;
                };
                let value: Value = serde_json::from_str(data).unwrap();
                if event == "agui.tool_call.approval_required" {
                    if mode == "predispatch" && !closed {
                        self.close_peer_before_dispatch().await;
                        closed = true;
                    }
                    self.post(
                        &format!("/api/uar/runs/{run}/tool-approval"),
                        json!({"approved":true,"approval_id":value["approval_id"]}),
                        200,
                    )
                    .await;
                }
                events.push(json!({"event":event,"data":value}));
                if event == "agui.done" || event == "agui.error" {
                    if mode == "predispatch" {
                        assert!(
                            closed,
                            "predispatch fixture never reached approval boundary"
                        );
                    }
                    let path = self.directory.join("run-events.json");
                    std::fs::write(&path, serde_json::to_vec_pretty(&events).unwrap()).unwrap();
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
                            .unwrap();
                    }
                    return events;
                }
            }
        }
    }

    async fn stop(&mut self) {
        assert!(
            Command::new("/bin/kill")
                .arg("-TERM")
                .arg(self.child.id().to_string())
                .status()
                .unwrap()
                .success()
        );
        let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
        while self.child.try_wait().unwrap().is_none() {
            assert!(
                tokio::time::Instant::now() < deadline,
                "server shutdown timeout; logs at {}",
                self.directory.display()
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        for pid in self.peer_pids() {
            let _ = Command::new("/bin/kill")
                .arg("-TERM")
                .arg(pid.to_string())
                .status();
        }
        if std::thread::panicking() {
            eprintln!("dot04 process evidence at {}", self.directory.display());
        }
    }
}

fn sha256(bytes: &[u8]) -> String {
    format!(
        "sha256:{}",
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
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

fn finalize(document: &mut Value) {
    document.as_object_mut().unwrap().remove("contentDigest");
    document["contentDigest"] = json!(sha256(&serde_json::to_vec(&canonical(document)).unwrap()));
}

fn reference(document: &Value) -> Value {
    json!({"id":document["id"],"version":document["version"],"digest":document["contentDigest"]})
}

fn team_token_capacity() -> u64 {
    let model = universal_agent_runtime::llm::catalog::ModelCatalog::global()
        .model("openai", "gpt-4o-mini")
        .expect("fixture model must exist in the actual runtime catalog");
    // The settings-only profile has no final-wire token ceiling, so admission
    // reserves catalog limits for both the tool call and the final prose call.
    model
        .limits
        .context_window
        .checked_add(model.limits.max_output)
        .and_then(|tokens| tokens.checked_mul(2))
        .expect("two catalog-bounded fixture calls must fit a finite token budget")
}

async fn install_definition(node: &Node, mode: &str, ordinary: bool) -> Value {
    let capabilities = node.get("/api/v1/collaboration/capabilities").await;
    assert_eq!(
        capabilities["collaboration"]["activation"]["teamExecution"],
        true
    );
    let catalog = node.get("/api/uar/skills/deployment-catalog").await;
    let skill = catalog["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["skillId"] == "dot04")
        .expect("actual startup-installed filesystem skill");
    assert_eq!(skill["availability"], "available");
    assert_eq!(skill["skillRef"]["requiredTools"], json!([TEAM_TOOL]));
    let mut agent: Value = serde_json::from_str(include_str!(
        "../fixtures/collaboration/agent-definition.json"
    ))
    .unwrap();
    agent["id"] = json!(format!("urn:uar:dot04:agent:{mode}"));
    agent["skills"] = json!([skill["skillRef"].clone()]);
    agent["input"] = json!({});
    agent["output"] = json!({});
    // Cases use explicit task input and request no inherited context resources.
    agent["context"] = json!({"mode":"none","artifacts":[],"history":"none","memoryScopes":[]});
    // One user, assistant call, tool reply, and final prose; system is pinned.
    agent["contextStrategy"]["value"]["max_messages"] = json!(4);
    // Draft.2 requires the retained source and semantic metadata. Keep the
    // valid fixture values and bind its explicit rename to this scenario ID.
    agent["renameMapping"]["targetId"] = agent["id"].clone();
    finalize(&mut agent);
    let mut team: Value = serde_json::from_str(include_str!(
        "../fixtures/collaboration/team-definition.json"
    ))
    .unwrap();
    team["id"] = json!(format!("urn:uar:dot04:team:{mode}"));
    team["members"][0]["definition"] = reference(&agent);
    team["input"] = json!({});
    team["output"] = json!({});
    if !ordinary {
        team["budget"]["maxTokens"] = json!(team_token_capacity());
    }
    finalize(&mut team);
    // Retain operator acceptance and resolve its existing immutable workflow;
    // this package does not start or activate the workflow.
    let workflow: Value = serde_json::from_str(include_str!(
        "../fixtures/collaboration/workflow-definition.json"
    ))
    .unwrap();
    let mut files = serde_json::Map::new();
    let mut inventory = Vec::new();
    for (path, document) in [
        ("agent-definition.json", &agent),
        ("team-definition.json", &team),
        ("workflow-definition.json", &workflow),
    ] {
        let source = serde_json::to_string_pretty(document).unwrap();
        inventory.push(json!({"path":path,"kind":document["kind"],"definition":reference(document),"byteDigest":sha256(source.as_bytes())}));
        files.insert(path.into(), json!(source));
    }
    let mut manifest: Value = serde_json::from_str(include_str!(
        "../fixtures/collaboration/package-manifest.json"
    ))
    .unwrap();
    manifest["id"] = json!(format!("urn:uar:dot04:package:{mode}"));
    manifest["entrypoints"] = json!([reference(if ordinary { &agent } else { &team })]);
    manifest["files"] = json!(inventory);
    manifest["lock"] = json!([
        {"requestedBy":team["id"],"reference":reference(&agent),"resolvedPath":"agent-definition.json"},
        {"requestedBy":team["id"],"reference":reference(&workflow),"resolvedPath":"workflow-definition.json"}
    ]);
    finalize(&mut manifest);
    node.post(
        "/api/v1/collaboration/packages:install",
        json!({"commandId":format!("dot04-package-{mode}"),
        "manifest":serde_json::to_string_pretty(&manifest).unwrap(),"files":files}),
        201,
    )
    .await;
    let mut binding: Value = serde_json::from_str(include_str!(
        "../fixtures/collaboration/deployment-binding.json"
    ))
    .unwrap();
    binding["id"] = json!(format!("urn:uar:dot04:binding:{mode}"));
    binding["package"] = reference(&manifest);
    binding["ownerId"] = capabilities["bindingOwnerId"].clone();
    binding["workspaceId"] = json!(WORKSPACE);
    binding["runtimeInstanceId"] = capabilities["instance"]["id"].clone();
    binding["modelBindings"][0]["modelId"] = json!("gpt-4o-mini");
    binding["modelBindings"][0]["profile"] =
        json!({"id":"uar.openai-compatible-chat.settings-v1","revision":1});
    binding["modelBindings"][0]["settingsRevision"] = json!(1);
    let mut bound_skill = skill["skillRef"].clone();
    bound_skill["installedLocation"] = skill["privateBinding"]["installedLocation"].clone();
    binding["skillBindings"] = json!([bound_skill]);
    binding["representationGrantRefs"] = json!([]);
    if !ordinary {
        binding["effectiveBudget"]["maxTokens"] = json!(team_token_capacity());
    }
    finalize(&mut binding);
    let installed = node
        .post(
            "/api/v1/collaboration/deployment-bindings",
            json!({
        "commandId":format!("dot04-binding-{mode}"),"expectedRevision":0,"binding":binding}),
            201,
        )
        .await;
    let preflight_path = node.directory.join("binding-install.json");
    std::fs::write(
        &preflight_path,
        serde_json::to_vec_pretty(&installed).unwrap(),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&preflight_path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    assert_eq!(
        installed["preflight"]["activationSupported"], true,
        "real team binding must activate"
    );
    if ordinary {
        let mut instance = node
            .post(
                "/api/uar/agent-instances/v1",
                json!({
            "deploymentBindingId":binding["id"],"profile":"request"}),
                201,
            )
            .await;
        instance["fixtureOwner"] = capabilities["bindingOwnerId"].clone();
        return instance;
    }
    node.post(
        "/api/v1/collaboration/team-instances",
        json!({"commandId":format!("dot04-create-team-{mode}"),
        "deploymentBindingId":binding["id"],"teamDefinition":reference(&team),"input":{"mode":mode},
        "memberSlots":[{"role":"reviewer","count":1}]}),
        201,
    )
    .await
}

fn task<'a>(team: &'a Value, id: &str) -> &'a Value {
    team["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|task| task["id"] == id)
        .unwrap()
}

async fn team_case(
    node: &Node,
    mode: &str,
    legacy: Option<(&Database, &peer::Peer)>,
) -> (String, Value) {
    let mut team = install_definition(node, mode, false).await;
    let id = team["id"].as_str().unwrap().to_owned();
    let path = format!("/api/v1/collaboration/team-instances/{id}");
    for (task_id, depends_on) in [("effect", json!([])), ("dependent", json!(["effect"]))] {
        team = node.post(&format!("{path}/tasks"), json!({"commandId":format!("{mode}-create-{task_id}"),
            "taskId":task_id,"expectedTeamRevision":team["revision"],"title":task_id,"role":"reviewer",
            "input":{"mode":mode},"outputContract":{},"dependsOn":depends_on}), 201).await;
    }
    let member = team["members"][0]["id"].clone();
    team = node.post(&format!("{path}/tasks/effect/claim"), json!({"commandId":format!("{mode}-claim"),
        "expectedTeamRevision":team["revision"],"expectedTaskRevision":task(&team,"effect")["revision"],
        "memberId":member}), 200).await;
    let admitted = node.post(&format!("{path}/tasks/effect/admit"), json!({"commandId":format!("{mode}-admit"),
        "expectedTeamRevision":team["revision"],"expectedTaskRevision":task(&team,"effect")["revision"],
        "memberId":member,"reservation":{"tokens":team_token_capacity(),"costMicrounits":1000000,"elapsedSeconds":120},
        "contextArtifactIds":[]}), 200).await;
    let run = admitted["runId"].as_str().unwrap().to_owned();
    let events = if let Some((database, model)) = legacy {
        let inject = async {
            tokio::time::timeout(WAIT, model.seen.paused.notified())
                .await
                .expect("actual provider paused after successful tool before final prose");
            let store = database.connect().await;
            let receipt = CanonicalToolReceipt::acquire(
                peer::OWNER,
                &run,
                u64::MAX / 2,
                "legacy-process-call",
                "legacy__historical",
                CanonicalReceiptSource::Mcp,
                json!({"historical":"outcome-not-recorded"}),
                std::iter::empty::<(String, Vec<u8>)>(),
                true,
                0,
            )
            .unwrap();
            assert!(receipt.outcome.is_none());
            store.save_canonical_tool_receipt(&receipt).await.unwrap();
            model.seen.resume.notify_one();
        };
        let (events, ()) = tokio::join!(node.run_events(&run, mode), inject);
        events
    } else {
        node.run_events(&run, mode).await
    };
    if mode == "no_tool" {
        assert!(
            !events
                .iter()
                .any(|event| event["event"] == "agui.tool_result"),
            "no-tool control called a tool"
        );
    } else {
        assert_tool_end(&events, TEAM_TOOL, mode);
    }
    let deadline = tokio::time::Instant::now() + WAIT;
    let attempt = loop {
        let execution = node.get(&format!("{path}/execution")).await;
        let attempt = execution["attempts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|attempt| attempt["id"] == admitted["id"])
            .unwrap();
        if attempt["executionOutcome"].is_string() {
            break attempt.clone();
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "team consumer did not settle"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    };
    let held = legacy.is_some() || mode == "unknown";
    let expected = if held {
        "uncertain"
    } else {
        match mode {
            "success" | "no_tool" => "succeeded",
            "application" => "failed",
            _ => unreachable!(),
        }
    };
    let artifacts = node.get(&format!("{path}/artifacts")).await;
    retain_private_json(
        &node.directory.join("team-artifact-evidence.json"),
        &json!({"attempt":attempt,"artifacts":artifacts}),
    );
    assert_eq!(attempt["executionOutcome"], expected);
    let attempt_artifacts = artifacts
        .as_array()
        .unwrap()
        .iter()
        .filter(|artifact| artifact["attemptId"] == admitted["id"])
        .collect::<Vec<_>>();
    assert_eq!(
        attempt_artifacts.len(),
        usize::from(expected == "succeeded"),
        "failed or unconfirmed tool prose published a team output artifact"
    );
    if let Some(artifact) = attempt_artifacts.first() {
        assert_eq!(artifact["content"], attempt["output"]);
        assert_eq!(artifact["ownerId"], attempt["ownerId"]);
    }
    team = node.get(&path).await;
    assert_eq!(
        task(&team, "effect")["status"],
        if held { "blocked" } else { expected }
    );
    assert_eq!(
        task(&team, "dependent")["status"],
        if expected == "succeeded" {
            "ready"
        } else {
            "queued"
        }
    );
    if mode == "application" {
        assert_eq!(
            attempt["stateReason"],
            "team_tool_application_or_transport_failed"
        );
        assert!(
            attempt["output"].is_null(),
            "failed tool final prose promoted team output"
        );
    }
    if held {
        assert_eq!(attempt["effectDisposition"], "uncertain");
        assert!(attempt["output"].is_null(), "unconfirmed tool prose became team output");
        assert!(
            attempt["usage"].is_null(),
            "unknown effect released reservation"
        );
    }
    std::fs::write(
        node.directory.join("team-terminal.json"),
        serde_json::to_vec_pretty(&json!({"team":team,"attempt":attempt})).unwrap(),
    )
    .unwrap();
    assert_eq!(team["ownerId"], attempt["ownerId"]);
    (
        run,
        json!({"team":id,"attempt":admitted["id"],"teamOutcome":expected,"legacyHold":legacy.is_some(),
            "admissionOwner":team["ownerId"]}),
    )
}

async fn instance_case(node: &Node, mode: &str) -> (String, String, Value) {
    let instance = install_definition(node, mode, true).await;
    let id = instance["instanceId"].as_str().unwrap();
    let path = format!("/api/uar/agent-instances/v1/{id}");
    let command = uuid::Uuid::new_v4().to_string();
    node.post(
        &format!("{path}/turns"),
        json!({"commandId":command,"prompt":mode}),
        202,
    )
    .await;
    let deadline = tokio::time::Instant::now() + WAIT;
    let run = loop {
        let view = node.get(&path).await;
        if let Some(run) = view["commands"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["commandId"] == command)
            .and_then(|entry| entry["rootRunId"].as_str())
        {
            break run.to_owned();
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "instance never registered its real root run"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    };
    let events = node.run_events(&run, mode).await;
    let tool = events
        .iter()
        .filter(|event| event["event"] == "agui.tool_result")
        .find_map(|event| {
            event["data"]["name"]
                .as_str()
                .filter(|name| name.ends_with("__effect"))
        })
        .expect("actual instance called the advertised MCP tool")
        .to_owned();
    assert_tool_end(&events, &tool, mode);
    let view = loop {
        let view = node.get(&path).await;
        let status = view["commands"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["commandId"] == command)
            .unwrap()["status"]
            .as_str()
            .unwrap();
        if !matches!(status, "accepted" | "running")
            && (mode == "unknown" || view["lifecycle"] == "dormant")
        {
            break view;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "instance did not settle/passivate; logs at {}",
            node.directory.display()
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    };
    assert_eq!(
        view["recovery"],
        if mode == "unknown" {
            "effect_uncertain"
        } else {
            "ready"
        }
    );
    if mode == "unknown" {
        assert_eq!(
            view["commands"]
                .as_array()
                .unwrap()
                .iter()
                .find(|entry| entry["commandId"] == command)
                .unwrap()["status"],
            "uncertain"
        );
    } else {
        assert_eq!(
            view["lifecycle"], "dormant",
            "known result retained an unsettled effect hold"
        );
    }
    std::fs::write(
        node.directory.join("instance-terminal.json"),
        serde_json::to_vec_pretty(&view).unwrap(),
    )
    .unwrap();
    (
        run,
        tool,
        json!({"instance":id,"owner":instance["fixtureOwner"],"command":command,
        "recovery":view["recovery"],"lifecycle":view["lifecycle"]}),
    )
}

async fn durable_instance(store: &SurrealDbProvider, evidence: &Value) {
    let saved = store
        .load_agent_instance(
            evidence["owner"].as_str().unwrap(),
            WORKSPACE,
            evidence["instance"].as_str().unwrap(),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        serde_json::to_value(saved.recovery).unwrap(),
        evidence["recovery"]
    );
    assert_eq!(
        serde_json::to_value(saved.lifecycle).unwrap(),
        evidence["lifecycle"]
    );
}

async fn concurrent_binary_observer(
    root: &Path,
    writer: &mut Node,
    database: &Database,
    model: &peer::Peer,
    label: &str,
    team: &Value,
) -> Value {
    let path = format!(
        "/api/v1/collaboration/team-instances/{}",
        team["team"].as_str().unwrap()
    );
    let before = writer.get("/api/v1/collaboration/execution-owner").await;
    assert_eq!(
        before["ownsExecution"], true,
        "source runtime must own its actual catalog"
    );
    let requests = model.seen.requests.lock().unwrap().len();
    let effects = peer::effects(&writer.trace).len();
    let mut observer = Node::boot(
        root,
        &format!("{label}-concurrent-observer"),
        database,
        model,
    )
    .await;
    assert!(writer.child.try_wait().unwrap().is_none());
    assert!(observer.child.try_wait().unwrap().is_none());
    assert_ne!(writer.child.id(), observer.child.id());
    let overlap = chrono::Utc::now().to_rfc3339();
    let writer_view = writer.get(&path).await;
    let observer_view = observer.get(&path).await;
    assert!(
        writer_view == observer_view,
        "second actual binary observed different durable team state"
    );
    let writer_execution = writer.get(&format!("{path}/execution")).await;
    let observer_execution = observer.get(&format!("{path}/execution")).await;
    assert!(
        writer_execution == observer_execution,
        "second actual binary observed different durable attempts"
    );
    let persistence = writer.get("/api/config/persistence").await;
    let other_persistence = observer.get("/api/config/persistence").await;
    assert!(
        persistence == other_persistence,
        "concurrent binaries selected different persistence descriptors"
    );
    assert_eq!(persistence["mode"], "remote");
    let first = writer.get("/api/v1/collaboration/capabilities").await;
    let second = observer.get("/api/v1/collaboration/capabilities").await;
    assert_eq!(first["bindingOwnerId"], second["bindingOwnerId"]);
    assert_eq!(first["catalogStorage"], second["catalogStorage"]);
    let writer_ownership = writer.get("/api/v1/collaboration/execution-owner").await;
    let observer_ownership = observer.get("/api/v1/collaboration/execution-owner").await;
    assert_eq!(writer_ownership["ownsExecution"], true);
    assert_eq!(
        observer_ownership["ownsExecution"], false,
        "observer stole writer execution ownership"
    );
    assert!(
        before["claim"] == writer_ownership["claim"]
            && before["claim"] == observer_ownership["claim"]
    );
    assert_eq!(
        second["collaboration"]["activation"]["teamExecution"], false,
        "non-owning runtime advertised active team execution"
    );
    assert_eq!(
        model.seen.requests.lock().unwrap().len(),
        requests,
        "observer replayed a model turn"
    );
    assert_eq!(peer::effects(&writer.trace).len(), effects);
    assert!(peer::effects(&observer.trace).is_empty());
    let evidence = json!({"kind":"concurrent_runtime_binary_observer","writerPid":writer.child.id(),
        "observerPid":observer.child.id(),"writerStartedAt":writer.started_at,"observerStartedAt":observer.started_at,
        "bothAliveObservedAt":overlap,"database":database.name,"namespace":database.namespace,
        "persistenceDescriptorSha256":sha256(&serde_json::to_vec(&persistence).unwrap()),
        "catalogStorage":second["catalogStorage"],"authenticatedOwner":second["bindingOwnerId"],
        "observerOwnsExecution":false,"observerTeamExecutionAdvertised":false,
        "observerEvidenceDirectory":observer.directory});
    std::fs::write(
        writer.directory.join("concurrent-binaries.json"),
        serde_json::to_vec_pretty(&evidence).unwrap(),
    )
    .unwrap();
    observer.stop().await;
    let retained = writer.get("/api/v1/collaboration/execution-owner").await;
    assert_eq!(retained["ownsExecution"], true);
    assert!(
        retained["claim"] == before["claim"],
        "observer shutdown changed writer authority"
    );
    evidence
}

async fn restarted_binary_observer(
    root: &Path,
    database: &Database,
    model: &peer::Peer,
    label: &str,
    team: &Value,
    previous_pid: u32,
    previous_state: &Value,
) -> Value {
    let requests = model.seen.requests.lock().unwrap().len();
    let mut restarted = Node::boot(
        root,
        &format!("{label}-restarted-observer"),
        database,
        model,
    )
    .await;
    assert_ne!(restarted.child.id(), previous_pid);
    let path = format!(
        "/api/v1/collaboration/team-instances/{}",
        team["team"].as_str().unwrap()
    );
    let view = restarted.get(&path).await;
    assert!(
        view == *previous_state,
        "fresh actual binary reinterpreted durable team status"
    );
    let execution = restarted.get(&format!("{path}/execution")).await;
    let attempt = execution["attempts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|attempt| attempt["id"] == team["attempt"])
        .unwrap();
    assert_eq!(attempt["executionOutcome"], team["teamOutcome"]);
    assert_eq!(
        task(&view, "dependent")["status"],
        if team["teamOutcome"] == "succeeded" {
            "ready"
        } else {
            "queued"
        }
    );
    assert_eq!(
        model.seen.requests.lock().unwrap().len(),
        requests,
        "restart replayed a model turn"
    );
    assert!(
        peer::effects(&restarted.trace).is_empty(),
        "restart replayed a tool effect"
    );
    let evidence = json!({"kind":"restarted_runtime_binary_observer","previousPid":previous_pid,
        "restartedPid":restarted.child.id(),"startedAt":restarted.started_at,"database":database.name,
        "teamOutcome":attempt["executionOutcome"],"evidenceDirectory":restarted.directory});
    restarted.stop().await;
    evidence
}

/// Import uniquely identified historical fault records into the actual shared
/// store, then exercise the public recovery consumer. Existing records and the
/// current execution authority retain their identities and outcomes.
async fn historical_recovery_cases(node: &Node, database: &Database, source: &Value) -> Vec<Value> {
    use universal_agent_runtime::uar::{
        domain::team_wait::{AttemptAuthority, AttributedPayload, KernelTeamYield, TeamWait},
        domain::team_execution::TeamProfileRef,
        runtime::thread::{AgentThread, AgentThreadResult},
    };
    let store = database.connect().await;
    let catalog = SurrealCollaborationStorage::new(store.client(), "surrealdb");
    let current = catalog.load_state().await.unwrap();
    let original = current
        .team_execution_attempts
        .values()
        .find(|attempt| attempt.id == source["attempt"].as_str().unwrap())
        .unwrap()
        .clone();
    let team_key = current
        .team_instances
        .iter()
        .find(|(_, team)| team.id == original.team_id)
        .map(|(key, _)| key.clone())
        .unwrap();
    let original_team = &current.team_instances[&team_key];
    let original_task = original_team
        .tasks
        .iter()
        .find(|task| task.id == original.task_id)
        .unwrap();
    assert_eq!(
        original.owner_id,
        source["admissionOwner"].as_str().unwrap()
    );
    let original_evidence = store
        .list_tool_admission_evidence(&original.owner_id)
        .await
        .unwrap();
    retain_private_json(
        &node.directory.join("historical-admission-source.json"),
        &json!({
            "queriedAdmissionOwner":original.owner_id,"run":original.run_id,"evidence":original_evidence
        }),
    );
    let source_facts = original_evidence
        .iter()
        .filter(|record| {
            record.run_id == original.run_id
                && matches!(
                    record.state,
                    ToolAdmissionEvidenceState::ClaimIntent
                        | ToolAdmissionEvidenceState::ApplicationFailed
                )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        source_facts.len(),
        2,
        "historical fixture derives its claim and typed result from actual execution"
    );
    assert_eq!(
        original.execution_fence.as_ref(),
        current.execution_claim.as_ref().map(|claim| &claim.fence)
    );
    let original_receipt = store
        .list_canonical_tool_receipts(peer::OWNER, &original.run_id)
        .await
        .unwrap()
        .into_iter()
        .find(|receipt| receipt.tool == TEAM_TOOL)
        .unwrap();
    assert_eq!(
        original_receipt.outcome,
        Some(super::expected("application"))
    );
    let mut next = current.clone();
    let mut fixtures = Vec::new();
    for label in [
        "known-application-failure",
        "legacy",
        "native-legacy-failure",
        "same-name-unrelated-native",
    ] {
        let legacy = label == "legacy";
        let native = label == "native-legacy-failure";
        let collision = label == "same-name-unrelated-native";
        let mut attempt = original.clone();
        attempt.id = uuid::Uuid::new_v4().to_string();
        attempt.run_id = uuid::Uuid::new_v4().to_string();
        attempt.task_id = format!("historical-{label}");
        attempt.status = "uncertain".into();
        attempt.execution_outcome = Some("uncertain".into());
        attempt.effect_disposition = "uncertain".into();
        attempt.accounting_state = "reserved-unknown".into();
        attempt.usage = None;
        attempt.output = None;
        attempt.state_reason = Some("Imported historical joined-yield recovery fixture".into());
        let mut root = AgentThread::root(
            peer::OWNER.into(),
            original_team.members[0].definition.id.clone(),
            attempt.run_id.clone(),
        )
        .unwrap();
        attempt.root_id = root.thread_id.clone();
        attempt.approval_scope_id = root.thread_id.clone();
        let wait_id = uuid::Uuid::new_v4().to_string();
        let control = KernelTeamYield {
            kind: "team-yield".into(),
            wait_id: wait_id.clone(),
            yielding_attempt_id: attempt.id.clone(),
            durable_receipt_id: format!("historical-wait-{label}"),
            remaining_tool_calls: Vec::new(),
        };
        let registered = store.create_agent_root(peer::OWNER, &root).await.unwrap();
        root.finish_turn(AgentThreadResult::Yielded {
            control: control.clone(),
        })
        .unwrap();
        store
            .update_agent_thread(peer::OWNER, registered.revision, &root)
            .await
            .unwrap();
        let mut task = original_task.clone();
        task.id = attempt.task_id.clone();
        task.title = format!("Historical recovery {label}");
        task.status = "blocked".into();
        task.output = None;
        task.depends_on.clear();
        next.team_instances
            .get_mut(&team_key)
            .unwrap()
            .tasks
            .push(task);
        let authority = AttemptAuthority {
            owner_id: attempt.owner_id.clone(),
            workspace_id: attempt.workspace_id.clone(),
            team_id: attempt.team_id.clone(),
            task_id: attempt.task_id.clone(),
            member_id: attempt.member_id.clone(),
            attempt_id: attempt.id.clone(),
            run_id: attempt.run_id.clone(),
            task_ownership_epoch: attempt.ownership_epoch,
            member_revision: attempt.member_revision,
            binding: TeamProfileRef {
                id: original_team.binding.id.clone(),
                revision: attempt.binding_revision,
            },
            execution_fence: attempt.execution_fence.clone().unwrap(),
        };
        let wait = TeamWait {
            wait_id: wait_id.clone(),
            authority,
            target_task_ids: vec!["dependent".into()],
            predicate: "all-terminal".into(),
            continuation_input: AttributedPayload {
                text: "historical continuation".into(),
                artifact_ids: Vec::new(),
            },
            continuation_reservation: attempt.reservation.clone(),
            state: "yield_requested".into(),
            continuation_attempt_id: None,
            wake_outcomes: Vec::new(),
            reason_code: None,
        };
        next.team_waits.insert(wait_id.clone(), wait);
        next.team_yields.insert(attempt.id.clone(), control);
        let key = format!(
            "{}\u{1f}{}\u{1f}{}\u{1f}{}",
            attempt.owner_id, attempt.workspace_id, attempt.team_id, attempt.id
        );
        next.team_execution_attempts.insert(key, attempt.clone());
        let invocation = uuid::Uuid::new_v4().to_string();
        let admission = uuid::Uuid::new_v4().to_string();
        for original in &source_facts {
            let mut record = (*original).clone();
            record.evidence_id = uuid::Uuid::new_v4().to_string();
            record.root_run_id = attempt.run_id.clone();
            record.run_id = attempt.run_id.clone();
            record.invocation_id = invocation.clone();
            record.admission_id = admission.clone();
            if native || collision {
                record.tool_name = "historical_native_control".into();
                if record.state == ToolAdmissionEvidenceState::ApplicationFailed {
                    record.state = ToolAdmissionEvidenceState::Failed;
                }
                record.admission_owner = if native {
                    universal_agent_runtime::uar::persistence::tool_admission::AdmissionOwner::UarRuntime
                } else {
                    universal_agent_runtime::uar::persistence::tool_admission::AdmissionOwner::PairedHost
                };
            }
            store.save_tool_admission_evidence(&record).await.unwrap();
        }
        let unrelated_invocation = uuid::Uuid::new_v4().to_string();
        let unrelated_admission = uuid::Uuid::new_v4().to_string();
        let unrelated_call = format!("unrelated-native-call-{label}");
        if collision {
            assert_ne!(unrelated_invocation, invocation);
            assert_ne!(unrelated_call, invocation);
            assert_ne!(unrelated_call, unrelated_invocation);
            for original in &source_facts {
                let mut record = (*original).clone();
                record.evidence_id = uuid::Uuid::new_v4().to_string();
                record.root_run_id = attempt.run_id.clone();
                record.run_id = attempt.run_id.clone();
                record.invocation_id = unrelated_invocation.clone();
                record.admission_id = unrelated_admission.clone();
                record.tool_name = "historical_native_control".into();
                record.admission_owner = universal_agent_runtime::uar::persistence::tool_admission::AdmissionOwner::UarRuntime;
                if record.state == ToolAdmissionEvidenceState::ApplicationFailed {
                    record.state = ToolAdmissionEvidenceState::Failed;
                }
                store.save_tool_admission_evidence(&record).await.unwrap();
            }
        }
        let receipt = if legacy || native || collision {
            CanonicalToolReceipt::acquire(
                peer::OWNER,
                &attempt.run_id,
                1,
                format!("historical-{label}-call"),
                if native || collision {
                    "historical_native_control"
                } else {
                    TEAM_TOOL
                },
                if native || collision {
                    CanonicalReceiptSource::Native
                } else {
                    CanonicalReceiptSource::Mcp
                },
                json!({"historical":"outcome-not-recorded"}),
                std::iter::empty::<(String, Vec<u8>)>(),
                true,
                0,
            )
            .unwrap()
        } else {
            let mut receipt = original_receipt.clone();
            receipt.run_id = attempt.run_id.clone();
            receipt.call_id = "historical-known-call".into();
            receipt
        };
        let mut receipt = receipt;
        if collision {
            receipt.call_id = unrelated_call.clone();
        }
        store.save_canonical_tool_receipt(&receipt).await.unwrap();
        if collision {
            let facts = store
                .list_tool_admission_evidence(&original.owner_id)
                .await
                .unwrap();
            let failures = facts
                .iter()
                .filter(|record| {
                    record.run_id == attempt.run_id
                        && record.tool_name == "historical_native_control"
                        && record.state == ToolAdmissionEvidenceState::Failed
                })
                .collect::<Vec<_>>();
            assert_eq!(failures.len(), 2);
            assert!(failures.iter().any(|record| record.invocation_id == invocation
                && record.admission_owner == universal_agent_runtime::uar::persistence::tool_admission::AdmissionOwner::PairedHost));
            assert!(failures.iter().any(|record| record.invocation_id == unrelated_invocation
                && record.admission_owner == universal_agent_runtime::uar::persistence::tool_admission::AdmissionOwner::UarRuntime));
            assert_eq!(receipt.source, CanonicalReceiptSource::Native);
            assert_eq!(receipt.call_id, unrelated_call);
            assert!(receipt.outcome.is_none());
        }
        fixtures.push(
            json!({"boundary":"team_recovery_process","mode":label,"run":attempt.run_id,
            "team":attempt.team_id,"attempt":attempt.id,"wait":wait_id,"database":database.name,
            "historicalFixture":true,"claimedInvocationId":invocation,
            "unrelatedNativeInvocationId":if collision { Some(unrelated_invocation) } else { None },
            "unrelatedCanonicalCallId":if collision { Some(unrelated_call) } else { None }}),
        );
    }
    next.generation += 1;
    next.team_instances.get_mut(&team_key).unwrap().revision += 1;
    assert!(
        catalog
            .compare_and_swap(current.generation, &next)
            .await
            .unwrap(),
        "historical import raced a catalog writer"
    );
    let path = format!("/api/v1/collaboration/team-instances/{}", original.team_id);
    let team = node.get(&path).await;
    let recovered = node.post(&format!("{path}/recover"), json!({"commandId":"historical-recovery-control",
        "expectedTeamRevision":team["revision"],"reason":"Controlled historical joined-yield receipt acceptance"}), 200).await;
    assert_eq!(
        recovered,
        json!([]),
        "historical recovery launched a new model turn"
    );
    let durable = catalog.load_state().await.unwrap();
    for fixture in &fixtures {
        let attempt = durable
            .team_execution_attempts
            .values()
            .find(|attempt| attempt.id == fixture["attempt"].as_str().unwrap())
            .unwrap();
        let wait = &durable.team_waits[fixture["wait"].as_str().unwrap()];
        if matches!(
            fixture["mode"].as_str(),
            Some("legacy" | "same-name-unrelated-native")
        ) {
            assert_eq!(attempt.status, "uncertain");
            assert_eq!(attempt.effect_disposition, "uncertain");
            assert_eq!(
                wait.state, "yield_requested",
                "legacy MCP absence promoted a joined yield"
            );
        } else {
            assert_eq!(
                attempt.status, "yielded",
                "explicit application failure was treated as unknown"
            );
            assert_eq!(attempt.effect_disposition, "confirmed");
            assert_eq!(wait.state, "waiting");
        }
        assert!(
            attempt.usage.is_none(),
            "historical recovery released unknown provider accounting"
        );
        assert!(wait.continuation_attempt_id.is_none());
    }
    std::fs::write(
        node.directory.join("historical-recovery.json"),
        serde_json::to_vec_pretty(&fixtures).unwrap(),
    )
    .unwrap();
    fixtures
}

fn assert_tool_end(events: &[Value], tool: &str, mode: &str) {
    let ends = events
        .iter()
        .filter(|event| event["event"] == "agui.tool_result" && event["data"]["name"] == tool)
        .collect::<Vec<_>>();
    assert_eq!(ends.len(), 1, "actual RunManager ToolEnd cardinality");
    assert_eq!(ends[0]["data"]["success"], mode == "success");
    let output: Value = serde_json::from_str(ends[0]["data"]["content"].as_str().unwrap()).unwrap();
    if mode == "success" {
        peer::assert_content(&output, mode);
    } else {
        assert_eq!(
            output["outcome"],
            serde_json::to_value(super::expected(mode)).unwrap()
        );
        if mode == "application" {
            peer::assert_content(&output["result"], mode);
        } else {
            assert!(output["result"].is_null());
        }
    }
    if mode == "unknown" {
        assert!(events.iter().any(|event| event["event"] == "agui.error"
            && event["data"]["code"] == "TOOL_OUTCOME_UNKNOWN"));
    }
}

fn retain_private_json(path: &Path, value: &Value) {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).unwrap();
    serde_json::to_writer_pretty(&mut file, value).unwrap();
    file.sync_all().unwrap();
}

async fn durable_result(
    database: &Database,
    run: &str,
    tool: &str,
    mode: &str,
    team: &Value,
    evidence_directory: &Path,
) -> SurrealDbProvider {
    let store = database.connect().await;
    let receipts = store
        .list_canonical_tool_receipts(peer::OWNER, run)
        .await
        .unwrap();
    // Team effects use the authenticated binding partition. Canonical history
    // and ordinary/instance admission retain their existing subject partition.
    let admission_owner = if team["team"].is_string() {
        team["admissionOwner"]
            .as_str()
            .expect("actual team owner was retained")
    } else {
        peer::OWNER
    };
    let evidence = store
        .list_tool_admission_evidence(admission_owner)
        .await
        .unwrap();
    let canonical = receipts
        .iter()
        .map(|receipt| {
            json!({
                "owner":receipt.owner_id,"run":receipt.run_id,"callId":receipt.call_id,
                "tool":receipt.tool,"source":receipt.source,"outcome":receipt.outcome
            })
        })
        .collect::<Vec<_>>();
    retain_private_json(
        &evidence_directory.join("durable-admission-evidence.json"),
        &json!({
            "queriedAdmissionOwner":admission_owner,"canonicalOwner":peer::OWNER,
            "run":run,"tool":tool,"canonical":canonical,"evidence":evidence
        }),
    );
    if mode == "no_tool" {
        assert!(
            receipts.is_empty(),
            "no-tool control acquired a tool receipt"
        );
        assert!(
            !evidence.iter().any(|record| record.run_id == run),
            "no-tool control claimed an effect"
        );
    } else {
        let receipts = receipts
            .iter()
            .filter(|receipt| receipt.tool == tool)
            .collect::<Vec<_>>();
        assert_eq!(
            receipts.len(),
            1,
            "binary canonical result persisted across shutdown"
        );
        assert_eq!(receipts[0].outcome, Some(super::expected(mode)));
        let value = receipts[0].typed_value.as_ref().unwrap();
        if mode == "success" {
            peer::assert_content(value, mode);
        }
        if mode == "application" {
            peer::assert_content(&value["result"], mode);
        }
        let terminal = evidence
            .iter()
            .filter(|record| {
                record.run_id == run && record.state == super::evidence(super::expected(mode))
            })
            .count();
        assert_eq!(
            terminal, 1,
            "binary durable admission agrees with canonical result"
        );
        if mode != "success" {
            assert!(!evidence.iter().any(|record| record.run_id == run
                && record.state == ToolAdmissionEvidenceState::Succeeded));
        }
    }
    if let Some(id) = team["team"].as_str() {
        let catalog = SurrealCollaborationStorage::new(store.client(), "surrealkv");
        let state = catalog.load_state().await.unwrap();
        let attempt = state
            .team_execution_attempts
            .values()
            .find(|attempt| attempt.run_id == run)
            .unwrap();
        assert_eq!(
            attempt.execution_outcome.as_deref(),
            team["teamOutcome"].as_str()
        );
        if mode != "unknown" {
            let root = store
                .load_agent_thread(peer::OWNER, &attempt.root_id)
                .await
                .unwrap()
                .unwrap();
            assert!(
                matches!(root.thread.result,
                Some(universal_agent_runtime::uar::runtime::thread::AgentThreadResult::Completed { ref output })
                    if output == "post-tool-control"),
                "team must consume the real Completed actor result"
            );
        }
        let saved = state
            .team_instances
            .values()
            .find(|team| team.id == id)
            .unwrap();
        let dependent = saved
            .tasks
            .iter()
            .find(|task| task.id == "dependent")
            .unwrap();
        assert_eq!(
            dependent.status,
            if team["teamOutcome"] == "succeeded" {
                "ready"
            } else {
                "queued"
            }
        );
        if team["legacyHold"] == true {
            assert!(
                store
                    .list_canonical_tool_receipts(peer::OWNER, run)
                    .await
                    .unwrap()
                    .iter()
                    .any(|receipt| receipt.call_id == "legacy-process-call"
                        && receipt.outcome.is_none())
            );
            assert_eq!(attempt.effect_disposition, "uncertain");
            assert!(attempt.usage.is_none());
        }
    }
    store
}

/// Reopen the exact scoped fixture backend after its owning server has exited.
pub async fn scoped_outcome_store(
    root: &Path,
    shared: bool,
    mode: &str,
) -> std::sync::Arc<dyn PersistenceLayer> {
    std::sync::Arc::new(
        Database::configuration(root, shared, &format!("scoped-{mode}"))
            .connect()
            .await,
    )
}

pub fn assert_scoped_effect_count(root: &Path, mode: &str) {
    let trace = root.join(format!("binary-scoped-{mode}/effects.jsonl"));
    assert_eq!(
        peer::effects(&trace).len(),
        usize::from(mode != "predispatch"),
        "observing durable scoped outcomes replayed an effect"
    );
}

/// Enter scoped orchestration through authenticated public server admission.
/// A public ResolvedTurn constructor cannot manufacture its verified owner.
pub async fn scoped_outcome_run(
    root: &Path,
    shared: bool,
    mode: &str,
) -> (String, std::sync::Arc<dyn PersistenceLayer>) {
    let label = format!("scoped-{mode}");
    let database = Database::configuration(root, shared, &label);
    let model = peer::Peer::start(root).await;
    let mut node = Node::boot(root, &label, &database, &model).await;
    let mut artifact = universal_agent_runtime::uar::defaults::default_agent();
    artifact.id = format!("dot04-scoped-{mode}");
    artifact.memory.kb.enabled = false;
    artifact.policy.tools.allow = vec![peer::TOOL.into()];
    let admitted = node
        .post(
            "/api/uar/runs",
            json!({"artifact":artifact,"input":mode}),
            200,
        )
        .await;
    let run = admitted["run_id"].as_str().unwrap().to_owned();
    // run_events retains the original stream privately before assertions.
    let events = node.run_events(&run, mode).await;
    assert_tool_end(&events, peer::TOOL, mode);
    assert_eq!(
        model.seen.requests.lock().unwrap().len(),
        if mode == "unknown" { 1 } else { 2 },
        "unknown result triggered a model retry turn"
    );
    assert_eq!(
        peer::effects(&node.trace).len(),
        usize::from(mode != "predispatch"),
        "external effect was replayed or predispatch rejection executed"
    );
    node.stop().await;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    while !node.peer_pids().is_empty() {
        assert!(
            tokio::time::Instant::now() < deadline,
            "scoped server MCP children did not exit"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let path = node.directory.join("scoped-entrypoint.json");
    std::fs::write(
        &path,
        serde_json::to_vec_pretty(&json!({
            "kind":"authenticated_scoped_runtime_binary","pid":node.child.id(),
            "startedAt":node.started_at,"run":run,"mode":mode,"database":database.name,
            "route":"POST /api/uar/runs","events":"run-events.json",
            "serverExited":node.child.try_wait().unwrap().is_some(),"mcpChildrenExited":true
        }))
        .unwrap(),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    drop(node);
    assert_scoped_effect_count(root, mode);
    (run, scoped_outcome_store(root, shared, mode).await)
}

/// Execute actual startup-composed ordinary runs and durable team consumers.
/// The embedded database is separate from the parent fixture's open store.
pub async fn production_process_cases(root: &Path, shared: bool) -> Vec<Value> {
    let mut rows = Vec::new();
    for (boundary, modes) in [
        (
            "server_process",
            &["success", "application", "predispatch", "unknown"][..],
        ),
        (
            "team_process",
            &["success", "application", "unknown", "no_tool"][..],
        ),
    ] {
        for mode in modes {
            let label = format!("{boundary}-{mode}");
            let database = Database::configuration(root, shared, &label);
            let model = peer::Peer::start(root).await;
            let mut node = Node::boot(root, &label, &database, &model).await;
            let (run, team) = if boundary == "team_process" {
                team_case(&node, mode, None).await
            } else {
                let mut artifact = universal_agent_runtime::uar::defaults::default_agent();
                artifact.id = format!("dot04-process-{mode}");
                artifact.memory.kb.enabled = false;
                artifact.policy.tools.allow = vec![peer::TOOL.into()];
                let admitted = node
                    .post(
                        "/api/uar/runs",
                        json!({"artifact":artifact,"input":mode}),
                        200,
                    )
                    .await;
                let run = admitted["run_id"].as_str().unwrap().to_owned();
                let events = node.run_events(&run, mode).await;
                assert_tool_end(&events, peer::TOOL, mode);
                (run, Value::Null)
            };
            assert_eq!(
                model.seen.requests.lock().unwrap().len(),
                if matches!(*mode, "unknown" | "no_tool") {
                    1
                } else {
                    2
                },
                "outcome changed model continuation cardinality"
            );
            assert_eq!(
                peer::effects(&node.trace).len(),
                usize::from(!matches!(*mode, "predispatch" | "no_tool")),
                "effect executed twice or before predispatch rejection"
            );
            let tool = if boundary == "team_process" {
                TEAM_TOOL
            } else {
                peer::TOOL
            };
            if shared && boundary == "team_process" && *mode == "application" {
                rows.extend(historical_recovery_cases(&node, &database, &team).await);
            }
            let concurrent = if shared && boundary == "team_process" {
                concurrent_binary_observer(root, &mut node, &database, &model, &label, &team).await
            } else {
                Value::Null
            };
            let previous_pid = node.child.id();
            let previous_state = if boundary == "team_process" {
                node.get(&format!(
                    "/api/v1/collaboration/team-instances/{}",
                    team["team"].as_str().unwrap()
                ))
                .await
            } else {
                Value::Null
            };
            node.stop().await;
            let restarted = if boundary == "team_process" {
                restarted_binary_observer(
                    root,
                    &database,
                    &model,
                    &label,
                    &team,
                    previous_pid,
                    &previous_state,
                )
                .await
            } else {
                Value::Null
            };
            durable_result(&database, &run, tool, mode, &team, &node.directory).await;
            let outcome = if *mode == "no_tool" {
                Value::Null
            } else {
                serde_json::to_value(super::expected(mode)).unwrap()
            };
            rows.push(json!({"boundary":boundary,"mode":mode,"run":run,"tool":tool,
                "outcome":outcome,"teamEvidence":team,"concurrentBinaryEvidence":concurrent,
                "restartedBinaryEvidence":restarted,"database":database.name,"evidenceDirectory":node.directory}));
        }
    }
    if shared {
        let database = Database::configuration(root, true, "team-process-legacy-hold");
        let model = peer::Peer::start(root).await;
        model
            .seen
            .pause_after_tool
            .store(true, std::sync::atomic::Ordering::Release);
        let mut node = Node::boot(root, "team-process-legacy-hold", &database, &model).await;
        let (run, team) = team_case(&node, "success", Some((&database, &model))).await;
        assert_eq!(model.seen.requests.lock().unwrap().len(), 2);
        assert_eq!(peer::effects(&node.trace).len(), 1);
        let concurrent = concurrent_binary_observer(
            root,
            &mut node,
            &database,
            &model,
            "team-process-legacy-hold",
            &team,
        )
        .await;
        let previous_pid = node.child.id();
        let previous_state = node
            .get(&format!(
                "/api/v1/collaboration/team-instances/{}",
                team["team"].as_str().unwrap()
            ))
            .await;
        node.stop().await;
        let restarted = restarted_binary_observer(
            root,
            &database,
            &model,
            "team-process-legacy-hold",
            &team,
            previous_pid,
            &previous_state,
        )
        .await;
        durable_result(&database, &run, TEAM_TOOL, "success", &team, &node.directory).await;
        rows.push(json!({"boundary":"team_process","mode":"legacy-hold","run":run,"tool":TEAM_TOOL,
            "teamEvidence":team,"concurrentBinaryEvidence":concurrent,"restartedBinaryEvidence":restarted,
            "database":database.name,"evidenceDirectory":node.directory}));
    }
    for mode in ["success", "application", "unknown"] {
        let label = format!("instance-process-{mode}");
        let database = Database::configuration(root, shared, &label);
        let model = peer::Peer::start(root).await;
        let mut node = Node::boot(root, &label, &database, &model).await;
        let (run, tool, instance) = instance_case(&node, mode).await;
        assert_eq!(
            model.seen.requests.lock().unwrap().len(),
            if mode == "unknown" { 1 } else { 2 }
        );
        assert_eq!(peer::effects(&node.trace).len(), 1);
        node.stop().await;
        let store =
            durable_result(&database, &run, &tool, mode, &Value::Null, &node.directory).await;
        durable_instance(&store, &instance).await;
        drop(store);
        rows.push(
            json!({"boundary":"instance_process","mode":mode,"run":run,"tool":tool,
            "outcome":super::expected(mode),"instanceEvidence":instance,"database":database.name,
            "evidenceDirectory":node.directory}),
        );
    }
    rows
}
