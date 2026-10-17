//! Incremental wire normalization for native Anthropic SSE streams.
//!
//! The pinned Liter decoder remains responsible for SSE field parsing, UTF-8
//! validation, and its incomplete-frame limit. This adapter only canonicalizes
//! the transport details that Liter intentionally leaves opaque: one initial
//! UTF-8 BOM and CR/LF/CRLF line endings.

use std::fmt;

const UTF8_BOM: &[u8; 3] = b"\xEF\xBB\xBF";

/// An invalid initial byte prefix observed before the first SSE byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AnthropicSseFramingError {
    /// The transport ended in a partial UTF-8 BOM.
    IncompleteInitialBom,
}

impl fmt::Display for AnthropicSseFramingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IncompleteInitialBom => formatter.write_str("incomplete initial UTF-8 BOM"),
        }
    }
}

impl std::error::Error for AnthropicSseFramingError {}

/// Canonicalizes byte transport framing without buffering a complete SSE event.
///
/// A CR produces its LF immediately. If the following byte is LF it is
/// swallowed, which means a complete CR-terminated event is dispatched without
/// waiting for the next transport chunk.
#[derive(Default)]
pub(super) struct AnthropicSseFramer {
    initial: Vec<u8>,
    initial_decided: bool,
    swallow_following_lf: bool,
}

impl AnthropicSseFramer {
    /// Normalize one transport chunk into canonical LF-delimited bytes.
    pub(super) fn push(&mut self, bytes: &[u8]) -> Vec<u8> {
        let mut canonical = Vec::with_capacity(bytes.len());
        let mut offset = 0;

        if !self.initial_decided {
            while offset < bytes.len() && !self.initial_decided {
                self.initial.push(bytes[offset]);
                offset += 1;

                if !UTF8_BOM.starts_with(&self.initial) {
                    let initial = std::mem::take(&mut self.initial);
                    self.initial.clear();
                    self.initial_decided = true;
                    for byte in initial {
                        self.push_byte(byte, &mut canonical);
                    }
                } else if self.initial.len() == UTF8_BOM.len() {
                    self.initial.clear();
                    self.initial_decided = true;
                }
            }
        }

        for &byte in &bytes[offset..] {
            self.push_byte(byte, &mut canonical);
        }

        canonical
    }

    fn push_byte(&mut self, byte: u8, canonical: &mut Vec<u8>) {
        if self.swallow_following_lf {
            self.swallow_following_lf = false;
            if byte == b'\n' {
                return;
            }
        }
        if byte == b'\r' {
            canonical.push(b'\n');
            self.swallow_following_lf = true;
        } else {
            canonical.push(byte);
        }
    }

    /// Validate the initial prefix at transport EOF.
    pub(super) fn finish(&self) -> Result<(), AnthropicSseFramingError> {
        if self.initial_decided || self.initial.is_empty() {
            Ok(())
        } else {
            Err(AnthropicSseFramingError::IncompleteInitialBom)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AnthropicSseFramer;

    #[test]
    fn normalizes_cr_without_waiting_for_a_following_byte() {
        let mut framer = AnthropicSseFramer::default();
        assert_eq!(framer.push(b"data: {}\r\r"), b"data: {}\n\n");
        assert_eq!(framer.push(b"\n"), b"");
    }

    #[test]
    fn accepts_a_split_initial_bom_only_once() {
        let mut framer = AnthropicSseFramer::default();
        assert_eq!(framer.push(b"\xEF"), b"");
        assert_eq!(framer.push(b"\xBB\xBFdata: {}\n\n"), b"data: {}\n\n");
        assert_eq!(framer.push(b"\xEF\xBB\xBF"), b"\xEF\xBB\xBF");
    }
}
