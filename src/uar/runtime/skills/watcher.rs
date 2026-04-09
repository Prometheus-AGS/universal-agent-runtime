//! Filesystem watcher for skill hot-reload.
//!
//! Monitors a skills directory for file changes (create, modify, remove)
//! and triggers a full skill refresh via `SkillService::refresh()`.

use super::service::SkillService;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Watches a skills directory for filesystem changes and triggers reloads.
pub struct SkillFileWatcher {
    /// The underlying filesystem watcher handle.
    /// Kept alive so the watcher thread continues running.
    _watcher: RecommendedWatcher,
}

impl std::fmt::Debug for SkillFileWatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkillFileWatcher").finish()
    }
}

impl SkillFileWatcher {
    /// Create a new watcher that monitors `skills_dir` and reloads skills
    /// through the given `SkillService` on any relevant file change.
    ///
    /// Returns the watcher handle. The watcher stops when this handle is dropped.
    pub fn new(skill_service: Arc<SkillService>, skills_dir: &Path) -> anyhow::Result<Self> {
        let (tx, rx) = mpsc::channel::<Event>(64);

        // Create the notify watcher with a channel-based event handler.
        // `notify` v7's `RecommendedWatcher` uses a synchronous callback,
        // so we bridge into async via an mpsc channel.
        let watcher_tx = tx.clone();
        let mut watcher = RecommendedWatcher::new(
            move |result: Result<Event, notify::Error>| match result {
                Ok(event) => {
                    if let Err(e) = watcher_tx.blocking_send(event) {
                        warn!("Skill watcher channel send failed: {e}");
                    }
                }
                Err(e) => {
                    error!("Skill watcher error: {e}");
                }
            },
            Config::default(),
        )?;

        watcher.watch(skills_dir, RecursiveMode::Recursive)?;
        info!("Skill file watcher started on {}", skills_dir.display());

        // Spawn background task to process events with debouncing.
        tokio::spawn(Self::event_loop(skill_service, rx));

        Ok(Self { _watcher: watcher })
    }

    /// Background event loop that debounces filesystem events and triggers
    /// a skill refresh when relevant changes are detected.
    async fn event_loop(skill_service: Arc<SkillService>, mut rx: mpsc::Receiver<Event>) {
        // Debounce interval — coalesce rapid successive writes into one refresh.
        let debounce = Duration::from_millis(500);

        loop {
            // Wait for the first event.
            let Some(event) = rx.recv().await else {
                info!("Skill watcher channel closed, stopping event loop");
                return;
            };

            if !Self::is_relevant(&event) {
                continue;
            }

            debug!("Skill file change detected: {:?}", event.kind);

            // Drain any additional events that arrive within the debounce window.
            tokio::time::sleep(debounce).await;
            while rx.try_recv().is_ok() {
                // Discard; we will do a single refresh for the whole batch.
            }

            info!("Refreshing skills after filesystem change");
            match skill_service.refresh().await {
                Ok(skills) => {
                    info!("Skills refreshed successfully: {} skills loaded", skills.len());
                }
                Err(e) => {
                    error!("Failed to refresh skills after file change: {e:?}");
                }
            }
        }
    }

    /// Returns `true` if the event is a file change we care about
    /// (create, modify, or remove of a likely skill file).
    fn is_relevant(event: &Event) -> bool {
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                // Only react to YAML/MD files (skill definitions).
                event.paths.iter().any(|p| {
                    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
                    matches!(ext, "yaml" | "yml" | "md" | "toml" | "json")
                })
            }
            _ => false,
        }
    }
}

/// Start watching the given skills directory for changes.
///
/// Returns a `SkillFileWatcher` handle. The watcher runs in a background
/// tokio task and will stop when the handle is dropped.
pub async fn start_skill_watcher(
    skill_service: Arc<SkillService>,
    skills_dir: &str,
) -> anyhow::Result<SkillFileWatcher> {
    let path = Path::new(skills_dir);
    if !path.exists() {
        anyhow::bail!(
            "Skills directory does not exist: {}",
            path.display()
        );
    }

    SkillFileWatcher::new(skill_service, path)
}
