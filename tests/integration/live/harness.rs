//! Minimal real-server boot harness for the live integration tier's baseline
//! feature cases (live-integration-baseline-coverage, task group 1).
//!
//! Rather than hand-construct `AppState` field-by-field (its ~20 fields
//! include several whose constructors are non-obvious — see
//! `openspec/changes/live-integration-baseline-coverage/appstate-field-plan.md`
//! for that research), this harness calls the actual public
//! `universal_agent_runtime::server::start_server_with_listeners` entry point against
//! a config built entirely from an
//! explicit temp YAML file. This is both simpler and a stronger proof: the
//! baseline cases exercise the real sidecar boot path, not a hand-approximated
//! one.
//!
//! The server's handler functions (`api_chat_completion`, etc.) are
//! `pub(crate)`, so this harness cannot call them directly even if it wanted
//! to — which is fine, since HTTP-level testing never needs to.

use std::sync::Once;
use std::time::Duration;

use tokio_util::sync::CancellationToken;
use tokio::io::{AsyncBufReadExt as _, AsyncWriteExt as _};
use universal_agent_runtime::config::Cli;
use universal_agent_runtime::config_manager::ConfigManager;
use universal_agent_runtime::server::start_server_with_listeners_and_shutdowns;

static TRACING_INIT: Once = Once::new();
static SCRATCH_SWEEP: Once = Once::new();

/// `start_server`'s internal `tracing::info!`/`error!` calls are otherwise
/// silently dropped in a test binary (no ambient subscriber) — which made a
/// prior failure (memory service falling back to `None`) far harder to
/// diagnose than it needed to be. `Once`-guarded so repeated
/// `boot_test_server` calls across tests in the same process don't panic on
/// double-init. `RUST_LOG` controls verbosity; default `info` to catch
/// exactly the sort of soft-fail-and-fall-back errors this tier cares about.
fn init_tracing_once() {
    TRACING_INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .with_test_writer()
            .try_init();
    });
}

/// JWT secret the harness bakes into every booted server's config. Exposed so
/// tests that need an authenticated request can mint a Bearer token the
/// server will actually verify (the middleware parses a provided token even
/// when `jwt_required: false`, yielding a real non-anonymous `UserContext`).
pub const HARNESS_JWT_SECRET: &str = "test-secret-not-for-production";

/// Mint the verified UAR-instance identity used by peer-first orchestration
/// cases. Call this after booting the server so UAR has installed its guarded
/// process-level JWT provider before the test encodes a token.
#[allow(dead_code)] // The BDD target shares this harness but has no peer-auth scenario.
pub fn mint_harness_peer_token() -> String {
    let claims = universal_agent_runtime::uar::security::claims::UserClaims {
        sub: "live-harness-peer".to_string(),
        name: Some("Live Harness UAR Peer".to_string()),
        roles: Some(vec!["service".to_string()]),
        tenant_id: None,
        uar_instance_id: Some("live-harness-uar".to_string()),
        exp: usize::MAX,
    };
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(HARNESS_JWT_SECRET.as_bytes()),
    )
    .expect("mint harness UAR peer JWT")
}

/// Which optional services a baseline case needs, beyond the always-on
/// SurrealDB-embedded persistence layer `start_server` requires
/// unconditionally (see appstate-field-plan.md's persistence finding).
#[derive(Debug, Clone, Copy, Default)]
pub struct ServiceNeeds {
    /// Enables `memory.enabled` with an embedded (SurrealKV file, not a
    /// network service) backend and local (non-OpenAI) embeddings.
    pub memory: bool,
}

/// A running real UAR server instance, bound to an ephemeral loopback port.
///
/// The server's returned future is `!Send` (a `tracing::info!` call in
/// its body holds a non-`Send` value across an `.await`), so it cannot be
/// handed to `tokio::spawn` directly. Rather than touch `src/server.rs` to
/// chase that down (out of scope for a test-infra change — see design.md),
/// this harness runs it on a dedicated OS thread with its own
/// current-thread Tokio runtime, which has no `Send` requirement on the
/// future it drives. Dropping the handle cancels and joins that thread so a
/// full integration binary does not retain every previously booted server.
#[allow(dead_code)] // The BDD target imports this harness but only reads `base_url`.
pub struct TestServerHandle {
    pub base_url: String,
    shutdown: CancellationToken,
    // Explicit shutdown takes and joins the thread exactly once. Drop only
    // requests cancellation and never reports completed resource release.
    thread: Option<std::thread::JoinHandle<anyhow::Result<()>>>,
}

/// A real UAR server isolated in a child test process.
///
/// A dedicated process lets the parent distinguish server-runtime shutdown
/// from process exit. The child can remain alive at a post-runtime barrier
/// while the parent proves that a second UAR opens the same SurrealKV path.
#[allow(dead_code)] // Consumed by capability cases, which the BDD target omits.
pub struct ProcessTestServerHandle {
    pub base_url: String,
    child: Option<tokio::process::Child>,
    control: Option<tokio::io::BufReader<tokio::net::TcpStream>>,
    control_dir: Option<tempfile::TempDir>,
}

impl ProcessTestServerHandle {
    /// Terminate the owned host without running its graceful shutdown path.
    /// Used by the C06 phase gate to observe recovery of a persisted active turn.
    #[allow(dead_code)] // Only the C06 real-host gate needs abrupt termination.
    pub async fn crash(mut self) {
        let mut child = self.child.take().expect("child server process");
        child.start_kill().expect("kill active child server process");
        let status = child.wait().await.expect("wait for crashed child server process");
        assert!(
            !status.success(),
            "child exited gracefully instead of crashing"
        );
    }

    /// Trigger the child harness's caller-owned token and await normal process
    /// exit after the post-runtime resource-release barrier.
    #[allow(dead_code)] // Consumed by capability cases, which the BDD target omits.
    pub async fn shutdown(self) {
        self.shutdown_with_signal("TERM").await;
    }

    async fn shutdown_with_signal(self, signal: &str) {
        let barrier = self.shutdown_to_pre_exit_barrier(signal).await;
        barrier.allow_exit().await;
    }

    #[allow(dead_code)] // Consumed by capability cases, which the BDD target omits.
    pub async fn shutdown_to_pre_exit_barrier(
        mut self,
        signal: &str,
    ) -> ProcessTestServerExitBarrier {
        let mut control = self.control.take().expect("child control stream");
        control.get_mut().write_all(b"SHUTDOWN\n").await.expect("request child shutdown");
        let child = self.child.take().expect("child server process");
        #[cfg(unix)]
        {
            let status = std::process::Command::new("/bin/kill")
                .arg(format!("-{signal}"))
                .arg(child.id().expect("child pid").to_string()).status().expect("send shutdown signal");
            assert!(status.success(), "failed to send child signal");
        }
        #[cfg(not(unix))]
        let _ = signal;
        let mut acknowledgement = String::new();
        control.read_line(&mut acknowledgement).await.expect("read child release acknowledgement");
        assert_eq!(acknowledgement, "RESOURCES_RELEASED\n", "child did not complete shutdown");
        ProcessTestServerExitBarrier {
            child: Some(child), control: Some(control), control_dir: self.control_dir.take(),
        }
    }
}

#[allow(dead_code)] // Consumed by capability cases, which the BDD target omits.
pub struct ProcessTestServerExitBarrier {
    child: Option<tokio::process::Child>,
    control: Option<tokio::io::BufReader<tokio::net::TcpStream>>,
    control_dir: Option<tempfile::TempDir>,
}

impl ProcessTestServerExitBarrier {
    #[allow(dead_code)] // Consumed by capability cases, which the BDD target omits.
    pub async fn allow_exit(mut self) {
        let _control_dir = self.control_dir.take().expect("child control directory");
        let mut control = self.control.take().expect("child control stream");
        control.get_mut().write_all(b"ALLOW_EXIT\n").await.expect("release child pre-exit barrier");
        let mut child = self.child.take().expect("child server process");
        let status = child.wait().await.expect("wait for child server process");
        assert!(
            status.success(),
            "child server process exited unsuccessfully: {status}"
        );
    }
}

impl Drop for TestServerHandle {
    fn drop(&mut self) {
        // Emergency cancellation only. Async close/finish_case owns the join
        // acknowledgement; destructors must not block a panicking test thread.
        self.shutdown.cancel();
    }
}

impl Drop for ProcessTestServerExitBarrier {
    fn drop(&mut self) {
        if let Some(child) = self.child.as_mut() { let _ = child.start_kill(); }
    }
}

impl Drop for ProcessTestServerHandle {
    fn drop(&mut self) {
        if let Some(child) = self.child.as_mut() { let _ = child.start_kill(); }
    }
}

impl TestServerHandle {
    /// Graceful closure is explicitly awaited before successful fixture completion.
    pub async fn close(self) {
        self.shutdown.cancel();
        self.wait_for_exit().await;
    }

    /// Keep fixture ownership outside a fallible test body, then finalize before
    /// forwarding its panic to the test runner.
    pub async fn run_scoped<F, T>(self, body: F) -> T
    where F: std::future::Future<Output = T> {
        use futures::FutureExt;
        let outcome = std::panic::AssertUnwindSafe(body).catch_unwind().await;
        self.finish_case(outcome).await
    }

    pub async fn finish_case<T>(self, outcome: std::thread::Result<T>) -> T {
        use futures::FutureExt;
        let cleanup = std::panic::AssertUnwindSafe(self.close()).catch_unwind().await;
        match outcome {
            Err(panic) => {
                if cleanup.is_err() { eprintln!("server cleanup also failed; original failure retained"); }
                std::panic::resume_unwind(panic)
            }
            Ok(value) => { cleanup.unwrap_or_else(|panic| std::panic::resume_unwind(panic)); value }
        }
    }

    /// Fire the caller-owned HTTP graceful-shutdown trigger.
    #[allow(dead_code)] // Consumed by the child helper, which the BDD target omits.
    pub fn trigger_shutdown(&self) {
        self.shutdown.cancel();
    }

    /// Await the dedicated server thread after a shutdown trigger has fired.
    #[allow(dead_code)] // Consumed by the child helper, which the BDD target omits.
    pub async fn wait_for_exit(mut self) {
        let thread = self.thread.take().expect("server thread handle");
        let server_result = tokio::task::spawn_blocking(move || thread.join())
            .await
            .expect("join server thread task")
            .expect("server thread panicked");
        server_result.expect("server exited with an error");
    }
}

/// Best-effort removal of THIS process's temp scratch once, at process exit,
/// registered lazily on the first `boot_test_server` call. Per-handle
/// `Drop`-time cleanup does not work here: the detached `start_server` thread
/// keeps its surrealkv persistence dir open and actively written for the rest
/// of the process, so removing it mid-run either fails or is immediately
/// recreated. Sweeping stale scratch (from prior, now-exited processes) at
/// startup is the robust approach — see [`sweep_stale_scratch`].
fn scratch_prefix() -> std::path::PathBuf {
    std::env::temp_dir().join("uar-live-itest-")
}

/// Remove `uar-live-itest-*` scratch left by earlier test-binary runs.
///
/// Each entry embeds the creating process's PID and is only ever written
/// while that process lives (the detached server holds it open). Anything
/// last-modified more than a few minutes ago therefore belongs to a process
/// that has since exited (a live suite finishes in well under a minute), so
/// it is safe to delete — and safe against a *concurrently* running test
/// binary, whose scratch is fresh. Called once per process (Once-guarded)
/// from `boot_test_server`, so accumulation across runs self-limits to at
/// most one run's worth (~80MB) at any time.
fn sweep_stale_scratch() {
    let prefix = scratch_prefix();
    let prefix_str = prefix.to_string_lossy().to_string();
    let Some(parent) = prefix.parent().map(std::path::Path::to_path_buf) else {
        return;
    };
    let cutoff = std::time::SystemTime::now() - Duration::from_secs(300);
    let Ok(entries) = std::fs::read_dir(&parent) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.to_string_lossy().starts_with(&prefix_str) {
            continue;
        }
        let stale = entry
            .metadata()
            .and_then(|m| m.modified())
            .map(|mtime| mtime < cutoff)
            .unwrap_or(false);
        if stale {
            let _ = std::fs::remove_dir_all(&path);
            let _ = std::fs::remove_file(&path);
        }
    }
}

fn unique_temp_path(tag: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "uar-live-itest-{tag}-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ))
}

/// Boot a real UAR server pointed at `llm_base_url`/`llm_model` (the
/// live-integration-gate-resolved backend), with only the services `needs`
/// requires beyond the mandatory embedded-SurrealDB persistence layer.
///
/// Every config value is set explicitly in a throwaway temp YAML file —
/// `AppConfig::load_with_cli` (which `ConfigManager::load_without_watcher`
/// calls internally) falls back to probing `./config.yaml` and
/// `~/.uar/config.yaml` when no `--config` is given, and both exist on at
/// least one developer machine this harness was built on. An explicit
/// `cli.config` path is the only way to guarantee this harness never reads
/// someone's real local configuration.
pub async fn boot_test_server(
    llm_base_url: &str,
    llm_model: &str,
    needs: ServiceNeeds,
) -> TestServerHandle {
    let persistence_path = unique_temp_path("persistence");
    boot_test_server_inner(
        llm_base_url,
        llm_model,
        needs,
        persistence_path,
        None,
        None,
        None,
    )
    .await
}

/// Boot a real UAR server on a caller-owned persistence path. The process
/// harness reuses this seam after awaiting a normal child exit.
#[allow(dead_code)] // Consumed by the child helper, which the BDD target omits.
pub async fn boot_test_server_with_persistence_path(
    llm_base_url: &str,
    llm_model: &str,
    needs: ServiceNeeds,
    persistence_path: &std::path::Path,
) -> TestServerHandle {
    boot_test_server_with_persistence_path_and_instance_id(
        llm_base_url,
        llm_model,
        needs,
        persistence_path,
        None,
        None,
        None,
    )
    .await
}

/// The C06 live gate keeps one configured service identity across process boots.
pub async fn boot_test_server_with_persistence_path_and_instance_id(
    llm_base_url: &str,
    llm_model: &str,
    needs: ServiceNeeds,
    persistence_path: &std::path::Path,
    service_instance_id: Option<&str>,
    http_port: Option<u16>,
    file_root: Option<&std::path::Path>,
) -> TestServerHandle {
    boot_test_server_inner(
        llm_base_url,
        llm_model,
        needs,
        persistence_path.to_path_buf(),
        service_instance_id,
        http_port,
        file_root,
    )
    .await
}

/// Boot a real server in a fresh child process on a caller-owned persistence
/// path. This is the cold-restart harness used by L4 capability cases.
#[allow(dead_code)] // Consumed by capability cases, which the BDD target omits.
pub async fn boot_test_server_process(
    llm_base_url: &str,
    llm_model: &str,
    needs: ServiceNeeds,
    persistence_path: &std::path::Path,
) -> ProcessTestServerHandle {
    boot_test_server_process_with_instance_id(
        llm_base_url,
        llm_model,
        needs,
        persistence_path,
        None,
        None,
        None,
    )
    .await
}

/// Boot the same production path with an explicit, restart-stable service identity.
pub async fn boot_test_server_process_with_instance_id(
    llm_base_url: &str,
    llm_model: &str,
    needs: ServiceNeeds,
    persistence_path: &std::path::Path,
    service_instance_id: Option<&str>,
    http_port: Option<u16>,
    file_root: Option<&std::path::Path>,
) -> ProcessTestServerHandle {
    let control_dir = tempfile::tempdir().expect("create child-server control directory");
    let control_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind child control");
    let control_token = uuid::Uuid::new_v4().to_string();
    let stderr_file = std::fs::File::create(control_dir.path().join("stderr.log"))
        .expect("create child-server stderr capture");
    let mut command = std::process::Command::new(
        std::env::current_exe().expect("resolve integration test executable"),
    );
    command
        .arg("--exact")
        .arg("live::harness::tests::process_server_helper")
        .arg("--nocapture")
        .arg("--test-threads=1")
        .env("UAR_TEST_SERVER_CHILD", "1")
        .env("UAR_TEST_SERVER_LLM_BASE_URL", llm_base_url)
        .env("UAR_TEST_SERVER_LLM_MODEL", llm_model)
        .env(
            "UAR_TEST_SERVER_MEMORY",
            if needs.memory { "1" } else { "0" },
        )
        .env("UAR_TEST_SERVER_PERSISTENCE_PATH", persistence_path)
        .env("UAR_TEST_SERVER_CONTROL_ADDRESS", control_listener.local_addr().expect("control address").to_string())
        .env("UAR_TEST_SERVER_CONTROL_TOKEN", &control_token)
        .env_remove("UAR_TEST_SERVER_INSTANCE_ID")
        .env_remove("UAR_TEST_SERVER_HTTP_PORT")
        .env_remove("UAR_TEST_SERVER_FILE_ROOT")
        .stderr(std::process::Stdio::from(stderr_file));
    if let Some(instance_id) = service_instance_id {
        command.env("UAR_TEST_SERVER_INSTANCE_ID", instance_id);
    }
    if let Some(port) = http_port {
        command.env("UAR_TEST_SERVER_HTTP_PORT", port.to_string());
    }
    if let Some(root) = file_root {
        command.env("UAR_TEST_SERVER_FILE_ROOT", root);
    }
    let mut child = tokio::process::Command::from(command).kill_on_drop(true).spawn().expect("spawn child server process");
    let (stream, _) = tokio::select! {
        accepted = control_listener.accept() => accepted.expect("accept child control"),
        status = child.wait() => panic!("child exited before readiness: {status:?}"),
    };
    let mut control = tokio::io::BufReader::new(stream);
    let mut ready = String::new();
    control.read_line(&mut ready).await.expect("read child readiness");
    let (token, base_url) = ready.trim().split_once(' ').expect("child readiness frame");
    assert_eq!(token, control_token, "child control identity");
    ProcessTestServerHandle {
        base_url: base_url.to_owned(), child: Some(child), control: Some(control), control_dir: Some(control_dir),
    }
}

async fn boot_test_server_inner(
    llm_base_url: &str,
    llm_model: &str,
    needs: ServiceNeeds,
    persistence_path: std::path::PathBuf,
    service_instance_id: Option<&str>,
    http_port: Option<u16>,
    file_root: Option<&std::path::Path>,
) -> TestServerHandle {
    init_tracing_once();
    SCRATCH_SWEEP.call_once(sweep_stale_scratch);
    let listener = std::net::TcpListener::bind(("127.0.0.1", http_port.unwrap_or(0)))
        .expect("bind harness listener");
    listener
        .set_nonblocking(true)
        .expect("set harness listener nonblocking");
    let port = listener
        .local_addr()
        .expect("harness listener address")
        .port();
    let grpc_listener =
        std::net::TcpListener::bind("127.0.0.1:0").expect("bind ephemeral harness gRPC listener");
    grpc_listener
        .set_nonblocking(true)
        .expect("set harness gRPC listener nonblocking");
    let grpc_port = grpc_listener
        .local_addr()
        .expect("harness gRPC listener address")
        .port();

    let memory_yaml = if needs.memory {
        let memory_path = unique_temp_path("memory");
        // `db_path` is a BARE filesystem path: `MemoryService::new` builds the
        // endpoint itself with `format!("surrealkv://{db_path}")`. Passing a
        // scheme-qualified value here yields `surrealkv://surrealkv:///tmp/…`,
        // which SurrealDB accepts as a path and then fails to open.
        format!(
            "\nmemory:\n  enabled: true\n  db_path: \"{}\"\n  embedding_provider: \"local\"\n  embedding_model: \"BAAI/bge-small-en-v1.5\"\n",
            memory_path.display()
        )
    } else {
        String::new()
    };

    let service_instance_yaml = service_instance_id.map_or_else(String::new, |instance_id| {
        format!("service_instance:\n  instance_id: \"{instance_id}\"\n")
    });
    let native_tools_yaml = file_root.map_or_else(String::new, |root| {
        let quoted = serde_json::to_string(root.to_string_lossy().as_ref())
            .expect("quote C06 file root for YAML");
        format!(
            "native_tools:\n  file_tools_enabled: true\n  file_allowed_paths:\n    - {quoted}\n  terminal_exec_enabled: true\n  terminal_use_sandbox: false\n"
        )
    });
    // The C06 protected-tool fixture uses authenticated, mandatory governance.
    // Other live-harness cases retain their existing optional local posture.
    let jwt_required = file_root.is_some();
    let yaml = format!(
        "skill_source_roots: []\nsecurity:\n  jwt_required: {jwt_required}\n  jwt_secret: \"{HARNESS_JWT_SECRET}\"\n  settings_admin_key: \"live-harness-admin-key\"\n\
         resilience:\n  rate_limit_enabled: false\n\
         persistence:\n  provider: \"surreal\"\n  database_url: \"surrealkv://{}\"\n\
         acp:\n  enabled: true\n  path: \"/acp\"\n  auth_required: true\n\
         llm:\n  model: \"{llm_model}\"\n  base_url: \"{llm_base_url}\"\n\
         server:\n  host: \"127.0.0.1\"\n  port: {port}\n  shutdown_timeout_secs: 30\n\
  grpc_port: {grpc_port}\n\
         {memory_yaml}{service_instance_yaml}{native_tools_yaml}",
        persistence_path.display(),
    );

    let config_path = unique_temp_path("config").with_extension("yaml");
    std::fs::write(&config_path, yaml).expect("write temp harness config");
    let mcp_config_path = unique_temp_path("mcp").with_extension("json");
    std::fs::write(&mcp_config_path, r#"{"mcpServers":{}}"#)
        .expect("write empty harness MCP config");

    let cli = Cli {
        env_file: None,
        config: Some(config_path.to_string_lossy().to_string()),
        port: None,
        jwt_required: None,
        rate_limit_enabled: None,
        timeout_disabled: None,
        external_cache_enabled: None,
        llm_model: None,
        llm_api_key: None,
        llm_base_url: None,
        llm_protocol: None,
        llm_budget_limit: None,
        failover_enabled: None,
        native_file_tools: None,
        native_web_fetch: None,
        native_terminal_exec: None,
        skill_evolution_enabled: None,
        skill_evolution_model: None,
        acp_enabled: None,
        acp_path: None,
        strict_config: false,
        command: None,
    };

    // The sidecar entry point takes an `Arc<ConfigManager>` (hot-reload via arc-swap,
    // f53b988). `load_without_watcher` is the documented test constructor: the
    // harness writes a throwaway temp config that never changes, so a file
    // watcher would be pure overhead — and one watcher task per booted server
    // would leak across the tier's many `boot_test_server` calls.
    let config = ConfigManager::load_without_watcher(cli)
        .await
        .expect("load harness config");

    let shutdown = CancellationToken::new();
    let thread_shutdown = shutdown.clone();
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
    let thread = std::thread::Builder::new()
        .name("uar-live-itest-server".to_string())
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("build current-thread runtime for server");
            rt.block_on(async move {
                let listener = tokio::net::TcpListener::from_std(listener)
                    .expect("register harness listener with Tokio");
                let grpc_listener = tokio::net::TcpListener::from_std(grpc_listener)
                    .expect("register harness gRPC listener with Tokio");
                let result = start_server_with_listeners_and_shutdowns(
                    config,
                    listener,
                    grpc_listener,
                    Some(mcp_config_path),
                    ready_tx,
                    Some(thread_shutdown.clone()),
                    Some(thread_shutdown),
                )
                .await;
                if let Err(error) = &result {
                    eprintln!("start_server_sidecar exited with error: {error:?}");
                }
                result
            })
        })
        .expect("spawn dedicated live integration server thread");

    let mut handle = TestServerHandle { base_url: String::new(), shutdown, thread: Some(thread) };
    let ready_addr = match ready_rx.await {
        Ok(address) => address,
        Err(_) => {
            handle.close().await;
            panic!("server exited before signaling readiness");
        }
    };
    handle.base_url = format!("http://{ready_addr}");
    handle
}


#[cfg(test)]
mod tests {
    #[allow(
        unused_imports,
        reason = "the Cucumber custom test harness compiles this unit-test module without executing it"
    )]
    use super::*;
    #[allow(
        unused_imports,
        reason = "the Cucumber custom test harness compiles this unit-test module without executing it"
    )]
    #[cfg(not(feature = "afc-c06-gate"))]
    use crate::live::stub_llm::{FixtureResponse, RequestFingerprint};
    #[allow(
        unused_imports,
        reason = "the Cucumber custom test harness compiles this unit-test module without executing it"
    )]
    #[cfg(not(feature = "afc-c06-gate"))]
    use crate::live::stub_llm::{FixtureSet, start_stub_llm};
    #[cfg(not(feature = "afc-c06-gate"))]
    use serial_test::serial;

    // #[serial]: booting a real server (real embedded SurrealDB, real
    // orchestrator, real MCP subprocess spawns) is heavy enough that running
    // several concurrently causes health-check timeouts under cargo test's
    // default parallelism — confirmed by a full-suite run that passed each
    // module individually but failed 7/16 tests together. Every test that
    // calls boot_test_server (here and in baseline_cases.rs) is #[serial]
    // for this reason, not for shared mutable state.
    #[cfg(not(feature = "afc-c06-gate"))]
    #[tokio::test]
    #[serial]
    async fn boots_and_answers_health_check() {
        let stub = start_stub_llm(FixtureSet::new()).await;

        let server = boot_test_server(
            &stub.base_url,
            "openai/gpt-5.4-mini",
            ServiceNeeds::default(),
        )
        .await;
        let fixture_outcome = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(async {


        let client = reqwest::Client::new();
        let resp = client
            .get(format!("{}/health", server.base_url))
            .send()
            .await
            .expect("health request");
        assert!(resp.status().is_success());
        })).await;
        server.finish_case(fixture_outcome).await

    }

    #[cfg(not(feature = "afc-c06-gate"))]
    #[tokio::test]
    #[serial]
    async fn chat_completion_flows_through_the_real_server_to_the_stub() {
        // The real orchestrator (a) strips the `provider/` prefix before the
        // wire request — the stub sees the bare model name, not
        // "openai/gpt-5.4-mini" — and (b) always attaches a tool schema
        // (native skills / MCP tools are registered unconditionally, per
        // CLAUDE.md's "tools are non-optional" design), so `has_tools` is
        // effectively always `true` for real requests. Discovered by running
        // this test against the real server, not assumed — see
        // appstate-field-plan.md addendum.
        let fixtures = FixtureSet::new().with(
            RequestFingerprint {
                model: "gpt-5.4-mini".to_string(),
                last_user_message: "ping".to_string(),
                has_tools: true,
                has_tool_result: false,
            },
            FixtureResponse::Content("pong".to_string()),
        );
        let stub = start_stub_llm(fixtures).await;
        let server = boot_test_server(
            &stub.base_url,
            "openai/gpt-5.4-mini",
            ServiceNeeds::default(),
        )
        .await;
        let fixture_outcome = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(async {


        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/api/chat/completion", server.base_url))
            .json(&serde_json::json!({
                "model": "openai/gpt-5.4-mini",
                "messages": [{"role": "user", "content": "ping"}],
                "stream": false,
            }))
            .send()
            .await
            .expect("chat completion request");

        assert!(
            resp.status().is_success(),
            "expected 2xx, got {} — body: {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        );
        })).await;
        server.finish_case(fixture_outcome).await

    }

    #[cfg(not(feature = "afc-c06-gate"))]
    #[tokio::test]
    #[serial]
    async fn caller_owned_http_cancellation_remains_nonterminating_before_sigint() {
        let stub = start_stub_llm(FixtureSet::new()).await;
        let scratch = tempfile::tempdir().expect("create SIGINT persistence scratch");
        let server = boot_test_server_process(
            &stub.base_url,
            "openai/gpt-5.4-mini",
            ServiceNeeds::default(),
            &scratch.path().join("surrealkv"),
        )
        .await;

        server.shutdown_with_signal("INT").await;
    }

    #[tokio::test]
    async fn process_server_helper() {
        if std::env::var_os("UAR_TEST_SERVER_CHILD").is_none() {
            return;
        }

        let llm_base_url =
            std::env::var("UAR_TEST_SERVER_LLM_BASE_URL").expect("child-server LLM base URL");
        let llm_model = std::env::var("UAR_TEST_SERVER_LLM_MODEL").expect("child-server LLM model");
        let needs = ServiceNeeds {
            memory: std::env::var("UAR_TEST_SERVER_MEMORY").as_deref() == Ok("1"),
        };
        let persistence_path = std::path::PathBuf::from(
            std::env::var_os("UAR_TEST_SERVER_PERSISTENCE_PATH")
                .expect("child-server persistence path"),
        );
        let control_address = std::env::var("UAR_TEST_SERVER_CONTROL_ADDRESS").expect("child control address");
        let control_token = std::env::var("UAR_TEST_SERVER_CONTROL_TOKEN").expect("child control token");

        let service_instance_id = std::env::var("UAR_TEST_SERVER_INSTANCE_ID").ok();
        let http_port = std::env::var("UAR_TEST_SERVER_HTTP_PORT")
            .ok()
            .map(|value| value.parse::<u16>().expect("child-server HTTP port"));
        let file_root = std::env::var_os("UAR_TEST_SERVER_FILE_ROOT").map(std::path::PathBuf::from);
        let server = boot_test_server_with_persistence_path_and_instance_id(
            &llm_base_url,
            &llm_model,
            needs,
            &persistence_path,
            service_instance_id.as_deref(),
            http_port,
            file_root.as_deref(),
        )
        .await;
        let stream = tokio::net::TcpStream::connect(control_address).await.expect("connect child control");
        let mut control = tokio::io::BufReader::new(stream);
        control.get_mut().write_all(format!("{control_token} {}\n", server.base_url).as_bytes()).await.expect("publish child readiness");
        let mut command = String::new();
        control.read_line(&mut command).await.expect("receive shutdown command");
        assert_eq!(command, "SHUTDOWN\n");
        server.close().await;
        control.get_mut().write_all(b"RESOURCES_RELEASED\n").await.expect("publish resource release");
        command.clear();
        control.read_line(&mut command).await.expect("receive exit acknowledgement");
        assert_eq!(command, "ALLOW_EXIT\n");
    }
}
