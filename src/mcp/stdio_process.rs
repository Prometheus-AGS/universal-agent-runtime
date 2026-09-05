//! Host ownership of stdio children, including cancelled initialization attempts.

use std::io;
use std::process::Stdio;
use std::sync::{
    Arc, RwLock,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use rmcp::{
    RoleClient,
    service::{RxJsonRpcMessage, TxJsonRpcMessage},
    transport::{Transport, async_rw::AsyncRwTransport},
};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::watch;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

// Match the existing stdio graceful-exit budget. A child that ignores EOF must
// be killed and reaped rather than outliving a cancelled readiness attempt.
const STDIO_EXIT_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Default)]
struct SupervisorState {
    closed: RwLock<bool>,
    cancellation: CancellationToken,
    tasks: TaskTracker,
    cleanup_failed: Arc<AtomicBool>,
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
            .stderr(Stdio::inherit())
            .kill_on_drop(true);
        let mut child = command.spawn()?;
        // Both handles were explicitly piped on the command immediately above.
        let stdout = child.stdout.take().expect("piped MCP stdout");
        let stdin = child.stdin.take().expect("piped MCP stdin");
        let cancellation = self.0.cancellation.child_token();
        let process_cancellation = cancellation.clone();
        let cleanup_failed = Arc::clone(&self.0.cleanup_failed);
        let (completion, finished) = watch::channel(None);
        // The tracker owns the join barrier, including attempts that never
        // produce a RunningService. No process task retains the supervisor Arc.
        self.0.tasks.spawn(async move {
            let success = reap_child(child, process_cancellation).await.is_ok();
            if !success {
                cleanup_failed.store(true, Ordering::Release);
            }
            completion.send_replace(Some(success));
        });
        Ok(SupervisedStdioTransport {
            inner: AsyncRwTransport::new(stdout, stdin),
            cancellation,
            finished,
        })
    }

    pub(crate) async fn shutdown(&self) -> io::Result<()> {
        {
            let mut closed = self
                .0
                .closed
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            *closed = true;
            self.0.cancellation.cancel();
            self.0.tasks.close();
        }
        // Cancel-safe: another shutdown call can resume this same join barrier.
        self.0.tasks.wait().await;
        if self.0.cleanup_failed.load(Ordering::Acquire) {
            return Err(io::Error::other("MCP child cleanup failed"));
        }
        Ok(())
    }
}

async fn reap_child(mut child: Child, cancellation: CancellationToken) -> io::Result<()> {
    tokio::select! {
        result = child.wait() => {
            if result.is_ok() { return Ok(()); }
            // A wait error is not proof that the process exited.
            return child.kill().await;
        }
        _ = cancellation.cancelled() => {}
    }
    match tokio::time::timeout(STDIO_EXIT_TIMEOUT, child.wait()).await {
        Ok(Ok(_)) => Ok(()),
        _ => child.kill().await,
    }
}

pub(crate) struct SupervisedStdioTransport {
    inner: AsyncRwTransport<RoleClient, ChildStdout, ChildStdin>,
    cancellation: CancellationToken,
    finished: watch::Receiver<Option<bool>>,
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
        // Start the exit deadline before closing the writer: a blocked write
        // must not prevent the host from killing a child that stopped reading.
        self.cancellation.cancel();
        let close_result = self.inner.close().await;
        loop {
            let finished = *self.finished.borrow_and_update();
            match finished {
                Some(true) => return close_result,
                Some(false) => return Err(io::Error::other("MCP child cleanup failed")),
                None => {}
            }
            self.finished
                .changed()
                .await
                .map_err(|_| io::Error::other("MCP child cleanup did not complete"))?;
        }
    }
}
