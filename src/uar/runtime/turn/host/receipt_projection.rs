//! Project before both the no-store return and canonical acquisition/save.
use super::RunSecretScrubber;
use crate::uar::persistence::agent_threads::{
    CanonicalRawSegment, CanonicalReceiptStoreError, CanonicalSecretProjection,
};
use serde_json::Value;

#[derive(Debug)]
pub struct ProjectedCanonicalResult {
    pub value: Value,
    pub raw_segments: Vec<CanonicalRawSegment>,
    pub secret_projection: CanonicalSecretProjection,
}

impl RunSecretScrubber {
    /// Existing acquisition failures remain the caller's acquisition_complete
    /// flag. Opaque/unavailable raw streams are omitted, never fabricated or
    /// retained in an alternate archive. Hashes describe only retained bytes.
    pub fn project_receipt(
        &self,
        value: Value,
        raw_segments: Vec<CanonicalRawSegment>,
    ) -> Result<ProjectedCanonicalResult, CanonicalReceiptStoreError> {
        let (value, mut redacted) = self.0.value(value);
        let mut projected = Vec::with_capacity(raw_segments.len());
        let mut omitted_raw_segments = 0;
        for segment in raw_segments {
            let Some(bytes) = segment.verified_bytes()? else {
                omitted_raw_segments += 1;
                continue;
            };
            let Ok(text) = std::str::from_utf8(&bytes) else {
                omitted_raw_segments += 1;
                continue;
            };
            let (name, name_changed) = self.0.text(&segment.name);
            let (text, text_changed) = self.0.text(text);
            redacted |= name_changed || text_changed;
            projected.push(CanonicalRawSegment::new(name, text.as_bytes()));
        }
        Ok(ProjectedCanonicalResult {
            value,
            raw_segments: projected,
            secret_projection: CanonicalSecretProjection {
                version: 1,
                redacted,
                omitted_raw_segments,
            },
        })
    }
}
