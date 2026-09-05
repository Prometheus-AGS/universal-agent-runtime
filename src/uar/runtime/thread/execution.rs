//! Lossless host completion capture for mailbox callers of the shared run
//! kernel. This is not a second LLM loop and does not subscribe to lossy SSE.

use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

use crate::uar::domain::events::NormalizedEvent;

use super::AgentThreadResult;

/// Owned by the run emitter. Dropping it before a terminal event closes the
/// receiver; disappearance is an error, never an empty successful response.
pub(crate) struct RunCompletionCapture {
    output: String,
    failure: Option<(String, String)>,
    terminal: Option<AgentThreadResult>,
    reply: Option<oneshot::Sender<AgentThreadResult>>,
    keeps_run_alive: bool,
}

impl std::fmt::Debug for RunCompletionCapture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunCompletionCapture")
            .field("pending", &self.reply.is_some())
            .finish_non_exhaustive()
    }
}

impl RunCompletionCapture {
    pub(crate) fn channel() -> (Self, oneshot::Receiver<AgentThreadResult>) {
        let (reply, receiver) = oneshot::channel();
        (
            Self {
                output: String::new(),
                failure: None,
                terminal: None,
                reply: Some(reply),
                keeps_run_alive: true,
            },
            receiver,
        )
    }

    /// A root's internal cleanup observer is not an independent user/mailbox.
    pub(crate) fn observer_channel() -> (Self, oneshot::Receiver<AgentThreadResult>) {
        let (mut capture, receiver) = Self::channel();
        capture.keeps_run_alive = false;
        (capture, receiver)
    }

    /// A mailbox waiting on completion owns work independently of SSE viewers.
    /// An abandoned receiver does not keep that ownership alive.
    pub(crate) fn has_waiter(&self) -> bool {
        self.keeps_run_alive && self.reply.as_ref().is_some_and(|reply| !reply.is_closed())
    }

    /// Called in the emitter's existing event-order lock before broadcasting.
    /// A terminal event freezes the result but does not release the mailbox.
    /// The run's unwind guard releases it after the executing future exits.
    pub(crate) fn record(&mut self, event: &NormalizedEvent) {
        if self.reply.is_none() || self.terminal.is_some() {
            return;
        }
        let terminal = match event {
            NormalizedEvent::ChatDelta { text_delta, .. } => {
                self.output.push_str(text_delta);
                None
            }
            NormalizedEvent::SycophancyCorrected { corrected_text, .. } => {
                self.output.clone_from(corrected_text);
                None
            }
            NormalizedEvent::Error { code, message, .. } => {
                if self.failure.is_none() {
                    self.failure = Some((code.clone(), message.clone()));
                }
                None
            }
            NormalizedEvent::Cancelled { .. } => Some(AgentThreadResult::Cancelled),
            NormalizedEvent::RunDone { .. } | NormalizedEvent::RunDoneWithUsage { .. } => {
                Some(match self.failure.take() {
                    Some((code, message)) => AgentThreadResult::Failed { code, message },
                    None => AgentThreadResult::Completed {
                        output: std::mem::take(&mut self.output),
                    },
                })
            }
            _ => None,
        };
        if let Some(result) = terminal {
            self.output.clear();
            self.terminal = Some(result);
        }
    }

    fn finish(&mut self) {
        let Some(reply) = self.reply.take() else {
            return;
        };
        let terminal = if std::thread::panicking() {
            Some(AgentThreadResult::Failed {
                code: "kernel_panicked".into(),
                message: "Run kernel failed while unwinding".into(),
            })
        } else {
            self.terminal.take()
        };
        if let Some(result) = terminal {
            let _ = reply.send(result);
        }
        // No terminal event closes the sender, never reports empty success.
    }
}

/// Owned by assembly until launch, then by the executing run future. Declare
/// it before execution locals so streams/tool futures unwind before its reply.
/// It holds no emitter, avoiding an approval/emitter ownership cycle.
pub(crate) struct RunCompletionGuard {
    capture: Option<Arc<Mutex<RunCompletionCapture>>>,
}

impl RunCompletionGuard {
    pub(crate) fn new(capture: Option<Arc<Mutex<RunCompletionCapture>>>) -> Self {
        Self { capture }
    }

    /// A caught unwind or failed async finalizer supersedes an earlier terminal
    /// event. The mailbox must not receive success before owned cleanup finishes.
    pub(crate) fn fail(&self, code: &str, message: &str) {
        if let Some(capture) = &self.capture {
            let mut capture = capture
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if capture.reply.is_some() {
                capture.output.clear();
                capture.terminal = Some(AgentThreadResult::Failed {
                    code: code.to_owned(),
                    message: message.to_owned(),
                });
            }
        }
    }
}

impl Drop for RunCompletionGuard {
    fn drop(&mut self) {
        if let Some(capture) = &self.capture {
            capture
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .finish();
        }
    }
}
