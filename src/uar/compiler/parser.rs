//! UAR-AGENT-MD Markdown parser.
//!
//! Parses a UAR-AGENT-MD document into an [`AgentDescriptorIR`] (for complete
//! documents) or populates a [`PartialAgentDescriptorIR`] (for incomplete
//! documents in conversational mode).
//!
//! The parser:
//! 1. Extracts the agent name from the `# Agent: <name>` H1 heading.
//! 2. Splits the document into sections on `## <SectionName>` H2 headings.
//! 3. For each section, extracts the YAML code block and deserializes it into
//!    the corresponding IR section struct.

use sha2::{Digest, Sha256};

use super::error::CompileError;
use super::ir::{
    AgentDescriptorIR, LegacyRenameMapping, LegacySourceRecord, LegacySourceSection,
    PartialAgentDescriptorIR, SectionName,
};

const LEGACY_AGENT_PROFILE: &str = "urn:prometheus:uar:agent-md:1.1";

struct ExtractedDocument {
    agent_name: String,
    top_level_heading: String,
    sections: Vec<ExtractedSection>,
}

struct ExtractedSection {
    heading: String,
    canonical: Option<SectionName>,
    content: String,
}

/// Parse a complete UAR-AGENT-MD document into a full [`AgentDescriptorIR`].
///
/// Returns an error if any required section is missing or unparseable.
pub fn parse(markdown: &str) -> Result<AgentDescriptorIR, CompileError> {
    let partial = parse_partial(markdown)?;
    let missing_sections = find_missing_sections(&partial);
    partial.try_into_complete().ok_or_else(|| {
        CompileError::MissingSection(format!(
            "document is incomplete; missing sections: {}",
            missing_sections.join(", ")
        ))
    })
}

/// Parse a UAR-AGENT-MD document into a [`PartialAgentDescriptorIR`], tolerating
/// missing sections. Used by conversational mode and for lenient parsing.
pub fn parse_partial(markdown: &str) -> Result<PartialAgentDescriptorIR, CompileError> {
    let extracted = extract_sections(markdown)?;
    let mut ir = PartialAgentDescriptorIR {
        agent_name: Some(extracted.agent_name.clone()),
        ..Default::default()
    };

    for section in &extracted.sections {
        if let Some(section_name) = section.canonical {
            deserialize_section(&mut ir, section_name, section_yaml(&section.content))?;
        } else if serde_norway::from_str::<serde_json::Value>(section_yaml(&section.content))
            .ok()
            .and_then(|value| value.get("required").and_then(serde_json::Value::as_bool))
            .unwrap_or(false)
        {
            return Err(CompileError::Structure(format!(
                "unknown mandatory section '{}' cannot map to /legacySections/{}",
                section.heading,
                section.heading.replace('~', "~0").replace('/', "~1")
            )));
        }
    }

    ir.source = build_source_record(markdown, &extracted, &ir);

    Ok(ir)
}

/// Extract the agent name (from H1) and all H2 sections with their YAML content.
fn extract_sections(markdown: &str) -> Result<ExtractedDocument, CompileError> {
    let mut agent_name = None;
    let mut top_level_heading = None;
    let mut sections = Vec::new();
    let mut current: Option<ExtractedSection> = None;

    for line in markdown.split_inclusive('\n') {
        let heading_line = line.trim_end_matches(['\r', '\n']);
        if let Some(heading) = heading_line.strip_prefix("## ") {
            if let Some(previous) = current.take() {
                sections.push(previous);
            }
            current = Some(ExtractedSection {
                heading: heading_line.to_owned(),
                canonical: SectionName::from_heading(heading),
                content: String::new(),
            });
            continue;
        }

        if let Some(heading) = heading_line.strip_prefix("# ") {
            let Some(name) = heading.strip_prefix("Agent: ") else {
                return Err(CompileError::Structure(format!(
                    "unsupported top-level heading '# {heading}'; expected '# Agent: <name>'"
                )));
            };
            if name.trim().is_empty() || agent_name.is_some() {
                return Err(CompileError::Structure(
                    "document must contain exactly one non-empty '# Agent: <name>' heading".into(),
                ));
            }
            agent_name = Some(name.trim().to_owned());
            top_level_heading = Some(heading_line.to_owned());
            continue;
        }

        if let Some(section) = current.as_mut() {
            section.content.push_str(line);
        }
    }

    if let Some(previous) = current {
        sections.push(previous);
    }

    Ok(ExtractedDocument {
        agent_name: agent_name
            .ok_or_else(|| CompileError::Structure("missing '# Agent: <name>' heading".into()))?,
        top_level_heading: top_level_heading.expect("agent name and heading are set together"),
        sections,
    })
}

fn section_yaml(content: &str) -> &str {
    let trimmed = content.trim();
    if !trimmed.starts_with("```") {
        return trimmed;
    }
    let Some(first_newline) = trimmed.find('\n') else {
        return trimmed;
    };
    let body = &trimmed[first_newline + 1..];
    body.strip_suffix("```").map_or(body, str::trim_end)
}

fn build_source_record(
    markdown: &str,
    extracted: &ExtractedDocument,
    ir: &PartialAgentDescriptorIR,
) -> LegacySourceRecord {
    let heading_slug = slugify(&extracted.agent_name);
    let metadata_id = extracted
        .sections
        .iter()
        .find(|section| section.canonical == Some(SectionName::Metadata))
        .and_then(|section| {
            serde_norway::from_str::<serde_json::Value>(section_yaml(&section.content)).ok()
        })
        .and_then(|value| {
            value
                .get("id")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        });
    let source_id = metadata_id
        .clone()
        .unwrap_or_else(|| format!("urn:uar:legacy:{heading_slug}"));
    let version = ir
        .metadata
        .as_ref()
        .map(|metadata| metadata.version.clone())
        .unwrap_or_else(|| "0.0.0".to_owned());
    let mut authored_fields = Vec::new();
    let sections = extracted
        .sections
        .iter()
        .enumerate()
        .map(|(ordinal, section)| {
            let value = serde_norway::from_str::<serde_json::Value>(section_yaml(&section.content))
                .unwrap_or_else(|_| {
                    serde_json::Value::String(section_yaml(&section.content).to_owned())
                });
            if let Some(canonical) = section.canonical {
                let authored_value = if canonical == SectionName::Skills {
                    value.get("skills").unwrap_or(&value)
                } else {
                    &value
                };
                collect_authored_fields(
                    authored_value,
                    &format!("/{}", section_pointer(canonical)),
                    &mut authored_fields,
                );
            }
            LegacySourceSection {
                heading: section.heading.clone(),
                canonical: section.canonical,
                ordinal,
                content: section.content.clone(),
                value,
            }
        })
        .collect();

    authored_fields.sort();
    authored_fields.dedup();
    LegacySourceRecord {
        profile: LEGACY_AGENT_PROFILE.to_owned(),
        id: source_id.clone(),
        version,
        digest: sha256(markdown.as_bytes()),
        revision: None,
        migrated_at: Some(chrono::Utc::now()),
        top_level_heading: extracted.top_level_heading.clone(),
        original: markdown.to_owned(),
        sections,
        authored_fields,
        rename_mapping: LegacyRenameMapping {
            source_id: source_id.clone(),
            target_id: source_id,
            reason: if metadata_id.is_some() {
                "unchanged".to_owned()
            } else {
                "heading-slug".to_owned()
            },
        },
    }
}

fn collect_authored_fields(value: &serde_json::Value, pointer: &str, fields: &mut Vec<String>) {
    fields.push(pointer.to_owned());
    match value {
        serde_json::Value::Object(object) => {
            for (key, child) in object {
                collect_authored_fields(
                    child,
                    &format!("{pointer}/{}", key.replace('~', "~0").replace('/', "~1")),
                    fields,
                );
            }
        }
        serde_json::Value::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                collect_authored_fields(child, &format!("{pointer}/{index}"), fields);
            }
        }
        _ => {}
    }
}

fn section_pointer(section: SectionName) -> &'static str {
    match section {
        SectionName::Metadata => "metadata",
        SectionName::Identity => "identity",
        SectionName::Ui => "ui",
        SectionName::Capabilities => "capabilities",
        SectionName::Skills => "skills",
        SectionName::Tools => "tools",
        SectionName::McpServers => "mcp_servers",
        SectionName::Knowledge => "knowledge",
        SectionName::Memory => "memory",
        SectionName::A2A => "a2a",
        SectionName::Governance => "governance",
        SectionName::Budgets => "budgets",
        SectionName::Execution => "execution",
        SectionName::Observability => "observability",
        SectionName::Deployment => "deployment",
        SectionName::ModelRequirements => "model_requirements",
        SectionName::PromptDialect => "prompt_dialect",
        SectionName::RagConfiguration => "rag_configuration",
        SectionName::ContextStrategy => "context_strategy",
        SectionName::ApiHarness => "api_harness",
    }
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut separator = false;
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            separator = false;
        } else if !separator && !slug.is_empty() {
            slug.push('-');
            separator = true;
        }
    }
    slug.trim_matches('-').to_owned()
}

fn sha256(bytes: &[u8]) -> String {
    let mut encoded = String::from("sha256:");
    for byte in Sha256::digest(bytes) {
        use std::fmt::Write as _;
        write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    encoded
}

/// Deserialize a YAML string into the appropriate section of the partial IR.
fn deserialize_section(
    ir: &mut PartialAgentDescriptorIR,
    section: SectionName,
    yaml: &str,
) -> Result<(), CompileError> {
    let yaml = yaml.trim();
    if yaml.is_empty() {
        return Ok(());
    }

    let map_err = |e: serde_norway::Error| CompileError::SectionDeserialize {
        section: section.display_name().into(),
        message: e.to_string(),
    };

    match section {
        SectionName::Metadata => {
            ir.metadata = Some(serde_norway::from_str(yaml).map_err(map_err)?);
        }
        SectionName::Identity => {
            ir.identity = Some(serde_norway::from_str(yaml).map_err(map_err)?);
        }
        SectionName::Ui => {
            ir.ui = Some(serde_norway::from_str(yaml).map_err(map_err)?);
        }
        SectionName::Capabilities => {
            ir.capabilities = Some(serde_norway::from_str(yaml).map_err(map_err)?);
        }
        SectionName::Skills => {
            ir.skills = Some(match serde_norway::from_str(yaml) {
                Ok(section) => section,
                Err(_) => super::ir::SkillsSection {
                    skills: serde_norway::from_str(yaml).map_err(map_err)?,
                },
            });
        }
        SectionName::Tools => {
            ir.tools = Some(serde_norway::from_str(yaml).map_err(map_err)?);
        }
        SectionName::McpServers => {
            ir.mcp_servers = Some(serde_norway::from_str(yaml).map_err(map_err)?);
        }
        SectionName::Knowledge => {
            ir.knowledge = Some(serde_norway::from_str(yaml).map_err(map_err)?);
        }
        SectionName::Memory => {
            let mut value: serde_json::Value = serde_norway::from_str(yaml).map_err(map_err)?;
            if let Some(conversation) = value
                .as_object_mut()
                .and_then(|object| object.get_mut("conversation"))
                && let Some(enabled) = conversation.as_bool()
            {
                *conversation = serde_json::json!({ "enabled": enabled });
            }
            ir.memory = Some(deserialize_json_section(value, section)?);
        }
        SectionName::A2A => {
            ir.a2a = Some(serde_norway::from_str(yaml).map_err(map_err)?);
        }
        SectionName::Governance => {
            ir.governance = Some(serde_norway::from_str(yaml).map_err(map_err)?);
        }
        SectionName::Budgets => {
            ir.budgets = Some(serde_norway::from_str(yaml).map_err(map_err)?);
        }
        SectionName::Execution => {
            ir.execution = Some(serde_norway::from_str(yaml).map_err(map_err)?);
        }
        SectionName::Observability => {
            ir.observability = Some(serde_norway::from_str(yaml).map_err(map_err)?);
        }
        SectionName::Deployment => {
            let mut value: serde_json::Value = serde_norway::from_str(yaml).map_err(map_err)?;
            if let Some(profiles) = value
                .as_object_mut()
                .and_then(|object| object.get_mut("profiles"))
                .and_then(serde_json::Value::as_array_mut)
            {
                for profile in profiles {
                    if let Some(id) = profile.as_str() {
                        *profile = serde_json::json!({ "id": id });
                    }
                }
            }
            ir.deployment = Some(deserialize_json_section(value, section)?);
        }
        SectionName::ModelRequirements => {
            ir.model_requirements = Some(deserialize_json_section(
                requirement_value(yaml, section)?,
                section,
            )?);
        }
        SectionName::PromptDialect => {
            let value = requirement_value(yaml, section)?;
            let value = match value {
                serde_json::Value::String(value) if value == "default" => serde_json::json!({}),
                serde_json::Value::String(value) => serde_json::json!({ "dialect": value }),
                value => value,
            };
            ir.prompt_dialect = Some(deserialize_json_section(value, section)?);
        }
        SectionName::RagConfiguration => {
            let mut value = requirement_value(yaml, section)?;
            if let Some(object) = value.as_object_mut()
                && let Some(mode) = object.remove("mode")
                && mode == "disabled"
            {
                object.insert("enabled".to_owned(), serde_json::Value::Bool(false));
            }
            ir.rag_configuration = Some(deserialize_json_section(value, section)?);
        }
        SectionName::ContextStrategy => {
            let mut value = requirement_value(yaml, section)?;
            if let Some(object) = value.as_object_mut()
                && let Some(mode) = object.remove("mode")
                && mode == "selected"
            {
                object.insert(
                    "type".to_owned(),
                    serde_json::Value::String("auto".to_owned()),
                );
            }
            ir.context_strategy = Some(deserialize_json_section(value, section)?);
        }
        SectionName::ApiHarness => {
            let value = requirement_value(yaml, section)?;
            ir.api_harness = Some(deserialize_json_section(value, section)?);
        }
    }

    Ok(())
}

fn deserialize_json_section<T: serde::de::DeserializeOwned>(
    value: serde_json::Value,
    section: SectionName,
) -> Result<T, CompileError> {
    serde_json::from_value(value).map_err(|error| CompileError::SectionDeserialize {
        section: section.display_name().into(),
        message: error.to_string(),
    })
}

fn requirement_value(yaml: &str, section: SectionName) -> Result<serde_json::Value, CompileError> {
    let mut value: serde_json::Value =
        serde_norway::from_str(yaml).map_err(|error| CompileError::SectionDeserialize {
            section: section.display_name().into(),
            message: error.to_string(),
        })?;
    if let Some(object) = value.as_object_mut() {
        object.remove("required");
        if let Some(inner) = object.remove("value") {
            return Ok(inner);
        }
    }
    Ok(value)
}

/// Find which required sections are missing from a partial IR.
fn find_missing_sections(ir: &PartialAgentDescriptorIR) -> Vec<String> {
    let mut missing = Vec::new();

    if ir.agent_name.is_none() {
        missing.push("Agent Name".into());
    }
    if ir.metadata.is_none() {
        missing.push(SectionName::Metadata.display_name().into());
    }
    if ir.identity.is_none() {
        missing.push(SectionName::Identity.display_name().into());
    }
    if ir.ui.is_none() {
        missing.push(SectionName::Ui.display_name().into());
    }
    if ir.capabilities.is_none() {
        missing.push(SectionName::Capabilities.display_name().into());
    }
    if ir.skills.is_none() {
        missing.push(SectionName::Skills.display_name().into());
    }
    if ir.tools.is_none() {
        missing.push(SectionName::Tools.display_name().into());
    }
    if ir.mcp_servers.is_none() {
        missing.push(SectionName::McpServers.display_name().into());
    }
    if ir.knowledge.is_none() {
        missing.push(SectionName::Knowledge.display_name().into());
    }
    if ir.memory.is_none() {
        missing.push(SectionName::Memory.display_name().into());
    }
    if ir.a2a.is_none() {
        missing.push(SectionName::A2A.display_name().into());
    }
    if ir.governance.is_none() {
        missing.push(SectionName::Governance.display_name().into());
    }
    if ir.budgets.is_none() {
        missing.push(SectionName::Budgets.display_name().into());
    }
    if ir.execution.is_none() {
        missing.push(SectionName::Execution.display_name().into());
    }
    if ir.observability.is_none() {
        missing.push(SectionName::Observability.display_name().into());
    }
    if ir.deployment.is_none() {
        missing.push(SectionName::Deployment.display_name().into());
    }

    missing
}

/// A minimal but complete UAR-AGENT-MD document (all 15 v1.1 sections,
/// trivial values) — used as a base fixture by this module's own tests and,
/// via `pub(crate)`, by CH-14's conformance-harness tests
/// (`uar::compiler::conformance`), which append v2 sections to it rather
/// than duplicating all 15 required sections themselves.
#[cfg(test)]
pub(crate) fn minimal_agent_md() -> String {
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

#[cfg(test)]
mod tests {
    use super::super::ir::ContextStrategySection;
    use super::*;

    #[test]
    fn test_parse_complete_document() {
        let md = minimal_agent_md();
        let result = parse(&md);
        assert!(result.is_ok(), "parse failed: {result:?}");
        let ir = result.unwrap();
        assert_eq!(ir.agent_name, "Test Agent");
        assert_eq!(ir.metadata.version, "1.0");
        assert_eq!(ir.identity.name, "test-agent");
        assert!(ir.capabilities.streaming);
    }

    #[test]
    fn test_parse_partial_document() {
        let md = r#"# Agent: Partial Bot

## Metadata
```yaml
version: "0.1"
```

## Identity
```yaml
name: "partial"
role: "helper"
persona: "I help with things"
```
"#;
        let result = parse_partial(md);
        assert!(result.is_ok());
        let ir = result.unwrap();
        assert_eq!(ir.agent_name.as_deref(), Some("Partial Bot"));
        assert!(ir.metadata.is_some());
        assert!(ir.identity.is_some());
        assert!(ir.tools.is_none());
        assert!(ir.governance.is_none());
    }

    #[test]
    fn test_parse_rejects_missing_heading() {
        let md = "Just some random markdown without an agent heading.";
        let result = parse(md);
        assert!(result.is_err());
    }

    #[test]
    fn test_section_name_from_heading() {
        assert_eq!(
            SectionName::from_heading("Metadata"),
            Some(SectionName::Metadata)
        );
        assert_eq!(
            SectionName::from_heading("UI (A2UI)"),
            Some(SectionName::Ui)
        );
        assert_eq!(
            SectionName::from_heading("MCP Servers"),
            Some(SectionName::McpServers)
        );
        assert_eq!(
            SectionName::from_heading("Budgets & Constraints"),
            Some(SectionName::Budgets)
        );
        assert_eq!(SectionName::from_heading("Unknown Section"), None);
    }

    #[test]
    fn test_partial_try_into_complete_fails_when_missing() {
        let partial = PartialAgentDescriptorIR::default();
        assert!(partial.try_into_complete().is_none());
    }

    // ── CH-12 agent-spec-v2 ─────────────────────────────────────────────

    #[test]
    fn v1_1_document_without_v2_sections_still_parses_with_defaults() {
        // `minimal_agent_md()` predates CH-12 and declares none of the five
        // v2 sections — this is the "v1.1 still loads" backward-compat
        // contract in test form.
        let md = minimal_agent_md();
        let ir = parse(&md).expect("v1.1-style document must still parse");
        assert!(!ir.model_requirements.needs_tools);
        assert!(ir.prompt_dialect.dialect.is_none());
        assert!(!ir.rag_configuration.enabled);
        assert!(matches!(ir.context_strategy, ContextStrategySection::Auto));
        assert!(ir.api_harness.protocols.is_empty());
    }

    #[test]
    fn v2_sections_parse_when_declared() {
        let mut md = minimal_agent_md();
        md.push_str(
            r#"
## Model Requirements
```yaml
needs_tools: true
needs_reasoning: true
min_context: 200000
preferred_provider: "anthropic"
```

## Prompt Dialect
```yaml
dialect: "anthropic_xml"
wants_reasoning: true
hard: true
```

## RAG Configuration
```yaml
enabled: true
decomposition: true
verification: true
knowledge_base_ids: ["kb-1"]
```

## Context Strategy
```yaml
type: "hierarchical"
short_term_turns: 5
mid_term_summary_tokens: 2000
```

## API Harness
```yaml
protocols: ["a2a", "agui"]
stream_mode: "agui_spec"
```
"#,
        );

        let ir = parse(&md).expect("v2 document must parse");
        assert!(ir.model_requirements.needs_tools);
        assert!(ir.model_requirements.needs_reasoning);
        assert_eq!(ir.model_requirements.min_context, Some(200_000));
        assert_eq!(
            ir.model_requirements.preferred_provider.as_deref(),
            Some("anthropic")
        );
        assert_eq!(ir.prompt_dialect.dialect.as_deref(), Some("anthropic_xml"));
        assert!(ir.prompt_dialect.hard);
        assert!(ir.rag_configuration.enabled);
        assert_eq!(ir.rag_configuration.knowledge_base_ids, vec!["kb-1"]);
        match ir.context_strategy {
            ContextStrategySection::Hierarchical {
                short_term_turns,
                mid_term_summary_tokens,
                ..
            } => {
                assert_eq!(short_term_turns, Some(5));
                assert_eq!(mid_term_summary_tokens, Some(2000));
            }
            other => panic!("expected Hierarchical, got {other:?}"),
        }
        assert_eq!(ir.api_harness.protocols, vec!["a2a", "agui"]);
        assert_eq!(ir.api_harness.stream_mode.as_deref(), Some("agui_spec"));
    }

    #[test]
    fn v2_section_headings_recognized() {
        assert_eq!(
            SectionName::from_heading("Model Requirements"),
            Some(SectionName::ModelRequirements)
        );
        assert_eq!(
            SectionName::from_heading("Prompt Dialect"),
            Some(SectionName::PromptDialect)
        );
        assert_eq!(
            SectionName::from_heading("RAG Configuration"),
            Some(SectionName::RagConfiguration)
        );
        assert_eq!(
            SectionName::from_heading("Context Strategy"),
            Some(SectionName::ContextStrategy)
        );
        assert_eq!(
            SectionName::from_heading("API Harness"),
            Some(SectionName::ApiHarness)
        );
    }
}
