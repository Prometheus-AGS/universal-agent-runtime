//! Streaming tool call extractor for instruction-tuned models.
//!
//! When a model emits `<tool_call>...</tool_call>` blocks in its text output
//! (because tools were injected via XML into the system prompt), this module
//! provides a streaming state machine that detects those blocks and converts
//! them into proper [`NormalizedEvent`] tool call events.

use crate::normalized::NormalizedEvent;
use uuid::Uuid;

/// Tag that opens a tool call block.
const OPEN_TAG: &str = "<tool_call>";
/// Tag that closes a tool call block.
const CLOSE_TAG: &str = "</tool_call>";

/// Internal state of the extraction state machine.
#[derive(Debug)]
enum ExtractorState {
    /// Accumulating regular text output.
    Text,
    /// Inside a `<tool_call>` block, accumulating JSON.
    InToolCallTag {
        /// Buffer of content inside the tag so far.
        buffer: String,
        /// The index assigned to this tool call.
        call_index: usize,
    },
}

/// Streaming state machine that detects `<tool_call>` blocks in text deltas
/// and converts them into [`NormalizedEvent`] tool call events.
///
/// # Usage
///
/// Create one extractor per streaming response. Feed each text delta through
/// [`process_delta`](Self::process_delta), and call [`flush`](Self::flush)
/// when the stream ends.
#[derive(Debug)]
pub struct ToolCallExtractor {
    state: ExtractorState,
    next_call_index: usize,
    /// Partial tag buffer for when a tag is split across deltas.
    partial_tag: String,
}

impl ToolCallExtractor {
    /// Create a new extractor in the initial text state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: ExtractorState::Text,
            next_call_index: 0,
            partial_tag: String::new(),
        }
    }

    /// Process a text delta and return zero or more normalized events.
    ///
    /// In `Text` state, emits `MessageDelta` for regular text and transitions
    /// to `InToolCallTag` when `<tool_call>` is detected.
    ///
    /// In `InToolCallTag` state, buffers content and emits `ToolCallDelta`
    /// events. When `</tool_call>` is detected, parses the JSON and emits
    /// `ToolCallComplete` (or `Error` on parse failure), then transitions
    /// back to `Text`.
    pub fn process_delta(&mut self, text: &str) -> Vec<NormalizedEvent> {
        let mut events = Vec::new();

        // If we had a partial tag from a previous delta, prepend it.
        let input = if self.partial_tag.is_empty() {
            text.to_string()
        } else {
            let combined = format!("{}{text}", self.partial_tag);
            self.partial_tag.clear();
            combined
        };

        let mut remaining = input.as_str();

        while !remaining.is_empty() {
            match &mut self.state {
                ExtractorState::Text => {
                    // Check for the open tag.
                    if let Some(tag_start) = remaining.find(OPEN_TAG) {
                        // Emit any text before the tag.
                        if tag_start > 0 {
                            events.push(NormalizedEvent::MessageDelta {
                                text: remaining[..tag_start].to_string(),
                            });
                        }
                        let call_index = self.next_call_index;
                        self.next_call_index += 1;
                        self.state = ExtractorState::InToolCallTag {
                            buffer: String::new(),
                            call_index,
                        };
                        remaining = &remaining[tag_start + OPEN_TAG.len()..];
                    } else if could_be_partial_tag(remaining, OPEN_TAG) {
                        // The tail of remaining might be the start of a tag
                        // split across deltas. Emit the safe prefix as text
                        // and buffer only the overlapping suffix.
                        let overlap = tag_prefix_overlap(remaining, OPEN_TAG);
                        let safe_len = remaining.len() - overlap;
                        if safe_len > 0 {
                            events.push(NormalizedEvent::MessageDelta {
                                text: remaining[..safe_len].to_string(),
                            });
                        }
                        self.partial_tag.push_str(&remaining[safe_len..]);
                        remaining = "";
                    } else {
                        // No tag found; emit all text.
                        events.push(NormalizedEvent::MessageDelta {
                            text: remaining.to_string(),
                        });
                        remaining = "";
                    }
                }
                ExtractorState::InToolCallTag { buffer, call_index } => {
                    let call_index = *call_index;
                    // Check for the close tag.
                    if let Some(tag_start) = remaining.find(CLOSE_TAG) {
                        buffer.push_str(&remaining[..tag_start]);
                        let json_str = buffer.trim().to_string();

                        // Try to parse the tool call JSON.
                        events.extend(parse_tool_call_json(&json_str, call_index));

                        remaining = &remaining[tag_start + CLOSE_TAG.len()..];
                        self.state = ExtractorState::Text;
                    } else if could_be_partial_tag(remaining, CLOSE_TAG) {
                        // Might be a partial close tag. Check the tail.
                        // Find how much of the end of remaining could be a
                        // prefix of CLOSE_TAG.
                        let overlap = tag_prefix_overlap(remaining, CLOSE_TAG);
                        if overlap > 0 && overlap < remaining.len() {
                            let safe = &remaining[..remaining.len() - overlap];
                            buffer.push_str(safe);
                            events.push(NormalizedEvent::ToolCallDelta {
                                call_index,
                                id: None,
                                name: None,
                                arguments_delta: Some(safe.to_string()),
                            });
                            self.partial_tag
                                .push_str(&remaining[remaining.len() - overlap..]);
                        } else {
                            // Entire remaining is a partial close tag prefix.
                            self.partial_tag.push_str(remaining);
                        }
                        remaining = "";
                    } else {
                        // No close tag; buffer and emit delta.
                        buffer.push_str(remaining);
                        events.push(NormalizedEvent::ToolCallDelta {
                            call_index,
                            id: None,
                            name: None,
                            arguments_delta: Some(remaining.to_string()),
                        });
                        remaining = "";
                    }
                }
            }
        }

        events
    }

    /// Flush any remaining state at the end of the stream.
    ///
    /// If we are mid-tag, emits an error event. If there is buffered partial
    /// tag text that turned out to not be a tag, emits it as a `MessageDelta`.
    pub fn flush(&mut self) -> Vec<NormalizedEvent> {
        let mut events = Vec::new();

        // Drain the partial tag buffer first.
        let partial = std::mem::take(&mut self.partial_tag);

        // Replace state with Text so we can inspect the old state by value.
        let old_state = std::mem::replace(&mut self.state, ExtractorState::Text);

        match old_state {
            ExtractorState::Text => {
                // Any buffered partial tag text is just regular text.
                if !partial.is_empty() {
                    events.push(NormalizedEvent::MessageDelta { text: partial });
                }
            }
            ExtractorState::InToolCallTag {
                mut buffer,
                call_index,
            } => {
                buffer.push_str(&partial);
                if !buffer.is_empty() {
                    events.push(NormalizedEvent::Error {
                        message: format!(
                            "Unterminated tool call block (index {call_index}): {}",
                            buffer.chars().take(200).collect::<String>()
                        ),
                        code: Some("tool_call_parse_error".to_string()),
                    });
                }
            }
        }

        events
    }
}

impl Default for ToolCallExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse the JSON content of a `<tool_call>` block and emit the appropriate
/// events (`ToolCallComplete` on success, `Error` on failure).
fn parse_tool_call_json(json_str: &str, call_index: usize) -> Vec<NormalizedEvent> {
    let mut events = Vec::new();
    let id = format!("call_{}", Uuid::new_v4());

    match serde_json::from_str::<serde_json::Value>(json_str) {
        Ok(parsed) => {
            let name = parsed
                .get("name")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown")
                .to_string();

            let arguments = parsed
                .get("input")
                .or_else(|| parsed.get("arguments"))
                .or_else(|| parsed.get("parameters"))
                .cloned()
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

            let arguments_json =
                serde_json::to_string(&arguments).unwrap_or_else(|_| "{}".to_string());

            events.push(NormalizedEvent::ToolCallComplete {
                call_index,
                id,
                name,
                arguments_json,
            });
        }
        Err(e) => {
            events.push(NormalizedEvent::Error {
                message: format!("Failed to parse tool call JSON: {e}"),
                code: Some("tool_call_parse_error".to_string()),
            });
        }
    }

    events
}

/// Check if the tail of `text` could be the beginning of `tag`.
///
/// Returns `true` if some non-empty suffix of `text` matches a prefix of `tag`.
fn could_be_partial_tag(text: &str, tag: &str) -> bool {
    tag_prefix_overlap(text, tag) > 0
}

/// Return the length of the longest suffix of `text` that is a prefix of `tag`.
fn tag_prefix_overlap(text: &str, tag: &str) -> usize {
    let text_bytes = text.as_bytes();
    let tag_bytes = tag.as_bytes();
    let max_check = text_bytes.len().min(tag_bytes.len());

    for overlap in (1..=max_check).rev() {
        if text_bytes[text_bytes.len() - overlap..] == tag_bytes[..overlap] {
            return overlap;
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text_passthrough() {
        let mut extractor = ToolCallExtractor::new();
        let events = extractor.process_delta("Hello, world!");
        assert_eq!(events.len(), 1);
        assert!(
            matches!(&events[0], NormalizedEvent::MessageDelta { text } if text == "Hello, world!")
        );
    }

    #[test]
    fn test_complete_tool_call_in_single_delta() {
        let mut extractor = ToolCallExtractor::new();
        let events = extractor
            .process_delta("<tool_call>\n{\"name\": \"get_weather\", \"input\": {\"location\": \"NYC\"}}\n</tool_call>");

        // Should have a ToolCallComplete event.
        let complete = events
            .iter()
            .find(|e| matches!(e, NormalizedEvent::ToolCallComplete { .. }));
        assert!(complete.is_some());

        if let Some(NormalizedEvent::ToolCallComplete {
            name,
            arguments_json,
            call_index,
            ..
        }) = complete
        {
            assert_eq!(name, "get_weather");
            assert_eq!(*call_index, 0);
            let args: serde_json::Value = serde_json::from_str(arguments_json).unwrap();
            assert_eq!(args["location"], "NYC");
        }
    }

    #[test]
    fn test_text_before_and_after_tool_call() {
        let mut extractor = ToolCallExtractor::new();
        let events = extractor.process_delta(
            "Let me check. <tool_call>\n{\"name\": \"search\", \"input\": {}}\n</tool_call> Done!",
        );

        let text_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, NormalizedEvent::MessageDelta { .. }))
            .collect();
        assert!(text_events.len() >= 2);

        // First text should be "Let me check. "
        assert!(
            matches!(&text_events[0], NormalizedEvent::MessageDelta { text } if text == "Let me check. ")
        );
        // Last text should be " Done!"
        assert!(
            matches!(text_events.last().unwrap(), NormalizedEvent::MessageDelta { text } if text == " Done!")
        );
    }

    #[test]
    fn test_tag_split_across_deltas() {
        let mut extractor = ToolCallExtractor::new();

        // Split the open tag across two deltas.
        let events1 = extractor.process_delta("Hello <tool_");
        // The "<tool_" part should be buffered, "Hello " should be emitted.
        let text_events: Vec<_> = events1
            .iter()
            .filter(|e| matches!(e, NormalizedEvent::MessageDelta { .. }))
            .collect();
        assert!(!text_events.is_empty());

        let events2 =
            extractor.process_delta("call>\n{\"name\": \"test\", \"input\": {}}\n</tool_call>");
        let complete = events2
            .iter()
            .find(|e| matches!(e, NormalizedEvent::ToolCallComplete { .. }));
        assert!(complete.is_some());
    }

    #[test]
    fn test_malformed_json_emits_error() {
        let mut extractor = ToolCallExtractor::new();
        let events = extractor.process_delta("<tool_call>\nnot valid json\n</tool_call>");

        let error = events
            .iter()
            .find(|e| matches!(e, NormalizedEvent::Error { .. }));
        assert!(error.is_some());
        if let Some(NormalizedEvent::Error { code, .. }) = error {
            assert_eq!(code.as_deref(), Some("tool_call_parse_error"));
        }
    }

    #[test]
    fn test_multiple_tool_calls() {
        let mut extractor = ToolCallExtractor::new();
        let input = "<tool_call>\n{\"name\": \"a\", \"input\": {}}\n</tool_call>\
                     <tool_call>\n{\"name\": \"b\", \"input\": {}}\n</tool_call>";
        let events = extractor.process_delta(input);

        let completes: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, NormalizedEvent::ToolCallComplete { .. }))
            .collect();
        assert_eq!(completes.len(), 2);

        if let NormalizedEvent::ToolCallComplete {
            call_index, name, ..
        } = &completes[0]
        {
            assert_eq!(*call_index, 0);
            assert_eq!(name, "a");
        }
        if let NormalizedEvent::ToolCallComplete {
            call_index, name, ..
        } = &completes[1]
        {
            assert_eq!(*call_index, 1);
            assert_eq!(name, "b");
        }
    }

    #[test]
    fn test_flush_unterminated_tag() {
        let mut extractor = ToolCallExtractor::new();
        let _ = extractor.process_delta("<tool_call>\n{\"name\": \"test\"");
        let events = extractor.flush();

        let error = events
            .iter()
            .find(|e| matches!(e, NormalizedEvent::Error { .. }));
        assert!(error.is_some());
    }

    #[test]
    fn test_flush_partial_tag_as_text() {
        let mut extractor = ToolCallExtractor::new();
        let events1 = extractor.process_delta("Hello <tool_");
        // Should buffer the partial tag.
        assert!(
            events1
                .iter()
                .any(|e| matches!(e, NormalizedEvent::MessageDelta { text } if text == "Hello "))
        );

        let events2 = extractor.flush();
        // The partial tag should be emitted as text since it wasn't completed.
        assert!(
            events2
                .iter()
                .any(|e| matches!(e, NormalizedEvent::MessageDelta { text } if text == "<tool_"))
        );
    }

    #[test]
    fn test_tag_prefix_overlap() {
        assert_eq!(tag_prefix_overlap("abc<", "<tool_call>"), 1);
        assert_eq!(tag_prefix_overlap("abc<tool", "<tool_call>"), 5);
        // Full tag as suffix: overlap equals full tag length.
        // In practice, `find()` catches this before we check overlap.
        assert_eq!(tag_prefix_overlap("abc<tool_call>", "<tool_call>"), 11);
        assert_eq!(tag_prefix_overlap("hello", "<tool_call>"), 0);
    }

    #[test]
    fn test_arguments_field_fallback() {
        let mut extractor = ToolCallExtractor::new();
        // Use "arguments" instead of "input".
        let events = extractor.process_delta(
            "<tool_call>\n{\"name\": \"test\", \"arguments\": {\"key\": \"val\"}}\n</tool_call>",
        );
        let complete = events
            .iter()
            .find(|e| matches!(e, NormalizedEvent::ToolCallComplete { .. }));
        assert!(complete.is_some());
        if let Some(NormalizedEvent::ToolCallComplete { arguments_json, .. }) = complete {
            let args: serde_json::Value = serde_json::from_str(arguments_json).unwrap();
            assert_eq!(args["key"], "val");
        }
    }
}
