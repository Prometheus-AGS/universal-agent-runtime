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

/// Get the Cedar entity type for a skill resource.
fn skill_type() -> EntityTypeName {
    "Skill".parse().expect("valid entity type")
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

/// Build a Cedar entity UID for a skill resource.
pub fn skill_uid(skill_id: &str) -> EntityUid {
    EntityUid::from_type_name_and_id(skill_type(), EntityId::new(skill_id))
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

/// Build a Cedar authorization request for a skill mutation.
///
/// Unlike tool execution requests, skill mutations carry context (environment,
/// confidence score, validation status) that Cedar policies evaluate against.
///
/// # Arguments
///
/// * `agent_id` — The agent attempting the mutation.
/// * `action` — The mutation action (e.g., `"skill.mutate"`, `"skill.promote"`).
/// * `skill_id` — The skill being mutated.
/// * `context_json` — JSON object with mutation context (environment, scores, etc.).
pub fn skill_mutation_request(
    agent_id: &str,
    action: &str,
    skill_id: &str,
    context_json: &str,
) -> anyhow::Result<Request> {
    let principal = agent_uid(agent_id);
    let action = action_uid(action);
    let resource = skill_uid(skill_id);

    let context = if context_json.is_empty() {
        Context::empty()
    } else {
        Context::from_json_str(context_json, None)
            .map_err(|e| anyhow::anyhow!("Invalid Cedar context JSON: {e}"))?
    };

    Request::new(principal, action, resource, context, None)
        .map_err(|e| anyhow::anyhow!("Failed to create Cedar skill mutation request: {e}"))
}

/// Common action names used in policy evaluation.
pub mod actions {
    /// Executing a tool (MCP or native).
    pub const EXECUTE_TOOL: &str = "execute_tool";
    /// Making an LLM API call.
    pub const CALL_LLM: &str = "call_llm";
    /// Validating an LLM response before it is returned to the user.
    pub const VALIDATE_OUTPUT: &str = "validate_output";
    /// Spawning a new agent actor.
    pub const SPAWN_AGENT: &str = "spawn_agent";
    /// Sending inter-agent collaboration messages.
    pub const COLLABORATE: &str = "collaborate";
    /// Accessing a knowledge base.
    pub const ACCESS_KNOWLEDGE: &str = "access_knowledge";

    // ── Skill mutation actions (self-learning pipeline) ──────────────

    /// Optimizing a skill's prompt (dspy-rs writes back to SKILL.md).
    pub const SKILL_MUTATE: &str = "skill.mutate";
    /// Generating a new skill from gap detection (PMPO pipeline).
    pub const SKILL_GENERATE: &str = "skill.generate";
    /// Promoting a generated skill from staging to active.
    pub const SKILL_PROMOTE: &str = "skill.promote";
    /// Capturing execution traces for the learning pipeline.
    pub const TRACE_CAPTURE: &str = "trace.capture";
}
