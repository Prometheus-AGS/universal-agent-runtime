//! Owned root workers for graph requests that do not have an actor mailbox.
//! The HTTP preparation waiter is not the owner of persistence or cleanup.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use futures::FutureExt;
use tokio::sync::{Mutex as AsyncMutex, oneshot, watch};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::uar::runtime::{manager::RunManager, turn::RunExecutionRequest};

use super::actor_host::ActorThreadSession;

#[derive(Default)]
pub(crate) struct GraphRootSupervisor {
    jobs: Mutex<Jobs>,
}

#[derive(Default)]
struct Jobs {
    closed: bool,
    entries: BTreeMap<String, Arc<Job>>,
}

struct Job {
    cancellation: CancellationToken,
    session: Arc<AsyncMutex<ActorThreadSession>>,
    joined: AsyncMutex<Joined>,
}

struct Joined {
    handle: Option<JoinHandle<Result<(), String>>>,
    failure: Option<String>,
}

struct PreparationLifetime(Option<CancellationToken>);

impl Drop for PreparationLifetime {
    fn drop(&mut self) {
        if let Some(cancellation) = &self.0 {
            cancellation.cancel();
        }
    }
}

impl Job {
    fn finished(&self) -> bool {
        self.joined
            .try_lock()
            .is_ok_and(|joined| joined.handle.as_ref().is_none_or(JoinHandle::is_finished))
    }

    async fn join(&self) -> anyhow::Result<()> {
        let mut joined = self.joined.lock().await;
        if let Some(handle) = joined.handle.as_mut() {
            let outcome = handle.await;
            // Save consumption before another await, including after a panic.
            joined.handle = None;
            joined.failure = match outcome {
                Ok(Ok(())) => None,
                Ok(Err(error)) => Some(error),
                Err(error) => Some(error.to_string()),
            };
        }
        self.cancellation.cancel();
        // The worker is no longer running. Retain the session across cancelled
        // join callers so uncertain root writes can still be reconciled.
        self.session.lock().await.finish_abandoned().await?;
        match &joined.failure {
            Some(error) => Err(anyhow::anyhow!("Graph root worker failed: {error}")),
            None => Ok(()),
        }
    }
}

impl GraphRootSupervisor {
    pub(crate) async fn start(
        &self,
        manager: RunManager,
        mut request: RunExecutionRequest,
        run_id: String,
        cancellation: CancellationToken,
    ) -> anyhow::Result<()> {
        let mut preparation = PreparationLifetime(Some(cancellation.clone()));
        let owner = request
            .verified_owner
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Graph delegation requires a verified host owner"))?;
        anyhow::ensure!(
            request.user_id.as_deref() == Some(owner.user_id()),
            "Graph owner mismatch"
        );
        let persistence = manager
            .persistence
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Graph delegation requires thread persistence"))?;
        self.reap_finished().await;
        let session_id = request
            .session_id
            .get_or_insert_with(|| uuid::Uuid::new_v4().to_string())
            .clone();
        let (state, _state_observer) = watch::channel(None);
        let owned_root = Arc::new(AsyncMutex::new(None));
        let session = Arc::new(AsyncMutex::new(ActorThreadSession::new(
            owner,
            request.artifact.clone(),
            session_id,
            Arc::new(manager.clone()),
            persistence,
            cancellation.clone(),
            state,
            owned_root,
        )));
        let (prepared, ready) = oneshot::channel();
        let job = {
            let mut jobs = self
                .jobs
                .lock()
                .map_err(|_| anyhow::anyhow!("Graph root registry unavailable"))?;
            anyhow::ensure!(
                !jobs.closed && !cancellation.is_cancelled(),
                "Graph root host is shutting down"
            );
            anyhow::ensure!(
                !jobs.entries.contains_key(&run_id),
                "Graph root already registered"
            );
            let execution_session = Arc::clone(&session);
            let execution_run_id = run_id.clone();
            let worker_cancellation = cancellation.clone();
            let failure_request = request.clone();
            let handle = tokio::spawn(async move {
                let outcome = std::panic::AssertUnwindSafe(async {
                    execution_session
                        .lock()
                        .await
                        .execute_request(request, execution_run_id.clone(), Some(prepared), None)
                        .await
                        .map(|_| ())
                })
                .catch_unwind()
                .await;
                let result = match outcome {
                    Ok(result) => result.map_err(|error| error.to_string()),
                    Err(_) => Err("Graph root worker panicked".to_string()),
                };
                if let Err(error) = &result {
                    worker_cancellation.cancel();
                    tracing::error!(run_id = %execution_run_id, %error, "Graph root retains failed execution");
                    manager
                        .record_graph_root_failure(&failure_request, &execution_run_id)
                        .await;
                }
                result
            });
            // No await between launch and publication of its owned handle.
            let job = Arc::new(Job {
                cancellation,
                session,
                joined: AsyncMutex::new(Joined {
                    handle: Some(handle),
                    failure: None,
                }),
            });
            jobs.entries.insert(run_id, Arc::clone(&job));
            job
        };
        if ready.await.is_err() {
            // A failed start can still own a root write or prepared producer.
            // Joining does not discard its failure receipt from the registry.
            job.join().await?;
            anyhow::bail!("Graph root ended before preparation completed");
        }
        preparation.0 = None;
        Ok(())
    }

    async fn reap_finished(&self) {
        let finished = match self.jobs.lock() {
            Ok(jobs) => jobs
                .entries
                .iter()
                .filter(|(_, job)| job.finished())
                .map(|(id, job)| (id.clone(), Arc::clone(job)))
                .collect::<Vec<_>>(),
            Err(_) => return,
        };
        for (id, job) in finished {
            if job.join().await.is_ok()
                && let Ok(mut jobs) = self.jobs.lock()
                && jobs
                    .entries
                    .get(&id)
                    .is_some_and(|current| Arc::ptr_eq(current, &job))
            {
                jobs.entries.remove(&id);
            }
        }
    }

    pub(crate) async fn shutdown(&self) -> anyhow::Result<()> {
        let jobs = {
            let mut jobs = self
                .jobs
                .lock()
                .map_err(|_| anyhow::anyhow!("Graph root registry unavailable"))?;
            jobs.closed = true;
            jobs.entries
                .iter()
                .map(|(id, job)| (id.clone(), Arc::clone(job)))
                .collect::<Vec<_>>()
        };
        for (_, job) in &jobs {
            job.cancellation.cancel();
        }
        let mut failure = None;
        for (id, job) in jobs {
            match job.join().await {
                Ok(()) => {
                    let mut jobs = self
                        .jobs
                        .lock()
                        .map_err(|_| anyhow::anyhow!("Graph root registry unavailable"))?;
                    if jobs
                        .entries
                        .get(&id)
                        .is_some_and(|current| Arc::ptr_eq(current, &job))
                    {
                        jobs.entries.remove(&id);
                    }
                }
                Err(error) => {
                    failure.get_or_insert(error);
                }
            }
        }
        failure.map_or(Ok(()), Err)
    }
}
