//! Real process/HTTP collaborators for principal credential acceptance.

use axum::{Json, Router, extract::State, http::HeaderMap, response::IntoResponse, routing::post};
use reqwest::Method;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    time::Duration,
};
use universal_agent_runtime::uar::security::claims::CredentialPrincipal;
use universal_agent_runtime::uar::security::credentials::{CredentialOwnerBinding, CredentialTenancyPolicy, ProviderCredentialPolicyConfig};
pub const INSTALLATION_ID: &str = "00000000-0000-0000-0000-000000000001";
pub fn credential_policy() -> CredentialTenancyPolicy { CredentialTenancyPolicy::from_config(ProviderCredentialPolicyConfig { installation_id: Some(INSTALLATION_ID.to_owned()), ..Default::default() }).unwrap() }

pub const TEST_KEY: &[u8; 32] = b"0123456789abcdef0123456789abcdef";
const JWT_SECRET: &str = "credential-principal-integration-jwt";

pub struct Database {
    _local: Option<tempfile::TempDir>,
    pub url: String,
    pub namespace: String,
    pub database: String,
    pub user: Option<String>,
    pub password: Option<String>,
}

impl Database {
    pub fn local() -> Self {
        let local = match std::env::var_os("UAR_PHASE_RESOURCE_ROOT") {
            Some(root) => tempfile::Builder::new().prefix("native-credential-").tempdir_in(root).unwrap(),
            None => tempfile::tempdir().unwrap(),
        };
        Self {
            url: format!("surrealkv://{}", local.path().join("database").display()),
            _local: Some(local),
            namespace: "credential_test".into(),
            database: "credentials".into(),
            user: None,
            password: None,
        }
    }

    pub async fn shared() -> Self {
        Self {
            _local: None,
            url: std::env::var("UAR_CREDENTIAL_TEST_SURREAL_URL")
                .expect("explicit durable SurrealDB URL required"),
            namespace: format!("credential_{}", uuid::Uuid::new_v4().simple()),
            database: "credentials".into(),
            user: std::env::var("UAR_CREDENTIAL_TEST_SURREAL_USER").ok(),
            password: std::env::var("UAR_CREDENTIAL_TEST_SURREAL_PASS").ok(),
        }
    }

    pub async fn connect(&self) -> surrealdb::Surreal<surrealdb::engine::any::Any> {
        let db = surrealdb::engine::any::connect(&self.url).await.unwrap();
        if let (Some(user), Some(password)) = (&self.user, &self.password) {
            db.signin(surrealdb::opt::auth::Root {
                username: user.clone(),
                password: password.clone(),
            })
            .await
            .unwrap();
        }
        db.use_ns(&self.namespace)
            .use_db(&self.database)
            .await
            .unwrap();
        db
    }
}

pub struct Node {
    child: Mutex<Child>,
    pub base: String,
    pub workspace: tempfile::TempDir,
}

impl Node {
    pub async fn boot(
        database: &Database,
        provider: &Provider,
        issuer: Option<&str>,
        subjects: &[String],
        mappings: Option<
            Vec<universal_agent_runtime::uar::security::credentials::LegacyCredentialOwnerMapping>,
        >,
    ) -> Self {
        let workspace = tempfile::tempdir().unwrap();
        for dir in ["home", "work/policies", "no-skills"] {
            std::fs::create_dir_all(workspace.path().join(dir)).unwrap();
        }
        for name in [
            "default.cedar",
            "tool-approval.cedar",
            "skill-mutation.cedar",
        ] {
            std::fs::copy(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("policies")
                    .join(name),
                workspace.path().join("work/policies").join(name),
            )
            .unwrap();
        }
        let port = std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let grpc = std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let mut security = json!({"jwt_required":true,"jwt_secret":JWT_SECRET,"settings_mutation_auth_required":false});
        if let Some(issuer) = issuer {
            security["deployment_profile"] = json!("remote");
            security["jwt_algorithm"] = json!("HS256");
            security["jwt_issuer"] = json!(issuer);
            security["jwt_audience"] = json!("credential-runtime");
            security["workspace_authorities"]=json!(subjects.iter().flat_map(|subject| ["tenant-a","tenant-b"].map(|tenant|
                json!({"issuer":issuer,"subject":subject,"tenant_id":tenant,"workspace_id":"credential-workspace"}))).collect::<Vec<_>>());
        }
        security["provider_credentials"] = json!({"mode":"single_owner","installation_id":INSTALLATION_ID});
        let verifier_config = serde_json::from_value(security.clone()).unwrap();
        initialize_jwt_verifier(&verifier_config).await;
        let config = json!({"security":security,"resilience":{"rate_limit_enabled":false},
            "persistence":{"provider":"surreal","database_url":database.url,"surreal_ns":database.namespace,
                "surreal_db":database.database,"surreal_user":database.user,"surreal_pass":database.password},
            "llm":{"model":"openai/gpt-5.4-mini","base_url":provider.url,"api_key":"credential-test-house"},
            "server":{"host":"127.0.0.1","port":port,"grpc_port":grpc,"shutdown_timeout_secs":5,
                "ingest_directory":workspace.path().join("ingest")},
            "memory":{"enabled":false}});
        let path = workspace.path().join("config.yaml");
        std::fs::write(&path, serde_json::to_vec_pretty(&config).unwrap()).unwrap();
        let stdout = std::fs::File::create(workspace.path().join("stdout.log")).unwrap();
        let stderr = std::fs::File::create(workspace.path().join("stderr.log")).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_universal-agent-runtime"));
        command
            .arg("--config")
            .arg(path)
            .current_dir(workspace.path().join("work"))
            .env_clear()
            .env("PATH", std::env::var_os("PATH").unwrap_or_default())
            .env("HOME", workspace.path().join("home"))
            .env("UAR_BUILTIN_SKILLS_DIR", workspace.path().join("no-skills"))
            // The runtime's cwd is the per-test workspace, so the default
            // relative models directory would not resolve.
            .env(
                "UAR_MODELS_DIR",
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/uar/runtime/matching/models"),
            )
            .env(
                "CREDENTIAL_ENCRYPTION_KEY",
                std::str::from_utf8(TEST_KEY).unwrap(),
            )
            .env("UAR_REMOTE_SURREAL_DURABILITY_ATTESTED", "1")
            .stdin(Stdio::null())
            .stdout(stdout)
            .stderr(stderr);
        if let Some(mappings) = mappings {
            let file = workspace.path().join("operator-owner-mappings.json");
            std::fs::write(&file, serde_json::to_vec(&mappings).unwrap()).unwrap();
            command.env("CREDENTIAL_OWNER_MAPPING_FILE", file);
        }
        let child = command.spawn().expect("start actual runtime binary");
        let node = Self {
            child: Mutex::new(child),
            base: format!("http://127.0.0.1:{port}"),
            workspace,
        };
        let deadline = tokio::time::Instant::now() + Duration::from_secs(180);
        loop {
            if let Ok(response) = reqwest::get(format!("{}/health", node.base)).await
                && response.status().is_success()
            {
                break;
            }
            assert!(
                node.child.lock().unwrap().try_wait().unwrap().is_none(),
                "runtime exited before readiness; logs at {}",
                node.workspace.path().display()
            );
            assert!(
                tokio::time::Instant::now() < deadline,
                "runtime readiness timeout; logs at {}",
                node.workspace.path().display()
            );
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        node
    }

    fn retain_failure(&self, stage: &str, payload: &str) {
        let mut child = self.child.lock().unwrap();
        let status = child.try_wait().unwrap();
        std::fs::write(self.workspace.path().join("run-failure.log"), payload).unwrap();
        std::fs::write(
            self.workspace.path().join("run-failure-state.json"),
            serde_json::to_vec_pretty(&json!({
                "stage": stage,
                "child_pid": child.id(),
                "child_exit_status": status.map(|status| status.to_string()),
                "child_still_running": status.is_none()
            }))
            .unwrap(),
        )
        .unwrap();
    }

    pub async fn stop(mut self) {
        #[cfg(unix)]
        assert!(
            Command::new("/bin/kill")
                .arg("-TERM")
                .arg(self.child.get_mut().unwrap().id().to_string())
                .status()
                .unwrap()
                .success()
        );
        #[cfg(not(unix))]
        self.child.get_mut().unwrap().kill().unwrap();
        let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
        while self.child.get_mut().unwrap().try_wait().unwrap().is_none() {
            assert!(
                tokio::time::Instant::now() < deadline,
                "runtime shutdown timed out"
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        let child = self.child.get_mut().unwrap();
        if child.try_wait().ok().flatten().is_none() {
            let _ = child.kill();
            let _ = child.wait();
        }
        if std::thread::panicking() {
            // Failed process logs remain local evidence; no credential-bearing
            // output is copied into the test runner's diagnostic stream.
            let saved = std::mem::replace(&mut self.workspace, tempfile::tempdir().unwrap()).keep();
            eprintln!(
                "credential process evidence retained at {}",
                saved.display()
            );
        }
    }
}

pub(super) async fn initialize_jwt_verifier(config: &universal_agent_runtime::config::SecurityConfig) {
    // UAR must acquire its process provider before the fixture signs a token.
    let status = universal_agent_runtime::uar::security::middleware::authenticate_bearer_context(
        config,
        "not-a-token",
    )
    .await
    .expect_err("malformed initialization token must be rejected");
    assert_eq!(status, axum::http::StatusCode::UNAUTHORIZED);
}

pub fn token(issuer: Option<&str>, tenant: Option<&str>, subject: &str) -> String {
    let mut claims =
        json!({"sub":subject,"roles":["user"],"exp":chrono::Utc::now().timestamp()+3600});
    if let Some(issuer) = issuer {
        claims["iss"] = json!(issuer);
        claims["aud"] = json!("credential-runtime");
    }
    if let Some(tenant) = tenant {
        claims["tenant_id"] = json!(tenant);
    }
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
    .unwrap()
}

pub async fn authenticated(
    issuer: Option<&str>,
    tenant: Option<&str>,
    subject: &str,
) -> CredentialOwnerBinding {
    let config:universal_agent_runtime::config::SecurityConfig=serde_json::from_value(json!({
        "jwt_required":true,"jwt_secret":JWT_SECRET,"jwt_issuer":issuer,"jwt_audience":issuer.map(|_|"credential-runtime"),
        "settings_mutation_auth_required":false })).unwrap();
    initialize_jwt_verifier(&config).await;
    let context = universal_agent_runtime::uar::security::middleware::authenticate_bearer_context(
        &config,
        &token(issuer, tenant, subject),
    )
    .await
    .unwrap();
    credential_policy().bind_principal(&CredentialPrincipal::from_verified_context(&context).unwrap()).unwrap()
}

pub async fn call(
    node: &Node,
    credential: &str,
    method: Method,
    suffix: &str,
    body: Option<Value>,
    expected: u16,
) -> Value {
    let mut request = reqwest::Client::new()
        .request(method, format!("{}/api/uar/credentials{suffix}", node.base))
        .bearer_auth(credential)
        .header("x-uar-workspace-id", "credential-workspace");
    if let Some(body) = body {
        request = request.json(&body);
    }
    let response = request.send().await.unwrap();
    assert_eq!(
        response.status().as_u16(),
        expected,
        "credential HTTP {suffix}"
    );
    let bytes = response.bytes().await.unwrap();
    assert!(
        !bytes
            .windows(b"api_key_encrypted".len())
            .any(|window| window == b"api_key_encrypted"),
        "ciphertext leaked via API"
    );
    if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    }
}

pub struct Provider {
    pub url: String,
    seen: Arc<Mutex<Vec<(String, String)>>>,
    task: tokio::task::JoinHandle<()>,
}

fn auth_digest(value: &str) -> String {
    Sha256::digest(value.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

async fn completion(
    State(seen): State<Arc<Mutex<Vec<(String, String)>>>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> axum::response::Response {
    let prompt = body["messages"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|message| message["role"] == "user")
        .next_back()
        .and_then(|message| message["content"].as_str())
        .unwrap_or_default()
        .to_owned();
    let digest = auth_digest(
        headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default(),
    );
    seen.lock().unwrap().push((prompt, digest));
    let chunk = |delta: Value, finish: Value| {
        json!({"id":"credential-test","object":"chat.completion.chunk","created":1,
        "model":"gpt-5.4-mini","choices":[{"index":0,"delta":delta,"finish_reason":finish}]})
    };
    if body["stream"] == true {
        let output = format!(
            "data: {}\n\ndata: {}\n\ndata: [DONE]\n\n",
            chunk(
                json!({"role":"assistant","content":"credential-used"}),
                Value::Null
            ),
            chunk(json!({}), json!("stop"))
        );
        ([("content-type", "text/event-stream")], output).into_response()
    } else {
        Json(json!({"id":"credential-test","object":"chat.completion","created":1,"model":"gpt-5.4-mini",
            "choices":[{"index":0,"message":{"role":"assistant","content":"credential-used"},"finish_reason":"stop"}],
            "usage":{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2}})).into_response()
    }
}

impl Provider {
    pub async fn start() -> Self {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/v1", listener.local_addr().unwrap());
        let app = Router::new()
            .route("/v1/chat/completions", post(completion))
            .route("/chat/completions", post(completion))
            .with_state(seen.clone());
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        Self { url, seen, task }
    }

    pub async fn prove_run(&self, node: &Node, credential: &str, expected_key: &str, prompt: &str) {
        let client = reqwest::Client::new();
        let mut artifact =
            serde_json::to_value(universal_agent_runtime::uar::defaults::default_agent()).unwrap();
        artifact["id"] = json!(format!("credential-test-{}", uuid::Uuid::new_v4()));
        let created = client
            .post(format!("{}/api/uar/runs", node.base))
            .bearer_auth(credential)
            .header("x-uar-workspace-id", "credential-workspace")
            .json(&json!({"artifact":artifact,"input":prompt}))
            .send()
            .await;
        if let Err(error) = &created {
            node.retain_failure("run-create-request", &error.to_string());
        }
        let created = created.unwrap();
        assert!(created.status().is_success(), "credential run rejected");
        let body: Value = created.json().await.unwrap();
        let run = body["run_id"].as_str().unwrap();
        let response = client
            .get(format!("{}/api/uar/runs/{run}/stream", node.base))
            .bearer_auth(credential)
            .header("x-uar-workspace-id", "credential-workspace")
            .send()
            .await;
        if let Err(error) = &response {
            node.retain_failure("run-stream-request", &error.to_string());
        }
        let mut response = response.unwrap();
        assert!(response.status().is_success());
        let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
        let mut stream = String::new();
        while !stream.contains("event: agui.done") {
            if stream.contains("event: agui.error") {
                node.retain_failure("run-stream-error", &stream);
            }
            assert!(
                !stream.contains("event: agui.error"),
                "credential run failed; process logs retained on assertion failure"
            );
            let chunk = tokio::time::timeout_at(deadline, response.chunk())
                .await
                .unwrap()
                .unwrap()
                .expect("run terminal event");
            stream.push_str(&String::from_utf8_lossy(&chunk));
        }
        let expected = auth_digest(&format!("Bearer {expected_key}"));
        assert!(
            self.seen
                .lock()
                .unwrap()
                .iter()
                .any(|(seen_prompt, digest)| seen_prompt == prompt && digest == &expected),
            "upstream did not receive the selected credential"
        );
    }
}

impl Drop for Provider {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl Provider {
    /// Only account labels/counts leave the private fixture; raw headers never do.
    pub fn contact_count(&self) -> usize { self.seen.lock().unwrap().len() }
    pub fn saw_key(&self, key: &str) -> bool {
        let digest = auth_digest(&format!("Bearer {key}"));
        self.seen.lock().unwrap().iter().any(|(_, observed)| observed == &digest)
    }
}

/// Obtain actual middleware proof for an embedded host's production request API.
pub async fn authenticated_context(issuer: &str, tenant: &str, subject: &str) -> universal_agent_runtime::uar::security::claims::UserContext {
    let config: universal_agent_runtime::config::SecurityConfig = serde_json::from_value(json!({
        "jwt_required":true,"jwt_secret":JWT_SECRET,"jwt_issuer":issuer,
        "jwt_audience":"credential-runtime","settings_mutation_auth_required":false
    })).unwrap();
    initialize_jwt_verifier(&config).await;
    universal_agent_runtime::uar::security::middleware::authenticate_bearer_context(
        &config, &token(Some(issuer),Some(tenant),subject)).await.unwrap()
}
