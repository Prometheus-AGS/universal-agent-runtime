//! High-level system for managing the lifecycle of agent actors.
//!
//! [`ActorCollaboration`] is the primary entry point: it spawns, tracks,
//! lists, and tears down [`AgentActor`](super::agent_actor::AgentActor)
//! instances.

use std::collections::HashMap;
use std::sync::Arc;

use ractor::{Actor, ActorRef};
use tokio::sync::{Mutex, RwLock, watch};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::uar::persistence::{PersistenceLayer, agent_threads::PersistedAgentThread};
use crate::uar::runtime::manager::RunManager;

use super::agent_actor::{AgentActor, AgentActorArgs};
use super::messages::{ActorInfo, ActorOwner, ActorStatus, AgentMessage, AgentReply};

// ---------------------------------------------------------------------------
// Actor handle — one per spawned actor
// ---------------------------------------------------------------------------

/// A handle to a running agent actor, stored in the collaboration system.
struct ActorHandle {
    /// Reference for sending messages.
    actor_ref: ActorRef<AgentMessage>,
    /// Background join handle for the actor's event loop.
    join_handle: Mutex<ActorJoin>,
    /// Agent artifact ID.
    agent_id: String,
    session_id: String,
    cancellation: CancellationToken,
    thread: watch::Receiver<Option<PersistedAgentThread>>,
    owned_root: Arc<Mutex<Option<crate::uar::runtime::thread::actor_host::ActorRootBinding>>>,
    session: Arc<Mutex<crate::uar::runtime::thread::actor_host::ActorThreadSession>>,
}

struct ActorJoin {
    handle: Option<JoinHandle<()>>,
    failure: Option<String>,
}

type ActorRegistry = Arc<RwLock<HashMap<(ActorOwner, String), Arc<ActorHandle>>>>;

impl std::fmt::Debug for ActorHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActorHandle")
            .field("agent_id", &self.agent_id)
            .finish()
    }
}

impl ActorHandle {
    async fn finish_root(&self) -> anyhow::Result<()> {
        // Mailbox join precedes this call. Retain the complete session, not
        // only its producer, so uncertain start/terminal writes survive stop.
        self.session.lock().await.finish_abandoned().await
    }

    fn is_finished(&self) -> bool {
        self.join_handle
            .try_lock()
            .is_ok_and(|handle| handle.handle.as_ref().is_none_or(JoinHandle::is_finished))
    }

    async fn join(&self) -> anyhow::Result<()> {
        let mut slot = self.join_handle.lock().await;
        if let Some(handle) = slot.handle.as_mut() {
            let result = handle.await;
            slot.handle = None;
            slot.failure = result.err().map(|error| error.to_string());
        }
        // Await by reference: cancelling an HTTP stop request leaves the handle
        // in the registry, where shutdown can still join it. Consume it only
        // after completion, so a second stop never polls a completed handle.
        match &slot.failure {
            Some(error) => Err(anyhow::anyhow!("Actor mailbox failed: {error}")),
            None => Ok(()),
        }
    }
}

// ---------------------------------------------------------------------------
// ActorCollaboration
// ---------------------------------------------------------------------------

/// A transport's invocation receipt, not ownership of the actor worker.
/// Only snapshots matching run_id belong to this invocation.
pub struct ActorTurn {
    pub run_id: String,
    pub thread: watch::Receiver<Option<PersistedAgentThread>>,
    pub(crate) artifacts: crate::uar::runtime::thread::artifacts::RunArtifactCollector,
    pub completion: tokio::sync::oneshot::Receiver<
        Result<PersistedAgentThread, super::messages::ActorRunError>,
    >,
}

impl std::fmt::Debug for ActorTurn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActorTurn")
            .field("run_id", &self.run_id)
            .finish_non_exhaustive()
    }
}

/// Exact host-resolved actor capability. Reusing its public name cannot retarget
/// a transport's submission or cleanup to a replacement actor.
#[derive(Clone)]
pub struct ActorSession {
    handle: Arc<ActorHandle>,
    registry: ActorRegistry,
    key: (ActorOwner, String),
}

impl std::fmt::Debug for ActorSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActorSession")
            .field("agent_id", &self.handle.agent_id)
            .finish_non_exhaustive()
    }
}

impl ActorSession {
    pub fn submit_prompt(&self, content: String) -> anyhow::Result<ActorTurn> {
        let handle = &self.handle;
        anyhow::ensure!(
            !handle.cancellation.is_cancelled() && !handle.is_finished(),
            "Actor is stopping or stopped"
        );
        let run_id = uuid::Uuid::new_v4().to_string();
        let artifacts = crate::uar::runtime::thread::artifacts::RunArtifactCollector::new(
            self.key.0.clone(),
            run_id.clone(),
        );
        let (reply, completion) = tokio::sync::oneshot::channel();
        handle
            .actor_ref
            .send_message(AgentMessage::UserRun {
                run_id: run_id.clone(),
                content,
                artifacts: artifacts.clone(),
                reply,
            })
            .map_err(|error| anyhow::anyhow!("Actor submission failed: {error}"))?;
        Ok(ActorTurn {
            run_id,
            thread: handle.thread.clone(),
            artifacts,
            completion,
        })
    }

    pub async fn stop(&self) -> anyhow::Result<()> {
        self.handle.cancellation.cancel();
        let _ = self.handle.actor_ref.send_message(AgentMessage::Shutdown);
        let joined = self.handle.join().await;
        self.handle.finish_root().await?;
        joined?;
        let mut registry = self.registry.write().await;
        if registry
            .get(&self.key)
            .is_some_and(|current| Arc::ptr_eq(current, &self.handle))
        {
            registry.remove(&self.key);
        }
        Ok(())
    }
}

/// Manages the lifecycle of agent actors and provides APIs for spawning,
/// messaging, and stopping them.
///
/// Thread-safe — all methods take `&self` and internal state is behind a
/// [`RwLock`].
pub struct ActorCollaboration {
    /// Active actors keyed by their actor name/ID.
    actors: ActorRegistry,
    /// All inference and tool execution goes through this shared kernel.
    manager: Arc<RunManager>,
    /// One shared provider for actor thread records, including memory mode.
    persistence: Option<Arc<dyn PersistenceLayer>>,
    /// Closing actor admission must not cancel unrelated runs in the manager.
    cancellation: CancellationToken,
}

impl std::fmt::Debug for ActorCollaboration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActorCollaboration")
            .field("actors", &"<locked>")
            .finish()
    }
}

impl ActorCollaboration {
    pub(crate) fn run_usage(
        &self,
        run_id: &str,
    ) -> crate::uar::runtime::cost_budget::RunUsageSnapshot {
        self.manager.run_usage(run_id)
    }

    /// Create a new actor collaboration system.
    pub fn new(manager: Arc<RunManager>) -> Self {
        let persistence = manager.persistence.clone();
        #[cfg(feature = "in-memory-backend")]
        let persistence = persistence.or_else(|| {
            Some(
                Arc::new(crate::uar::persistence::providers::memory::InMemoryProvider::new())
                    as Arc<dyn PersistenceLayer>,
            )
        });
        Self {
            actors: Arc::new(RwLock::new(HashMap::new())),
            cancellation: manager.root_cancellation_token().child_token(),
            manager,
            persistence,
        }
    }

    /// Spawn a new agent actor.
    ///
    /// Returns the actor's name/ID on success.
    pub async fn spawn_agent(
        &self,
        owner: &ActorOwner,
        actor_name: String,
        agent_id: String,
        system_prompt: Option<String>,
    ) -> anyhow::Result<String> {
        self.spawn_session(owner, actor_name.clone(), agent_id, system_prompt)
            .await?;
        Ok(actor_name)
    }

    /// Publish and return one exact actor instance, with no name lookup gap.
    pub async fn spawn_session(
        &self,
        owner: &ActorOwner,
        actor_name: String,
        agent_id: String,
        system_prompt: Option<String>,
    ) -> anyhow::Result<ActorSession> {
        self.spawn_session_inner(owner, actor_name, agent_id, system_prompt, None)
            .await
    }

    pub(crate) async fn spawn_governed_session(
        &self,
        owner: &ActorOwner,
        actor_name: String,
        agent_id: String,
        policy: crate::uar::domain::policy::RunPolicy,
        presentation_negotiation: Option<
            crate::uar::a2ui::presentation_selection::PresentationNegotiation,
        >,
        budgets: crate::uar::runtime::thread::policy_intersection::ThreadBudgets,
        usage_grant: crate::uar::api::a2a::contract::UarUsageGrant,
        sandbox: crate::uar::runtime::thread::policy_intersection::SandboxPermissions,
        accounting_id: String,
    ) -> anyhow::Result<ActorSession> {
        self.spawn_session_inner(
            owner,
            actor_name,
            agent_id,
            None,
            Some((
                policy,
                presentation_negotiation,
                budgets,
                usage_grant,
                sandbox,
                accounting_id,
            )),
        )
        .await
    }

    async fn spawn_session_inner(
        &self,
        owner: &ActorOwner,
        actor_name: String,
        agent_id: String,
        system_prompt: Option<String>,
        constraints: Option<(
            crate::uar::domain::policy::RunPolicy,
            Option<crate::uar::a2ui::presentation_selection::PresentationNegotiation>,
            crate::uar::runtime::thread::policy_intersection::ThreadBudgets,
            crate::uar::api::a2a::contract::UarUsageGrant,
            crate::uar::runtime::thread::policy_intersection::SandboxPermissions,
            String,
        )>,
    ) -> anyhow::Result<ActorSession> {
        let persistence = self.persistence.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Actor threads require a configured persistence provider")
        })?;
        let mut artifact = self.manager.resolve_registered_agent(&agent_id).await?;
        // Preserve the authenticated root user's explicit prompt override on
        // this execution copy; never mutate the registered artifact or policy.
        if let Some(system_prompt) = system_prompt {
            artifact.prompt.system = system_prompt;
        }
        // Reserve the owner-qualified name through publication. The previous
        // read-then-write sequence allowed concurrent spawns to overwrite one
        // another while leaving an untracked actor running.
        let key = (owner.clone(), actor_name.clone());
        let mut actors = self.actors.write().await;
        if self.cancellation.is_cancelled() {
            anyhow::bail!("Actor collaboration system is shutting down");
        }
        if actors.contains_key(&key) {
            anyhow::bail!("Actor with name '{actor_name}' already exists");
        }

        let session_id = uuid::Uuid::new_v4().to_string();
        let constraints = match constraints {
            Some((
                policy,
                presentation_negotiation,
                budgets,
                usage_grant,
                sandbox,
                accounting_id,
            )) => {
                artifact.policy.tools.execution_mode =
                    crate::uar::runtime::thread::policy_intersection::intersect_execution_mode(
                        &sandbox.execution_mode,
                        &artifact.policy.tools.execution_mode,
                    );
                Some(
                    crate::uar::runtime::thread::actor_host::RemoteRootConstraints {
                        policy: self
                            .manager
                            .resolve_remote_policy_constraint(&artifact, owner, &session_id, policy)
                            .await?,
                        budgets,
                        usage_grant: crate::uar::runtime::cost_budget::RemoteUsageGrantBinding {
                            accounting_id,
                            grant: usage_grant,
                            started_at: std::time::Instant::now(),
                        },
                        sandbox,
                        presentation_negotiation,
                    },
                )
            }
            None => None,
        };
        let cancellation = self.cancellation.child_token();
        let (state, thread) = watch::channel(None);
        let owned_root = Arc::new(Mutex::new(None));
        let session = Arc::new(Mutex::new(
            crate::uar::runtime::thread::actor_host::ActorThreadSession::new_with_constraints(
                owner.clone(),
                artifact,
                session_id.clone(),
                Arc::clone(&self.manager),
                Arc::clone(persistence),
                cancellation.clone(),
                state,
                Arc::clone(&owned_root),
                constraints,
            ),
        ));
        let args = AgentActorArgs {
            agent_id: agent_id.clone(),
            session: Arc::clone(&session),
            cancellation: cancellation.clone(),
        };

        // Public names belong to our owner-scoped registry, not ractor's global
        // name registry. Different owners may safely choose the same name.
        let (actor_ref, join_handle) = Actor::spawn(None, AgentActor, args)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to spawn actor: {e}"))?;

        info!(
            actor_name = %actor_name,
            agent_id = %agent_id,
            "Agent actor spawned"
        );

        let handle = Arc::new(ActorHandle {
            actor_ref,
            join_handle: Mutex::new(ActorJoin {
                handle: Some(join_handle),
                failure: None,
            }),
            agent_id,
            session_id,
            cancellation,
            thread,
            owned_root,
            session,
        });

        actors.insert(key.clone(), Arc::clone(&handle));

        Ok(ActorSession {
            handle,
            registry: Arc::clone(&self.actors),
            key,
        })
    }

    /// Resolve an owner-qualified actor once for a transport's entire task.
    pub async fn session(
        &self,
        owner: &ActorOwner,
        actor_name: &str,
    ) -> anyhow::Result<ActorSession> {
        let actors = self.actors.read().await;
        let key = (owner.clone(), actor_name.to_owned());
        let handle = actors
            .get(&key)
            .ok_or_else(|| anyhow::anyhow!("Actor not found"))?;
        Ok(ActorSession {
            handle: Arc::clone(handle),
            registry: Arc::clone(&self.actors),
            key,
        })
    }

    /// Send a user prompt to a named actor and wait for the reply.
    pub async fn send_prompt(
        &self,
        owner: &ActorOwner,
        actor_name: &str,
        content: String,
    ) -> anyhow::Result<AgentReply> {
        let actors = self.actors.read().await;
        let actor_ref = actors
            .get(&(owner.clone(), actor_name.to_string()))
            .ok_or_else(|| anyhow::anyhow!("Actor '{actor_name}' not found"))?
            .actor_ref
            .clone();
        drop(actors);

        let (tx, rx) = tokio::sync::oneshot::channel();
        actor_ref
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
    /// Delegates to the target's named artifact as a child of the source's live
    /// root. An idle/finished source is not silently replaced by a new root.
    pub async fn collaborate(
        &self,
        owner: &ActorOwner,
        from_actor: &str,
        to_actor: &str,
        task: String,
    ) -> anyhow::Result<AgentReply> {
        let actors = self.actors.read().await;

        let from = actors
            .get(&(owner.clone(), from_actor.to_string()))
            .ok_or_else(|| anyhow::anyhow!("Source actor '{from_actor}' not found"))?;

        let to = actors
            .get(&(owner.clone(), to_actor.to_string()))
            .ok_or_else(|| anyhow::anyhow!("Target actor '{to_actor}' not found"))?;
        anyhow::ensure!(
            !from.cancellation.is_cancelled()
                && !to.cancellation.is_cancelled()
                && !from.is_finished()
                && !to.is_finished(),
            "Actor is stopping or stopped"
        );
        let from_agent_id = from.agent_id.clone();
        let owned_root = Arc::clone(&from.owned_root);
        let artifact_id = to.agent_id.clone();
        drop(actors);

        let root = owned_root
            .lock()
            .await
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Source actor has no live root turn"))?;
        let thread = self
            .manager
            .collaborate_actor_root(
                owner,
                &root,
                crate::uar::runtime::thread::spawn::AgentSpawnRequest {
                    artifact_id,
                    delegated_prompt: task,
                    task_name: None,
                    history_fork: crate::uar::runtime::thread::spawn::HistoryForkMode::None,
                },
            )
            .await?;
        let metadata = serde_json::json!({
            "thread_id": thread.thread_id, "parent_thread_id": thread.parent_thread_id,
            "root_run_id": thread.root_run_id, "run_id": thread.run_id,
            "status": thread.status, "collaboration_from": from_agent_id,
        });
        let (content, success) = match thread.result {
            Some(crate::uar::runtime::thread::AgentThreadResult::Completed { output }) => {
                (output, true)
            }
            Some(crate::uar::runtime::thread::AgentThreadResult::Failed { message, .. }) => {
                (message, false)
            }
            Some(crate::uar::runtime::thread::AgentThreadResult::Cancelled) => {
                ("Run cancelled".into(), false)
            }
            None => anyhow::bail!("Child returned without a terminal result"),
        };
        Ok(AgentReply {
            content,
            success,
            metadata,
        })
    }

    /// List all active actors.
    pub async fn list_actors(&self, owner: &ActorOwner) -> Vec<ActorInfo> {
        let actors = self.actors.read().await;
        actors
            .iter()
            .filter(|((actor_owner, _), _)| actor_owner == owner)
            .map(|((_, name), handle)| {
                let thread = handle.thread.borrow().clone();
                ActorInfo {
                    id: name.clone(),
                    agent_id: handle.agent_id.clone(),
                    status: if handle.is_finished() {
                        ActorStatus::Stopped
                    } else if handle.cancellation.is_cancelled() {
                        ActorStatus::Stopping
                    } else {
                        ActorStatus::Running
                    },
                    session_id: handle.session_id.clone(),
                    thread_id: thread
                        .as_ref()
                        .map(|record| record.thread.thread_id.clone()),
                    run_id: thread.and_then(|record| record.thread.run_id),
                }
            })
            .collect()
    }

    /// Stop a named actor gracefully.
    pub async fn stop_actor(&self, owner: &ActorOwner, actor_name: &str) -> anyhow::Result<()> {
        let session = self.session(owner, actor_name).await?;
        info!(actor_name = %actor_name, "Actor shutdown requested");
        session
            .stop()
            .await
            .map_err(|error| anyhow::anyhow!("Actor '{actor_name}' shutdown failed: {error}"))
    }

    /// Permanently close admission, stop all actors, and drain the registry.
    pub async fn shutdown_all(&self) -> anyhow::Result<()> {
        // Cancel before waiting for the registry lock. A spawn already holding
        // it may finish publication, but its token is cancelled and the drain
        // below owns its join handle. Later spawns cannot reopen the system.
        self.cancellation.cancel();
        let stopping = {
            let actors = self.actors.read().await;
            actors
                .iter()
                .map(|(key, handle)| (key.clone(), Arc::clone(handle)))
                .collect::<Vec<_>>()
        };
        // Signal every mailbox first, then join outside the registry lock.
        // Dropping a JoinHandle detaches work; it does not finish shutdown.
        for ((_, name), handle) in &stopping {
            handle.cancellation.cancel();
            let _ = handle.actor_ref.send_message(AgentMessage::Shutdown);
            info!(actor_name = %name, "Shutting down actor");
        }
        let mut failure = None;
        for ((owner, name), handle) in stopping {
            let joined = handle.join().await;
            let settled = handle.finish_root().await;
            if let Err(error) = joined.and(settled) {
                tracing::error!(actor_name = %name, %error, "Actor shutdown failed");
                failure.get_or_insert(error);
                continue;
            }
            // Admission is permanently closed. Retain each handle until its
            // join finishes even if the cleanup future itself is cancelled.
            self.actors.write().await.remove(&(owner, name));
        }
        failure.map_or(Ok(()), Err)
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
