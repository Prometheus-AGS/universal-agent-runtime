//! High-level system for managing the lifecycle of agent actors.
//!
//! [`ActorCollaboration`] is the primary entry point: it spawns, tracks,
//! lists, and tears down [`AgentActor`](super::agent_actor::AgentActor)
//! instances.

use std::collections::HashMap;
use std::sync::Arc;

use ractor::{Actor, ActorRef};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::info;

use crate::config::LlmConfig;
use crate::mcp::registry::McpRegistry;
use crate::uar::runtime::native_skill::NativeSkillRegistry;

use super::agent_actor::{AgentActor, AgentActorArgs};
use super::messages::{ActorInfo, ActorStatus, AgentMessage, AgentReply};

// ---------------------------------------------------------------------------
// Actor handle — one per spawned actor
// ---------------------------------------------------------------------------

/// A handle to a running agent actor, stored in the collaboration system.
struct ActorHandle {
    /// Reference for sending messages.
    actor_ref: ActorRef<AgentMessage>,
    /// Background join handle for the actor's event loop.
    #[allow(dead_code)]
    join_handle: JoinHandle<()>,
    /// Agent artifact ID.
    agent_id: String,
}

impl std::fmt::Debug for ActorHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActorHandle")
            .field("agent_id", &self.agent_id)
            .finish()
    }
}

// ---------------------------------------------------------------------------
// ActorCollaboration
// ---------------------------------------------------------------------------

/// Manages the lifecycle of agent actors and provides APIs for spawning,
/// messaging, and stopping them.
///
/// Thread-safe — all methods take `&self` and internal state is behind a
/// [`RwLock`].
pub struct ActorCollaboration {
    /// Active actors keyed by their actor name/ID.
    actors: RwLock<HashMap<String, ActorHandle>>,
    /// Shared LLM config for spawning new orchestrators.
    llm_config: LlmConfig,
    /// Global MCP registry.
    mcp: Arc<McpRegistry>,
    /// Global native skill registry.
    native_skills: Arc<NativeSkillRegistry>,
}

impl std::fmt::Debug for ActorCollaboration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActorCollaboration")
            .field("actors", &"<locked>")
            .finish()
    }
}

impl ActorCollaboration {
    /// Create a new actor collaboration system.
    pub fn new(
        llm_config: LlmConfig,
        mcp: Arc<McpRegistry>,
        native_skills: Arc<NativeSkillRegistry>,
    ) -> Self {
        Self {
            actors: RwLock::new(HashMap::new()),
            llm_config,
            mcp,
            native_skills,
        }
    }

    /// Spawn a new agent actor.
    ///
    /// Returns the actor's name/ID on success.
    pub async fn spawn_agent(
        &self,
        actor_name: String,
        agent_id: String,
        system_prompt: Option<String>,
    ) -> anyhow::Result<String> {
        // Check for duplicate names
        {
            let actors = self.actors.read().await;
            if actors.contains_key(&actor_name) {
                anyhow::bail!("Actor with name '{actor_name}' already exists");
            }
        }

        let args = AgentActorArgs {
            agent_id: agent_id.clone(),
            llm_config: self.llm_config.clone(),
            mcp: Arc::clone(&self.mcp),
            native_skills: Arc::clone(&self.native_skills),
            system_prompt,
        };

        let (actor_ref, join_handle) = Actor::spawn(Some(actor_name.clone()), AgentActor, args)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to spawn actor: {e}"))?;

        info!(
            actor_name = %actor_name,
            agent_id = %agent_id,
            "Agent actor spawned"
        );

        let handle = ActorHandle {
            actor_ref,
            join_handle,
            agent_id,
        };

        self.actors.write().await.insert(actor_name.clone(), handle);

        Ok(actor_name)
    }

    /// Send a user prompt to a named actor and wait for the reply.
    pub async fn send_prompt(
        &self,
        actor_name: &str,
        content: String,
    ) -> anyhow::Result<AgentReply> {
        let actors = self.actors.read().await;
        let handle = actors
            .get(actor_name)
            .ok_or_else(|| anyhow::anyhow!("Actor '{actor_name}' not found"))?;

        let (tx, rx) = tokio::sync::oneshot::channel();
        handle
            .actor_ref
            .send_message(AgentMessage::UserPrompt {
                content,
                reply: Some(tx),
            })
            .map_err(|e| anyhow::anyhow!("Failed to send message: {e}"))?;

        rx.await
            .map_err(|_| anyhow::anyhow!("Actor stopped before replying"))
    }

    /// Request collaboration between two actors.
    ///
    /// Sends a collaboration request from `from_actor` to `to_actor` and
    /// returns the response.
    pub async fn collaborate(
        &self,
        from_actor: &str,
        to_actor: &str,
        task: String,
    ) -> anyhow::Result<AgentReply> {
        let actors = self.actors.read().await;

        let from = actors
            .get(from_actor)
            .ok_or_else(|| anyhow::anyhow!("Source actor '{from_actor}' not found"))?;

        let to = actors
            .get(to_actor)
            .ok_or_else(|| anyhow::anyhow!("Target actor '{to_actor}' not found"))?;

        let (tx, rx) = tokio::sync::oneshot::channel();
        to.actor_ref
            .send_message(AgentMessage::Collaborate {
                from_agent_id: from.agent_id.clone(),
                task,
                reply: tx,
            })
            .map_err(|e| anyhow::anyhow!("Failed to send collaboration message: {e}"))?;

        rx.await
            .map_err(|_| anyhow::anyhow!("Target actor stopped before replying"))
    }

    /// List all active actors.
    pub async fn list_actors(&self) -> Vec<ActorInfo> {
        let actors = self.actors.read().await;
        actors
            .iter()
            .map(|(name, handle)| ActorInfo {
                id: name.clone(),
                agent_id: handle.agent_id.clone(),
                status: ActorStatus::Running,
            })
            .collect()
    }

    /// Stop a named actor gracefully.
    pub async fn stop_actor(&self, actor_name: &str) -> anyhow::Result<()> {
        let handle = {
            let mut actors = self.actors.write().await;
            actors
                .remove(actor_name)
                .ok_or_else(|| anyhow::anyhow!("Actor '{actor_name}' not found"))?
        };

        // Send shutdown message (best effort).
        let _ = handle.actor_ref.send_message(AgentMessage::Shutdown);

        info!(actor_name = %actor_name, "Actor stopped");
        Ok(())
    }

    /// Stop all actors and drain the registry.
    pub async fn shutdown_all(&self) {
        let mut actors = self.actors.write().await;
        for (name, handle) in actors.drain() {
            let _ = handle.actor_ref.send_message(AgentMessage::Shutdown);
            info!(actor_name = %name, "Shutting down actor");
        }
    }

    /// Number of active actors.
    pub async fn len(&self) -> usize {
        self.actors.read().await.len()
    }

    /// Whether there are any active actors.
    pub async fn is_empty(&self) -> bool {
        self.actors.read().await.is_empty()
    }
}
