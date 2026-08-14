use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use axum::{
    Router,
    body::Body,
    extract::{
        FromRequestParts, Request, State,
        ws::{Message as AxMessage, WebSocket, WebSocketUpgrade},
    },
    http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::any,
};
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use jsonwebtoken::{EncodingKey, Header, crypto::rust_crypto, encode};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, protocol::Message as WsMessage};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(
    name = "uar-jwt-proxy",
    about = "Reverse proxy that injects an HS256 JWT into every request bound for UAR."
)]
struct Cli {
    /// Address to bind the proxy to.
    #[arg(long, env = "PROXY_LISTEN", default_value = "127.0.0.1:8088")]
    listen: SocketAddr,

    /// Path to a UAR config.yaml. If omitted, the proxy looks for $CONFIG_FILE,
    /// then ./config.yaml, then ~/.uar/config.yaml.
    #[arg(long, env = "CONFIG_FILE")]
    config: Option<PathBuf>,

    /// Upstream UAR base URL. Overrides the config file value when set.
    #[arg(long, env = "PROXY_UPSTREAM")]
    upstream: Option<String>,

    /// HS256 signing secret. Overrides `security.jwt_secret` from the config file.
    #[arg(long, env = "UAR_SECURITY__JWT_SECRET")]
    secret: Option<String>,

    /// `sub` claim baked into the minted token.
    #[arg(long, env = "PROXY_JWT_SUB", default_value = "dev")]
    sub: String,

    /// Optional `name` claim.
    #[arg(long, env = "PROXY_JWT_NAME", default_value = "Local Dev")]
    name: String,

    /// Optional `roles` claim (comma-separated).
    #[arg(long, env = "PROXY_JWT_ROLES", default_value = "admin,user")]
    roles: String,

    /// Token TTL in seconds.
    #[arg(long, env = "PROXY_JWT_TTL_SECS", default_value_t = 3600)]
    ttl_secs: u64,
}

#[derive(Debug, Default, Deserialize)]
struct UarConfig {
    #[serde(default)]
    server: Option<ServerSection>,
    #[serde(default)]
    security: Option<SecuritySection>,
}

#[derive(Debug, Deserialize)]
struct ServerSection {
    #[serde(default)]
    host: Option<String>,
    #[serde(default)]
    port: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct SecuritySection {
    #[serde(default)]
    jwt_secret: Option<String>,
}

#[derive(Serialize)]
struct Claims<'a> {
    sub: &'a str,
    name: &'a str,
    roles: Vec<&'a str>,
    exp: usize,
}

struct AppState {
    upstream: String,
    client: reqwest::Client,
    encoding_key: EncodingKey,
    sub: String,
    name: String,
    roles: Vec<String>,
    ttl_secs: u64,
}

static UAR_PROVIDER: OnceLock<Result<(), ()>> = OnceLock::new();

fn ensure_rustcrypto_provider() -> Result<()> {
    match UAR_PROVIDER.get_or_init(|| {
        rust_crypto::DEFAULT_PROVIDER
            .install_default()
            .map_err(|_already_initialized| ())
    }) {
        Ok(()) => Ok(()),
        Err(()) => anyhow::bail!(
            "jsonwebtoken process provider was initialized before uar-jwt-proxy could install RustCrypto"
        ),
    }
}

impl AppState {
    fn mint_token(&self) -> Result<String> {
        let exp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs()
            .saturating_add(self.ttl_secs) as usize;
        let claims = Claims {
            sub: &self.sub,
            name: &self.name,
            roles: self.roles.iter().map(String::as_str).collect(),
            exp,
        };
        Ok(encode(&Header::default(), &claims, &self.encoding_key)?)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    ensure_rustcrypto_provider().context("initializing JWT crypto provider")?;

    let cli = Cli::parse();

    let (cfg, cfg_path) = load_config(cli.config.as_deref())?;

    let upstream = cli
        .upstream
        .clone()
        .or_else(|| upstream_from_config(&cfg))
        .unwrap_or_else(|| "http://127.0.0.1:1906".to_string());

    let secret = cli
        .secret
        .clone()
        .or_else(|| cfg.security.as_ref().and_then(|s| s.jwt_secret.clone()))
        .context(
            "JWT secret not provided: pass --secret, set UAR_SECURITY__JWT_SECRET, or put \
             security.jwt_secret in the config file",
        )?;

    let roles: Vec<String> = cli
        .roles
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();

    let state = Arc::new(AppState {
        upstream: upstream.trim_end_matches('/').to_string(),
        client: reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .pool_idle_timeout(Some(Duration::from_secs(90)))
            .build()
            .context("failed to build reqwest client")?,
        encoding_key: EncodingKey::from_secret(secret.as_bytes()),
        sub: cli.sub,
        name: cli.name,
        roles,
        ttl_secs: cli.ttl_secs,
    });

    let preview = state.mint_token()?;
    tracing::info!(
        listen = %cli.listen,
        upstream = %state.upstream,
        config = ?cfg_path,
        sub = %state.sub,
        ttl_secs = state.ttl_secs,
        sample_token = %&preview[..preview.len().min(24)],
        "uar-jwt-proxy ready"
    );

    let app = Router::new().fallback(any(dispatch)).with_state(state);

    let listener = tokio::net::TcpListener::bind(cli.listen).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn load_config(cli_path: Option<&Path>) -> Result<(UarConfig, Option<PathBuf>)> {
    let candidates: Vec<PathBuf> = if let Some(p) = cli_path {
        vec![p.to_path_buf()]
    } else {
        let mut v = Vec::new();
        v.push(PathBuf::from("config.yaml"));
        if let Some(home) = dirs::home_dir() {
            v.push(home.join(".uar").join("config.yaml"));
        }
        v
    };

    for path in candidates {
        if path.exists() {
            let raw = std::fs::read_to_string(&path)
                .with_context(|| format!("reading config file {}", path.display()))?;
            let cfg: UarConfig = serde_yaml::from_str(&raw)
                .with_context(|| format!("parsing config file {}", path.display()))?;
            return Ok((cfg, Some(path)));
        }
    }
    Ok((UarConfig::default(), None))
}

fn upstream_from_config(cfg: &UarConfig) -> Option<String> {
    let server = cfg.server.as_ref()?;
    let port = server.port?;
    let host_raw = server.host.as_deref().unwrap_or("127.0.0.1");
    // Bind addresses like 0.0.0.0 / :: aren't routable from the browser side; rewrite.
    let host = match host_raw {
        "0.0.0.0" | "::" | "[::]" | "" => "127.0.0.1",
        other => other,
    };
    Some(format!("http://{host}:{port}"))
}

async fn dispatch(State(state): State<Arc<AppState>>, req: Request) -> Response {
    if is_websocket_upgrade(req.headers()) {
        let (mut parts, _body) = req.into_parts();
        let path_and_query = parts
            .uri
            .path_and_query()
            .map(|p| p.as_str().to_string())
            .unwrap_or_else(|| "/".to_string());
        let subprotocols = parts
            .headers
            .get(http::header::SEC_WEBSOCKET_PROTOCOL)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let ws_url = match build_ws_url(&state.upstream, &path_and_query) {
            Ok(u) => u,
            Err(e) => {
                tracing::error!(error = ?e, "bad upstream ws url");
                return StatusCode::BAD_GATEWAY.into_response();
            }
        };
        let target = UpstreamTarget {
            ws_url,
            subprotocols,
        };
        match WebSocketUpgrade::from_request_parts(&mut parts, &()).await {
            Ok(upgrade) => upgrade.on_upgrade(move |ws| async move {
                if let Err(err) = bridge_websocket(state, ws, target).await {
                    tracing::warn!(error = ?err, "websocket bridge ended with error");
                }
            }),
            Err(err) => {
                tracing::error!(error = ?err, "failed to accept websocket upgrade");
                StatusCode::BAD_REQUEST.into_response()
            }
        }
    } else {
        match forward_http(state, req).await {
            Ok(resp) => resp,
            Err(err) => {
                tracing::error!(error = ?err, "proxy http error");
                Response::builder()
                    .status(StatusCode::BAD_GATEWAY)
                    .body(Body::from(format!("uar-jwt-proxy: {err}")))
                    .unwrap()
            }
        }
    }
}

fn is_websocket_upgrade(headers: &HeaderMap) -> bool {
    headers
        .get(http::header::UPGRADE)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("websocket"))
        .unwrap_or(false)
}

async fn forward_http(state: Arc<AppState>, req: Request) -> Result<Response> {
    let (parts, body) = req.into_parts();

    let token = state.mint_token()?;
    let url = build_http_url(&state.upstream, &parts.uri)?;
    let method = reqwest_method(&parts.method);

    let mut headers = clone_forwarded_headers(&parts.headers);
    headers.insert(
        reqwest::header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}"))?,
    );

    let body_bytes = axum::body::to_bytes(body, usize::MAX).await?;

    let upstream_resp = state
        .client
        .request(method, url)
        .headers(headers)
        .body(body_bytes)
        .send()
        .await?;

    let status = upstream_resp.status();
    let upstream_headers = upstream_resp.headers().clone();
    let stream = upstream_resp.bytes_stream();

    let mut builder = Response::builder().status(status.as_u16());
    for (name, value) in upstream_headers.iter() {
        if is_hop_by_hop(name.as_str()) {
            continue;
        }
        builder = builder.header(name.as_str(), value.as_bytes());
    }
    Ok(builder.body(Body::from_stream(stream))?)
}

fn build_http_url(upstream: &str, uri: &Uri) -> Result<reqwest::Url> {
    let path_and_query = uri.path_and_query().map(|p| p.as_str()).unwrap_or("/");
    Ok(reqwest::Url::parse(&format!("{upstream}{path_and_query}"))?)
}

fn reqwest_method(m: &Method) -> reqwest::Method {
    reqwest::Method::from_bytes(m.as_str().as_bytes()).unwrap_or(reqwest::Method::GET)
}

fn clone_forwarded_headers(src: &HeaderMap) -> reqwest::header::HeaderMap {
    let mut out = reqwest::header::HeaderMap::new();
    for (name, value) in src.iter() {
        let n = name.as_str().to_ascii_lowercase();
        if is_hop_by_hop(&n) || n == "host" || n == "authorization" || n == "content-length" {
            continue;
        }
        if let (Ok(hn), Ok(hv)) = (
            HeaderName::from_bytes(name.as_str().as_bytes()),
            HeaderValue::from_bytes(value.as_bytes()),
        ) {
            out.insert(hn, hv);
        }
    }
    out
}

fn is_hop_by_hop(name: &str) -> bool {
    matches!(
        name,
        "connection"
            | "proxy-connection"
            | "keep-alive"
            | "transfer-encoding"
            | "te"
            | "trailer"
            | "upgrade"
    )
}

async fn bridge_websocket(
    state: Arc<AppState>,
    client_ws: WebSocket,
    target: UpstreamTarget,
) -> Result<()> {
    let token = state.mint_token()?;
    let mut request = target
        .ws_url
        .as_str()
        .into_client_request()
        .context("building upstream websocket request")?;
    let headers = request.headers_mut();
    headers.insert(
        http::header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}"))?,
    );
    if let Some(proto) = target.subprotocols {
        if let Ok(val) = HeaderValue::from_str(&proto) {
            headers.insert(http::header::SEC_WEBSOCKET_PROTOCOL, val);
        }
    }

    let (upstream_ws, _resp) = tokio_tungstenite::connect_async(request)
        .await
        .context("connecting to upstream websocket")?;

    let (mut up_sink, mut up_stream) = upstream_ws.split();
    let (mut client_sink, mut client_stream) = client_ws.split();

    let client_to_upstream = async {
        while let Some(msg) = client_stream.next().await {
            match msg {
                Ok(AxMessage::Text(t)) => {
                    up_sink.send(WsMessage::Text(t.to_string().into())).await?
                }
                Ok(AxMessage::Binary(b)) => {
                    up_sink.send(WsMessage::Binary(b.to_vec().into())).await?
                }
                Ok(AxMessage::Ping(p)) => up_sink.send(WsMessage::Ping(p.to_vec().into())).await?,
                Ok(AxMessage::Pong(p)) => up_sink.send(WsMessage::Pong(p.to_vec().into())).await?,
                Ok(AxMessage::Close(frame)) => {
                    let cf = frame.map(|f| tokio_tungstenite::tungstenite::protocol::CloseFrame {
                        code: f.code.into(),
                        reason: f.reason.to_string().into(),
                    });
                    up_sink.send(WsMessage::Close(cf)).await?;
                    break;
                }
                Err(e) => return Err(anyhow::Error::from(e)),
            }
        }
        Ok::<(), anyhow::Error>(())
    };

    let upstream_to_client = async {
        while let Some(msg) = up_stream.next().await {
            match msg {
                Ok(WsMessage::Text(t)) => {
                    client_sink
                        .send(AxMessage::Text(t.to_string().into()))
                        .await?
                }
                Ok(WsMessage::Binary(b)) => {
                    client_sink
                        .send(AxMessage::Binary(b.to_vec().into()))
                        .await?
                }
                Ok(WsMessage::Ping(p)) => {
                    client_sink.send(AxMessage::Ping(p.to_vec().into())).await?
                }
                Ok(WsMessage::Pong(p)) => {
                    client_sink.send(AxMessage::Pong(p.to_vec().into())).await?
                }
                Ok(WsMessage::Close(frame)) => {
                    let cf = frame.map(|f| axum::extract::ws::CloseFrame {
                        code: f.code.into(),
                        reason: f.reason.to_string().into(),
                    });
                    client_sink.send(AxMessage::Close(cf)).await?;
                    break;
                }
                Ok(WsMessage::Frame(_)) => {}
                Err(e) => return Err(anyhow::Error::from(e)),
            }
        }
        Ok::<(), anyhow::Error>(())
    };

    tokio::select! {
        r = client_to_upstream => r?,
        r = upstream_to_client => r?,
    }
    Ok(())
}

struct UpstreamTarget {
    ws_url: reqwest::Url,
    subprotocols: Option<String>,
}

fn build_ws_url(upstream_http: &str, path_and_query: &str) -> Result<reqwest::Url> {
    let ws_base = if let Some(rest) = upstream_http.strip_prefix("https://") {
        format!("wss://{rest}")
    } else if let Some(rest) = upstream_http.strip_prefix("http://") {
        format!("ws://{rest}")
    } else {
        upstream_http.to_string()
    };
    Ok(reqwest::Url::parse(&format!("{ws_base}{path_and_query}"))?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rustcrypto_installation_is_idempotent() {
        ensure_rustcrypto_provider().expect("initial provider installation should succeed");
        ensure_rustcrypto_provider().expect("repeated provider installation should succeed");
    }

    #[test]
    fn app_state_mints_hs256_token() {
        ensure_rustcrypto_provider().expect("provider installation should succeed");
        let state = AppState {
            upstream: "http://127.0.0.1:1906".to_string(),
            client: reqwest::Client::new(),
            encoding_key: EncodingKey::from_secret(b"secret"),
            sub: "test-user".to_string(),
            name: "Test User".to_string(),
            roles: vec!["user".to_string()],
            ttl_secs: 60,
        };

        let token = state.mint_token().expect("token minting should succeed");
        assert_eq!(token.split('.').count(), 3);
    }
}
