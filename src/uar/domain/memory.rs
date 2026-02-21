//! Agent memory domain types — re-exported from the `surreal-memory` library.
//!
//! This module uses the full-featured types from `surreal_memory` rather than
//! a lightweight stub, providing proper multi-scope support, version history,
//! and semantic deduplication metadata.

pub use surreal_memory::storage::MemoryStorage;
pub use surreal_memory::{Memory, MemoryHistory, MemoryScope, MemoryType};

/// Lightweight search result wrapper for UAR internal use.
#[derive(Debug, Clone)]
pub struct MemoryMatch {
    pub memory: Memory,
    pub score: f32,
}
