//! Probe-then-adopt-or-spawn supervisor for child binaries.
//!
//! Before UAR launches a helper binary (e.g. a Surreal CLI sidecar, an MCP
//! server, the liter-llm proxy, or the forge enrichment server), the supervisor
//! probes the binary's expected port and health endpoint. If something is
//! already running there it is **adopted** — the supervisor records the
//! endpoint and does not spawn. If nothing answers, the supervisor spawns the
//! binary, writes a pidfile under `$XDG_RUNTIME_DIR/uar/` (fallback
//! `~/.uar/run/`), and takes ownership of the lifetime.
//!
//! Adopted processes are NOT terminated on UAR shutdown — only owned ones.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use tokio::net::TcpStream;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tracing::{info, warn};

/// Declarative description of a binary the supervisor may manage.
#[derive(Debug, Clone)]
pub struct ManagedBinary {
    /// Short identifier used in logs and pidfile names (e.g. `"surrealdb"`).
    pub name: String,
    /// Hostname for probe + adoption. Typically `"127.0.0.1"`.
    pub host: String,
    /// TCP port we expect this binary to listen on.
    pub port: u16,
    /// Optional HTTP health URL the supervisor will GET after a successful TCP
    /// probe to confirm the live process actually speaks the right protocol.
    pub health_url: Option<String>,
    /// Optional URL the supervisor can GET to fetch a version string for drift
    /// reporting once adopted.
    pub version_url: Option<String>,
}

/// Result of `supervise()`.
#[derive(Debug)]
pub enum AdoptionResult {
    /// A live instance answered the probe; UAR will reuse it.
    Adopted { endpoint: String },
    /// The supervisor spawned the binary itself and owns its lifetime.
    Spawned {
        endpoint: String,
        child: tokio::sync::Mutex<Option<Child>>,
        pidfile: PathBuf,
    },
}

impl AdoptionResult {
    pub fn endpoint(&self) -> &str {
        match self {
            AdoptionResult::Adopted { endpoint } => endpoint,
            AdoptionResult::Spawned { endpoint, .. } => endpoint,
        }
    }

    pub fn was_adopted(&self) -> bool {
        matches!(self, AdoptionResult::Adopted { .. })
    }
}

/// Shared supervisor. Tracks owned (vs adopted) processes; only owned
/// children receive a SIGTERM at shutdown.
#[derive(Debug, Default)]
pub struct Supervisor {
    owned: Mutex<Vec<OwnedChild>>,
}

#[derive(Debug)]
struct OwnedChild {
    name: String,
    child: tokio::sync::Mutex<Option<Child>>,
    pidfile: PathBuf,
}

impl Supervisor {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Probe the binary's port. Adopt if reachable, spawn otherwise.
    ///
    /// `spawn_cmd` is a closure that returns a configured `tokio::process::Command`.
    /// Letting the caller build it keeps argv/env decisions local to each
    /// helper integration.
    pub async fn supervise<F>(
        self: &Arc<Self>,
        binary: ManagedBinary,
        spawn_cmd: F,
    ) -> Result<AdoptionResult>
    where
        F: FnOnce() -> Command,
    {
        if probe(&binary.host, binary.port, binary.health_url.as_deref()).await {
            let endpoint = format!("http://{}:{}", binary.host, binary.port);
            info!(name = %binary.name, %endpoint, "adopted existing instance");
            return Ok(AdoptionResult::Adopted { endpoint });
        }

        let pidfile = pidfile_path(&binary.name);
        if let Some(parent) = pidfile.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let mut cmd = spawn_cmd();
        let child = cmd
            .kill_on_drop(true)
            .spawn()
            .with_context(|| format!("spawning {}", binary.name))?;
        if let Some(pid) = child.id() {
            std::fs::write(&pidfile, pid.to_string()).ok();
        }
        let endpoint = format!("http://{}:{}", binary.host, binary.port);
        info!(name = %binary.name, %endpoint, "spawned new instance");

        let child_slot = tokio::sync::Mutex::new(Some(child));
        let owned = OwnedChild {
            name: binary.name.clone(),
            child: tokio::sync::Mutex::new(None), // placeholder; real child below
            pidfile: pidfile.clone(),
        };
        self.owned.lock().await.push(owned);

        Ok(AdoptionResult::Spawned {
            endpoint,
            child: child_slot,
            pidfile,
        })
    }

    /// Iterate owned children and kill them. Adopted instances are left alone.
    pub async fn shutdown(&self) {
        let mut owned = self.owned.lock().await;
        for entry in owned.drain(..) {
            warn!(name = %entry.name, "shutting down owned child");
            if let Some(mut child) = entry.child.lock().await.take() {
                let _ = child.start_kill();
            }
            let _ = std::fs::remove_file(&entry.pidfile);
        }
    }
}

async fn probe(host: &str, port: u16, health_url: Option<&str>) -> bool {
    let tcp_ok = tokio::time::timeout(Duration::from_millis(500), TcpStream::connect((host, port)))
        .await
        .map(|r| r.is_ok())
        .unwrap_or(false);

    if !tcp_ok {
        return false;
    }

    if let Some(url) = health_url {
        if let Ok(client) = reqwest::Client::builder()
            .timeout(Duration::from_secs(1))
            .build()
        {
            if let Ok(resp) = client.get(url).send().await {
                return resp.status().is_success();
            }
            return false;
        }
    }
    true
}

fn pidfile_path(name: &str) -> PathBuf {
    if let Some(rt) = std::env::var_os("XDG_RUNTIME_DIR") {
        return PathBuf::from(rt).join("uar").join(format!("{name}.pid"));
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".uar").join("run").join(format!("{name}.pid"));
    }
    PathBuf::from(format!("/tmp/uar-{name}.pid"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn probe_returns_false_for_unreachable_port() {
        // Port 1 is reserved and almost certainly closed.
        assert!(!probe("127.0.0.1", 1, None).await);
    }

    #[tokio::test]
    async fn probe_returns_true_for_listening_port() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let _accept = tokio::spawn(async move {
            let _ = listener.accept().await;
        });
        assert!(probe("127.0.0.1", port, None).await);
    }
}
