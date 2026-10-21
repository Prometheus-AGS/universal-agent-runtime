//! A2A JSON-RPC 2.0 HTTP client.
//!
//! Sends negotiated A2A v1 requests and consumes task event streams from peers that
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

#[path = "client_stream.rs"]
mod streaming;
pub(super) use streaming::A2ATaskStream;

/// Rejection or unavailable discovery before any task mutation was dispatched.
#[derive(Debug, thiserror::Error)]
#[error("required A2A streaming interface is unavailable: {0}")]
pub struct UnsupportedStreamingPeer(#[source] anyhow::Error);

/// An authoritative, exact A2A v1 JSON-RPC streaming interface.
#[derive(Clone, Debug)]
pub struct StreamingPeer {
    endpoint: String,
    tenant: Option<String>,
}

/// HTTP client for the A2A JSON-RPC 2.0 protocol.
#[derive(Clone)]
pub struct A2AClient {
    http: Client,
    peers: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, StreamingPeer>>>,
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
    /// Create a client without a wall-clock completion deadline.
    /// The host cancellation lifetime owns remote cancellation and settlement.
    #[expect(
        clippy::expect_used,
        reason = "reqwest default builder is infallible in practice"
    )]
    pub fn new() -> Self {
        Self {
            http: Client::builder()
                .build()
                .expect("failed to build reqwest client"),
            bearer_token: None,
            peers: Default::default(),
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

        let mut req = self.http.post(url).header("A2A-Version", "1.0").json(&body);
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
        Ok(super::task_execution::A2ATaskExecution::for_cleanup(
            self.clone(),
            url,
            contract,
            task_id,
            context_id,
            parent,
        ))
    }

    /// Resolve the standard discovery card and require this exact advertised
    /// interface before the host creates a child or dispatches a mutation.
    /// Legacy endpoints are not rewritten to another agent's advertised URL.
    pub async fn require_streaming_peer(&self, url: &str) -> Result<StreamingPeer, UnsupportedStreamingPeer> {
        self.discover_streaming_peer(url).await.map_err(UnsupportedStreamingPeer)
    }

    async fn discover_streaming_peer(&self, url: &str) -> Result<StreamingPeer> {
        let endpoint = reqwest::Url::parse(url).context("invalid A2A endpoint")?;
        anyhow::ensure!(matches!(endpoint.scheme(), "http" | "https")
            && endpoint.username().is_empty() && endpoint.password().is_none()
            && endpoint.fragment().is_none(), "invalid A2A HTTP endpoint");
        let card_url = endpoint.join("/.well-known/agent-card.json")?;
        let mut request = self.http.get(card_url).header("A2A-Version", "1.0");
        if let Some(token) = &self.bearer_token { request = request.bearer_auth(token); }
        let response = request.send().await.context("A2A discovery unavailable")?;
        anyhow::ensure!(response.status().is_success(), "A2A discovery rejected");
        let card: serde_json::Value = response.json().await.context("invalid A2A AgentCard")?;
        anyhow::ensure!(card.pointer("/capabilities/streaming").and_then(serde_json::Value::as_bool) == Some(true),
            "A2A peer does not support required event streaming");
        let interface = card.get("supportedInterfaces").and_then(serde_json::Value::as_array)
            .and_then(|interfaces| interfaces.iter().find(|interface| {
                interface.get("protocolBinding").and_then(serde_json::Value::as_str) == Some("JSONRPC")
                    && interface.get("protocolVersion").and_then(serde_json::Value::as_str) == Some("1.0")
                    && interface.get("url").and_then(serde_json::Value::as_str)
                        .and_then(|url| reqwest::Url::parse(url).ok()).as_ref() == Some(&endpoint)
            })).ok_or_else(|| anyhow!("A2A peer does not advertise this exact v1.0 JSONRPC interface"))?;
        let peer = StreamingPeer {
            endpoint: endpoint.to_string(),
            tenant: interface.get("tenant").and_then(serde_json::Value::as_str).map(str::to_owned),
        };
        self.peers.write().await.insert(url.to_owned(), peer.clone());
        Ok(peer)
    }

    async fn streaming_peer(&self, url: &str) -> Result<StreamingPeer> {
        if let Some(peer) = self.peers.read().await.get(url).cloned() { return Ok(peer); }
        self.require_streaming_peer(url).await.map_err(Into::into)
    }

    async fn task_params(&self, url: &str, task_id: &str) -> Result<serde_json::Value> {
        let mut params = json!({"id": task_id});
        if let Some(peer) = self.peers.read().await.get(url)
            && let Some(tenant) = &peer.tenant { params["tenant"] = json!(tenant); }
        Ok(params)
    }

    /// Send one task turn using the negotiated v1 interface. A missing response
    /// is uncertain creation; callers must never automatically replay it.
    pub async fn send_message(&self, url: &str, message: &Message) -> Result<Task> {
        self.send_streaming_capable_message(url, message, None, None, None).await
    }

    pub(super) async fn send_governed_message(
        &self, url: &str, message: &Message, contract: &UarDelegationContract,
        task_id: Option<&str>, context_id: Option<&str>,
    ) -> Result<Task> {
        anyhow::ensure!(self.bearer_token.is_some(), "governed A2A delegation requires authentication");
        contract.validate()?;
        self.send_streaming_capable_message(url, message, Some(contract), task_id, context_id).await
    }

    async fn send_streaming_capable_message(
        &self, url: &str, message: &Message, contract: Option<&UarDelegationContract>,
        task_id: Option<&str>, context_id: Option<&str>,
    ) -> Result<Task> {
        let peer = self.streaming_peer(url).await?;
        let mut params = json!({
            "message": streaming::message(message, task_id, context_id)?,
            "configuration": {"returnImmediately": true},
        });
        if let Some(tenant) = peer.tenant { params["tenant"] = json!(tenant); }
        if let Some(contract) = contract { params["metadata"] = json!({ UAR_DELEGATION_CONTRACT_METADATA: contract }); }
        let result = self.rpc_call(&peer.endpoint, "SendMessage", params).await?;
        let task = streaming::task(result.get("task").ok_or_else(|| anyhow!("A2A send did not return a task receipt"))?)?;
        validate_binding(&task, task_id, context_id)?;
        if let Some(contract) = contract { validate_governed_receipt(&task, contract)?; }
        Ok(task)
    }

    pub(super) async fn subscribe_task(
        &self, url: &str, initial: Task, contract: Option<UarDelegationContract>,
    ) -> Result<A2ATaskStream> {
        let peer = self.streaming_peer(url).await?;
        let id = Uuid::new_v4().to_string();
        let mut params = json!({"id": initial.id});
        if let Some(tenant) = peer.tenant { params["tenant"] = json!(tenant); }
        let mut request = self.http.post(&peer.endpoint).header("A2A-Version", "1.0")
            .header(reqwest::header::ACCEPT, "text/event-stream")
            .json(&json!({"jsonrpc": "2.0", "id": id, "method": "SubscribeToTask", "params": params}));
        if let Some(token) = &self.bearer_token { request = request.bearer_auth(token); }
        let response = request.send().await.context("A2A subscription unavailable")?;
        anyhow::ensure!(response.status().is_success(), "A2A subscription rejected");
        anyhow::ensure!(response.headers().get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()).is_some_and(|value| value.split(';').next() == Some("text/event-stream")),
            "A2A subscription did not return an event stream");
        Ok(A2ATaskStream::new(response, id, initial, contract))
    }

    /// Retrieve a task by ID from a remote agent (`GetTask`).
    pub async fn get_task(&self, url: &str, task_id: &str) -> Result<Task> {
        let params = self.task_params(url, task_id).await?;
        let result = self.rpc_call(url, "GetTask", params).await?;
        let task = streaming::task(&result)?;
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

    /// Cancel a task on the remote agent (`CancelTask`).
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
        let params = self.task_params(url, task_id).await?;
        let result = self.rpc_call(url, "CancelTask", params).await?;
        let task = streaming::task(&result)?;
        anyhow::ensure!(
            task.id == task_id,
            "A2A cancellation receipt belongs to another task"
        );
        Ok(task)
    }
}

pub(super) fn validate_binding(task: &Task, task_id: Option<&str>, context_id: Option<&str>) -> Result<()> {
    anyhow::ensure!(!task.id.trim().is_empty(), "A2A task receipt has no identity");
    if let Some(id) = task_id { anyhow::ensure!(task.id == id, "A2A receipt belongs to another task"); }
    if let Some(id) = context_id { anyhow::ensure!(task.context_id.as_deref() == Some(id), "A2A receipt belongs to another context"); }
    Ok(())
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
