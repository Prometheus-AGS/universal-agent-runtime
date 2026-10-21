//! Host ownership of stdio children, including cancelled initialization attempts.

use std::io;
use futures::FutureExt as _;
use std::process::Stdio;
use std::sync::{
    Arc, RwLock,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

use rmcp::{
    RoleClient,
    service::{RxJsonRpcMessage, TxJsonRpcMessage},
    transport::{Transport, async_rw::AsyncRwTransport},
};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::watch;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

/// Child termination is observed independently from the signal requesting it.
#[derive(Clone, Copy, Debug)]
struct ChildExit {
    forced: bool,
    exit_code: Option<i32>,
}

/// Local resource release report; forced termination is never labelled graceful.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct StdioShutdownReport {
    pub(crate) forced_processes: usize,
}

#[derive(Default)]
struct SupervisorState {
    closed: RwLock<bool>,
    cancellation: CancellationToken,
    tasks: TaskTracker,
    cleanup_failed: Arc<AtomicBool>,
    forced_processes: Arc<AtomicUsize>,
}

impl Drop for SupervisorState {
    fn drop(&mut self) {
        self.cancellation.cancel();
        self.tasks.close();
    }
}

#[derive(Clone, Default)]
pub(crate) struct StdioProcessSupervisor(Arc<SupervisorState>);

impl std::fmt::Debug for StdioProcessSupervisor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StdioProcessSupervisor")
            .field("process_tasks", &self.0.tasks.len())
            .finish_non_exhaustive()
    }
}

impl StdioProcessSupervisor {
    pub(crate) fn spawn(&self, mut command: Command) -> io::Result<SupervisedStdioTransport> {
        // Close admission and register process ownership under the same lock;
        // TaskTracker::close by itself does not prohibit subsequent spawning.
        let closed = self
            .0
            .closed
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *closed {
            return Err(io::Error::other("MCP process supervisor is shutting down"));
        }
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            // MCP children may echo granted credentials before the protocol handshake.
            // Keep raw stderr outside ordinary host logs; lifecycle errors remain typed.
            .stderr(Stdio::null())
            .kill_on_drop(true);
        let mut child = command.spawn()?;
        // Both handles were explicitly piped on the command immediately above.
        let stdout = child.stdout.take().expect("piped MCP stdout");
        let stdin = child.stdin.take().expect("piped MCP stdin");
        let cancellation = self.0.cancellation.child_token();
        let process_cancellation = cancellation.clone();
        let cleanup_failed = Arc::clone(&self.0.cleanup_failed);
        let forced_processes = Arc::clone(&self.0.forced_processes);
        let process_id = child.id();
        let (completion, finished) = watch::channel(None);
        // The tracker owns the join barrier, including attempts that never
        // produce a RunningService. No process task retains the supervisor Arc.
        self.0.tasks.spawn(async move {
            let outcome = std::panic::AssertUnwindSafe(reap_child(child, process_cancellation))
                .catch_unwind().await.unwrap_or_else(|_| Err(io::Error::other("MCP child owner panicked; reaping unconfirmed")));
            match &outcome {
                Ok(exit) => {
                    if exit.forced { forced_processes.fetch_add(1, Ordering::AcqRel); }
                    tracing::info!(name: "mcp.process.reaped", process_id,
                        outcome = if exit.forced { "forced" } else { "exited" },
                        exit_code = exit.exit_code, "Owned MCP child termination observed");
                }
                Err(_) => { cleanup_failed.store(true, Ordering::Release); }
            }
            completion.send_replace(Some(outcome.map_err(|error| error.kind())));
        });
        Ok(SupervisedStdioTransport {
            inner: AsyncRwTransport::new(stdout, stdin),
            cancellation,
            finished,
        })
    }

    /// Explicit force-stop request. The shutdown barrier still awaits real exit.
    pub(crate) fn request_force_stop(&self) {
        self.0.cancellation.cancel();
    }

    pub(crate) async fn shutdown(&self) -> io::Result<()> {
        let report = self.shutdown_report().await?;
        if report.forced_processes != 0 {
            tracing::warn!(forced_processes = report.forced_processes,
                "MCP scope released after explicit forced termination");
        }
        Ok(())
    }

    pub(crate) async fn shutdown_report(&self) -> io::Result<StdioShutdownReport> {
        {
            let mut closed = self
                .0
                .closed
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            *closed = true;
            self.request_force_stop();
            self.0.tasks.close();
        }
        // Cancel-safe: another shutdown call can resume this same join barrier.
        self.0.tasks.wait().await;
        if self.0.cleanup_failed.load(Ordering::Acquire) {
            return Err(io::Error::other("MCP child cleanup failed"));
        }
        Ok(StdioShutdownReport { forced_processes: self.0.forced_processes.load(Ordering::Acquire) })
    }
}

async fn reap_child(mut child: Child, cancellation: CancellationToken) -> io::Result<ChildExit> {
    tokio::select! {
        // Prefer already-observed exit over a simultaneous cancellation request.
        biased;
        result = child.wait() => result.map(|status| ChildExit { forced: false, exit_code: status.code() }),
        _ = cancellation.cancelled() => {
            // Cancellation is an explicit force-stop request. Sending a signal
            // does not release ownership; successful wait is the acknowledgement.
            let forced = child.start_kill().is_ok();
            if !forced {
                tracing::warn!("MCP force-stop signal failed; retaining ownership until child exit");
            }
            let status = child.wait().await?;
            Ok(ChildExit { forced, exit_code: status.code() })
        }
    }
}

pub(crate) struct SupervisedStdioTransport {
    inner: AsyncRwTransport<RoleClient, ChildStdout, ChildStdin>,
    cancellation: CancellationToken,
    finished: watch::Receiver<Option<Result<ChildExit, io::ErrorKind>>>,
}

pub(crate) struct StdioProcessReceipt {
    finished: watch::Receiver<Option<Result<ChildExit, io::ErrorKind>>>,
}

impl StdioProcessReceipt {
    pub(crate) async fn wait(mut self) -> io::Result<StdioShutdownReport> {
        loop {
            let finished = *self.finished.borrow_and_update();
            match finished {
                Some(Ok(exit)) => return Ok(StdioShutdownReport { forced_processes: usize::from(exit.forced) }),
                Some(Err(kind)) => return Err(io::Error::new(kind, "MCP child cleanup failed")),
                None => {}
            }
            self.finished.changed().await.map_err(|_| io::Error::other("MCP child reaper stopped without an acknowledgement"))?;
        }
    }
}

impl SupervisedStdioTransport {
    pub(crate) fn receipt(&self) -> StdioProcessReceipt {
        StdioProcessReceipt { finished: self.finished.clone() }
    }
}

impl Drop for SupervisedStdioTransport {
    fn drop(&mut self) {
        self.cancellation.cancel();
    }
}

impl Transport<RoleClient> for SupervisedStdioTransport {
    type Error = io::Error;

    fn send(
        &mut self,
        item: TxJsonRpcMessage<RoleClient>,
    ) -> impl Future<Output = io::Result<()>> + Send + 'static {
        self.inner.send(item)
    }

    async fn receive(&mut self) -> Option<RxJsonRpcMessage<RoleClient>> {
        self.inner.receive().await
    }

    async fn close(&mut self) -> io::Result<()> {
        // EOF is the graceful request. Keep the process owner alive until its
        // wait operation acknowledges actual termination. Cancellation of this
        // waiter cannot cancel the owner task or discard the retained outcome.
        let close_result = self.inner.close().await;
        loop {
            let finished = *self.finished.borrow_and_update();
            match finished {
                Some(Ok(exit)) if exit.forced => return Err(io::Error::new(
                    io::ErrorKind::Interrupted, "MCP child was forcibly terminated and reaped")),
                Some(Ok(_)) => return close_result,
                Some(Err(kind)) => return Err(io::Error::new(kind, "MCP child cleanup failed")),
                None => {}
            }
            self.finished
                .changed()
                .await
                .map_err(|_| io::Error::other("MCP child cleanup did not complete"))?;
        }
    }
}
