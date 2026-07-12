//! Realtime change-notification spine.
//!
//! Provides a process-internal broadcast bus fed by SurrealDB live queries
//! (`db.select("table").live()`), re-exposing change events to the SPA as
//! Server-Sent Events at `/api/live/{topic}`. Any view that holds a subscribed
//! entity will re-render on remote mutations, eliminating stale data across
//! toolbars, dropdowns, headers, and list pages.

#[cfg(feature = "surreal-backend")]
pub mod surreal_bus;

#[cfg(feature = "postgres-backend")]
pub mod postgres_bus;

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Backend-neutral realtime change bus.
///
/// The SSE handler (`live.rs`) depends only on this trait, so the concrete
/// source — `SurrealDB` live queries ([`surreal_bus::LiveQueryBus`]) or Postgres
/// `LISTEN/NOTIFY` ([`postgres_bus::PostgresNotifyBus`]) — can be swapped at
/// startup based on the configured persistence backend.
pub trait RealtimeBus: Send + Sync + std::fmt::Debug {
    /// Subscribe to a topic's broadcast stream. `None` when the topic is not
    /// enrolled on this bus.
    fn subscribe(&self, topic: EntityTopic) -> Option<tokio::sync::broadcast::Receiver<LiveEvent>>;

    /// Number of active subscribers on a topic (for observability).
    fn subscriber_count(&self, topic: EntityTopic) -> usize;
}

/// Topics enrolled with the bus at startup. Each topic backs one Surreal
/// `.select().live()` stream and one broadcast channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityTopic {
    KnowledgeBases,
    KnowledgeDocuments,
    Agents,
    Providers,
    Models,
    Skills,
    Settings,
    /// Chat threads — aliases the existing `sessions` table.
    Threads,
    /// Memory items stored by surreal-memory.
    Memory,
    /// Compiler sessions (ephemeral; table may not exist on first run).
    CompilerSessions,
}

impl EntityTopic {
    /// All topics enrolled at startup.
    pub const ALL: &'static [EntityTopic] = &[
        EntityTopic::KnowledgeBases,
        EntityTopic::KnowledgeDocuments,
        EntityTopic::Agents,
        EntityTopic::Providers,
        EntityTopic::Models,
        EntityTopic::Skills,
        EntityTopic::Settings,
        EntityTopic::Threads,
        EntityTopic::Memory,
        EntityTopic::CompilerSessions,
    ];

    /// SurrealDB table name backing this topic.
    pub fn table(self) -> &'static str {
        match self {
            EntityTopic::KnowledgeBases => "knowledge_bases",
            EntityTopic::KnowledgeDocuments => "knowledge_documents",
            EntityTopic::Agents => "agents",
            EntityTopic::Providers => "providers",
            EntityTopic::Models => "models",
            EntityTopic::Skills => "skills",
            EntityTopic::Settings => "settings",
            // `threads` is a frontend-friendly alias for the persisted `sessions` table.
            EntityTopic::Threads => "sessions",
            EntityTopic::Memory => "memory",
            EntityTopic::CompilerSessions => "compiler_sessions",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            EntityTopic::KnowledgeBases => "knowledge_bases",
            EntityTopic::KnowledgeDocuments => "knowledge_documents",
            EntityTopic::Agents => "agents",
            EntityTopic::Providers => "providers",
            EntityTopic::Models => "models",
            EntityTopic::Skills => "skills",
            EntityTopic::Settings => "settings",
            EntityTopic::Threads => "threads",
            EntityTopic::Memory => "memory",
            EntityTopic::CompilerSessions => "compiler_sessions",
        }
    }
}

impl fmt::Display for EntityTopic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for EntityTopic {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "knowledge_bases" | "knowledge_base" => Ok(EntityTopic::KnowledgeBases),
            "knowledge_documents" | "knowledge_document" => Ok(EntityTopic::KnowledgeDocuments),
            "agents" | "agent" => Ok(EntityTopic::Agents),
            "providers" | "provider" => Ok(EntityTopic::Providers),
            "models" | "model" => Ok(EntityTopic::Models),
            "skills" | "skill" => Ok(EntityTopic::Skills),
            "settings" | "setting" => Ok(EntityTopic::Settings),
            "threads" | "thread" | "sessions" | "session" => Ok(EntityTopic::Threads),
            "memory" | "memories" => Ok(EntityTopic::Memory),
            "compiler_sessions" | "compiler_session" => Ok(EntityTopic::CompilerSessions),
            _ => Err(()),
        }
    }
}

/// Action carried by a `LiveEvent`. Mirrors `surrealdb::Action`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LiveAction {
    Create,
    Update,
    Delete,
}

/// A single change event delivered by the bus.
#[derive(Debug, Clone, Serialize)]
pub struct LiveEvent {
    pub action: LiveAction,
    pub topic: EntityTopic,
    pub id: String,
    pub data: serde_json::Value,
}
