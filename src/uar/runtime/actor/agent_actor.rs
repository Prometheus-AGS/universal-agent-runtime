//! An actor mailbox over the shared thread/run host. Actors do not construct
//! an LLM driver, orchestrator, tool registry, or private conversation history.

use std::sync::Arc;

use ractor::{Actor, ActorProcessingErr, ActorRef};
use tokio_util::sync::CancellationToken;

use crate::uar::runtime::thread::{AgentThreadResult, actor_host::ActorThreadSession};

use super::messages::{ActorStatus, AgentMessage, AgentReply};

/// Mailbox-owned state. The host owns thread persistence and run execution.
pub struct AgentActorState {
    pub agent_id: String,
    pub status: ActorStatus,
    session: Arc<tokio::sync::Mutex<ActorThreadSession>>,
    cancellation: CancellationToken,
}

impl std::fmt::Debug for AgentActorState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentActorState")
            .field("agent_id", &self.agent_id)
            .field("status", &self.status)
            .finish_non_exhaustive()
    }
}

/// Serialized mailbox adapter, not an alternative agent execution kernel.
#[derive(Debug)]
pub struct AgentActor;

/// Trusted-host arguments. Identity and artifact are resolved before spawning.
pub struct AgentActorArgs {
    pub agent_id: String,
    pub cancellation: CancellationToken,
    pub(crate) session: Arc<tokio::sync::Mutex<ActorThreadSession>>,
}

impl std::fmt::Debug for AgentActorArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentActorArgs")
            .field("agent_id", &self.agent_id)
            .finish_non_exhaustive()
    }
}

impl AgentActorState {
    async fn execute(&mut self, content: String) -> AgentReply {
        if self.cancellation.is_cancelled() {
            return AgentReply {
                content: "Actor has been stopped".into(),
                success: false,
                metadata: serde_json::json!({"code": "actor_stopped"}),
            };
        }
        match self.session.lock().await.execute(content).await {
            Ok(record) => {
                let metadata = serde_json::json!({
                    "thread_id": record.thread.thread_id,
                    "root_run_id": record.thread.root_run_id,
                    "run_id": record.thread.run_id,
                    "status": record.thread.status,
                });
                let (content, success) = match record.thread.result {
                    Some(AgentThreadResult::Completed { output }) => (output, true),
                    Some(AgentThreadResult::Failed { message, .. }) => (message, false),
                    Some(AgentThreadResult::Cancelled) => ("Run cancelled".into(), false),
                    None => ("Thread returned without a terminal result".into(), false),
                };
                AgentReply {
                    content,
                    success,
                    metadata,
                }
            }
            Err(error) => {
                tracing::error!(agent_id = %self.agent_id, error = %error, "Actor host execution failed");
                AgentReply {
                    content: "Actor host execution failed".into(),
                    success: false,
                    metadata: serde_json::json!({"code": "actor_host_failed"}),
                }
            }
        }
    }
}

impl Actor for AgentActor {
    type Msg = AgentMessage;
    type State = AgentActorState;
    type Arguments = AgentActorArgs;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let agent_id = args.agent_id;
        let session = args.session;
        Ok(AgentActorState {
            agent_id,
            status: ActorStatus::Running,
            session,
            cancellation: args.cancellation,
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            AgentMessage::UserRun {
                run_id,
                content,
                artifacts,
                reply,
            } => {
                let result = if state.cancellation.is_cancelled() {
                    Err(super::messages::ActorRunError::Stopped)
                } else {
                    state
                        .session
                        .lock()
                        .await
                        .execute_named(content, run_id, Some(artifacts.clone()))
                        .await
                        .map_err(super::messages::ActorRunError::Host)
                };
                let result = artifacts
                    .close()
                    .map_err(super::messages::ActorRunError::Host)
                    .and(result);
                let _ = reply.send(result);
            }
            AgentMessage::UserPrompt { content, reply } => {
                let result = state.execute(content).await;
                if let Some(reply) = reply {
                    let _ = reply.send(result);
                }
            }
            AgentMessage::Collaborate { reply, .. } => {
                // A mailbox message contains no verified root capability. The
                // authenticated collaboration host now enters ThreadService.
                let _ = reply.send(AgentReply {
                    content: "Collaboration requires the verified root thread host".into(),
                    success: false,
                    metadata: serde_json::json!({"code": "collaboration_host_required"}),
                });
            }
            AgentMessage::ToolResult { .. } => {
                // Only the governed run loop can pair and ingest tool results.
                return Err("Actor mailboxes do not accept standalone tool results".into());
            }
            AgentMessage::Shutdown => {
                state.cancellation.cancel();
                state.status = ActorStatus::Stopping;
                myself.stop(None);
            }
        }
        Ok(())
    }

    async fn post_stop(
        &self,
        _myself: ActorRef<Self::Msg>,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        state.cancellation.cancel();
        if let Err(error) = state.session.lock().await.settle_uncertain().await {
            tracing::error!(agent_id = %state.agent_id, error = %error,
                "Actor stopped with an unresolved thread transition");
        }
        state.status = ActorStatus::Stopped;
        Ok(())
    }
}
