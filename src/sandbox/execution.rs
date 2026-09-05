//! Trusted-host ownership of ephemeral sandbox operations. Dropping a model
//! stream cancels its operation, not the task that owns creation and cleanup.
//! Unknown create/destroy outcomes are retained; mutations are never replayed.

use std::collections::BTreeMap;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::FutureExt;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::Instrument;

use super::bindings::SandboxBinding;
use super::{
    ExecutionRequest, ExecutionResult, SandboxConfig, SandboxError, SandboxHandle, SandboxRunner,
};

/// Content-free lifecycle state for an operation owned by the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxOperationPhase {
    Pending,
    Creating,
    Executing,
    Destroying,
    Released,
    CreationUnconfirmed,
    CleanupUnconfirmed,
}

/// Host diagnostics contain identities and state, never code, env or output.
#[derive(Debug, Clone)]
pub struct SandboxOperationSnapshot {
    pub run_id: String,
    pub operation_id: String,
    pub sandbox_id: Option<String>,
    pub phase: SandboxOperationPhase,
    pub runner_type: super::runner::RunnerType,
}

/// Model-safe failure. Backend bodies remain in the error chain, not Display.
#[derive(Debug, Clone, thiserror::Error)]
pub enum SandboxExecutionError {
    #[error("Sandbox execution is unavailable or its run has closed")]
    Unavailable,
    #[error("Sandbox execution cancelled")]
    Cancelled,
    #[error("Sandbox execution deadline exceeded")]
    DeadlineExceeded,
    #[error("Sandbox backend execution failed")]
    Backend(#[source] Arc<SandboxError>),
    #[error("Sandbox {stage} outcome is unconfirmed (operation {operation_id})")]
    Unconfirmed {
        operation_id: String,
        stage: &'static str,
        #[source]
        source: Option<Arc<SandboxError>>,
    },
}

#[derive(Clone)]
struct Outcome {
    result: Result<ExecutionResult, SandboxExecutionError>,
    cleanup_confirmed: bool,
}

struct Receipt {
    phase: SandboxOperationPhase,
    handle: Option<SandboxHandle>,
    runner_type: super::runner::RunnerType,
}

struct JoinedOperation {
    handle: Option<JoinHandle<Outcome>>,
    outcome: Option<Outcome>,
    worker_failed: bool,
}

struct Operation {
    id: String,
    // Retain the exact backend and handle after an uncertain cleanup. A new
    // environment/client lookup must not be used to reconcile this resource.
    _runner: Arc<dyn SandboxRunner>,
    receipt: Arc<Mutex<Receipt>>,
    joined: Mutex<JoinedOperation>,
}

impl Operation {
    async fn join(&self) -> Outcome {
        let mut joined = self.joined.lock().await;
        if let Some(handle) = joined.handle.as_mut() {
            // Borrow the handle. Cancelling a waiter must leave it joinable.
            let result = handle.await;
            // Publish consumption before another await. In particular, receipt
            // inspection after a failed worker can block behind diagnostics;
            // cancelling that waiter must never leave a consumed handle to poll.
            joined.handle = None;
            joined.outcome = Some(match result {
                Ok(outcome) => outcome,
                Err(_) => {
                    joined.worker_failed = true;
                    Outcome {
                        result: Err(unconfirmed(&self.id, "worker", None)),
                        cleanup_confirmed: false,
                    }
                }
            });
        }
        if joined.worker_failed {
            let mut receipt = self.receipt.lock().await;
            let cleanup_confirmed = matches!(
                receipt.phase,
                SandboxOperationPhase::Pending | SandboxOperationPhase::Released
            );
            receipt.phase = match receipt.phase {
                SandboxOperationPhase::Pending | SandboxOperationPhase::Released => {
                    SandboxOperationPhase::Released
                }
                SandboxOperationPhase::Creating | SandboxOperationPhase::CreationUnconfirmed => {
                    SandboxOperationPhase::CreationUnconfirmed
                }
                _ => SandboxOperationPhase::CleanupUnconfirmed,
            };
            joined.outcome = Some(Outcome {
                result: Err(unconfirmed(&self.id, "worker", None)),
                cleanup_confirmed,
            });
            joined.worker_failed = false;
        }
        joined.outcome.clone().unwrap_or_else(|| Outcome {
            result: Err(unconfirmed(&self.id, "completion", None)),
            cleanup_confirmed: false,
        })
    }
}

#[derive(Default)]
struct Jobs {
    closed: bool,
    operations: Vec<Arc<Operation>>,
}

struct RunState {
    id: String,
    cancellation: CancellationToken,
    deadline: Option<Instant>,
    binding: Option<Arc<SandboxBinding>>,
    jobs: Mutex<Jobs>,
}

/// A run's request handle. Only the supervisor can mint this scope.
#[derive(Clone)]
pub struct SandboxRun {
    inner: Arc<RunState>,
}

impl std::fmt::Debug for SandboxRun {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SandboxRun")
            .field("run_id", &self.inner.id)
            .finish_non_exhaustive()
    }
}

/// Owned by the executing run future, not its tools. Unwind closes admission;
/// the supervisor retains outstanding jobs until an async drain joins them.
#[derive(Debug)]
pub struct SandboxRunLease {
    scope: SandboxRun,
}

impl SandboxRunLease {
    /// Borrow the run's authority without transferring lifetime ownership.
    pub fn scope(&self) -> SandboxRun {
        self.scope.clone()
    }
}

impl Drop for SandboxRunLease {
    fn drop(&mut self) {
        self.scope.inner.cancellation.cancel();
    }
}

#[derive(Default)]
struct Runs {
    closed: bool,
    entries: BTreeMap<String, SandboxRun>,
}

/// Manager-owned registry. Failed receipts remain available to later shutdown
/// calls and diagnostics; a second drain cannot turn uncertainty into success.
#[derive(Default)]
pub struct SandboxSupervisor {
    runs: Mutex<Runs>,
}

impl std::fmt::Debug for SandboxSupervisor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SandboxSupervisor").finish_non_exhaustive()
    }
}

impl SandboxSupervisor {
    /// Register a unique run before exposing its sandbox execution scope.
    ///
    /// # Errors
    /// Rejects duplicate identities, cancellation or permanently closed admission.
    pub async fn open_run(
        &self,
        run_id: String,
        cancellation: &CancellationToken,
        deadline: Option<Instant>,
        binding: Option<Arc<SandboxBinding>>,
    ) -> Result<SandboxRunLease, SandboxExecutionError> {
        let mut runs = self.runs.lock().await;
        if runs.closed || cancellation.is_cancelled() || runs.entries.contains_key(&run_id) {
            return Err(SandboxExecutionError::Unavailable);
        }
        if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            return Err(SandboxExecutionError::DeadlineExceeded);
        }
        let scope = SandboxRun {
            inner: Arc::new(RunState {
                id: run_id.clone(),
                cancellation: cancellation.child_token(),
                jobs: Mutex::new(Jobs::default()),
                deadline,
                binding,
            }),
        };
        runs.entries.insert(run_id, scope.clone());
        Ok(SandboxRunLease { scope })
    }

    /// Cancel and join every operation before removing its exact run scope.
    ///
    /// # Errors
    /// Retains and reports unconfirmed creation, cleanup or worker completion.
    pub async fn finish_run(&self, run: &SandboxRun) -> Result<(), SandboxExecutionError> {
        run.drain().await?;
        let mut runs = self.runs.lock().await;
        if runs
            .entries
            .get(&run.inner.id)
            .is_some_and(|current| Arc::ptr_eq(&current.inner, &run.inner))
        {
            runs.entries.remove(&run.inner.id);
        }
        Ok(())
    }

    /// Permanently close admission, cancel all scopes, and join all known jobs.
    ///
    /// # Errors
    /// Returns the first unresolved failure after attempting every run's drain.
    pub async fn shutdown(&self) -> Result<(), SandboxExecutionError> {
        let snapshot = {
            let mut runs = self.runs.lock().await;
            runs.closed = true;
            runs.entries.values().cloned().collect::<Vec<_>>()
        };
        for run in &snapshot {
            run.inner.cancellation.cancel();
        }
        let mut failure = None;
        for run in snapshot {
            if let Err(error) = self.finish_run(&run).await {
                failure.get_or_insert(error);
            }
        }
        failure.map_or(Ok(()), Err)
    }

    /// Inspect retained operations without exposing prompts, credentials or output.
    pub async fn operations(&self) -> Vec<SandboxOperationSnapshot> {
        let runs = self
            .runs
            .lock()
            .await
            .entries
            .values()
            .cloned()
            .collect::<Vec<_>>();
        let mut snapshots = Vec::new();
        for run in runs {
            let operations = run.inner.jobs.lock().await.operations.clone();
            for operation in operations {
                let receipt = operation.receipt.lock().await;
                snapshots.push(SandboxOperationSnapshot {
                    run_id: run.inner.id.clone(),
                    operation_id: operation.id.clone(),
                    sandbox_id: receipt.handle.as_ref().map(|handle| handle.id.clone()),
                    phase: receipt.phase,
                    runner_type: receipt.runner_type,
                });
            }
        }
        snapshots
    }
}

struct CancelOperation(CancellationToken);

impl Drop for CancelOperation {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

impl SandboxRun {
    /// Execute one approved ephemeral operation under the retained host task.
    /// Dropping this future requests cancellation; it never detaches ownership.
    ///
    /// # Errors
    /// Rejects non-isolating backends, persistent modes and closed scopes, and
    /// reports execution/cleanup failures without a direct-execution fallback.
    pub async fn execute(
        &self,
        runner: Arc<dyn SandboxRunner>,
        request: ExecutionRequest,
    ) -> Result<ExecutionResult, SandboxExecutionError> {
        let binding = self
            .inner
            .binding
            .as_ref()
            .ok_or(SandboxExecutionError::Unavailable)?;
        let config = binding
            .execution_config(&runner, &request)
            .map_err(|_| SandboxExecutionError::Unavailable)?;
        let runner_type = runner.capabilities().runner_type;
        let cancellation = self.inner.cancellation.child_token();
        let _cancel_on_drop = CancelOperation(cancellation.clone());
        let operation = {
            let mut jobs = self.inner.jobs.lock().await;
            if jobs.closed || cancellation.is_cancelled() {
                return Err(SandboxExecutionError::Unavailable);
            }
            if self
                .inner
                .deadline
                .is_some_and(|deadline| Instant::now() >= deadline)
            {
                return Err(SandboxExecutionError::DeadlineExceeded);
            }
            let id = uuid::Uuid::new_v4().to_string();
            let receipt = Arc::new(Mutex::new(Receipt {
                phase: SandboxOperationPhase::Pending,
                handle: None,
                runner_type,
            }));
            let task = tokio::spawn(
                execute_owned(
                    id.clone(),
                    Arc::clone(&runner),
                    config,
                    request,
                    cancellation,
                    Arc::clone(&receipt),
                    self.inner.deadline,
                )
                .instrument(tracing::Span::current()),
            );
            let operation = Arc::new(Operation {
                id,
                _runner: runner,
                receipt,
                joined: Mutex::new(JoinedOperation {
                    handle: Some(task),
                    outcome: None,
                    worker_failed: false,
                }),
            });
            jobs.operations.push(Arc::clone(&operation));
            operation
        };
        let outcome = operation.join().await;
        if outcome.cleanup_confirmed {
            self.inner
                .jobs
                .lock()
                .await
                .operations
                .retain(|current| !Arc::ptr_eq(current, &operation));
        } else {
            // Do not authorize another remote mutation while an earlier one
            // has an unresolved effect. Model-visible failure is not rollback.
            self.inner.cancellation.cancel();
        }
        outcome.result
    }

    async fn drain(&self) -> Result<(), SandboxExecutionError> {
        self.inner.cancellation.cancel();
        let operations = {
            let mut jobs = self.inner.jobs.lock().await;
            jobs.closed = true;
            jobs.operations.clone()
        };
        let mut failure = None;
        for operation in operations {
            let outcome = operation.join().await;
            if outcome.cleanup_confirmed {
                self.inner
                    .jobs
                    .lock()
                    .await
                    .operations
                    .retain(|current| !Arc::ptr_eq(current, &operation));
            } else {
                failure.get_or_insert_with(|| {
                    outcome
                        .result
                        .err()
                        .unwrap_or_else(|| unconfirmed(&operation.id, "cleanup", None))
                });
            }
        }
        failure.map_or(Ok(()), Err)
    }
}

fn unconfirmed(
    id: &str,
    stage: &'static str,
    source: Option<SandboxError>,
) -> SandboxExecutionError {
    SandboxExecutionError::Unconfirmed {
        operation_id: id.to_owned(),
        stage,
        source: source.map(Arc::new),
    }
}

async fn execute_owned(
    id: String,
    runner: Arc<dyn SandboxRunner>,
    config: SandboxConfig,
    request: ExecutionRequest,
    cancellation: CancellationToken,
    receipt: Arc<Mutex<Receipt>>,
    root_deadline: Option<Instant>,
) -> Outcome {
    if cancellation.is_cancelled() {
        receipt.lock().await.phase = SandboxOperationPhase::Released;
        return Outcome {
            result: Err(SandboxExecutionError::Cancelled),
            cleanup_confirmed: true,
        };
    }
    let timeout_secs = request
        .timeout_seconds
        .unwrap_or(config.timeout_secs)
        .min(config.timeout_secs);
    let Some(local_deadline) = Instant::now().checked_add(Duration::from_secs(timeout_secs)) else {
        receipt.lock().await.phase = SandboxOperationPhase::Released;
        return Outcome {
            result: Err(SandboxExecutionError::Unavailable),
            cleanup_confirmed: true,
        };
    };
    let deadline = root_deadline.map_or(local_deadline, |root| root.min(local_deadline));
    if Instant::now() >= deadline {
        receipt.lock().await.phase = SandboxOperationPhase::Released;
        return Outcome {
            result: Err(SandboxExecutionError::DeadlineExceeded),
            cleanup_confirmed: true,
        };
    }
    receipt.lock().await.phase = SandboxOperationPhase::Creating;
    // Creation cannot be cancelled safely without a server-assigned handle or
    // an idempotent creation protocol. Await its one response even on cancel.
    let handle = match AssertUnwindSafe(async { runner.create(config).await })
        .catch_unwind()
        .await
    {
        Ok(Ok(handle)) => handle,
        response => {
            receipt.lock().await.phase = SandboxOperationPhase::CreationUnconfirmed;
            let source = match response {
                Ok(Err(error)) => Some(error),
                _ => None,
            };
            crate::uar::telemetry::metrics::record_sandbox_error("creation_unconfirmed");
            return Outcome {
                result: Err(unconfirmed(&id, "creation", source)),
                cleanup_confirmed: false,
            };
        }
    };
    {
        let mut receipt = receipt.lock().await;
        receipt.handle = Some(handle.clone());
        receipt.phase = SandboxOperationPhase::Executing;
    }
    let language = sandbox_language_label(&request.language);
    let runner_type = receipt.lock().await.runner_type;
    crate::uar::telemetry::metrics::record_sandbox_created(
        sandbox_runner_type_label(runner_type),
        language,
    );
    crate::uar::telemetry::metrics::sandbox_active_inc();
    let result = tokio::select! {
        biased;
        () = cancellation.cancelled() => Err(SandboxExecutionError::Cancelled),
        () = tokio::time::sleep_until(deadline.into()) => Err(SandboxExecutionError::DeadlineExceeded),
        result = AssertUnwindSafe(async { runner.execute(&handle, request).await }).catch_unwind() => match result {
            Ok(result) => result.map_err(|error| SandboxExecutionError::Backend(Arc::new(error))),
            Err(_) => Err(unconfirmed(&id, "execution", None)),
        },
    };
    match &result {
        Ok(result) => crate::uar::telemetry::metrics::record_sandbox_execution(
            language,
            if result.exit_code == 0 {
                "success"
            } else {
                "error"
            },
            sandbox_duration_secs(result.execution_time_ms),
        ),
        Err(SandboxExecutionError::Backend(error)) => {
            crate::uar::telemetry::metrics::record_sandbox_error(sandbox_error_type(error))
        }
        Err(SandboxExecutionError::Cancelled) => {
            crate::uar::telemetry::metrics::record_sandbox_error("cancelled")
        }
        Err(SandboxExecutionError::DeadlineExceeded) => {
            crate::uar::telemetry::metrics::record_sandbox_error("timeout")
        }
        Err(_) => crate::uar::telemetry::metrics::record_sandbox_error("execution_unconfirmed"),
    }
    receipt.lock().await.phase = SandboxOperationPhase::Destroying;
    // Never cancel or blindly retry destruction. A failed response leaves the
    // exact handle/backend retained for host reconciliation, not fake success.
    match AssertUnwindSafe(async { runner.destroy(handle).await })
        .catch_unwind()
        .await
    {
        Ok(Ok(())) => {
            let mut receipt = receipt.lock().await;
            receipt.phase = SandboxOperationPhase::Released;
            receipt.handle = None;
            crate::uar::telemetry::metrics::sandbox_active_dec();
            Outcome {
                result,
                cleanup_confirmed: true,
            }
        }
        response => {
            receipt.lock().await.phase = SandboxOperationPhase::CleanupUnconfirmed;
            let source = match response {
                Ok(Err(error)) => Some(error),
                _ => None,
            };
            crate::uar::telemetry::metrics::record_sandbox_error("cleanup_unconfirmed");
            Outcome {
                result: Err(unconfirmed(&id, "cleanup", source)),
                cleanup_confirmed: false,
            }
        }
    }
}

fn sandbox_language_label(language: &super::Language) -> &'static str {
    match language {
        super::Language::Bash => "bash",
        super::Language::Python => "python",
        super::Language::Node => "node",
        super::Language::Rust => "rust",
    }
}

fn sandbox_runner_type_label(runner: super::runner::RunnerType) -> &'static str {
    match runner {
        super::runner::RunnerType::MicroVm => "microsandbox",
        super::runner::RunnerType::Wasmtime => "wasmtime",
        super::runner::RunnerType::Remote => "remote",
    }
}

fn sandbox_error_type(error: &SandboxError) -> &'static str {
    match error {
        SandboxError::CreationFailed(_) => "creation_failed",
        SandboxError::ExecutionFailed(_) => "execution_failed",
        SandboxError::FileError(_) => "file_error",
        SandboxError::NotFound(_) => "not_found",
        SandboxError::CapacityExceeded(_) => "capacity_exceeded",
        SandboxError::Timeout(_) => "timeout",
        SandboxError::RunnerUnavailable(_) => "runner_unavailable",
    }
}

#[expect(clippy::cast_precision_loss, reason = "duration ms fits within f64")]
fn sandbox_duration_secs(ms: u64) -> f64 {
    ms as f64 / 1000.0
}
