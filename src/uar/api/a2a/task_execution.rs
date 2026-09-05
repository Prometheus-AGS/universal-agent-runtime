//! Resumable client-side ownership of one remote task. The trusted host must
//! retain this object until settlement; dropping a waiter borrows, rather than
//! destroys, its in-flight mutation. No background task is detached here.

use std::{pin::Pin, time::Duration};

use tokio_util::sync::CancellationToken;

use super::{
    client::A2AClient,
    contract::UarDelegationContract,
    types::{Message, Task, TaskState},
};

type Request = Pin<Box<dyn Future<Output = anyhow::Result<Task>> + Send>>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum RequestKind {
    Send,
    Poll,
    Cancel,
    Reconcile,
}

/// A failed receipt never proves that the peer stopped executing.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum A2AExecutionError {
    #[error("A2A task cancelled before sending")]
    NotStarted,
    #[error("A2A task creation is unconfirmed; the send must not be replayed")]
    CreationUnconfirmed,
    #[error("A2A task cleanup is unconfirmed")]
    CleanupUnconfirmed,
}

/// Exact endpoint/client/task binding retained by the calling host. This owns
/// protocol progress, not authorization, child persistence or a root budget.
///
/// `execute` and `cancel_and_wait` can be dropped and resumed on the same object.
/// The owner must keep the object and drive cleanup: dropping the object itself
/// cannot perform async cleanup and does not prove that remote work has stopped.
pub struct A2ATaskExecution {
    client: A2AClient,
    endpoint: String,
    cancellation: CancellationToken,
    message: Option<Message>,
    contract: Option<UarDelegationContract>,
    task_id: Option<String>,
    context_id: Option<String>,
    pending: Option<(RequestKind, Request)>,
    task: Option<Task>,
    outcome: Option<Result<Task, A2AExecutionError>>,
    cleanup_attempted: bool,
    cleanup_confirmed: bool,
}

impl std::fmt::Debug for A2ATaskExecution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("A2ATaskExecution")
            .field("task_id", &self.task.as_ref().map(|task| &task.id))
            .field("cleanup_confirmed", &self.cleanup_confirmed())
            .finish_non_exhaustive()
    }
}

impl A2ATaskExecution {
    pub(super) fn new(
        client: A2AClient,
        endpoint: String,
        message: Message,
        contract: Option<UarDelegationContract>,
        task_id: Option<String>,
        context_id: Option<String>,
        parent: &CancellationToken,
    ) -> Self {
        Self {
            client,
            endpoint,
            cancellation: parent.child_token(),
            message: Some(message),
            contract,
            task_id,
            context_id,
            pending: None,
            task: None,
            outcome: None,
            cleanup_attempted: false,
            cleanup_confirmed: false,
        }
    }

    pub(super) fn for_cleanup(
        client: A2AClient,
        endpoint: String,
        contract: UarDelegationContract,
        task_id: String,
        parent: &CancellationToken,
    ) -> Self {
        Self {
            client,
            endpoint,
            cancellation: parent.child_token(),
            message: None,
            contract: Some(contract),
            task_id: Some(task_id),
            context_id: None,
            pending: None,
            task: None,
            outcome: None,
            cleanup_attempted: false,
            cleanup_confirmed: false,
        }
    }

    /// Last exact task receipt. Working/input-required are not completion.
    pub fn snapshot(&self) -> Option<&Task> {
        self.task.as_ref()
    }

    /// Whether the peer confirmed a terminal state, or no send was started.
    /// This is a protocol receipt, not attestation of a remote server's internals.
    pub fn cleanup_confirmed(&self) -> bool {
        self.cleanup_confirmed
    }

    /// Whether execution produced a terminal task receipt. A reusable remote
    /// actor can still be open after this becomes true.
    pub fn terminal_confirmed(&self) -> bool {
        matches!(&self.outcome, Some(Ok(_)))
    }

    fn request(&mut self, kind: RequestKind, task_id: String) {
        if kind == RequestKind::Cancel {
            self.cleanup_attempted = true;
        }
        let client = self.client.clone();
        let endpoint = self.endpoint.clone();
        let contract = self.contract.clone();
        self.pending = Some((
            kind,
            Box::pin(async move {
                match (kind, contract.as_ref()) {
                    (RequestKind::Cancel, Some(contract)) => {
                        client
                            .cancel_governed_task_receipt(&endpoint, &task_id, contract)
                            .await
                    }
                    (RequestKind::Cancel, None) => {
                        client.cancel_task_receipt(&endpoint, &task_id).await
                    }
                    (_, Some(contract)) => {
                        client
                            .get_governed_task(&endpoint, &task_id, contract)
                            .await
                    }
                    (_, None) => client.get_task(&endpoint, &task_id).await,
                }
            }),
        ));
    }

    /// Drive only the mutation that creates or continues the remote task and
    /// return its authenticated identity receipt. The request future remains
    /// owned by this object if the caller stops waiting.
    pub async fn start(&mut self) -> Result<Task, A2AExecutionError> {
        loop {
            if let Some(task) = &self.task {
                return Ok(task.clone());
            }
            if let Some(outcome) = &self.outcome {
                return outcome.clone();
            }
            if self.pending.is_none() {
                let Some(message) = self.message.take() else {
                    self.outcome = Some(Err(A2AExecutionError::CreationUnconfirmed));
                    continue;
                };
                if self.cancellation.is_cancelled() {
                    self.outcome = Some(Err(A2AExecutionError::NotStarted));
                    self.cleanup_confirmed = true;
                    continue;
                }
                let client = self.client.clone();
                let endpoint = self.endpoint.clone();
                let contract = self.contract.clone();
                let task_id = self.task_id.clone();
                let context_id = self.context_id.clone();
                self.pending = Some((
                    RequestKind::Send,
                    Box::pin(async move {
                        match contract.as_ref() {
                            Some(contract) => {
                                client
                                    .send_governed_message(
                                        &endpoint,
                                        &message,
                                        contract,
                                        task_id.as_deref(),
                                        context_id.as_deref(),
                                    )
                                    .await
                            }
                            None => client.send_message(&endpoint, &message).await,
                        }
                    }),
                ));
            }
            let Some((kind, request)) = self.pending.as_mut() else {
                continue;
            };
            debug_assert!(*kind == RequestKind::Send);
            let result = request.await;
            self.pending = None;
            match result {
                Ok(task) => {
                    let cleanup_unconfirmed = task.cleanup_unconfirmed();
                    let terminal = matches!(
                        task.status.state,
                        TaskState::Completed | TaskState::Canceled | TaskState::Failed
                    ) && !cleanup_unconfirmed;
                    if cleanup_unconfirmed {
                        self.cancellation.cancel();
                    }
                    self.task_id = Some(task.id.clone());
                    self.context_id = task.context_id.clone();
                    self.task = Some(task.clone());
                    if terminal && !self.cancellation.is_cancelled() {
                        self.outcome = Some(Ok(task.clone()));
                    }
                    return Ok(task);
                }
                Err(_) => {
                    self.outcome = Some(Err(if self.task_id.is_some() {
                        A2AExecutionError::CleanupUnconfirmed
                    } else {
                        A2AExecutionError::CreationUnconfirmed
                    }));
                }
            }
        }
    }

    /// Drive this task to a terminal receipt, propagating parent cancellation.
    ///
    /// A pending send is allowed to return its task ID before cancellation is
    /// sent. A pending cancel is likewise retained. Only read-only polling may
    /// be interrupted. Every HTTP request uses the client's bounded timeout.
    ///
    /// # Errors
    /// A lost send response retains creation uncertainty without replaying it.
    /// A failed/active cancel receipt gets one read-only reconciliation; absent
    /// terminal evidence remains unconfirmed instead of reporting cancellation.
    pub async fn execute(&mut self) -> Result<Task, A2AExecutionError> {
        if self.task.is_none() && self.message.is_some() {
            self.start().await?;
        }
        loop {
            if let Some(outcome) = &self.outcome {
                return outcome.clone();
            }
            if self.pending.is_none() {
                if let Some(task) = &self.task {
                    let id = task.id.clone();
                    if self.cancellation.is_cancelled() {
                        self.request(RequestKind::Cancel, id);
                    } else {
                        tokio::select! {
                            biased;
                            () = self.cancellation.cancelled() => continue,
                            () = tokio::time::sleep(Duration::from_millis(250)) => {},
                        }
                        self.request(RequestKind::Poll, id);
                    }
                } else {
                    self.outcome = Some(Err(A2AExecutionError::CreationUnconfirmed));
                    continue;
                }
            }

            let Some((kind, request)) = self.pending.as_mut() else {
                continue;
            };
            let kind = *kind;
            let result = if kind == RequestKind::Poll {
                tokio::select! {
                    biased;
                    () = self.cancellation.cancelled() => {
                        self.pending = None;
                        continue;
                    },
                    result = request => result,
                }
            } else {
                request.await
            };
            // Consume a completed future and publish its receipt before any
            // other await. A cancelled waiter must never poll it a second time.
            self.pending = None;
            match result {
                Ok(task) => {
                    // UAR can finish the execution as failed while retaining
                    // unresolved cleanup. That is not a settlement receipt.
                    let cleanup_unconfirmed = task.cleanup_unconfirmed();
                    let terminal = matches!(
                        task.status.state,
                        TaskState::Completed | TaskState::Canceled | TaskState::Failed
                    ) && !cleanup_unconfirmed;
                    if cleanup_unconfirmed {
                        self.cancellation.cancel();
                    }
                    self.task = Some(task.clone());
                    if terminal {
                        let close_marker = task
                            .metadata
                            .get(super::contract::UAR_CLEANUP_CLOSED_METADATA)
                            .and_then(serde_json::Value::as_bool)
                            == Some(true);
                        let direct_close = kind == RequestKind::Cancel
                            && (self.contract.is_none() || close_marker);
                        let reconciled_close = kind == RequestKind::Reconcile && close_marker;
                        if direct_close || reconciled_close {
                            self.cleanup_confirmed = true;
                            self.outcome = Some(Ok(task));
                        } else if matches!(kind, RequestKind::Cancel | RequestKind::Reconcile) {
                            self.outcome = Some(Err(A2AExecutionError::CleanupUnconfirmed));
                        } else if self.cancellation.is_cancelled() {
                            self.request(RequestKind::Cancel, task.id);
                        } else {
                            // A concurrent completion is not relabelled cancelled.
                            self.outcome = Some(Ok(task));
                        }
                    } else if kind == RequestKind::Reconcile {
                        self.outcome = Some(Err(A2AExecutionError::CleanupUnconfirmed));
                    } else if kind == RequestKind::Cancel {
                        self.request(RequestKind::Reconcile, task.id);
                    }
                }
                Err(_) => match kind {
                    RequestKind::Send => {
                        self.outcome = Some(Err(if self.task_id.is_some() {
                            A2AExecutionError::CleanupUnconfirmed
                        } else {
                            A2AExecutionError::CreationUnconfirmed
                        }))
                    }
                    RequestKind::Reconcile => {
                        self.outcome = Some(Err(A2AExecutionError::CleanupUnconfirmed));
                    }
                    RequestKind::Poll => {
                        // Poll failure does not abandon a known live task.
                        self.cancellation.cancel();
                    }
                    RequestKind::Cancel => {
                        if let Some(task_id) = self
                            .task
                            .as_ref()
                            .map(|task| task.id.clone())
                            .or_else(|| self.task_id.clone())
                        {
                            self.request(RequestKind::Reconcile, task_id);
                        } else {
                            self.outcome = Some(Err(A2AExecutionError::CreationUnconfirmed));
                        }
                    }
                },
            }
        }
    }

    /// Request cleanup and await its receipt on this exact binding. A repeated
    /// cleanup after uncertainty only reads status; it never replays a mutation.
    ///
    /// # Errors
    /// Retains unknown creation or cleanup. The owner must not remove this
    /// operation from its registry while `cleanup_confirmed()` is false.
    pub async fn cancel_and_wait(&mut self) -> Result<Task, A2AExecutionError> {
        self.cancellation.cancel();
        if self.cleanup_confirmed {
            return self
                .outcome
                .clone()
                .unwrap_or(Err(A2AExecutionError::NotStarted));
        }
        if matches!(
            self.outcome,
            Some(Ok(_)) | Some(Err(A2AExecutionError::CleanupUnconfirmed))
        ) {
            if let Some(task_id) = self
                .task
                .as_ref()
                .map(|task| task.id.clone())
                .or_else(|| self.task_id.clone())
            {
                self.request(
                    if self.cleanup_attempted {
                        RequestKind::Reconcile
                    } else {
                        RequestKind::Cancel
                    },
                    task_id,
                );
                self.outcome = None;
            }
        } else if self.outcome.is_none()
            && self.task.is_none()
            && self.message.is_none()
            && let Some(task_id) = self.task_id.clone()
        {
            self.request(
                if self.cleanup_attempted {
                    RequestKind::Reconcile
                } else {
                    RequestKind::Cancel
                },
                task_id,
            );
        }
        self.execute().await
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet, HashMap};
    use std::sync::{Arc, Mutex};

    use axum::{
        Json, Router,
        extract::State,
        http::{HeaderMap, header::AUTHORIZATION},
        routing::post,
    };
    use serde_json::{Value, json};

    use super::*;
    use crate::uar::api::a2a::contract::{
        UAR_CLEANUP_CLOSED_METADATA, UAR_DELEGATION_ACK_METADATA, UAR_DELEGATION_CONTRACT_METADATA,
        UAR_DELEGATION_CONTRACT_VERSION, UarDelegationAcknowledgement, UarUsageGrant,
    };
    use crate::uar::context::ContextStrategy;
    use crate::uar::domain::artifact::ToolExecutionMode;
    use crate::uar::domain::policy::{
        ChatMode, ModelRoute, ResourceSelection, RunPolicy, SelectionMode, ToolApprovalPolicy,
    };
    use crate::uar::runtime::thread::policy_intersection::{SandboxPermissions, ThreadBudgets};

    #[derive(Default)]
    struct PeerState {
        methods: Vec<String>,
        cancelled_task_ids: Vec<String>,
        authorization_headers: Vec<String>,
        contracts: HashMap<String, UarDelegationContract>,
    }

    async fn peer(
        State(state): State<Arc<Mutex<PeerState>>>,
        headers: HeaderMap,
        Json(request): Json<Value>,
    ) -> Json<Value> {
        let request_id = request.get("id").cloned().unwrap_or(Value::Null);
        let method = request
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let authorization = headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_owned();

        let result = match method {
            "message/send" => {
                let contract = serde_json::from_value::<UarDelegationContract>(
                    request["params"]["metadata"][UAR_DELEGATION_CONTRACT_METADATA].clone(),
                )
                .expect("governed send carries a valid UAR delegation contract");
                let task_id = format!("task-{}", contract.child_thread_id);
                let task = peer_task(&task_id, TaskState::Working, &contract, false);
                let mut state = state.lock().expect("peer state lock");
                state.methods.push(method.to_owned());
                state.authorization_headers.push(authorization);
                state.contracts.insert(task_id, contract);
                serde_json::to_value(task).expect("working task serializes")
            }
            "tasks/cancel" => {
                let task_id = request["params"]["id"]
                    .as_str()
                    .expect("cancel carries a task id")
                    .to_owned();
                let mut state = state.lock().expect("peer state lock");
                state.methods.push(method.to_owned());
                state.cancelled_task_ids.push(task_id.clone());
                state.authorization_headers.push(authorization);
                let contract = state
                    .contracts
                    .get(&task_id)
                    .expect("cancel targets a task created by this peer")
                    .clone();
                serde_json::to_value(peer_task(&task_id, TaskState::Canceled, &contract, true))
                    .expect("canceled task serializes")
            }
            _ => panic!("unexpected A2A method: {method}"),
        };

        Json(json!({"jsonrpc": "2.0", "id": request_id, "result": result}))
    }

    fn peer_task(
        task_id: &str,
        state: TaskState,
        contract: &UarDelegationContract,
        cleanup_closed: bool,
    ) -> Task {
        let mut metadata = HashMap::from([(
            UAR_DELEGATION_ACK_METADATA.to_owned(),
            serde_json::to_value(
                UarDelegationAcknowledgement::for_contract(contract, Some(task_id.to_owned()))
                    .expect("delegation acknowledgement is valid"),
            )
            .expect("delegation acknowledgement serializes"),
        )]);
        if cleanup_closed {
            metadata.insert(UAR_CLEANUP_CLOSED_METADATA.to_owned(), json!(true));
        }
        Task {
            id: task_id.to_owned(),
            context_id: Some(format!("context-{task_id}")),
            status: super::super::types::TaskStatus {
                state,
                message: None,
                timestamp: None,
            },
            history: Vec::new(),
            artifacts: Vec::new(),
            metadata,
        }
    }

    fn contract(child_thread_id: &str) -> UarDelegationContract {
        let none = ResourceSelection {
            mode: SelectionMode::None,
            ..ResourceSelection::default()
        };
        UarDelegationContract {
            version: UAR_DELEGATION_CONTRACT_VERSION,
            source_instance_id: "source-uar".to_owned(),
            target_instance_id: "target-uar".to_owned(),
            owner_id: "owner".to_owned(),
            root_run_id: "root-run".to_owned(),
            parent_thread_id: "root-thread".to_owned(),
            child_thread_id: child_thread_id.to_owned(),
            target_agent_id: "peer-agent".to_owned(),
            policy: RunPolicy {
                chat_mode: Some(ChatMode::Agent),
                agent_id: Some("peer-agent".to_owned()),
                model: Some(ModelRoute {
                    provider_id: "peer-provider".to_owned(),
                    model_id: "peer-model".to_owned(),
                }),
                skills: none.clone(),
                tools: none.clone(),
                mcp_servers: none.clone(),
                presentations: none.clone(),
                knowledge_bases: none,
                memory_enabled: Some(false),
                prompt_caching_enabled: Some(false),
                context_strategy: Some(ContextStrategy::Auto),
                tool_approval: ToolApprovalPolicy::Auto,
                ..RunPolicy::default()
            }
            .into(),
            budgets: ThreadBudgets::default(),
            usage_grant: UarUsageGrant::default(),
            sandbox: SandboxPermissions {
                execution_mode: ToolExecutionMode::Direct,
                network_enabled: true,
                filesystem: BTreeMap::new(),
                environment: BTreeSet::new(),
            },
            presentation_negotiation: None,
        }
    }

    #[tokio::test]
    async fn root_cancellation_cancels_every_child_and_sends_peer_tasks_cancel() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("peer listener binds");
        let endpoint = format!("http://{}/", listener.local_addr().expect("peer address"));
        let state = Arc::new(Mutex::new(PeerState::default()));
        let peer_state = Arc::clone(&state);
        let server = tokio::spawn(async move {
            axum::serve(
                listener,
                Router::new().route("/", post(peer)).with_state(peer_state),
            )
            .await
            .expect("peer server runs");
        });

        let root = CancellationToken::new();
        let local_child_a = root.child_token();
        let local_child_b = root.child_token();
        let client = A2AClient::new().with_bearer_token("peer-token");
        let mut remote_child_a = client
            .governed_task_execution(
                endpoint.clone(),
                Message::user_text("delegate a"),
                contract("child-a"),
                &root,
            )
            .expect("first governed child binds");
        let mut remote_child_b = client
            .governed_task_execution(
                endpoint,
                Message::user_text("delegate b"),
                contract("child-b"),
                &root,
            )
            .expect("second governed child binds");

        assert_eq!(
            remote_child_a
                .start()
                .await
                .expect("first peer task starts")
                .status
                .state,
            TaskState::Working
        );
        assert_eq!(
            remote_child_b
                .start()
                .await
                .expect("second peer task starts")
                .status
                .state,
            TaskState::Working
        );

        root.cancel();
        assert!(local_child_a.is_cancelled());
        assert!(local_child_b.is_cancelled());
        let (receipt_a, receipt_b) =
            tokio::join!(remote_child_a.execute(), remote_child_b.execute());
        assert_eq!(
            receipt_a
                .expect("first peer confirms cancellation")
                .status
                .state,
            TaskState::Canceled
        );
        assert_eq!(
            receipt_b
                .expect("second peer confirms cancellation")
                .status
                .state,
            TaskState::Canceled
        );
        assert!(remote_child_a.cleanup_confirmed());
        assert!(remote_child_b.cleanup_confirmed());

        let state = state.lock().expect("peer state lock");
        assert_eq!(
            state
                .methods
                .iter()
                .filter(|method| method.as_str() == "message/send")
                .count(),
            2
        );
        assert_eq!(
            state
                .methods
                .iter()
                .filter(|method| method.as_str() == "tasks/cancel")
                .count(),
            2
        );
        let mut cancelled_task_ids = state.cancelled_task_ids.clone();
        cancelled_task_ids.sort();
        assert_eq!(
            cancelled_task_ids,
            vec!["task-child-a".to_owned(), "task-child-b".to_owned()]
        );
        assert!(
            state
                .authorization_headers
                .iter()
                .all(|header| header == "Bearer peer-token")
        );
        drop(state);

        server.abort();
        let _ = server.await;
    }
}
