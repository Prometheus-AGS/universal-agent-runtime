//! Trusted graph-tool dispatch and ownership of an in-flight call. Cancelling
//! a node waiter does not drop a request that may already have caused effects.

use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use futures::{FutureExt, StreamExt};

use crate::llm::Orchestrator;
use crate::uar::a2ui::realtime::A2uiReplayBackbone;
use crate::uar::domain::events::RuntimeEventSink;

enum GraphOutput {
    Tool(String),
    Model(super::turn::GraphModelTurn),
    Unconfirmed(String),
}

type PendingCall = Pin<Box<dyn Future<Output = anyhow::Result<GraphOutput>> + Send>>;

#[derive(Default)]
struct Calls {
    pending: Option<PendingCall>,
    failure: Option<String>,
}

impl Calls {
    fn record_receipt(&mut self, result: &anyhow::Result<GraphOutput>) {
        let failure = match result {
            Ok(GraphOutput::Model(turn)) => turn.receipt_error.as_ref(),
            Ok(GraphOutput::Unconfirmed(error)) => Some(error),
            _ => None,
        };
        if let Some(error) = failure {
            self.failure.get_or_insert_with(|| error.clone());
        }
    }
}

/// Run-bound tool capability minted and retained by the trusted graph host.
/// Graph state cannot construct it or replace its policy, owner or transports.
pub struct GraphToolHost {
    run_id: String,
    orchestrator: Arc<Orchestrator>,
    events: Arc<dyn RuntimeEventSink>,
    closed: AtomicBool,
    cancellation: tokio_util::sync::CancellationToken,
    dialogue: crate::session::Session,
    conversation: crate::session::Session,
    persistence: Option<Arc<dyn crate::uar::persistence::PersistenceLayer>>,
    a2ui: Arc<dyn A2uiReplayBackbone>,
    presentations: Arc<super::super::presentations::RunPresentationSnapshot>,
    calls: tokio::sync::Mutex<Calls>,
}

impl std::fmt::Debug for GraphToolHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphToolHost")
            .field("run_id", &self.run_id)
            .finish_non_exhaustive()
    }
}

impl GraphToolHost {
    pub(crate) fn new(
        run_id: String,
        orchestrator: Arc<Orchestrator>,
        events: Arc<dyn RuntimeEventSink>,
        cancellation: &tokio_util::sync::CancellationToken,
        dialogue: crate::session::Session,
        conversation: crate::session::Session,
        persistence: Option<Arc<dyn crate::uar::persistence::PersistenceLayer>>,
        a2ui: Arc<dyn A2uiReplayBackbone>,
        presentations: Arc<super::super::presentations::RunPresentationSnapshot>,
    ) -> Self {
        Self {
            run_id,
            orchestrator,
            events,
            dialogue,
            conversation,
            persistence,
            a2ui,
            presentations,
            closed: AtomicBool::new(false),
            cancellation: cancellation.child_token(),
            calls: tokio::sync::Mutex::new(Calls::default()),
        }
    }

    pub(crate) async fn execute(
        &self,
        run_id: &str,
        step: u32,
        name: String,
        arguments: serde_json::Value,
    ) -> anyhow::Result<String> {
        anyhow::ensure!(run_id == self.run_id, "Graph tool belongs to another run");
        let mut calls = self.calls.lock().await;
        anyhow::ensure!(
            !self.closed.load(Ordering::Acquire),
            "Graph tool host is closed"
        );
        anyhow::ensure!(
            calls.failure.is_none(),
            "Graph history settlement remains unconfirmed"
        );
        anyhow::ensure!(
            calls.pending.is_none(),
            "Previous graph tool call requires settlement"
        );
        let run_id = self.run_id.clone();
        let orchestrator = Arc::clone(&self.orchestrator);
        let events = Arc::clone(&self.events);
        let cancellation = self.cancellation.clone();
        calls.pending = Some(Box::pin(async move {
            std::panic::AssertUnwindSafe(orchestrator.execute_graph_mcp_tool(
                &run_id,
                step,
                &name,
                arguments,
                events.as_ref(),
                &cancellation,
            ))
            .catch_unwind()
            .await
            .unwrap_or_else(|_| Err(anyhow::anyhow!("Graph tool execution panicked")))
            .map(GraphOutput::Tool)
        }));
        // Borrow the future in its retained slot. A dropped caller releases the
        // mutex but leaves the exact request for the root's shutdown path.
        let result = calls.pending.as_mut().expect("call installed above").await;
        calls.pending = None;
        match result? {
            GraphOutput::Tool(content) => Ok(content),
            GraphOutput::Model(_) => anyhow::bail!("Graph operation returned another receipt kind"),
            GraphOutput::Unconfirmed(error) => anyhow::bail!(error),
        }
    }

    pub(crate) async fn model_turn(
        &self,
        run_id: &str,
        mut messages: Vec<serde_json::Value>,
        system_prompt: Option<String>,
    ) -> anyhow::Result<super::turn::GraphModelTurn> {
        anyhow::ensure!(
            run_id == self.run_id,
            "Graph model turn belongs to another run"
        );
        crate::uar::runtime::context::normalize::normalize_provider_messages(&mut messages)?;
        let mut messages: Vec<crate::llm::Message> =
            serde_json::from_value(serde_json::Value::Array(messages))?;
        let mut calls = self.calls.lock().await;
        anyhow::ensure!(
            !self.closed.load(Ordering::Acquire),
            "Graph tool host is closed"
        );
        anyhow::ensure!(
            calls.failure.is_none(),
            "Graph history settlement remains unconfirmed"
        );
        anyhow::ensure!(
            calls.pending.is_none(),
            "Previous graph operation requires settlement"
        );
        let run_id = self.run_id.clone();
        let orchestrator = Arc::clone(&self.orchestrator);
        let events = Arc::clone(&self.events);
        let cancellation = self.cancellation.clone();
        let conversation = self.conversation.clone();
        let persistence = self.persistence.clone();
        let mut transcript = super::turn::GraphTurnTranscript::new(
            run_id.clone(),
            messages.clone(),
            self.dialogue.clone(),
            conversation.clone(),
            Arc::clone(&self.a2ui),
            Arc::clone(&self.presentations),
            cancellation.clone(),
        );
        // Node instructions apply to this request only, not subsequent nodes
        // or the persisted dialogue returned by the transcript.
        if let Some(prompt) = system_prompt {
            messages.insert(
                0,
                crate::llm::Message {
                    role: crate::llm::MessageRole::System,
                    content: crate::llm::MessageContent::text(prompt),
                    tool_call_id: None,
                    tool_calls: None,
                },
            );
        }
        let operation = async move {
            let outcome = std::panic::AssertUnwindSafe(async {
                let stream = orchestrator
                    .graph_chat_with_history(&run_id, messages, &cancellation)
                    .await?;
                futures::pin_mut!(stream);
                loop {
                    if cancellation.is_cancelled() && !transcript.has_outstanding_tools() {
                        anyhow::bail!("Graph model turn cancelled");
                    }
                    let event = stream.next().await.ok_or_else(|| {
                        anyhow::anyhow!("Graph model turn ended without a terminal event")
                    })?;
                    if transcript.observe(event, events.as_ref()).await? {
                        anyhow::ensure!(!cancellation.is_cancelled(), "Graph model turn cancelled");
                        return Ok::<_, anyhow::Error>(());
                    }
                }
            })
            .catch_unwind()
            .await
            .unwrap_or_else(|_| Err(anyhow::anyhow!("Graph model turn panicked")));
            let mut result = transcript.finish(
                outcome.err().map(|error| error.to_string()),
                cancellation.is_cancelled(),
            );
            if let Some(db) = persistence
                && db.save_session(&conversation).await.is_err()
            {
                result.error = Some("Graph session persistence failed".into());
                result.receipt_error = result.error.clone();
            }
            Ok(GraphOutput::Model(result))
        };
        calls.pending = Some(Box::pin(async move {
            // Cover finalization and persistence too: a completed, panicked
            // future must never remain available for shutdown to poll again.
            std::panic::AssertUnwindSafe(operation)
                .catch_unwind()
                .await
                .unwrap_or_else(|_| {
                    Ok(GraphOutput::Unconfirmed(
                        "Graph history settlement panicked".into(),
                    ))
                })
        }));
        let result = calls.pending.as_mut().expect("turn installed above").await;
        calls.pending = None;
        calls.record_receipt(&result);
        match result? {
            GraphOutput::Model(turn) => Ok(turn),
            GraphOutput::Tool(_) => anyhow::bail!("Graph operation returned another receipt kind"),
            GraphOutput::Unconfirmed(error) => anyhow::bail!(error),
        }
    }

    /// Close admission and drain the exact pending future without replaying it.
    /// This proves local request settlement, not rollback of remote effects.
    pub(crate) async fn shutdown(&self) -> anyhow::Result<()> {
        self.closed.store(true, Ordering::Release);
        self.cancellation.cancel();
        let mut calls = self.calls.lock().await;
        if let Some(pending) = calls.pending.as_mut() {
            let result = pending.await;
            calls.pending = None;
            calls.record_receipt(&result);
        }
        if let Some(error) = &calls.failure {
            anyhow::bail!(error.clone());
        }
        Ok(())
    }

    pub(crate) fn check_remote_compatibility(&self, run_id: &str) -> anyhow::Result<()> {
        anyhow::ensure!(
            run_id == self.run_id,
            "Graph delegation belongs to another run"
        );
        anyhow::ensure!(
            !self.closed.load(Ordering::Acquire) && !self.cancellation.is_cancelled(),
            "Graph tool host is closed"
        );
        self.orchestrator.check_graph_remote_compatibility()
    }
}
