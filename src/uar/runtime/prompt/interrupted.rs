//! Persistent markers for assistant turns that did not finish.

use serde::{Deserialize, Serialize};

use super::{Authority, PromptFragment, PromptRole, PromptSection, Retention};

/// Why a partially generated assistant turn stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnInterruptionReason {
    ProviderError,
    Cancelled,
}

impl TurnInterruptionReason {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ProviderError => "provider_error",
            Self::Cancelled => "cancelled",
        }
    }
}

/// Host-authored evidence that preceding assistant content is incomplete.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnInterrupted {
    pub run_id: String,
    pub reason: TurnInterruptionReason,
}

impl TurnInterrupted {
    /// Render the interruption as a session-retained conversation fragment.
    #[must_use]
    pub fn into_fragment(self) -> PromptFragment {
        PromptFragment::new(
            format!("turn_interrupted.{}", self.run_id),
            PromptSection::ConversationHistory,
            format!("run:{}", self.run_id),
            Authority::Host,
            PromptRole::Assistant,
            Retention::Session,
            format!(
                "[TurnInterrupted: {}]\nThe preceding assistant content is partial, not a completed response.",
                self.reason.as_str()
            ),
        )
    }
}
