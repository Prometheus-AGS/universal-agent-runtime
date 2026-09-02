//! Conversation-history normalization applied before every provider request.
//!
//! Enforces two invariants on a message list:
//!
//! 1. every assistant tool call has exactly one tool result;
//! 2. every tool result corresponds to an assistant tool call.
//!
//! A call without a result gets a synthetic, typed `cancelled` result inserted
//! after the call's result block, so the provider never sees a dangling call. A
//! result without a call is removed, so the provider never sees an orphaned
//! output (both shapes produce HTTP 400 from OpenAI- and Anthropic-style APIs).
//!
//! The design follows the invariant set in Codex CLI's
//! `core/src/context_manager/normalize.rs` (Apache-2.0), including the
//! reverse-index insertion so earlier positions stay valid while inserting.
//! No Codex code is vendored; the message model here is UAR's own.

use std::collections::HashSet;

use crate::llm::{Message, MessageContent, MessageRole};

/// Substring present in every synthetic cancelled result body.
///
/// The body is a small JSON object; this constant is the `status` field so a
/// caller can recognize a synthetic result without parsing.
pub const SYNTHETIC_CANCELLED_MARKER: &str = "\"status\":\"cancelled\"";

/// Substring present in every synthetic error result body.
pub const SYNTHETIC_ERROR_MARKER: &str = "\"status\":\"error\"";

/// Why a synthetic tool result was inserted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntheticReason {
    /// The call never produced a result before the turn ended.
    Cancelled,
    /// The call failed before a result could be recorded.
    Error(String),
}

/// What [`normalize_history`] changed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NormalizeReport {
    /// Call ids that received a synthetic result, in history order.
    pub synthesized: Vec<String>,
    /// Call ids (or `""` when absent) of removed orphan results, in history order.
    pub removed: Vec<String>,
}

impl NormalizeReport {
    /// True when nothing was changed.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.synthesized.is_empty() && self.removed.is_empty()
    }
}

/// Build the synthetic tool result recorded for a call that has no real result.
///
/// # Examples
///
/// ```
/// use universal_agent_runtime::uar::runtime::context::normalize::{
///     synthetic_tool_result, SyntheticReason, SYNTHETIC_CANCELLED_MARKER,
/// };
/// let m = synthetic_tool_result("c1", &SyntheticReason::Cancelled);
/// assert_eq!(m.tool_call_id.as_deref(), Some("c1"));
/// assert!(m.content.as_text().unwrap().contains(SYNTHETIC_CANCELLED_MARKER));
/// ```
#[must_use]
pub fn synthetic_tool_result(call_id: &str, reason: &SyntheticReason) -> Message {
    let body = match reason {
        SyntheticReason::Cancelled => serde_json::json!({
            "status": "cancelled",
            "tool_call_id": call_id,
            "message": "tool call did not return a result before the turn ended",
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

/// Normalize `messages` in place so every tool call has exactly one result and
/// every result has a call. Returns what changed.
///
/// Orphan results are removed first, so a later assistant call with the same id
/// cannot accidentally adopt a stale output. Missing results are then
/// synthesized as `cancelled`, walking assistant messages from the end so that
/// insertions never shift an index the walk has yet to visit.
///
/// # Examples
///
/// ```
/// use universal_agent_runtime::llm::{Message, MessageContent, MessageRole};
/// use universal_agent_runtime::uar::runtime::context::normalize::normalize_history;
/// let mut history = vec![Message {
///     role: MessageRole::Tool,
///     content: MessageContent::text("orphan"),
///     tool_call_id: Some("x".into()),
///     tool_calls: None,
/// }];
/// let report = normalize_history(&mut history);
/// assert!(history.is_empty());
/// assert_eq!(report.removed, vec!["x".to_string()]);
/// ```
pub fn normalize_history(messages: &mut Vec<Message>) -> NormalizeReport {
    let mut report = NormalizeReport::default();

    let call_ids: HashSet<String> = messages
        .iter()
        .filter(|m| m.role == MessageRole::Assistant)
        .flat_map(|m| m.tool_calls.iter().flatten())
        .map(|call| call.id.clone())
        .collect();

    // Invariant 2: remove results that no call produced.
    messages.retain(|m| {
        if m.role != MessageRole::Tool {
            return true;
        }
        let keep = m
            .tool_call_id
            .as_deref()
            .is_some_and(|id| call_ids.contains(id));
        if !keep {
            report
                .removed
                .push(m.tool_call_id.clone().unwrap_or_default());
        }
        keep
    });

    // Invariant 1: every call has exactly one result. Walk assistant messages
    // from the end so insertions after index `i` never move an unvisited one.
    let assistant_positions: Vec<usize> = messages
        .iter()
        .enumerate()
        .filter(|(_, m)| m.role == MessageRole::Assistant && m.tool_calls.is_some())
        .map(|(i, _)| i)
        .collect();

    // Filled back-to-front: each assistant message's ids are prepended as a
    // block, so the final vector is in history order.
    let mut synthesized: Vec<String> = Vec::new();
    for &i in assistant_positions.iter().rev() {
        let ids: Vec<String> = messages[i]
            .tool_calls
            .iter()
            .flatten()
            .map(|c| c.id.clone())
            .collect();

        // The result block is the run of Tool messages immediately after the call.
        let mut block_end = i + 1;
        while block_end < messages.len() && messages[block_end].role == MessageRole::Tool {
            block_end += 1;
        }
        let present: HashSet<String> = messages[i + 1..block_end]
            .iter()
            .filter_map(|m| m.tool_call_id.clone())
            .collect();

        // Duplicate results for one id: keep the first, drop the rest.
        let mut seen: HashSet<String> = HashSet::new();
        let mut j = i + 1;
        while j < block_end {
            let dup = messages[j]
                .tool_call_id
                .as_deref()
                .is_some_and(|id| !seen.insert(id.to_string()));
            if dup {
                report
                    .removed
                    .push(messages[j].tool_call_id.clone().unwrap_or_default());
                messages.remove(j);
                block_end -= 1;
            } else {
                j += 1;
            }
        }

        let mut insert_at = block_end;
        let mut this_block: Vec<String> = Vec::new();
        for id in ids.iter().filter(|id| !present.contains(id.as_str())) {
            messages.insert(
                insert_at,
                synthetic_tool_result(id, &SyntheticReason::Cancelled),
            );
            insert_at += 1;
            this_block.push(id.clone());
        }
        this_block.extend(synthesized);
        synthesized = this_block;
    }
    report.synthesized = synthesized;

    report
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
            tool_calls: Some(ids.iter().map(|i| call(i)).collect()),
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
    fn clean_history_is_untouched() {
        let mut h = vec![assistant(&["a"]), result("a")];
        let r = normalize_history(&mut h);
        assert!(r.is_clean());
        assert_eq!(h.len(), 2);
        assert_eq!(h[0].role, MessageRole::Assistant);
        assert_eq!(h[1].tool_call_id.as_deref(), Some("a"));
        assert_eq!(h[1].content.as_text(), Some("ok"));
    }

    #[test]
    fn two_calls_in_a_row_each_get_their_results_in_order() {
        let mut h = vec![assistant(&["a", "b"]), assistant(&["c"]), result("c")];
        let r = normalize_history(&mut h);
        assert_eq!(r.synthesized, vec!["a", "b"]);
        assert_eq!(h[1].tool_call_id.as_deref(), Some("a"));
        assert_eq!(h[2].tool_call_id.as_deref(), Some("b"));
        assert_eq!(h[3].role, MessageRole::Assistant);
        assert_eq!(h[4].tool_call_id.as_deref(), Some("c"));
    }

    #[test]
    fn duplicate_result_for_one_call_is_collapsed() {
        let mut h = vec![assistant(&["a"]), result("a"), result("a")];
        let r = normalize_history(&mut h);
        assert_eq!(r.removed, vec!["a"]);
        assert_eq!(h.len(), 2);
    }
}
