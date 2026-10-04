//! Chunking must not cut inside a version number. A chunk that ends at
//! "v0." or "Obsidian 1." scores highly on a version question and pushes the
//! chunk that carries the full version out of the results.

use async_trait::async_trait;
use std::sync::Arc;
use universal_agent_runtime::uar::rag::chunking::{Chunker, ChunkingStrategy};
use universal_agent_runtime::uar::rag::embeddings::{EmbeddingBackend, EmbeddingError};

const DIM: usize = 64;

/// Gives every text its own axis, so no two sentences are similar and the
/// semantic chunker never merges them. Its output is then the sentence split.
#[derive(Debug)]
struct OrthogonalBackend;

#[async_trait]
impl EmbeddingBackend for OrthogonalBackend {
    fn backend_name(&self) -> &str {
        "orthogonal"
    }

    fn vector_dimension(&self) -> usize {
        DIM
    }

    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        Ok((0..texts.len())
            .map(|i| {
                let mut v = vec![0.0; DIM];
                v[i % DIM] = 1.0;
                v
            })
            .collect())
    }
}

const TEXT: &str = "IPFS Sync for Obsidian is at v0.2.0. It needs Obsidian 1.12.3 or later. \
The plugin syncs notes between devices. See the changelog for details.";

async fn chunks_for(strategy: ChunkingStrategy) -> Vec<String> {
    Chunker::new(strategy, Some(Arc::new(OrthogonalBackend)))
        .chunk(TEXT)
        .await
        .unwrap()
}

fn assert_versions_intact(strategy_name: &str, chunks: &[String]) {
    for fragment in ["v0.", "Obsidian 1."] {
        assert!(
            chunks.iter().all(|c| !c.ends_with(fragment)),
            "{strategy_name} cut a chunk at {fragment:?}: {chunks:#?}"
        );
    }
    for version in ["v0.2.0", "1.12.3"] {
        assert!(
            chunks.iter().any(|c| c.contains(version)),
            "{strategy_name} lost {version:?}: {chunks:#?}"
        );
    }
}

#[tokio::test]
async fn recursive_default_keeps_version_numbers_intact() {
    let chunks = chunks_for(ChunkingStrategy::Recursive { size: 512 }).await;

    assert_versions_intact("Recursive{512}", &chunks);
}

#[tokio::test]
async fn recursive_with_a_small_size_keeps_version_numbers_intact() {
    let chunks = chunks_for(ChunkingStrategy::Recursive { size: 40 }).await;

    assert_versions_intact("Recursive{40}", &chunks);
}

#[tokio::test]
async fn sentence_strategy_keeps_version_numbers_intact() {
    let chunks = chunks_for(ChunkingStrategy::Sentence).await;

    assert_versions_intact("Sentence", &chunks);
}

#[tokio::test]
async fn semantic_strategy_keeps_version_numbers_intact() {
    let chunks = chunks_for(ChunkingStrategy::Semantic { threshold: 0.5 }).await;

    assert_versions_intact("Semantic", &chunks);
}

#[tokio::test]
async fn sentence_strategy_still_splits_real_sentences() {
    let chunks = chunks_for(ChunkingStrategy::Sentence).await;

    assert_eq!(
        chunks,
        vec![
            "IPFS Sync for Obsidian is at v0.2.0.",
            "It needs Obsidian 1.12.3 or later.",
            "The plugin syncs notes between devices.",
            "See the changelog for details.",
        ]
    );
}

#[tokio::test]
async fn sentence_strategy_keeps_a_list_marker_with_its_item() {
    let chunks = Chunker::new(ChunkingStrategy::Sentence, None)
        .chunk("1. Install the plugin. 2. Restart Obsidian.\nIs it synced? Yes! Done")
        .await
        .unwrap();

    assert_eq!(
        chunks,
        vec![
            "1. Install the plugin.",
            "2. Restart Obsidian.",
            "Is it synced?",
            "Yes!",
            "Done",
        ]
    );
}

#[tokio::test]
async fn semantic_strategy_also_breaks_at_newlines() {
    let chunks = Chunker::new(
        ChunkingStrategy::Semantic { threshold: 0.5 },
        Some(Arc::new(OrthogonalBackend)),
    )
    .chunk("Title line\nBody at v0.2.0.")
    .await
    .unwrap();

    assert_eq!(chunks, vec!["Title line", "Body at v0.2.0."]);
}
