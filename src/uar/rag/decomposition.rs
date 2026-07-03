//! Query decomposition (CH-11 rag-hardening, §5.4 item 1 of docs/uar-next.md).
//!
//! Breaks a user query into sub-queries so a compound/multi-part question
//! retrieves against each part instead of one averaged embedding that
//! blurs all of them together. Heuristic (not LLM-based) per plan.md's D-A
//! decision ("harden in-process" — no new LLM-call dependency for this
//! pass). [`QueryDecomposer`] is the seam: swap in an LLM-based decomposer
//! later without touching callers (`RagRetrievalPipeline` only depends on
//! the trait).

/// Breaks a query into 1+ sub-queries to retrieve against independently.
pub trait QueryDecomposer: std::fmt::Debug + Send + Sync {
    /// Decompose `query`. MUST always return at least one element — the
    /// original query verbatim when no split applies — so simple queries
    /// retrieve exactly as they did before this pipeline existed.
    fn decompose(&self, query: &str) -> Vec<String>;
}

/// Splits on sentence boundaries and coordinating conjunctions when a query
/// looks compound (e.g. "What is X and how does it relate to Y?").
///
/// Deliberately conservative: a query that doesn't match any split pattern
/// is returned unchanged as the sole sub-query, so this never makes a
/// simple query's retrieval worse than before decomposition existed.
#[derive(Debug, Default, Clone, Copy)]
pub struct HeuristicDecomposer;

/// Sub-query joiners this decomposer recognizes, checked in order. Kept
/// lowercase; matching is case-insensitive.
const SPLIT_MARKERS: &[&str] = &[
    " and also ",
    " as well as ",
    ". also, ",
    "; ",
    ". ",
    " and how ",
    " and what ",
    " and why ",
    " and when ",
    " and where ",
];

impl QueryDecomposer for HeuristicDecomposer {
    fn decompose(&self, query: &str) -> Vec<String> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return vec![String::new()];
        }

        let lower = trimmed.to_lowercase();
        for marker in SPLIT_MARKERS {
            if let Some(idx) = lower.find(marker) {
                let (head, tail) = trimmed.split_at(idx);
                let tail = &tail[marker.len()..];
                let sub_queries: Vec<String> = [head, tail]
                    .into_iter()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(ToString::to_string)
                    .collect();
                // Only decompose if BOTH halves are substantive (>3 words) —
                // otherwise a short trailing clause ("...and why?") would
                // become a near-empty, low-signal sub-query that dilutes
                // results rather than sharpening them.
                if sub_queries.len() == 2
                    && sub_queries
                        .iter()
                        .all(|s| s.split_whitespace().count() >= 3)
                {
                    return sub_queries;
                }
            }
        }

        vec![trimmed.to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_query_is_not_split() {
        let d = HeuristicDecomposer;
        assert_eq!(
            d.decompose("What is the capital of France?"),
            vec!["What is the capital of France?".to_string()]
        );
    }

    #[test]
    fn empty_query_returns_single_empty_element() {
        let d = HeuristicDecomposer;
        assert_eq!(d.decompose(""), vec![String::new()]);
        assert_eq!(d.decompose("   "), vec![String::new()]);
    }

    #[test]
    fn compound_query_splits_on_and_also() {
        let d = HeuristicDecomposer;
        let result = d
            .decompose("How does the compiler validate agent specs and also how does signing work");
        assert_eq!(result.len(), 2);
        assert!(result[0].to_lowercase().contains("compiler"));
        assert!(result[1].to_lowercase().contains("signing"));
    }

    #[test]
    fn short_trailing_clause_does_not_force_a_split() {
        let d = HeuristicDecomposer;
        // "and why?" is too short a second half to be a useful sub-query.
        let result = d.decompose("Explain the ingestion pipeline in detail and why?");
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn semicolon_splits_into_two_substantive_clauses() {
        let d = HeuristicDecomposer;
        let result = d.decompose(
            "Describe the chunking strategy in detail; describe the Leiden extraction pipeline",
        );
        assert_eq!(result.len(), 2);
    }
}
