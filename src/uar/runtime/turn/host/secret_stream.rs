//! One copied content stream's bounded undecided suffix, never execution input.
use super::RunSecretScrubber;
use super::secret_projection::SecretProjection;
use crate::normalized::NormalizedEvent;
use std::collections::BTreeMap;

pub struct SecretStream {
    projection: SecretProjection,
    pending: String,
}

/// Projection of a copied orchestrator stream. Raw producer accumulators stay
/// private for execution; each display/model-content channel has separate carry.
pub struct ProjectedEventStream {
    scrubber: RunSecretScrubber,
    text: SecretStream,
    thinking: SecretStream,
    reasoning: SecretStream,
    arguments: BTreeMap<usize, (SecretStream, Option<String>, Option<String>)>,
    sandbox: BTreeMap<String, SecretStream>,
}

impl ProjectedEventStream {
    pub fn new(scrubber: RunSecretScrubber) -> Self {
        Self {
            text: scrubber.stream(),
            thinking: scrubber.stream(),
            reasoning: scrubber.stream(),
            arguments: BTreeMap::new(),
            sandbox: BTreeMap::new(),
            scrubber,
        }
    }

    pub fn push(&mut self, mut event: NormalizedEvent) -> Vec<NormalizedEvent> {
        use NormalizedEvent::*;
        let mut prefix = Vec::new();
        match &mut event {
            MessageDelta { text } => *text = self.text.push(text),
            ThinkingDelta { text } => *text = self.thinking.push(text),
            ReasoningDelta { text } => *text = self.reasoning.push(text),
            ToolCallDelta {
                call_index,
                id,
                name,
                arguments_delta,
            } => {
                let entry = self
                    .arguments
                    .entry(*call_index)
                    .or_insert_with(|| (self.scrubber.stream(), None, None));
                if id.is_some() {
                    entry.1.clone_from(id);
                }
                if name.is_some() {
                    entry.2.clone_from(name);
                }
                if let Some(fragment) = arguments_delta {
                    *fragment = entry.0.push(fragment);
                }
            }
            ToolCallComplete {
                call_index,
                arguments_json,
                ..
            } => {
                if let Some((mut stream, id, name)) = self.arguments.remove(call_index) {
                    let tail = stream.finish();
                    if !tail.is_empty() {
                        prefix.push(ToolCallDelta {
                            call_index: *call_index,
                            id,
                            name,
                            arguments_delta: Some(tail),
                        });
                    }
                }
                *arguments_json = self.scrubber.project_json_text(arguments_json);
            }
            SandboxOutput { stream, data } => {
                *data = self
                    .sandbox
                    .entry(stream.clone())
                    .or_insert_with(|| self.scrubber.stream())
                    .push(data);
            }
            ToolResult { content, .. } => *content = self.scrubber.project_json_text(content),
            Error { message, .. } => {
                *message = self.scrubber.scrub(message);
                prefix = self.finish();
            }
            Done => prefix = self.finish(),
            CitationAdded(citation) => {
                citation.url = self.scrubber.scrub(&citation.url);
                citation.title = citation.title.take().map(|text| self.scrubber.scrub(&text));
                citation.snippet = citation
                    .snippet
                    .take()
                    .map(|text| self.scrubber.scrub(&text));
            }
            MemoryUpdate { value, .. } => *value = self.scrubber.scrub(value),
            Custom {
                source,
                event_name,
                payload,
            } => {
                // UAR request manifests are typed identities/hashes. Other
                // provider custom payloads are copied untrusted content.
                let host_manifest = (source == "uar.request" && event_name == "attempt_manifest")
                    || (source == "uar.turn" && event_name == "resolved_step");
                if !host_manifest {
                    *payload = self.scrubber.project_value(std::mem::take(payload));
                }
            }
            _ => {}
        }
        prefix.push(event);
        prefix
    }

    /// Also called when the upstream ends without emitting a terminal event.
    pub fn finish(&mut self) -> Vec<NormalizedEvent> {
        use NormalizedEvent::*;
        let mut events = Vec::new();
        let text = self.text.finish();
        if !text.is_empty() {
            events.push(MessageDelta { text });
        }
        let text = self.thinking.finish();
        if !text.is_empty() {
            events.push(ThinkingDelta { text });
        }
        let text = self.reasoning.finish();
        if !text.is_empty() {
            events.push(ReasoningDelta { text });
        }
        for (call_index, (mut stream, id, name)) in std::mem::take(&mut self.arguments) {
            let tail = stream.finish();
            if !tail.is_empty() {
                events.push(ToolCallDelta {
                    call_index,
                    id,
                    name,
                    arguments_delta: Some(tail),
                });
            }
        }
        for (stream, mut projection) in std::mem::take(&mut self.sandbox) {
            let data = projection.finish();
            if !data.is_empty() {
                events.push(SandboxOutput { stream, data });
            }
        }
        events
    }
}

impl std::fmt::Debug for SecretStream {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SecretStream([redacted])")
    }
}

impl SecretStream {
    pub(super) fn new(projection: SecretProjection) -> Self {
        Self {
            projection,
            pending: String::new(),
        }
    }

    /// Return only bytes that cannot belong to a later completed secret match.
    /// Retained state is a proper prefix of a finite variant, at most its length
    /// minus one. Each logical stream needs its own instance.
    pub fn push(&mut self, fragment: &str) -> String {
        self.pending.push_str(fragment);
        let (output, consumed, _) = self.projection.prefix(&self.pending, false);
        self.pending.drain(..consumed);
        output
    }

    /// Flush at terminal, cancellation, error, or logical stream completion.
    pub fn finish(&mut self) -> String {
        self.projection.text(&std::mem::take(&mut self.pending)).0
    }
}
