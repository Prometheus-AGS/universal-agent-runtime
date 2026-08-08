//! Minimal real-server boot harness for the live integration tier's baseline
//! feature cases (live-integration-baseline-coverage, task group 1).
//!
//! Rather than hand-construct `AppState` field-by-field (its ~20 fields
//! include several whose constructors are non-obvious — see
//! `openspec/changes/live-integration-baseline-coverage/appstate-field-plan.md`
//! for that research), this harness calls the actual public
//! `universal_agent_runtime::server::start_server` entry point — the exact
//! function production uses — against a config built entirely from an
//! explicit temp YAML file. This is both simpler and a stronger proof: the
//! baseline cases exercise the real boot path, not a hand-approximated one.
//!
//! `start_server`'s handler functions (`api_chat_completion`, etc.) are
//! `pub(crate)`, so this harness cannot call them directly even if it wanted
//! to — which is fine, since HTTP-level testing never needs to.

use std::sync::Once;
use std::time::Duration;

use universal_agent_runtime::config::Cli;
use universal_agent_runtime::config_manager::ConfigManager;
use universal_agent_runtime::server::start_server;

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
/// `start_server`'s returned future is `!Send` (a `tracing::info!` call in
/// its body holds a non-`Send` value across an `.await`), so it cannot be
/// handed to `tokio::spawn` directly. Rather than touch `src/server.rs` to
/// chase that down (out of scope for a test-infra change — see design.md),
/// this harness runs it on a dedicated OS thread with its own
/// current-thread Tokio runtime, which has no `Send` requirement on the
/// future it drives.
pub struct TestServerHandle {
    pub base_url: String,
    // Held only to keep the background thread from being detached from
    // view; `start_server` has no external shutdown hook (it listens for
    // process-level SIGINT/SIGTERM internally), so there is nothing
    // graceful to signal here. The thread is intentionally not joined on
    // drop — it lives for the remainder of the test binary's process,
    // which is acceptable for an 8-case test suite each on its own
    // ephemeral port and temp config/db path.
    _thread: std::thread::JoinHandle<()>,
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

fn find_free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("bind ephemeral port")
        .local_addr()
        .expect("local_addr")
        .port()
    // Listener drops here. The tiny loopback race window between this and
    // start_server re-binding is the same trade-off `uar-sidecar` accepts
    // for its own pre-bind-discover-drop-rebind dance (see start_server's
    // doc comment for start_server_sidecar).
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
    init_tracing_once();
    SCRATCH_SWEEP.call_once(sweep_stale_scratch);
    let port = find_free_port();
    let persistence_path = unique_temp_path("persistence");

    let memory_yaml = if needs.memory {
        let memory_path = unique_temp_path("memory");
        format!(
            "\nmemory:\n  enabled: true\n  db_path: \"surrealkv://{}\"\n  embedding_provider: \"local\"\n",
            memory_path.display()
        )
    } else {
        String::new()
    };

    let yaml = format!(
        "security:\n  jwt_required: false\n  jwt_secret: \"{HARNESS_JWT_SECRET}\"\n\
         resilience:\n  rate_limit_enabled: false\n\
         persistence:\n  provider: \"surreal\"\n  database_url: \"surrealkv://{}\"\n\
         llm:\n  model: \"{llm_model}\"\n  base_url: \"{llm_base_url}\"\n\
         server:\n  host: \"127.0.0.1\"\n  port: {port}\n\
         {memory_yaml}",
        persistence_path.display(),
    );

    let config_path = unique_temp_path("config").with_extension("yaml");
    std::fs::write(&config_path, yaml).expect("write temp harness config");

    let cli = Cli {
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

    // `start_server` takes an `Arc<ConfigManager>` (hot-reload via arc-swap,
    // f53b988). `load_without_watcher` is the documented test constructor: the
    // harness writes a throwaway temp config that never changes, so a file
    // watcher would be pure overhead — and one watcher task per booted server
    // would leak across the tier's many `boot_test_server` calls.
    let config = ConfigManager::load_without_watcher(cli)
        .await
        .expect("load harness config");

    let thread = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build current-thread runtime for start_server");
        rt.block_on(async move {
            if let Err(e) = start_server(config).await {
                // The test-side health-check poll surfaces a boot failure
                // as a clear timeout+panic; this eprintln gives the actual
                // cause when debugging a failed run.
                eprintln!("start_server exited with error: {e:?}");
            }
        });
    });

    let base_url = format!("http://127.0.0.1:{port}");
    wait_for_health(&base_url).await;

    TestServerHandle {
        base_url,
        _thread: thread,
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
}
