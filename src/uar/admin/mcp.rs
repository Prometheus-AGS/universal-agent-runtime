//! Transport-free MCP server administration.
//!
//! WHY THIS EXISTS SEPARATELY FROM `uar::api::mcp_admin`
//!
//! The HTTP handlers own `AppState` and axum extractors, so an EMBEDDED
//! container (mobile, macOS) could not reach any of this — it had no listener
//! to call. That is why the embedded control plane reported "MCP server
//! administration is not available on the embedded runtime".
//!
//! The storage and hydration logic never actually needed a transport: it needs
//! a `SettingsManager` and an `McpRegistry`. Pulling it here lets the SDK
//! `Runtime` call it in-process while the HTTP layer becomes a thin adapter
//! over the SAME code, so embedded and remote cannot drift apart.
//!
//! CONFIGURATION LIVES IN THE DATABASE, NOT IN FILES
//!
//! Config files SEED the store on first boot (`hydrate` persists the
//! file-derived registry when the store is empty) and the store is the source
//! of truth from then on. That is what makes a runtime API change take effect
//! without a restart, a file write, or a polling loop — the same contract on
//! an embedded device and a remote server.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::mcp::config::McpServerEntry;
use crate::mcp::registry::McpRegistry;
use crate::uar::settings::manager::SettingsManager;

/// Settings key holding the MCP server map. Shared with `api::mcp_admin` so
/// both surfaces read and write the same rows.
pub const SETTINGS_KEY: &str = "mcp.servers";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMcpServer {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub entry: McpServerEntry,
}

fn default_true() -> bool {
    true
}

/// What happened to a live MCP connection as a result of a write.
///
/// A save must not silently tear down a healthy connection: dropping and
/// reconnecting an already-connected server mid-session can abort in-flight
/// tool calls and force auth re-negotiation. So an edit to a CONNECTED server
/// is stored durably and reported as deferred rather than applied behind the
/// caller's back — the caller decides whether to reconnect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ApplyOutcome {
    /// The registry now matches the store; nothing was connected to disturb.
    Applied,
    /// Stored, but a live connection was left running. `reason` is safe to show.
    Deferred { reason: String },
    /// The server was removed from both the store and the registry.
    Removed,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveResult {
    pub name: String,
    pub outcome: ApplyOutcome,
}

/// Read the stored server map. Falls back to whatever the registry was
/// constructed with (i.e. the config file) so a store that has never been
/// seeded still reports the real configuration rather than nothing.
pub async fn list(
    registry: &Arc<McpRegistry>,
    manager: Option<&Arc<SettingsManager>>,
) -> HashMap<String, StoredMcpServer> {
    if let Some(manager) = manager
        && let Some(value) = manager.get_value(SETTINGS_KEY).await
        && let Ok(servers) = serde_json::from_value::<HashMap<String, StoredMcpServer>>(value)
    {
        return servers;
    }
    registry
        .server_entries()
        .into_iter()
        .map(|(name, entry)| {
            (
                name,
                StoredMcpServer {
                    enabled: true,
                    entry,
                },
            )
        })
        .collect()
}

async fn write(
    manager: &Arc<SettingsManager>,
    servers: &HashMap<String, StoredMcpServer>,
) -> Result<(), String> {
    manager
        .set_value(
            SETTINGS_KEY,
            serde_json::to_value(servers).map_err(|error| error.to_string())?,
        )
        .await
        .map_err(|error| error.to_string())
}

/// Persist a server, then apply it to the live registry when that is safe.
///
/// Safety here is the B-mode contract: a server that is NOT currently
/// connected is applied immediately (adds and removals take effect live), while
/// an edit to a CONNECTED server is stored and deferred to the next session.
pub async fn save(
    registry: &Arc<McpRegistry>,
    manager: Option<&Arc<SettingsManager>>,
    name: String,
    server: StoredMcpServer,
) -> Result<SaveResult, String> {
    server
        .entry
        .validate_sandbox_policy(&name)
        .map_err(|error| error.to_string())?;
    let manager = manager.ok_or_else(|| "UAR settings storage is unavailable".to_string())?;

    let mut servers = list(registry, Some(manager)).await;
    let connected = registry.server_names().iter().any(|item| item == &name);
    servers.insert(name.clone(), server.clone());
    write(manager, &servers).await?;

    if connected {
        return Ok(SaveResult {
            name,
            outcome: ApplyOutcome::Deferred {
                reason: "saved; the running connection keeps its current settings until the \
                         next session, so in-flight tool calls are not interrupted"
                    .to_string(),
            },
        });
    }

    if server.enabled {
        registry
            .upsert_server(name.clone(), server.entry)
            .await
            .map_err(|error| error.to_string())?;
    }
    Ok(SaveResult {
        name,
        outcome: ApplyOutcome::Applied,
    })
}

/// Remove a server from the store and from the live registry.
///
/// Removal applies immediately even when connected: the caller's intent is that
/// the server stop being used, and leaving it running would contradict that.
pub async fn delete(
    registry: &Arc<McpRegistry>,
    manager: Option<&Arc<SettingsManager>>,
    name: &str,
) -> Result<SaveResult, String> {
    let manager = manager.ok_or_else(|| "UAR settings storage is unavailable".to_string())?;
    let mut servers = list(registry, Some(manager)).await;
    servers.remove(name);
    write(manager, &servers).await?;
    registry.remove_server(name);
    Ok(SaveResult {
        name: name.to_string(),
        outcome: ApplyOutcome::Removed,
    })
}

/// Seed the store from the file-derived registry on first boot, then make the
/// registry match the store.
///
/// Called once during startup, where tearing the registry down is safe because
/// nothing is connected yet. This is the ONLY path that drops every connection;
/// `save` deliberately does not (see `ApplyOutcome::Deferred`).
pub async fn hydrate(
    registry: &Arc<McpRegistry>,
    manager: &Arc<SettingsManager>,
) -> anyhow::Result<()> {
    let current = registry
        .server_entries()
        .into_iter()
        .map(|(name, entry)| {
            (
                name,
                StoredMcpServer {
                    enabled: true,
                    entry,
                },
            )
        })
        .collect::<HashMap<_, _>>();
    let stored = manager
        .get_value(SETTINGS_KEY)
        .await
        .and_then(|value| serde_json::from_value::<HashMap<String, StoredMcpServer>>(value).ok())
        .unwrap_or_default();

    // The file→database seed: an empty store adopts the file configuration
    // once, and every later read comes from the database.
    let seed = stored.is_empty() && !current.is_empty();
    let effective = if seed { current } else { stored };
    for (name, server) in &effective {
        server.entry.validate_sandbox_policy(name)?;
    }
    if seed {
        manager
            .set_value(SETTINGS_KEY, serde_json::to_value(&effective)?)
            .await?;
    }

    for name in registry.server_names() {
        registry.remove_server(&name);
    }
    for (name, server) in effective {
        if server.enabled
            && let Err(error) = registry.upsert_server(name.clone(), server.entry).await
        {
            tracing::warn!(server = %name, %error, "persisted MCP server could not reconnect");
        }
    }
    Ok(())
}
