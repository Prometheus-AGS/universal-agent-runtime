//! Host-owned graph dialogue receipts. Tool results enter run/conversation
//! history before event publication, including when a node waiter is dropped.

use std::collections::BTreeMap;

use crate::llm::{Message, MessageContent, MessageRole, ToolCall, ToolCallFunction};
use crate::normalized::NormalizedEvent;
use crate::session::Session;
use crate::uar::a2ui::realtime::A2uiReplayBackbone;
use crate::uar::domain::events::{NormalizedEvent as RuntimeEvent, RuntimeEventSink};
use crate::uar::runtime::context::normalize::{SyntheticReason, synthetic_tool_result};
use crate::uar::runtime::prompt::{TurnInterrupted, TurnInterruptionReason};

pub(crate) struct GraphModelTurn {
    pub(crate) messages: Vec<Message>,
    pub(crate) text: String,
    pub(crate) error: Option<String>,
    pub(crate) receipt_error: Option<String>,
}

pub(crate) struct GraphTurnTranscript {
    run_id: String,
    history: Vec<Message>,
    dialogue: Session,
    conversation: Session,
    a2ui: std::sync::Arc<dyn A2uiReplayBackbone>,
    presentations: std::sync::Arc<super::super::presentations::RunPresentationSnapshot>,
    cancellation: tokio_util::sync::CancellationToken,
    text: String,
    calls: Vec<ToolCall>,
    outstanding: BTreeMap<String, (usize, String)>,
}

impl GraphTurnTranscript {
    pub(crate) fn new(
        run_id: String,
        history: Vec<Message>,
        dialogue: Session,
        conversation: Session,
        a2ui: std::sync::Arc<dyn A2uiReplayBackbone>,
        presentations: std::sync::Arc<super::super::presentations::RunPresentationSnapshot>,
        cancellation: tokio_util::sync::CancellationToken,
    ) -> Self {
        Self {
            run_id,
            history,
            dialogue,
            conversation,
            a2ui,
            presentations,
            cancellation,
            text: String::new(),
            calls: Vec::new(),
            outstanding: BTreeMap::new(),
        }
    }

    fn record(&mut self, message: Message) {
        self.dialogue.add_message(message.clone());
        self.conversation.add_message(message.clone());
        self.history.push(message);
    }

    fn flush_assistant(&mut self) {
        if self.text.is_empty() && self.calls.is_empty() {
            return;
        }
        let message = Message {
            role: MessageRole::Assistant,
            content: MessageContent::text(std::mem::take(&mut self.text)),
            tool_call_id: None,
            tool_calls: (!self.calls.is_empty()).then(|| std::mem::take(&mut self.calls)),
        };
        self.record(message);
    }

    pub(crate) fn has_outstanding_tools(&self) -> bool {
        !self.outstanding.is_empty()
    }

    /// Returns true only for the orchestrator's explicit terminal event.
    pub(crate) async fn observe(
        &mut self,
        event: NormalizedEvent,
        events: &dyn RuntimeEventSink,
    ) -> anyhow::Result<bool> {
        let run_id = self.run_id.clone();
        let event = match event {
            NormalizedEvent::MessageDelta { text } => {
                self.text.push_str(&text);
                Some(RuntimeEvent::ChatDelta {
                    run_id,
                    text_delta: text,
                })
            }
            NormalizedEvent::ThinkingDelta { text } => Some(RuntimeEvent::ThinkingDelta {
                run_id,
                text_delta: text,
            }),
            NormalizedEvent::ReasoningDelta { text } => Some(RuntimeEvent::ReasoningDelta {
                run_id,
                text_delta: text,
            }),
            NormalizedEvent::ToolCallComplete {
                call_index,
                id,
                name,
                arguments_json,
            } => {
                anyhow::ensure!(
                    !id.is_empty() && !self.outstanding.contains_key(&id),
                    "Graph model returned a missing or duplicate tool call ID"
                );
                self.outstanding
                    .insert(id.clone(), (call_index, name.clone()));
                self.calls.push(ToolCall {
                    id: id.clone(),
                    call_type: "function".into(),
                    function: ToolCallFunction {
                        name: name.clone(),
                        arguments: arguments_json.clone(),
                    },
                });
                Some(RuntimeEvent::ToolStart {
                    run_id,
                    call_index,
                    tool_call_id: id,
                    tool: name,
                    input: serde_json::from_str(&arguments_json)
                        .unwrap_or(serde_json::Value::String(arguments_json)),
                })
            }
            NormalizedEvent::ToolResult {
                id,
                name,
                content,
                success,
            } => {
                let (call_index, tool) = self
                    .outstanding
                    .get(&id)
                    .ok_or_else(|| anyhow::anyhow!("Graph model returned an orphan tool result"))?
                    .clone();
                anyhow::ensure!(tool == name, "Graph tool result identity changed");
                self.flush_assistant();
                self.record(Message {
                    role: MessageRole::Tool,
                    content: MessageContent::text(content.clone()),
                    tool_call_id: Some(id.clone()),
                    tool_calls: None,
                });
                self.outstanding.remove(&id);
                let output = serde_json::from_str(&content)
                    .unwrap_or_else(|_| serde_json::Value::String(content));
                crate::uar::runtime::a2ui_output::publish_tool_output(
                    &run_id,
                    &tool,
                    success,
                    &id,
                    &self.presentations,
                    &self.cancellation,
                    self.a2ui.as_ref(),
                    events,
                )
                .await;
                Some(RuntimeEvent::ToolEnd {
                    run_id,
                    call_index,
                    tool_call_id: id,
                    tool,
                    output,
                    ok: success,
                })
            }
            NormalizedEvent::ToolCallDelta {
                call_index,
                id: Some(id),
                arguments_delta: Some(delta),
                ..
            } => Some(RuntimeEvent::ToolDelta {
                run_id,
                call_index,
                tool_call_id: id,
                delta: serde_json::Value::String(delta),
            }),
            NormalizedEvent::SkillActivation { name, status } => {
                Some(RuntimeEvent::SkillActivated {
                    run_id,
                    skill_id: name.clone(),
                    title: name,
                    selection_method: status,
                })
            }
            NormalizedEvent::CitationAdded(citation) => Some(RuntimeEvent::Citation {
                run_id,
                sources: vec![crate::uar::domain::events::CitationSource {
                    title: citation.title.unwrap_or_else(|| citation.url.clone()),
                    url: citation.url,
                    snippet: citation.snippet,
                }],
            }),
            NormalizedEvent::Error { message, .. } => {
                anyhow::bail!("Graph model turn failed: {message}")
            }
            NormalizedEvent::Done => return Ok(true),
            // The graph engine owns RuntimeStep numbering. Model wrappers own
            // usage charges; neither counter is reconstructed from UI events.
            _ => None,
        };
        if let Some(event) = event {
            events.emit(event).await;
        }
        Ok(false)
    }

    pub(crate) fn finish(mut self, mut error: Option<String>, cancelled: bool) -> GraphModelTurn {
        if !self.outstanding.is_empty() && error.is_none() {
            error = Some("Graph model ended with unresolved tool calls".into());
        }
        let text = self.text.clone();
        if error.is_some() && (!self.text.is_empty() || !self.calls.is_empty()) {
            let marker = TurnInterrupted {
                run_id: self.run_id.clone(),
                reason: if cancelled {
                    TurnInterruptionReason::Cancelled
                } else {
                    TurnInterruptionReason::ProviderError
                },
            }
            .into_fragment();
            self.text
                .push_str(&format!("\n\n{}", marker.marked_content()));
        }
        self.flush_assistant();
        let reason = if cancelled {
            SyntheticReason::Cancelled
        } else {
            SyntheticReason::Error("Graph turn ended before a tool result was received".into())
        };
        let missing = std::mem::take(&mut self.outstanding);
        for (id, _) in missing {
            self.record(synthetic_tool_result(&id, &reason));
        }
        GraphModelTurn {
            messages: self.history,
            text,
            error,
            receipt_error: None,
        }
    }
}
