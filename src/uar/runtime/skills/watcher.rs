//! Filesystem watcher for skill hot-reload.
//!
//! Monitors a skills directory for file changes and triggers a full
//! `SkillService::refresh()` only from retained filesystem events.

use std::path::Path;
use std::sync::Arc;

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

use super::service::SkillService;
use crate::uar::runtime::lifecycle::{CloseReport, ResourceScope};

/// Watches a skills directory and owns the event-processing task that refreshes
/// its skill service. The task belongs to `scope`; dropping a caller only asks
/// it to stop, while [`Self::close`] retains and awaits its acknowledgement.
pub struct SkillFileWatcher {
    scope: ResourceScope,
    stop: CancellationToken,
}

/// Drop order is part of the native watcher closure contract. The notify
/// callback may be blocked in `blocking_send` when shutdown starts; closing the
/// receiver first releases that callback before `RecommendedWatcher::drop` can
/// join its native thread.
struct WatcherTaskResources {
    receiver: mpsc::Receiver<Event>,
    watcher: RecommendedWatcher,
}

impl Drop for WatcherTaskResources {
    fn drop(&mut self) {
        self.receiver.close();
    }
}

impl std::fmt::Debug for SkillFileWatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkillFileWatcher").finish_non_exhaustive()
    }
}

impl SkillFileWatcher {
    /// Compatibility constructor for embeddings that do not yet supply a host
    /// scope. New trusted-host construction should use [`Self::new_in_scope`]
    /// and retain the returned watcher through host shutdown.
    pub fn new(skill_service: Arc<SkillService>, skills_dir: &Path) -> anyhow::Result<Self> {
        Self::new_in_scope(
            skill_service,
            skills_dir,
            ResourceScope::standalone("skill-file-watcher", "skill-runtime"),
        )
    }

    /// Register filesystem event processing in an already-owned lifecycle
    /// scope before the watcher becomes available to callers.
    pub fn new_in_scope(
        skill_service: Arc<SkillService>,
        skills_dir: &Path,
        scope: ResourceScope,
    ) -> anyhow::Result<Self> {
        // Cover native watcher construction itself. A host close racing this
        // constructor waits for this lease, and the lease moves into the
        // tracked event task only after the native registration succeeds.
        let construction = scope
            .admit_named("skill-file-watcher-construction")
            .map_err(|error| anyhow::anyhow!("skill file watcher lifecycle admission closed: {error}"))?;
        let (watcher_tx, receiver) = mpsc::channel::<Event>(64);
        let watcher = RecommendedWatcher::new(
            move |result: Result<Event, notify::Error>| match result {
                Ok(event) => {
                    if let Err(error) = watcher_tx.blocking_send(event) {
                        warn!(%error, "Skill watcher event receiver is unavailable");
                    }
                }
                Err(error) => {
                    error!(%error, "Skill watcher reported an error");
                }
            },
            Config::default(),
        )?;
        // Keep the receiver and native handle in one drop-ordered guard from
        // the first point at which the callback can run. If registration
        // fails, or the later task admission is rejected, this guard closes
        // the receiver before dropping the native watcher and unblocks an
        // in-flight callback `blocking_send`.
        let mut resources = WatcherTaskResources { receiver, watcher };
        resources.watcher.watch(skills_dir, RecursiveMode::Recursive)?;
        let stop = CancellationToken::new();
        let scope_cancellation = scope.cancellation_token();
        let task_stop = stop.clone();
        scope
            .spawn("skill-file-events", async move {
                // The task owns the OS watcher. A host-driven scope close
                // cancels this task and waits for it, so the native watch
                // registration is released before the close report exists.
                Self::event_loop(
                    skill_service,
                    task_stop,
                    scope_cancellation,
                    resources,
                )
                .await;
                // The constructor lease moves with the task so a scope close
                // cannot report release while the native registration remains
                // owned by this admitted operation.
                construction.complete();
                Ok(())
            })
            .map_err(|error| anyhow::anyhow!("skill file watcher lifecycle admission closed: {error}"))?;
        info!(path = %skills_dir.display(), "Skill file watcher started");
        Ok(Self {
            scope,
            stop,
        })
    }

    /// Request watcher stop and retain the joined task report. A refresh that
    /// was already admitted is allowed to return its actual result first.
    pub async fn close(self) -> CloseReport {
        self.stop.cancel();
        self.scope.close().await
    }

    async fn event_loop(
        skill_service: Arc<SkillService>,
        stop: CancellationToken,
        scope_cancellation: CancellationToken,
        mut resources: WatcherTaskResources,
    ) {
        loop {
            let event = tokio::select! {
                biased;
                _ = stop.cancelled() => break,
                _ = scope_cancellation.cancelled() => break,
                event = resources.receiver.recv() => event,
            };
            let Some(event) = event else {
                info!("Skill watcher channel closed");
                break;
            };
            let mut relevant = Self::is_relevant(&event);
            // Coalesce the already-delivered burst only. No duration is used to
            // decide freshness or correctness; a subsequent event triggers a
            // subsequent authoritative refresh.
            while let Ok(event) = resources.receiver.try_recv() {
                relevant |= Self::is_relevant(&event);
            }
            if !relevant {
                continue;
            }
            debug!("Refreshing skills after committed filesystem events");
            match skill_service.refresh().await {
                Ok(skills) => {
                    info!(count = skills.len(), "Skills refreshed after filesystem event");
                }
                Err(error) => {
                    error!(%error, "Skill refresh failed after filesystem event");
                }
            }
        }
    }

    fn is_relevant(event: &Event) -> bool {
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => event
                .paths
                .iter()
                .any(|path| {
                    matches!(
                        path.extension().and_then(|extension| extension.to_str()),
                        Some("yaml" | "yml" | "md" | "toml" | "json")
                    )
                }),
            _ => false,
        }
    }
}

impl Drop for SkillFileWatcher {
    fn drop(&mut self) {
        self.stop.cancel();
    }
}

/// Start watching the given skills directory with compatibility standalone
/// ownership. Trusted hosts should call [`SkillFileWatcher::new_in_scope`].
pub async fn start_skill_watcher(
    skill_service: Arc<SkillService>,
    skills_dir: &str,
) -> anyhow::Result<SkillFileWatcher> {
    let path = Path::new(skills_dir);
    if !path.exists() {
        anyhow::bail!("Skills directory does not exist: {}", path.display());
    }
    SkillFileWatcher::new(skill_service, path)
}
