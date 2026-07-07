//! Kreuzberg file processing provider.
//!
//! Uses the Kreuzberg Rust core for high-performance document extraction.
//! Supports 56+ file formats including PDF, Office documents, images, and more.

use async_trait::async_trait;
use std::fmt::Write;
use std::path::Path;
use std::time::Duration;

use crate::config::KreuzbergConfig;

use super::provider::{ExtractedImage, FileProcessor, ProcessingError, ProcessingResult};

/// Kreuzberg-based file processor using the native Rust core.
///
/// Provides high-performance document extraction with OCR support.
/// Supports PDFs, Office documents, images, HTML, XML, archives, and more.
#[derive(Debug)]
pub struct KreuzbergProvider {
    config: KreuzbergConfig,
}

impl KreuzbergProvider {
    /// Create a new Kreuzberg provider with the given configuration.
    pub fn new(config: KreuzbergConfig) -> Self {
        Self { config }
    }

    /// Create a provider with default configuration.
    pub fn with_defaults() -> Self {
        Self {
            config: KreuzbergConfig::default(),
        }
    }

    /// Build the Kreuzberg `ExtractionConfig` from our config.
    fn build_extraction_config(&self) -> kreuzberg::ExtractionConfig {
        let mut config = kreuzberg::ExtractionConfig::default();

        // Map output format string → v4 enum
        config.output_format = match self.config.output_format.as_str() {
            "markdown" => kreuzberg::OutputFormat::Markdown,
            _ => kreuzberg::OutputFormat::Plain,
        };

        // Configure OCR settings
        if self.config.ocr_enabled {
            let ocr_config = kreuzberg::OcrConfig {
                backend: self.config.ocr_backend.clone(),
                language: self.config.ocr_language.clone(),
                ..kreuzberg::OcrConfig::default()
            };
            config.ocr = Some(ocr_config);
        }

        // Force OCR setting
        config.force_ocr = self.config.force_ocr;

        // Wire chunking config when present
        if let Some(chunking) = &self.config.chunking {
            config.chunking = Some(kreuzberg::ChunkingConfig {
                max_characters: chunking.max_characters,
                overlap: chunking.overlap,
                ..kreuzberg::ChunkingConfig::default()
            });
        }

        config
    }
}

#[async_trait]
impl FileProcessor for KreuzbergProvider {
    async fn process(&self, path: &Path) -> Result<ProcessingResult, ProcessingError> {
        let file_len = tokio::fs::metadata(path).await?.len();
        if file_len > self.config.max_input_bytes {
            return Err(ProcessingError::ResourceLimitExceeded(format!(
                "document is {file_len} bytes, exceeds max_input_bytes ({})",
                self.config.max_input_bytes
            )));
        }

        let extraction_config = self.build_extraction_config();

        // Use kreuzberg's synchronous API wrapped in spawn_blocking
        // for async compatibility
        let path_buf = path.to_path_buf();
        let result = tokio::time::timeout(
            Duration::from_secs(self.config.extraction_timeout_secs),
            tokio::task::spawn_blocking(move || {
                kreuzberg::extract_file_sync(path_buf, None, &extraction_config)
            }),
        )
        .await
        .map_err(|_elapsed| {
            ProcessingError::ResourceLimitExceeded(format!(
                "extraction exceeded {}s timeout",
                self.config.extraction_timeout_secs
            ))
        })?
        .map_err(|e| ProcessingError::ProviderError(format!("Task join error: {e}")))?
        .map_err(|e| ProcessingError::ProviderError(format!("Kreuzberg error: {e}")))?;

        // Extract MIME type from the file
        let mime_type = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        // Convert tables to a simple format if present
        let mut content = result.content.clone();

        // Append tables in markdown format if extracted
        if !result.tables.is_empty() {
            content.push_str("\n\n## Extracted Tables\n\n");
            for (i, table) in result.tables.iter().enumerate() {
                let _ = writeln!(content, "### Table {}\n", i + 1);
                let _ = writeln!(content, "{}\n", table.markdown);
            }
        }

        // Convert metadata to JSON
        let metadata = self.config.extract_metadata.then(|| {
            serde_json::json!({
                "title": result.metadata.title,
                "subject": result.metadata.subject,
                "authors": result.metadata.authors,
                "keywords": result.metadata.keywords,
                "language": result.metadata.language,
                "pages": result.metadata.pages,
            })
        });

        // Extract any embedded images (if available)
        let images: Vec<ExtractedImage> = result
            .images
            .unwrap_or_default()
            .into_iter()
            .map(|img| ExtractedImage {
                data: img.data.to_vec(),
                mime_type: format!("image/{}", img.format),
                description: None,
            })
            .collect();

        Ok(ProcessingResult {
            content,
            mime_type,
            metadata,
            images,
        })
    }

    fn supports_mime_type(&self, mime_type: &str) -> bool {
        if mime_type.starts_with("image/") && !self.config.ocr_enabled {
            return false;
        }

        // Kreuzberg supports 56+ file formats
        matches!(
            mime_type,
            // PDFs
            "application/pdf"
            // Microsoft Office
            | "application/msword"
            | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            | "application/vnd.ms-excel"
            | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            | "application/vnd.ms-powerpoint"
            | "application/vnd.openxmlformats-officedocument.presentationml.presentation"
            // OpenDocument
            | "application/vnd.oasis.opendocument.text"
            | "application/vnd.oasis.opendocument.spreadsheet"
            | "application/vnd.oasis.opendocument.presentation"
            // Images (OCR)
            | "image/png"
            | "image/jpeg"
            | "image/jpg"
            | "image/gif"
            | "image/webp"
            | "image/bmp"
            | "image/tiff"
            // Text formats
            | "text/plain"
            | "text/markdown"
            | "text/html"
            | "text/xml"
            | "text/csv"
            | "text/rtf"
            | "application/json"
            | "application/xml"
            // Archives (kreuzberg extracts contents)
            | "application/zip"
            | "application/x-tar"
            | "application/gzip"
            // Email
            | "message/rfc822"
            | "application/vnd.ms-outlook"
            // Code files
            | "text/x-python"
            | "text/x-rust"
            | "text/javascript"
            | "text/typescript"
            | "text/x-java"
            | "text/x-c"
            | "text/x-c++"
        )
    }

    fn provider_name(&self) -> &'static str {
        "Kreuzberg"
    }
}

/// Process bytes directly without writing to disk.
/// Useful for processing uploaded file data.
#[allow(dead_code)]
pub async fn process_bytes(
    data: &[u8],
    mime_type: &str,
    config: &KreuzbergConfig,
) -> Result<ProcessingResult, ProcessingError> {
    if data.len() as u64 > config.max_input_bytes {
        return Err(ProcessingError::ResourceLimitExceeded(format!(
            "document is {} bytes, exceeds max_input_bytes ({})",
            data.len(),
            config.max_input_bytes
        )));
    }

    let mut extraction_config = kreuzberg::ExtractionConfig::default();

    // Map output format string → v4 enum
    extraction_config.output_format = match config.output_format.as_str() {
        "markdown" => kreuzberg::OutputFormat::Markdown,
        _ => kreuzberg::OutputFormat::Plain,
    };

    // Configure OCR
    if config.ocr_enabled {
        let ocr_config = kreuzberg::OcrConfig {
            backend: config.ocr_backend.clone(),
            language: config.ocr_language.clone(),
            ..kreuzberg::OcrConfig::default()
        };
        extraction_config.ocr = Some(ocr_config);
    }

    extraction_config.force_ocr = config.force_ocr;

    // Wire chunking if configured
    if let Some(chunking) = &config.chunking {
        extraction_config.chunking = Some(kreuzberg::ChunkingConfig {
            max_characters: chunking.max_characters,
            overlap: chunking.overlap,
            ..kreuzberg::ChunkingConfig::default()
        });
    }

    let data_owned = data.to_vec();
    let mime_for_extraction = mime_type.to_string();
    let mime_for_result = mime_type.to_string();

    let result = tokio::time::timeout(
        Duration::from_secs(config.extraction_timeout_secs),
        tokio::task::spawn_blocking(move || {
            kreuzberg::extract_bytes_sync(&data_owned, &mime_for_extraction, &extraction_config)
        }),
    )
    .await
    .map_err(|_elapsed| {
        ProcessingError::ResourceLimitExceeded(format!(
            "extraction exceeded {}s timeout",
            config.extraction_timeout_secs
        ))
    })?
    .map_err(|e| ProcessingError::ProviderError(format!("Task join error: {e}")))?
    .map_err(|e| ProcessingError::ProviderError(format!("Kreuzberg error: {e}")))?;

    let mut content = result.content;

    // Append tables
    if !result.tables.is_empty() {
        content.push_str("\n\n## Extracted Tables\n\n");
        for (i, table) in result.tables.iter().enumerate() {
            let _ = writeln!(content, "### Table {}\n", i + 1);
            let _ = writeln!(content, "{}\n", table.markdown);
        }
    }

    let metadata = config.extract_metadata.then(|| {
        serde_json::json!({
            "title": result.metadata.title,
            "subject": result.metadata.subject,
            "authors": result.metadata.authors,
            "keywords": result.metadata.keywords,
            "language": result.metadata.language,
            "pages": result.metadata.pages,
        })
    });

    let images: Vec<ExtractedImage> = result
        .images
        .unwrap_or_default()
        .into_iter()
        .map(|img| ExtractedImage {
            data: img.data.to_vec(),
            mime_type: format!("image/{}", img.format),
            description: None,
        })
        .collect();

    Ok(ProcessingResult {
        content,
        mime_type: mime_for_result,
        metadata,
        images,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kreuzberg_provider_creation() {
        let provider = KreuzbergProvider::with_defaults();
        assert_eq!(provider.provider_name(), "Kreuzberg");
    }

    #[test]
    fn test_mime_type_support() {
        let provider = KreuzbergProvider::with_defaults();

        // PDFs should be supported
        assert!(provider.supports_mime_type("application/pdf"));

        // Images require OCR to be enabled.
        assert!(!provider.supports_mime_type("image/png"));
        let ocr_provider = KreuzbergProvider::new(KreuzbergConfig {
            ocr_enabled: true,
            ..KreuzbergConfig::default()
        });
        assert!(ocr_provider.supports_mime_type("image/png"));
        assert!(ocr_provider.supports_mime_type("image/jpeg"));

        // Office documents
        assert!(provider.supports_mime_type(
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        ));

        // Text files
        assert!(provider.supports_mime_type("text/plain"));
        assert!(provider.supports_mime_type("text/markdown"));
    }

    #[test]
    fn test_custom_config() {
        let config = KreuzbergConfig {
            ocr_enabled: true,
            force_ocr: true,
            ocr_backend: "tesseract".to_string(),
            ocr_language: "eng+deu".to_string(),
            pdf_dpi: 150,
            extract_tables: true,
            extract_metadata: true,
            output_format: "markdown".to_string(),
            chunking: None,
            ..KreuzbergConfig::default()
        };

        let provider = KreuzbergProvider::new(config);
        assert_eq!(provider.provider_name(), "Kreuzberg");
    }

    #[tokio::test]
    async fn test_process_bytes_rejects_oversized_input() {
        let config = KreuzbergConfig {
            max_input_bytes: 16,
            ..KreuzbergConfig::default()
        };
        let data = vec![0u8; 17];

        let err = process_bytes(&data, "text/plain", &config)
            .await
            .expect_err("oversized input must be rejected before reaching kreuzberg");

        assert!(matches!(err, ProcessingError::ResourceLimitExceeded(_)));
    }

    #[tokio::test]
    async fn test_process_bytes_accepts_input_within_limit() {
        let config = KreuzbergConfig {
            max_input_bytes: 4096,
            ..KreuzbergConfig::default()
        };
        let data = b"hello world".to_vec();

        let result = process_bytes(&data, "text/plain", &config).await;

        assert!(result.is_ok(), "input within the size cap should proceed to extraction");
    }
}
