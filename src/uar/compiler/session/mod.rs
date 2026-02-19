//! Compiler session state for conversational mode.
//!
//! A [`CompilerSession`] tracks the incremental construction of an agent
//! descriptor through multi-turn conversation. It holds:
//! - A [`PartialAgentDescriptorIR`] being progressively filled.
//! - Conversation history for the LLM.
//! - A log of section updates for auditability.

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod persistence;

use super::ir::{PartialAgentDescriptorIR, SectionName};

/// A multi-turn compiler session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerSession {
    /// Unique session ID.
    pub id: String,
    /// When the session was created.
    pub created_at: DateTime<Utc>,
    /// When the session was last updated.
    pub updated_at: DateTime<Utc>,
    /// Current state of the partial descriptor.
    pub partial_ir: PartialAgentDescriptorIR,
    /// Conversation turns for this session.
    pub conversation: Vec<ConversationTurn>,
    /// Sections that have been updated during this session.
    pub updated_sections: HashSet<String>,
    /// Current session status.
    pub status: SessionStatus,
}

/// A single conversation turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub role: TurnRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

/// Who sent this turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TurnRole {
    User,
    Agent,
    System,
}

/// Session lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    /// Actively gathering information.
    Active,
    /// All sections complete, ready for compilation.
    Ready,
    /// Compilation in progress.
    Compiling,
    /// Successfully compiled.
    Completed,
    /// Session was cancelled.
    Cancelled,
    /// Compilation failed.
    Failed,
}

impl CompilerSession {
    /// Create a new session.
    #[must_use]
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            created_at: now,
            updated_at: now,
            partial_ir: PartialAgentDescriptorIR::default(),
            conversation: Vec::new(),
            updated_sections: HashSet::new(),
            status: SessionStatus::Active,
        }
    }

    /// Add a conversation turn.
    pub fn add_turn(&mut self, role: TurnRole, content: String) {
        self.conversation.push(ConversationTurn {
            role,
            content,
            timestamp: Utc::now(),
        });
        self.updated_at = Utc::now();
    }

    /// Record that a section was updated.
    pub fn mark_section_updated(&mut self, section: SectionName) {
        self.updated_sections
            .insert(section.display_name().to_string());
        self.updated_at = Utc::now();
    }

    /// Get the number of filled sections.
    #[must_use]
    pub fn filled_count(&self) -> usize {
        let ir = &self.partial_ir;
        let mut count = 0;
        if ir.agent_name.is_some() {
            count += 1;
        }
        if ir.metadata.is_some() {
            count += 1;
        }
        if ir.identity.is_some() {
            count += 1;
        }
        if ir.ui.is_some() {
            count += 1;
        }
        if ir.capabilities.is_some() {
            count += 1;
        }
        if ir.skills.is_some() {
            count += 1;
        }
        if ir.tools.is_some() {
            count += 1;
        }
        if ir.mcp_servers.is_some() {
            count += 1;
        }
        if ir.knowledge.is_some() {
            count += 1;
        }
        if ir.memory.is_some() {
            count += 1;
        }
        if ir.a2a.is_some() {
            count += 1;
        }
        if ir.governance.is_some() {
            count += 1;
        }
        if ir.budgets.is_some() {
            count += 1;
        }
        if ir.execution.is_some() {
            count += 1;
        }
        if ir.observability.is_some() {
            count += 1;
        }
        if ir.deployment.is_some() {
            count += 1;
        }
        count
    }

    /// Total required sections (agent_name + 15 sections = 16).
    #[must_use]
    pub const fn total_sections() -> usize {
        16
    }
}

impl Default for CompilerSession {
    fn default() -> Self {
        Self::new()
    }
}
