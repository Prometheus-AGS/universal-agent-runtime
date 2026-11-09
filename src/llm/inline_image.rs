//! Bounded host admission for the ordinary chat inline-image contract.
//!
//! Syntax metadata is useful when projecting already-admitted canonical history;
//! it is not a substitute for decoder admission and grants no resource authority.

use base64::{Engine as _, engine::general_purpose::STANDARD};

use super::ImageUrl;

/// The existing whole HTTP envelope also caps a standalone image field.
pub(crate) const MAX_INLINE_IMAGE_BYTES: usize = 10 * 1024 * 1024;
/// Bound repeated decoder work before starting any image in a current turn.
pub(crate) const MAX_INLINE_IMAGES: usize = 8;

/// Payload-free syntax metadata. `decoded_bytes` counts encoded-file bytes,
/// not the decoder's pixel allocation or a provider's image-token estimate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct InlineImageMetadata {
    pub(crate) mime_type: &'static str,
    pub(crate) encoded_bytes: usize,
    pub(crate) decoded_bytes: usize,
}

/// Fixed diagnostics only: neither codec errors nor input bytes are retained.
#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("{message}")]
pub(crate) struct InlineImageError {
    pub(crate) code: &'static str,
    pub(crate) message: &'static str,
}

fn invalid_image(code: &'static str, message: &'static str) -> InlineImageError {
    InlineImageError { code, message }
}

fn decode_inline_image(image: &ImageUrl) -> Result<(InlineImageMetadata, Vec<u8>), InlineImageError> {
    if image.detail.as_deref().is_some_and(|detail| !matches!(detail, "auto" | "low" | "high")) {
        return Err(invalid_image("invalid_image_detail", "Image detail must be auto, low or high"));
    }
    if image.url.len() > MAX_INLINE_IMAGE_BYTES {
        return Err(invalid_image("image_too_large", "Inline image exceeds the 10 MiB encoded limit"));
    }
    let (mime_type, encoded) = if let Some(encoded) = image.url.strip_prefix("data:image/png;base64,") {
        ("image/png", encoded)
    } else if let Some(encoded) = image.url.strip_prefix("data:image/jpeg;base64,") {
        ("image/jpeg", encoded)
    } else {
        return Err(invalid_image("unsupported_image_source", "Images require inline base64 PNG or JPEG data URLs"));
    };
    if encoded.is_empty() {
        return Err(invalid_image("invalid_image_base64", "Image requires nonempty standard padded base64"));
    }
    let bytes = STANDARD.decode(encoded)
        .map_err(|_| invalid_image("invalid_image_base64", "Image requires valid standard padded base64"))?;
    let signature_matches = match mime_type {
        "image/png" => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "image/jpeg" => bytes.starts_with(&[0xff, 0xd8, 0xff]),
        _ => false,
    };
    if !signature_matches {
        return Err(invalid_image("image_mime_mismatch", "Image bytes do not match the declared PNG or JPEG MIME type"));
    }
    Ok((InlineImageMetadata {
        mime_type,
        encoded_bytes: image.url.len(),
        decoded_bytes: bytes.len(),
    }, bytes))
}

/// Check only the supported inline grammar, base64 and MIME signature.
/// Callers must not treat this as full decoder acceptance or use it to exempt
/// arbitrary JSON/state from secret projection.
pub(crate) fn inline_image_metadata(image: &ImageUrl) -> Result<InlineImageMetadata, InlineImageError> {
    decode_inline_image(image).map(|(metadata, _)| metadata)
}

/// Decode before host admission. Original URL/detail remain unchanged on success.
/// The worker holds its concurrency permit and host lifecycle lease even when
/// the requesting HTTP future is dropped; a timeout joins before returning.
#[cfg(feature = "document-intelligence")]
pub(crate) async fn validate_inline_image(image: &ImageUrl) -> Result<InlineImageMetadata, InlineImageError> {
    use std::time::Duration;

    static DECODERS: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(2);
    const DECODE_BUDGET_SECS: u64 = 10;
    const MAX_DECODED_LIVE_BYTES: usize = 100 * 1024 * 1024;

    // No waiting queue retaining arbitrary request payloads. This permit is
    // moved into the actual worker, not released when its waiter is cancelled.
    let permit = DECODERS.try_acquire().map_err(|_| {
        invalid_image("image_decoder_busy", "Inline image decoder capacity is busy; retry later")
    })?;
    if image.url.len() > MAX_INLINE_IMAGE_BYTES {
        return Err(invalid_image("image_too_large", "Inline image exceeds the 10 MiB encoded limit"));
    }
    let image = image.clone();
    let handle = tokio::runtime::Handle::current();
    let cancel = xberg::cancellation::CancellationToken::new();
    struct CancelOnDrop(xberg::cancellation::CancellationToken);
    impl Drop for CancelOnDrop {
        fn drop(&mut self) {
            self.0.cancel();
        }
    }
    let cancel_on_drop = CancelOnDrop(cancel.clone());
    let mut worker = crate::uar::runtime::lifecycle::spawn_blocking("inline-image-admission", move || {
        let _permit = permit;
        let (metadata, bytes) = decode_inline_image(&image)?;
        if xberg::detect_mime_type_from_bytes(&bytes).ok().as_deref() != Some(metadata.mime_type) {
            return Err(invalid_image("image_mime_mismatch", "Image bytes do not match the declared PNG or JPEG MIME type"));
        }
        let config = xberg::ExtractionConfig {
            disable_ocr: true,
            use_cache: false,
            enable_quality_processing: false,
            postprocessor: Some(xberg::PostProcessorConfig {
                enabled: false,
                ..Default::default()
            }),
            security_limits: Some(xberg::SecurityLimits {
                max_content_size: MAX_DECODED_LIVE_BYTES,
                ..Default::default()
            }),
            max_concurrent_extractions: Some(1),
            extraction_timeout_secs: Some(DECODE_BUDGET_SECS),
            cancel_token: Some(cancel),
            ..Default::default()
        };
        // Bytes-only input: no URI, filename, OCR, cache, captioning or operator
        // configuration. The PNG/JPEG extractor decodes before returning metadata.
        let input = xberg::ExtractInput::from_bytes(bytes, metadata.mime_type, None);
        let result = handle.block_on(xberg::extract(input, &config))
            .map_err(|_| invalid_image("invalid_image_data", "Image decoder rejected the image or its allocation limit"))?;
        if !result.errors.is_empty() || result.results.len() != 1 {
            return Err(invalid_image("invalid_image_data", "Image decoder did not return one valid image"));
        }
        let document = &result.results[0];
        let expected_format = if metadata.mime_type == "image/png" { "PNG" } else { "JPEG" };
        if document.mime_type.as_ref() != metadata.mime_type
            || !matches!(&document.metadata.format, Some(xberg::FormatMetadata::Image(image))
                if image.format == expected_format && image.width > 0 && image.height > 0)
        {
            return Err(invalid_image("invalid_image_data", "Image decoder returned incompatible image metadata"));
        }
        Ok(metadata)
    }).map_err(|_| invalid_image("image_decoder_unavailable", "Inline image decoder admission is closed"))?;

    match tokio::time::timeout(Duration::from_secs(DECODE_BUDGET_SECS), &mut worker).await {
        Ok(result) => result.map_err(|_| invalid_image("image_decoder_unavailable", "Inline image decoder worker failed"))?,
        Err(_) => {
            cancel_on_drop.0.cancel();
            worker.abort();
            // A running blocking decoder cannot be forcibly stopped. Join it
            // and retain the lease/permit until actual completion, then refuse.
            let _joined = worker.await;
            Err(invalid_image("image_decode_timeout", "Inline image decoding exceeded its time budget"))
        }
    }
}

/// Builds without the pinned decoder must explicitly refuse image admission.
#[cfg(not(feature = "document-intelligence"))]
pub(crate) async fn validate_inline_image(_image: &ImageUrl) -> Result<InlineImageMetadata, InlineImageError> {
    Err(invalid_image("image_decoder_unavailable", "Inline images require the document-intelligence feature"))
}
