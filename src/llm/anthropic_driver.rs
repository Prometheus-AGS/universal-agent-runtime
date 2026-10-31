//! Native Anthropic Messages API driver.
//!
//! Speaks the Anthropic wire protocol directly via reqwest, bypassing liter-llm
//! for full support of prompt caching, extended thinking, and native streaming.

use std::pin::Pin;

use async_stream::stream;
use futures::Stream;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue, RETRY_AFTER};
use tracing::{debug, warn};

use super::anthropic_cache::CacheStrategy;
use super::anthropic_sse::AnthropicSseFramer;
use super::anthropic_streaming::StreamState;
use super::anthropic_types::*;
use super::{LlmDriver, LlmRequest, ProviderError};
use crate::normalized::NormalizedEvent;

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

fn messages_endpoint(base_url: Option<String>) -> String {
    let Some(base_url) = base_url else {
        return ANTHROPIC_API_URL.to_string();
    };
    let base_url = base_url.trim_end_matches('/');
    if base_url.ends_with("/v1/messages") {
        base_url.to_string()
    } else if base_url.ends_with("/v1") {
        format!("{base_url}/messages")
    } else {
        format!("{base_url}/v1/messages")
    }
}

/// Native Anthropic Messages API driver.
///
/// Implements [`LlmDriver`] by speaking directly to the Anthropic API
/// via reqwest streaming SSE, preserving prompt caching, extended thinking,
/// and native content block types.
pub struct AnthropicDriver {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
    max_tokens: u32,
    default_cache_strategy: Option<CacheStrategy>,
    default_thinking_config: Option<ThinkingConfig>,
}

impl std::fmt::Debug for AnthropicDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnthropicDriver")
            .field("model", &self.model)
            .field("base_url", &self.base_url)
            .finish()
    }
}

impl AnthropicDriver {
    /// Create a new Anthropic driver.
    pub fn new(
        api_key: String,
        model: String,
        base_url: Option<String>,
        max_tokens: Option<u32>,
        cache_strategy: Option<CacheStrategy>,
        thinking_config: Option<ThinkingConfig>,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: messages_endpoint(base_url),
            model,
            max_tokens: max_tokens.unwrap_or(8192),
            default_cache_strategy: cache_strategy,
            default_thinking_config: thinking_config,
        }
    }

    /// Build the Anthropic Messages API request from an LlmRequest.
    fn build_request(&self, req: &LlmRequest) -> MessagesRequest {
        // Convert JSON messages to Anthropic format
        let (system_blocks, messages) = self.convert_messages(&req.messages);

        // Convert OpenAI-format tools to Anthropic format
        let tools = if req.tools.is_empty() {
            None
        } else {
            Some(self.convert_tools(&req.tools))
        };

        // Use system from anthropic_system field if present, otherwise from messages
        let system = if let Some(ref sys) = req.anthropic_system {
            Some(
                sys.iter()
                    .filter_map(|v| {
                        let text = v.get("text")?.as_str()?;
                        Some(SystemBlock {
                            block_type: "text".to_string(),
                            text: text.to_string(),
                            cache_control: v
                                .get("cache_control")
                                .map(|_| CacheControl::ephemeral()),
                        })
                    })
                    .collect(),
            )
        } else if system_blocks.is_empty() {
            None
        } else {
            Some(system_blocks)
        };

        // Resolve thinking config
        let thinking = req
            .thinking_config
            .clone()
            .or_else(|| self.default_thinking_config.clone());

        MessagesRequest {
            model: self.model.clone(),
            max_tokens: req.output_token_cap.map_or(self.max_tokens, |cap| self.max_tokens.min(cap)),
            system,
            messages,
            tools,
            tool_choice: match req.tool_choice.as_ref() {
                Some(super::ToolChoice::None) => Some(ToolChoice::None),
                Some(super::ToolChoice::Required) => Some(ToolChoice::Any {
                    disable_parallel_tool_use: req.parallel_tool_calls.map(|parallel| !parallel),
                }),
                Some(super::ToolChoice::Named(name)) => Some(ToolChoice::Tool {
                    name: name.clone(),
                    disable_parallel_tool_use: req.parallel_tool_calls.map(|parallel| !parallel),
                }),
                Some(super::ToolChoice::Auto) | None if !req.tools.is_empty() => Some(ToolChoice::Auto {
                    disable_parallel_tool_use: req.parallel_tool_calls.map(|parallel| !parallel),
                }),
                _ => None,
            },
            stream: true,
            thinking,
        }
    }

    fn build_request_with_strategy(&self, req: &LlmRequest) -> MessagesRequest {
        let mut api_request = self.build_request(req);
        if let Some(strategy) = req
            .cache_strategy
            .as_ref()
            .or(self.default_cache_strategy.as_ref())
        {
            strategy.apply(&mut api_request);
        }
        api_request
    }

    /// Convert JSON messages (OpenAI-ish format) to Anthropic Messages format.
    /// Returns (system_blocks, conversation_messages).
    fn convert_messages(
        &self,
        messages: &[serde_json::Value],
    ) -> (Vec<SystemBlock>, Vec<AnthropicMessage>) {
        let mut system_blocks = Vec::new();
        let mut anthropic_messages = Vec::new();

        for msg in messages {
            let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("user");

            match role {
                "system" => {
                    let text = msg
                        .get("content")
                        .and_then(|c| c.as_str())
                        .unwrap_or("")
                        .to_string();
                    system_blocks.push(SystemBlock {
                        block_type: "text".to_string(),
                        text,
                        cache_control: None,
                    });
                }
                "tool" => {
                    // Tool results → user message with tool_result content block
                    let tool_use_id = msg
                        .get("tool_call_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let content = msg
                        .get("content")
                        .and_then(|c| c.as_str())
                        .unwrap_or("")
                        .to_string();

                    anthropic_messages.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: serde_json::json!([{
                            "type": "tool_result",
                            "tool_use_id": tool_use_id,
                            "content": content
                        }]),
                    });
                }
                "assistant" => {
                    // Check for tool calls in assistant message
                    let mut content_blocks = Vec::new();

                    if let Some(text) = msg.get("content").and_then(|c| c.as_str()) {
                        if !text.is_empty() {
                            content_blocks.push(serde_json::json!({"type": "text", "text": text}));
                        }
                    }

                    if let Some(tool_calls) = msg.get("tool_calls").and_then(|tc| tc.as_array()) {
                        for tc in tool_calls {
                            let id = tc.get("id").and_then(|v| v.as_str()).unwrap_or("");
                            let name = tc
                                .get("function")
                                .and_then(|f| f.get("name"))
                                .and_then(|n| n.as_str())
                                .unwrap_or("");
                            let args_str = tc
                                .get("function")
                                .and_then(|f| f.get("arguments"))
                                .and_then(|a| a.as_str())
                                .unwrap_or("{}");
                            let input: serde_json::Value =
                                serde_json::from_str(args_str).unwrap_or_default();

                            content_blocks.push(serde_json::json!({
                                "type": "tool_use",
                                "id": id,
                                "name": name,
                                "input": input
                            }));
                        }
                    }

                    if content_blocks.is_empty() {
                        let text = msg.get("content").cloned().unwrap_or(serde_json::json!(""));
                        anthropic_messages.push(AnthropicMessage {
                            role: "assistant".to_string(),
                            content: text,
                        });
                    } else {
                        anthropic_messages.push(AnthropicMessage {
                            role: "assistant".to_string(),
                            content: serde_json::Value::Array(content_blocks),
                        });
                    }
                }
                _ => {
                    // user or any other role
                    let content = msg.get("content").cloned().unwrap_or(serde_json::json!(""));
                    anthropic_messages.push(AnthropicMessage {
                        role: role.to_string(),
                        content,
                    });
                }
            }
        }

        (system_blocks, anthropic_messages)
    }

    /// Convert OpenAI-format tool definitions to Anthropic ToolDef format.
    fn convert_tools(&self, tools: &[serde_json::Value]) -> Vec<ToolDef> {
        tools
            .iter()
            .filter_map(|tool| {
                let func = tool.get("function")?;
                Some(ToolDef {
                    name: func.get("name")?.as_str()?.to_string(),
                    description: func
                        .get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("")
                        .to_string(),
                    input_schema: func
                        .get("parameters")
                        .cloned()
                        .unwrap_or(serde_json::json!({"type": "object", "properties": {}})),
                    strict: func.get("strict").and_then(serde_json::Value::as_bool),
                    cache_control: None,
                })
            })
            .collect()
    }

    /// Build reqwest headers for Anthropic API.
    fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&self.api_key).unwrap_or_else(|_| HeaderValue::from_static("")),
        );
        headers.insert(
            "anthropic-version",
            HeaderValue::from_static(ANTHROPIC_VERSION),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers
    }
}

#[async_trait::async_trait]
impl LlmDriver for AnthropicDriver {
    fn with_bound_model(&self, model: &str) -> anyhow::Result<std::sync::Arc<dyn LlmDriver>> {
        let model = model
            .strip_prefix("anthropic/")
            .filter(|model| !model.trim().is_empty())
            .ok_or_else(|| {
                anyhow::anyhow!("Child model is outside the inherited Anthropic binding")
            })?;
        Ok(std::sync::Arc::new(Self {
            client: self.client.clone(),
            api_key: self.api_key.clone(),
            base_url: self.base_url.clone(),
            model: model.to_string(),
            max_tokens: self.max_tokens,
            default_cache_strategy: self.default_cache_strategy.clone(),
            default_thinking_config: self.default_thinking_config.clone(),
        }))
    }

    async fn stream(
        &self,
        req: LlmRequest,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>> {
        if req.output_token_cap == Some(0) {
            return Err(ProviderError::invalid_request("Output token cap must be positive").into());
        }
        req.validate_tool_controls()
            .map_err(|error| ProviderError::invalid_request(error.to_string()))?;
        // Reject malformed declarations before the converter can omit a tool.
        for tool in &req.tools {
            serde_json::from_value::<liter_llm::ChatCompletionTool>(tool.clone())
                .map_err(|error| ProviderError::invalid_request(format!("invalid tool declaration: {error}")))?;
        }
        if matches!(req.tool_choice.as_ref(), Some(super::ToolChoice::None))
            && req.parallel_tool_calls.is_some()
        {
            return Err(ProviderError::invalid_request(
                "provider capability error: parallel_tool_calls cannot accompany Anthropic tool_choice none",
            ).into());
        }
        if req.budget_contract.is_some() {
            return Err(ProviderError::invalid_request(
                "Anthropic driver has no supported final-wire budget contract",
            )
            .into());
        }
        if matches!(req.tool_choice.as_ref(), Some(super::ToolChoice::Required | super::ToolChoice::Named(_)))
            && req.thinking_config.as_ref().or(self.default_thinking_config.as_ref())
                .is_some_and(|thinking| thinking.thinking_type == "enabled")
        {
            return Err(ProviderError::invalid_request(
                "provider capability error: Anthropic extended thinking cannot force tool use",
            ).into());
        }
        let api_request = self.build_request_with_strategy(&req);

        let body = serde_json::to_string(&api_request)
            .map_err(|error| ProviderError::invalid_request(error.to_string()))?;
        debug!(
            model = %api_request.model,
            has_tools = api_request.tools.is_some(),
            has_thinking = api_request.thinking.is_some(),
            "Sending Anthropic Messages API request"
        );

        let response = self
            .client
            .post(&self.base_url)
            .headers(self.build_headers())
            .body(body)
            .send()
            .await
            .map_err(ProviderError::from_reqwest)?;

        let status = response.status();
        if !status.is_success() {
            let retry_after = response
                .headers()
                .get(RETRY_AFTER)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.trim().parse::<u64>().ok())
                .map(std::time::Duration::from_secs);
            let error_body = response.text().await.unwrap_or_default();
            let error_msg = serde_json::from_str::<serde_json::Value>(&error_body)
                .ok()
                .and_then(|v| {
                    v.get("error")
                        .and_then(|e| e.get("message"))
                        .and_then(|m| m.as_str())
                        .map(String::from)
                })
                .unwrap_or_else(|| format!("Anthropic API error: {status}"));

            return Err(ProviderError::from_http(
                status.as_u16(),
                retry_after,
                format!("Anthropic API returned HTTP {status}: {error_msg}"),
            )
            .into());
        }

        // Stream SSE response
        let request_id = uuid::Uuid::new_v4().to_string();
        let state = StreamState::new(request_id);
        let metrics_model = self.model.clone();

        Ok(decode_anthropic_sse(
            response.bytes_stream(),
            state,
            metrics_model,
        ))
    }
}

/// Decode a native Anthropic SSE byte stream into normalized events.
///
/// This is the production transport-framing, SSE-decoding, JSON-parsing and
/// normalization loop used by `AnthropicDriver::stream`. It is generic over
/// the chunk type so deterministic byte partitions can be fed to it without a
/// network; the driver passes `reqwest::Response::bytes_stream()`.
pub(super) fn decode_anthropic_sse<S, B>(
    byte_stream: S,
    mut state: StreamState,
    metrics_model: String,
) -> Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>
where
    S: Stream<Item = Result<B, reqwest::Error>> + Send + 'static,
    B: AsRef<[u8]> + Send + 'static,
{
    let event_stream = stream! {
            use futures::StreamExt;

            let mut decoder = liter_llm::SseDecoder::default();
            let mut framer = AnthropicSseFramer::default();
            let mut saw_terminal = false;

            // Pin the byte_stream for iteration
            let mut byte_stream = std::pin::pin!(byte_stream);

            while let Some(chunk_result) = byte_stream.next().await {
                let chunk = match chunk_result {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        warn!("SSE stream transport failed");
                        yield Err(ProviderError::from_reqwest(e).into());
                        return;
                    }
                };

                let canonical = framer.push(chunk.as_ref());
                if let Err(error) = decoder.push(&canonical) {
                    yield Err(ProviderError::from_liter(error).into());
                    return;
                }
                while let Some(frame) = decoder.next_event() {
                    if saw_terminal {
                        yield Err(ProviderError::new(None, super::ProviderErrorKind::Stream, None,
                            "Anthropic sent data after message_stop").into());
                        return;
                    }
                    let payload: serde_json::Value = match serde_json::from_str(&frame.data) {
                        Ok(value) => value,
                        Err(_) => {
                            yield Err(ProviderError::new(None, super::ProviderErrorKind::Stream, None,
                                "Malformed Anthropic SSE event").into());
                            return;
                        }
                    };
                    let Some(event_type) = payload.get("type").and_then(serde_json::Value::as_str) else {
                        yield Err(ProviderError::new(None, super::ProviderErrorKind::Stream, None,
                            "Malformed Anthropic SSE event").into());
                        return;
                    };
                    if !frame.event.is_empty() && event_type != frame.event {
                        yield Err(ProviderError::new(None, super::ProviderErrorKind::Stream, None,
                            "Anthropic SSE event name disagrees with payload type").into());
                        return;
                    }
                    if !is_known_stream_event_type(event_type) {
                        continue;
                    }
                    let event: StreamEvent = match serde_json::from_value(payload) {
                        Ok(event) => event,
                        Err(_) => {
                            yield Err(ProviderError::new(None, super::ProviderErrorKind::Stream, None,
                                "Malformed Anthropic SSE event").into());
                            return;
                        }
                    };
                    saw_terminal = matches!(&event, StreamEvent::MessageStop);
                    for normalized in state.process_event(event) {
                        if matches!(&normalized, NormalizedEvent::Done) {
                            // Validate trailing bytes and EOF before exposing success.
                            continue;
                        }
                        if let NormalizedEvent::Usage {
                            prompt_tokens, completion_tokens, cached_tokens,
                            cache_creation_tokens, ..
                        } = &normalized {
                            crate::uar::telemetry::metrics::record_llm_tokens(
                                "anthropic", &metrics_model,
                                u64::from(*prompt_tokens), u64::from(*completion_tokens),
                            );
                            if cached_tokens.is_some() || cache_creation_tokens.is_some() {
                                crate::uar::telemetry::metrics::record_cache_tokens(
                                    "anthropic", &metrics_model,
                                    cache_creation_tokens.unwrap_or(0), cached_tokens.unwrap_or(0),
                                );
                            }
                        }
                        let failed = matches!(&normalized, NormalizedEvent::Error { .. });
                        yield Ok(normalized);
                        if failed { return; }
                    }
                }
            }
            if let Err(error) = framer.finish() {
                yield Err(ProviderError::new(None, super::ProviderErrorKind::Stream, None,
                    error.to_string()).into());
                return;
            }
            if let Err(error) = decoder.finish() {
                yield Err(ProviderError::from_liter(error).into());
                return;
            }
            if !saw_terminal {
                yield Err(ProviderError::new(None, super::ProviderErrorKind::Stream, None,
                    "Anthropic stream ended without message_stop").into());
                return;
            }
            yield Ok(NormalizedEvent::Done);
    };

    Box::pin(event_stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(cache_strategy: Option<CacheStrategy>) -> LlmRequest {
        LlmRequest {
            messages: vec![
                serde_json::json!({
                    "role": "system",
                    "content": "stable ".repeat(700)
                }),
                serde_json::json!({
                    "role": "user",
                    "content": "hello"
                }),
            ],
            tools: Vec::new(),
            tool_choice: None,
            parallel_tool_calls: None,
            output_token_cap: None,
            cache_strategy,
            thinking_config: None,
            anthropic_system: None,
            extra_params: None,
            budget_contract: None,
        }
    }

    #[test]
    fn request_cache_strategy_controls_cache_control_serialization() {
        let driver =
            AnthropicDriver::new("test".into(), "claude-test".into(), None, None, None, None);
        let enabled = serde_json::to_value(
            driver.build_request_with_strategy(&request(Some(CacheStrategy::default()))),
        )
        .expect("serialize enabled request");
        let disabled = serde_json::to_value(driver.build_request_with_strategy(&request(None)))
            .expect("serialize disabled request");

        assert_eq!(enabled["system"][0]["cache_control"]["type"], "ephemeral");
        assert!(disabled["system"][0].get("cache_control").is_none());
    }

    #[test]
    fn provider_roots_are_normalized_to_the_messages_endpoint() {
        assert_eq!(
            messages_endpoint(Some("https://api.anthropic.com".to_string())),
            "https://api.anthropic.com/v1/messages"
        );
        assert_eq!(
            messages_endpoint(Some("https://proxy.example/v1".to_string())),
            "https://proxy.example/v1/messages"
        );
        assert_eq!(
            messages_endpoint(Some("https://proxy.example/v1/messages/".to_string())),
            "https://proxy.example/v1/messages"
        );
    }

    #[tokio::test]
    async fn non_success_response_is_a_driver_error_at_the_messages_endpoint() {
        use axum::{Json, Router, http::StatusCode, routing::post};

        let app = Router::new().route(
            "/v1/messages",
            post(|| async {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "error": {"message": "invalid test credential"}
                    })),
                )
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind stub");
        let address = listener.local_addr().expect("stub address");
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("serve stub");
        });

        let driver = AnthropicDriver::new(
            "invalid".into(),
            "claude-test".into(),
            Some(format!("http://{address}")),
            None,
            None,
            None,
        );
        let error = match driver.stream(request(None)).await {
            Ok(_) => panic!("non-success responses must activate orchestrator failover"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("HTTP 401"));
        assert!(error.to_string().contains("invalid test credential"));

        server.abort();
    }
}
