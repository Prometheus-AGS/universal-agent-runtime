//! Immutable metadata used to advertise, govern, validate, and schedule tools.
//!
//! A descriptor is the single runtime view of a tool. Execution code consumes
//! these fields directly; it does not infer effects or approval requirements
//! from provider-visible names.

use std::sync::Arc;

use jsonschema::Validator;
use serde_json::{Value, json};
use thiserror::Error;

use crate::uar::runtime::context::truncate::TruncationPolicy;

/// Where a tool implementation entered the runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolSource {
    /// An in-process skill implementing [`NativeSkill`](crate::uar::runtime::native_skill::NativeSkill).
    NativeSkill,
    /// A tool discovered from a Model Context Protocol server.
    Mcp,
    /// An in-process runtime tool outside the skill registry.
    BuiltIn,
}

/// The externally observable effect of executing a tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolEffect {
    /// Reads state without modifying its environment.
    ReadOnly,
    /// Mutates state outside the model process.
    ExternalMutation,
    /// Executes code or shell commands.
    CodeExecution,
    /// No trustworthy effect declaration is available.
    Unknown,
}

/// Whether governance may allow a tool without a human approval pause.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalClass {
    /// The descriptor does not independently require human approval.
    NotRequired,
    /// The descriptor requires approval unless stronger governance denies it.
    Required,
}

/// How a tool is exposed to a model or runtime caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exposure {
    /// Advertised in the initial model tool list.
    Eager,
    /// Discoverable and eligible for a later model step.
    Deferred,
    /// Omitted from model-visible surfaces.
    Hidden,
    /// Callable by the model but not by ordinary host clients.
    ModelOnly,
}

/// A provider-visible name was assigned to two non-identical descriptors.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("conflicting tool descriptors share provider-visible name '{provider_name}'")]
pub struct ToolCollision {
    pub provider_name: String,
}

/// Failure to assemble an executable tool descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ToolAssemblyError {
    /// The declared input schema could not be compiled.
    #[error("invalid input schema for tool '{provider_name}': {message}")]
    InvalidSchema {
        provider_name: String,
        message: String,
    },
    /// A provider-visible name maps to incompatible tool definitions.
    #[error(transparent)]
    Collision(#[from] ToolCollision),
}

/// The immutable runtime definition of one executable tool.
#[derive(Debug, Clone)]
pub struct ToolDescriptor {
    /// Stable source-local identifier.
    pub id: String,
    /// Provider-safe name advertised to the model.
    pub provider_name: String,
    /// Human-readable model-facing description.
    pub description: String,
    /// Registration source.
    pub source: ToolSource,
    /// MCP server name when the source is [`ToolSource::Mcp`].
    pub server: Option<String>,
    /// JSON Schema advertised to the model and enforced before execution.
    pub input_schema: Value,
    /// Validator compiled once when this descriptor is assembled.
    pub validator: Arc<Validator>,
    /// Declared effect used by the execution scheduler.
    pub effect: ToolEffect,
    /// Descriptor-level approval classification.
    pub approval_class: ApprovalClass,
    /// Whether execution must be routed through a sandbox.
    pub sandbox_required: bool,
    /// Optional key used to serialize conflicting read-only calls.
    pub concurrency_key: Option<String>,
    /// Model and host exposure class.
    pub exposure: Exposure,
    /// Optional descriptor-specific output bound.
    pub output_limit: Option<TruncationPolicy>,
}

impl ToolDescriptor {
    /// Render this descriptor in the OpenAI-compatible function-tool shape.
    #[must_use]
    pub fn openai_tool_json(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": self.provider_name,
                "description": self.description,
                "parameters": self.input_schema,
            }
        })
    }

    /// Compare descriptor semantics, excluding the compiled validator derived
    /// from `input_schema`.
    #[must_use]
    pub fn equivalent_to(&self, other: &Self) -> bool {
        self.id == other.id
            && self.provider_name == other.provider_name
            && self.description == other.description
            && self.source == other.source
            && self.server == other.server
            && self.input_schema == other.input_schema
            && self.effect == other.effect
            && self.approval_class == other.approval_class
            && self.sandbox_required == other.sandbox_required
            && self.concurrency_key == other.concurrency_key
            && self.exposure == other.exposure
            && self.output_limit == other.output_limit
    }
}
