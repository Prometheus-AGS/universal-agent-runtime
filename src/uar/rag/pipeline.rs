//! RAG retrieval pipeline (CH-11 rag-hardening): decomposition → per-subquery
//! search → dedup/fuse → verification → audit event.
//!
//! Ties together [`super::decomposition::QueryDecomposer`] and
//! [`super::verification::verify`] around a caller-supplied
//! [`RetrievalBackend`], so this module has zero direct dependency on the
//! persistence layer or the embedding model. That's the seam plan.md asks
//! for: extracting RAG to a separate Knowledge Service later means moving
//! this pipeline behind an MCP boundary and swapping the backend
//! implementation — the pipeline logic itself does not change.

use std::collections::HashMap;

use async_trait::async_trait;
use uuid::Uuid;

use crate::uar::domain::knowledge::KnowledgeMatch;

use super::decomposition::{HeuristicDecomposer, QueryDecomposer};
use super::verification::{VerificationVerdict, verify};

/// Executes a single sub-query search against whatever backs actual
/// retrieval (embedding + vector/graph search). Implemented by callers —
/// this crate has no opinion on persistence or embedding models.
#[async_trait]
pub trait RetrievalBackend: Send + Sync {
    /// Run one search for `sub_query` and return scored matches.
    async fn search_one(
        &self,
        sub_query: &str,
        limit: usize,
        min_score: f32,
    ) -> anyhow::Result<Vec<KnowledgeMatch>>;
}

/// Behavior knobs for the retrieval pipeline.
#[derive(Debug, Clone)]
pub struct RagRetrievalConfig {
    /// When true, chunks that fail the verification pass (share no content
    /// terms with the sub-query that retrieved them) are dropped from the
    /// results entirely. Default `false`: verification only ANNOTATES the
    /// audit event, it does not change what's returned — preserves existing
    /// caller-observable behavior (Rule 32) while still surfacing the signal
    /// for callers/operators who want to opt into stricter filtering.
    pub drop_uncorroborated: bool,
}

impl Default for RagRetrievalConfig {
    fn default() -> Self {
        Self {
            drop_uncorroborated: false,
        }
    }
}

/// A retrieved chunk with pipeline-added provenance.
#[derive(Debug, Clone)]
struct PipelineChunk {
    m: KnowledgeMatch,
    sub_query: String,
    verdict: VerificationVerdict,
}

/// Decomposition → search → dedup → verification → audit pipeline.
///
/// Generic over the decomposer so a future LLM-based one can be substituted
/// without changing this type's logic (only [`Self::with_decomposer`]'s
/// argument changes at the call site).
#[derive(Debug)]
pub struct RagRetrievalPipeline<D: QueryDecomposer = HeuristicDecomposer> {
    decomposer: D,
    config: RagRetrievalConfig,
}

impl RagRetrievalPipeline<HeuristicDecomposer> {
    /// Default pipeline: heuristic decomposition, verification-annotate-only.
    #[must_use]
    pub fn new() -> Self {
        Self {
            decomposer: HeuristicDecomposer,
            config: RagRetrievalConfig::default(),
        }
    }
}

impl Default for RagRetrievalPipeline<HeuristicDecomposer> {
    fn default() -> Self {
        Self::new()
    }
}

impl<D: QueryDecomposer> RagRetrievalPipeline<D> {
    /// Build a pipeline with a custom decomposer and/or config.
    pub fn with_decomposer(decomposer: D, config: RagRetrievalConfig) -> Self {
        Self { decomposer, config }
    }

    /// Run the full pipeline for `query` against `kb_id`, using `backend`
    /// for the actual per-sub-query search.
    ///
    /// # Errors
    ///
    /// Propagates the first sub-query search error from `backend`.
    pub async fn retrieve(
        &self,
        backend: &dyn RetrievalBackend,
        kb_id: &str,
        query: &str,
        limit: usize,
        min_score: f32,
    ) -> anyhow::Result<Vec<KnowledgeMatch>> {
        let sub_queries = self.decomposer.decompose(query);

        let mut by_chunk_id: HashMap<Uuid, PipelineChunk> = HashMap::new();
        for sub_query in &sub_queries {
            let matches = backend.search_one(sub_query, limit, min_score).await?;
            for m in matches {
                let verdict = verify(sub_query, &m.chunk.content);
                by_chunk_id
                    .entry(m.chunk.id)
                    .and_modify(|existing| {
                        // Same chunk surfaced by multiple sub-queries — keep
                        // whichever hit scored it higher.
                        if m.score > existing.m.score {
                            existing.m = m.clone();
                            existing.sub_query = sub_query.clone();
                            existing.verdict = verdict;
                        }
                    })
                    .or_insert_with(|| PipelineChunk {
                        m,
                        sub_query: sub_query.clone(),
                        verdict,
                    });
            }
        }

        let candidates_before_dedup: usize = by_chunk_id.len();
        let uncorroborated_count = by_chunk_id
            .values()
            .filter(|c| c.verdict == VerificationVerdict::Uncorroborated)
            .count();

        let mut results: Vec<PipelineChunk> = by_chunk_id.into_values().collect();
        if self.config.drop_uncorroborated {
            results.retain(|c| c.verdict == VerificationVerdict::Corroborated);
        }
        results.sort_by(|a, b| {
            b.m.score
                .partial_cmp(&a.m.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);

        // Audit trail (Rule 34): what was retrieved and why, for this query.
        tracing::info!(
            name: "rag.retrieval.decision",
            kb_id = kb_id,
            query = query,
            sub_query_count = sub_queries.len(),
            candidates_before_dedup = candidates_before_dedup,
            uncorroborated_count = uncorroborated_count,
            drop_uncorroborated = self.config.drop_uncorroborated,
            returned_count = results.len(),
            chunk_ids = %results
                .iter()
                .map(|c| c.m.chunk.id.to_string())
                .collect::<Vec<_>>()
                .join(","),
            "RAG retrieval pipeline decision"
        );

        Ok(results.into_iter().map(|c| c.m).collect())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fmt::Write as _,
        sync::{Arc, Mutex},
    };

    use super::*;
    use crate::uar::domain::knowledge::KnowledgeChunk;
    use tracing::{
        Subscriber,
        field::{Field, Visit},
        instrument::WithSubscriber,
    };
    use tracing_subscriber::{Layer, layer::Context, prelude::*};

    #[derive(Clone, Default)]
    struct CapturedEvents(Arc<Mutex<Vec<(String, String)>>>);

    #[derive(Default)]
    struct CapturedFields(String);

    impl Visit for CapturedFields {
        fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
            let _ = write!(&mut self.0, "{}={value:?} ", field.name());
        }
    }

    impl<S: Subscriber> Layer<S> for CapturedEvents {
        fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
            let mut fields = CapturedFields::default();
            event.record(&mut fields);
            self.0
                .lock()
                .expect("capture lock")
                .push((event.metadata().name().to_string(), fields.0));
        }
    }

    fn make_match(id: Uuid, content: &str, score: f32) -> KnowledgeMatch {
        KnowledgeMatch {
            chunk: KnowledgeChunk {
                id,
                owner_id: "anonymous".to_string(),
                kb_id: "test-kb".to_string(),
                document_id: None,
                content: content.to_string(),
                metadata: None,
                embedding: Vec::new(),
                created_at: "2026-01-01T00:00:00Z".to_string(),
            },
            score,
        }
    }

    /// Backend that returns a fixed set of matches per sub-query, so pipeline
    /// logic (dedup, verification, truncation) is testable without a real
    /// embedder/persistence layer.
    struct FixedBackend {
        by_query: HashMap<String, Vec<KnowledgeMatch>>,
    }

    #[async_trait]
    impl RetrievalBackend for FixedBackend {
        async fn search_one(
            &self,
            sub_query: &str,
            limit: usize,
            _min_score: f32,
        ) -> anyhow::Result<Vec<KnowledgeMatch>> {
            Ok(self
                .by_query
                .get(sub_query)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .take(limit)
                .collect())
        }
    }

    #[tokio::test]
    async fn single_query_passthrough_matches_backend() {
        let id = Uuid::new_v4();
        let backend = FixedBackend {
            by_query: HashMap::from([(
                "compiler validation stages".to_string(),
                vec![make_match(id, "The compiler validates each stage.", 0.9)],
            )]),
        };
        let pipeline = RagRetrievalPipeline::new();
        let results = pipeline
            .retrieve(&backend, "kb-1", "compiler validation stages", 10, 0.0)
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].chunk.id, id);
    }

    #[tokio::test]
    async fn same_chunk_from_two_subqueries_dedupes_keeping_higher_score() {
        let id = Uuid::new_v4();
        let backend = FixedBackend {
            by_query: HashMap::from([
                (
                    "the compiler validates specs".to_string(),
                    vec![make_match(id, "compiler spec validation content", 0.5)],
                ),
                (
                    "signing works how".to_string(),
                    vec![make_match(id, "compiler spec validation content", 0.9)],
                ),
            ]),
        };
        let pipeline = RagRetrievalPipeline::new();
        let results = pipeline
            .retrieve(
                &backend,
                "kb-1",
                "the compiler validates specs and also signing works how",
                10,
                0.0,
            )
            .await
            .unwrap();
        assert_eq!(results.len(), 1, "duplicate chunk must be deduped");
        assert!((results[0].score - 0.9).abs() < f32::EPSILON);
    }

    #[tokio::test]
    async fn result_limit_applies_after_cross_query_scoring() {
        let lower_id = Uuid::new_v4();
        let higher_id = Uuid::new_v4();
        let backend = FixedBackend {
            by_query: HashMap::from([
                (
                    "compiler validation stages explained".to_string(),
                    vec![make_match(lower_id, "compiler validation details", 0.7)],
                ),
                (
                    "signing workflow details explained".to_string(),
                    vec![make_match(higher_id, "signing workflow details", 0.9)],
                ),
            ]),
        };
        let results = RagRetrievalPipeline::new()
            .retrieve(
                &backend,
                "kb-1",
                "compiler validation stages explained and also signing workflow details explained",
                1,
                0.0,
            )
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].chunk.id, higher_id);
    }

    #[tokio::test]
    async fn drop_uncorroborated_filters_when_enabled() {
        let related_id = Uuid::new_v4();
        let unrelated_id = Uuid::new_v4();
        let backend = FixedBackend {
            by_query: HashMap::from([(
                "compiler validation".to_string(),
                vec![
                    make_match(related_id, "the compiler validates every spec", 0.9),
                    make_match(unrelated_id, "garden gnomes are ceramic figurines", 0.8),
                ],
            )]),
        };
        let pipeline = RagRetrievalPipeline::with_decomposer(
            HeuristicDecomposer,
            RagRetrievalConfig {
                drop_uncorroborated: true,
            },
        );
        let results = pipeline
            .retrieve(&backend, "kb-1", "compiler validation", 10, 0.0)
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].chunk.id, related_id);
    }

    #[tokio::test]
    async fn uncorroborated_not_dropped_by_default() {
        let related_id = Uuid::new_v4();
        let unrelated_id = Uuid::new_v4();
        let backend = FixedBackend {
            by_query: HashMap::from([(
                "compiler validation".to_string(),
                vec![
                    make_match(related_id, "the compiler validates every spec", 0.9),
                    make_match(unrelated_id, "garden gnomes are ceramic figurines", 0.8),
                ],
            )]),
        };
        let pipeline = RagRetrievalPipeline::new();
        let results = pipeline
            .retrieve(&backend, "kb-1", "compiler validation", 10, 0.0)
            .await
            .unwrap();
        assert_eq!(
            results.len(),
            2,
            "default config must not change existing caller-observable result count"
        );
    }

    #[tokio::test]
    async fn retrieval_emits_decision_audit_event() {
        let id = Uuid::new_v4();
        let backend = FixedBackend {
            by_query: HashMap::from([(
                "compiler validation".to_string(),
                vec![make_match(id, "the compiler validates every spec", 0.9)],
            )]),
        };
        let events = CapturedEvents::default();
        let subscriber = tracing_subscriber::registry().with(events.clone());

        RagRetrievalPipeline::new()
            .retrieve(&backend, "kb-audit", "compiler validation", 3, 0.7)
            .with_subscriber(subscriber)
            .await
            .expect("retrieval succeeds");

        let captured = events.0.lock().expect("capture lock");
        let (_, fields) = captured
            .iter()
            .find(|(name, _)| name == "rag.retrieval.decision")
            .expect("named retrieval decision event");
        assert!(fields.contains("kb_id=\"kb-audit\""));
        assert!(fields.contains("returned_count=1"));
    }
}
