//! LLM driver implementation backed by [`liter_llm::DefaultClient`].
//!
//! This replaces the legacy `ChatCompletionsDriver` and `ResponsesDriver` with a
//! single unified driver that handles all 142+ providers via liter-llm's provider
//! abstraction, including tool-call normalization across Anthropic, Google, Mistral,
//! and all OpenAI-compatible APIs.

use std::collections::BTreeMap;
use std::sync::Arc;

use futures::{Stream, StreamExt};
use liter_llm::{
    ChatCompletionChunk, ChatCompletionRequest, ClientConfig, DefaultClient, FinishReason,
    LlmClient, StreamOptions, ToolChoice, ToolChoiceMode,
};

use crate::normalized::NormalizedEvent;
use crate::uar::telemetry::metrics as telemetry_metrics;

use super::{LlmDriver, LlmRequest};

/// Accumulated state for a streaming tool call being assembled from deltas.
#[derive(Default)]
struct ToolAccum {
    id: Option<String>,
    name: Option<String>,
    args: String,
}

/// LLM driver powered by liter-llm's `DefaultClient`.
///
/// Handles provider auto-detection, streaming, and tool-call normalization
/// for 142+ providers through a single unified interface.
pub struct LiterLlmDriver {
    client: Arc<DefaultClient>,
    model: String,
    parallel_tool_calls: Option<bool>,
}

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for LiterLlmDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LiterLlmDriver")
            .field("model", &self.model)
            .field("parallel_tool_calls", &self.parallel_tool_calls)
            .finish()
    }
}

impl LiterLlmDriver {
    /// Create a new driver from a `ClientConfig` and model identifier.
    ///
    /// The `model` should use liter-llm's `provider/model` naming convention
    /// (e.g., `"openai/gpt-4o"`, `"anthropic/claude-sonnet-4"`).
    /// # Errors
    ///
    /// Returns an error if the `DefaultClient` cannot be constructed (e.g.
    /// invalid headers or provider validation failure).
    pub fn new(config: ClientConfig, model: String, parallel_tool_calls: Option<bool>) -> anyhow::Result<Self> {
        let client = Arc::new(DefaultClient::new(config, Some(&model))?);
        Ok(Self {
            client,
            model,
            parallel_tool_calls,
        })
    }

    /// Create a driver from a pre-built `DefaultClient`.
    #[must_use]
    pub fn from_client(
        client: Arc<DefaultClient>,
        model: String,
        parallel_tool_calls: Option<bool>,
    ) -> Self {
        Self {
            client,
            model,
            parallel_tool_calls,
        }
    }

    /// Get the model identifier.
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }
}

#[async_trait::async_trait]
impl LlmDriver for LiterLlmDriver {
    async fn stream(
        &self,
        req: LlmRequest,
    ) -> anyhow::Result<
        std::pin::Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>,
    > {
        let messages = convert_messages(&req.messages)?;
        let tools = convert_tools(&req.tools);

        let mut chat_req = ChatCompletionRequest::default();
        chat_req.model = self.model.clone();
        chat_req.messages = messages;
        chat_req.tools = if tools.is_empty() { None } else { Some(tools) };
        chat_req.parallel_tool_calls = self.parallel_tool_calls;
        chat_req.stream_options = Some(StreamOptions {
            include_usage: Some(true),
        });

        if chat_req.tools.is_some() {
            chat_req.tool_choice = Some(ToolChoice::Mode(ToolChoiceMode::Auto));
        }

        // Collect chunks eagerly into a Vec, then stream owned events.
        // This is necessary because liter-llm's BoxStream borrows from the client
        // and cannot be moved into a 'static stream directly.
        let metrics_model = self.model.clone();

        let chunk_stream = self.client.chat_stream(chat_req).await?;
        let chunks: Vec<Result<ChatCompletionChunk, _>> = chunk_stream.collect().await;

        let out = async_stream::stream! {
            let mut tool_accum: BTreeMap<u32, ToolAccum> = BTreeMap::new();
            let mut chunk_count: u64 = 0;
            let mut event_count: u64 = 0;

            for chunk_result in chunks {
                let chunk: ChatCompletionChunk = match chunk_result {
                    Ok(c) => c,
                    Err(e) => {
                        yield Ok(NormalizedEvent::Error {
                            message: e.to_string(),
                            code: Some("liter_llm_stream_error".to_string()),
                        });
                        break;
                    }
                };

                chunk_count += 1;

                for choice in &chunk.choices {
                    if let Some(ref text) = choice.delta.content {
                        if !text.is_empty() {
                            event_count += 1;
                            yield Ok(NormalizedEvent::MessageDelta {
                                text: text.clone(),
                            });
                        }
                    }

                    if let Some(ref tool_calls) = choice.delta.tool_calls {
                        for tc in tool_calls {
                            let idx = tc.index;
                            let accum = tool_accum.entry(idx).or_default();

                            if let Some(ref id) = tc.id {
                                accum.id = Some(id.clone());
                            }
                            if let Some(ref func) = tc.function {
                                if let Some(ref name) = func.name {
                                    accum.name = Some(name.clone());
                                }
                                if let Some(ref args) = func.arguments {
                                    accum.args.push_str(args);
                                }
                            }

                            event_count += 1;
                            yield Ok(NormalizedEvent::ToolCallDelta {
                                call_index: idx as usize,
                                id: tc.id.clone(),
                                name: tc.function.as_ref().and_then(|f| f.name.clone()),
                                arguments_delta: tc.function.as_ref().and_then(|f| f.arguments.clone()),
                            });
                        }
                    }

                    if let Some(ref reason) = choice.finish_reason {
                        if matches!(reason, FinishReason::ToolCalls) {
                            for (idx, accum) in &tool_accum {
                                if let (Some(id), Some(name)) = (&accum.id, &accum.name) {
                                    event_count += 1;
                                    yield Ok(NormalizedEvent::ToolCallComplete {
                                        call_index: *idx as usize,
                                        id: id.clone(),
                                        name: name.clone(),
                                        arguments_json: accum.args.clone(),
                                    });
                                }
                            }
                        }
                    }
                }

                if let Some(ref usage) = chunk.usage {
                    event_count += 1;
                    #[expect(clippy::cast_possible_truncation, reason = "token counts fit in u32")]
                    {
                        yield Ok(NormalizedEvent::Usage {
                            prompt_tokens: usage.prompt_tokens as u32,
                            completion_tokens: usage.completion_tokens as u32,
                            total_tokens: usage.total_tokens as u32,
                            cached_tokens: None,
                            cache_creation_tokens: None,
                        });
                    }

                    // Record LLM token metrics
                    let (provider, model_name) = metrics_model
                        .split_once('/')
                        .unwrap_or(("unknown", &metrics_model));
                    telemetry_metrics::record_llm_tokens(
                        provider,
                        model_name,
                        usage.prompt_tokens,
                        usage.completion_tokens,
                    );
                }
            }

            tracing::info!(
                total_chunks = chunk_count,
                total_events = event_count,
                "liter-llm stream complete"
            );

            yield Ok(NormalizedEvent::Done);
        };

        Ok(Box::pin(out))
    }
}

/// Convert UAR's JSON messages to liter-llm's typed `Message` enum.
fn convert_messages(
    messages: &[serde_json::Value],
) -> anyhow::Result<Vec<liter_llm::Message>> {
    messages
        .iter()
        .map(|msg| {
            serde_json::from_value::<liter_llm::Message>(msg.clone()).map_err(|e| {
                anyhow::anyhow!("Failed to convert message to liter-llm format: {e}")
            })
        })
        .collect()
}

/// Convert UAR's JSON tool schemas to liter-llm's typed `ChatCompletionTool`.
fn convert_tools(tools: &[serde_json::Value]) -> Vec<liter_llm::ChatCompletionTool> {
    tools
        .iter()
        .filter_map(|tool| serde_json::from_value::<liter_llm::ChatCompletionTool>(tool.clone()).ok())
        .collect()
}
