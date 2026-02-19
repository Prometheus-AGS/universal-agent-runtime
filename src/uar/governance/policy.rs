//! Domain types for UAR Cedar policy evaluation.
//!
//! Defines the entity types (principal, action, resource) and helper
//! constructors for building Cedar [`Request`]s from UAR runtime context.

use cedar_policy::{Context, EntityId, EntityTypeName, EntityUid, Request};

// ---------------------------------------------------------------------------
// Entity type names (lazy, cached)
// ---------------------------------------------------------------------------

/// Get the Cedar entity type for an agent principal.
fn agent_type() -> EntityTypeName {
    "Agent".parse().expect("valid entity type")
}

/// Get the Cedar entity type for an action.
fn action_type() -> EntityTypeName {
    "Action".parse().expect("valid entity type")
}

/// Get the Cedar entity type for a tool resource.
fn tool_type() -> EntityTypeName {
    "Tool".parse().expect("valid entity type")
}

// ---------------------------------------------------------------------------
// Request builders
// ---------------------------------------------------------------------------

/// Build a Cedar entity UID for an agent.
pub fn agent_uid(agent_id: &str) -> EntityUid {
    EntityUid::from_type_name_and_id(agent_type(), EntityId::new(agent_id))
}

/// Build a Cedar entity UID for an action.
pub fn action_uid(action_name: &str) -> EntityUid {
    EntityUid::from_type_name_and_id(action_type(), EntityId::new(action_name))
}

/// Build a Cedar entity UID for a tool resource.
pub fn tool_uid(tool_name: &str) -> EntityUid {
    EntityUid::from_type_name_and_id(tool_type(), EntityId::new(tool_name))
}

/// Build a Cedar authorization request for a tool execution.
///
/// # Arguments
///
/// * `agent_id` — The agent attempting the action.
/// * `action` — The action being attempted (e.g., `"execute_tool"`, `"call_llm"`).
/// * `resource` — The tool or resource name.
///
/// # Errors
///
/// Returns an error if the Cedar request cannot be constructed.
pub fn tool_execution_request(
    agent_id: &str,
    action: &str,
    resource: &str,
) -> anyhow::Result<Request> {
    let principal = agent_uid(agent_id);
    let action = action_uid(action);
    let resource = tool_uid(resource);

    Request::new(
        principal,
        action,
        resource,
        Context::empty(),
        None, // no schema validation
    )
    .map_err(|e| anyhow::anyhow!("Failed to create Cedar request: {e}"))
}

/// Common action names used in policy evaluation.
pub mod actions {
    /// Executing a tool (MCP or native).
    pub const EXECUTE_TOOL: &str = "execute_tool";
    /// Making an LLM API call.
    pub const CALL_LLM: &str = "call_llm";
    /// Spawning a new agent actor.
    pub const SPAWN_AGENT: &str = "spawn_agent";
    /// Sending inter-agent collaboration messages.
    pub const COLLABORATE: &str = "collaborate";
    /// Accessing a knowledge base.
    pub const ACCESS_KNOWLEDGE: &str = "access_knowledge";
}
