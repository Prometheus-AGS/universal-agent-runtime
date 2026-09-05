//! Completeness analysis for partial agent descriptor IRs.
//!
//! The [`CompletenessAnalyzer`] inspects a [`PartialAgentDescriptorIR`] and
//! determines which sections are present, which are missing, and whether the
//! document is ready for compilation.

use serde::{Deserialize, Serialize};

use super::ir::{PartialAgentDescriptorIR, SectionName};

/// Result of a completeness analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletenessReport {
    /// Sections that are present and populated.
    pub present: Vec<String>,
    /// Sections that are missing.
    pub missing: Vec<String>,
    /// Overall completeness percentage (0-100).
    pub completeness_pct: f64,
    /// Whether the document is ready for compilation.
    pub is_ready: bool,
    /// Human-readable summary of what's needed.
    pub summary: String,
    /// Targeted questions the conversational agent should ask next.
    pub next_questions: Vec<String>,
}

/// Analyzes a partial IR for completeness.
#[derive(Debug)]
pub struct CompletenessAnalyzer;

impl CompletenessAnalyzer {
    /// Analyze a partial IR and produce a [`CompletenessReport`].
    #[must_use]
    pub fn analyze(ir: &PartialAgentDescriptorIR) -> CompletenessReport {
        let mut present = Vec::new();
        let mut missing = Vec::new();

        // Check agent name
        if ir.agent_name.is_some() {
            present.push("Agent Name".to_string());
        } else {
            missing.push("Agent Name".to_string());
        }

        // Check all 15 sections
        for section in SectionName::ALL {
            if Self::is_section_present(ir, section) {
                present.push(section.display_name().to_string());
            } else {
                missing.push(section.display_name().to_string());
            }
        }

        let total = present.len() + missing.len();
        let completeness_pct = if total > 0 {
            (present.len() as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let is_ready = missing.is_empty();

        let summary = if is_ready {
            "All sections are present. The document is ready for compilation.".into()
        } else {
            format!(
                "{} of {} sections complete ({:.0}%). Missing: {}",
                present.len(),
                total,
                completeness_pct,
                missing.join(", ")
            )
        };

        let next_questions = Self::generate_questions(&missing);

        CompletenessReport {
            present,
            missing,
            completeness_pct,
            is_ready,
            summary,
            next_questions,
        }
    }

    /// Check if a specific section is present in the partial IR.
    fn is_section_present(ir: &PartialAgentDescriptorIR, section: SectionName) -> bool {
        match section {
            SectionName::Metadata => ir.metadata.is_some(),
            SectionName::Identity => ir.identity.is_some(),
            SectionName::Ui => ir.ui.is_some(),
            SectionName::Capabilities => ir.capabilities.is_some(),
            SectionName::Skills => ir.skills.is_some(),
            SectionName::Tools => ir.tools.is_some(),
            SectionName::McpServers => ir.mcp_servers.is_some(),
            SectionName::Knowledge => ir.knowledge.is_some(),
            SectionName::Memory => ir.memory.is_some(),
            SectionName::A2A => ir.a2a.is_some(),
            SectionName::Governance => ir.governance.is_some(),
            SectionName::Budgets => ir.budgets.is_some(),
            SectionName::Execution => ir.execution.is_some(),
            SectionName::Observability => ir.observability.is_some(),
            SectionName::Deployment => ir.deployment.is_some(),
            // v2 (CH-12) sections — not part of `SectionName::ALL`, so this
            // loop never reaches these arms; included for match
            // exhaustiveness and in case a future caller iterates all
            // variants directly.
            SectionName::ModelRequirements => ir.model_requirements.is_some(),
            SectionName::PromptDialect => ir.prompt_dialect.is_some(),
            SectionName::RagConfiguration => ir.rag_configuration.is_some(),
            SectionName::ContextStrategy => ir.context_strategy.is_some(),
            SectionName::ApiHarness => ir.api_harness.is_some(),
        }
    }

    /// Generate targeted questions for the conversational agent to ask,
    /// prioritized by importance.
    fn generate_questions(missing: &[String]) -> Vec<String> {
        let mut questions = Vec::new();

        for section in missing {
            let q = match section.as_str() {
                "Agent Name" => "What would you like to name this agent?",
                "Metadata" => {
                    "What version should I assign? Any description, tags, or license info?"
                }
                "Identity" => {
                    "What role should this agent play? Describe its persona and any system prompt."
                }
                "UI (A2UI)" => {
                    "Should this agent expose any UI forms, actions, or artifact renderers?"
                }
                "Capabilities" => {
                    "Which capabilities should be enabled? (streaming, file upload, code execution, web browsing)"
                }
                "Skills" => "Are there any skills (native or external) this agent should use?",
                "Tools" => "Which tools should this agent have access to? Any allow/deny lists?",
                "MCP Servers" => {
                    "Does this agent need to connect to any MCP (Model Context Protocol) servers?"
                }
                "Knowledge Base" => {
                    "Should this agent have a knowledge base? If so, what are the sources (files, URLs, databases)?"
                }
                "Memory Model" => {
                    "How should conversation memory work? Should there be persistent memory across sessions?"
                }
                "A2A Contracts" => {
                    "Will this agent expose any A2A endpoints or depend on other agents?"
                }
                "Governance" => "Are there Cedar policies or governance rules for this agent?",
                "Budgets & Constraints" => "Any token limits, cost caps, or rate limits?",
                "Execution Model" => {
                    "What execution mode? (sequential, parallel, reactive) Any iteration limits?"
                }
                "Observability" => "Should tracing, metrics, or structured logging be enabled?",
                "Deployment Profiles" => {
                    "What LLM provider/model should be used? Any resource constraints (memory, CPU)?"
                }
                _ => continue,
            };
            questions.push(q.into());
        }

        // Return at most 3 questions at a time to avoid overwhelming the user
        questions.truncate(3);
        questions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uar::compiler::ir::*;

    #[test]
    fn test_empty_partial_ir_is_not_ready() {
        let ir = PartialAgentDescriptorIR::default();
        let report = CompletenessAnalyzer::analyze(&ir);
        assert!(!report.is_ready);
        assert_eq!(report.missing.len(), 16); // agent_name + 15 sections
        assert!(!report.next_questions.is_empty());
    }

    #[test]
    fn test_complete_partial_ir_is_ready() {
        let ir = PartialAgentDescriptorIR {
            agent_name: Some("Test".into()),
            metadata: Some(MetadataSection {
                version: "1.0".into(),
                schema_version: None,
                description: None,
                author: None,
                license: None,
                tags: vec![],
                created: None,
                updated: None,
            }),
            identity: Some(IdentitySection {
                name: "test".into(),
                role: "helper".into(),
                persona: "A test".into(),
                system_prompt: None,
                instructions: vec![],
                greeting: None,
            }),
            ui: Some(UiSection {
                forms: vec![],
                artifacts: vec![],
                actions: vec![],
            }),
            capabilities: Some(CapabilitiesSection {
                streaming: false,
                file_upload: false,
                image_generation: false,
                code_execution: false,
                web_browsing: false,
                extensions: Default::default(),
            }),
            skills: Some(SkillsSection { skills: vec![] }),
            tools: Some(ToolsSection {
                tools: vec![],
                allow: vec![],
                deny: vec![],
            }),
            mcp_servers: Some(McpServersSection { servers: vec![] }),
            knowledge: Some(KnowledgeSection {
                sources: vec![],
                retrieval: None,
            }),
            memory: Some(MemorySection {
                conversation: ConversationMemoryConfig::default(),
                persistent: None,
            }),
            a2a: Some(A2ASection {
                endpoints: vec![],
                dependencies: vec![],
            }),
            governance: Some(GovernanceSection {
                cedar_policies: vec![],
                cedar_inline: None,
                data_classification: None,
                audit: AuditConfig::default(),
            }),
            budgets: Some(BudgetsSection {
                max_tokens_per_turn: None,
                max_tokens_per_session: None,
                max_tool_calls_per_turn: None,
                max_cost_per_session_usd: None,
                timeout_seconds: None,
                rate_limit: None,
            }),
            execution: Some(ExecutionSection {
                mode: None,
                max_iterations: None,
                stop_conditions: vec![],
                fallback_behavior: None,
            }),
            observability: Some(ObservabilitySection {
                tracing: TracingConfig::default(),
                metrics: MetricsConfig::default(),
                logging: LoggingConfig::default(),
            }),
            deployment: Some(DeploymentSection { profiles: vec![] }),
            // v2 (CH-12) sections are intentionally excluded from
            // completeness gating — a v1.1-style IR without them is still
            // "ready".
            ..Default::default()
        };

        let report = CompletenessAnalyzer::analyze(&ir);
        assert!(report.is_ready);
        assert!(report.missing.is_empty());
        assert_eq!(report.completeness_pct, 100.0);
    }

    #[test]
    fn test_partial_ir_questions_are_limited() {
        let ir = PartialAgentDescriptorIR::default();
        let report = CompletenessAnalyzer::analyze(&ir);
        assert!(report.next_questions.len() <= 3);
    }
}
