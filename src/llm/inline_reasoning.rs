//! Inline model reasoning inside ordinary reply text.
//!
//! Some OpenAI-compatible models (for example MiniMax-M3 through the Liter
//! gateway) write reasoning into `content` as `<think>…</think>` instead of a
//! separate reasoning field, so it arrives as ordinary message text. Callers
//! that need the answer itself — exact registered-agent selection and a
//! delegated child's contribution — split it out here.
//!
//! Rules: `<think>` and `<thinking>` tags match case-insensitively and may
//! carry attributes; closed blocks are removed; an unclosed opening tag runs
//! to the end and counts as reasoning; a closing tag before any opening tag
//! (a chat template that injected the opening tag) makes everything before it
//! reasoning.
//!
//! Limits: a literal tag inside a genuine answer is treated as reasoning, and
//! text after an unclosed tag is reasoning even when it was joined from a
//! later step.

const TAG: &str = "think";
const TAG_SUFFIX: &str = "ing";

/// A reply split into its answer and whether reasoning was present.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InlineReasoning {
    /// Reply text with every reasoning region removed, trimmed.
    pub(crate) answer: String,
    /// Whether any reasoning region (closed, unclosed or orphan) was found.
    pub(crate) had_reasoning: bool,
    /// Whether the reply ended inside an unclosed reasoning block.
    pub(crate) unclosed: bool,
}

/// Split inline `<think>`/`<thinking>` reasoning from reply text.
pub(crate) fn split_inline_reasoning(text: &str) -> InlineReasoning {
    // ASCII lowercasing keeps byte offsets, and every tag boundary is ASCII,
    // so offsets found in `lower` are valid char boundaries in `text`.
    let lower = text.to_ascii_lowercase();
    let mut answer = String::new();
    let mut had_reasoning = false;
    let mut cursor = 0;

    if let Some((_, close_end)) = find_close(&lower, 0)
        && find_open(&lower, 0).is_none_or(|(open_start, _)| open_start > close_end)
    {
        had_reasoning = true;
        cursor = close_end;
    }

    loop {
        let Some((open_start, open_end)) = find_open(&lower, cursor) else {
            answer.push_str(&text[cursor..]);
            break;
        };
        answer.push_str(&text[cursor..open_start]);
        had_reasoning = true;
        match open_end.and_then(|end| find_close(&lower, end)) {
            Some((_, close_end)) => cursor = close_end,
            None => {
                return InlineReasoning {
                    answer: answer.trim().to_owned(),
                    had_reasoning,
                    unclosed: true,
                };
            }
        }
    }

    InlineReasoning {
        answer: answer.trim().to_owned(),
        had_reasoning,
        unclosed: false,
    }
}

/// Find an opening tag at or after `from`. Returns its start and the offset
/// just past its `>`, or `None` for the end when the tag itself is unfinished.
fn find_open(lower: &str, from: usize) -> Option<(usize, Option<usize>)> {
    let mut search = from;
    while let Some(relative) = lower[search..].find("<think") {
        let start = search + relative;
        let mut name_end = start + 1 + TAG.len();
        if lower[name_end..].starts_with(TAG_SUFFIX) {
            name_end += TAG_SUFFIX.len();
        }
        match lower[name_end..].chars().next() {
            Some('>') => return Some((start, Some(name_end + 1))),
            Some(c) if c.is_whitespace() => {
                let end = lower[name_end..].find('>').map(|gt| name_end + gt + 1);
                return Some((start, end));
            }
            None => return Some((start, None)),
            Some(_) => search = start + 1,
        }
    }
    None
}

/// Find a closing tag at or after `from`. Returns its start and the offset
/// just past its `>`.
fn find_close(lower: &str, from: usize) -> Option<(usize, usize)> {
    let mut search = from;
    while let Some(relative) = lower[search..].find("</think") {
        let start = search + relative;
        let mut name_end = start + 2 + TAG.len();
        if lower[name_end..].starts_with(TAG_SUFFIX) {
            name_end += TAG_SUFFIX.len();
        }
        let rest = &lower[name_end..];
        let trimmed = rest.trim_start();
        if trimmed.starts_with('>') {
            return Some((start, name_end + (rest.len() - trimmed.len()) + 1));
        }
        search = start + 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split(text: &str) -> (String, bool, bool) {
        let result = split_inline_reasoning(text);
        (result.answer, result.had_reasoning, result.unclosed)
    }

    #[test]
    fn plain_reply_is_unchanged() {
        assert_eq!(split("  math-specialist \n"), ("math-specialist".into(), false, false));
    }

    #[test]
    fn closed_block_before_answer_is_removed() {
        assert_eq!(
            split("<think>\nThe user asks about ferns.\n</think>\n\nplant-clinic"),
            ("plant-clinic".into(), true, false)
        );
    }

    #[test]
    fn tags_match_case_insensitively_with_attributes_and_long_name() {
        assert_eq!(split("<THINK>a</Think>x"), ("x".into(), true, false));
        assert_eq!(split("<think mode=\"brief\">a</think >x"), ("x".into(), true, false));
        assert_eq!(split("<thinking>a</thinking>x"), ("x".into(), true, false));
    }

    #[test]
    fn unclosed_block_runs_to_the_end() {
        assert_eq!(
            split("<think>Drafting: Fernwell Plant Clinic SIGNATURE"),
            (String::new(), true, true)
        );
        assert_eq!(split("Answer first. <think>then reasoning"), ("Answer first.".into(), true, true));
    }

    #[test]
    fn unfinished_opening_tag_counts_as_unclosed() {
        assert_eq!(split("answer <think"), ("answer".into(), true, true));
        assert_eq!(split("answer <think attr"), ("answer".into(), true, true));
    }

    #[test]
    fn orphan_closing_tag_marks_template_injected_reasoning() {
        assert_eq!(split("reasoning only here</think>\n\nwriter"), ("writer".into(), true, false));
    }

    #[test]
    fn multiple_blocks_and_text_between_are_kept_in_order() {
        assert_eq!(split("<think>a</think>one <think>b</think>two"), ("one two".into(), true, false));
    }

    #[test]
    fn similar_tag_names_are_not_reasoning() {
        assert_eq!(split("<thinker>keep</thinker>"), ("<thinker>keep</thinker>".into(), false, false));
    }

    #[test]
    fn empty_and_reasoning_only_replies() {
        assert_eq!(split(""), (String::new(), false, false));
        assert_eq!(split("<think>only</think>"), (String::new(), true, false));
    }

    #[test]
    fn non_ascii_text_keeps_boundaries() {
        assert_eq!(split("<think>é</think>Chèvre ✓"), ("Chèvre ✓".into(), true, false));
    }
}
