#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use futures::FutureExt as _;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Stdio};
use tokio::io::{AsyncBufReadExt as _, AsyncReadExt as _, AsyncWriteExt as _};
use tokio_util::sync::CancellationToken;

const SHUTDOWN_CHILD_ENV: &str = "UAR_SHUTDOWN_TEST_CHILD";

// The MCP fixture acknowledges EOF and then waits for an explicit parent
// release. Registry shutdown must await the real process exit after that event.
const SHUTDOWN_MCP_FIXTURE: &str = r#"
import json
import socket
import sys

for line in sys.stdin:
    try:
        request = json.loads(line)
    except json.JSONDecodeError:
        continue
    request_id = request.get("id")
    if request_id is None:
        continue
    method = request.get("method")
    if method == "initialize":
        result = {
            "protocolVersion": request.get("params", {}).get("protocolVersion", "2024-11-05"),
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "uar-shutdown-test", "version": "1.0.0"},
        }
    elif method == "tools/list":
        result = {"tools": []}
    else:
        print(json.dumps({"jsonrpc": "2.0", "id": request_id,
            "error": {"code": -32601, "message": "method not found"}}), flush=True)
        continue
    print(json.dumps({"jsonrpc": "2.0", "id": request_id, "result": result}), flush=True)

host, port = sys.argv[1].rsplit(":", 1)
with socket.create_connection((host, int(port))) as control:
    control.sendall(b"MCP_EOF\n")
    with control.makefile("rb") as incoming:
        if incoming.readline() != b"RELEASE\n":
            raise RuntimeError("MCP fixture release was not acknowledged")
"#;

type FixtureEvents = tokio::sync::mpsc::UnboundedSender<serde_json::Value>;

fn fixture_event(events: &FixtureEvents, name: &str) {
    events.send(json!({"event": name})).expect("parent fixture event receiver");
}

#[derive(Clone)]
struct ShutdownSseState {
    release: CancellationToken,
    events: FixtureEvents,
}

struct SseReleaseReceipt(FixtureEvents);
impl Drop for SseReleaseReceipt {
    fn drop(&mut self) { let _ = self.0.send(json!({"event": "stream_released"})); }
}

#[derive(Deserialize)]
struct ShutdownChildReady {
    primary: SocketAddr,
    companion: SocketAddr,
}

struct ShutdownChild {
    child: tokio::process::Child,
    control_dir: tempfile::TempDir,
    control: tokio::net::tcp::OwnedWriteHalf,
    events: tokio::io::Lines<tokio::io::BufReader<tokio::net::tcp::OwnedReadHalf>>,
    pending: Vec<serde_json::Value>,
    stderr: Option<tokio::task::JoinHandle<String>>,
    sse: Option<tokio::task::JoinHandle<String>>,
    ready: Option<ShutdownChildReady>,
    exit: Option<ShutdownChildExit>,
}

#[derive(Clone)]
struct ShutdownChildExit {
    status: ExitStatus,
    stderr: String,
    sse: Option<String>,
}

impl ShutdownChild {
    async fn spawn(mode: &str) -> Self {
        let control_dir = tempfile::tempdir().expect("create fixture directory");
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind fixture control");
        let mut child = tokio::process::Command::new(std::env::current_exe().expect("test executable"))
            .args(["--exact", "server::tests::shutdown_process_child", "--nocapture", "--test-threads=1"])
            .env(SHUTDOWN_CHILD_ENV, mode)
            .env("UAR_SHUTDOWN_TEST_CONTROL", listener.local_addr().unwrap().to_string())
            .env("UAR_SHUTDOWN_TEST_CONTROL_DIR", control_dir.path())
            .stdout(Stdio::null()).stderr(Stdio::piped()).kill_on_drop(true)
            .spawn().expect("spawn shutdown child");
        let mut pipe = child.stderr.take().expect("child stderr");
        let stderr = tokio::spawn(async move {
            let mut bytes = Vec::new();
            pipe.read_to_end(&mut bytes).await.expect("drain child stderr");
            String::from_utf8_lossy(&bytes).into_owned()
        });
        let stream = tokio::select! {
            result = listener.accept() => result.expect("accept fixture control").0,
            status = child.wait() => {
                let stderr = stderr.await.expect("join child stderr");
                panic!("child exited before control attachment: {status:?}: {stderr}");
            }
        };
        let (read, write) = stream.into_split();
        Self { child, control_dir, control: write,
            events: tokio::io::BufReader::new(read).lines(), pending: Vec::new(),
            stderr: Some(stderr), sse: None, ready: None, exit: None }
    }

    async fn command(&mut self, command: &str) {
        self.control.write_all(format!("{command}\n").as_bytes()).await.expect("fixture command");
    }

    async fn event(&mut self, name: &str) -> serde_json::Value {
        loop {
            if let Some(index) = self.pending.iter().position(|event| event["event"] == name) {
                return self.pending.remove(index);
            }
            let line = tokio::select! {
                line = self.events.next_line() => line.expect("fixture receipt read").expect("fixture closed before receipt"),
                status = self.child.wait() => panic!("child exited before {name}: {status:?}"),
            };
            self.pending.push(serde_json::from_str(&line).expect("fixture receipt JSON"));
        }
    }

    async fn probe_incomplete(&mut self) {
        self.command("PROBE_COMPLETE").await;
        let receipt = self.event("completion_probe").await;
        assert_eq!(receipt["process_complete"], false, "completion was certified while cleanup was held");
        assert_eq!(receipt["cleanup_complete"], false);
    }

    fn start_sse(&mut self) {
        let address = self.ready.as_ref().expect("fixture ready").primary;
        self.sse = Some(tokio::spawn(async move {
            let mut stream = tokio::net::TcpStream::connect(address).await.expect("connect SSE");
            stream.write_all(format!("GET /sse HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n").as_bytes())
                .await.expect("write SSE request");
            let mut response = String::new();
            stream.read_to_string(&mut response).await.expect("SSE termination acknowledgement");
            response
        }));
    }

    async fn finish(&mut self) -> ShutdownChildExit {
        if let Some(exit) = &self.exit { return exit.clone(); }
        // A failed assertion must release every controlled dependency before
        // awaiting actual child termination. EOF is not a successful close claim.
        let _ = self.control.write_all(b"SHUTDOWN\nRELEASE_SSE\nRELEASE_MCP\nRELEASE_CLEANUP\nFINISH\n").await;
        let status = self.child.wait().await.expect("reap shutdown child");
        let stderr = self.stderr.take().expect("stderr owner").await.expect("join stderr reader");
        let sse = match self.sse.take() {
            Some(reader) => Some(reader.await.expect("join SSE reader")),
            None => None,
        };
        let exit = ShutdownChildExit { status, stderr, sse };
        self.exit = Some(exit.clone());
        exit
    }
}

impl Drop for ShutdownChild {
    fn drop(&mut self) {
        if self.exit.is_none() { let _ = self.child.start_kill(); }
    }
}

async fn with_shutdown_child<F>(mode: &str, body: F) -> ShutdownChildExit
where
    F: for<'a> FnOnce(&'a mut ShutdownChild) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>,
{
    let mut fixture = ShutdownChild::spawn(mode).await;
    let body_result = std::panic::AssertUnwindSafe(async {
        let ready = fixture.event("ready").await;
        fixture.ready = Some(serde_json::from_value(ready).expect("read child readiness"));
        body(&mut fixture).await;
    }).catch_unwind().await;
    let cleanup_result = std::panic::AssertUnwindSafe(fixture.finish()).catch_unwind().await;
    if let Err(original) = body_result {
        match &cleanup_result {
            Ok(exit) => eprintln!("shutdown fixture cleanup after body panic: status={}, graceful={}",
                exit.status, exit.stderr.contains("UAR_SHUTDOWN outcome=graceful_complete")),
            Err(_) => eprintln!("shutdown fixture cleanup also panicked; retaining original body panic"),
        }
        std::panic::resume_unwind(original);
    }
    let exit = cleanup_result.unwrap_or_else(|failure| std::panic::resume_unwind(failure));
    assert!(exit.status.success(), "child status {}: {}", exit.status, exit.stderr);
    assert!(exit.stderr.contains("UAR_SHUTDOWN outcome=graceful_complete"), "missing completion receipt: {}", exit.stderr);
    assert!(!exit.stderr.contains("deadline_enforced"));
    exit
}

async fn shutdown_test_sse(State(state): State<ShutdownSseState>) -> Sse<
    std::pin::Pin<Box<dyn futures::Stream<Item = Result<Event, Infallible>> + Send + 'static>>,
> {
    fixture_event(&state.events, "stream_started");
    let receipt = SseReleaseReceipt(state.events.clone());
    let stream = futures::stream::unfold((0_u8, state, receipt), |(step, state, receipt)| async move {
        match step {
            0 => Some((Ok(Event::default().data("started")), (1, state, receipt))),
            1 => {
                state.release.cancelled().await;
                Some((Ok(Event::default().data("done")), (2, state, receipt)))
            }
            _ => None,
        }
    });
    Sse::new(Box::pin(stream))
}

async fn run_shutdown_process_child(mode: &str, control_dir: &Path) {
    let control = tokio::net::TcpStream::connect(std::env::var("UAR_SHUTDOWN_TEST_CONTROL").unwrap())
        .await.expect("connect parent control");
    let (read, mut write) = control.into_split();
    let (events, mut event_receiver) = tokio::sync::mpsc::unbounded_channel::<serde_json::Value>();
    let event_writer = tokio::spawn(async move {
        while let Some(event) = event_receiver.recv().await {
            write.write_all(format!("{event}\n").as_bytes()).await.expect("publish fixture event");
        }
        write.shutdown().await.expect("close fixture events");
    });
    let primary = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("primary listener");
    let primary_addr = primary.local_addr().unwrap();
    let companion = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("companion listener");
    let companion_addr = companion.local_addr().unwrap();
    let coordinator = ShutdownCoordinator::new(true);
    let process_shutdown = CancellationToken::new();
    let sse_release = CancellationToken::new();
    let mcp_release = CancellationToken::new();
    let (cleanup_release, cleanup_waiter) = std::sync::mpsc::channel::<()>();
    let cleanup_waiter = Arc::new(std::sync::Mutex::new(cleanup_waiter));
    let control_reader = {
        let process_shutdown = process_shutdown.clone();
        let sse_release = sse_release.clone();
        let mcp_release = mcp_release.clone();
        let coordinator = coordinator.clone();
        let events = events.clone();
        tokio::spawn(async move {
            let mut lines = tokio::io::BufReader::new(read).lines();
            while let Some(command) = lines.next_line().await.expect("read parent command") {
                match command.as_str() {
                    "SHUTDOWN" => process_shutdown.cancel(),
                    "RELEASE_SSE" => sse_release.cancel(),
                    "RELEASE_MCP" => mcp_release.cancel(),
                    "RELEASE_CLEANUP" => { let _ = cleanup_release.send(()); }
                    "PROBE_COMPLETE" => {
                        coordinator.complete();
                        events.send(json!({"event":"completion_probe",
                            "cleanup_complete":coordinator.cleanup_complete.is_cancelled(),
                            "process_complete":coordinator.process_complete.is_cancelled()})).unwrap();
                    }
                    "FINISH" => break,
                    other => panic!("unknown fixture command: {other}"),
                }
            }
            // Parent EOF releases controlled test barriers and requests shutdown;
            // the main child still joins cleanup before returning.
            process_shutdown.cancel(); sse_release.cancel(); mcp_release.cancel();
            let _ = cleanup_release.send(());
        })
    };
    let app = Router::new().route("/sse", get(shutdown_test_sse))
        .with_state(ShutdownSseState { release: sse_release, events: events.clone() });
    let stderr_lock = mode == "stderr-lock";
    let held = matches!(mode, "held-cleanup" | "held-cleanup-mcp" | "held-sse" | "embedded" | "stderr-lock");
    let cleanup: ShutdownCleanup = {
        let events = events.clone();
        Arc::new(move || {
            // This owned blocking worker is deliberately held by an explicit
            // parent event, including the stderr-lock case. No linger duration.
            let stderr = std::io::stderr();
            let _stderr_guard = stderr_lock.then(|| stderr.lock());
            fixture_event(&events, "cleanup_started");
            if held { cleanup_waiter.lock().unwrap().recv().expect("cleanup release"); }
            fixture_event(&events, "cleanup_released");
        })
    };
    let mut mcp_monitor = None;
    let registry = if mode == "held-cleanup-mcp" {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let monitor_events = events.clone();
        mcp_monitor = Some(tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("MCP EOF control");
            let mut stream = tokio::io::BufReader::new(stream);
            let mut receipt = String::new();
            stream.read_line(&mut receipt).await.unwrap();
            assert_eq!(receipt, "MCP_EOF\n");
            fixture_event(&monitor_events, "mcp_eof");
            mcp_release.cancelled().await;
            stream.get_mut().write_all(b"RELEASE\n").await.unwrap();
            let mut trailing = Vec::new();
            stream.read_to_end(&mut trailing).await.expect("MCP control EOF");
        }));
        let fixture_path = control_dir.join("mcp-shutdown-fixture.py");
        std::fs::write(&fixture_path, SHUTDOWN_MCP_FIXTURE).unwrap();
        let config = crate::mcp::config::McpConfig { mcp_servers: HashMap::from([(
            "shutdown-fixture".to_owned(), crate::mcp::config::McpServerEntry::Stdio {
                command: "python3".to_owned(), args: vec![fixture_path.display().to_string(), address.to_string()],
                env: HashMap::new(), sandboxed: false, preapproved_tools: Default::default(),
            })]) };
        Some(Arc::new(McpRegistry::from_config(&config).await.expect("MCP initialize")))
    } else { None };
    let store = if mode == "embedded" {
        Some(surrealdb::engine::any::connect(format!("surrealkv://{}", control_dir.join("store").display()))
            .await.expect("open owned embedded store"))
    } else { None };
    let async_cleanup: ShutdownAsyncCleanup = {
        let events = events.clone();
        Arc::new(move || {
            let registry = registry.clone(); let store = store.clone(); let events = events.clone();
            Box::pin(async move {
                if let Some(registry) = registry { registry.shutdown_report().await.expect("MCP owner release"); fixture_event(&events, "mcp_reaped"); }
                if let Some(store) = store { store.shutdown().await.expect("close embedded store"); fixture_event(&events, "store_closed"); }
                fixture_event(&events, "async_cleanup_released");
            })
        })
    };
    events.send(json!({"event":"ready", "primary":primary_addr, "companion":companion_addr})).unwrap();
    serve_on_listener(primary, Some(companion), app, 0, Some(cleanup), Some(async_cleanup),
        CancellationToken::new(), None, Some(process_shutdown), coordinator.clone())
        .await.expect("serve shutdown fixture");
    coordinator.wait_for_cleanup().await;
    coordinator.complete();
    fixture_event(&events, "server_complete");
    control_reader.await.expect("join control reader");
    if let Some(monitor) = mcp_monitor { monitor.await.expect("join MCP monitor"); }
    drop(events);
    event_writer.await.expect("join event writer");
}

#[test]
fn shutdown_process_child() {
    let Some(mode) = std::env::var_os(SHUTDOWN_CHILD_ENV) else { return; };
    let directory = PathBuf::from(std::env::var_os("UAR_SHUTDOWN_TEST_CONTROL_DIR").unwrap());
    let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().unwrap();
    runtime.block_on(run_shutdown_process_child(&mode.to_string_lossy(), &directory));
}

#[tokio::test]
async fn shutdown_process_explicit_request_joins_idle_resources() {
    with_shutdown_child("idle", |child| Box::pin(async move {
        child.command("SHUTDOWN").await;
        child.event("server_complete").await;
    })).await;
}

#[tokio::test]
async fn shutdown_process_sse_and_cleanup_require_release_acknowledgements() {
    let exit = with_shutdown_child("held-sse", |child| Box::pin(async move {
        child.start_sse(); child.event("stream_started").await;
        child.command("SHUTDOWN").await; child.event("cleanup_started").await;
        child.probe_incomplete().await;
        child.command("RELEASE_SSE").await; child.event("stream_released").await;
        child.probe_incomplete().await;
        child.command("RELEASE_CLEANUP").await; child.event("server_complete").await;
        let addresses = child.ready.as_ref().unwrap();
        assert!(tokio::net::TcpStream::connect(addresses.primary).await.is_err());
        assert!(tokio::net::TcpStream::connect(addresses.companion).await.is_err());
    })).await;
    let response = exit.sse.expect("joined SSE reader");
    assert!(response.contains("content-type: text/event-stream"));
    assert!(response.contains("data: started"));
    assert!(response.contains("data: done"));
}

#[tokio::test]
async fn shutdown_process_held_cleanup_cannot_certify_completion() {
    with_shutdown_child("held-cleanup", |child| Box::pin(async move {
        child.command("SHUTDOWN").await; child.event("cleanup_started").await;
        child.event("async_cleanup_released").await; child.probe_incomplete().await;
        child.command("RELEASE_CLEANUP").await; child.event("cleanup_released").await;
        child.event("server_complete").await;
    })).await;
}

#[tokio::test]
async fn shutdown_process_mcp_eof_is_not_process_termination() {
    with_shutdown_child("held-cleanup-mcp", |child| Box::pin(async move {
        child.command("SHUTDOWN").await; child.event("cleanup_started").await;
        child.event("mcp_eof").await; child.probe_incomplete().await;
        child.command("RELEASE_MCP").await; child.event("mcp_reaped").await;
        child.probe_incomplete().await; child.command("RELEASE_CLEANUP").await;
        child.event("server_complete").await;
    })).await;
}

#[tokio::test]
async fn shutdown_process_registered_cleanup_precedes_completion() {
    with_shutdown_child("normal-cleanup", |child| Box::pin(async move {
        child.command("SHUTDOWN").await;
        child.event("cleanup_released").await; child.event("async_cleanup_released").await;
        child.event("server_complete").await;
    })).await;
}

#[tokio::test]
async fn shutdown_process_store_reopens_once_after_acknowledged_exit() {
    with_shutdown_child("embedded", |child| Box::pin(async move {
        child.command("SHUTDOWN").await; child.event("cleanup_started").await;
        child.event("store_closed").await; child.probe_incomplete().await;
        child.command("RELEASE_CLEANUP").await; child.event("server_complete").await;
        let exit = child.finish().await;
        assert!(exit.status.success());
        let reopened = surrealdb::engine::any::connect(format!("surrealkv://{}", child.control_dir.path().join("store").display()))
            .await.expect("one immediate open after acknowledged process exit");
        reopened.shutdown().await.expect("close reopened store");
    })).await;
}

#[tokio::test]
async fn shutdown_process_stderr_lock_is_owned_and_joined() {
    with_shutdown_child("stderr-lock", |child| Box::pin(async move {
        child.command("SHUTDOWN").await; child.event("cleanup_started").await;
        child.probe_incomplete().await;
        child.command("RELEASE_CLEANUP").await; child.event("server_complete").await;
    })).await;
}

#[tokio::test]
async fn shutdown_process_parent_panic_releases_and_reaps_fixture() {
    let addresses = Arc::new(std::sync::Mutex::new(None));
    let captured = Arc::clone(&addresses);
    let result = std::panic::AssertUnwindSafe(with_shutdown_child("held-cleanup-mcp", move |child| {
        Box::pin(async move {
            let ready = child.ready.as_ref().unwrap();
            *captured.lock().unwrap() = Some((ready.primary, ready.companion));
            child.command("SHUTDOWN").await;
            child.event("cleanup_started").await;
            child.event("mcp_eof").await;
            panic!("controlled parent body failure");
        })
    })).catch_unwind().await;
    let failure = match result { Err(failure) => failure, Ok(_) => panic!("original body panic must propagate") };
    assert_eq!(failure.downcast_ref::<&str>(), Some(&"controlled parent body failure"));
    let (primary, companion) = addresses.lock().unwrap().take().expect("captured listeners");
    // One acquisition after the parent scope joined the process; no probes or retries.
    let _primary = tokio::net::TcpListener::bind(primary).await.expect("primary listener released");
    let _companion = tokio::net::TcpListener::bind(companion).await.expect("companion listener released");
}

    #[test]
    fn governance_bootstrap_error_finalizes_fail_closed() {
        let (mutation, gate, status) =
            crate::uar::governance::runtime_control::governance_runtime_handles("localhost");
        mutation.record_installed_authentication(false);
        mutation.declare_ingress("primary-http").expect("declare");
        let proof = mutation
            .register_bound_ingress("primary-http", "127.0.0.1:1906".parse().expect("address"))
            .expect("register");
        mutation
            .seal_ingress_inventory(&[proof])
            .expect("seal inventory");

        finalize_failed_governance_bootstrap(&mutation, &anyhow::anyhow!("seed failed"))
            .expect("failure finalization succeeds");

        let snapshot = status.snapshot();
        assert!(gate.effective_enabled());
        assert!(!snapshot.mutation_available);
        assert!(snapshot.reasons.contains(
            &crate::uar::governance::runtime_control::GovernanceStatusReason::PersistenceUnavailable
        ));
    }

    #[test]
    fn ensure_toolu_id_prefixes_non_prefixed_ids() {
        assert_eq!(ensure_toolu_id("abc"), "toolu_abc");
        assert_eq!(ensure_toolu_id("toolu_xyz"), "toolu_xyz");
    }

    #[tokio::test]
    async fn security_txt_has_the_required_rfc9116_fields() {
        let (headers, body) = security_txt_handler().await;
        assert_eq!(headers[0].1, "text/plain; charset=utf-8");
        assert!(body.contains("Contact: https://"));
        assert!(body.contains("Expires: "));
        // A real reporting channel, not a fabricated email/PGP key.
        assert!(body.contains("security/advisories/new"));
    }

    #[test]
    fn convert_anthropic_messages_round_trips_tool_blocks() {
        let req = AnthropicMessagesRequest {
            model: "claude-sonnet-4-20250514".to_string(),
            messages: vec![
                AnthropicMessageInput {
                    role: "assistant".to_string(),
                    content: AnthropicContentInput::Blocks(vec![AnthropicContentBlockInput {
                        block_type: "tool_use".to_string(),
                        id: Some("call_1".to_string()),
                        name: Some("edit_file".to_string()),
                        input: Some(json!({"path":"foo.rs"})),
                        ..AnthropicContentBlockInput::default()
                    }]),
                    _extra: HashMap::new(),
                },
                AnthropicMessageInput {
                    role: "user".to_string(),
                    content: AnthropicContentInput::Blocks(vec![AnthropicContentBlockInput {
                        block_type: "tool_result".to_string(),
                        tool_use_id: Some("call_1".to_string()),
                        content: Some(json!("ok")),
                        ..AnthropicContentBlockInput::default()
                    }]),
                    _extra: HashMap::new(),
                },
            ],
            system: None,
            tools: Vec::new(),
            stream: false,
            session_id: None,
            agent_id: None,
            prompt_caching_enabled: None,
            _extra: HashMap::new(),
        };

        let out = convert_anthropic_messages_to_openai(&req);
        assert_eq!(out.len(), 2);
        assert_eq!(
            out[0].get("role").and_then(Value::as_str),
            Some("assistant")
        );
        assert_eq!(
            out[0]
                .get("tool_calls")
                .and_then(Value::as_array)
                .map(Vec::len),
            Some(1)
        );
        assert_eq!(out[1].get("role").and_then(Value::as_str), Some("tool"));
        assert_eq!(
            out[1].get("tool_call_id").and_then(Value::as_str),
            Some("toolu_call_1")
        );
    }

    #[test]
    fn request_timeout_policy_skips_chat_stream_routes() {
        assert!(!should_apply_request_timeout("/api/chat/completion"));
        assert!(!should_apply_request_timeout("/v1/chat/completions"));
        assert!(!should_apply_request_timeout("/v1/messages"));

        assert!(should_apply_request_timeout("/api/generate-title"));
        assert!(should_apply_request_timeout("/api/threads"));
        assert!(should_apply_request_timeout("/assets/index.js"));
    }

    #[tokio::test]
    async fn cors_layer_allows_configured_origin_methods_and_headers() {
        use axum::{Router, body::Body, http::Method, http::header, routing::get};
        use tower::ServiceExt;

        let app = Router::new()
            .route("/ping", get(|| async { "ok" }))
            .layer(build_cors_layer(&["https://example.com".to_owned()]).expect("valid origin"));

        let req = Request::builder()
            .method(Method::OPTIONS)
            .uri("/ping")
            .header(header::ORIGIN, "https://example.com")
            .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
            .header(
                header::ACCESS_CONTROL_REQUEST_HEADERS,
                "x-custom,content-type",
            )
            .body(Body::empty())
            .expect("preflight request should build");

        let res = app
            .oneshot(req)
            .await
            .expect("preflight request should succeed");
        assert!(res.status().is_success());
        assert!(
            res.headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .is_some()
        );
        assert!(
            res.headers()
                .get(header::ACCESS_CONTROL_ALLOW_METHODS)
                .is_some()
        );
        assert!(
            res.headers()
                .get(header::ACCESS_CONTROL_ALLOW_HEADERS)
                .is_some()
        );
    }

    #[test]
    fn cors_configuration_rejects_wildcard_and_opaque_origins() {
        for origin in ["", "*", "null"] {
            assert!(build_cors_layer(&[origin.to_owned()]).is_err());
        }
    }

    #[tokio::test]
    async fn cors_layer_omits_allow_origin_for_default_and_unlisted_origins() {
        use axum::{Router, body::Body, http::header, routing::get};
        use tower::ServiceExt;

        for allowed in [Vec::new(), vec!["https://allowed.example".to_owned()]] {
            let app = Router::new()
                .route("/ping", get(|| async { "ok" }))
                .layer(build_cors_layer(&allowed).expect("valid origin configuration"));
            let request = Request::builder()
                .uri("/ping")
                .header(header::ORIGIN, "https://unlisted.example")
                .body(Body::empty())
                .expect("origin request");
            let response = app.oneshot(request).await.expect("response");
            assert!(!response.headers().contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN));
        }
    }

    #[test]
    fn provider_anthropic_usage_preserves_reported_cache_fields_exactly() {
        let usage = provider_anthropic_usage(41, 7, Some(29), Some(13));
        assert_eq!(usage.input_tokens, 41);
        assert_eq!(usage.output_tokens, 7);
        assert_eq!(usage.cache_creation_input_tokens, 13);
        assert_eq!(usage.cache_read_input_tokens, 29);
    }

    #[test]
    fn absent_provider_usage_never_fabricates_cache_activity() {
        let usage = finalized_anthropic_usage(None, 9);
        assert_eq!(usage.input_tokens, 0);
        assert_eq!(usage.output_tokens, 9);
        assert_eq!(usage.cache_creation_input_tokens, 0);
        assert_eq!(usage.cache_read_input_tokens, 0);
    }

    #[test]
    fn normalize_legacy_openai_base_url_rewrites_known_non_openai_provider() {
        let mut config = crate::config::LlmConfig {
            model: "alibaba/qwen3.6-plus".to_string(),
            base_url: Some("https://api.openai.com".to_string()),
            ..crate::config::LlmConfig::default()
        };

        normalize_legacy_openai_base_url(&mut config);

        assert_eq!(
            config.base_url.as_deref(),
            Some("https://dashscope-intl.aliyuncs.com/compatible-mode/v1")
        );
    }

    #[test]
    fn normalize_legacy_openai_base_url_adds_openai_v1_path() {
        let mut config = crate::config::LlmConfig {
            model: "openai/gpt-5.4".to_string(),
            base_url: Some("https://api.openai.com".to_string()),
            ..crate::config::LlmConfig::default()
        };

        normalize_legacy_openai_base_url(&mut config);

        assert_eq!(
            config.base_url.as_deref(),
            Some("https://api.openai.com/v1")
        );
    }

    #[test]
    fn provider_catalog_status_marks_moonshot_credential_blocked() {
        let (status, detail) =
            provider_catalog_status("moonshotai", false, Some("MOONSHOT_API_KEY"));

        assert_eq!(status, "credential-blocked");
        assert!(detail.contains("MOONSHOT_API_KEY"));
        assert!(!detail.contains("sk-"));
    }

    #[test]
    fn provider_catalog_status_prefers_configured_over_missing_credential() {
        let (status, detail) =
            provider_catalog_status("moonshotai", true, Some("MOONSHOT_API_KEY"));

        assert_eq!(status, "configured");
        assert!(detail.contains("configured"));
    }
