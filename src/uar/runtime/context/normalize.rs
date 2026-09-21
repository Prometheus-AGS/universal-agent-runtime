//! Lossless conversation-history validation before provider requests.
//!
//! Validation never repairs history by deleting records or inventing tool
//! outcomes. A missing, orphaned, duplicate, or misplaced record is returned as
//! an explicit error so the host can recover it from durable state or block the
//! request without changing canonical history.

use std::collections::{HashMap, HashSet};

use anyhow::Context as _;

use crate::llm::{Message, MessageContent, MessageRole};

/// Substring present in every explicitly recorded cancelled result body.
pub const SYNTHETIC_CANCELLED_MARKER: &str = "\"status\":\"cancelled\"";

/// Substring present in every explicitly recorded error result body.
pub const SYNTHETIC_ERROR_MARKER: &str = "\"status\":\"error\"";

/// A terminal outcome proven by durable host state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntheticReason {
    /// The durable host record proves the call was cancelled.
    Cancelled,
    /// The durable host record proves the call failed.
    Error(String),
}

/// Counts observed while validating a complete history.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NormalizeReport {
    /// Number of assistant tool calls validated.
    pub tool_calls: usize,
    /// Number of matching tool results validated.
    pub tool_results: usize,
}

impl NormalizeReport {
    /// True when every counted call has one result.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.tool_calls == self.tool_results
    }
}

/// Why canonical history cannot be sent to a provider.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum HistoryValidationError {
    /// Two calls claim the same identity.
    #[error("conflicting tool call identity '{call_id}'")]
    ConflictingCallIdentity { call_id: String },
    /// A completed call has no recorded result.
    #[error("tool call '{call_id}' has no result; terminal state is unproven")]
    MissingResult { call_id: String },
    /// A result has no call anywhere in canonical history.
    #[error("tool result '{call_id}' has no matching call")]
    OrphanResult { call_id: String },
    /// A tool result has no identity.
    #[error("tool result at history index {index} has no tool_call_id")]
    ResultWithoutIdentity { index: usize },
    /// More than one result claims the same call.
    #[error("tool call '{call_id}' has duplicate results")]
    DuplicateResult { call_id: String },
    /// A result exists but is outside the contiguous result block for its call.
    #[error("tool result '{call_id}' is not adjacent to its owning assistant call")]
    MisplacedResult { call_id: String },
}

/// Build a typed terminal result after the host has verified durable evidence.
///
/// This helper does not inspect history and is never called by validation. Its
/// caller owns the proof that the call is terminal and must persist provenance
/// with the canonical receipt before adding the returned record.
#[must_use]
pub fn synthetic_tool_result(call_id: &str, reason: &SyntheticReason) -> Message {
    let body = match reason {
        SyntheticReason::Cancelled => serde_json::json!({
            "status": "cancelled",
            "tool_call_id": call_id,
            "message": "durable host state records the tool call as cancelled",
        }),
        SyntheticReason::Error(detail) => serde_json::json!({
            "status": "error",
            "tool_call_id": call_id,
            "message": detail,
        }),
    };
    Message {
        role: MessageRole::Tool,
        content: MessageContent::text(body.to_string()),
        tool_call_id: Some(call_id.to_string()),
        tool_calls: None,
    }
}

/// Validate canonical tool call/result structure without modifying `messages`.
///
/// Each assistant call owns the contiguous block of tool messages immediately
/// following it. Calls and results must be one-to-one, and call identities must
/// be unique across the history.
pub fn normalize_history(messages: &[Message]) -> Result<NormalizeReport, HistoryValidationError> {
    let mut call_owners = HashMap::new();
    let mut result_positions: HashMap<&str, Vec<usize>> = HashMap::new();
    let mut tool_calls = 0usize;
    for (index, message) in messages.iter().enumerate() {
        for call in message.tool_calls.iter().flatten() {
            tool_calls += 1;
            if call_owners.insert(call.id.as_str(), index).is_some() {
                return Err(HistoryValidationError::ConflictingCallIdentity {
                    call_id: call.id.clone(),
                });
            }
        }
        if message.role == MessageRole::Tool
            && let Some(call_id) = message.tool_call_id.as_deref()
        {
            result_positions.entry(call_id).or_default().push(index);
        }
    }

    let mut tool_results = 0usize;
    let mut index = 0usize;
    while index < messages.len() {
        let message = &messages[index];
        if message.role == MessageRole::Assistant {
            let owned: Vec<&str> = message
                .tool_calls
                .iter()
                .flatten()
                .map(|call| call.id.as_str())
                .collect();
            if !owned.is_empty() {
                let owned_set: HashSet<&str> = owned.iter().copied().collect();
                let mut seen = HashSet::new();
                let mut result_index = index + 1;
                while result_index < messages.len()
                    && messages[result_index].role == MessageRole::Tool
                {
                    let result_id = messages[result_index].tool_call_id.as_deref().ok_or(
                        HistoryValidationError::ResultWithoutIdentity {
                            index: result_index,
                        },
                    )?;
                    if !owned_set.contains(result_id) {
                        return Err(if call_owners.contains_key(result_id) {
                            HistoryValidationError::MisplacedResult {
                                call_id: result_id.to_string(),
                            }
                        } else {
                            HistoryValidationError::OrphanResult {
                                call_id: result_id.to_string(),
                            }
                        });
                    }
                    if !seen.insert(result_id) {
                        return Err(HistoryValidationError::DuplicateResult {
                            call_id: result_id.to_string(),
                        });
                    }
                    tool_results += 1;
                    result_index += 1;
                }
                if let Some(missing) = owned.into_iter().find(|id| !seen.contains(id)) {
                    return Err(if result_positions.contains_key(missing) {
                        HistoryValidationError::MisplacedResult {
                            call_id: missing.to_string(),
                        }
                    } else {
                        HistoryValidationError::MissingResult {
                            call_id: missing.to_string(),
                        }
                    });
                }
                index = result_index;
                continue;
            }
        }

        if message.role == MessageRole::Tool {
            let result_id = message
                .tool_call_id
                .as_deref()
                .ok_or(HistoryValidationError::ResultWithoutIdentity { index })?;
            return Err(if call_owners.contains_key(result_id) {
                HistoryValidationError::MisplacedResult {
                    call_id: result_id.to_string(),
                }
            } else {
                HistoryValidationError::OrphanResult {
                    call_id: result_id.to_string(),
                }
            });
        }
        index += 1;
    }

    Ok(NormalizeReport {
        tool_calls,
        tool_results,
    })
}

/// Validate the JSON message vector immediately before an [`LlmDriver`]
/// request. The input remains byte-for-byte unchanged on success and failure.
///
/// [`LlmDriver`]: crate::llm::LlmDriver
pub fn normalize_provider_messages(
    messages: &[serde_json::Value],
) -> anyhow::Result<NormalizeReport> {
    let typed: Vec<Message> = messages
        .iter()
        .cloned()
        .map(serde_json::from_value)
        .collect::<Result<_, _>>()
        .context("provider history contains a message outside UAR's typed message contract")?;
    normalize_history(&typed).map_err(anyhow::Error::new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{ToolCall, ToolCallFunction};

    fn call(id: &str) -> ToolCall {
        ToolCall {
            id: id.to_string(),
            call_type: "function".to_string(),
            function: ToolCallFunction {
                name: "t".to_string(),
                arguments: "{}".to_string(),
            },
        }
    }

    fn assistant(ids: &[&str]) -> Message {
        Message {
            role: MessageRole::Assistant,
            content: MessageContent::text(""),
            tool_call_id: None,
            tool_calls: Some(ids.iter().map(|id| call(id)).collect()),
        }
    }

    fn result(id: &str) -> Message {
        Message {
            role: MessageRole::Tool,
            content: MessageContent::text("ok"),
            tool_call_id: Some(id.to_string()),
            tool_calls: None,
        }
    }

    #[test]
    fn complete_parallel_history_is_valid() {
        let history = vec![assistant(&["a", "b"]), result("b"), result("a")];
        let report = normalize_history(&history).expect("parallel group is complete");
        assert_eq!(report.tool_calls, 2);
        assert_eq!(report.tool_results, 2);
    }

    #[test]
    fn missing_result_is_not_synthesized() {
        let history = vec![assistant(&["a"])];
        let before = serde_json::to_value(&history).expect("serialize history");
        assert_eq!(
            normalize_history(&history),
            Err(HistoryValidationError::MissingResult {
                call_id: "a".to_string(),
            })
        );
        assert_eq!(serde_json::to_value(&history).unwrap(), before);
    }

    #[test]
    fn duplicate_result_is_not_removed() {
        let history = vec![assistant(&["a"]), result("a"), result("a")];
        let before = serde_json::to_value(&history).expect("serialize history");
        assert_eq!(
            normalize_history(&history),
            Err(HistoryValidationError::DuplicateResult {
                call_id: "a".to_string(),
            })
        );
        assert_eq!(serde_json::to_value(&history).unwrap(), before);
    }
}
