//! Tool normalization driver wrapper.
//!
//! Wraps any [`LlmDriver`] and normalizes its tool call output to match
//! Claude Sonnet/Opus 4.6 behavior, regardless of the backend model's native
//! tool-call capability.

use std::pin::Pin;
use std::sync::Arc;

use async_stream::stream;
use futures::{Stream, StreamExt};

use super::capability_registry::{ModelCapabilityProfile, ToolCallCapability};
use super::tool_extractor::ToolCallExtractor;
use super::xml_tool_injector;
use super::{LlmDriver, LlmRequest};
use crate::normalized::NormalizedEvent;

/// Driver wrapper that normalizes tool calls from any backend model
/// to match Claude Sonnet/Opus 4.6 format.
pub struct ToolNormalizerDriver {
    inner: Arc<dyn LlmDriver>,
    profile: ModelCapabilityProfile,
}

impl std::fmt::Debug for ToolNormalizerDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolNormalizerDriver")
            .field("capability", &self.profile.tool_call_capability)
            .finish()
    }
}

impl ToolNormalizerDriver {
    /// Create a new normalizer wrapping the given driver with the specified
    /// capability profile for the target model.
    pub fn new(inner: Arc<dyn LlmDriver>, profile: ModelCapabilityProfile) -> Self {
        Self { inner, profile }
    }
}

#[async_trait::async_trait]
impl LlmDriver for ToolNormalizerDriver {
    async fn stream(
        &self,
        mut req: LlmRequest,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>> {
        crate::uar::runtime::context::normalize::normalize_provider_messages(&mut req.messages)?;
        match self.profile.tool_call_capability {
            ToolCallCapability::Native => {
                // Pass through — model speaks tool calls natively
                self.inner.stream(req).await
            }
            ToolCallCapability::InstructionTuned => {
                // Inject XML tool schema into system prompt
                xml_tool_injector::inject_tools_into_system_prompt(&mut req);

                let inner_stream = self.inner.stream(req).await?;
                let mut extractor = ToolCallExtractor::new();

                let normalized_stream = stream! {
                    let mut inner_stream = std::pin::pin!(inner_stream);
                    while let Some(event_result) = inner_stream.next().await {
                        match event_result {
                            Ok(NormalizedEvent::MessageDelta { text }) => {
                                // Run text through the extractor to detect tool calls
                                for extracted in extractor.process_delta(&text) {
                                    yield Ok(extracted);
                                }
                            }
                            Ok(NormalizedEvent::Done) => {
                                // Flush any remaining extractor state
                                for flushed in extractor.flush() {
                                    yield Ok(flushed);
                                }
                                yield Ok(NormalizedEvent::Done);
                            }
                            other => yield other,
                        }
                    }
                };

                Ok(Box::pin(normalized_stream))
            }
            ToolCallCapability::GrammarConstrained => {
                // For grammar-constrained models, we could inject JSON schema
                // as a structured output constraint. For now, treat like InstructionTuned.
                xml_tool_injector::inject_tools_into_system_prompt(&mut req);

                let inner_stream = self.inner.stream(req).await?;
                let mut extractor = ToolCallExtractor::new();

                let normalized_stream = stream! {
                    let mut inner_stream = std::pin::pin!(inner_stream);
                    while let Some(event_result) = inner_stream.next().await {
                        match event_result {
                            Ok(NormalizedEvent::MessageDelta { text }) => {
                                for extracted in extractor.process_delta(&text) {
                                    yield Ok(extracted);
                                }
                            }
                            Ok(NormalizedEvent::Done) => {
                                for flushed in extractor.flush() {
                                    yield Ok(flushed);
                                }
                                yield Ok(NormalizedEvent::Done);
                            }
                            other => yield other,
                        }
                    }
                };

                Ok(Box::pin(normalized_stream))
            }
            ToolCallCapability::TextOnly => {
                // Full prompt engineering — include XML schema + few-shot examples
                xml_tool_injector::inject_tools_into_system_prompt(&mut req);
                // Could add few-shot examples here for TextOnly models

                let inner_stream = self.inner.stream(req).await?;
                let mut extractor = ToolCallExtractor::new();

                let normalized_stream = stream! {
                    let mut inner_stream = std::pin::pin!(inner_stream);
                    while let Some(event_result) = inner_stream.next().await {
                        match event_result {
                            Ok(NormalizedEvent::MessageDelta { text }) => {
                                for extracted in extractor.process_delta(&text) {
                                    yield Ok(extracted);
                                }
                            }
                            Ok(NormalizedEvent::Done) => {
                                for flushed in extractor.flush() {
                                    yield Ok(flushed);
                                }
                                yield Ok(NormalizedEvent::Done);
                            }
                            other => yield other,
                        }
                    }
                };

                Ok(Box::pin(normalized_stream))
            }
        }
    }
}
