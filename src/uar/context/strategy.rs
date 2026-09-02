use crate::llm::{LlmDriver, Message, MessageContent, MessageRole};
use serde::{Deserialize, Serialize};

/// Context management strategy for controlling how conversation history is
/// trimmed before being sent to the LLM.
///
/// These strategies are independent of the token-budget–based
/// [`crate::uar::domain::context::ContextStrategy`] and operate at the
/// message-count / structural level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContextStrategy {
    /// Send the full history unmodified.
    None,

    /// Keep only the most recent `max_messages` turns.
    SlidingWindow {
        #[serde(default = "default_max_messages")]
        max_messages: usize,
    },

    /// Summarize older messages when the history grows beyond `threshold` turns.
    /// Falls back to a 50-message sliding window until LLM summarisation is
    /// implemented.
    Summarize {
        #[serde(default = "default_summarize_threshold")]
        threshold: usize,
        #[serde(default = "default_summary_max_tokens")]
        summary_max_tokens: usize,
        model: Option<String>,
    },

    /// Keep the first `keep_first` messages and the last `keep_last` messages,
    /// dropping everything in the middle.
    TruncateMiddle {
        #[serde(default = "default_keep_first")]
        keep_first: usize,
        #[serde(default = "default_keep_last")]
        keep_last: usize,
    },

    /// Three-tier memory management (short / mid / long-term).
    Hierarchical {
        #[serde(default = "default_short_term_turns")]
        short_term_turns: usize,
        #[serde(default = "default_mid_term_tokens")]
        mid_term_summary_tokens: usize,
        #[serde(default = "default_long_term_tokens")]
        long_term_facts_tokens: usize,
    },

    /// Select a strategy tuned to the resolved model's context window at
    /// call time (CH-05, [`strategy_for_model`]) rather than a fixed
    /// configuration. Resolved by [`resolve_effective_strategy`] before
    /// reaching [`trim_count`]/[`apply_strategy`]/[`trim_with_summarization`]
    /// — those low-level functions treat an unresolved `Auto` as a
    /// conservative 128K-context default so they never panic on it.
    Auto,
}

// `pub(crate)` (not private): CH-14's conformance harness
// (`uar::compiler::conformance`) reuses these as the single source of truth
// for "what does the runtime default to when a v2 IR section leaves a field
// unset" instead of duplicating these numbers.
pub(crate) fn default_max_messages() -> usize {
    20
}
pub(crate) fn default_summarize_threshold() -> usize {
    6
}
pub(crate) fn default_summary_max_tokens() -> usize {
    500
}
pub(crate) fn default_keep_first() -> usize {
    2
}
pub(crate) fn default_keep_last() -> usize {
    4
}
pub(crate) fn default_short_term_turns() -> usize {
    5
}
pub(crate) fn default_mid_term_tokens() -> usize {
    2000
}
pub(crate) fn default_long_term_tokens() -> usize {
    500
}

impl Default for ContextStrategy {
    fn default() -> Self {
        Self::SlidingWindow {
            max_messages: default_max_messages(),
        }
    }
}

// The character-ratio estimator that once lived here is gone: every token
// count now goes through `uar::runtime::context::token_service::TokenService`,
// so one run cannot disagree with itself about what a token is.

/// Trim `history` under `strategy`, keeping `system` pinned at index 0.
///
/// This is the structural (message-count) half of history reduction. The
/// system message is never passed to a reducer, so no strategy can drop the
/// agent's identity, its policy summary, or its skill overlays; only
/// conversation turns are reduced. The pinned message is prepended to the
/// result when present.
///
/// # Examples
///
/// ```
/// use universal_agent_runtime::llm::{Message, MessageContent, MessageRole};
/// use universal_agent_runtime::uar::context::{trim_history, ContextStrategy};
///
/// let system = Message {
///     role: MessageRole::System,
///     content: MessageContent::text("identity"),
///     tool_call_id: None,
///     tool_calls: None,
/// };
/// let history: Vec<Message> = (0..40)
///     .map(|i| Message {
///         role: MessageRole::User,
///         content: MessageContent::text(format!("turn-{i}")),
///         tool_call_id: None,
///         tool_calls: None,
///     })
///     .collect();
///
/// let out = trim_history(
///     Some(system),
///     history,
///     &ContextStrategy::SlidingWindow { max_messages: 10 },
/// );
/// assert_eq!(out.len(), 11);
/// assert_eq!(out[0].role, MessageRole::System);
/// ```
#[must_use]
pub fn trim_history(
    system: Option<Message>,
    history: Vec<Message>,
    strategy: &ContextStrategy,
) -> Vec<Message> {
    let trimmed = trim_count(history, strategy);
    prepend_pinned(system, trimmed)
}

/// Async counterpart of [`trim_history`]: runs LLM-backed summarization for
/// the strategies that need it, with the system message pinned out of reach.
pub async fn trim_history_with_summarization(
    system: Option<Message>,
    history: Vec<Message>,
    strategy: &ContextStrategy,
    driver: Option<&dyn LlmDriver>,
) -> Vec<Message> {
    let trimmed = trim_with_summarization(history, strategy, driver).await;
    prepend_pinned(system, trimmed)
}

fn prepend_pinned(system: Option<Message>, rest: Vec<Message>) -> Vec<Message> {
    match system {
        Some(sys) => {
            let mut out = Vec::with_capacity(rest.len() + 1);
            out.push(sys);
            out.extend(rest);
            out
        }
        None => rest,
    }
}

/// Split a message list into its leading system message, if any, and the
/// conversation turns that follow.
///
/// Only a system message at index 0 is treated as pinned; a later system
/// message is an ordinary turn (for example a summarization marker) and stays
/// in the reducible history.
///
/// # Examples
///
/// ```
/// use universal_agent_runtime::llm::{Message, MessageContent, MessageRole};
/// use universal_agent_runtime::uar::context::split_pinned_system;
///
/// let msgs = vec![
///     Message { role: MessageRole::System, content: MessageContent::text("s"), tool_call_id: None, tool_calls: None },
///     Message { role: MessageRole::User, content: MessageContent::text("u"), tool_call_id: None, tool_calls: None },
/// ];
/// let (system, history) = split_pinned_system(msgs);
/// assert!(system.is_some());
/// assert_eq!(history.len(), 1);
/// ```
#[must_use]
pub fn split_pinned_system(mut messages: Vec<Message>) -> (Option<Message>, Vec<Message>) {
    if messages
        .first()
        .is_some_and(|m| m.role == MessageRole::System)
    {
        let system = messages.remove(0);
        (Some(system), messages)
    } else {
        (None, messages)
    }
}

/// Apply a [`ContextStrategy`] to any cloneable list, trimming by count/position.
///
/// This is the typed counterpart to [`apply_strategy`] — useful when messages are
/// already strongly typed (e.g. `Vec<crate::llm::Message>`) and you want to avoid
/// round-tripping through `serde_json::Value`.
#[must_use]
pub fn trim_count<T: Clone>(items: Vec<T>, strategy: &ContextStrategy) -> Vec<T> {
    match strategy {
        ContextStrategy::None => items,

        ContextStrategy::SlidingWindow { max_messages } => {
            let n = *max_messages;
            if items.len() <= n {
                items
            } else {
                items[items.len() - n..].to_vec()
            }
        }

        ContextStrategy::TruncateMiddle {
            keep_first,
            keep_last,
        } => {
            let total = keep_first + keep_last;
            if items.len() <= total {
                items
            } else {
                let mut result = items[..*keep_first].to_vec();
                result.extend_from_slice(&items[items.len() - keep_last..]);
                result
            }
        }

        // Real Summarize/Hierarchical behavior needs an LLM call and is
        // therefore async — see `trim_with_summarization`. This sync entry
        // point (and `Auto` reached unresolved) fall back to a generous
        // sliding window, same as before CH-05.
        ContextStrategy::Summarize { .. } | ContextStrategy::Hierarchical { .. } => {
            const FALLBACK: usize = 50;
            if items.len() <= FALLBACK {
                items
            } else {
                items[items.len() - FALLBACK..].to_vec()
            }
        }

        ContextStrategy::Auto => {
            trim_count(items, &strategy_for_model(DEFAULT_AUTO_CONTEXT_TOKENS))
        }
    }
}

/// Apply a [`ContextStrategy`] to a message list and return the filtered list.
///
/// Messages are `serde_json::Value` objects with at minimum a `"role"` and
/// `"content"` field (OpenAI chat format).
#[must_use]
pub fn apply_strategy(
    messages: &[serde_json::Value],
    strategy: &ContextStrategy,
) -> Vec<serde_json::Value> {
    match strategy {
        ContextStrategy::None => messages.to_vec(),

        ContextStrategy::SlidingWindow { max_messages } => {
            if messages.len() <= *max_messages {
                messages.to_vec()
            } else {
                messages[messages.len() - max_messages..].to_vec()
            }
        }

        ContextStrategy::TruncateMiddle {
            keep_first,
            keep_last,
        } => {
            let total_keep = keep_first + keep_last;
            if messages.len() <= total_keep {
                messages.to_vec()
            } else {
                let mut result = messages[..*keep_first].to_vec();
                result.extend_from_slice(&messages[messages.len() - keep_last..]);
                result
            }
        }

        // No `apply_strategy` caller currently needs real LLM summarization
        // (see `trim_with_summarization` for the `Message`-typed real path
        // used by `RunManager`) — this JSON-`Value` entry point keeps the
        // pre-CH-05 sliding-window fallback.
        ContextStrategy::Summarize { .. } | ContextStrategy::Hierarchical { .. } => {
            const FALLBACK_MAX: usize = 50;
            if messages.len() <= FALLBACK_MAX {
                messages.to_vec()
            } else {
                messages[messages.len() - FALLBACK_MAX..].to_vec()
            }
        }

        ContextStrategy::Auto => {
            apply_strategy(messages, &strategy_for_model(DEFAULT_AUTO_CONTEXT_TOKENS))
        }
    }
}

/// Conservative context-window assumption for `Auto` resolution when no
/// model-specific figure is available (e.g. reached `trim_count`/
/// `apply_strategy` directly instead of via `resolve_effective_strategy`).
const DEFAULT_AUTO_CONTEXT_TOKENS: u32 = 128_000;

/// Choose a context strategy tuned to a model's effective context window
/// (CH-05, fable §13). Larger-context models get a proportionally larger
/// sliding window before summarisation kicks in; small-context models
/// summarise sooner. `effective_context_tokens` should be the *usable*
/// window (typically 50-80% of advertised), not the advertised maximum.
///
/// This is the per-model *selection* layer; the chosen strategy is then
/// applied by [`apply_strategy`]. Placement (lost-in-the-middle mitigation)
/// is handled by [`keep_first_last`].
#[must_use]
pub fn strategy_for_model(effective_context_tokens: u32) -> ContextStrategy {
    // ~4 chars/token; budget ~60% of the effective window to history, leaving
    // room for the system prompt, tools, and the response.
    let history_tokens = (f64::from(effective_context_tokens) * 0.6) as u32;
    // Assume ~250 tokens/message average → derive a message budget, clamped.
    let max_messages = (history_tokens / 250).clamp(20, 400) as usize;
    if effective_context_tokens >= 200_000 {
        // Big-context models: prefer summarisation past a high threshold so
        // long-horizon coherence is preserved rather than truncated.
        ContextStrategy::Summarize {
            threshold: max_messages,
            summary_max_tokens: 2_000,
            model: None,
        }
    } else {
        ContextStrategy::SlidingWindow { max_messages }
    }
}

/// Resolve [`ContextStrategy::Auto`] into a concrete strategy using the
/// caller's model-specific effective context window (CH-05). Any other
/// strategy passes through unchanged — this is the seam a caller with real
/// model information (e.g. `RunManager`) should use *before* reaching
/// [`trim_count`]/[`apply_strategy`]/[`trim_with_summarization`], which only
/// see `DEFAULT_AUTO_CONTEXT_TOKENS` if `Auto` reaches them unresolved.
#[must_use]
pub fn resolve_effective_strategy(
    strategy: &ContextStrategy,
    effective_context_tokens: Option<u32>,
) -> ContextStrategy {
    match strategy {
        ContextStrategy::Auto => {
            strategy_for_model(effective_context_tokens.unwrap_or(DEFAULT_AUTO_CONTEXT_TOKENS))
        }
        other => other.clone(),
    }
}

/// Real Summarize/Hierarchical implementation (CH-05): calls
/// [`crate::uar::runtime::context::summarizer::summarize_messages`] (an
/// existing LLM-backed summarizer already used by the token-budget
/// `ContextManager`) instead of the sliding-window fallback `trim_count`
/// uses. Falls back to `trim_count`'s sliding-window behavior when no
/// `driver` is supplied, or when a summarization call fails/returns nothing
/// — a flaky or absent driver must never break a run by dropping history
/// outright.
///
/// `Hierarchical` produces a genuine three-tier result: the most recent
/// `short_term_turns` messages are kept verbatim; everything older is split
/// in half and each half is summarized with its own LLM pass (long-term:
/// the older half, mid-term: the newer-but-not-recent half) when there's
/// enough of it to benefit from two distinct compression passes, else the
/// whole older bulk gets one mid-term-only pass. Placement follows the
/// lost-in-the-middle mitigation ([`keep_first_last`]'s principle, fable
/// §13) structurally: long-term facts lead (high-attention head), verbatim
/// recent turns trail (high-attention tail — and causally required to be
/// last), the mid-term summary sits between the two as the least-attended,
/// least-critical tier by design.
pub async fn trim_with_summarization(
    messages: Vec<Message>,
    strategy: &ContextStrategy,
    driver: Option<&dyn LlmDriver>,
) -> Vec<Message> {
    fn summary_message(label: &str, text: &str) -> Message {
        Message {
            role: MessageRole::System,
            content: MessageContent::text(format!("[{label}]\n{text}")),
            tool_call_id: None,
            tool_calls: None,
        }
    }

    match strategy {
        ContextStrategy::Summarize { threshold, .. } => {
            if messages.len() <= *threshold {
                return messages;
            }
            let Some(driver) = driver else {
                return trim_count(messages, strategy);
            };
            let split = messages.len().saturating_sub(*threshold);
            let (old, recent) = messages.split_at(split);
            match crate::uar::runtime::context::summarizer::summarize_messages(old, driver).await {
                Ok(summary) if !summary.is_empty() => {
                    let mut out = Vec::with_capacity(recent.len() + 1);
                    out.push(summary_message("Earlier conversation summary", &summary));
                    out.extend_from_slice(recent);
                    out
                }
                _ => trim_count(messages, strategy),
            }
        }

        ContextStrategy::Hierarchical {
            short_term_turns, ..
        } => {
            if messages.len() <= *short_term_turns {
                return messages;
            }
            let Some(driver) = driver else {
                return trim_count(messages, strategy);
            };
            let recent_split = messages.len().saturating_sub(*short_term_turns);
            let (older, recent) = messages.split_at(recent_split);

            let (long_term_source, mid_term_source): (&[Message], &[Message]) = if older.len() > 4 {
                let mid_split = older.len() / 2;
                (&older[..mid_split], &older[mid_split..])
            } else {
                (&[], older)
            };

            let mut synthetic = Vec::new();
            if !long_term_source.is_empty()
                && let Ok(facts) = crate::uar::runtime::context::summarizer::summarize_messages(
                    long_term_source,
                    driver,
                )
                .await
                && !facts.is_empty()
            {
                synthetic.push(summary_message("Long-term facts", &facts));
            }
            if !mid_term_source.is_empty()
                && let Ok(summary) = crate::uar::runtime::context::summarizer::summarize_messages(
                    mid_term_source,
                    driver,
                )
                .await
                && !summary.is_empty()
            {
                synthetic.push(summary_message("Recent-history summary", &summary));
            }

            if synthetic.is_empty() {
                // Both summarization calls failed/produced nothing — fall
                // back rather than silently dropping all older context.
                return trim_count(messages, strategy);
            }
            synthetic.extend_from_slice(recent);
            synthetic
        }

        ContextStrategy::Auto => {
            Box::pin(trim_with_summarization(
                messages,
                &strategy_for_model(DEFAULT_AUTO_CONTEXT_TOKENS),
                driver,
            ))
            .await
        }

        _ => trim_count(messages, strategy),
    }
}

/// Positional-bias mitigation (lost-in-the-middle): reorder so the most
/// important items sit at the beginning and end of the context, where models
/// attend most reliably (Anthropic's ~18% middle drop vs 30-50% elsewhere,
/// fable §13). Keeps the first `head` and last `tail` items in place and
/// moves the middle to the end (least-attended region) preserving order.
#[must_use]
pub fn keep_first_last<T: Clone>(items: &[T], head: usize, tail: usize) -> Vec<T> {
    if items.len() <= head + tail {
        return items.to_vec();
    }
    let mut out = Vec::with_capacity(items.len());
    out.extend_from_slice(&items[..head]);
    out.extend_from_slice(&items[items.len() - tail..]);
    out.extend_from_slice(&items[head..items.len() - tail]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msgs(n: usize) -> Vec<serde_json::Value> {
        (0..n)
            .map(|i| serde_json::json!({"role": "user", "content": format!("msg-{i}")}))
            .collect()
    }

    #[test]
    fn none_returns_all() {
        let m = msgs(5);
        assert_eq!(apply_strategy(&m, &ContextStrategy::None).len(), 5);
    }

    #[test]
    fn sliding_window_trims() {
        let m = msgs(10);
        let result = apply_strategy(&m, &ContextStrategy::SlidingWindow { max_messages: 3 });
        assert_eq!(result.len(), 3);
        assert_eq!(result[0]["content"], "msg-7");
        assert_eq!(result[2]["content"], "msg-9");
    }

    #[test]
    fn sliding_window_no_op_when_under_limit() {
        let m = msgs(3);
        let result = apply_strategy(&m, &ContextStrategy::SlidingWindow { max_messages: 10 });
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn truncate_middle_keeps_ends() {
        let m = msgs(8);
        let result = apply_strategy(
            &m,
            &ContextStrategy::TruncateMiddle {
                keep_first: 2,
                keep_last: 3,
            },
        );
        assert_eq!(result.len(), 5);
        assert_eq!(result[0]["content"], "msg-0");
        assert_eq!(result[1]["content"], "msg-1");
        assert_eq!(result[2]["content"], "msg-5");
        assert_eq!(result[4]["content"], "msg-7");
    }

    #[test]
    fn truncate_middle_no_op_when_under_total_keep() {
        let m = msgs(4);
        let result = apply_strategy(
            &m,
            &ContextStrategy::TruncateMiddle {
                keep_first: 2,
                keep_last: 3,
            },
        );
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn summarize_falls_back_to_sliding_window() {
        let m = msgs(60);
        let result = apply_strategy(
            &m,
            &ContextStrategy::Summarize {
                threshold: 6,
                summary_max_tokens: 500,
                model: None,
            },
        );
        assert_eq!(result.len(), 50);
    }

    #[test]
    fn auto_resolves_to_sliding_window_below_200k() {
        let resolved = resolve_effective_strategy(&ContextStrategy::Auto, Some(64_000));
        assert!(matches!(resolved, ContextStrategy::SlidingWindow { .. }));
    }

    #[test]
    fn auto_resolves_to_summarize_at_large_context() {
        let resolved = resolve_effective_strategy(&ContextStrategy::Auto, Some(250_000));
        assert!(matches!(resolved, ContextStrategy::Summarize { .. }));
    }

    #[test]
    fn auto_falls_back_to_default_when_no_model_info() {
        let resolved = resolve_effective_strategy(&ContextStrategy::Auto, None);
        assert_eq!(resolved, strategy_for_model(DEFAULT_AUTO_CONTEXT_TOKENS));
    }

    #[test]
    fn non_auto_strategy_passes_through_unresolved() {
        let s = ContextStrategy::SlidingWindow { max_messages: 7 };
        let resolved = resolve_effective_strategy(&s, Some(1_000_000));
        assert_eq!(resolved, s);
    }

    fn typed_msgs(n: usize) -> Vec<Message> {
        (0..n)
            .map(|i| Message {
                role: MessageRole::User,
                content: MessageContent::text(format!("msg-{i}")),
                tool_call_id: None,
                tool_calls: None,
            })
            .collect()
    }

    #[tokio::test]
    async fn summarize_with_driver_produces_real_summary_plus_recent_tail() {
        let driver = crate::llm::mock_driver::MockLlmDriver::echo();
        let m = typed_msgs(10);
        let result = trim_with_summarization(
            m,
            &ContextStrategy::Summarize {
                threshold: 3,
                summary_max_tokens: 500,
                model: None,
            },
            Some(&driver),
        )
        .await;
        // 1 synthetic summary message + the last 3 kept verbatim.
        assert_eq!(result.len(), 4);
        assert!(
            result[0]
                .content
                .as_text()
                .unwrap()
                .contains("Hello from mock!"),
            "first message should be the LLM-produced summary, got: {:?}",
            result[0].content
        );
        assert_eq!(result[1].content.as_text().unwrap(), "msg-7");
        assert_eq!(result[3].content.as_text().unwrap(), "msg-9");
    }

    #[tokio::test]
    async fn summarize_without_driver_falls_back_to_sliding_window() {
        let m = typed_msgs(10);
        let result = trim_with_summarization(
            m,
            &ContextStrategy::Summarize {
                threshold: 3,
                summary_max_tokens: 500,
                model: None,
            },
            None,
        )
        .await;
        // Falls back to trim_count's Summarize arm: 50-msg fallback window,
        // no-op here since we only have 10.
        assert_eq!(result.len(), 10);
    }

    #[tokio::test]
    async fn summarize_below_threshold_is_a_no_op() {
        let m = typed_msgs(2);
        let driver = crate::llm::mock_driver::MockLlmDriver::echo();
        let result = trim_with_summarization(
            m,
            &ContextStrategy::Summarize {
                threshold: 5,
                summary_max_tokens: 500,
                model: None,
            },
            Some(&driver),
        )
        .await;
        assert_eq!(result.len(), 2);
        assert_eq!(driver.call_count(), 0, "no summarization call was needed");
    }

    #[tokio::test]
    async fn hierarchical_with_driver_produces_three_tiers() {
        let driver = crate::llm::mock_driver::MockLlmDriver::echo();
        let m = typed_msgs(20);
        let result = trim_with_summarization(
            m,
            &ContextStrategy::Hierarchical {
                short_term_turns: 4,
                mid_term_summary_tokens: 500,
                long_term_facts_tokens: 200,
            },
            Some(&driver),
        )
        .await;
        // long-term-facts + mid-term-summary + 4 verbatim recent turns.
        assert_eq!(result.len(), 6);
        assert!(
            result[0]
                .content
                .as_text()
                .unwrap()
                .contains("Long-term facts")
        );
        assert!(
            result[1]
                .content
                .as_text()
                .unwrap()
                .contains("Recent-history summary")
        );
        assert_eq!(result[2].content.as_text().unwrap(), "msg-16");
        assert_eq!(result[5].content.as_text().unwrap(), "msg-19");
        // Both the long-term and mid-term tiers each triggered one call.
        assert_eq!(driver.call_count(), 2);
    }

    #[tokio::test]
    async fn hierarchical_small_older_bulk_is_mid_term_only() {
        let driver = crate::llm::mock_driver::MockLlmDriver::echo();
        // 4 short-term + 3 older = only one summarization pass (older.len() <= 4).
        let m = typed_msgs(7);
        let result = trim_with_summarization(
            m,
            &ContextStrategy::Hierarchical {
                short_term_turns: 4,
                mid_term_summary_tokens: 500,
                long_term_facts_tokens: 200,
            },
            Some(&driver),
        )
        .await;
        // 1 mid-term summary + 4 verbatim recent turns.
        assert_eq!(result.len(), 5);
        assert!(
            result[0]
                .content
                .as_text()
                .unwrap()
                .contains("Recent-history summary")
        );
        assert_eq!(driver.call_count(), 1);
    }

    #[tokio::test]
    async fn hierarchical_below_threshold_is_a_no_op() {
        let m = typed_msgs(3);
        let result = trim_with_summarization(
            m,
            &ContextStrategy::Hierarchical {
                short_term_turns: 5,
                mid_term_summary_tokens: 500,
                long_term_facts_tokens: 200,
            },
            None,
        )
        .await;
        assert_eq!(result.len(), 3);
    }

    #[tokio::test]
    async fn auto_unresolved_uses_conservative_default_no_summarization() {
        // `Auto` reaching `trim_with_summarization` directly (i.e. NOT
        // pre-resolved via `resolve_effective_strategy`) uses the
        // conservative 128K default, which resolves to SlidingWindow (below
        // the 200K Summarize threshold) — so it must NOT call the driver.
        let driver = crate::llm::mock_driver::MockLlmDriver::echo();
        let m = typed_msgs(500);
        let result = trim_with_summarization(m, &ContextStrategy::Auto, Some(&driver)).await;
        assert_eq!(driver.call_count(), 0);
        assert!(result.len() < 500);
    }

    #[tokio::test]
    async fn auto_resolved_against_large_context_summarizes() {
        // The realistic flow: a caller with real model info (e.g.
        // `RunManager`) resolves `Auto` via `resolve_effective_strategy`
        // *before* calling `trim_with_summarization` — that resolved
        // strategy is what actually triggers summarization.
        let driver = crate::llm::mock_driver::MockLlmDriver::echo();
        let m = typed_msgs(500);
        let resolved = resolve_effective_strategy(&ContextStrategy::Auto, Some(250_000));
        assert!(matches!(resolved, ContextStrategy::Summarize { .. }));
        let result = trim_with_summarization(m, &resolved, Some(&driver)).await;
        assert!(driver.call_count() > 0);
        assert!(result.len() < 500);
    }
}
