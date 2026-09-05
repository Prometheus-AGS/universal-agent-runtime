//! External LLM driver adapter.
//!
//! This driver is the narrow in-process seam for hosts that own a local model
//! runtime outside UAR but still want UAR to own agent execution, tool loops,
//! skills, and event semantics. The host supplies an async handler that accepts
//! UAR's canonical [`LlmRequest`] and returns normalized stream events.

use crate::llm::{LlmDriver, LlmRequest, ProviderError};
use crate::normalized::NormalizedEvent;
use async_trait::async_trait;
use futures::{Stream, StreamExt, future::BoxFuture};
use std::pin::Pin;
use std::sync::Arc;

/// Normalized stream returned by an external driver handler.
pub type ExternalDriverStream = Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>;

/// Async host callback used by [`ExternalLlmDriver`].
pub type ExternalDriverHandler = Arc<
    dyn Fn(LlmRequest) -> BoxFuture<'static, anyhow::Result<ExternalDriverStream>> + Send + Sync,
>;

/// UAR `LlmDriver` implementation backed by a host-provided async handler.
///
/// The adapter deliberately owns no provider credentials, model catalog, or
/// process supervision. Those remain host/runtime concerns. UAR receives only
/// the canonical request and normalized event stream it needs to run agents.
#[derive(Clone)]
pub struct ExternalLlmDriver {
    name: String,
    handler: ExternalDriverHandler,
}

impl ExternalLlmDriver {
    /// Create a new external driver.
    pub fn new(name: impl Into<String>, handler: ExternalDriverHandler) -> Self {
        Self {
            name: name.into(),
            handler,
        }
    }

    /// Human-readable host/provider name for diagnostics.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl std::fmt::Debug for ExternalLlmDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalLlmDriver")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl LlmDriver for ExternalLlmDriver {
    async fn stream(&self, req: LlmRequest) -> anyhow::Result<ExternalDriverStream> {
        match (self.handler)(req).await {
            Ok(stream) => Ok(Box::pin(stream.map(|event| match event {
                Ok(event) => Ok(event),
                Err(error) if ProviderError::from_anyhow(&error).is_some() => Err(error),
                Err(error) => Err(ProviderError::external(error.to_string()).into()),
            }))),
            Err(error) if ProviderError::from_anyhow(&error).is_some() => Err(error),
            Err(error) => Err(ProviderError::external(error.to_string()).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn external_driver_delegates_request_and_streams_events() {
        let handler: ExternalDriverHandler = Arc::new(|request| {
            Box::pin(async move {
                let text = request
                    .messages
                    .first()
                    .and_then(|message| message.get("content"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("<missing>")
                    .to_owned();
                Ok(Box::pin(futures::stream::iter([
                    Ok(NormalizedEvent::MessageDelta { text }),
                    Ok(NormalizedEvent::Done),
                ])) as ExternalDriverStream)
            })
        });

        let driver = ExternalLlmDriver::new("knowme-local", handler);
        assert_eq!(driver.name(), "knowme-local");

        let mut stream = driver
            .stream(LlmRequest {
                messages: vec![serde_json::json!({
                    "role": "user",
                    "content": "hello"
                })],
                tools: vec![serde_json::json!({
                    "type": "function",
                    "function": {"name": "noop"}
                })],
                cache_strategy: None,
                thinking_config: None,
                anthropic_system: None,
                extra_params: Some(serde_json::json!({"temperature": 0.2})),
            })
            .await
            .expect("external driver stream starts");

        assert_eq!(
            stream
                .next()
                .await
                .expect("first event exists")
                .expect("first event is ok"),
            NormalizedEvent::MessageDelta {
                text: "hello".into()
            }
        );
        assert_eq!(
            stream
                .next()
                .await
                .expect("second event exists")
                .expect("second event is ok"),
            NormalizedEvent::Done
        );
    }
}
