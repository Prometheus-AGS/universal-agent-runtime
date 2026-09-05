//! A2A JSON-RPC 2.0 HTTP client.
//!
//! Sends `message/send` and `tasks/get` requests to remote agents that
//! implement the A2A protocol.  Responses are normalised into the shared
//! domain types defined in [`super::types`].

use anyhow::{Context, Result, anyhow};
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

use super::contract::{
    UAR_DELEGATION_ACK_METADATA, UAR_DELEGATION_CONTRACT_METADATA, UarDelegationAcknowledgement,
    UarDelegationContract,
};
use super::types::{Message, Task, TaskState};

/// HTTP client for the A2A JSON-RPC 2.0 protocol.
#[derive(Clone)]
pub struct A2AClient {
    http: Client,
    /// Optional bearer token attached to every request.
    bearer_token: Option<String>,
}

impl std::fmt::Debug for A2AClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("A2AClient")
            .field("authenticated", &self.bearer_token.is_some())
            .finish_non_exhaustive()
    }
}

impl A2AClient {
    /// Create a new client with default timeout (30 s).
    #[expect(
        clippy::expect_used,
        reason = "reqwest default builder is infallible in practice"
    )]
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

        // The peer's response is not a receipt for this operation unless it
        // carries the exact request identity and the expected wire version.
        if json.get("jsonrpc").and_then(|value| value.as_str()) != Some("2.0")
            || json.get("id").and_then(|value| value.as_str()) != Some(id.as_str())
        {
            return Err(anyhow!("A2A response does not match this JSON-RPC request"));
        }

        if !status.is_success() {
            if let Some(err) = json.get("error") {
                return Err(anyhow!(
                    "A2A error {}: {}",
                    err.get("code").and_then(|c| c.as_i64()).unwrap_or(-1),
                    err.get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("unknown")
                ));
            }
            return Err(anyhow!("A2A request failed with HTTP {status}"));
        }

        if let Some(err) = json.get("error") {
            return Err(anyhow!(
                "A2A JSON-RPC error {}: {}",
                err.get("code").and_then(|c| c.as_i64()).unwrap_or(-1),
                err.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("unknown")
            ));
        }

        json.get("result")
            .cloned()
            .ok_or_else(|| anyhow!("A2A response missing 'result' field"))
    }

    // ── Public API ───────────────────────────────────────────────────────────

    /// Bind one resumable remote task to the parent's cancellation lifetime.
    /// The trusted host must retain and drive this object through cleanup;
    /// this method neither starts a detached job nor grants remote execution.
    #[must_use]
    pub fn task_execution(
        &self,
        url: String,
        message: Message,
        parent: &tokio_util::sync::CancellationToken,
    ) -> super::task_execution::A2ATaskExecution {
        super::task_execution::A2ATaskExecution::new(
            self.clone(),
            url,
            message,
            None,
            None,
            None,
            parent,
        )
    }

    /// Bind a governed UAR-peer task. Unlike generic A2A, every receipt must
    /// acknowledge the exact inherited contract and this client must be
    /// authenticated.
    pub fn governed_task_execution(
        &self,
        url: String,
        message: Message,
        contract: UarDelegationContract,
        parent: &tokio_util::sync::CancellationToken,
    ) -> anyhow::Result<super::task_execution::A2ATaskExecution> {
        anyhow::ensure!(
            self.bearer_token.is_some(),
            "governed A2A delegation requires authentication"
        );
        contract.validate()?;
        Ok(super::task_execution::A2ATaskExecution::new(
            self.clone(),
            url,
            message,
            Some(contract),
            None,
            None,
            parent,
        ))
    }

    /// Continue one exact governed peer task. Both identifiers come from the
    /// persisted authenticated receipt; callers cannot replace either one.
    pub(crate) fn governed_task_execution_for_task(
        &self,
        url: String,
        message: Message,
        contract: UarDelegationContract,
        task_id: String,
        context_id: String,
        parent: &tokio_util::sync::CancellationToken,
    ) -> anyhow::Result<super::task_execution::A2ATaskExecution> {
        anyhow::ensure!(
            self.bearer_token.is_some(),
            "governed A2A delegation requires authentication"
        );
        contract.validate()?;
        anyhow::ensure!(
            !task_id.trim().is_empty() && !context_id.trim().is_empty(),
            "persisted A2A task binding is invalid"
        );
        Ok(super::task_execution::A2ATaskExecution::new(
            self.clone(),
            url,
            message,
            Some(contract),
            Some(task_id),
            Some(context_id),
            parent,
        ))
    }

    pub(crate) fn governed_task_cleanup(
        &self,
        url: String,
        contract: UarDelegationContract,
        task_id: String,
        parent: &tokio_util::sync::CancellationToken,
    ) -> anyhow::Result<super::task_execution::A2ATaskExecution> {
        anyhow::ensure!(
            self.bearer_token.is_some(),
            "governed A2A delegation requires authentication"
        );
        contract.validate()?;
        anyhow::ensure!(
            !task_id.trim().is_empty(),
            "persisted A2A task binding is invalid"
        );
        Ok(super::task_execution::A2ATaskExecution::for_cleanup(
            self.clone(),
            url,
            contract,
            task_id,
            parent,
        ))
    }

    /// Send a message to a remote agent (`message/send`).
    ///
    /// Returns the `Task` that was created or updated by the remote agent.
    pub async fn send_message(&self, url: &str, message: &Message) -> Result<Task> {
        // MessageSendParams wraps the message; sending Role/Part fields at the
        // params root is rejected by this runtime's own inbound dispatcher.
        let params = json!({"message": message});
        let result = self.rpc_call(url, "message/send", params).await?;
        let task: Task = serde_json::from_value(result)
            .context("deserialise task from message/send response")?;
        anyhow::ensure!(
            !task.id.trim().is_empty(),
            "A2A task receipt has no identity"
        );
        Ok(task)
    }

    pub(super) async fn send_governed_message(
        &self,
        url: &str,
        message: &Message,
        contract: &UarDelegationContract,
        task_id: Option<&str>,
        context_id: Option<&str>,
    ) -> Result<Task> {
        anyhow::ensure!(
            self.bearer_token.is_some(),
            "governed A2A delegation requires authentication"
        );
        contract.validate()?;
        let params = json!({
            "message": message,
            "task_id": task_id,
            "context_id": context_id,
            "metadata": { UAR_DELEGATION_CONTRACT_METADATA: contract },
        });
        let result = self.rpc_call(url, "message/send", params).await?;
        let task: Task = serde_json::from_value(result)
            .context("deserialise task from governed message/send response")?;
        anyhow::ensure!(
            !task.id.trim().is_empty(),
            "A2A task receipt has no identity"
        );
        if let Some(task_id) = task_id {
            anyhow::ensure!(
                task.id == task_id,
                "A2A follow-up receipt belongs to another task"
            );
        }
        if let Some(context_id) = context_id {
            anyhow::ensure!(
                task.context_id.as_deref() == Some(context_id),
                "A2A follow-up receipt belongs to another context"
            );
        }
        validate_governed_receipt(&task, contract)?;
        Ok(task)
    }

    /// Retrieve a task by ID from a remote agent (`tasks/get`).
    pub async fn get_task(&self, url: &str, task_id: &str) -> Result<Task> {
        let params = json!({"id": task_id});
        let result = self.rpc_call(url, "tasks/get", params).await?;
        let task: Task =
            serde_json::from_value(result).context("deserialise task from tasks/get response")?;
        anyhow::ensure!(
            task.id == task_id,
            "A2A task receipt belongs to another task"
        );
        Ok(task)
    }

    pub(super) async fn get_governed_task(
        &self,
        url: &str,
        task_id: &str,
        contract: &UarDelegationContract,
    ) -> Result<Task> {
        let task = self.get_task(url, task_id).await?;
        validate_governed_receipt(&task, contract)?;
        Ok(task)
    }

    /// Cancel a task on the remote agent (`tasks/cancel`).
    pub async fn cancel_task(&self, url: &str, task_id: &str) -> Result<()> {
        let task = self.cancel_task_receipt(url, task_id).await?;
        anyhow::ensure!(
            task.status.state == TaskState::Canceled && !task.cleanup_unconfirmed(),
            "A2A peer did not confirm task cancellation"
        );
        Ok(())
    }

    pub(super) async fn cancel_governed_task_receipt(
        &self,
        url: &str,
        task_id: &str,
        contract: &UarDelegationContract,
    ) -> Result<Task> {
        let task = self.cancel_task_receipt(url, task_id).await?;
        validate_governed_receipt(&task, contract)?;
        Ok(task)
    }

    /// Request cancellation while retaining the peer's exact terminal receipt.
    /// An HTTP success alone does not prove that remote work was cancelled.
    pub async fn cancel_task_receipt(&self, url: &str, task_id: &str) -> Result<Task> {
        let params = json!({"id": task_id});
        let result = self.rpc_call(url, "tasks/cancel", params).await?;
        let task: Task = serde_json::from_value(result)
            .context("deserialise task from tasks/cancel response")?;
        anyhow::ensure!(
            task.id == task_id,
            "A2A cancellation receipt belongs to another task"
        );
        Ok(task)
    }
}

fn validate_governed_receipt(task: &Task, contract: &UarDelegationContract) -> anyhow::Result<()> {
    let acknowledgement = task
        .metadata
        .get(UAR_DELEGATION_ACK_METADATA)
        .ok_or_else(|| anyhow!("A2A peer omitted the UAR delegation acknowledgement"))?;
    let acknowledgement: UarDelegationAcknowledgement =
        serde_json::from_value(acknowledgement.clone())
            .context("A2A peer returned a malformed UAR delegation acknowledgement")?;
    acknowledgement.validate_for(contract)
}

impl Default for A2AClient {
    fn default() -> Self {
        Self::new()
    }
}
