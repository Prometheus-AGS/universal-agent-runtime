//! Complete copied event payloads. Streaming producers must project fragments
//! with independent SecretStreams before publishing/persisting content.
use super::RunSecretScrubber;
use crate::uar::domain::events::{ArtifactPayload, NormalizedEvent};

/// Explicit private admission publication state shared by emitter clones.
/// Raw staged content never enters EventHistory or a persistence provider.
#[derive(Default)]
pub struct AdmissionProjection {
    capture: Option<RunSecretScrubber>,
    pending: Vec<NormalizedEvent>,
    bytes: usize,
    refused: Option<&'static str>,
    reduced_failure_only: bool,
}

impl std::fmt::Debug for AdmissionProjection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdmissionProjection")
            .field("events", &self.pending.len())
            .field("bytes", &self.bytes)
            .field("ready", &self.capture.is_some())
            .field("refused", &self.refused)
            .field("reduced_failure_only", &self.reduced_failure_only)
            .finish()
    }
}

impl AdmissionProjection {
    /// Discard unpublishable pre-admission content before already-reduced host
    /// failure frames are emitted. The resulting projection admits only
    /// terminal lifecycle and error frames, so a caller cannot use this seam
    /// to publish ordinary content before full admission completes.
    pub(crate) fn begin_reduced_failure(&mut self, capture: RunSecretScrubber) {
        self.pending.clear();
        self.bytes = 0;
        self.refused = None;
        self.capture = Some(capture);
        self.reduced_failure_only = true;
    }

    /// Presentation observations require a completed admission corpus. Reduced
    /// binding failures carry only diagnostics, even when a scrubber is present.
    pub(crate) fn suppresses_presentation(&self) -> bool {
        self.capture.is_none() || self.reduced_failure_only
    }

    pub fn project_or_stage(&mut self, mut event: NormalizedEvent) -> Option<NormalizedEvent> {
        if let Some(capture) = &self.capture {
            if self.reduced_failure_only
                && !matches!(
                    &event,
                    NormalizedEvent::Error { .. }
                        | NormalizedEvent::RunDone { .. }
                        | NormalizedEvent::RunDoneWithUsage { .. }
                        | NormalizedEvent::Cancelled { .. }
                )
            {
                return None;
            }
            capture.project_event(&mut event);
            return Some(event);
        }
        match &mut event {
            NormalizedEvent::Error { message, .. } => {
                self.pending.clear();
                self.bytes = 0;
                self.refused.get_or_insert("ADMISSION_FAILED");
                *message = "Run admission failed before protected content publication".into();
                return Some(event);
            }
            NormalizedEvent::RunDone { .. }
            | NormalizedEvent::RunDoneWithUsage { .. }
            | NormalizedEvent::Cancelled { .. } => {
                self.pending.clear();
                self.bytes = 0;
                self.refused.get_or_insert("ADMISSION_ENDED");
                return Some(event);
            }
            _ => {}
        }
        if self.refused.is_some() {
            return None;
        }
        let encoded = serde_json::to_vec(&event).map_or(usize::MAX, |bytes| bytes.len());
        if self.pending.len() >= 1000
            || self
                .bytes
                .checked_add(encoded)
                .is_none_or(|bytes| bytes > 4 * 1024 * 1024)
        {
            self.pending.clear();
            self.bytes = 0;
            self.refused = Some("ADMISSION_CONTENT_LIMIT_EXCEEDED");
            return None;
        }
        self.bytes += encoded;
        self.pending.push(event);
        None
    }

    /// Publish one immutable complete corpus to all previously cloned sinks.
    /// The caller emits returned events through the now-ready projection.
    pub fn complete(
        &mut self,
        capture: RunSecretScrubber,
    ) -> Result<Vec<NormalizedEvent>, &'static str> {
        if let Some(code) = self.refused {
            return Err(code);
        }
        self.capture = Some(capture);
        self.bytes = 0;
        Ok(std::mem::take(&mut self.pending))
    }
}

#[cfg(test)]
mod admission_tests {
    use super::*;

    #[test]
    fn early_admission_refusal_keeps_content_and_presentations_private() {
        let mut projection = AdmissionProjection::default();
        assert!(projection.suppresses_presentation());
        assert!(projection.project_or_stage(NormalizedEvent::ChatDelta {
            run_id: "refused".into(),
            text_delta: "unadmitted content".into(),
        }).is_none());
        let event = projection.project_or_stage(NormalizedEvent::Error {
            run_id: "refused".into(),
            code: "budget_admission_failed".into(),
            message: "private accounting detail".into(),
        }).expect("public terminal admission error");
        assert_eq!(event, NormalizedEvent::Error {
            run_id: "refused".into(),
            code: "budget_admission_failed".into(),
            message: "Run admission failed before protected content publication".into(),
        });
        assert!(projection.suppresses_presentation());
        assert!(projection.project_or_stage(NormalizedEvent::RunDone {
            run_id: "refused".into(),
        }).is_some());
        assert!(projection.suppresses_presentation());
        assert_eq!(projection.complete(RunSecretScrubber::default()), Err("ADMISSION_FAILED"));
        assert!(projection.suppresses_presentation());
    }

    #[test]
    fn completed_admission_restores_content_and_presentation_publication() {
        let mut projection = AdmissionProjection::default();
        let event = NormalizedEvent::ChatDelta {
            run_id: "admitted".into(),
            text_delta: "admitted content".into(),
        };
        assert!(projection.project_or_stage(event.clone()).is_none());
        assert!(projection.suppresses_presentation());
        let staged = projection.complete(RunSecretScrubber::default()).unwrap();
        assert_eq!(staged, vec![event.clone()]);
        assert!(!projection.suppresses_presentation());
        assert_eq!(projection.project_or_stage(event.clone()), Some(event));
        assert!(projection.project_or_stage(NormalizedEvent::RunDone {
            run_id: "admitted".into(),
        }).is_some());
        assert!(!projection.suppresses_presentation());
    }

    #[test]
    fn reduced_failure_capture_does_not_admit_content_or_presentations() {
        let mut projection = AdmissionProjection::default();
        projection.begin_reduced_failure(RunSecretScrubber::default());
        assert!(projection.suppresses_presentation());
        assert!(projection.project_or_stage(NormalizedEvent::ChatDelta {
            run_id: "reduced".into(),
            text_delta: "unadmitted content".into(),
        }).is_none());
        let event = NormalizedEvent::Error {
            run_id: "reduced".into(),
            code: "orchestrator_start_failed".into(),
            message: "Run binding refused during client_construction; diagnostic reference fixture".into(),
        };
        assert_eq!(projection.project_or_stage(event.clone()), Some(event));
        assert!(projection.suppresses_presentation());
    }
}

impl RunSecretScrubber {
    /// Admission guard for immutable executable snapshots: no silent rewrite
    /// of the artifact whose revision and policy authorize execution.
    pub fn contains_captured(&self, value: &serde_json::Value) -> bool {
        self.0.value(value.clone()).1
    }
    /// Project only ordinary content fields in a model message; role, call IDs,
    /// function targets and provider control fields stay byte-exact.
    pub fn project_message_json(&self, message: &mut serde_json::Value) {
        if let Some(content) = message.get_mut("content") {
            *content = self.project_value(std::mem::take(content));
        }
        if let Some(calls) = message
            .get_mut("tool_calls")
            .and_then(|value| value.as_array_mut())
        {
            for call in calls {
                if let Some(arguments) = call
                    .get_mut("function")
                    .and_then(|function| function.get_mut("arguments"))
                    && let Some(text) = arguments.as_str()
                {
                    *arguments = self.project_json_text(text).into();
                }
            }
        }
    }

    pub fn project_message(&self, message: &mut crate::llm::Message) {
        use crate::llm::{ContentPart, MessageContent};
        match &mut message.content {
            MessageContent::Text { content } => *content = self.scrub(content),
            MessageContent::Parts { content } => {
                for part in content {
                    match part {
                        ContentPart::Text { text } => *text = self.scrub(text),
                        ContentPart::ImageUrl { image_url } => {
                            image_url.url = self.scrub(&image_url.url)
                        }
                    }
                }
            }
        }
        if let Some(calls) = &mut message.tool_calls {
            for call in calls {
                call.function.arguments = self.project_json_text(&call.function.arguments);
            }
        }
    }

    /// Project canonical protected history without turning validated inline
    /// images into generic secret-redaction placeholders. Image URLs remain
    /// eligible only when they still satisfy the strict ingress syntax rule.
    pub(crate) fn project_canonical_message(
        &self,
        message: &mut crate::llm::Message,
    ) -> anyhow::Result<()> {
        use crate::llm::{ContentPart, MessageContent};

        match &mut message.content {
            MessageContent::Text { content } => *content = self.scrub(content),
            MessageContent::Parts { content } => {
                for part in content {
                    match part {
                        ContentPart::Text { text } => *text = self.scrub(text),
                        ContentPart::ImageUrl { image_url } => {
                            // The ordinary typed ingress has already validated
                            // supported inline images. Preserve those bytes for
                            // canonical history. Legacy image transports retain
                            // the prior generic projection rather than gaining
                            // a new global rejection path here.
                            if crate::llm::inline_image::inline_image_metadata(image_url).is_err() {
                                image_url.url = self.scrub(&image_url.url);
                            }
                        }
                    }
                }
            }
        }
        if let Some(calls) = &mut message.tool_calls {
            for call in calls {
                call.function.arguments = self.project_json_text(&call.function.arguments);
            }
        }
        Ok(())
    }

    /// JSON canonical-history adapter for provider and graph checkpoint seams.
    /// It accepts only serialized [`crate::llm::Message`] values, so arbitrary
    /// JSON cannot use the inline-image preservation path.
    pub(crate) fn project_canonical_message_json(
        &self,
        message: &mut serde_json::Value,
    ) -> anyhow::Result<()> {
        let mut typed = serde_json::from_value::<crate::llm::Message>(std::mem::take(message))?;
        self.project_canonical_message(&mut typed)?;
        *message = serde_json::to_value(typed)?;
        Ok(())
    }

    pub fn project_fragments(&self, fragments: &mut [crate::uar::runtime::prompt::PromptFragment]) {
        for fragment in fragments {
            fragment.content = self.scrub(&fragment.content);
            fragment.content_hash = crate::uar::runtime::prompt::fragment::content_hash(
                fragment.role,
                fragment.section,
                &fragment.content,
            );
        }
    }

    /// Preserve typed control envelopes, IDs, approval capabilities and targets.
    /// Only copied human/model/tool content is projected here.
    pub fn project_event(&self, event: &mut NormalizedEvent) {
        use NormalizedEvent::*;
        match event {
            ChatDelta { text_delta, .. }
            | ThinkingDelta { text_delta, .. }
            | ReasoningDelta { text_delta, .. } => *text_delta = self.scrub(text_delta),
            Error { message, .. }
            | RagDiagnostic { message, .. }
            | PresentationDiagnostic { message, .. } => *message = self.scrub(message),
            ClientToolsRequested { calls, .. } => {
                for call in calls {
                    call.function.arguments = self.project_json_text(&call.function.arguments);
                }
            }
            ToolStart { input, .. } => *input = self.project_value(std::mem::take(input)),
            ToolDelta { delta, .. } => *delta = self.project_value(std::mem::take(delta)),
            ToolEnd { output, .. } => *output = self.project_value(std::mem::take(output)),
            ToolCallApprovalRequired {
                arguments_json,
                risk_reason,
                ..
            } => {
                *arguments_json = self.project_json_text(arguments_json);
                *risk_reason = self.scrub(risk_reason);
            }
            ToolCallDenied { reason, .. } => *reason = self.scrub(reason),
            Citation { sources, .. } => {
                for source in sources {
                    source.title = self.scrub(&source.title);
                    source.url = self.scrub(&source.url);
                    source.snippet = source.snippet.take().map(|text| self.scrub(&text));
                }
            }
            RagCitations { citations, .. } => {
                for citation in citations {
                    citation.document_name = self.scrub(&citation.document_name);
                    citation.snippet = self.scrub(&citation.snippet);
                }
            }
            MemoryRecall { items, .. } => {
                for item in items {
                    item.value = self.scrub(&item.value);
                }
            }
            MemoryMutation { content, .. } => *content = self.scrub(content),
            SkillActivated { title, .. } => *title = self.scrub(title),
            AgentSelected { selected_agent_name, .. } => {
                *selected_agent_name = self.scrub(selected_agent_name);
            }
            SycophancyFlagged {
                classifications, ..
            } => {
                for item in classifications {
                    item.rationale = self.scrub(&item.rationale);
                }
            }
            SycophancyCorrected { corrected_text, .. } => {
                *corrected_text = self.scrub(corrected_text)
            }
            Artifact { artifact, .. }
            | ArtifactDisplay { artifact, .. }
            | ArtifactInputRequest { artifact, .. } => self.project_artifact(artifact),
            StatePatch { patch, .. } => {
                for operation in patch {
                    if let Some(value) = operation.value.take() {
                        operation.value = Some(self.project_value(value));
                    }
                }
            }
            _ => {} // Remaining variants are bounded control/lifecycle data.
        }
    }

    /// A copied JSON argument/result retains valid JSON even when a captured
    /// value includes JSON punctuation. Non-JSON content remains plain text.
    pub fn project_json_text(&self, text: &str) -> String {
        match serde_json::from_str::<serde_json::Value>(text) {
            Ok(value) => self.project_value(value).to_string(),
            Err(_) => self.scrub(text),
        }
    }

    fn project_artifact(&self, artifact: &mut ArtifactPayload) {
        artifact.title = self.scrub(&artifact.title);
        artifact.content = self.scrub(&artifact.content);
        if let Some(warnings) = artifact.metadata.get_mut("warnings") {
            *warnings = self.project_value(std::mem::take(warnings));
        }
        // Metadata includes presentation identity and surface capabilities;
        // those control values must remain exact, as do artifact_id/type.
    }
}
