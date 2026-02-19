//! AgentCard for the UAR Compiler Agent.
//!
//! Served at `/.well-known/agent.json` per A2A RC v1.0 §3.
//! Declares the compiler agent's capabilities, skills, and auth schemes.

use super::types::{AgentCapabilities, AgentCard, AgentSkill, SecurityScheme};

/// Build the AgentCard for the UAR Compiler Agent.
///
/// The `base_url` should be the public-facing URL of this UAR instance,
/// e.g. `http://localhost:3928`.
pub fn build_agent_card(base_url: &str) -> AgentCard {
    AgentCard {
        name: "UAR Compiler Agent".into(),
        description: concat!(
            "Compiles UAR-AGENT-MD specification documents into signed, runnable agent ",
            "descriptors through the PMPO Spec→Plan→Execute→Reflect pipeline. ",
            "Supports both single-shot compilation and multi-turn conversational extraction."
        )
        .into(),
        url: format!("{base_url}/a2a/compiler"),
        version: "0.1.0".into(),
        documentation_url: Some(
            "https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/COMPILER.md"
                .into(),
        ),
        capabilities: AgentCapabilities {
            streaming: false,
            push_notifications: false,
            state_transition_history: true,
        },
        skills: vec![
            AgentSkill {
                id: "uar.compile".into(),
                name: "Compile Agent Specification".into(),
                description: concat!(
                    "Accepts a complete UAR-AGENT-MD document and compiles it into a signed ",
                    "agent descriptor JSON through the 8-stage PMPO pipeline."
                )
                .into(),
                tags: vec!["compiler".into(), "agent".into(), "uar".into()],
                examples: vec!["Compile this agent spec: # Agent: MyBot\n## Metadata\n...".into()],
                input_modes: vec!["text".into()],
                output_modes: vec!["data".into()],
            },
            AgentSkill {
                id: "uar.compile.conversational".into(),
                name: "Conversational Agent Builder".into(),
                description: concat!(
                    "Guides the user through building a UAR-AGENT-MD specification ",
                    "incrementally via multi-turn conversation, then compiles the result."
                )
                .into(),
                tags: vec!["compiler".into(), "conversational".into(), "builder".into()],
                examples: vec![
                    "I want to create a customer support agent that uses GPT-4o.".into(),
                    "Help me build an agent with access to my knowledge base.".into(),
                ],
                input_modes: vec!["text".into()],
                output_modes: vec!["text".into(), "data".into()],
            },
        ],
        security_schemes: vec![
            SecurityScheme {
                scheme_type: "http".into(),
                scheme: Some("bearer".into()),
                description: Some("JWT token obtained from /api/uar/auth/exchange".into()),
                location: None,
                name: None,
            },
            SecurityScheme {
                scheme_type: "apiKey".into(),
                scheme: None,
                description: Some("API key passed in the X-API-Key header".into()),
                location: Some("header".into()),
                name: Some("X-API-Key".into()),
            },
        ],
        default_input_modes: vec!["text".into()],
        default_output_modes: vec!["text".into(), "data".into()],
    }
}
