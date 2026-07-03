//! Retrieval verification pass (CH-11 rag-hardening, §5.4 item 2 of docs/uar-next.md).
//!
//! A cheap, in-process corroboration check applied to every retrieved chunk:
//! it must share at least one non-trivial (stopword-filtered) term with the
//! query it was retrieved for. This catches the case where vector search
//! returns a chunk above the score threshold that isn't actually lexically
//! related to the query (embedding drift / an off-topic false positive) —
//! it is NOT the full LLM-based fact-cross-referencing layer docs/uar-next.md
//! §5.4 envisions long-term; that needs a model call and is future work.
//! This is the free first pass that needs no LLM.

use std::collections::HashSet;

/// A minimal, dependency-free English stopword list — just enough to avoid
/// "the"/"a"/"is" trivially satisfying the overlap check on every query.
/// Not exhaustive; false negatives here just mean this pass is slightly more
/// lenient, never more strict, than a full stopword list would be.
const STOPWORDS: &[&str] = &[
    "a", "an", "the", "is", "are", "was", "were", "be", "been", "being", "and", "or", "but", "if",
    "of", "at", "by", "for", "with", "about", "to", "from", "in", "on", "into", "how", "what",
    "why", "when", "where", "who", "which", "does", "do", "did", "it", "its", "this", "that",
    "these", "those",
];

fn content_terms(text: &str) -> HashSet<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .map(str::to_lowercase)
        .filter(|w| w.len() > 2 && !STOPWORDS.contains(&w.as_str()))
        .collect()
}

/// Verdict from the verification pass for a single retrieved chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationVerdict {
    /// Chunk shares at least one content term with the query — corroborated.
    Corroborated,
    /// Chunk shares no content terms with the query — likely an embedding
    /// false positive; the caller decides whether to drop or just flag it.
    Uncorroborated,
}

/// Verify one retrieved chunk against the query that retrieved it.
///
/// Returns [`VerificationVerdict::Corroborated`] whenever either side has no
/// extractable content terms (an all-stopword query, or empty chunk content)
/// — verification abstains rather than penalizing degenerate input, since a
/// false "uncorroborated" would silently drop a possibly-relevant result for
/// a reason unrelated to actual relevance.
#[must_use]
pub fn verify(query: &str, chunk_content: &str) -> VerificationVerdict {
    let query_terms = content_terms(query);
    if query_terms.is_empty() {
        return VerificationVerdict::Corroborated;
    }
    let chunk_terms = content_terms(chunk_content);
    if chunk_terms.is_empty() {
        return VerificationVerdict::Corroborated;
    }

    if query_terms.intersection(&chunk_terms).next().is_some() {
        VerificationVerdict::Corroborated
    } else {
        VerificationVerdict::Uncorroborated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlapping_terms_are_corroborated() {
        assert_eq!(
            verify(
                "how does the compiler validate agent specs",
                "The compiler validates each UAR-AGENT-MD spec against the s01_frontmatter stage."
            ),
            VerificationVerdict::Corroborated
        );
    }

    #[test]
    fn no_overlap_is_uncorroborated() {
        assert_eq!(
            verify(
                "what is the pricing for gpt models",
                "The garden gnome collection features ceramic figurines from the 1980s."
            ),
            VerificationVerdict::Uncorroborated
        );
    }

    #[test]
    fn all_stopword_query_abstains_as_corroborated() {
        assert_eq!(
            verify("is it the", "some unrelated chunk content here"),
            VerificationVerdict::Corroborated
        );
    }

    #[test]
    fn empty_chunk_abstains_as_corroborated() {
        assert_eq!(
            verify("compiler validation stages", ""),
            VerificationVerdict::Corroborated
        );
    }
}
