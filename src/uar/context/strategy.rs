use serde::{Deserialize, Serialize};

/// Context management strategy for controlling how conversation history is
/// trimmed before being sent to the LLM.
///
/// These strategies are independent of the token-budget–based
/// [`crate::uar::domain::context::ContextStrategy`] and operate at the
/// message-count / structural level.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Falls back to a 50-message sliding window until full implementation.
    Hierarchical {
        #[serde(default = "default_short_term_turns")]
        short_term_turns: usize,
        #[serde(default = "default_mid_term_tokens")]
        mid_term_summary_tokens: usize,
        #[serde(default = "default_long_term_tokens")]
        long_term_facts_tokens: usize,
    },
}

fn default_max_messages() -> usize {
    20
}
fn default_summarize_threshold() -> usize {
    6
}
fn default_summary_max_tokens() -> usize {
    500
}
fn default_keep_first() -> usize {
    2
}
fn default_keep_last() -> usize {
    4
}
fn default_short_term_turns() -> usize {
    5
}
fn default_mid_term_tokens() -> usize {
    2000
}
fn default_long_term_tokens() -> usize {
    500
}

impl Default for ContextStrategy {
    fn default() -> Self {
        Self::SlidingWindow {
            max_messages: default_max_messages(),
        }
    }
}

/// Quick token estimation: ~4 characters per token.
#[must_use]
pub fn estimate_tokens(text: &str) -> usize {
    (text.len() + 3) / 4
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

        ContextStrategy::Summarize { .. } | ContextStrategy::Hierarchical { .. } => {
            const FALLBACK: usize = 50;
            if items.len() <= FALLBACK {
                items
            } else {
                items[items.len() - FALLBACK..].to_vec()
            }
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

        // Summarize and Hierarchical require LLM calls.
        // V1: fall back to a generous sliding window.
        ContextStrategy::Summarize { .. } | ContextStrategy::Hierarchical { .. } => {
            const FALLBACK_MAX: usize = 50;
            if messages.len() <= FALLBACK_MAX {
                messages.to_vec()
            } else {
                messages[messages.len() - FALLBACK_MAX..].to_vec()
            }
        }
    }
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
}
