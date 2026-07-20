//! Mock LLM driver for deterministic integration tests.

use crate::llm::{LlmDriver, LlmRequest};
use crate::normalized::NormalizedEvent;
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

/// Canned responses for deterministic testing.
#[derive(Clone)]
pub struct MockLlmDriver {
    responses: Arc<Mutex<Vec<Vec<NormalizedEvent>>>>,
    call_count: Arc<Mutex<usize>>,
    requests: Arc<Mutex<Vec<LlmRequest>>>,
}

impl MockLlmDriver {
    /// Create a mock with a fixed sequence of response event lists.
    ///
    /// Each call to `stream()` consumes the next response list (cycling if exhausted).
    pub fn new(responses: Vec<Vec<NormalizedEvent>>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses)),
            call_count: Arc::new(Mutex::new(0)),
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a mock that always returns a simple text response.
    pub fn echo() -> Self {
        Self::new(vec![vec![
            NormalizedEvent::MessageDelta {
                text: "Hello from mock!".to_string(),
            },
            NormalizedEvent::Done,
        ]])
    }

    /// How many times `stream()` has been called.
    pub fn call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }

    /// Requests captured in call order for deterministic adapter assertions.
    pub fn requests(&self) -> Vec<LlmRequest> {
        self.requests.lock().unwrap().clone()
    }
}

impl std::fmt::Debug for MockLlmDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockLlmDriver")
            .field("call_count", &self.call_count())
            .finish()
    }
}

#[async_trait]
impl LlmDriver for MockLlmDriver {
    async fn stream(
        &self,
        req: LlmRequest,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>> {
        let mut count = self.call_count.lock().unwrap();
        let idx = *count;
        *count += 1;
        drop(count);
        self.requests.lock().unwrap().push(req);

        let responses = self.responses.lock().unwrap();
        let events = responses
            .get(idx % responses.len())
            .cloned()
            .unwrap_or_default();
        drop(responses);

        let stream = async_stream::stream! {
            for event in events {
                yield Ok(event);
            }
        };

        Ok(Box::pin(stream))
    }
}
