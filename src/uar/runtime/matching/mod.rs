pub mod intent;
pub mod tag;
#[cfg(feature = "local-models")]
pub mod vector;
#[cfg(not(feature = "local-models"))]
#[path = "vector_disabled.rs"]
pub mod vector;

pub use intent::{
    ClassificationResult, ClassifierBackend, ClassifierConfig, HybridClassifier, IntentClassifier,
    IntentScore, create_classifier,
};
pub use tag::TagMatcher;
pub use vector::VectorMatcher;

/// Cosine similarity between two vectors.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}
