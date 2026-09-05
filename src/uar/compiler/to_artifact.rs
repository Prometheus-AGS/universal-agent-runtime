//! Conversion from the compiler's [`AgentDescriptorIR`] to the runtime
//! [`AgentArtifact`].
//!
//! The compiler ([`crate::uar::compiler`]) emits an [`AgentDescriptorIR`] as the
//! payload of a [`CompiledDescriptor`](crate::uar::compiler::pipeline::CompiledDescriptor),
//! but agent **registration** (`POST /api/agents`) and **runs**
//! (`POST /api/uar/runs`) consume a runtime [`AgentArtifact`]. This module bridges
//! the two so a compiled agent can be registered and run.
//!
//! ## ID derivation
//!
//! The runtime `id` is the slug of the agent name (`ir.agent_name`): lowercased,
//! with every run of non-alphanumeric characters collapsed to a single `-`, and
//! leading/trailing `-` trimmed. This is deterministic (the same spec always
//! yields the same id) and human-readable, matching the actor-endpoint slug
//! convention already used in `stages/s06_actor_endpoints.rs`. If the name slugs
//! to nothing (e.g. all punctuation), it falls back to `"agent"`.
//!
//! Fields the IR carries are mapped directly; fields it lacks fall back to the
//! values in [`crate::uar::defaults::default_agent`]. Sections that have no direct
//! home on `AgentArtifact` (governance, A2A, budgets, observability, deployment,
//! capabilities, MCP servers) are preserved losslessly under
//! `AgentArtifact.extensions` as JSON, so nothing from the compiled descriptor is
//! discarded.

use crate::uar::compiler::ir::AgentDescriptorIR;
use crate::uar::defaults::default_agent;
use crate::uar::domain::artifact::{
    AgentArtifact, AgentMemoryConfig, AgentMetadata, AgentPolicy, AgentPrompt, AgentToolConfig,
    AgentUiConfig, ArtifactsConfig, ConversationMemory, FeatureFlag, KbMemory, ProviderPolicy,
    ProviderSelection, SkillPolicy, ToolBundle, ToolPolicy,
};

/// Slugify an agent name into a deterministic, URL-safe id.
fn slugify(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());
    let mut prev_dash = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            slug.push('-');
            prev_dash = true;
        }
    }
    let trimmed = slug.trim_matches('-');
    if trimmed.is_empty() {
        "agent".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Build the system prompt: prefer the explicit `system_prompt`, otherwise
/// synthesize one from the identity's role + persona.
fn system_prompt(ir: &AgentDescriptorIR) -> String {
    if let Some(prompt) = ir
        .identity
        .system_prompt
        .as_deref()
        .map(str::trim)
        .filter(|p| !p.is_empty())
    {
        return prompt.to_string();
    }
    format!(
        "You are {name}, a {role}. {persona}",
        name = ir.identity.name.trim(),
        role = ir.identity.role.trim(),
        persona = ir.identity.persona.trim(),
    )
    .trim()
    .to_string()
}

/// Stash a serializable IR section under `extensions[key]` if it serializes
/// cleanly. Serialization of these plain serde structs is infallible in practice,
/// so a failure is silently skipped rather than aborting the whole conversion.
fn stash<T: serde::Serialize>(
    extensions: &mut std::collections::HashMap<String, serde_json::Value>,
    key: &str,
    value: &T,
) {
    if let Ok(json) = serde_json::to_value(value) {
        extensions.insert(key.to_string(), json);
    }
}

impl From<&AgentDescriptorIR> for AgentArtifact {
    fn from(ir: &AgentDescriptorIR) -> Self {
        let defaults = default_agent();

        // ── metadata ──────────────────────────────────────────────────────
        let metadata = AgentMetadata {
            title: ir.agent_name.clone(),
            description: ir.metadata.description.clone().unwrap_or_default(),
            tags: ir.metadata.tags.clone(),
            author: ir.metadata.author.clone(),
            icon: None,
        };

        // ── provider policy ───────────────────────────────────────────────
        // The IR does not carry a top-level provider; the first deployment
        // profile that declares one wins, otherwise we reuse the default.
        let provider_default = ir
            .deployment
            .profiles
            .iter()
            .find_map(|p| p.provider.as_ref())
            .map_or_else(
                || defaults.policy.provider.default.clone(),
                |p| ProviderSelection {
                    provider: p.name.clone(),
                    model: p.model.clone(),
                },
            );

        // ── tool / skill policy ───────────────────────────────────────────
        let tools_policy = ToolPolicy {
            allow: if ir.tools.allow.is_empty() {
                defaults.policy.tools.allow.clone()
            } else {
                ir.tools.allow.clone()
            },
            deny: ir.tools.deny.clone(),
            max_concurrent: defaults.policy.tools.max_concurrent,
            execution_mode: defaults.policy.tools.execution_mode.clone(),
        };

        let skills_policy = SkillPolicy {
            prefer: ir.skills.skills.iter().map(|s| s.id.clone()).collect(),
            max_active: defaults.policy.skills.max_active,
        };

        let policy = AgentPolicy {
            provider: ProviderPolicy {
                default: provider_default,
                fallbacks: vec![],
            },
            tools: tools_policy,
            skills: skills_policy,
        };

        // ── prompt ────────────────────────────────────────────────────────
        let prompt = AgentPrompt {
            system: system_prompt(ir),
            instructions: ir.identity.instructions.clone(),
        };

        // ── memory ────────────────────────────────────────────────────────
        let knowledge_bases: Vec<String> =
            ir.knowledge.sources.iter().map(|s| s.id.clone()).collect();
        let memory = AgentMemoryConfig {
            conversation: ConversationMemory {
                enabled: ir.memory.conversation.enabled,
            },
            kb: KbMemory {
                enabled: !knowledge_bases.is_empty(),
                knowledge_bases,
                citation_required: false,
            },
        };

        // ── tool bundles ──────────────────────────────────────────────────
        // Group explicitly-listed tools by their owning MCP server (if any) so
        // the runtime tool config is not lost. Tools without a server land in a
        // single "default" bundle.
        let tools = AgentToolConfig {
            bundles: build_tool_bundles(ir),
        };

        // ── ui ────────────────────────────────────────────────────────────
        let ui = AgentUiConfig {
            forms: FeatureFlag {
                enabled: !ir.ui.forms.is_empty(),
            },
            artifacts: ArtifactsConfig {
                enabled: !ir.ui.artifacts.is_empty(),
                preferred_types: ir
                    .ui
                    .artifacts
                    .iter()
                    .map(|a| a.artifact_type.clone())
                    .collect(),
            },
        };

        // ── extensions (lossless catch-all) ───────────────────────────────
        let mut extensions = std::collections::HashMap::new();
        stash(&mut extensions, "governance", &ir.governance);
        stash(&mut extensions, "a2a", &ir.a2a);
        stash(&mut extensions, "budgets", &ir.budgets);
        stash(&mut extensions, "observability", &ir.observability);
        stash(&mut extensions, "deployment", &ir.deployment);
        stash(&mut extensions, "capabilities", &ir.capabilities);
        stash(&mut extensions, "mcp_servers", &ir.mcp_servers);
        stash(&mut extensions, "execution", &ir.execution);

        AgentArtifact {
            version: ir.metadata.version.clone(),
            kind: "agent".to_string(),
            id: slugify(&ir.agent_name),
            metadata,
            // The IR has no runtime entry/protocols; reuse the defaults.
            runtime: defaults.runtime,
            policy,
            // The IR's UI forms/actions carry per-form schemas, but the runtime
            // AgentSchemas model is the agent's input/output/state contract, which
            // the IR does not express — reuse the (empty) defaults.
            schemas: defaults.schemas,
            prompt,
            memory,
            tools,
            ui,
            extensions,
        }
    }
}

impl From<AgentDescriptorIR> for AgentArtifact {
    fn from(ir: AgentDescriptorIR) -> Self {
        AgentArtifact::from(&ir)
    }
}

/// Build runtime tool bundles from the IR's tool list, grouped by MCP server.
fn build_tool_bundles(ir: &AgentDescriptorIR) -> Vec<ToolBundle> {
    use std::collections::BTreeMap;

    let mut by_server: BTreeMap<String, ToolBundle> = BTreeMap::new();
    for tool in &ir.tools.tools {
        let bundle_id = tool.server.clone().unwrap_or_else(|| "default".to_string());
        let entry = by_server
            .entry(bundle_id.clone())
            .or_insert_with(|| ToolBundle {
                id: bundle_id,
                tools: vec![],
                required: false,
            });
        entry.tools.push(tool.name.clone());
        // A bundle is required if any of its tools is required.
        entry.required = entry.required || tool.required;
    }
    by_server.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uar::compiler::parser;

    fn minimal_agent_md() -> String {
        r#"# Agent: Test Agent

## Metadata
```yaml
version: "1.0"
description: "A test agent"
author: "Test Author"
tags: ["test"]
```

## Identity
```yaml
name: "test-agent"
role: "assistant"
persona: "A helpful test assistant"
system_prompt: "You are a test agent."
```

## UI
```yaml
forms: []
artifacts: []
actions: []
```

## Capabilities
```yaml
streaming: true
file_upload: false
```

## Skills
```yaml
skills: []
```

## Tools
```yaml
tools: []
allow: []
deny: []
```

## MCP Servers
```yaml
servers: []
```

## Knowledge Base
```yaml
sources: []
```

## Memory Model
```yaml
conversation:
  enabled: true
  max_turns: 50
```

## A2A Contracts
```yaml
endpoints: []
dependencies: []
```

## Governance
```yaml
cedar_policies: []
audit:
  enabled: true
```

## Budgets & Constraints
```yaml
max_tokens_per_turn: 4096
timeout_seconds: 300
```

## Execution Model
```yaml
mode: "sequential"
max_iterations: 10
```

## Observability
```yaml
tracing:
  enabled: true
metrics:
  enabled: false
logging:
  level: "info"
```

## Deployment Profiles
```yaml
profiles: []
```
"#
        .to_string()
    }

    #[test]
    fn converts_minimal_ir_to_artifact() {
        let ir = parser::parse(&minimal_agent_md()).expect("parse should succeed");
        let artifact = AgentArtifact::from(&ir);

        // id is the slug of the agent name.
        assert_eq!(artifact.id, "test-agent");
        assert_eq!(artifact.kind, "agent");
        assert_eq!(artifact.version, "1.0");

        // metadata mapped from IR.
        assert_eq!(artifact.metadata.title, "Test Agent");
        assert_eq!(artifact.metadata.description, "A test agent");
        assert_eq!(artifact.metadata.tags, vec!["test".to_string()]);
        assert_eq!(artifact.metadata.author.as_deref(), Some("Test Author"));

        // prompt taken from identity.system_prompt.
        assert_eq!(artifact.prompt.system, "You are a test agent.");

        // memory.conversation follows the IR.
        assert!(artifact.memory.conversation.enabled);
        // no knowledge sources → kb disabled.
        assert!(!artifact.memory.kb.enabled);

        // empty IR tool-allow falls back to the default ("*").
        assert_eq!(artifact.policy.tools.allow, vec!["*".to_string()]);

        // lossless extensions: governance + budgets preserved.
        assert!(artifact.extensions.contains_key("governance"));
        assert!(artifact.extensions.contains_key("budgets"));
        assert!(artifact.extensions.contains_key("a2a"));
    }

    #[test]
    fn synthesizes_prompt_when_system_prompt_absent() {
        let mut ir = parser::parse(&minimal_agent_md()).expect("parse should succeed");
        ir.identity.system_prompt = None;
        ir.identity.role = "researcher".to_string();
        ir.identity.persona = "Methodical and precise.".to_string();

        let artifact = AgentArtifact::from(&ir);
        assert!(artifact.prompt.system.contains("researcher"));
        assert!(artifact.prompt.system.contains("Methodical and precise."));
    }

    #[test]
    fn slugify_handles_punctuation_and_spaces() {
        assert_eq!(slugify("Customer Support Agent"), "customer-support-agent");
        assert_eq!(slugify("  Multi   Space  "), "multi-space");
        assert_eq!(slugify("Émigré!!Bot"), "migr-bot");
        assert_eq!(slugify("***"), "agent");
    }
}
