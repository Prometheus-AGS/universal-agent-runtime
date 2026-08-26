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
    pub fn new(
        config: ClientConfig,
        model: String,
        parallel_tool_calls: Option<bool>,
    ) -> anyhow::Result<Self> {
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

fn build_chat_request(
    model: &str,
    parallel_tool_calls: Option<bool>,
    req: &LlmRequest,
) -> anyhow::Result<ChatCompletionRequest> {
    let messages = convert_messages(&req.messages)?;
    let tools = convert_tools(&req.tools);

    let mut chat_req = ChatCompletionRequest::default();
    chat_req.model = model.to_owned();
    chat_req.messages = messages;
    chat_req.tools = if tools.is_empty() { None } else { Some(tools) };
    chat_req.parallel_tool_calls = parallel_tool_calls;
    chat_req.stream_options = Some(StreamOptions {
        include_usage: Some(true),
    });

    if chat_req.tools.is_some() {
        chat_req.tool_choice = Some(ToolChoice::Mode(ToolChoiceMode::Auto));
    }

    // CH-04: per-model dialect params (extended-thinking budgets, reasoning
    // persistence toggles) computed by `PromptDialectEngine`, merged
    // verbatim into the outbound request body.
    chat_req.extra_body = req.extra_params.clone();
    Ok(chat_req)
}

#[async_trait::async_trait]
impl LlmDriver for LiterLlmDriver {
    #[tracing::instrument(
        name = "llm.call",
        skip(self, req),
        fields(model = %self.model),
    )]
    async fn stream(
        &self,
        req: LlmRequest,
    ) -> anyhow::Result<std::pin::Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>>
    {
        let chat_req = build_chat_request(&self.model, self.parallel_tool_calls, &req)?;

        // Collect chunks eagerly into a Vec, then stream owned events.
        // This is necessary because liter-llm's BoxStream borrows from the client
        // and cannot be moved into a 'static stream directly.
        let metrics_model = self.model.clone();

        // Time the full LLM call (request → all chunks collected) and record it as
        // a per-call latency histogram.
        let call_start = std::time::Instant::now();
        let chunk_stream = self.client.chat_stream(chat_req).await?;
        let chunks: Vec<Result<ChatCompletionChunk, _>> = chunk_stream.collect().await;
        {
            let (provider, model_name) = metrics_model
                .split_once('/')
                .unwrap_or(("unknown", &metrics_model));
            telemetry_metrics::record_llm_call_latency(
                provider,
                model_name,
                call_start.elapsed().as_secs_f64(),
            );
        }

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
                    let cached_read = usage
                        .prompt_tokens_details
                        .as_ref()
                        .map(|d| d.cached_tokens);
                    #[expect(clippy::cast_possible_truncation, reason = "token counts fit in u32")]
                    {
                        yield Ok(NormalizedEvent::Usage {
                            prompt_tokens: usage.prompt_tokens as u32,
                            completion_tokens: usage.completion_tokens as u32,
                            total_tokens: usage.total_tokens as u32,
                            cached_tokens: cached_read.map(|t| t as u32),
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
                    // Cache-read tokens only: liter exposes the cached (read)
                    // portion via prompt_tokens_details; cache-creation/write is
                    // folded into provider billing and not separately reported.
                    if let Some(read) = cached_read {
                        #[expect(
                            clippy::cast_possible_truncation,
                            reason = "token counts fit in u32"
                        )]
                        telemetry_metrics::record_cache_tokens(provider, model_name, 0, read as u32);
                    }
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

#[cfg(test)]
mod prompt_caching_tests {
    use super::*;
    use crate::llm::anthropic_cache::CacheStrategy;

    fn request(cache_strategy: Option<CacheStrategy>) -> LlmRequest {
        LlmRequest {
            messages: vec![serde_json::json!({"role": "user", "content": "hello"})],
            tools: Vec::new(),
            cache_strategy,
            thinking_config: None,
            anthropic_system: None,
            extra_params: Some(serde_json::json!({"temperature": 0.2})),
        }
    }

    #[test]
    fn openai_compatible_body_is_unchanged_by_uar_cache_strategy() {
        let enabled = build_chat_request(
            "openai/gpt-test",
            Some(true),
            &request(Some(CacheStrategy::default())),
        )
        .expect("enabled request");
        let disabled = build_chat_request("openai/gpt-test", Some(true), &request(None))
            .expect("disabled request");

        assert_eq!(
            serde_json::to_value(enabled).expect("serialize enabled request"),
            serde_json::to_value(disabled).expect("serialize disabled request"),
            "UAR prompt-caching policy must not alter OpenAI-compatible bodies"
        );
    }
}

/// Convert UAR's JSON messages to liter-llm's typed `Message` enum.
fn convert_messages(messages: &[serde_json::Value]) -> anyhow::Result<Vec<liter_llm::Message>> {
    messages
        .iter()
        .map(|msg| {
            serde_json::from_value::<liter_llm::Message>(msg.clone())
                .map_err(|e| anyhow::anyhow!("Failed to convert message to liter-llm format: {e}"))
        })
        .collect()
}

/// Convert UAR's JSON tool schemas to liter-llm's typed `ChatCompletionTool`.
fn convert_tools(tools: &[serde_json::Value]) -> Vec<liter_llm::ChatCompletionTool> {
    tools
        .iter()
        .filter_map(|tool| {
            serde_json::from_value::<liter_llm::ChatCompletionTool>(tool.clone()).ok()
        })
        .collect()
}
