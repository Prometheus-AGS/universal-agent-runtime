//! Message types for inter-agent communication.
//!
//! Defines the typed message envelope that agent actors exchange,
//! plus reply types for request-response patterns.

use serde::{Deserialize, Serialize};

/// Messages that can be sent to an [`AgentActor`](super::agent_actor::AgentActor).
#[derive(Debug)]
pub enum AgentMessage {
    /// A user or system prompt to process.
    UserPrompt {
        /// The text prompt to handle.
        content: String,
        /// Optional reply channel for the response.
        reply: Option<tokio::sync::oneshot::Sender<AgentReply>>,
    },

    /// Request collaboration from this actor.
    /// Another agent is asking this actor to help with a sub-task.
    Collaborate {
        /// ID of the requesting agent.
        from_agent_id: String,
        /// The task description or query.
        task: String,
        /// Reply channel for the collaboration result.
        reply: tokio::sync::oneshot::Sender<AgentReply>,
    },

    /// A tool execution result being returned to the actor.
    ToolResult {
        /// Tool call ID.
        tool_call_id: String,
        /// Serialized result content.
        content: serde_json::Value,
        /// Whether the tool call succeeded.
        success: bool,
    },

    /// Administrative: request the actor to stop gracefully.
    Shutdown,
}

/// Reply sent back through oneshot channels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentReply {
    /// The agent's response content.
    pub content: String,
    /// Whether the operation succeeded.
    pub success: bool,
    /// Optional metadata about the response.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Summary information about a running actor (for API responses).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorInfo {
    /// Actor name / ID.
    pub id: String,
    /// Agent artifact ID this actor is running.
    pub agent_id: String,
    /// Current status.
    pub status: ActorStatus,
}

/// Actor lifecycle status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActorStatus {
    /// Actor is starting up.
    Starting,
    /// Actor is running and ready to process messages.
    Running,
    /// Actor is shutting down.
    Stopping,
    /// Actor has stopped.
    Stopped,
}
