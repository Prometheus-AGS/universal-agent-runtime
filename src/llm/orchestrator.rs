//! LLM orchestrator with tool loop execution.
//!
//! The orchestrator manages the complete lifecycle of an LLM interaction:
//! 1. Send user message to the LLM
//! 2. Stream the response, detecting tool calls
//! 3. Execute tool calls via MCP
//! 4. Feed tool results back to the LLM
//! 5. Repeat until the model produces a final response
//!
//! # Example
//!
//! ```rust,ignore
//! use universal_agent_runtime::config::LlmConfig;
//! use universal_agent_runtime::llm::Orchestrator;
//! use universal_agent_runtime::mcp::registry::McpRegistry;
//! use universal_agent_runtime::uar::runtime::native_skill::NativeSkillRegistry;
//!
//! let llm_config = LlmConfig::default();
//! let mcp = McpRegistry::load_from_file("mcp.json").await?;
//! let native_skills = Arc::new(NativeSkillRegistry::new());
//! let orchestrator = Orchestrator::new(llm_config, mcp, native_skills)?;
//!
//! let stream = orchestrator.chat("Hello, what time is it?").await?;
//! ```

use std::collections::BTreeMap;
use std::pin::Pin;
use std::sync::Arc;

use futures::{Stream, StreamExt};
use uuid::Uuid;

use crate::config::{FailoverConfig, LlmConfig};
use crate::mcp::registry::McpRegistry;
use crate::normalized::NormalizedEvent;
use crate::uar::runtime::native_skill::NativeSkillRegistry;

use super::{
    LiterLlmDriver, LlmDriver, LlmRequest, Message,
    MessageContent, MessageRole, ToolCall, ToolCallFunction,
};

/// Result of a tool approval gate check.
#[derive(Debug, Clone)]
pub enum ToolApprovalResult {
    /// Tool is approved for execution.
    Approved,
    /// Tool execution was rejected by the user or timed out.
    Rejected { reason: String },
}

/// A callback invoked before each tool call execution to allow approval/rejection.
/// Returns `Approved` to proceed or `Rejected` to skip the tool call.
pub type ToolApprovalGate = Arc<
    dyn Fn(
            String,              // tool_call_id
            String,              // tool_name
            String,              // arguments_json
            usize,               // call_index
        ) -> Pin<Box<dyn Future<Output = ToolApprovalResult> + Send>>
        + Send
        + Sync,
>;

/// Maximum number of tool loop iterations to prevent infinite loops.
const MAX_TOOL_ITERATIONS: usize = 10;

/// Accumulated state for a streaming tool call.
#[derive(Debug, Default, Clone)]
struct ToolCallAccumulator {
    id: Option<String>,
    name: Option<String>,
    arguments: String,
}

/// LLM orchestrator with tool loop execution.
///
/// The orchestrator wraps an [`LlmDriver`] and adds:
/// - Tool call detection and accumulation
/// - Tool execution via MCP
/// - Automatic tool result feeding
/// - Request ID tracking
#[derive(Clone)]
pub struct Orchestrator {
    llm_config: LlmConfig,
    mcp: Arc<McpRegistry>,
    driver: Arc<dyn LlmDriver>,
    /// Optional fallback driver activated when the primary driver errors.
    fallback_driver: Option<Arc<dyn LlmDriver>>,
    /// Controls when/how to switch to the fallback driver.
    failover_config: FailoverConfig,
    native_skills: Arc<NativeSkillRegistry>,
    /// Optional gate that is consulted before each tool call execution.
    /// If `None`, all tool calls are approved automatically.
    tool_approval_gate: Option<ToolApprovalGate>,
}

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for Orchestrator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Orchestrator")
            .field("model", &self.llm_config.model)
            .field("mcp", &"McpRegistry")
            .finish()
    }
}

impl Orchestrator {
    /// Create a new orchestrator with the given LLM config, MCP registry, and native skill registry.
    ///
    /// Uses `LiterLlmDriver` backed by liter-llm's `DefaultClient` for all
    /// 142+ providers with unified tool-call normalization.
    /// # Errors
    ///
    /// Returns an error if the underlying LLM client cannot be constructed.
    #[allow(dead_code)]
    pub fn new(
        llm_config: LlmConfig,
        mcp: Arc<McpRegistry>,
        native_skills: Arc<NativeSkillRegistry>,
    ) -> anyhow::Result<Self> {
        let client_config = crate::config::build_client_config(&llm_config);
        let driver: Arc<dyn LlmDriver> = Arc::new(LiterLlmDriver::new(
            client_config,
            llm_config.model.clone(),
            llm_config.parallel_tool_calls,
        )?);

        Ok(Self {
            llm_config,
            mcp,
            driver,
            fallback_driver: None,
            failover_config: FailoverConfig::default(),
            native_skills,
            tool_approval_gate: None,
        })
    }

    /// Attach a fallback driver and failover configuration.
    ///
    /// When `failover_config.enabled` is `true` and the primary driver fails,
    /// the orchestrator will re-try the same request against `fallback_driver`.
    #[must_use]
    pub fn with_failover(
        mut self,
        fallback_driver: Arc<dyn LlmDriver>,
        failover_config: FailoverConfig,
    ) -> Self {
        self.fallback_driver = Some(fallback_driver);
        self.failover_config = failover_config;
        self
    }

    /// Get the LLM configuration.
    #[must_use]
    #[allow(dead_code)]
    pub fn llm_config(&self) -> &LlmConfig {
        &self.llm_config
    }

    /// Get the MCP registry.
    #[must_use]
    #[allow(dead_code)]
    pub fn mcp(&self) -> &McpRegistry {
        &self.mcp
    }

    /// Set a tool approval gate that will be consulted before each tool call.
    #[must_use]
    pub fn with_tool_approval_gate(mut self, gate: ToolApprovalGate) -> Self {
        self.tool_approval_gate = Some(gate);
        self
    }

    /// Start a chat interaction with the given user message.
    ///
    /// Returns a stream of [`NormalizedEvent`]s that includes:
    /// - `StreamStart` with a unique request ID
    /// - `MessageDelta` for assistant text
    /// - `ToolCallDelta` and `ToolCallComplete` for tool calls
    /// - `ToolResult` after tool execution
    /// - `Done` when complete
    ///
    /// The orchestrator will automatically execute tool calls and feed
    /// results back to the LLM until a final response is produced.
    #[allow(dead_code)]
    pub async fn chat(
        &self,
        user_message: &str,
    ) -> anyhow::Result<impl Stream<Item = NormalizedEvent> + Send> {
        self.chat_with_history(vec![Message {
            role: MessageRole::User,
            content: MessageContent::text(user_message),
            tool_call_id: None,
            tool_calls: None,
        }])
        .await
    }

    /// Start a chat interaction with existing message history.
    #[allow(clippy::too_many_lines)]
    pub async fn chat_with_history(
        &self,
        messages: Vec<Message>,
    ) -> anyhow::Result<impl Stream<Item = NormalizedEvent> + Send + 'static> {
        let request_id = Uuid::new_v4().to_string();
        let tools = self.mcp.openai_tools_json();

        tracing::info!(
            request_id = %request_id,
            message_count = messages.len(),
            tool_count = tools.len(),
            "Starting orchestrator chat"
        );

        // Log initial message history
        for (idx, msg) in messages.iter().enumerate() {
            let content_preview = msg.content.as_text().map_or(0, str::len);
            tracing::debug!(
                request_id = %request_id,
                message_index = idx,
                role = ?msg.role,
                content_length = content_preview,
                has_tool_calls = msg.tool_calls.is_some(),
                "Initial message"
            );
        }

        let orchestrator = self.clone();
        let messages = messages.clone();

        let stream = async_stream::stream! {
            // Emit stream start
            yield NormalizedEvent::StreamStart {
                request_id: request_id.clone(),
            };

            // Convert messages to JSON for the driver
            let mut message_json: Vec<serde_json::Value> = messages
                .iter()
                .map(|m| serde_json::to_value(m).unwrap_or_default())
                .collect();

            tracing::debug!(
                request_id = %request_id,
                "Converted messages to JSON for driver"
            );

            let mut iteration = 0;

            loop {
                if iteration >= MAX_TOOL_ITERATIONS {
                    tracing::error!(
                        request_id = %request_id,
                        iteration = iteration,
                        max_iterations = MAX_TOOL_ITERATIONS,
                        "Maximum tool loop iterations exceeded"
                    );
                    yield NormalizedEvent::Error {
                        message: "Maximum tool loop iterations exceeded".to_string(),
                        code: Some("MAX_ITERATIONS".to_string()),
                    };
                    break;
                }
                iteration += 1;

                tracing::info!(
                    request_id = %request_id,
                    iteration = iteration,
                    message_count = message_json.len(),
                    "Starting tool loop iteration"
                );

                let req = LlmRequest {
                    messages: message_json.clone(),
                    tools: tools.clone(),
                    cache_strategy: None,
                    thinking_config: None,
                    anthropic_system: None,
                };

                // Log the full request being sent to the LLM
                tracing::debug!(
                    request_id = %request_id,
                    iteration = iteration,
                    messages = ?req.messages,
                    tool_count = req.tools.len(),
                    "Sending request to LLM driver"
                );

                // Stream from the driver (with automatic failover if configured)
                let driver_stream = {
                    let primary = orchestrator.driver.stream(req.clone()).await;
                    match primary {
                        Ok(s) => {
                            tracing::debug!(
                                request_id = %request_id,
                                iteration = iteration,
                                "Driver stream created successfully"
                            );
                            s
                        }
                        Err(e) if orchestrator.failover_config.enabled => {
                            if let Some(ref fallback) = orchestrator.fallback_driver {
                                tracing::warn!(
                                    request_id = %request_id,
                                    iteration = iteration,
                                    primary_error = %e,
                                    "Primary LLM driver failed; attempting failover",
                                );
                                match fallback.stream(req).await {
                                    Ok(s) => s,
                                    Err(fe) => {
                                        tracing::error!(
                                            request_id = %request_id,
                                            primary_error = %e,
                                            fallback_error = %fe,
                                            "Fallback driver also failed",
                                        );
                                        yield NormalizedEvent::Error {
                                            message: format!(
                                                "primary: {e}; fallback: {fe}"
                                            ),
                                            code: None,
                                        };
                                        break;
                                    }
                                }
                            } else {
                                tracing::error!(
                                    request_id = %request_id,
                                    iteration = iteration,
                                    error = %e,
                                    "Primary driver failed; no fallback configured",
                                );
                                yield NormalizedEvent::Error {
                                    message: e.to_string(),
                                    code: None,
                                };
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::error!(
                                request_id = %request_id,
                                iteration = iteration,
                                error = %e,
                                "Failed to create driver stream"
                            );
                            yield NormalizedEvent::Error {
                                message: e.to_string(),
                                code: None,
                            };
                            break;
                        }
                    }
                };

                let mut tool_accumulators: BTreeMap<usize, ToolCallAccumulator> = BTreeMap::new();
                let mut assistant_text = String::new();
                let mut has_tool_calls = false;
                let mut finish_reason: Option<String> = None;

                futures::pin_mut!(driver_stream);

                while let Some(result) = driver_stream.next().await {
                    match result {
                        Ok(event) => {
                            match &event {
                                NormalizedEvent::MessageDelta { text } => {
                                    assistant_text.push_str(text);
                                }
                                NormalizedEvent::ToolCallDelta {
                                    call_index,
                                    id,
                                    name,
                                    arguments_delta,
                                } => {
                                    has_tool_calls = true;
                                    let acc = tool_accumulators.entry(*call_index).or_default();
                                    if acc.id.is_none() {
                                        acc.id = id.clone();
                                    }
                                    if acc.name.is_none() {
                                        acc.name = name.clone();
                                    }
                                    if let Some(delta) = arguments_delta {
                                        acc.arguments.push_str(delta);
                                    }
                                }
                                NormalizedEvent::ToolCallComplete { .. } => {
                                    has_tool_calls = true;
                                    finish_reason = Some("tool_calls".to_string());
                                }
                                NormalizedEvent::Done => {
                                    // Don't yield Done yet if we have tool calls to process
                                    if !has_tool_calls {
                                        yield event;
                                        return;
                                    }
                                    continue;
                                }
                                NormalizedEvent::Error { .. } => {
                                    yield event;
                                    return;
                                }
                                _ => {}
                            }
                            yield event;
                        }
                        Err(e) => {
                            yield NormalizedEvent::Error {
                                message: e.to_string(),
                                code: None,
                            };
                            return;
                        }
                    }
                }

                // If no tool calls, we're done
                if !has_tool_calls || finish_reason.as_deref() != Some("tool_calls") {
                    tracing::info!(
                        request_id = %request_id,
                        iteration = iteration,
                        has_tool_calls = has_tool_calls,
                        finish_reason = ?finish_reason,
                        "No tool calls to process, completing stream"
                    );
                    yield NormalizedEvent::Done;
                    break;
                }

                tracing::info!(
                    request_id = %request_id,
                    iteration = iteration,
                    accumulator_count = tool_accumulators.len(),
                    "Building tool calls from accumulators"
                );

                // Build tool calls from accumulators
                let tool_calls: Vec<ToolCall> = tool_accumulators
                    .values()
                    .filter_map(|acc| {
                        let id = acc.id.clone()?;
                        let name = acc.name.clone()?;
                        Some(ToolCall {
                            id,
                            call_type: "function".to_string(),
                            function: ToolCallFunction {
                                name,
                                arguments: acc.arguments.clone(),
                            },
                        })
                    })
                    .collect();

                if tool_calls.is_empty() {
                    tracing::warn!(
                        request_id = %request_id,
                        iteration = iteration,
                        "No valid tool calls built from accumulators"
                    );
                    yield NormalizedEvent::Done;
                    break;
                }

                tracing::info!(
                    request_id = %request_id,
                    iteration = iteration,
                    tool_call_count = tool_calls.len(),
                    "Built tool calls, adding to message history"
                );

                // Log each tool call
                for (idx, tc) in tool_calls.iter().enumerate() {
                    tracing::info!(
                        request_id = %request_id,
                        iteration = iteration,
                        tool_index = idx,
                        tool_id = %tc.id,
                        tool_name = %tc.function.name,
                        args_length = tc.function.arguments.len(),
                        "Tool call to execute"
                    );
                    tracing::debug!(
                        request_id = %request_id,
                        tool_id = %tc.id,
                        arguments = %tc.function.arguments,
                        "Tool call arguments"
                    );
                }

                // Add assistant message with tool calls to history
                message_json.push(serde_json::json!({
                    "role": "assistant",
                    "content": if assistant_text.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(assistant_text.clone()) },
                    "tool_calls": tool_calls.iter().map(|tc| {
                        serde_json::json!({
                            "id": tc.id,
                            "type": tc.call_type,
                            "function": {
                                "name": tc.function.name,
                                "arguments": tc.function.arguments
                            }
                        })
                    }).collect::<Vec<_>>()
                }));

                tracing::debug!(
                    request_id = %request_id,
                    iteration = iteration,
                    "Added assistant message with tool calls to history"
                );

                // Execute each tool call and emit results
                for (idx, tool_call) in tool_calls.iter().enumerate() {
                    let tool_name = &tool_call.function.name;
                    let arguments: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)
                        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

                    tracing::info!(
                        request_id = %request_id,
                        iteration = iteration,
                        tool_index = idx,
                        tool_id = %tool_call.id,
                        tool_name = %tool_name,
                        "Executing tool call"
                    );

                    // Check tool approval gate if configured
                    if let Some(ref gate) = orchestrator.tool_approval_gate {
                        let result = gate(
                            tool_call.id.clone(),
                            tool_name.clone(),
                            tool_call.function.arguments.clone(),
                            idx,
                        ).await;
                        if let ToolApprovalResult::Rejected { reason } = result {
                            tracing::warn!(
                                request_id = %request_id,
                                tool_id = %tool_call.id,
                                tool_name = %tool_name,
                                reason = %reason,
                                "Tool call rejected by approval gate"
                            );
                            let rejection_content = format!("Tool call rejected: {reason}");
                            yield NormalizedEvent::ToolResult {
                                id: tool_call.id.clone(),
                                name: tool_name.clone(),
                                content: rejection_content.clone(),
                                success: false,
                            };
                            message_json.push(serde_json::json!({
                                "role": "tool",
                                "tool_call_id": tool_call.id,
                                "content": rejection_content
                            }));
                            continue;
                        }
                    }

                    let (content, success) = {
                        // Priority: check native skills first, then fall back to MCP
                        if let Some(native_skill) = orchestrator.native_skills.get(tool_name).await {
                            tracing::info!(
                                request_id = %request_id,
                                iteration = iteration,
                                tool_id = %tool_call.id,
                                tool_name = %tool_name,
                                "Executing via native skill (bypassing MCP)"
                            );
                            match native_skill.execute(arguments.clone()).await {
                                Ok(result) => {
                                    let content = serde_json::to_string(&result).unwrap_or_default();
                                    tracing::info!(
                                        request_id = %request_id,
                                        tool_id = %tool_call.id,
                                        tool_name = %tool_name,
                                        result_length = content.len(),
                                        "Native skill execution succeeded"
                                    );
                                    (content, true)
                                }
                                Err(e) => {
                                    let error_msg = format!("Native skill error: {e}");
                                    tracing::error!(
                                        request_id = %request_id,
                                        tool_id = %tool_call.id,
                                        tool_name = %tool_name,
                                        error = %e,
                                        "Native skill execution failed"
                                    );
                                    (error_msg, false)
                                }
                            }
                        } else {
                            match orchestrator.mcp.call_namespaced_tool(tool_name, arguments.clone()).await {
                                Ok(result) => {
                                    let content = serde_json::to_string(&result).unwrap_or_default();
                                    tracing::info!(
                                        request_id = %request_id,
                                        iteration = iteration,
                                        tool_id = %tool_call.id,
                                        tool_name = %tool_name,
                                        result_length = content.len(),
                                        "Tool call succeeded"
                                    );
                                    tracing::debug!(
                                        request_id = %request_id,
                                        tool_id = %tool_call.id,
                                        result = %content,
                                        "Tool call result"
                                    );
                                    (content, true)
                                }
                                Err(e) => {
                                    let error_msg = format!("Error: {e}");
                                    tracing::error!(
                                        request_id = %request_id,
                                        iteration = iteration,
                                        tool_id = %tool_call.id,
                                        tool_name = %tool_name,
                                        error = %e,
                                        "Tool call failed"
                                    );
                                    (error_msg, false)
                                }
                            }
                        }
                    };

                    // Emit tool result event
                    yield NormalizedEvent::ToolResult {
                        id: tool_call.id.clone(),
                        name: tool_name.clone(),
                        content: content.clone(),
                        success,
                    };

                    // Add tool result to message history
                    message_json.push(serde_json::json!({
                        "role": "tool",
                        "tool_call_id": tool_call.id,
                        "content": content
                    }));

                    tracing::debug!(
                        request_id = %request_id,
                        iteration = iteration,
                        tool_id = %tool_call.id,
                        "Added tool result to message history"
                    );
                }

                tracing::info!(
                    request_id = %request_id,
                    iteration = iteration,
                    "All tool calls executed, continuing to next iteration"
                );

                // Continue the loop to get the next response
            }
        };

        Ok(stream)
    }

    /// Non-streaming chat for simple requests (e.g., title generation).
    ///
    /// This collects all message deltas into a single string response.
    pub async fn chat_non_streaming(&self, messages: Vec<Message>) -> anyhow::Result<String> {
        let request_id = Uuid::new_v4().to_string();
        let tools = Vec::new(); // No tools for simple requests

        tracing::debug!(
            request_id = %request_id,
            message_count = messages.len(),
            "Starting non-streaming chat"
        );

        let message_json: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| serde_json::to_value(m).unwrap_or_default())
            .collect();

        let req = LlmRequest {
            messages: message_json,
            tools,
            cache_strategy: None,
            thinking_config: None,
            anthropic_system: None,
        };

        // Stream from the driver and collect message deltas
        let mut stream = self.driver.stream(req).await?;
        let mut content = String::new();

        while let Some(event_result) = stream.next().await {
            match event_result {
                Ok(NormalizedEvent::MessageDelta { text }) => {
                    content.push_str(&text);
                }
                Err(e) => {
                    tracing::error!(request_id = %request_id, error = %e, "Error in stream");
                    return Err(e);
                }
                _ => {} // Ignore other events
            }
        }

        tracing::debug!(
            request_id = %request_id,
            content_length = content.len(),
            "Non-streaming chat completed"
        );

        Ok(content)
    }
}
