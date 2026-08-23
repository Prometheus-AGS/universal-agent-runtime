//! Minimal real-server boot harness for the live integration tier's baseline
//! feature cases (live-integration-baseline-coverage, task group 1).
//!
//! Rather than hand-construct `AppState` field-by-field (its ~20 fields
//! include several whose constructors are non-obvious — see
//! `openspec/changes/live-integration-baseline-coverage/appstate-field-plan.md`
//! for that research), this harness calls the actual public
//! `universal_agent_runtime::server::start_server_sidecar` entry point against
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
use universal_agent_runtime::config::Cli;
use universal_agent_runtime::config_manager::ConfigManager;
use universal_agent_runtime::server::start_server_sidecar;

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
/// future it drives.
#[allow(dead_code)] // The BDD target imports this harness but only reads `base_url`.
pub struct TestServerHandle {
    pub base_url: String,
    shutdown: CancellationToken,
    // Optional so `shutdown` can move the join handle into `spawn_blocking`.
    // Existing callers may still drop the handle, which detaches the server
    // thread exactly as before.
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
    child: Option<std::process::Child>,
    control_dir: Option<tempfile::TempDir>,
}

impl ProcessTestServerHandle {
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
        let control_dir = self.control_dir.as_ref().expect("child control directory");
        std::fs::write(control_dir.path().join("shutdown"), b"shutdown")
            .expect("write child-server shutdown control");
        let mut child = self.child.take().expect("child server process");
        let http_stopped = control_dir.path().join("http-stopped");
        let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
        while !http_stopped.exists() {
            assert!(
                child
                    .try_wait()
                    .expect("poll child server process")
                    .is_none(),
                "caller-owned HTTP cancellation terminated the child process"
            );
            assert!(
                tokio::time::Instant::now() < deadline,
                "caller-owned HTTP cancellation did not stop the listeners within 10s"
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        assert!(
            child
                .try_wait()
                .expect("poll child after HTTP stop")
                .is_none(),
            "caller-owned HTTP cancellation terminated the host process"
        );
        let pre_signal_stderr =
            std::fs::read_to_string(control_dir.path().join("stderr.log")).unwrap_or_default();
        assert!(
            !pre_signal_stderr.contains("UAR_SHUTDOWN outcome=deadline_enforced"),
            "caller-owned HTTP cancellation armed the process deadline: {pre_signal_stderr}"
        );
        #[cfg(unix)]
        {
            let signal_status = std::process::Command::new("/bin/kill")
                .arg(format!("-{signal}"))
                .arg(child.id().to_string())
                .status()
                .expect("send shutdown signal to child server process");
            assert!(
                signal_status.success(),
                "failed to send child {signal}: {signal_status}"
            );
        }
        #[cfg(not(unix))]
        panic!("process-server helper requires Unix SIGTERM support");
        let resources_released = control_dir.path().join("resources-released");
        let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
        while !resources_released.exists() {
            assert!(
                child
                    .try_wait()
                    .expect("poll child at resource-release barrier")
                    .is_none(),
                "child exited before publishing pre-exit resource release"
            );
            assert!(
                tokio::time::Instant::now() < deadline,
                "child did not release server resources within 10s"
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        assert!(
            child
                .try_wait()
                .expect("poll child at pre-exit barrier")
                .is_none(),
            "child process exited instead of waiting at the pre-exit barrier"
        );
        ProcessTestServerExitBarrier {
            child: Some(child),
            control_dir: self.control_dir.take(),
        }
    }
}

#[allow(dead_code)] // Consumed by capability cases, which the BDD target omits.
pub struct ProcessTestServerExitBarrier {
    child: Option<std::process::Child>,
    control_dir: Option<tempfile::TempDir>,
}

impl ProcessTestServerExitBarrier {
    #[allow(dead_code)] // Consumed by capability cases, which the BDD target omits.
    pub async fn allow_exit(mut self) {
        let control_dir = self.control_dir.take().expect("child control directory");
        std::fs::write(control_dir.path().join("allow-exit"), b"exit")
            .expect("release child pre-exit barrier");
        let mut child = self.child.take().expect("child server process");
        let status = tokio::task::spawn_blocking(move || child.wait())
            .await
            .expect("join child-server wait task")
            .expect("wait for child server process");
        assert!(
            status.success(),
            "child server process exited unsuccessfully: {status}"
        );
    }
}

impl Drop for ProcessTestServerExitBarrier {
    fn drop(&mut self) {
        let Some(child) = self.child.as_mut() else {
            return;
        };
        let _ = child.kill();
        let _ = child.wait();
    }
}

impl Drop for ProcessTestServerHandle {
    fn drop(&mut self) {
        let Some(child) = self.child.as_mut() else {
            return;
        };
        if let Some(control_dir) = self.control_dir.as_ref() {
            let _ = std::fs::write(control_dir.path().join("shutdown"), b"shutdown");
        }
        for _ in 0..20 {
            if child.try_wait().ok().flatten().is_some() {
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        // A panicking test must not strand its exclusively-owned helper
        // process. Graceful control is attempted first; kill is the bounded
        // cleanup fallback only when that child did not exit within one second.
        let _ = child.kill();
        let _ = child.wait();
    }
}

impl TestServerHandle {
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
    boot_test_server_inner(llm_base_url, llm_model, needs, persistence_path).await
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
    boot_test_server_inner(
        llm_base_url,
        llm_model,
        needs,
        persistence_path.to_path_buf(),
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
    let control_dir = tempfile::tempdir().expect("create child-server control directory");
    let ready_path = control_dir.path().join("ready");
    let stderr_file = std::fs::File::create(control_dir.path().join("stderr.log"))
        .expect("create child-server stderr capture");
    let mut child = std::process::Command::new(
        std::env::current_exe().expect("resolve integration test executable"),
    )
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
    .env("UAR_TEST_SERVER_CONTROL_DIR", control_dir.path())
    .stderr(std::process::Stdio::from(stderr_file))
    .spawn()
    .expect("spawn child server process");

    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    loop {
        if let Ok(base_url) = std::fs::read_to_string(&ready_path) {
            return ProcessTestServerHandle {
                base_url,
                child: Some(child),
                control_dir: Some(control_dir),
            };
        }
        if let Some(status) = child.try_wait().expect("poll child server process") {
            let stderr = std::fs::read_to_string(control_dir.path().join("stderr.log"))
                .unwrap_or_else(|error| format!("<failed to read child stderr: {error}>"));
            panic!("child server exited before readiness: {status}\n{stderr}");
        }
        if tokio::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("child server did not become ready within 60s");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn boot_test_server_inner(
    llm_base_url: &str,
    llm_model: &str,
    needs: ServiceNeeds,
    persistence_path: std::path::PathBuf,
) -> TestServerHandle {
    init_tracing_once();
    SCRATCH_SWEEP.call_once(sweep_stale_scratch);
    let listener =
        std::net::TcpListener::bind("127.0.0.1:0").expect("bind ephemeral harness listener");
    listener
        .set_nonblocking(true)
        .expect("set harness listener nonblocking");
    let port = listener
        .local_addr()
        .expect("harness listener address")
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

    let yaml = format!(
        "security:\n  jwt_required: false\n  jwt_secret: \"{HARNESS_JWT_SECRET}\"\n\
         resilience:\n  rate_limit_enabled: false\n\
         persistence:\n  provider: \"surreal\"\n  database_url: \"surrealkv://{}\"\n\
         acp:\n  enabled: true\n  path: \"/acp\"\n  auth_required: true\n\
         llm:\n  model: \"{llm_model}\"\n  base_url: \"{llm_base_url}\"\n\
         server:\n  host: \"127.0.0.1\"\n  port: {port}\n  shutdown_timeout_secs: 1\n\
         {memory_yaml}",
        persistence_path.display(),
    );

    let config_path = unique_temp_path("config").with_extension("yaml");
    std::fs::write(&config_path, yaml).expect("write temp harness config");

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
    let thread = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build current-thread runtime for server");
        rt.block_on(async move {
            let listener = tokio::net::TcpListener::from_std(listener)
                .expect("register harness listener with Tokio");
            let result =
                start_server_sidecar(config, listener, ready_tx, Some(thread_shutdown)).await;
            if let Err(error) = &result {
                eprintln!("start_server_sidecar exited with error: {error:?}");
            }
            result
        })
    });

    let ready_addr = tokio::time::timeout(Duration::from_secs(30), ready_rx)
        .await
        .expect("server did not signal readiness within 30s")
        .expect("server exited before signaling readiness");
    let base_url = format!("http://{ready_addr}");
    wait_for_health(&base_url).await;

    TestServerHandle {
        base_url,
        shutdown,
        thread: Some(thread),
    }
}

/// Poll `/health` until it responds or a 30s timeout elapses, closing the
/// race window between the harness starting the server task and the
/// listener actually accepting connections. 30s (was 10s): boot now loads
/// the real 34MB BGE embedding model and builds an ONNX Runtime session
/// (fix-embeddings-fastembed), which legitimately adds several seconds on
/// slower machines.
async fn wait_for_health(base_url: &str) {
    let client = reqwest::Client::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        if let Ok(resp) = client.get(format!("{base_url}/health")).send().await
            && resp.status().is_success()
        {
            return;
        }
        if tokio::time::Instant::now() >= deadline {
            panic!("server at {base_url} did not become healthy within 30s");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
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
    use crate::live::stub_llm::{FixtureResponse, RequestFingerprint};
    #[allow(
        unused_imports,
        reason = "the Cucumber custom test harness compiles this unit-test module without executing it"
    )]
    use crate::live::stub_llm::{FixtureSet, start_stub_llm};
    use serial_test::serial;

    // #[serial]: booting a real server (real embedded SurrealDB, real
    // orchestrator, real MCP subprocess spawns) is heavy enough that running
    // several concurrently causes health-check timeouts under cargo test's
    // default parallelism — confirmed by a full-suite run that passed each
    // module individually but failed 7/16 tests together. Every test that
    // calls boot_test_server (here and in baseline_cases.rs) is #[serial]
    // for this reason, not for shared mutable state.
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

        let client = reqwest::Client::new();
        let resp = client
            .get(format!("{}/health", server.base_url))
            .send()
            .await
            .expect("health request");
        assert!(resp.status().is_success());
    }

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
    }

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
        let control_dir = std::path::PathBuf::from(
            std::env::var_os("UAR_TEST_SERVER_CONTROL_DIR")
                .expect("child-server control directory"),
        );

        let server = boot_test_server_with_persistence_path(
            &llm_base_url,
            &llm_model,
            needs,
            &persistence_path,
        )
        .await;
        std::fs::write(control_dir.join("ready"), &server.base_url)
            .expect("publish child-server readiness");

        while !control_dir.join("shutdown").exists() {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        let client = reqwest::Client::new();
        let base_url = server.base_url.clone();
        server.trigger_shutdown();

        let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
        loop {
            if client
                .get(format!("{base_url}/health"))
                .send()
                .await
                .is_err()
            {
                break;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "caller token did not stop the child HTTP listener within 10s"
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        std::fs::write(control_dir.join("http-stopped"), b"stopped")
            .expect("publish caller-owned HTTP stop");
        server.wait_for_exit().await;
        std::fs::write(control_dir.join("resources-released"), b"released")
            .expect("publish pre-exit resource release");
        while !control_dir.join("allow-exit").exists() {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }
}
