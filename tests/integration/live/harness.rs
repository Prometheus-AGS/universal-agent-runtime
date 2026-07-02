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

use std::sync::Arc;
use std::time::Duration;

use universal_agent_runtime::config::{AppConfig, Cli};
use universal_agent_runtime::server::start_server;

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
/// `AppConfig::load_with_cli` falls back to probing `./config.yaml` and
/// `~/.uar/config.yaml` when no `--config` is given, and both exist on at
/// least one developer machine this harness was built on. An explicit
/// `cli.config` path is the only way to guarantee this harness never reads
/// someone's real local configuration.
pub async fn boot_test_server(
    llm_base_url: &str,
    llm_model: &str,
    needs: ServiceNeeds,
) -> TestServerHandle {
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
        "security:\n  jwt_required: false\n  jwt_secret: \"test-secret-not-for-production\"\n\
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
        command: None,
    };

    let config = Arc::new(AppConfig::load_with_cli(cli).expect("load harness config"));

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

/// Poll `/health` until it responds or a 10s timeout elapses, closing the
/// race window between the harness starting the server task and the
/// listener actually accepting connections.
async fn wait_for_health(base_url: &str) {
    let client = reqwest::Client::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        if let Ok(resp) = client.get(format!("{base_url}/health")).send().await
            && resp.status().is_success()
        {
            return;
        }
        if tokio::time::Instant::now() >= deadline {
            panic!("server at {base_url} did not become healthy within 10s");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::live::stub_llm::{FixtureResponse, RequestFingerprint};
    use crate::live::stub_llm::{FixtureSet, start_stub_llm};

    #[tokio::test]
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
