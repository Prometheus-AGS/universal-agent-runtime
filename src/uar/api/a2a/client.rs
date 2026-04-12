//! A2A JSON-RPC 2.0 HTTP client.
//!
//! Sends `message/send` and `tasks/get` requests to remote agents that
//! implement the A2A protocol.  Responses are normalised into the shared
//! domain types defined in [`super::types`].

use anyhow::{Context, Result, anyhow};
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

use super::types::{Message, Task};

/// HTTP client for the A2A JSON-RPC 2.0 protocol.
#[derive(Debug, Clone)]
pub struct A2AClient {
    http: Client,
    /// Optional bearer token attached to every request.
    bearer_token: Option<String>,
}

impl A2AClient {
    /// Create a new client with default timeout (30 s).
    pub fn new() -> Self {
        Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("failed to build reqwest client"),
            bearer_token: None,
        }
    }

    /// Attach a bearer token to all outgoing requests.
    #[must_use]
    pub fn with_bearer_token(mut self, token: impl Into<String>) -> Self {
        self.bearer_token = Some(token.into());
        self
    }

    // ── Core JSON-RPC helper ─────────────────────────────────────────────────

    async fn rpc_call(
        &self,
        url: &str,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let id = Uuid::new_v4().to_string();
        let body = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        let mut req = self.http.post(url).json(&body);
        if let Some(token) = &self.bearer_token {
            req = req.bearer_auth(token);
        }

        let resp = req.send().await.context("A2A HTTP request failed")?;
        let status = resp.status();

        let json: serde_json::Value = resp
            .json()
            .await
            .context("A2A response body is not valid JSON")?;

        if !status.is_success() {
            if let Some(err) = json.get("error") {
                return Err(anyhow!(
                    "A2A error {}: {}",
                    err.get("code").and_then(|c| c.as_i64()).unwrap_or(-1),
                    err.get("message").and_then(|m| m.as_str()).unwrap_or("unknown")
                ));
            }
            return Err(anyhow!("A2A request failed with HTTP {status}"));
        }

        if let Some(err) = json.get("error") {
            return Err(anyhow!(
                "A2A JSON-RPC error {}: {}",
                err.get("code").and_then(|c| c.as_i64()).unwrap_or(-1),
                err.get("message").and_then(|m| m.as_str()).unwrap_or("unknown")
            ));
        }

        json.get("result")
            .cloned()
            .ok_or_else(|| anyhow!("A2A response missing 'result' field"))
    }

    // ── Public API ───────────────────────────────────────────────────────────

    /// Send a message to a remote agent (`message/send`).
    ///
    /// Returns the `Task` that was created or updated by the remote agent.
    pub async fn send_message(&self, url: &str, message: &Message) -> Result<Task> {
        let params = serde_json::to_value(message).context("serialise message")?;
        let result = self.rpc_call(url, "message/send", params).await?;
        serde_json::from_value(result).context("deserialise task from message/send response")
    }

    /// Retrieve a task by ID from a remote agent (`tasks/get`).
    pub async fn get_task(&self, url: &str, task_id: &str) -> Result<Task> {
        let params = json!({"id": task_id});
        let result = self.rpc_call(url, "tasks/get", params).await?;
        serde_json::from_value(result).context("deserialise task from tasks/get response")
    }

    /// Cancel a task on the remote agent (`tasks/cancel`).
    pub async fn cancel_task(&self, url: &str, task_id: &str) -> Result<()> {
        let params = json!({"id": task_id});
        self.rpc_call(url, "tasks/cancel", params).await?;
        Ok(())
    }
}

impl Default for A2AClient {
    fn default() -> Self {
        Self::new()
    }
}
