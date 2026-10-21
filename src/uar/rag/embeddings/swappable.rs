//! Hot-swappable embedding backend handle.
//!
//! The runtime shares ONE embedding backend between the vector matcher
//! (queries), the ingest service (writes), and diagnostics. Persisted
//! `llm.embedding.*` settings are authoritative after first boot (ADR-010), so
//! a REST mutation must take effect on the next embed call without a restart —
//! which the previously fixed `Arc<dyn EmbeddingBackend>` cannot do.
//!
//! Coherence design:
//!
//! * Backend and metadata live together in ONE [`BackendSlot`]; a swap
//!   publishes the slot with a single atomic store. There is no window in
//!   which a reader can see one backend's metadata with another backend's
//!   vectors.
//! * Jobs PIN a slot (`pin()`) and use that one snapshot for the whole job —
//!   fingerprint check, chunking embeds, and document embeds. A concurrent
//!   swap therefore cannot interleave inside a job: the job finishes in the
//!   space it started in, and the next job starts in the new space.
//! * Durable knowledge-base chunks are protected a second time by the KB
//!   space guard (`KbConfig::ensure_embedding_space`): after a swap,
//!   previously indexed KBs refuse the new space explicitly instead of mixing
//!   vectors. The in-memory skill index is derived data and is re-indexed by
//!   the vector matcher on space change (see `matching/vector.rs`).
//!
//! The trait's borrowed-return metadata (`backend_name`, `model_id`,
//! `endpoint`) is served from `String` fields owned by the slot, so no
//! per-swap leak and no guard-escaping borrows are needed.

use std::fmt;
use std::sync::Arc;

use arc_swap::ArcSwapAny;

use super::{EmbeddingBackend, EmbeddingError};

/// One coherent backend + metadata snapshot. Metadata is captured eagerly so
/// every borrow the trait returns is tied to the slot itself.
pub struct BackendSlot {
    backend: Arc<dyn EmbeddingBackend>,
    backend_name: String,
    model_id: String,
    endpoint: Option<String>,
    vector_dimension: usize,
    provenance: Option<super::BackendProvenance>,
}

impl BackendSlot {
    fn capture(backend: Arc<dyn EmbeddingBackend>) -> Self {
        Self {
            backend_name: backend.backend_name().to_string(),
            model_id: backend.model_id().to_string(),
            endpoint: backend.endpoint().map(str::to_string),
            vector_dimension: backend.vector_dimension(),
            provenance: backend.provenance(),
            backend,
        }
    }
}

impl fmt::Debug for BackendSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BackendSlot")
            .field("backend_name", &self.backend_name)
            .field("model_id", &self.model_id)
            .field("vector_dimension", &self.vector_dimension)
            .finish()
    }
}

#[async_trait::async_trait]
impl EmbeddingBackend for BackendSlot {
    fn backend_name(&self) -> &str {
        &self.backend_name
    }

    fn vector_dimension(&self) -> usize {
        self.vector_dimension
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn endpoint(&self) -> Option<&str> {
        self.endpoint.as_deref()
    }

    fn provenance(&self) -> Option<super::BackendProvenance> {
        self.provenance.clone()
    }

    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        self.backend.embed(texts).await
    }

    async fn shutdown(&self) -> Result<(), EmbeddingError> {
        self.backend.shutdown().await
    }
}

/// Shared handle publishing the current [`BackendSlot`]. Consumers pin a slot
/// per job; they must not assume the handle itself is an `EmbeddingBackend`.
pub struct SwappableEmbeddingBackend {
    slot: ArcSwapAny<Arc<BackendSlot>>,
}

impl SwappableEmbeddingBackend {
    pub fn new(backend: Arc<dyn EmbeddingBackend>) -> Self {
        Self {
            slot: ArcSwapAny::new(Arc::new(BackendSlot::capture(backend))),
        }
    }

    /// Publish a new backend atomically. The caller must have constructed the
    /// backend successfully (validated configuration); this type only
    /// publishes. In-flight jobs keep their pinned slot and finish in the
    /// space they started in.
    pub fn swap(&self, backend: Arc<dyn EmbeddingBackend>) {
        self.slot.store(Arc::new(BackendSlot::capture(backend)));
    }

    /// Pin the current backend + metadata as ONE coherent snapshot for a job.
    pub fn pin(&self) -> Arc<dyn EmbeddingBackend> {
        self.slot.load_full()
    }

    /// Provenance of the currently published instance, for status surfaces.
    pub fn provenance(&self) -> Option<super::BackendProvenance> {
        self.slot.load_full().provenance.clone()
    }

    /// The embedding space the NEXT pinned job would produce vectors in.
    pub fn fingerprint(&self) -> crate::uar::domain::knowledge::EmbeddingFingerprint {
        let slot = self.slot.load_full();
        crate::uar::domain::knowledge::EmbeddingFingerprint {
            backend: slot.backend_name().to_string(),
            model: slot.model_id().to_string(),
            dimension: slot.vector_dimension(),
        }
    }

    /// Current backend identity for diagnostics.
    pub fn describe(&self) -> (String, String, usize) {
        let slot = self.slot.load_full();
        (
            slot.backend_name().to_string(),
            slot.model_id().to_string(),
            slot.vector_dimension(),
        )
    }

    /// Release the currently published backend after the host has drained all
    /// users of this shared handle.
    pub async fn shutdown(&self) -> Result<(), EmbeddingError> {
        self.pin().shutdown().await
    }
}

impl fmt::Debug for SwappableEmbeddingBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SwappableEmbeddingBackend")
            .field("active", &self.slot.load_full())
            .finish()
    }
}
