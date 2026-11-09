//! SSE decoder state machine for Anthropic Messages API streaming.
//!
//! Maps Anthropic streaming events to [`NormalizedEvent`] using a stateful
//! decoder that tracks content block types and accumulates tool call JSON.

use std::collections::BTreeMap;

use tracing::{debug, warn};

use super::anthropic_types::*;
use crate::normalized::NormalizedEvent;

/// Tracks state across a streaming Anthropic response.
///
/// The decoder processes [`StreamEvent`]s in order and emits zero or more
/// [`NormalizedEvent`]s for each input event. It maintains accumulators
/// for tool call JSON assembly and tracks the current content block type.
#[derive(Debug)]
pub struct StreamState {
    /// Unique identifier for this request/response pair.
    request_id: String,
    /// Index of the currently active content block.
    current_block_index: Option<usize>,
    /// Type of the currently active content block.
    current_block_type: Option<BlockType>,
    /// Accumulators for in-progress tool calls, keyed by block index.
    tool_accumulators: BTreeMap<usize, ToolAccum>,
    /// Running count of completed tool calls (used as `call_index`).
    tool_call_counter: usize,
    /// Accumulated usage across the stream.
    usage: UsageResponse,
    /// Actual provider usage was present; a default accumulator is not a report.
    usage_reported: bool,
    input_usage_reported: bool,
    final_output_usage_reported: bool,
    /// Whether we have emitted a `StreamStart` event.
    started: bool,
}

/// The type of a content block being streamed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockType {
    Text,
    ToolUse,
    Thinking,
}

/// Accumulator for assembling a tool call from streaming deltas.
#[derive(Debug, Default)]
struct ToolAccum {
    /// Tool call identifier.
    id: String,
    /// Tool name.
    name: String,
    /// Accumulated partial JSON for the tool input.
    input_json: String,
    /// Index of this tool call in the current response.
    call_index: usize,
}

impl StreamState {
    /// Create a new stream state for the given request ID.
    #[must_use]
    pub fn new(request_id: String) -> Self {
        Self {
            request_id,
            current_block_index: None,
            current_block_type: None,
            tool_accumulators: BTreeMap::new(),
            tool_call_counter: 0,
            usage: UsageResponse::default(),
            usage_reported: false,
            input_usage_reported: false,
            final_output_usage_reported: false,
            started: false,
        }
    }

    /// Process a single Anthropic streaming event and return normalized events.
    ///
    /// Most input events produce zero or one output events; `MessageStop`
    /// produces a `Usage` event followed by `Done`.
    pub fn process_event(&mut self, event: StreamEvent) -> Vec<NormalizedEvent> {
        match event {
            StreamEvent::MessageStart { message } => self.handle_message_start(message),
            StreamEvent::ContentBlockStart {
                index,
                content_block,
            } => self.handle_content_block_start(index, content_block),
            StreamEvent::ContentBlockDelta { index, delta } => {
                self.handle_content_block_delta(index, delta)
            }
            StreamEvent::ContentBlockStop { index } => self.handle_content_block_stop(index),
            StreamEvent::MessageDelta { delta: _, usage } => {
                self.handle_message_delta(usage);
                Vec::new()
            }
            StreamEvent::MessageStop => self.handle_message_stop(),
            StreamEvent::Error { error } => vec![NormalizedEvent::Error {
                message: error.message,
                code: Some(error.error_type),
            }],
            StreamEvent::Ping => Vec::new(),
        }
    }

    fn handle_message_start(&mut self, message: MessageStartData) -> Vec<NormalizedEvent> {
        let mut events = Vec::new();

        if !self.started {
            self.started = true;
            events.push(NormalizedEvent::StreamStart {
                request_id: self.request_id.clone(),
            });
        }

        // Store initial usage (input tokens).
        if let Some(usage) = message.usage {
            self.usage_reported = true;
            self.input_usage_reported = true;
            self.merge_usage(&usage);
        }

        debug!(
            message_id = %message.id,
            model = %message.model,
            "Anthropic stream started"
        );

        events
    }

    fn handle_content_block_start(
        &mut self,
        index: usize,
        content_block: ContentBlockStartData,
    ) -> Vec<NormalizedEvent> {
        self.current_block_index = Some(index);

        match content_block {
            ContentBlockStartData::Text { .. } => {
                self.current_block_type = Some(BlockType::Text);
            }
            ContentBlockStartData::ToolUse { id, name } => {
                self.current_block_type = Some(BlockType::ToolUse);
                let call_index = self.tool_call_counter;
                self.tool_accumulators.insert(
                    index,
                    ToolAccum {
                        id,
                        name,
                        input_json: String::new(),
                        call_index,
                    },
                );
                self.tool_call_counter += 1;
            }
            ContentBlockStartData::Thinking { .. } => {
                self.current_block_type = Some(BlockType::Thinking);
            }
        }

        // No normalized events emitted on block start.
        Vec::new()
    }

    fn handle_content_block_delta(
        &mut self,
        index: usize,
        delta: ContentBlockDeltaData,
    ) -> Vec<NormalizedEvent> {
        match delta {
            ContentBlockDeltaData::TextDelta { text } => {
                vec![NormalizedEvent::MessageDelta { text }]
            }
            ContentBlockDeltaData::InputJsonDelta { partial_json } => {
                if let Some(accum) = self.tool_accumulators.get_mut(&index) {
                    accum.input_json.push_str(&partial_json);
                    vec![NormalizedEvent::ToolCallDelta {
                        call_index: accum.call_index,
                        id: Some(accum.id.clone()),
                        name: Some(accum.name.clone()),
                        arguments_delta: Some(partial_json),
                    }]
                } else {
                    warn!(index, "Received input_json_delta for unknown tool block");
                    Vec::new()
                }
            }
            ContentBlockDeltaData::ThinkingDelta { thinking } => {
                vec![NormalizedEvent::ThinkingDelta { text: thinking }]
            }
        }
    }

    fn handle_content_block_stop(&mut self, index: usize) -> Vec<NormalizedEvent> {
        let events = if self.current_block_type == Some(BlockType::ToolUse) {
            if let Some(accum) = self.tool_accumulators.remove(&index) {
                vec![NormalizedEvent::ToolCallComplete {
                    call_index: accum.call_index,
                    id: accum.id,
                    name: accum.name,
                    arguments_json: accum.input_json,
                }]
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // Clear current block tracking.
        if self.current_block_index == Some(index) {
            self.current_block_index = None;
            self.current_block_type = None;
        }

        events
    }

    fn handle_message_delta(&mut self, usage: Option<MessageDeltaUsage>) {
        if let Some(usage) = usage {
            self.usage_reported = true;
            self.input_usage_reported |= usage.input_tokens.is_some();
            self.final_output_usage_reported = true;
            // The delta omits input counts when unchanged. Preserve the full
            // message_start validation and existing cumulative/cache merger.
            self.merge_usage(&UsageResponse {
                input_tokens: usage.input_tokens.unwrap_or(0),
                output_tokens: usage.output_tokens,
                cache_creation_input_tokens: usage.cache_creation_input_tokens,
                cache_read_input_tokens: usage.cache_read_input_tokens,
            });
        }
    }

    fn handle_message_stop(&mut self) -> Vec<NormalizedEvent> {
        if !self.usage_reported {
            return vec![NormalizedEvent::Done];
        }
        // The normalized counter type cannot represent a missing input/output
        // number. A partial native report must not manufacture measured zero.
        if !self.input_usage_reported || !self.final_output_usage_reported {
            return vec![
                NormalizedEvent::Error {
                    message: "Provider usage did not report complete input and final output counts"
                        .into(),
                    code: Some("provider_usage_incomplete".into()),
                },
                NormalizedEvent::Done,
            ];
        }
        // Anthropic reports uncached input separately from cache reads/writes.
        // Normalized prompt usage includes all three, with cache fields as
        // breakdowns rather than additional tokens.
        let prompt_tokens = self
            .usage
            .input_tokens
            .checked_add(self.usage.cache_read_input_tokens.unwrap_or(0))
            .and_then(|input| {
                input.checked_add(self.usage.cache_creation_input_tokens.unwrap_or(0))
            });
        let completion_tokens = self.usage.output_tokens;
        let total_tokens = prompt_tokens.and_then(|input| input.checked_add(completion_tokens));
        let (Some(prompt_tokens), Some(total_tokens)) = (prompt_tokens, total_tokens) else {
            return vec![
                NormalizedEvent::Error {
                    message: "Provider usage exceeds the supported counter range".into(),
                    code: Some("provider_usage_overflow".into()),
                },
                NormalizedEvent::Done,
            ];
        };

        vec![
            NormalizedEvent::Usage {
                prompt_tokens,
                completion_tokens,
                total_tokens,
                cached_tokens: self.usage.cache_read_input_tokens,
                uncached_input_tokens: Some(self.usage.input_tokens),
                cache_creation_tokens: self.usage.cache_creation_input_tokens,
            },
            NormalizedEvent::Done,
        ]
    }

    /// Merge incoming usage data into our running totals.
    ///
    /// Anthropic splits usage across `message_start` (input tokens) and
    /// `message_delta` (output tokens, cache tokens), so we accumulate.
    fn merge_usage(&mut self, incoming: &UsageResponse) {
        if incoming.input_tokens > 0 {
            self.usage.input_tokens = incoming.input_tokens;
        }
        if incoming.output_tokens > 0 {
            self.usage.output_tokens = incoming.output_tokens;
        }
        if incoming.cache_creation_input_tokens.is_some() {
            self.usage.cache_creation_input_tokens = incoming.cache_creation_input_tokens;
        }
        if incoming.cache_read_input_tokens.is_some() {
            self.usage.cache_read_input_tokens = incoming.cache_read_input_tokens;
        }
    }
}

/// Parse a raw SSE line pair into a [`StreamEvent`].
///
/// Anthropic's SSE format uses `event: <type>` and `data: <json>` lines.
/// This function expects the JSON `data:` line content (after stripping
/// the `data: ` prefix). The `event:` line is used by the HTTP client
/// to identify the event type, but the JSON payload itself is
/// self-describing via the `"type"` field.
///
/// Returns `None` for empty lines, comments, or unparseable data.
pub fn parse_sse_line(line: &str) -> Option<StreamEvent> {
    let line = line.trim();

    // Skip empty lines, comments, and event-type lines.
    if line.is_empty() || line.starts_with(':') || line.starts_with("event:") {
        return None;
    }

    // Extract the JSON payload from `data: {...}`.
    let json_str = line.strip_prefix("data: ").unwrap_or(line);

    // The Anthropic API sends `data: [DONE]` at the very end in some cases.
    if json_str == "[DONE]" {
        return None;
    }

    match serde_json::from_str::<StreamEvent>(json_str) {
        Ok(event) => Some(event),
        Err(e) => {
            warn!(
                error = %e,
                line = %json_str,
                "Failed to parse Anthropic SSE event"
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_state_emits_start_once() {
        let mut state = StreamState::new("req-1".to_string());

        let events = state.process_event(StreamEvent::MessageStart {
            message: MessageStartData {
                id: "msg_1".to_string(),
                model: "claude-sonnet-4-20250514".to_string(),
                role: "assistant".to_string(),
                usage: Some(UsageResponse {
                    input_tokens: 100,
                    output_tokens: 0,
                    ..Default::default()
                }),
            },
        });

        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            NormalizedEvent::StreamStart { request_id } if request_id == "req-1"
        ));

        // Second message_start should not emit StreamStart again.
        let events2 = state.process_event(StreamEvent::MessageStart {
            message: MessageStartData {
                id: "msg_1".to_string(),
                model: "claude-sonnet-4-20250514".to_string(),
                role: "assistant".to_string(),
                usage: None,
            },
        });
        assert!(events2.is_empty());
    }

    #[test]
    fn test_text_delta_produces_message_delta() {
        let mut state = StreamState::new("req-2".to_string());

        let events = state.process_event(StreamEvent::ContentBlockDelta {
            index: 0,
            delta: ContentBlockDeltaData::TextDelta {
                text: "Hello".to_string(),
            },
        });

        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            NormalizedEvent::MessageDelta { text } if text == "Hello"
        ));
    }

    #[test]
    fn test_tool_call_lifecycle() {
        let mut state = StreamState::new("req-3".to_string());

        // Start tool block.
        let events = state.process_event(StreamEvent::ContentBlockStart {
            index: 1,
            content_block: ContentBlockStartData::ToolUse {
                id: "toolu_1".to_string(),
                name: "get_weather".to_string(),
            },
        });
        assert!(events.is_empty());

        // Accumulate JSON.
        let events = state.process_event(StreamEvent::ContentBlockDelta {
            index: 1,
            delta: ContentBlockDeltaData::InputJsonDelta {
                partial_json: r#"{"city":"#.to_string(),
            },
        });
        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0], NormalizedEvent::ToolCallDelta { .. }));

        let events = state.process_event(StreamEvent::ContentBlockDelta {
            index: 1,
            delta: ContentBlockDeltaData::InputJsonDelta {
                partial_json: r#""NYC"}"#.to_string(),
            },
        });
        assert_eq!(events.len(), 1);

        // Complete tool block.
        let events = state.process_event(StreamEvent::ContentBlockStop { index: 1 });
        assert_eq!(events.len(), 1);
        match &events[0] {
            NormalizedEvent::ToolCallComplete {
                id,
                name,
                arguments_json,
                ..
            } => {
                assert_eq!(id, "toolu_1");
                assert_eq!(name, "get_weather");
                assert_eq!(arguments_json, r#"{"city":"NYC"}"#);
            }
            other => panic!("Expected ToolCallComplete, got {other:?}"),
        }
    }

    #[test]
    fn test_message_stop_emits_usage_and_done() {
        let mut state = StreamState::new("req-4".to_string());

        // Seed some usage.
        state.process_event(StreamEvent::MessageStart {
            message: MessageStartData {
                id: "msg_1".to_string(),
                model: "claude-sonnet-4-20250514".to_string(),
                role: "assistant".to_string(),
                usage: Some(UsageResponse {
                    input_tokens: 50,
                    output_tokens: 0,
                    cache_read_input_tokens: Some(20),
                    cache_creation_input_tokens: None,
                }),
            },
        });

        state.process_event(StreamEvent::MessageDelta {
            delta: MessageDeltaData {
                stop_reason: Some("end_turn".to_string()),
                stop_sequence: None,
            },
            usage: Some(MessageDeltaUsage {
                input_tokens: None,
                output_tokens: 30,
                cache_read_input_tokens: None,
                cache_creation_input_tokens: None,
            }),
        });

        let events = state.process_event(StreamEvent::MessageStop);
        assert_eq!(events.len(), 2);
        assert!(matches!(
            &events[0],
            NormalizedEvent::Usage {
                prompt_tokens: 70,
                uncached_input_tokens: Some(50),
                completion_tokens: 30,
                total_tokens: 100,
                cached_tokens: Some(20),
                ..
            }
        ));
        assert!(matches!(&events[1], NormalizedEvent::Done));
    }

    #[test]
    fn absent_usage_does_not_become_a_measured_zero_report() {
        let mut state = StreamState::new("owned-absent".into());
        state.process_event(StreamEvent::MessageStart {
            message: MessageStartData {
                id: "owned-message".into(),
                model: "owned-model".into(),
                role: "assistant".into(),
                usage: None,
            },
        });
        let events = state.process_event(StreamEvent::MessageStop);
        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0], NormalizedEvent::Done));
    }

    #[test]
    fn native_uncached_only_report_does_not_invent_cache_counts() {
        let mut state = StreamState::new("owned-uncached".into());
        state.process_event(StreamEvent::MessageStart {
            message: MessageStartData {
                id: "owned-message".into(), model: "owned-model".into(), role: "assistant".into(),
                usage: Some(UsageResponse { input_tokens: 7, ..Default::default() }),
            },
        });
        state.process_event(StreamEvent::MessageDelta {
            delta: MessageDeltaData { stop_reason: Some("end_turn".into()), stop_sequence: None },
            usage: Some(MessageDeltaUsage {
                input_tokens: None, output_tokens: 5,
                cache_read_input_tokens: None, cache_creation_input_tokens: None,
            }),
        });
        let events = state.process_event(StreamEvent::MessageStop);
        assert!(matches!(&events[0], NormalizedEvent::Usage {
            prompt_tokens: 7, completion_tokens: 5, uncached_input_tokens: Some(7),
            cached_tokens: None, cache_creation_tokens: None, ..
        }));
    }

    #[test]
    fn actual_zero_usage_still_reports_zero_and_optional_cache_availability() {
        let mut state = StreamState::new("owned-zero".into());
        state.process_event(StreamEvent::MessageStart {
            message: MessageStartData {
                id: "owned-message".into(),
                model: "owned-model".into(),
                role: "assistant".into(),
                usage: Some(UsageResponse {
                    input_tokens: 0,
                    output_tokens: 0,
                    cache_read_input_tokens: Some(0),
                    cache_creation_input_tokens: None,
                }),
            },
        });
        state.process_event(StreamEvent::MessageDelta {
            delta: MessageDeltaData {
                stop_reason: Some("end_turn".into()),
                stop_sequence: None,
            },
            usage: Some(MessageDeltaUsage {
                input_tokens: None,
                output_tokens: 0,
                cache_read_input_tokens: None,
                cache_creation_input_tokens: None,
            }),
        });
        let events = state.process_event(StreamEvent::MessageStop);
        assert!(matches!(
            &events[0],
            NormalizedEvent::Usage {
                prompt_tokens: 0,
                uncached_input_tokens: Some(0),
                completion_tokens: 0,
                cached_tokens: Some(0),
                cache_creation_tokens: None,
                ..
            }
        ));
    }

    #[test]
    fn partial_native_usage_refuses_instead_of_filling_missing_input_with_zero() {
        let mut state = StreamState::new("owned-partial".into());
        state.process_event(StreamEvent::MessageDelta {
            delta: MessageDeltaData {
                stop_reason: Some("end_turn".into()),
                stop_sequence: None,
            },
            usage: Some(MessageDeltaUsage {
                input_tokens: None,
                output_tokens: 5,
                cache_read_input_tokens: None,
                cache_creation_input_tokens: None,
            }),
        });
        let events = state.process_event(StreamEvent::MessageStop);
        assert!(
            matches!(&events[0], NormalizedEvent::Error { code:Some(code),.. } if code == "provider_usage_incomplete")
        );
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, NormalizedEvent::Usage { .. }))
        );
    }

    #[test]
    fn test_parse_sse_line_empty_and_comments() {
        assert!(parse_sse_line("").is_none());
        assert!(parse_sse_line(": keep-alive").is_none());
        assert!(parse_sse_line("event: message_start").is_none());
        assert!(parse_sse_line("data: [DONE]").is_none());
    }

    #[test]
    fn test_parse_sse_line_valid_ping() {
        let event = parse_sse_line(r#"data: {"type": "ping"}"#);
        assert!(matches!(event, Some(StreamEvent::Ping)));
    }

    #[test]
    fn test_thinking_delta() {
        let mut state = StreamState::new("req-5".to_string());

        state.process_event(StreamEvent::ContentBlockStart {
            index: 0,
            content_block: ContentBlockStartData::Thinking {
                thinking: String::new(),
            },
        });

        let events = state.process_event(StreamEvent::ContentBlockDelta {
            index: 0,
            delta: ContentBlockDeltaData::ThinkingDelta {
                thinking: "Let me think...".to_string(),
            },
        });

        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            NormalizedEvent::ThinkingDelta { text } if text == "Let me think..."
        ));
    }
}
