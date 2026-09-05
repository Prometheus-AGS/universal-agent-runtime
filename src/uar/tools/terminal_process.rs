//! Trusted-host ownership of directly launched terminal processes. This is a
//! lifetime boundary, not process-tree isolation or a delegated shell grant.

use std::collections::{BTreeMap, VecDeque};
use std::io;
use std::process::{ExitStatus, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::FutureExt;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::Instrument;

use crate::uar::runtime::context::truncate::DEFAULT_OUTPUT_BYTE_BUDGET;

#[derive(Debug, Clone, thiserror::Error)]
pub(crate) enum TerminalProcessError {
    #[error("Terminal run has closed or is unavailable")]
    Unavailable,
    #[error("Terminal command cancelled")]
    Cancelled,
    #[error("Terminal command deadline exceeded")]
    DeadlineExceeded,
    #[error("Terminal process I/O failed")]
    Io(#[source] Arc<io::Error>),
    #[error("Terminal process worker failed")]
    WorkerFailed,
    #[error("Terminal process cleanup is unconfirmed (operation {0})")]
    CleanupUnconfirmed(String),
}

#[derive(Clone)]
pub(crate) struct TerminalOutput {
    pub(crate) status: ExitStatus,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) stdout_bytes: u64,
    pub(crate) stderr_bytes: u64,
}

#[derive(Clone)]
struct Outcome {
    result: Result<TerminalOutput, TerminalProcessError>,
    reaped: bool,
}

struct Joined {
    handle: Option<JoinHandle<Outcome>>,
    outcome: Option<Outcome>,
}

struct Operation {
    id: String,
    // Retain the exact child even if the worker fails. A PID alone can be reused
    // and must not be treated as authority for a later cleanup attempt.
    _child: Arc<Mutex<Option<Child>>>,
    joined: Mutex<Joined>,
}

impl Operation {
    async fn join(&self) -> Outcome {
        let mut joined = self.joined.lock().await;
        if let Some(handle) = joined.handle.as_mut() {
            let result = handle.await;
            // Record consumption before another await so a cancelled waiter
            // cannot leave a completed JoinHandle to be polled again.
            joined.handle = None;
            joined.outcome = Some(result.unwrap_or_else(|_| Outcome {
                result: Err(TerminalProcessError::CleanupUnconfirmed(self.id.clone())),
                reaped: false,
            }));
        }
        joined.outcome.clone().unwrap_or_else(|| Outcome {
            result: Err(TerminalProcessError::CleanupUnconfirmed(self.id.clone())),
            reaped: false,
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
    jobs: Mutex<Jobs>,
}

/// A request-only process scope minted and retained by the trusted run host.
/// It carries lifetime ownership, not permission to execute a command.
#[derive(Clone)]
pub struct TerminalRun(Arc<RunState>);

impl std::fmt::Debug for TerminalRun {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TerminalRun")
            .field("run_id", &self.0.id)
            .finish_non_exhaustive()
    }
}

pub(crate) struct TerminalRunLease(TerminalRun);

impl TerminalRunLease {
    pub(crate) fn scope(&self) -> TerminalRun {
        self.0.clone()
    }
}

impl Drop for TerminalRunLease {
    fn drop(&mut self) {
        self.0.0.cancellation.cancel();
    }
}

#[derive(Default)]
struct Runs {
    closed: bool,
    entries: BTreeMap<String, TerminalRun>,
}

#[derive(Default)]
pub(crate) struct TerminalSupervisor(Mutex<Runs>);

impl TerminalSupervisor {
    pub(crate) async fn open_run(
        &self,
        id: String,
        cancellation: &CancellationToken,
        deadline: Option<Instant>,
    ) -> Result<TerminalRunLease, TerminalProcessError> {
        let mut runs = self.0.lock().await;
        if runs.closed || cancellation.is_cancelled() || runs.entries.contains_key(&id) {
            return Err(TerminalProcessError::Unavailable);
        }
        if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            return Err(TerminalProcessError::DeadlineExceeded);
        }
        let scope = TerminalRun(Arc::new(RunState {
            id: id.clone(),
            cancellation: cancellation.child_token(),
            deadline,
            jobs: Mutex::new(Jobs::default()),
        }));
        runs.entries.insert(id, scope.clone());
        Ok(TerminalRunLease(scope))
    }

    pub(crate) async fn finish_run(&self, scope: &TerminalRun) -> Result<(), TerminalProcessError> {
        scope.drain().await?;
        let mut runs = self.0.lock().await;
        if runs
            .entries
            .get(&scope.0.id)
            .is_some_and(|current| Arc::ptr_eq(&current.0, &scope.0))
        {
            runs.entries.remove(&scope.0.id);
        }
        Ok(())
    }

    pub(crate) async fn shutdown(&self) -> Result<(), TerminalProcessError> {
        let snapshot = {
            let mut runs = self.0.lock().await;
            runs.closed = true;
            runs.entries.values().cloned().collect::<Vec<_>>()
        };
        for scope in &snapshot {
            scope.0.cancellation.cancel();
        }
        let mut failure = None;
        for scope in snapshot {
            if let Err(error) = self.finish_run(&scope).await {
                failure.get_or_insert(error);
            }
        }
        failure.map_or(Ok(()), Err)
    }
}

struct CancelOperation(CancellationToken);

impl Drop for CancelOperation {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

impl TerminalRun {
    pub(crate) async fn execute(
        &self,
        mut command: Command,
        timeout: Duration,
    ) -> Result<TerminalOutput, TerminalProcessError> {
        let local_deadline = Instant::now()
            .checked_add(timeout)
            .ok_or(TerminalProcessError::Unavailable)?;
        let deadline = self
            .0
            .deadline
            .map_or(local_deadline, |root| root.min(local_deadline));
        let cancellation = self.0.cancellation.child_token();
        let _cancel_on_drop = CancelOperation(cancellation.clone());
        let operation = {
            let mut jobs = self.0.jobs.lock().await;
            if jobs.closed || cancellation.is_cancelled() {
                return Err(TerminalProcessError::Unavailable);
            }
            if Instant::now() >= deadline {
                return Err(TerminalProcessError::DeadlineExceeded);
            }
            command
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true);
            let child = command
                .spawn()
                .map_err(|error| TerminalProcessError::Io(Arc::new(error)))?;
            let child = Arc::new(Mutex::new(Some(child)));
            let id = uuid::Uuid::new_v4().to_string();
            // No await between launching the worker and publishing its handle
            // into the run registry. Closing admission uses this same lock.
            let handle = tokio::spawn(
                execute_owned(id.clone(), Arc::clone(&child), cancellation, deadline)
                    .instrument(tracing::Span::current()),
            );
            let operation = Arc::new(Operation {
                id,
                _child: child,
                joined: Mutex::new(Joined {
                    handle: Some(handle),
                    outcome: None,
                }),
            });
            jobs.operations.push(Arc::clone(&operation));
            operation
        };
        let outcome = operation.join().await;
        if outcome.reaped {
            self.0
                .jobs
                .lock()
                .await
                .operations
                .retain(|current| !Arc::ptr_eq(current, &operation));
        } else {
            self.0.cancellation.cancel();
        }
        outcome.result
    }

    async fn drain(&self) -> Result<(), TerminalProcessError> {
        self.0.cancellation.cancel();
        let operations = {
            let mut jobs = self.0.jobs.lock().await;
            jobs.closed = true;
            jobs.operations.clone()
        };
        let mut failure = None;
        for operation in operations {
            let outcome = operation.join().await;
            if outcome.reaped {
                self.0
                    .jobs
                    .lock()
                    .await
                    .operations
                    .retain(|current| !Arc::ptr_eq(current, &operation));
            } else {
                failure.get_or_insert(TerminalProcessError::CleanupUnconfirmed(
                    operation.id.clone(),
                ));
            }
        }
        failure.map_or(Ok(()), Err)
    }
}

async fn execute_owned(
    id: String,
    slot: Arc<Mutex<Option<Child>>>,
    cancellation: CancellationToken,
    deadline: Instant,
) -> Outcome {
    let mut slot = slot.lock().await;
    let Some(child) = slot.as_mut() else {
        return Outcome {
            result: Err(TerminalProcessError::CleanupUnconfirmed(id)),
            reaped: false,
        };
    };
    let execution = async {
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("Missing terminal stdout"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| io::Error::other("Missing terminal stderr"))?;
        let ((stdout, stdout_bytes), (stderr, stderr_bytes), status) =
            tokio::try_join!(capture_output(stdout), capture_output(stderr), child.wait(),)?;
        Ok::<_, io::Error>(TerminalOutput {
            status,
            stdout,
            stderr,
            stdout_bytes,
            stderr_bytes,
        })
    };
    let result = tokio::select! {
        biased;
        () = cancellation.cancelled() => Err(TerminalProcessError::Cancelled),
        () = tokio::time::sleep_until(deadline.into()) => Err(TerminalProcessError::DeadlineExceeded),
        result = std::panic::AssertUnwindSafe(execution).catch_unwind() => match result {
            Ok(result) => result.map_err(|error| TerminalProcessError::Io(Arc::new(error))),
            Err(_) => Err(TerminalProcessError::WorkerFailed),
        },
    };
    // The losing I/O future is gone and pipes are closed before killing. Await
    // this exact child, not a detached PID; a failed reap remains host-owned.
    if result.is_err() && child.kill().await.is_err() {
        return Outcome {
            result: Err(TerminalProcessError::CleanupUnconfirmed(id)),
            reaped: false,
        };
    }
    slot.take();
    Outcome {
        result,
        reaped: true,
    }
}

async fn capture_output(mut reader: impl AsyncRead + Unpin) -> io::Result<(String, u64)> {
    // Retain head and tail while draining the pipe, so a verbose command cannot
    // allocate unbounded memory or deadlock waiting for its reader to resume.
    let head_limit = DEFAULT_OUTPUT_BYTE_BUDGET / 2;
    let tail_limit = DEFAULT_OUTPUT_BYTE_BUDGET - head_limit;
    let mut head = Vec::with_capacity(head_limit);
    let mut tail = VecDeque::with_capacity(tail_limit);
    let mut total = 0_u64;
    let mut chunk = [0_u8; 8192];
    loop {
        let count = reader.read(&mut chunk).await?;
        if count == 0 {
            break;
        }
        total = total.saturating_add(count as u64);
        let prefix = count.min(head_limit - head.len());
        head.extend_from_slice(&chunk[..prefix]);
        for byte in &chunk[prefix..count] {
            if tail.len() == tail_limit {
                tail.pop_front();
            }
            tail.push_back(*byte);
        }
    }
    let omitted = total.saturating_sub((head.len() + tail.len()) as u64);
    if omitted == 0 {
        head.extend(tail);
        return Ok((String::from_utf8_lossy(&head).into_owned(), total));
    }
    let tail = tail.into_iter().collect::<Vec<_>>();
    Ok((
        format!(
            "{}\n[... {omitted} bytes omitted ...]\n{}",
            String::from_utf8_lossy(&head),
            String::from_utf8_lossy(&tail)
        ),
        total,
    ))
}
