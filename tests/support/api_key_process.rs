//! Actual runtime processes and isolated durable authority stores for dot-03.

use reqwest::{Method, Url};
use serde_json::{Value, json};
use std::{
    io::Write,
    path::PathBuf,
    process::{Child, ChildStdin, Command, Stdio},
    sync::Mutex,
    time::Duration,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::watch,
};

pub const JWT_SECRET: &str = "durable-api-key-fixture-jwt";
pub const ISSUER: &str = "durable-api-key-fixture-issuer";

pub struct Database {
    _local: Option<tempfile::TempDir>,
    pub provider: &'static str,
    pub url: String,
    pub namespace: String,
    pub database: String,
    pub user: Option<String>,
    pub password: Option<String>,
}

impl Database {
    pub fn local() -> Self {
        let local = tempfile::tempdir().unwrap();
        Self {
            url: format!("surrealkv://{}", local.path().join("keys").display()),
            _local: Some(local),
            provider: "surreal",
            namespace: "api_key_test".into(),
            database: "keys".into(),
            user: None,
            password: None,
        }
    }

    pub fn shared() -> Self {
        Self {
            _local: None,
            provider: "surreal",
            url: std::env::var("UAR_API_KEY_TEST_SURREAL_URL")
                .expect("isolated durable SurrealDB URL required"),
            namespace: format!("api_key_{}", uuid::Uuid::new_v4().simple()),
            database: "keys".into(),
            user: std::env::var("UAR_API_KEY_TEST_SURREAL_USER").ok(),
            password: std::env::var("UAR_API_KEY_TEST_SURREAL_PASS").ok(),
        }
    }

    #[cfg(feature = "in-memory-backend")]
    pub fn development_memory() -> Self {
        Self {
            _local: None,
            provider: "memory",
            url: String::new(),
            namespace: String::new(),
            database: String::new(),
            user: None,
            password: None,
        }
    }

    #[cfg(feature = "postgres-backend")]
    pub fn postgres() -> Self {
        Self {
            _local: None,
            provider: "postgres",
            url: std::env::var("UAR_API_KEY_TEST_POSTGRES_URL")
                .expect("isolated Postgres database URL required"),
            namespace: String::new(),
            database: String::new(),
            user: None,
            password: None,
        }
    }

    pub async fn connect(&self) -> surrealdb::Surreal<surrealdb::engine::any::Any> {
        let db = surrealdb::engine::any::connect(&self.url)
            .await
            .expect("fixture DB connection");
        if let (Some(user), Some(password)) = (&self.user, &self.password) {
            db.signin(surrealdb::opt::auth::Root {
                username: user.clone(),
                password: password.clone(),
            })
            .await
            .expect("fixture DB authentication");
        }
        db.use_ns(&self.namespace)
            .use_db(&self.database)
            .await
            .expect("fixture DB selection");
        db
    }
}

pub struct Node {
    child: Mutex<Child>,
    workspace: tempfile::TempDir,
    pub base: String,
    pub grpc: String,
    secrets: Mutex<Vec<String>>,
    _stdin: Option<ChildStdin>,
    launch_token: Option<String>,
}

impl Node {
    pub async fn boot(database: &Database, issuer: Option<&str>) -> Self {
        Self::boot_mode(database, issuer, false).await
    }

    pub async fn sidecar(database: &Database) -> Self {
        Self::boot_mode(database, None, true).await
    }

    async fn boot_mode(database: &Database, issuer: Option<&str>, sidecar: bool) -> Self {
        let workspace = tempfile::tempdir().unwrap();
        for directory in ["home", "work/policies", "no-skills"] {
            std::fs::create_dir_all(workspace.path().join(directory)).unwrap();
        }
        for file in [
            "default.cedar",
            "tool-approval.cedar",
            "skill-mutation.cedar",
        ] {
            std::fs::copy(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("policies")
                    .join(file),
                workspace.path().join("work/policies").join(file),
            )
            .unwrap();
        }
        let port = free_port();
        let grpc = free_port();
        let mut security = json!({"jwt_required":true,"jwt_secret":JWT_SECRET,"settings_mutation_auth_required":false});
        if let Some(issuer) = issuer {
            security["deployment_profile"] = json!("remote");
            security["jwt_algorithm"] = json!("HS256");
            security["jwt_issuer"] = json!(issuer);
            security["jwt_audience"] = json!("api-key-runtime");
            security["workspace_authorities"] = json!(["tenant-a", "tenant-b"].map(|tenant|
                json!({"issuer":issuer,"subject":"same","tenant_id":tenant,"workspace_id":"key-workspace"})));
        }
        let config = json!({"security":security,"resilience":{"rate_limit_enabled":false},
            "persistence":{"provider":database.provider,"database_url":database.url,
                "surreal_ns":database.namespace,"surreal_db":database.database,
                "surreal_user":database.user,"surreal_pass":database.password},
            "llm":{"model":"openai/gpt-5.4-mini","api_key":"unused-fixture-provider-key","base_url":"http://127.0.0.1:9"},
            "server":{"host":"127.0.0.1","port":port,"grpc_port":grpc,"shutdown_timeout_secs":5},
            "memory":{"enabled":false}});
        let path = workspace.path().join("config.yaml");
        std::fs::write(&path, serde_json::to_vec_pretty(&config).unwrap()).unwrap();
        let binary = if sidecar {
            env!("CARGO_BIN_EXE_uar-sidecar")
        } else {
            env!("CARGO_BIN_EXE_universal-agent-runtime")
        };
        let mut child = Command::new(binary)
            .arg("--config")
            .arg(path)
            .arg("--port")
            .arg(port.to_string())
            .current_dir(workspace.path().join("work"))
            .env_clear()
            .env("PATH", std::env::var_os("PATH").unwrap_or_default())
            .env("HOME", workspace.path().join("home"))
            .env("UAR_BUILTIN_SKILLS_DIR", workspace.path().join("no-skills"))
            .env("UAR_REMOTE_SURREAL_DURABILITY_ATTESTED", "1")
            .env("UAR_SECURITY__JWT_REQUIRED", "true")
            .stdin(if sidecar {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(std::fs::File::create(workspace.path().join("stdout.log")).unwrap())
            .stderr(std::fs::File::create(workspace.path().join("stderr.log")).unwrap())
            .spawn()
            .expect("actual runtime binary starts");
        let mut secrets = vec![JWT_SECRET.to_owned()];
        let launch_token = sidecar.then(|| {
            format!(
                "{}{}",
                uuid::Uuid::new_v4().simple(),
                uuid::Uuid::new_v4().simple()
            )
        });
        let mut stdin = child.stdin.take();
        if let Some(token) = &launch_token {
            secrets.push(token.clone());
            writeln!(stdin.as_mut().expect("sidecar launch channel"), "{token}").unwrap();
            stdin.as_mut().unwrap().flush().unwrap();
        }
        if !database.url.is_empty() {
            secrets.push(database.url.clone());
        }
        if let Some(password) = &database.password {
            secrets.push(password.clone());
        }
        let node = Self {
            child: Mutex::new(child),
            workspace,
            base: format!("http://127.0.0.1:{port}"),
            grpc: format!("http://127.0.0.1:{grpc}"),
            secrets: Mutex::new(secrets),
            _stdin: stdin,
            launch_token,
        };
        let deadline = tokio::time::Instant::now() + Duration::from_secs(180);
        loop {
            let mut probe = reqwest::Client::new().get(format!("{}/health", node.base));
            if let Some(token) = &node.launch_token {
                probe = probe.bearer_auth(token);
            }
            if probe
                .send()
                .await
                .is_ok_and(|response| response.status().is_success())
            {
                break;
            }
            assert!(
                node.child.lock().unwrap().try_wait().unwrap().is_none(),
                "runtime exited before readiness"
            );
            assert!(
                tokio::time::Instant::now() < deadline,
                "runtime readiness timeout"
            );
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        // The production gRPC listener is required by this fixture, not skipped.
        if !sidecar {
            let _ = universal_agent_runtime::uar::api::a2a::grpc::pb::agent_service_client::AgentServiceClient::connect(node.grpc.clone())
                .await.expect("production gRPC listener required");
        }
        node
    }

    pub fn redact(&self, secret: &str) {
        self.secrets.lock().unwrap().push(secret.to_owned());
    }

    pub async fn stop(mut self) {
        assert!(
            Command::new("/bin/kill")
                .arg("-TERM")
                .arg(self.child.get_mut().unwrap().id().to_string())
                .status()
                .unwrap()
                .success()
        );
        let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
        while self.child.get_mut().unwrap().try_wait().unwrap().is_none() {
            assert!(
                tokio::time::Instant::now() < deadline,
                "runtime shutdown timeout"
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    fn sanitize_logs(&self) {
        for file in ["stdout.log", "stderr.log"] {
            let path = self.workspace.path().join(file);
            if let Ok(mut content) = std::fs::read_to_string(&path) {
                for secret in self.secrets.lock().unwrap().iter() {
                    content = content.replace(secret, "[redacted]");
                }
                // Hashes never belong in evidence, including unexpected SDK errors.
                content = content
                    .lines()
                    .map(|line| {
                        if line.contains("$argon2") {
                            "[redacted hash-bearing log line]"
                        } else {
                            line
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                let _ = std::fs::write(path, content);
            }
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
        self.sanitize_logs();
        if std::thread::panicking() {
            let path = std::mem::replace(&mut self.workspace, tempfile::tempdir().unwrap()).keep();
            eprintln!(
                "sanitized API-key process evidence retained at {}",
                path.display()
            );
        }
    }
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

pub fn token(issuer: Option<&str>, tenant: &str, subject: &str) -> String {
    signed(
        json!({"sub":subject,"tenant_id":tenant,"roles":["user"],"exp":chrono::Utc::now().timestamp()+3600}),
        issuer,
    )
}

pub fn signed(mut claims: Value, issuer: Option<&str>) -> String {
    if let Some(issuer) = issuer {
        claims["iss"] = json!(issuer);
        claims["aud"] = json!("api-key-runtime");
    }
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
    .expect("synthetic fixture token")
}

pub async fn http(
    node: &Node,
    credential: &str,
    raw: bool,
    method: Method,
    path: &str,
    body: Option<Value>,
    status: u16,
) -> Value {
    node.redact(credential);
    let mut request = reqwest::Client::builder()
        .timeout(Duration::from_secs(75))
        .build()
        .unwrap()
        .request(method, format!("{}{path}", node.base))
        .header("x-uar-workspace-id", "key-workspace");
    request = if raw {
        request.header("x-api-key", credential)
    } else {
        request.bearer_auth(credential)
    };
    if let Some(body) = body {
        request = request.json(&body);
    }
    let response = request
        .send()
        .await
        .expect("actual HTTP outcome required within fixture deadline");
    assert_eq!(
        response.status().as_u16(),
        status,
        "API-key HTTP status for {path}"
    );
    let bytes = response.bytes().await.unwrap();
    assert!(
        !bytes
            .windows(b"key_hash".len())
            .any(|part| part == b"key_hash"),
        "hash field leaked through API"
    );
    let value: Value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    for key in ["raw_key", "token"] {
        if let Some(secret) = value[key].as_str() {
            node.redact(secret);
        }
    }
    value
}

pub async fn grpc(node: &Node, token: &str, expected: tonic::Code) {
    grpc_request(node, token, expected, None).await;
}

pub async fn issue_sidecar_grant(node: &Node, key: Option<&str>) -> String {
    let launch = node
        .launch_token
        .as_ref()
        .expect("actual sidecar host channel");
    let mut request = reqwest::Client::builder()
        .timeout(Duration::from_secs(75))
        .build()
        .unwrap()
        .post(format!("{}/api/uar/delegation-grants", node.base))
        .bearer_auth(launch)
        .json(&json!({"workspace_ids":["key-workspace"],"operations":["discovery"]}));
    request = if let Some(key) = key {
        node.redact(key);
        request.header("x-api-key", key)
    } else {
        request.header("x-uar-principal", "fixture-host-operator")
    };
    let response = request.send().await.expect("real sidecar grant issuance");
    assert_eq!(
        response.status().as_u16(),
        201,
        "sidecar grant issuance status"
    );
    let value: Value = response.json().await.expect("grant response shape");
    let token = value["token"]
        .as_str()
        .expect("grant credential exists")
        .to_owned();
    node.redact(&token);
    token
}

pub async fn issuer_grpc_after_outage(node: &Node, token: &str) {
    // An empty artifact selector is rejected after authentication and before
    // task storage. This distinguishes issuer admission from DB task lookup.
    grpc_request(node, token, tonic::Code::InvalidArgument, Some("")).await;
}

async fn grpc_request(node: &Node, token: &str, expected: tonic::Code, agent: Option<&str>) {
    use universal_agent_runtime::uar::api::a2a::grpc::pb::{
        GetTaskRequest, agent_service_client::AgentServiceClient,
    };
    node.redact(token);
    let mut client = AgentServiceClient::connect(node.grpc.clone())
        .await
        .expect("actual gRPC client");
    let mut request = tonic::Request::new(GetTaskRequest {
        task_id: "fixture-nonexistent-task".to_owned(),
    });
    request
        .metadata_mut()
        .insert("authorization", format!("Bearer {token}").parse().unwrap());
    request
        .metadata_mut()
        .insert("x-uar-workspace-id", "key-workspace".parse().unwrap());
    if let Some(agent) = agent {
        request
            .metadata_mut()
            .insert("x-uar-agent-id", agent.parse().unwrap());
    }
    let result = tokio::time::timeout(Duration::from_secs(75), client.task_get(request))
        .await
        .expect("actual gRPC outcome deadline");
    assert_eq!(
        result
            .err()
            .expect("nonexistent task must not succeed")
            .code(),
        expected,
        "gRPC admission outcome"
    );
}

/// Owns a real TCP forwarding boundary. Turning it off closes established
/// connections as well as new ones; neither runtime can retain a warm channel.
pub struct OutageProxy {
    offline: watch::Sender<bool>,
    task: tokio::task::JoinHandle<()>,
    pub url: String,
}

impl OutageProxy {
    pub async fn start(upstream: &str) -> Self {
        let parsed = Url::parse(upstream).expect("explicit fixture URL");
        assert!(
            matches!(parsed.scheme(), "ws" | "postgres" | "postgresql"),
            "fixture proxy requires non-TLS local DB transport"
        );
        let destination = format!(
            "{}:{}",
            parsed.host_str().expect("fixture host"),
            parsed
                .port_or_known_default()
                .unwrap_or(if parsed.scheme() == "ws" { 80 } else { 5432 })
        );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let mut proxy_url = parsed;
        proxy_url.set_host(Some("127.0.0.1")).unwrap();
        proxy_url.set_port(Some(port)).unwrap();
        let (offline, receiver) = watch::channel(false);
        let task = tokio::spawn(async move {
            loop {
                let Ok((mut downstream, _)) = listener.accept().await else {
                    break;
                };
                let mut receiver = receiver.clone();
                if *receiver.borrow() {
                    continue;
                }
                let destination = destination.clone();
                tokio::spawn(async move {
                    let Ok(mut upstream) = TcpStream::connect(destination).await else {
                        return;
                    };
                    tokio::select! {
                        _ = tokio::io::copy_bidirectional(&mut downstream, &mut upstream) => {},
                        _ = receiver.changed() => {},
                    }
                });
            }
        });
        Self {
            offline,
            task,
            url: proxy_url.to_string(),
        }
    }

    pub fn disconnect(&self) {
        self.offline.send_replace(true);
    }
}

impl Drop for OutageProxy {
    fn drop(&mut self) {
        self.offline.send_replace(true);
        self.task.abort();
    }
}
