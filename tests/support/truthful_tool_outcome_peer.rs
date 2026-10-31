//! Real protocol peers with deterministic failure injection; no provider inference claim.
use axum::{
    Json, Router,
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashMap},
    ffi::OsString,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};
use universal_agent_runtime::{
    mcp::{
        binding_cache::{McpBindingCache, McpBindingEnvironment, McpBindingRequest},
        catalog::{McpCatalog, ServerAuthentication, ServerDefinition, ServerSource},
        config::{McpConfig, McpServerEntry},
        preflight::McpPreflight,
        projection::{McpProjectionScope, McpServerProjection},
        registry::McpRegistry,
        runtime::{ConfiguredMcpConnector, McpRuntimeManager},
    },
    uar::{
        domain::policy::{PolicyResolutionInput, PolicyUniverse, resolve_run_policy},
        runtime::actor::messages::ActorOwner,
        security::claims::{UserClaims, UserContext},
    },
};

pub const TOOL: &str = "outcomes__effect";
pub const OWNER: &str = "dot04-outcome-owner";
pub const SECRET: &str = "dot04-synthetic-provider-key";

pub async fn user() -> UserContext {
    let secret = "dot04-signed-principal-fixture";
    let config = universal_agent_runtime::config::SecurityConfig {
        provider_credentials: Default::default(),
        deployment_profile: Default::default(), jwt_algorithm: None,
        workspace_authorities: Vec::new(),
        skill_required_roles: Default::default(),
        api_key_delegable_roles: universal_agent_runtime::config::SecurityConfig::default_api_key_delegable_roles(),
        api_key_admin_principals: Vec::new(), trusted_host_principals: Vec::new(),
        jwt_required: true, jwt_secret: secret.to_owned().into(), jwks_url: None,
        jwt_issuer: None, jwt_audience: None, jwt_validate_nbf: true,
        settings_mutation_auth_required: true, settings_admin_key: None,
        local_loopback_owner: false,
    };
    // Initialize crypto through the production verifier before signing, and
    // obtain identity only by verifying the resulting fixture credential.
    assert!(matches!(universal_agent_runtime::uar::security::middleware::authenticate_bearer_context(
        &config, "dot04-malformed-synthetic-token").await, Err(status) if status == axum::http::StatusCode::UNAUTHORIZED),
        "production verifier must reject malformed fixture credentials before signing");
    let token = jsonwebtoken::encode(&jsonwebtoken::Header::default(), &UserClaims {
        sub: OWNER.into(), name: None, roles: Some(vec!["user".into()]),
        tenant_id: None, uar_instance_id: None,
        exp: (chrono::Utc::now().timestamp() + 3600) as usize,
    }, &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes())).unwrap();
    universal_agent_runtime::uar::security::middleware::authenticate_bearer_context(&config, &token)
        .await.expect("actual signed fixture principal verification")
}

pub fn configuration(trace: &Path) -> McpConfig {
    McpConfig {
        mcp_servers: HashMap::from([(
            "outcomes".into(),
            McpServerEntry::Stdio {
                command: "/usr/bin/python3".into(),
                args: vec![
                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("tests/fixtures/truthful_tool_outcome_mcp.py")
                        .display()
                        .to_string(),
                    trace.display().to_string(),
                ],
                env: HashMap::new(),
                sandboxed: false,
                preapproved_tools: Default::default(),
            },
        )]),
    }
}

pub async fn scoped(trace: &Path) -> (McpRegistry, McpRuntimeManager, McpPreflight) {
    let config = configuration(trace);
    let registry = McpRegistry::from_config(&config).await.unwrap();
    let definition = Arc::new(
        ServerDefinition::new(
            "outcomes".into(),
            ServerSource::Global,
            config.mcp_servers["outcomes"].clone(),
            true,
            ServerAuthentication::NotRequired,
        )
        .unwrap(),
    );
    let environment = Arc::new(
        McpBindingEnvironment::new(
            trace.parent().unwrap().to_path_buf(),
            BTreeMap::from([(OsString::from("PATH"), OsString::from("/usr/bin:/bin"))]),
        )
        .unwrap(),
    );
    let owner = ActorOwner::from_verified_context(&user().await).unwrap();
    let request = McpBindingRequest::new(owner.clone(), definition.clone(), environment.clone());
    let runtime = McpRuntimeManager::new(
        McpBindingCache::default(),
        Arc::new(ConfiguredMcpConnector::default()),
        Duration::from_secs(15),
        Duration::from_secs(15),
    )
    .unwrap();
    let catalog = McpCatalog::from_definitions([definition.as_ref().clone()]).unwrap();
    let policy = resolve_run_policy(PolicyResolutionInput {
        universe: PolicyUniverse {
            tools: [TOOL.into()].into(),
            mcp_servers: ["outcomes".into()].into(),
            ..PolicyUniverse::default()
        },
        ..PolicyResolutionInput::default()
    });
    let projection =
        McpServerProjection::resolve(&catalog, &policy, &McpProjectionScope::default()).unwrap();
    // prepare() establishes the actual transport and discovery; preflight preserves it.
    runtime.prepare(Arc::new(request)).await.unwrap();
    let preflight = runtime
        .preflight(&projection, &owner, &environment)
        .await
        .unwrap();
    (registry, runtime, preflight)
}

pub fn effects(trace: &Path) -> Vec<Value> {
    std::fs::read_to_string(trace)
        .unwrap_or_default()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

pub fn assert_content(value: &Value, mode: &str) {
    assert_eq!(value["isError"], mode == "application");
    assert_eq!(value["content"][0]["text"], "fixture-response-control:λ");
    assert_eq!(value["content"][1]["type"], "image");
    assert_eq!(value["content"][1]["data"], "iVBORw0KGgo=");
    assert_eq!(
        value["content"][2]["resource"]["text"],
        "embedded-resource-control"
    );
    assert_eq!(
        value["structuredContent"],
        json!({"mode":mode,"nested":{"control":42}})
    );
    assert_eq!(value["_meta"], json!({"fixture":"metadata-control"}));
}

#[derive(Default)]
pub struct Seen {
    pub requests: Mutex<Vec<Value>>,
    request_log: PathBuf,
    pub pause_after_tool: std::sync::atomic::AtomicBool,
    pub paused: tokio::sync::Notify,
    pub resume: tokio::sync::Notify,
}

pub struct Peer {
    pub url: String,
    pub seen: Arc<Seen>,
    task: tokio::task::JoinHandle<()>,
}

impl Drop for Peer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl Peer {
    pub async fn start(root: &Path) -> Self {
        let log_directory = root.join("liter-requests");
        std::fs::create_dir_all(&log_directory).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&log_directory, std::fs::Permissions::from_mode(0o700))
                .unwrap();
        }
        let request_log = log_directory.join(format!("{}.jsonl", uuid::Uuid::new_v4()));
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        options.open(&request_log).unwrap();
        let seen = Arc::new(Seen {
            request_log,
            ..Seen::default()
        });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/v1", listener.local_addr().unwrap());
        let app = Router::new()
            .route("/v1/chat/completions", post(completion))
            .with_state(seen.clone());
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        Self { url, seen, task }
    }
}

async fn completion(State(seen): State<Arc<Seen>>, Json(body): Json<Value>) -> Response {
    {
        use std::io::Write;
        let mut requests = seen.requests.lock().unwrap();
        requests.push(body.clone());
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&seen.request_log)
            .unwrap();
        serde_json::to_writer(&mut file, &body).unwrap();
        file.write_all(b"\n").unwrap();
        file.sync_all().unwrap();
    }
    let messages = body["messages"].as_array().unwrap();
    fn find_mode(value: &Value) -> Option<String> {
        match value {
            Value::String(mode)
                if [
                    "success",
                    "application",
                    "predispatch",
                    "unknown",
                    "no_tool",
                ]
                .contains(&mode.as_str()) =>
            {
                Some(mode.clone())
            }
            Value::Object(values) => values.values().find_map(find_mode),
            Value::Array(values) => values.iter().find_map(find_mode),
            _ => None,
        }
    }
    fn parsed_mode(value: &Value) -> Option<String> {
        find_mode(value).or_else(|| {
            value
                .as_str()
                .and_then(|text| serde_json::from_str::<Value>(text).ok())
                .and_then(|parsed| find_mode(&parsed))
        })
    }
    let mode = messages
        .iter()
        .rev()
        .filter(|message| message["role"] == "user")
        .find_map(|message| parsed_mode(&message["content"]))
        .or_else(|| {
            messages.iter().rev().find_map(|message| {
                message["tool_calls"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .find_map(|call| parsed_mode(&call["function"]["arguments"]))
            })
        })
        .or_else(|| {
            messages
                .iter()
                .rev()
                .filter(|message| message["role"] == "tool")
                .find_map(|message| parsed_mode(&message["content"]))
        })
        .expect("controlled input or retained tool invocation names a scenario");
    let chunk = |delta: Value, finish: Value| {
        format!(
            "data: {}\n\n",
            json!({
                "id":"dot04-response", "object":"chat.completion.chunk", "created":1,
                "model":"gpt-4o-mini", "choices":[{"index":0,"delta":delta,"finish_reason":finish}]
            })
        )
    };
    let mut frames = String::new();
    if mode == "no_tool" || messages.iter().any(|message| message["role"] == "tool") {
        if seen
            .pause_after_tool
            .swap(false, std::sync::atomic::Ordering::AcqRel)
        {
            seen.paused.notify_one();
            seen.resume.notified().await;
        }
        frames.push_str(&chunk(
            json!({"role":"assistant","content":"post-tool-control"}),
            Value::Null,
        ));
        frames.push_str(&chunk(json!({}), json!("stop")));
    } else {
        let tool = body["tools"]
            .as_array()
            .unwrap()
            .iter()
            .find_map(|tool| {
                tool["function"]["name"]
                    .as_str()
                    .filter(|name| name.ends_with("__effect"))
            })
            .expect("actual effect tool was advertised");
        frames.push_str(&chunk(json!({"role":"assistant","tool_calls":[{"index":0,"id":"dot04-call","type":"function","function":{"name":tool,"arguments":json!({"mode":mode}).to_string()}}]}), Value::Null));
        frames.push_str(&chunk(json!({}), json!("tool_calls")));
    }
    frames.push_str("data: [DONE]\n\n");
    ([("content-type", "text/event-stream")], frames).into_response()
}
