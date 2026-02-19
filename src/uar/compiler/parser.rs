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

use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use tracing::warn;

use super::error::CompileError;
use super::ir::{AgentDescriptorIR, PartialAgentDescriptorIR, SectionName};

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
    let (agent_name, raw_sections) = extract_sections(markdown)?;
    let mut ir = PartialAgentDescriptorIR {
        agent_name: Some(agent_name),
        ..Default::default()
    };

    for (section_name, yaml_content) in &raw_sections {
        deserialize_section(&mut ir, *section_name, yaml_content)?;
    }

    Ok(ir)
}

/// Extract the agent name (from H1) and all H2 sections with their YAML content.
fn extract_sections(markdown: &str) -> Result<(String, Vec<(SectionName, String)>), CompileError> {
    let parser = Parser::new(markdown);

    let mut agent_name: Option<String> = None;
    let mut sections: Vec<(SectionName, String)> = Vec::new();

    // State machine for walking the Markdown AST
    let mut in_heading = false;
    let mut heading_level: Option<HeadingLevel> = None;
    let mut heading_text = String::new();
    let mut current_section: Option<SectionName> = None;
    let mut in_code_block = false;
    let mut code_content = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                // If we were accumulating a code block for a previous section,
                // finalize it before starting the new heading.
                if let Some(section) = current_section.take() {
                    if !code_content.trim().is_empty() {
                        sections.push((section, code_content.clone()));
                    }
                    code_content.clear();
                }

                in_heading = true;
                heading_level = Some(level);
                heading_text.clear();
            }
            Event::Text(text) if in_heading => {
                heading_text.push_str(&text);
            }
            Event::Code(code) if in_heading => {
                heading_text.push_str(&code);
            }
            Event::End(TagEnd::Heading(_)) => {
                in_heading = false;

                match heading_level {
                    Some(HeadingLevel::H1) => {
                        // Extract agent name from "# Agent: <name>"
                        let name = heading_text
                            .strip_prefix("Agent:")
                            .or_else(|| heading_text.strip_prefix("Agent "))
                            .unwrap_or(&heading_text)
                            .trim()
                            .to_string();
                        if !name.is_empty() {
                            agent_name = Some(name);
                        }
                    }
                    Some(HeadingLevel::H2) => {
                        // Try to match this heading to a known section
                        if let Some(section) = SectionName::from_heading(&heading_text) {
                            current_section = Some(section);
                            code_content.clear();
                        } else {
                            warn!(heading = %heading_text, "unknown section heading, skipping");
                        }
                    }
                    _ => {}
                }

                heading_text.clear();
                heading_level = None;
            }
            Event::Start(Tag::CodeBlock(_)) if current_section.is_some() => {
                in_code_block = true;
                code_content.clear();
            }
            Event::Text(text) if in_code_block => {
                code_content.push_str(&text);
            }
            Event::End(TagEnd::CodeBlock) if in_code_block => {
                in_code_block = false;
                // Keep code_content accumulated; it will be consumed when
                // the next heading starts or at the end of the document.
            }
            // For sections that use plain text/paragraphs instead of code blocks (e.g., governance cedar_inline)
            Event::Text(text) if current_section.is_some() && !in_code_block && !in_heading => {
                // Accumulate plain text as content if no code block has been seen yet for this section
                if code_content.is_empty() || code_content.ends_with('\n') {
                    code_content.push_str(&text);
                } else {
                    code_content.push_str(&text);
                }
            }
            _ => {}
        }
    }

    // Finalize the last section
    if let Some(section) = current_section.take() {
        if !code_content.trim().is_empty() {
            sections.push((section, code_content));
        }
    }

    let agent_name = agent_name
        .ok_or_else(|| CompileError::Structure("missing '# Agent: <name>' heading".into()))?;

    Ok((agent_name, sections))
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

    let map_err = |e: serde_yml::Error| CompileError::SectionDeserialize {
        section: section.display_name().into(),
        message: e.to_string(),
    };

    match section {
        SectionName::Metadata => {
            ir.metadata = Some(serde_yml::from_str(yaml).map_err(map_err)?);
        }
        SectionName::Identity => {
            ir.identity = Some(serde_yml::from_str(yaml).map_err(map_err)?);
        }
        SectionName::Ui => {
            ir.ui = Some(serde_yml::from_str(yaml).map_err(map_err)?);
        }
        SectionName::Capabilities => {
            ir.capabilities = Some(serde_yml::from_str(yaml).map_err(map_err)?);
        }
        SectionName::Skills => {
            ir.skills = Some(serde_yml::from_str(yaml).map_err(map_err)?);
        }
        SectionName::Tools => {
            ir.tools = Some(serde_yml::from_str(yaml).map_err(map_err)?);
        }
        SectionName::McpServers => {
            ir.mcp_servers = Some(serde_yml::from_str(yaml).map_err(map_err)?);
        }
        SectionName::Knowledge => {
            ir.knowledge = Some(serde_yml::from_str(yaml).map_err(map_err)?);
        }
        SectionName::Memory => {
            ir.memory = Some(serde_yml::from_str(yaml).map_err(map_err)?);
        }
        SectionName::A2A => {
            ir.a2a = Some(serde_yml::from_str(yaml).map_err(map_err)?);
        }
        SectionName::Governance => {
            ir.governance = Some(serde_yml::from_str(yaml).map_err(map_err)?);
        }
        SectionName::Budgets => {
            ir.budgets = Some(serde_yml::from_str(yaml).map_err(map_err)?);
        }
        SectionName::Execution => {
            ir.execution = Some(serde_yml::from_str(yaml).map_err(map_err)?);
        }
        SectionName::Observability => {
            ir.observability = Some(serde_yml::from_str(yaml).map_err(map_err)?);
        }
        SectionName::Deployment => {
            ir.deployment = Some(serde_yml::from_str(yaml).map_err(map_err)?);
        }
    }

    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
