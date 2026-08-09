//! Real embedding-inference tests. These require a local backend: without
//! `local-models`, `build_backend` falls through to `openai`, which needs an
//! API key and would panic rather than skip. The whole file is gated so it
//! still compiles under every profile.
#![cfg(feature = "local-models")]

use universal_agent_runtime::uar::rag::embeddings::EmbeddingConfig;
use universal_agent_runtime::uar::runtime::matching::vector::VectorMatcher;

fn make_matcher(threshold: f32) -> VectorMatcher {
    let config = EmbeddingConfig {
        models_dir: "src/uar/runtime/matching/models".to_string(),
        ..Default::default()
    };
    VectorMatcher::from_config(&config, threshold)
        .expect("VectorMatcher should build from default embedding config")
}

#[tokio::test]
async fn test_burn_embedding_initialization() {
    let matcher = make_matcher(0.7);
    // Initialize (should trigger model loading)
    let res = matcher.initialize().await;
    assert!(
        res.is_ok(),
        "Failed to initialize BurnVectorMatcher: {:?}",
        res.err()
    );
}

#[tokio::test]
async fn test_burn_embedding_shape() {
    let matcher = make_matcher(0.7);
    matcher.initialize().await.expect("Initialization failed");

    let texts = vec!["Hello world".to_string(), "Burn is great".to_string()];
    let embeddings = matcher.embed_batch(texts).await.expect("Embedding failed");

    assert_eq!(
        embeddings.len(),
        2,
        "Should return embeddings for all inputs"
    );
    assert_eq!(
        embeddings[0].len(),
        384,
        "Embedding dimension should be 384 for BG-Small-En-V1.5"
    );
}
