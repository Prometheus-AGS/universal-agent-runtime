//! Selected functional fixtures obtain identity only through the actual host
//! middleware and an externally provisioned loopback gateway. No credential is
//! read, copied, constructed, attached to a request, or returned as JSON here.

use axum::{Extension, Router, http::StatusCode, routing::get};
use clap::Parser as _;
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
    time::Duration,
};
use tokio_util::sync::CancellationToken;
use universal_agent_runtime::{
    AppState,
    config::Cli,
    config_manager::ConfigManager,
    uar::security::claims::{CredentialPrincipal, UserContext},
};

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DatabaseProfile {
    pub inspection_url: String,
    pub namespace: String,
    pub name: String,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NodeProfile {
    pub backend_bind: SocketAddr,
    pub grpc_bind: SocketAddr,
    pub gateway_url: String,
    pub workspace: String,
    pub product_config_path: PathBuf,
    pub product_env_path: Option<PathBuf>,
    pub model_peer_bind: SocketAddr,
    pub database: String,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Profile {
    pub root: PathBuf,
    pub nodes: BTreeMap<String, NodeProfile>,
    pub databases: BTreeMap<String, DatabaseProfile>,
    #[serde(default)]
    pub llm_profiles: BTreeMap<String, PathBuf>,
}

/// Opaque product configuration flows through the existing product loader and
/// driver factories; functional code never inspects provider credential fields.
pub fn llm_config(
    suite: &str,
    shared: bool,
    lane: &str,
) -> universal_agent_runtime::config::LlmConfig {
    let profile = profile(suite, shared);
    let path = profile
        .llm_profiles
        .get(lane)
        .expect("external controlled-wire product profile");
    assert!(path.is_absolute());
    let cli = Cli::try_parse_from([
        "fixture-driver",
        "--config",
        path.to_str().expect("opaque product profile path"),
    ])
    .expect("actual product CLI contract");
    universal_agent_runtime::config::load_llm_config(cli)
        .expect("product loads opaque provider profile")
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Manifest {
    dedicated: bool,
    suites: BTreeMap<String, BTreeMap<String, Profile>>,
}

pub fn loopback_url(value: &str) -> reqwest::Url {
    let url = reqwest::Url::parse(value).expect("explicit external loopback fixture URL");
    assert!(matches!(url.scheme(), "http" | "https" | "ws" | "wss"));
    assert!(
        url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none()
    );
    assert!(
        url.host_str()
            .and_then(|host| host
                .trim_matches(['[', ']'])
                .parse::<std::net::IpAddr>()
                .ok())
            .is_some_and(|host| host.is_loopback())
    );
    url
}

pub fn profile(suite: &str, shared: bool) -> Profile {
    let path = std::env::var_os("UAR_EXTERNAL_IDENTITY_FIXTURE")
        .expect("externally provisioned nonsecret fixture profile manifest");
    let manifest: Manifest =
        serde_json::from_slice(&std::fs::read(path).expect("nonsecret fixture manifest"))
            .expect("only declared nonsecret fixture metadata is accepted");
    assert!(
        manifest.dedicated,
        "selected functional candidate must be exclusively owned"
    );
    let profile = manifest
        .suites
        .get(suite)
        .and_then(|suite| suite.get(if shared { "shared" } else { "local" }))
        .expect("external fixture profile exists")
        .clone();
    assert!(profile.root.is_absolute());
    for node in profile.nodes.values() {
        assert!(node.backend_bind.ip().is_loopback() && node.backend_bind.port() != 0);
        assert!(node.grpc_bind.ip().is_loopback() && node.grpc_bind.port() != 0);
        assert!(node.model_peer_bind.ip().is_loopback() && node.model_peer_bind.port() != 0);
        assert_ne!(node.backend_bind, node.grpc_bind);
        let gateway = loopback_url(&node.gateway_url);
        assert!(matches!(gateway.scheme(), "http" | "https"));
        assert_eq!(
            gateway.path(),
            "/",
            "gateway profile is an origin, not an ambiguous path prefix"
        );
        assert!(node.product_config_path.is_absolute());
        assert!(
            node.product_env_path
                .as_ref()
                .is_none_or(|path| path.is_absolute())
        );
        assert!(!node.workspace.is_empty());
        assert!(profile.databases.contains_key(&node.database));
    }
    for database in profile.databases.values() {
        if let Some(path) = database.inspection_url.strip_prefix("surrealkv://") {
            assert!(
                Path::new(path).is_absolute() && Path::new(path).starts_with(&profile.root),
                "local inspection store must be owned"
            );
        } else {
            loopback_url(&database.inspection_url);
        }
        assert!(!database.namespace.is_empty() && !database.name.is_empty());
    }
    profile
}

struct HostThread {
    stop: CancellationToken,
    thread: Option<std::thread::JoinHandle<anyhow::Result<()>>>,
}
impl Drop for HostThread {
    fn drop(&mut self) {
        self.stop.cancel();
        if let Some(thread) = self.thread.take() {
            thread
                .join()
                .expect("owned identity host exits")
                .expect("production host cleanup");
        }
    }
}

/// Capture only a genuinely verified in-process value produced by production
/// auth_middleware. The trusted-host route cannot construct or deserialize one.
pub async fn user(suite: &str, shared: bool, expected_subject: &str) -> UserContext {
    static CAPTURED: OnceLock<Mutex<BTreeMap<String, UserContext>>> = OnceLock::new();
    let key = format!("{suite}:{shared}:{expected_subject}");
    let captured = CAPTURED.get_or_init(|| Mutex::new(BTreeMap::new()));
    if let Some(user) = captured.lock().expect("capture cache").get(&key).cloned() {
        return user;
    }
    let profile = profile(suite, shared);
    let node = profile
        .nodes
        .get("identity-capture")
        .expect("external identity-capture host stage")
        .clone();
    // In-process configuration is loaded only by the actual product loader;
    // an env-file is consumed by the product binary entrypoint, never this fixture.
    assert!(
        node.product_env_path.is_none(),
        "capture host profile must be self-contained product configuration"
    );
    let cli = Cli::try_parse_from([
        "fixture-host",
        "--config",
        node.product_config_path
            .to_str()
            .expect("opaque config path"),
    ])
    .expect("existing product CLI configuration contract");
    let config = ConfigManager::load_without_watcher(cli)
        .await
        .expect("product loads external host profile");
    let listener =
        std::net::TcpListener::bind(node.backend_bind).expect("owned fixed host listener");
    listener.set_nonblocking(true).unwrap();
    let grpc = std::net::TcpListener::bind(node.grpc_bind).expect("owned fixed gRPC listener");
    grpc.set_nonblocking(true).unwrap();
    let path = format!("/__fixture/owner-capture/{}", uuid::Uuid::new_v4());
    let (identity_tx, identity_rx) = tokio::sync::oneshot::channel();
    let identity_tx = Arc::new(Mutex::new(Some(identity_tx)));
    let routes: Router<AppState> = Router::new().route(
        &path,
        get(move |Extension(user): Extension<UserContext>| {
            let sender = identity_tx.clone();
            async move {
                if CredentialPrincipal::from_verified_context(&user).is_err() {
                    return StatusCode::UNAUTHORIZED;
                }
                let sender = sender.lock().expect("owned identity channel").take();
                match sender {
                    Some(sender) => {
                        if sender.send(user).is_ok() {
                            StatusCode::NO_CONTENT
                        } else {
                            StatusCode::CONFLICT
                        }
                    }
                    _ => StatusCode::CONFLICT,
                }
            }
        }),
    );
    let stop = CancellationToken::new();
    let thread_stop = stop.clone();
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
    // Same trusted-host current-thread embedding shape as the existing live
    // integration host, with its unchanged production-sized root stack.
    let thread = std::thread::Builder::new()
        .name("functional-external-identity-host".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("owned host runtime")
                .block_on(async move {
                    universal_agent_runtime::server::start_server_with_host_routes(
                        config,
                        tokio::net::TcpListener::from_std(listener).unwrap(),
                        tokio::net::TcpListener::from_std(grpc).unwrap(),
                        None,
                        ready_tx,
                        Some(thread_stop.clone()),
                        Some(thread_stop),
                        routes,
                    )
                    .await
                })
        })
        .expect("owned actual production host thread");
    let host = HostThread {
        stop,
        thread: Some(thread),
    };
    let ready = tokio::time::timeout(Duration::from_secs(180), ready_rx)
        .await
        .expect("host readiness bound")
        .expect("production host ready");
    assert_eq!(ready, node.backend_bind);
    let url = loopback_url(&node.gateway_url)
        .join(&path)
        .expect("actual external gateway capture route");
    let response = reqwest::Client::new()
        .get(url)
        .header("x-uar-workspace-id", &node.workspace)
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("credential-free external gateway request");
    assert_eq!(response.status().as_u16(), 204);
    let user = tokio::time::timeout(Duration::from_secs(30), identity_rx)
        .await
        .expect("verified host callback bound")
        .expect("production middleware supplied actual identity");
    assert_eq!(user.user_id, expected_subject);
    CredentialPrincipal::from_verified_context(&user).expect("host-captured verified principal");
    drop(host);
    captured
        .lock()
        .expect("capture cache")
        .insert(key, user.clone());
    user
}
