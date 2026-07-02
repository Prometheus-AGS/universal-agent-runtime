//! Backend selection for the live integration tier (proxy-integration-gate,
//! design.md D1/D2).
//!
//! `recorded` and `live` are the same code path — a real HTTP client against
//! some `base_url` — differing only in what that `base_url` resolves to.
//! This module owns picking that URL (and keeping the stub server alive for
//! the `recorded` case); task-group-3 test cases call [`resolve`] instead of
//! hard-coding either endpoint.

use super::stub_llm::{FixtureSet, StubLlmServer, start_stub_llm};

/// Env var selecting the backend. `recorded` (default) uses the in-process
/// stub server; `live` targets the real local proxy.
pub const BACKEND_ENV_VAR: &str = "UAR_LIVE_INTEGRATION_BACKEND";

/// Default local proxy address used by the `live` backend
/// (`ai.prometheus.openai-proxy`, see `scripts/live-integration.sh`).
pub const LIVE_PROXY_BASE_URL: &str = "http://127.0.0.1:8181/v1";

/// Model used for both backends — the stub doesn't care about the model
/// name, and this matches the proxy's routed model for live runs.
pub const LIVE_MODEL: &str = "openai/gpt-5.4-mini";

/// A resolved backend: the `base_url` to point `UAR_LLM__BASE_URL` at, plus
/// (for `recorded`) the running stub server that must stay alive for the
/// duration of the test — dropping it stops the server.
pub struct ResolvedBackend {
    pub base_url: String,
    pub model: String,
    /// `None` for `live` (no local server to keep alive); `Some` for
    /// `recorded` (drop-guard for the stub server task).
    _stub: Option<StubLlmServer>,
}

/// Resolve which backend to use from `UAR_LIVE_INTEGRATION_BACKEND`
/// (default: `recorded`), starting the in-process stub server with the given
/// fixtures when recorded. `live` ignores `fixtures` entirely — the real
/// proxy needs no canned responses.
pub async fn resolve(fixtures: FixtureSet) -> ResolvedBackend {
    let mode = std::env::var(BACKEND_ENV_VAR).unwrap_or_else(|_| "recorded".to_string());

    match mode.as_str() {
        "live" => ResolvedBackend {
            base_url: LIVE_PROXY_BASE_URL.to_string(),
            model: LIVE_MODEL.to_string(),
            _stub: None,
        },
        // Anything other than "live" (including unset / "recorded" / typos)
        // falls back to recorded — this tier must never silently hit a real
        // model when the operator didn't explicitly ask for it.
        _ => {
            let stub = start_stub_llm(fixtures).await;
            let base_url = stub.base_url.clone();
            ResolvedBackend {
                base_url,
                model: LIVE_MODEL.to_string(),
                _stub: Some(stub),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::live::stub_llm::{FixtureResponse, RequestFingerprint};
    use serial_test::serial;

    // `BACKEND_ENV_VAR` is process-global state; `#[serial]` prevents these
    // tests racing with each other (or with a real `UAR_LIVE_INTEGRATION_BACKEND`
    // left set in the environment by a different test run in the same binary).
    #[tokio::test]
    #[serial]
    async fn defaults_to_recorded_when_env_var_unset() {
        // SAFETY: test-only env mutation, guarded by #[serial].
        unsafe { std::env::remove_var(BACKEND_ENV_VAR) };

        let backend = resolve(FixtureSet::new()).await;

        assert_ne!(backend.base_url, LIVE_PROXY_BASE_URL);
        assert!(
            backend._stub.is_some(),
            "recorded backend must start the stub server"
        );
    }

    #[tokio::test]
    #[serial]
    async fn explicit_recorded_matches_default() {
        // SAFETY: test-only env mutation, guarded by #[serial].
        unsafe { std::env::set_var(BACKEND_ENV_VAR, "recorded") };

        let backend = resolve(FixtureSet::new()).await;

        assert_ne!(backend.base_url, LIVE_PROXY_BASE_URL);
        assert!(backend._stub.is_some());

        unsafe { std::env::remove_var(BACKEND_ENV_VAR) };
    }

    #[tokio::test]
    #[serial]
    async fn live_targets_the_real_proxy_and_starts_no_stub() {
        // SAFETY: test-only env mutation, guarded by #[serial].
        unsafe { std::env::set_var(BACKEND_ENV_VAR, "live") };

        let backend = resolve(FixtureSet::new()).await;

        assert_eq!(backend.base_url, LIVE_PROXY_BASE_URL);
        assert!(
            backend._stub.is_none(),
            "live backend must not start an in-process server"
        );

        unsafe { std::env::remove_var(BACKEND_ENV_VAR) };
    }

    #[tokio::test]
    #[serial]
    async fn recorded_backend_actually_serves_configured_fixtures() {
        // SAFETY: test-only env mutation, guarded by #[serial].
        unsafe { std::env::remove_var(BACKEND_ENV_VAR) };

        let fixtures = FixtureSet::new().with(
            RequestFingerprint {
                model: LIVE_MODEL.to_string(),
                last_user_message: "ping".to_string(),
                has_tools: false,
                has_tool_result: false,
            },
            FixtureResponse::Content("pong".to_string()),
        );
        let backend = resolve(fixtures).await;

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/chat/completions", backend.base_url))
            .json(&serde_json::json!({
                "model": backend.model,
                "messages": [{"role": "user", "content": "ping"}],
                "stream": false,
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
    }
}
