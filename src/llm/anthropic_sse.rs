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

/// Deterministic byte-partition acceptance tests for the native Anthropic SSE
/// path (issue #380 item 3).
///
/// Every test drives `decode_anthropic_sse`, the same transport framer ->
/// `liter_llm::SseDecoder` -> JSON -> `StreamState` loop that
/// `AnthropicDriver::stream` runs over `reqwest::Response::bytes_stream()`,
/// with scripted byte chunks instead of a socket.
#[cfg(test)]
mod byte_partition_tests {
    use futures::StreamExt;

    use crate::llm::anthropic_driver::decode_anthropic_sse;
    use crate::llm::anthropic_streaming::StreamState;
    use crate::llm::{ProviderError, ProviderErrorKind};
    use crate::normalized::NormalizedEvent;

    const REQUEST_ID: &str = "req-byte-partition";
    const MODEL: &str = "claude-partition-test";
    /// The pinned `liter_llm::SseDecoder` retained-byte limit (1 MiB).
    const DECODER_BUFFER_LIMIT: usize = 1024 * 1024;

    const MESSAGE_START_DATA: &str = r#"data: {"type":"message_start","message":{"id":"msg_01","type":"message","role":"assistant","model":"claude-partition-test","content":[],"stop_reason":null,"usage":{"input_tokens":25,"output_tokens":1}}}"#;
    const MESSAGE_STOP_DATA: &str = r#"data: {"type":"message_stop"}"#;

    /// Everything observable from one decode: events yielded before the stream
    /// ended and the terminal failure, if the stream ended in one.
    struct Outcome {
        events: Vec<NormalizedEvent>,
        failure: Option<(ProviderErrorKind, String)>,
    }

    /// Feed `chunks` through the production decoder loop.
    async fn decode(chunks: &[&[u8]]) -> Outcome {
        let items: Vec<Result<Vec<u8>, reqwest::Error>> =
            chunks.iter().map(|chunk| Ok(chunk.to_vec())).collect();
        let mut stream = decode_anthropic_sse(
            futures::stream::iter(items),
            StreamState::new(REQUEST_ID.to_string()),
            MODEL.to_string(),
        );

        let mut events = Vec::new();
        let mut failure = None;
        while let Some(item) = stream.next().await {
            match item {
                Ok(event) => events.push(event),
                Err(error) => {
                    let provider = ProviderError::from_anyhow(&error)
                        .expect("stream failures must be typed provider errors");
                    failure = Some((provider.kind, error.to_string()));
                    break;
                }
            }
        }
        if failure.is_some() {
            assert!(
                stream.next().await.is_none(),
                "a terminal failure must be the last stream item"
            );
        }
        Outcome { events, failure }
    }

    /// Render one SSE event: each line followed by `eol`, then a blank line.
    fn sse_event(eol: &str, lines: &[&str]) -> String {
        let mut event = String::new();
        for line in lines {
            event.push_str(line);
            event.push_str(eol);
        }
        event.push_str(eol);
        event
    }

    /// The fixed wire fixture, one complete SSE event per element.
    ///
    /// It mixes LF, CRLF and bare-CR line endings, a comment line, a field
    /// without a space after the colon, an ignored `id` field, a multi-line
    /// `data:` field, a forward-compatible unknown event, a ping, multibyte
    /// text (literal and JSON-escaped), tool-call JSON deltas split across
    /// events, usage in `message_start` and `message_delta`, and the terminal
    /// `message_stop` as the last element.
    fn fixture_events() -> Vec<String> {
        vec![
            sse_event("\n", &["id: 1", "event: message_start", MESSAGE_START_DATA]),
            sse_event(
                "\r\n",
                &[
                    ": keepalive",
                    "event:content_block_start",
                    r#"data:{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#,
                ],
            ),
            sse_event("\n", &["event: ping", r#"data: {"type": "ping"}"#]),
            sse_event(
                "\n",
                &[
                    "event: future_event",
                    r#"data: {"type":"future_event","detail":{"x":1}}"#,
                ],
            ),
            sse_event(
                "\r\n",
                &[
                    "event: content_block_delta",
                    r#"data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"café \"é\" 🦀"}}"#,
                ],
            ),
            sse_event(
                "\n",
                &[
                    "event: content_block_delta",
                    r#"data: {"type":"content_block_delta","index":0,"#,
                    r#"data: "delta":{"type":"text_delta","text":" ok\n"}}"#,
                ],
            ),
            sse_event(
                "\r",
                &[
                    "event: content_block_stop",
                    r#"data: {"type":"content_block_stop","index":0}"#,
                ],
            ),
            sse_event(
                "\n",
                &[
                    "event: content_block_start",
                    r#"data: {"type":"content_block_start","index":1,"content_block":{"type":"tool_use","id":"toolu_01","name":"get_weather","input":{}}}"#,
                ],
            ),
            sse_event(
                "\r\n",
                &[
                    "event: content_block_delta",
                    r#"data: {"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":""}}"#,
                ],
            ),
            sse_event(
                "\n",
                &[
                    "event: content_block_delta",
                    r#"data: {"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":"{\"city\":"}}"#,
                ],
            ),
            sse_event(
                "\r\n",
                &[
                    "event: content_block_delta",
                    r#"data: {"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":" \"Par"}}"#,
                ],
            ),
            sse_event(
                "\n",
                &[
                    "event: content_block_delta",
                    r#"data: {"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":"is\"}"}}"#,
                ],
            ),
            sse_event(
                "\n",
                &[
                    "event: content_block_stop",
                    r#"data: {"type":"content_block_stop","index":1}"#,
                ],
            ),
            sse_event(
                "\r\n",
                &[
                    "event: message_delta",
                    r#"data: {"type":"message_delta","delta":{"stop_reason":"tool_use","stop_sequence":null},"usage":{"output_tokens":42}}"#,
                ],
            ),
            sse_event("\n", &["event: message_stop", MESSAGE_STOP_DATA]),
        ]
    }

    fn fixture() -> Vec<u8> {
        fixture_events().concat().into_bytes()
    }

    /// The fixture minus its terminal `message_stop` event.
    fn fixture_without_terminal_event() -> Vec<u8> {
        let mut events = fixture_events();
        events.pop();
        events.concat().into_bytes()
    }

    /// The normalized sequence the fixture must produce, with the request ID
    /// controlled by `REQUEST_ID`.
    fn expected_events() -> Vec<NormalizedEvent> {
        let tool_delta = |fragment: &str| NormalizedEvent::ToolCallDelta {
            call_index: 0,
            id: Some("toolu_01".to_string()),
            name: Some("get_weather".to_string()),
            arguments_delta: Some(fragment.to_string()),
        };
        vec![
            NormalizedEvent::StreamStart {
                request_id: REQUEST_ID.to_string(),
            },
            NormalizedEvent::MessageDelta {
                text: "café \"é\" 🦀".to_string(),
            },
            NormalizedEvent::MessageDelta {
                text: " ok\n".to_string(),
            },
            tool_delta(""),
            tool_delta("{\"city\":"),
            tool_delta(" \"Par"),
            tool_delta("is\"}"),
            NormalizedEvent::ToolCallComplete {
                call_index: 0,
                id: "toolu_01".to_string(),
                name: "get_weather".to_string(),
                arguments_json: "{\"city\": \"Paris\"}".to_string(),
            },
            NormalizedEvent::Usage {
                prompt_tokens: 25,
                completion_tokens: 42,
                total_tokens: 67,
                cached_tokens: None,
                cache_creation_tokens: None,
            },
            NormalizedEvent::Done,
        ]
    }

    /// Everything the stream yields before the terminal `message_stop` is
    /// processed: the expected sequence without `Usage` and `Done`.
    fn expected_events_before_terminal() -> Vec<NormalizedEvent> {
        let mut events = expected_events();
        events.truncate(events.len() - 2);
        events
    }

    /// Decode the fixture as one chunk, require the expected sequence, and
    /// return it as the baseline every partition is compared against.
    async fn single_chunk_baseline(bytes: &[u8]) -> Vec<NormalizedEvent> {
        let outcome = decode(&[bytes]).await;
        assert_eq!(outcome.failure, None, "single chunk must not fail");
        assert_eq!(
            outcome.events,
            expected_events(),
            "single-chunk decode must match the expected normalized sequence"
        );
        outcome.events
    }

    async fn assert_partition_matches_baseline(
        baseline: &[NormalizedEvent],
        label: &str,
        chunks: &[&[u8]],
    ) {
        let outcome = decode(chunks).await;
        assert_eq!(outcome.failure, None, "{label}: unexpected failure");
        assert_eq!(
            outcome.events, baseline,
            "{label}: normalized events diverged from the single-chunk result"
        );
    }

    /// Split `bytes` at the non-decreasing byte offsets in `cuts`. Equal
    /// offsets produce empty chunks.
    fn partition<'a>(bytes: &'a [u8], cuts: &[usize]) -> Vec<&'a [u8]> {
        let mut chunks = Vec::with_capacity(cuts.len() + 1);
        let mut start = 0;
        for &cut in cuts {
            chunks.push(&bytes[start..cut]);
            start = cut;
        }
        chunks.push(&bytes[start..]);
        chunks
    }

    /// A fixed, deterministic set of multi-chunk partitions.
    fn fixed_partitions(bytes: &[u8]) -> Vec<(String, Vec<&[u8]>)> {
        let mut partitions: Vec<(String, Vec<&[u8]>)> = Vec::new();

        for size in [2, 3, 5, 7, 11, 13, 64, 1000] {
            partitions.push((format!("chunks of {size} bytes"), bytes.chunks(size).collect()));
        }

        let mut irregular = Vec::new();
        let mut at = 0;
        for size in [1usize, 2, 3, 5, 8, 13, 21].iter().cycle() {
            at += size;
            if at >= bytes.len() {
                break;
            }
            irregular.push(at);
        }
        partitions.push((
            "irregular 1/2/3/5/8/13/21 byte chunks".to_string(),
            partition(bytes, &irregular),
        ));

        let after_line_end: Vec<usize> = bytes
            .iter()
            .enumerate()
            .filter(|(_, byte)| matches!(**byte, b'\r' | b'\n'))
            .map(|(index, _)| index + 1)
            .collect();
        partitions.push((
            "split after every CR or LF".to_string(),
            partition(bytes, &after_line_end),
        ));

        let before_line_end: Vec<usize> = bytes
            .iter()
            .enumerate()
            .filter(|(_, byte)| matches!(**byte, b'\r' | b'\n'))
            .map(|(index, _)| index)
            .collect();
        partitions.push((
            "split before every CR or LF".to_string(),
            partition(bytes, &before_line_end),
        ));

        let empty_chunks: Vec<usize> = after_line_end
            .iter()
            .flat_map(|&cut| [cut, cut])
            .collect();
        partitions.push((
            "split after every CR or LF with empty chunks".to_string(),
            partition(bytes, &empty_chunks),
        ));

        let before_non_ascii: Vec<usize> = bytes
            .iter()
            .enumerate()
            .filter(|(_, byte)| !byte.is_ascii())
            .map(|(index, _)| index)
            .collect();
        partitions.push((
            "split before every byte of a multibyte character".to_string(),
            partition(bytes, &before_non_ascii),
        ));

        partitions
    }

    /// Representative partitions applied to terminal-condition cases.
    fn representative_partitions(bytes: &[u8]) -> Vec<(&'static str, Vec<&[u8]>)> {
        vec![
            ("single chunk", vec![bytes]),
            ("one byte at a time", bytes.chunks(1).collect()),
            ("7-byte chunks", bytes.chunks(7).collect()),
        ]
    }

    /// Require a visible stream failure containing `message_fragment`, and no
    /// successful terminal.
    fn assert_stream_failure(label: &str, outcome: &Outcome, message_fragment: &str) {
        assert!(
            !outcome.events.contains(&NormalizedEvent::Done),
            "{label}: a failed stream must not yield Done"
        );
        let (kind, message) = outcome
            .failure
            .as_ref()
            .unwrap_or_else(|| panic!("{label}: expected a visible stream failure"));
        assert_eq!(*kind, ProviderErrorKind::Stream, "{label}: failure kind");
        assert!(
            message.contains(message_fragment),
            "{label}: failure message {message:?} lacks {message_fragment:?}"
        );
    }

    #[tokio::test]
    async fn byte_partition_single_chunk_yields_expected_normalized_sequence() {
        let bytes = fixture();
        single_chunk_baseline(&bytes).await;
    }

    #[tokio::test]
    async fn byte_partition_every_two_chunk_split_matches_single_chunk() {
        let bytes = fixture();
        let baseline = single_chunk_baseline(&bytes).await;

        // Offsets 0 and len produce an empty first or second chunk.
        for offset in 0..=bytes.len() {
            let (head, tail) = bytes.split_at(offset);
            assert_partition_matches_baseline(
                &baseline,
                &format!("two chunks split at byte {offset}"),
                &[head, tail],
            )
            .await;
        }
    }

    #[tokio::test]
    async fn byte_partition_one_byte_at_a_time_matches_single_chunk() {
        let bytes = fixture();
        let baseline = single_chunk_baseline(&bytes).await;
        let chunks: Vec<&[u8]> = bytes.chunks(1).collect();

        assert_partition_matches_baseline(&baseline, "one byte at a time", &chunks).await;
    }

    #[tokio::test]
    async fn byte_partition_fixed_multi_chunk_partitions_match_single_chunk() {
        let bytes = fixture();
        let baseline = single_chunk_baseline(&bytes).await;

        for (label, chunks) in fixed_partitions(&bytes) {
            assert_partition_matches_baseline(&baseline, &label, &chunks).await;
        }
    }

    #[tokio::test]
    async fn byte_partition_valid_terminal_then_eof_emits_exactly_one_done() {
        let bytes = fixture();
        for (label, chunks) in representative_partitions(&bytes) {
            let outcome = decode(&chunks).await;
            assert_eq!(outcome.failure, None, "{label}: unexpected failure");
            let done_count = outcome
                .events
                .iter()
                .filter(|event| matches!(event, NormalizedEvent::Done))
                .count();
            assert_eq!(done_count, 1, "{label}: exactly one Done expected");
            assert_eq!(
                outcome.events.last(),
                Some(&NormalizedEvent::Done),
                "{label}: Done must be the final event"
            );
        }
    }

    #[tokio::test]
    async fn byte_partition_malformed_complete_json_fails_even_before_a_later_message_stop() {
        let mut bytes = sse_event("\n", &["event: message_start", MESSAGE_START_DATA]).into_bytes();
        bytes.extend_from_slice(
            sse_event("\n", &["event: content_block_delta", "data: {not json"]).as_bytes(),
        );
        bytes.extend_from_slice(
            sse_event("\n", &["event: message_stop", MESSAGE_STOP_DATA]).as_bytes(),
        );

        for (label, chunks) in representative_partitions(&bytes) {
            let outcome = decode(&chunks).await;
            assert_stream_failure(label, &outcome, "Malformed Anthropic SSE event");
            assert_eq!(
                outcome.events,
                vec![NormalizedEvent::StreamStart {
                    request_id: REQUEST_ID.to_string()
                }],
                "{label}: only events before the corruption may be yielded"
            );
        }
    }

    #[tokio::test]
    async fn byte_partition_invalid_utf8_fails_even_before_a_later_message_stop() {
        let mut bytes = sse_event("\n", &["event: message_start", MESSAGE_START_DATA]).into_bytes();
        bytes.extend_from_slice(
            b"event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"bad \xFF byte\"}}\n\n",
        );
        bytes.extend_from_slice(
            sse_event("\n", &["event: message_stop", MESSAGE_STOP_DATA]).as_bytes(),
        );

        for (label, chunks) in representative_partitions(&bytes) {
            let outcome = decode(&chunks).await;
            assert_stream_failure(label, &outcome, "invalid UTF-8");
            assert!(
                outcome
                    .events
                    .iter()
                    .all(|event| !matches!(event, NormalizedEvent::MessageDelta { .. })),
                "{label}: corrupted text must not be surfaced"
            );
        }
    }

    #[tokio::test]
    async fn byte_partition_truncated_utf8_codepoint_at_eof_fails_without_done() {
        let mut bytes = sse_event("\n", &["event: message_start", MESSAGE_START_DATA]).into_bytes();
        bytes.extend_from_slice(
            b"event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"caf\xC3",
        );

        for (label, chunks) in representative_partitions(&bytes) {
            let outcome = decode(&chunks).await;
            assert_stream_failure(label, &outcome, "incomplete event or UTF-8 codepoint");
        }
    }

    #[tokio::test]
    async fn byte_partition_oversized_event_fails_without_done() {
        let mut bytes = sse_event("\n", &["event: message_start", MESSAGE_START_DATA]).into_bytes();
        bytes.extend_from_slice(
            b"event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"",
        );
        bytes.extend(std::iter::repeat_n(b'a', DECODER_BUFFER_LIMIT));
        bytes.extend_from_slice(b"\"}}\n\n");
        bytes.extend_from_slice(
            sse_event("\n", &["event: message_stop", MESSAGE_STOP_DATA]).as_bytes(),
        );

        let partitions: [(&str, Vec<&[u8]>); 2] = [
            ("single chunk", vec![bytes.as_slice()]),
            ("64 KiB chunks", bytes.chunks(64 * 1024).collect()),
        ];
        for (label, chunks) in partitions {
            let outcome = decode(&chunks).await;
            assert_stream_failure(label, &outcome, "exceed");
        }
    }

    #[tokio::test]
    async fn byte_partition_missing_message_stop_at_eof_fails_without_done() {
        let bytes = fixture_without_terminal_event();

        for (label, chunks) in representative_partitions(&bytes) {
            let outcome = decode(&chunks).await;
            assert_stream_failure(label, &outcome, "ended without message_stop");
            assert_eq!(
                outcome.events,
                expected_events_before_terminal(),
                "{label}: content before the missing terminal must still be normalized"
            );
        }
    }

    #[tokio::test]
    async fn byte_partition_final_unterminated_event_at_eof_fails_without_done() {
        // The message_stop event is complete JSON but never receives its
        // dispatching blank line; EOF must not complete it.
        let unterminated_tails = [
            "event: message_stop\ndata: {\"type\":\"message_stop\"}\n",
            "event: message_stop\ndata: {\"type\":\"message_stop\"}",
        ];
        for tail in unterminated_tails {
            let mut bytes = fixture_without_terminal_event();
            bytes.extend_from_slice(tail.as_bytes());

            for (label, chunks) in representative_partitions(&bytes) {
                let outcome = decode(&chunks).await;
                assert_stream_failure(label, &outcome, "incomplete event or UTF-8 codepoint");
                assert_eq!(
                    outcome.events,
                    expected_events_before_terminal(),
                    "{label}: the pending message_stop must not produce Usage or Done"
                );
            }
        }
    }
}
