//! Bounded tool output.
//!
//! Tool results enter conversation history verbatim today, so one verbose
//! command can fill the context window in a single iteration. This module
//! truncates output once, at ingest, middle-out: the head and tail of the
//! output are kept because that is where a command's intent and its outcome
//! live, and a warning header states what was removed so the model can ask for
//! the rest.
//!
//! The policy shape and the header text follow Codex CLI's
//! `utils/output-truncation` (Apache-2.0). The implementation is UAR's own and
//! counts tokens through [`TokenService`] so budgets agree with the rest of the
//! runtime.

use super::token_service::TokenService;

/// Start of the header every truncated result carries.
pub const WARNING_HEADER_PREFIX: &str = "Warning: truncated output (original token count: ";

/// Marker placed where the middle of the output was removed.
const CUT_MARKER_OPEN: &str = "\n[... ";
const CUT_MARKER_CLOSE: &str = " bytes truncated ...]\n";

/// Default byte budget for a tool result when the tool declares none.
pub const DEFAULT_OUTPUT_BYTE_BUDGET: usize = 32_000;

/// How much of a tool result may enter history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TruncationPolicy {
    /// Keep at most this many bytes, header included.
    Bytes(usize),
    /// Keep at most this many tokens, header included.
    Tokens(usize),
}

impl Default for TruncationPolicy {
    fn default() -> Self {
        Self::Bytes(DEFAULT_OUTPUT_BYTE_BUDGET)
    }
}

/// Truncate `content` to `policy` if needed, prefixing a warning header that
/// states the original token count and total line count. Output already
/// within the policy is returned unchanged.
///
/// The returned string, header included, never exceeds the policy's byte
/// budget.
///
/// # Examples
///
/// ```
/// use universal_agent_runtime::uar::runtime::context::truncate::{formatted_truncate, TruncationPolicy, WARNING_HEADER_PREFIX};
/// assert_eq!(formatted_truncate("ok", TruncationPolicy::Bytes(100)), "ok");
/// let big = "x".repeat(10_000);
/// let out = formatted_truncate(&big, TruncationPolicy::Bytes(500));
/// assert!(out.len() <= 500);
/// assert!(out.starts_with(WARNING_HEADER_PREFIX));
/// ```
#[must_use]
pub fn formatted_truncate(content: &str, policy: TruncationPolicy) -> String {
    formatted_truncate_for_model(content, policy, "")
}

/// Truncate tool output against the resolved model's tokenizer. An empty or
/// unknown model uses [`TokenService`]'s documented `cl100k_base` fallback.
#[must_use]
pub fn formatted_truncate_for_model(
    content: &str,
    policy: TruncationPolicy,
    model: &str,
) -> String {
    let original_token_count = TokenService::count(model, content);
    let total_lines = content.lines().count();
    let header = format!(
        "{WARNING_HEADER_PREFIX}{original_token_count})\nTotal output lines: {total_lines}\n\n"
    );

    match policy {
        TruncationPolicy::Bytes(budget) => {
            if content.len() <= budget {
                return content.to_string();
            }
            if header.len() >= budget {
                return truncate_at_char_boundary(&header, budget).to_string();
            }
            let body = truncate_middle(content, budget - header.len());
            format!("{header}{body}")
        }
        TruncationPolicy::Tokens(budget) => {
            if original_token_count <= budget {
                return content.to_string();
            }
            truncate_to_token_budget(content, &header, budget, model)
        }
    }
}

fn truncate_to_token_budget(content: &str, header: &str, budget: usize, model: &str) -> String {
    if TokenService::count(model, header) >= budget {
        return prefix_within_token_budget(header, budget, model);
    }

    let mut body_byte_budget = content.len();
    loop {
        let body = truncate_middle(content, body_byte_budget);
        let output = format!("{header}{body}");
        let used = TokenService::count(model, &output);
        if used <= budget {
            return output;
        }

        let shrink = used.saturating_sub(budget).saturating_mul(4).max(1);
        let next = body_byte_budget.saturating_sub(shrink);
        if next == body_byte_budget {
            return prefix_within_token_budget(header, budget, model);
        }
        body_byte_budget = next;
    }
}

fn prefix_within_token_budget(content: &str, budget: usize, model: &str) -> String {
    if budget == 0 {
        return String::new();
    }

    let mut byte_budget = content.len();
    loop {
        let candidate = truncate_at_char_boundary(content, byte_budget);
        let used = TokenService::count(model, candidate);
        if used <= budget {
            return candidate.to_string();
        }
        let shrink = used.saturating_sub(budget).saturating_mul(4).max(1);
        byte_budget = byte_budget.saturating_sub(shrink);
    }
}

/// Keep the head and tail of `content` within `byte_budget` bytes, replacing
/// the middle with a marker that states how many bytes were removed. Cuts land
/// on character boundaries so the result is always valid UTF-8.
///
/// # Examples
///
/// ```
/// use universal_agent_runtime::uar::runtime::context::truncate::truncate_middle;
/// let s: String = (0..100).map(|i| format!("{i}\n")).collect();
/// let t = truncate_middle(&s, 60);
/// assert!(t.len() <= 60);
/// assert!(t.starts_with("0\n"));
/// assert!(t.ends_with("99\n"));
/// ```
#[must_use]
pub fn truncate_middle(content: &str, byte_budget: usize) -> String {
    if content.len() <= byte_budget {
        return content.to_string();
    }
    // Reserve space for the marker; the byte count inside it has at most as
    // many digits as the content length.
    let marker_width = CUT_MARKER_OPEN.len() + CUT_MARKER_CLOSE.len() + digits(content.len());
    if byte_budget <= marker_width {
        return truncate_at_char_boundary(content, byte_budget).to_string();
    }
    let keep = byte_budget - marker_width;
    let head_len = keep / 2;
    let tail_len = keep - head_len;

    let head = truncate_at_char_boundary(content, head_len);
    let tail_start = ceil_char_boundary(content, content.len() - tail_len);
    let tail = &content[tail_start..];
    let removed = content.len() - head.len() - tail.len();

    format!("{head}{CUT_MARKER_OPEN}{removed}{CUT_MARKER_CLOSE}{tail}")
}

fn digits(n: usize) -> usize {
    n.to_string().len()
}

/// Longest prefix of `s` that is at most `max` bytes and ends on a char boundary.
fn truncate_at_char_boundary(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    let mut end = max;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// Smallest char boundary at or after `index`.
fn ceil_char_boundary(s: &str, index: usize) -> usize {
    let mut i = index.min(s.len());
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn within_budget_is_unchanged() {
        assert_eq!(formatted_truncate("hi", TruncationPolicy::Bytes(10)), "hi");
        assert_eq!(formatted_truncate("hi", TruncationPolicy::Tokens(1)), "hi");
    }

    #[test]
    fn result_never_exceeds_budget_and_keeps_head_and_tail() {
        let content: String = (0..2_000).map(|i| format!("row-{i}\n")).collect();
        let out = formatted_truncate(&content, TruncationPolicy::Bytes(700));
        assert!(out.len() <= 700, "{}", out.len());
        assert!(out.starts_with(WARNING_HEADER_PREFIX));
        assert!(out.contains("Total output lines: 2000"));
        assert!(out.contains("row-0\n"));
        assert!(out.ends_with("row-1999\n"));
        assert!(out.contains("bytes truncated"));
    }

    #[test]
    fn cuts_respect_utf8_boundaries() {
        let content = "🦀".repeat(1_000);
        let out = truncate_middle(&content, 100);
        assert!(out.len() <= 100);
        assert!(std::str::from_utf8(out.as_bytes()).is_ok());
        assert!(out.starts_with('🦀'));
        assert!(out.ends_with('🦀'));
    }

    #[test]
    fn tiny_budget_still_returns_valid_prefix() {
        let out = formatted_truncate(&"a".repeat(100), TruncationPolicy::Bytes(5));
        assert!(out.len() <= 5);
    }
}
