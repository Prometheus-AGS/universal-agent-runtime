//! Numbered citation markers (`[1]`, `[2]`, ...) for RAG-retrieved knowledge
//! chunks, threaded through the existing normalized SSE event model
//! ([`crate::uar::domain::events::NormalizedEvent::RagCitations`]).
//!
//! [`CitationStream`] is the single reusable seam between "we retrieved some
//! chunks" and "the client can render `[1]`/`[2]` markers with a
//! hover-to-source panel": it assigns stable 1-based marker numbers to
//! retrieved chunks, renders the numbered block that gets injected into the
//! system prompt (so the model has a reason to emit `[1]`, `[2]` in its
//! answer), and converts to the wire event that carries those markers to the
//! client.
//!
//! This type intentionally carries no transport or UI concerns — it is built
//! from retrieval results ([`KnowledgeMatch`] or [`RAGContext`]) and produces
//! plain, serializable data ([`RagCitation`]). That keeps it reusable by
//! future consumers beyond the chat SSE stream (e.g. the A2UI surfaces in
//! later grade-A changes, or an eval harness that wants a run's citation set
//! without going through SSE at all).

use std::collections::HashMap;

use crate::uar::domain::events::{NormalizedEvent, RagCitation};
use crate::uar::domain::graph::{Citation, RAGContext};
use crate::uar::domain::knowledge::KnowledgeMatch;

/// Maximum characters kept from a chunk's content when building the snippet
/// shown in the hover-to-source panel and injected into the prompt block.
const SNIPPET_CHARS: usize = 240;

/// Ordered, numbered set of citation markers for a single RAG-augmented run.
///
/// Markers are assigned in the order chunks are supplied (first chunk is
/// `[1]`, second is `[2]`, ...), matching the order they're presented to the
/// model in the prompt block, so a model that cites `[1]` is citing the
/// first-listed (typically highest-relevance) chunk.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CitationStream {
    markers: Vec<RagCitation>,
}

impl CitationStream {
    /// An empty citation stream (no chunks retrieved / RAG disabled).
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Build a numbered citation stream from raw retrieval matches.
    ///
    /// `document_names` maps `document_id -> human-readable name`; chunks
    /// whose `document_id` is absent or unresolved fall back to the chunk id.
    #[must_use]
    pub fn from_matches(
        matches: &[KnowledgeMatch],
        document_names: &HashMap<String, String>,
    ) -> Self {
        let markers = matches
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let chunk_id = m.chunk.id.to_string();
                let document_name = m
                    .chunk
                    .document_id
                    .as_ref()
                    .and_then(|did| document_names.get(did))
                    .cloned()
                    .unwrap_or_else(|| {
                        m.chunk
                            .document_id
                            .clone()
                            .unwrap_or_else(|| chunk_id.clone())
                    });
                RagCitation {
                    marker: i + 1,
                    chunk_id,
                    document_id: m.chunk.document_id.clone(),
                    document_name,
                    relevance_score: m.score,
                    snippet: truncate_snippet(&m.chunk.content),
                }
            })
            .collect();
        Self { markers }
    }

    /// Build a numbered citation stream from an already-assembled
    /// [`RAGContext`] (e.g. from
    /// [`super::retrieval::HybridRetriever::build_context`]), preserving its
    /// existing citation ordering and reusing its precomputed snippet /
    /// document name.
    #[must_use]
    pub fn from_context(context: &RAGContext) -> Self {
        Self::from_citations(&context.citations)
    }

    /// Build a numbered citation stream from a list of domain [`Citation`]s,
    /// assigning markers in list order.
    #[must_use]
    pub fn from_citations(citations: &[Citation]) -> Self {
        let markers = citations
            .iter()
            .enumerate()
            .map(|(i, c)| RagCitation {
                marker: i + 1,
                chunk_id: c.chunk_id.clone(),
                document_id: c.document_id.clone(),
                document_name: c.document_name.clone(),
                relevance_score: c.relevance_score,
                snippet: truncate_snippet(&c.snippet),
            })
            .collect();
        Self { markers }
    }

    /// Whether any markers were retrieved.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.markers.is_empty()
    }

    /// Number of markers in this stream.
    #[must_use]
    pub fn len(&self) -> usize {
        self.markers.len()
    }

    /// The ordered markers, e.g. for building an eval/A2UI payload directly.
    #[must_use]
    pub fn markers(&self) -> &[RagCitation] {
        &self.markers
    }

    /// Render a `[1] ...\n[2] ...` block suitable for injection into the
    /// system prompt, so the model has numbered sources to cite back against.
    /// Returns an empty string when there are no markers.
    #[must_use]
    pub fn prompt_block(&self) -> String {
        if self.markers.is_empty() {
            return String::new();
        }
        let mut block = String::from("\n\n[RELEVANT KNOWLEDGE]\n");
        block.push_str(
            "Cite sources you use with their bracketed number, e.g. `[1]`, matching the list below.\n",
        );
        for m in &self.markers {
            block.push_str(&format!(
                "[{}] ({}) {}\n",
                m.marker, m.document_name, m.snippet
            ));
        }
        block
    }

    /// Convert to the SSE-facing [`NormalizedEvent::RagCitations`] event, or
    /// `None` when there are no markers (nothing worth emitting).
    #[must_use]
    pub fn to_normalized_event(&self, run_id: impl Into<String>) -> Option<NormalizedEvent> {
        if self.markers.is_empty() {
            return None;
        }
        Some(NormalizedEvent::RagCitations {
            run_id: run_id.into(),
            citations: self.markers.clone(),
        })
    }
}

fn truncate_snippet(content: &str) -> String {
    content.chars().take(SNIPPET_CHARS).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uar::domain::knowledge::KnowledgeChunk;
    use uuid::Uuid;

    fn make_match(document_id: Option<&str>, content: &str, score: f32) -> KnowledgeMatch {
        KnowledgeMatch {
            chunk: KnowledgeChunk {
                id: Uuid::new_v4(),
                kb_id: "kb-1".to_string(),
                document_id: document_id.map(str::to_string),
                content: content.to_string(),
                metadata: None,
                embedding: Vec::new(),
                created_at: "2026-01-01T00:00:00Z".to_string(),
            },
            score,
        }
    }

    #[test]
    fn empty_matches_produce_empty_stream() {
        let stream = CitationStream::from_matches(&[], &HashMap::new());
        assert!(stream.is_empty());
        assert_eq!(stream.len(), 0);
        assert_eq!(stream.prompt_block(), "");
        assert!(stream.to_normalized_event("run-1").is_none());
    }

    #[test]
    fn markers_are_1_based_and_in_order() {
        let matches = vec![
            make_match(Some("doc-a"), "first chunk content", 0.9),
            make_match(Some("doc-b"), "second chunk content", 0.5),
        ];
        let mut names = HashMap::new();
        names.insert("doc-a".to_string(), "Doc A.pdf".to_string());
        names.insert("doc-b".to_string(), "Doc B.pdf".to_string());

        let stream = CitationStream::from_matches(&matches, &names);
        assert_eq!(stream.len(), 2);
        assert_eq!(stream.markers()[0].marker, 1);
        assert_eq!(stream.markers()[0].document_name, "Doc A.pdf");
        assert_eq!(stream.markers()[1].marker, 2);
        assert_eq!(stream.markers()[1].document_name, "Doc B.pdf");
    }

    #[test]
    fn unresolved_document_name_falls_back_to_document_id() {
        let matches = vec![make_match(Some("doc-unknown"), "content", 0.7)];
        let stream = CitationStream::from_matches(&matches, &HashMap::new());
        assert_eq!(stream.markers()[0].document_name, "doc-unknown");
    }

    #[test]
    fn missing_document_id_falls_back_to_chunk_id() {
        let matches = vec![make_match(None, "content", 0.7)];
        let stream = CitationStream::from_matches(&matches, &HashMap::new());
        let marker = &stream.markers()[0];
        assert_eq!(marker.document_name, marker.chunk_id);
        assert!(marker.document_id.is_none());
    }

    #[test]
    fn prompt_block_contains_numbered_markers() {
        let matches = vec![
            make_match(Some("doc-a"), "alpha content", 0.9),
            make_match(Some("doc-b"), "beta content", 0.5),
        ];
        let stream = CitationStream::from_matches(&matches, &HashMap::new());
        let block = stream.prompt_block();
        assert!(block.contains("[1]"));
        assert!(block.contains("[2]"));
        assert!(block.contains("alpha content"));
        assert!(block.contains("beta content"));
    }

    #[test]
    fn snippet_is_truncated_to_max_chars() {
        let long_content = "a".repeat(1000);
        let matches = vec![make_match(Some("doc-a"), &long_content, 0.9)];
        let stream = CitationStream::from_matches(&matches, &HashMap::new());
        assert_eq!(stream.markers()[0].snippet.chars().count(), SNIPPET_CHARS);
    }

    #[test]
    fn to_normalized_event_carries_run_id_and_markers() {
        let matches = vec![make_match(Some("doc-a"), "content", 0.9)];
        let stream = CitationStream::from_matches(&matches, &HashMap::new());
        let event = stream
            .to_normalized_event("run-42")
            .expect("non-empty stream emits an event");
        match event {
            NormalizedEvent::RagCitations { run_id, citations } => {
                assert_eq!(run_id, "run-42");
                assert_eq!(citations.len(), 1);
                assert_eq!(citations[0].marker, 1);
            }
            other => panic!("expected RagCitations event, got {other:?}"),
        }
    }

    #[test]
    fn from_context_preserves_citation_order_and_fields() {
        let context = RAGContext {
            citations: vec![
                Citation {
                    chunk_id: "chunk-1".to_string(),
                    document_id: Some("doc-1".to_string()),
                    document_name: "Doc One".to_string(),
                    relevance_score: 0.8,
                    snippet: "snippet one".to_string(),
                    entities_mentioned: Vec::new(),
                },
                Citation {
                    chunk_id: "chunk-2".to_string(),
                    document_id: Some("doc-2".to_string()),
                    document_name: "Doc Two".to_string(),
                    relevance_score: 0.6,
                    snippet: "snippet two".to_string(),
                    entities_mentioned: Vec::new(),
                },
            ],
            ..Default::default()
        };
        let stream = CitationStream::from_context(&context);
        assert_eq!(stream.len(), 2);
        assert_eq!(stream.markers()[0].marker, 1);
        assert_eq!(stream.markers()[0].chunk_id, "chunk-1");
        assert_eq!(stream.markers()[1].marker, 2);
        assert_eq!(stream.markers()[1].chunk_id, "chunk-2");
    }
}
