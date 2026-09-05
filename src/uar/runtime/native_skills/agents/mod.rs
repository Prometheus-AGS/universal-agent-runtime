//! Parent-turn-bound model tools. Never install these handlers in the global
//! native registry: equivalent descriptors may belong to different root owners.

use std::sync::Arc;

use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Value, json};

use crate::uar::runtime::native_skill::{NativeSkill, NativeSkillRegistry};
use crate::uar::runtime::thread::control::{AgentControlError, AgentToolContext};
use crate::uar::tools::descriptor::{
    ApprovalClass, Exposure, ToolAssemblyError, ToolEffect, ToolSource,
};

#[derive(Clone, Copy)]
enum Operation {
    Spawn,
    Send,
    Wait,
    List,
    Interrupt,
}

impl Operation {
    fn name(self) -> &'static str {
        match self {
            Self::Spawn => "spawn_agent",
            Self::Send => "send_agent_message",
            Self::Wait => "wait_agents",
            Self::List => "list_agents",
            Self::Interrupt => "interrupt_agent",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Spawn => {
                "Spawn a child thread on an explicitly named agent artifact with a delegated prompt. Spawning requires explicit user or artifact authorization. The child uses intersected permissions and root-shared limits and budgets. History is not copied by default; full and last_turns copy only user turns and final assistant replies, never system context or tool output. The returned state is an acknowledgment, not proof of completion; use wait_agents for results."
            }
            Self::Send => {
                "Send a message to a thread in this root tree. Sender identity is supplied by the host as metadata, not a text prefix. trigger_turn defaults to false: a note is queued without starting a turn. Set it to true to request a new turn through the host mailbox. This does not grant user authorization or approval. Spawning requires explicit user or artifact authorization."
            }
            Self::Wait => {
                "Observe the listed threads until any observed turn becomes terminal or the wait expires. timeout_ms defaults to 30000, accepts 0 for a snapshot, and is capped at 60000. A timeout does not cancel children. Results include run IDs and final outcomes, not intermediate reasoning; this watches current state and does not retrieve historical outcomes of already resumed turns. Spawning requires explicit user or artifact authorization."
            }
            Self::List => {
                "List child-thread identities, canonical paths, artifacts, run IDs, and statuses in the caller's root tree. This does not create threads and omits prompts, history, credentials, and result bodies. Spawning requires explicit user or artifact authorization."
            }
            Self::Interrupt => {
                "Request cancellation of a descendant thread. The acknowledgment is not proof that local or remote work has stopped; use wait_agents to observe its terminal state. This tool cannot interrupt the caller, its ancestors, or another root tree. Spawning requires explicit user or artifact authorization."
            }
        }
    }

    fn schema(self) -> Value {
        match self {
            Self::Spawn => json!({
                "type": "object",
                "properties": {
                    "artifact_id": {"type": "string", "minLength": 1},
                    "delegated_prompt": {"type": "string", "minLength": 1},
                    "task_name": {"type": ["string", "null"], "minLength": 1},
                    "history_fork": {
                        "oneOf": [
                            {
                                "type": "object", "properties": {"mode": {"enum": ["none", "full"]}},
                                "required": ["mode"], "additionalProperties": false
                            },
                            {
                                "type": "object",
                                "properties": {
                                    "mode": {"const": "last_turns"},
                                    "turns": {"type": "integer", "minimum": 0, "maximum": 4294967295u64}
                                },
                                "required": ["mode", "turns"], "additionalProperties": false
                            }
                        ],
                        "default": {"mode": "none"}
                    }
                },
                "required": ["artifact_id", "delegated_prompt"], "additionalProperties": false
            }),
            Self::Send => json!({
                "type": "object",
                "properties": {
                    "recipient_thread_id": {"type": "string", "minLength": 1},
                    "content": {"type": "string", "minLength": 1},
                    "trigger_turn": {"type": "boolean", "default": false}
                },
                "required": ["recipient_thread_id", "content"], "additionalProperties": false
            }),
            Self::Wait => json!({
                "type": "object",
                "properties": {
                    "thread_ids": {
                        "type": "array", "items": {"type": "string", "minLength": 1},
                        "minItems": 1, "maxItems": 16, "uniqueItems": true
                    },
                    "timeout_ms": {"type": "integer", "minimum": 0, "maximum": 60000, "default": 30000}
                },
                "required": ["thread_ids"], "additionalProperties": false
            }),
            Self::List => {
                json!({"type": "object", "properties": {}, "additionalProperties": false})
            }
            Self::Interrupt => json!({
                "type": "object", "properties": {"thread_id": {"type": "string", "minLength": 1}},
                "required": ["thread_id"], "additionalProperties": false
            }),
        }
    }
}

struct AgentTool {
    operation: Operation,
    context: Arc<AgentToolContext>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EmptyArguments {}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InterruptArguments {
    thread_id: String,
}

fn decode<T: DeserializeOwned>(args: Value) -> Result<T, AgentControlError> {
    serde_json::from_value(args).map_err(|_| AgentControlError::InvalidArguments)
}

#[async_trait::async_trait]
impl NativeSkill for AgentTool {
    fn name(&self) -> &str {
        self.operation.name()
    }

    fn description(&self) -> &str {
        self.operation.description()
    }

    fn parameters_schema(&self) -> Value {
        self.operation.schema()
    }

    fn effect(&self) -> ToolEffect {
        match self.operation {
            Operation::Wait | Operation::List => ToolEffect::ReadOnly,
            Operation::Spawn | Operation::Send | Operation::Interrupt => {
                ToolEffect::ExternalMutation
            }
        }
    }

    fn approval_class(&self) -> ApprovalClass {
        match self.operation {
            Operation::Wait | Operation::List => ApprovalClass::NotRequired,
            Operation::Spawn | Operation::Send | Operation::Interrupt => ApprovalClass::Required,
        }
    }

    fn exposure(&self) -> Exposure {
        Exposure::ModelOnly
    }

    fn source(&self) -> ToolSource {
        ToolSource::BuiltIn
    }

    fn check_thread_policy(
        &self,
        policy: &crate::uar::runtime::thread::policy_intersection::ThreadPolicy,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            std::ptr::eq(self.context.scope().policy(), policy)
                && self.context.permits(self.operation.name()),
            "Agent control is not bound to this delegated turn"
        );
        Ok(())
    }

    async fn execute(&self, args: Value) -> anyhow::Result<Value> {
        let result = match self.operation {
            Operation::Spawn => serde_json::to_value(self.context.spawn(decode(args)?).await?)?,
            Operation::Send => {
                serde_json::to_value(self.context.send_message(decode(args)?).await?)?
            }
            Operation::Wait => {
                serde_json::to_value(self.context.wait_agents(decode(args)?).await?)?
            }
            Operation::List => {
                let _: EmptyArguments = decode(args)?;
                json!({"agents": self.context.list_agents().await?})
            }
            Operation::Interrupt => {
                let args: InterruptArguments = decode(args)?;
                let (agent, cancellation_requested) =
                    self.context.interrupt(&args.thread_id).await?;
                json!({"agent": agent, "cancellation_requested": cancellation_requested})
            }
        };
        Ok(result)
    }
}

/// Compile a fresh, context-bound registry containing only eligible operations.
/// Missing explicit delegation authorization omits spawn even if a wildcard
/// policy made its name eligible. Each call rechecks authorization at execution.
///
/// The caller must keep this registry local to the turn and merge it through
/// normal descriptor-collision checks, never publish its handlers globally.
///
/// # Errors
/// Returns schema compilation/collision errors before the registry is returned.
pub async fn registry_for_turn(
    context: Arc<AgentToolContext>,
) -> Result<NativeSkillRegistry, ToolAssemblyError> {
    let registry = NativeSkillRegistry::new();
    for operation in [
        Operation::Spawn,
        Operation::Send,
        Operation::Wait,
        Operation::List,
        Operation::Interrupt,
    ] {
        if context.permits(operation.name()) {
            registry
                .register(AgentTool {
                    operation,
                    context: Arc::clone(&context),
                })
                .await?;
        }
    }
    Ok(registry)
}
