use crate::config::{AppConfig, Cli};
use arc_swap::ArcSwap;
use notify::Watcher;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info};

/// Manages the live, reloadable application configuration.
///
/// The authoritative configuration is held in an `ArcSwap` so that handlers can
/// obtain a consistent `Arc<AppConfig>` for the duration of a request without
/// blocking writers. On `reload()` (or when the watched config file changes) a
/// new `AppConfig` is built and atomically swapped in; existing `Arc<AppConfig>`
/// instances remain valid and are dropped when no reader holds them.
#[derive(Debug)]
pub struct ConfigManager {
    /// Atomic, lock-free storage of the current configuration.
    current: ArcSwap<AppConfig>,
    /// The CLI arguments used to construct the original configuration. Reload
    /// re-applies these overrides so the operator's intent is preserved.
    cli: Cli,
    /// Resolved config file path that is being watched, if any.
    watched_path: Option<PathBuf>,
    /// When true, a reload that produces a configuration different from the
    /// initial load is treated as an error. This makes the effective config
    /// immutable after startup, which is useful for audited production
    /// deployments where any drift must be explicit.
    strict: AtomicBool,
    /// Initial configuration snapshot, used for strict-mode comparisons.
    initial: Arc<AppConfig>,
    /// Handle for the background watcher task. Held so the task is cancelled
    /// when the manager is dropped.
    watcher: Mutex<Option<tokio::task::JoinHandle<()>>>,
    /// Shutdown signal for the watcher task.
    watcher_shutdown: Option<tokio::sync::oneshot::Sender<()>>,
}

impl ConfigManager {
    /// Build the initial configuration from `cli` and start watching the
    /// loaded config file for changes.
    pub async fn load(cli: Cli) -> Result<Arc<Self>, ConfigError> {
        let config = AppConfig::load_with_cli(cli.clone()).map_err(ConfigError::Config)?;
        #[cfg(feature = "vault")]
        let mut config = config;
        #[cfg(feature = "vault")]
        crate::config::vault::resolve(&mut config).await?;
        config.security.validate().map_err(ConfigError::Config)?;
        let watched_path = resolve_watched_path(&cli);
        let initial = Arc::new(config);
        let current = ArcSwap::new(Arc::clone(&initial));

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

        let manager = Arc::new(Self {
            current,
            cli,
            watched_path,
            strict: AtomicBool::new(false),
            initial,
            watcher: Mutex::new(None),
            watcher_shutdown: Some(shutdown_tx),
        });

        let watcher_handle = manager.watch(shutdown_rx);
        *manager.watcher.lock().expect("watcher lock poisoned") = watcher_handle;

        Ok(manager)
    }

    /// Build a `ConfigManager` without starting a file watcher. Used in tests
    /// and in environments where the config file is not on a local filesystem.
    pub async fn load_without_watcher(cli: Cli) -> Result<Arc<Self>, ConfigError> {
        let config = AppConfig::load_with_cli(cli.clone()).map_err(ConfigError::Config)?;
        #[cfg(feature = "vault")]
        let mut config = config;
        #[cfg(feature = "vault")]
        crate::config::vault::resolve(&mut config).await?;
        config.security.validate().map_err(ConfigError::Config)?;
        let initial = Arc::new(config);
        Ok(Arc::new(Self {
            current: ArcSwap::new(Arc::clone(&initial)),
            cli,
            watched_path: None,
            strict: AtomicBool::new(false),
            initial,
            watcher: Mutex::new(None),
            watcher_shutdown: None,
        }))
    }

    /// Enable strict mode before the server starts accepting requests.
    pub fn set_strict(&self, strict: bool) {
        self.strict.store(strict, Ordering::SeqCst);
    }

    /// Return the current configuration snapshot.
    #[inline]
    pub fn current(&self) -> Arc<AppConfig> {
        self.current.load_full()
    }

    /// Rebuild the configuration from the original sources and atomically swap
    /// it in. If strict mode is enabled and the new config differs from the
    /// initial snapshot, the reload is rejected.
    pub async fn reload(&self) -> Result<(), ConfigError> {
        #[cfg(feature = "vault")]
        let mut new_config =
            AppConfig::load_with_cli(self.cli.clone()).map_err(ConfigError::Config)?;
        #[cfg(not(feature = "vault"))]
        let new_config = AppConfig::load_with_cli(self.cli.clone()).map_err(ConfigError::Config)?;
        #[cfg(feature = "vault")]
        crate::config::vault::resolve(&mut new_config).await?;
        new_config
            .security
            .validate()
            .map_err(ConfigError::Config)?;
        let new_arc = Arc::new(new_config);

        if self.strict.load(Ordering::SeqCst) && !configs_equal(&self.initial, &new_arc) {
            return Err(ConfigError::StrictConflict(
                "reloaded configuration differs from the initial strict snapshot".into(),
            ));
        }

        self.current.store(Arc::clone(&new_arc));
        info!(name: "config.reloaded", "Configuration reloaded successfully");
        Ok(())
    }

    /// Path currently watched for changes, if any.
    pub fn watched_path(&self) -> Option<&PathBuf> {
        self.watched_path.as_ref()
    }

    /// Spawn a background watcher that calls `reload()` when the config file
    /// changes. Events are debounced to avoid reloading on partial writes.
    fn watch(
        self: &Arc<Self>,
        shutdown: tokio::sync::oneshot::Receiver<()>,
    ) -> Option<tokio::task::JoinHandle<()>> {
        let path = self.watched_path.clone()?;
        let manager = Arc::clone(self);
        let (tx, mut rx) = mpsc::channel::<notify::Event>(16);

        let mut watcher = match notify::recommended_watcher(move |event| {
            if let Ok(event) = event {
                let _ = tx.try_send(event);
            }
        }) {
            Ok(w) => w,
            Err(e) => {
                error!(name: "config.watcher.failed", error = %e, "Failed to start config file watcher");
                return None;
            }
        };

        if let Err(e) = watcher.watch(&path, notify::RecursiveMode::NonRecursive) {
            error!(name: "config.watcher.watch_failed", path = %path.display(), error = %e, "Failed to watch config file");
            return None;
        }

        let mut shutdown_fut = Box::pin(shutdown);
        let handle = tokio::spawn(async move {
            let mut debounce = tokio::time::interval(Duration::from_millis(500));
            debounce.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            let mut pending = false;

            loop {
                tokio::select! {
                    _ = &mut shutdown_fut => break,
                    _ = debounce.tick() => {
                        if pending {
                            pending = false;
                            if let Err(e) = manager.reload().await {
                                error!(name: "config.reload.failed", error = %e, "Automatic config reload failed");
                            }
                        }
                    }
                    Some(event) = rx.recv() => {
                        // We only care about content modifications or new files.
                        if matches!(
                            event.kind,
                            notify::EventKind::Modify(_) | notify::EventKind::Create(_)
                        ) {
                            pending = true;
                            debounce.reset_immediately();
                        }
                    }
                }
            }
        });

        Some(handle)
    }
}

impl Drop for ConfigManager {
    fn drop(&mut self) {
        if let Some(tx) = self.watcher_shutdown.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.watcher.lock().expect("watcher lock poisoned").take() {
            handle.abort();
        }
    }
}

/// Errors that can occur while managing or reloading configuration.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("configuration error: {0}")]
    Config(#[source] config::ConfigError),
    #[error("strict config conflict: {0}")]
    StrictConflict(String),
    #[error("Vault error: {0}")]
    Vault(String),
}

impl From<config::ConfigError> for ConfigError {
    fn from(e: config::ConfigError) -> Self {
        Self::Config(e)
    }
}

/// Determine which file path should be watched for changes.
fn resolve_watched_path(cli: &Cli) -> Option<PathBuf> {
    if let Some(path) = &cli.config {
        return Some(PathBuf::from(path));
    }
    let cwd = PathBuf::from("config.yaml");
    if cwd.exists() {
        return Some(cwd);
    }
    if let Some(mut home) = dirs::home_dir() {
        home.push(".uar");
        home.push("config.yaml");
        if home.exists() {
            return Some(home);
        }
    }
    None
}

/// Compare two configurations for strict-mode equality. This is intentionally
/// a shallow JSON-based comparison: it catches meaningful scalar, vector, and
/// nested object changes without requiring `AppConfig` to implement `Eq`.
fn configs_equal(a: &AppConfig, b: &AppConfig) -> bool {
    // `AppConfig` derives `Debug`, so the formatted representation is stable for
    // the same struct definition and is sufficient for the strict-mode guard.
    format!("{a:?}") == format!("{b:?}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser as _;
    use std::io::Write;

    #[tokio::test]
    async fn config_manager_stores_and_returns_current() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.yaml");
        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(
            file,
            "security:\n  jwt_required: false\n  settings_mutation_auth_required: false\nserver:\n  port: 1906"
        )
        .unwrap();
        let cli = Cli::parse_from(["uar", "--config", path.to_str().unwrap()]);
        let manager = ConfigManager::load_without_watcher(cli).await.unwrap();
        let current = manager.current();
        assert_eq!(current.server.port, 1906);
    }

    #[tokio::test]
    async fn config_manager_reload_swaps_current() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.yaml");
        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(
            file,
            "server:\n  port: 1906\nsecurity:\n  jwt_required: false\n  settings_mutation_auth_required: false"
        )
        .unwrap();

        let cli = Cli::parse_from(["uar", "--config", path.to_str().unwrap()]);
        let manager = ConfigManager::load_without_watcher(cli).await.unwrap();
        assert_eq!(manager.current().server.port, 1906);

        // Mutate the file and reload.
        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(
            file,
            "server:\n  port: 9090\nsecurity:\n  jwt_required: false\n  settings_mutation_auth_required: false"
        )
        .unwrap();
        manager.reload().await.unwrap();
        assert_eq!(manager.current().server.port, 9090);
    }

    #[tokio::test]
    async fn strict_mode_rejects_changed_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.yaml");
        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(
            file,
            "server:\n  port: 1906\nsecurity:\n  jwt_required: false\n  settings_mutation_auth_required: false"
        )
        .unwrap();

        let cli = Cli::parse_from(["uar", "--config", path.to_str().unwrap()]);
        let manager = ConfigManager::load_without_watcher(cli).await.unwrap();
        manager.set_strict(true);

        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(
            file,
            "server:\n  port: 9090\nsecurity:\n  jwt_required: false\n  settings_mutation_auth_required: false"
        )
        .unwrap();
        let err = manager.reload().await.unwrap_err();
        assert!(matches!(err, ConfigError::StrictConflict(_)));
    }
}
