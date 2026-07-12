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
