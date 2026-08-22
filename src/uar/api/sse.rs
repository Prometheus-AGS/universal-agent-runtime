use crate::uar::domain::events::NormalizedEvent;
use crate::uar::runtime::manager::StreamEvent;
use axum::response::sse::{Event, Sse};
use futures::{Stream, StreamExt};
use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::time::Duration;

#[derive(Clone, Debug)]
pub(crate) struct AguiReplaySnapshot {
    cursor: u64,
    run_id: String,
    state: Option<serde_json::Value>,
    messages: Vec<serde_json::Value>,
}

fn pointer_segments(path: &str) -> Option<Vec<String>> {
    if path.is_empty() {
        return Some(Vec::new());
    }
    path.strip_prefix('/').map(|pointer| {
        pointer
            .split('/')
            .map(|segment| segment.replace("~1", "/").replace("~0", "~"))
            .collect()
    })
}

fn value_at_mut<'a>(
    mut value: &'a mut serde_json::Value,
    segments: &[String],
) -> Option<&'a mut serde_json::Value> {
    for segment in segments {
        value = match value {
            serde_json::Value::Object(object) => object.get_mut(segment)?,
            serde_json::Value::Array(array) => array.get_mut(segment.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    Some(value)
}

fn apply_state_patch(
    state: &mut serde_json::Value,
    patch: &[crate::uar::domain::events::StatePatchOp],
) -> bool {
    for operation in patch {
        let Some(segments) = pointer_segments(&operation.path) else {
            return false;
        };
        if segments.is_empty() {
            match operation.op.as_str() {
                "add" | "replace" => {
                    let Some(value) = operation.value.clone() else {
                        return false;
                    };
                    *state = value;
                }
                _ => return false,
            }
            continue;
        }

        let Some(key) = segments.last().cloned() else {
            return false;
        };
        let Some(parent) = value_at_mut(state, &segments[..segments.len() - 1]) else {
            return false;
        };
        match parent {
            serde_json::Value::Object(object) => match operation.op.as_str() {
                "add" => {
                    let Some(value) = operation.value.clone() else {
                        return false;
                    };
                    object.insert(key, value);
                }
                "replace" if object.contains_key(&key) => {
                    let Some(value) = operation.value.clone() else {
                        return false;
                    };
                    object.insert(key, value);
                }
                "remove" if object.remove(&key).is_some() => {}
                _ => return false,
            },
            serde_json::Value::Array(array) => {
                let index = if key == "-" {
                    array.len()
                } else if let Ok(index) = key.parse::<usize>() {
                    index
                } else {
                    return false;
                };
                match operation.op.as_str() {
                    "add" if index <= array.len() => {
                        let Some(value) = operation.value.clone() else {
                            return false;
                        };
                        array.insert(index, value);
                    }
                    "replace" if index < array.len() => {
                        let Some(value) = operation.value.clone() else {
                            return false;
                        };
                        array[index] = value;
                    }
                    "remove" if index < array.len() => {
                        array.remove(index);
                    }
                    _ => return false,
                }
            }
            _ => return false,
        }
    }
    true
}

#[must_use]
pub(crate) fn build_agui_replay_snapshot(
    run_id: &str,
    history: &[StreamEvent],
    cursor: u64,
) -> AguiReplaySnapshot {
    let mut state = serde_json::json!({
        "run": null,
        "a2ui": { "surfaces": {} }
    });
    let mut state_synchronized = true;
    let mut assistant_text = String::new();

    for event in history.iter().filter(|event| event.id <= cursor) {
        match &event.event {
            NormalizedEvent::StatePatch { patch, .. } if state_synchronized => {
                state_synchronized = apply_state_patch(&mut state, patch);
            }
            NormalizedEvent::ChatDelta { text_delta, .. } => assistant_text.push_str(text_delta),
            _ => {}
        }
    }

    let messages = if assistant_text.is_empty() {
        Vec::new()
    } else {
        vec![serde_json::json!({
            "id": format!("{run_id}:assistant"),
            "role": "assistant",
            "content": assistant_text,
        })]
    };
    AguiReplaySnapshot {
        cursor,
        run_id: run_id.to_string(),
        state: state_synchronized.then_some(state),
        messages,
    }
}

struct AguiSpecFrame {
    event_name: &'static str,
    payload: serde_json::Value,
    ordinal: u64,
}

#[derive(Default)]
struct AguiSpecProjector {
    seen_tool_calls: HashSet<String>,
    pending_tool_args: HashMap<String, Vec<AguiSpecFrame>>,
}

impl AguiSpecProjector {
    fn project(&mut self, event: &NormalizedEvent) -> Vec<AguiSpecFrame> {
        if let NormalizedEvent::ToolDelta { tool_call_id, .. } = event
            && !self.seen_tool_calls.contains(tool_call_id)
        {
            if let Some((event_name, payload)) = to_agui_spec_event(event) {
                let pending = self
                    .pending_tool_args
                    .entry(tool_call_id.clone())
                    .or_default();
                pending.push(AguiSpecFrame {
                    event_name,
                    payload,
                    ordinal: 0,
                });
            }
            return Vec::new();
        }

        let tool_start = match event {
            NormalizedEvent::ToolStart {
                run_id,
                tool_call_id,
                tool,
                ..
            }
            | NormalizedEvent::ToolEnd {
                run_id,
                tool_call_id,
                tool,
                ..
            } => Some((run_id, tool_call_id, tool.as_str())),
            _ => None,
        };

        let mut frames = Vec::new();
        if let Some((run_id, tool_call_id, tool_name)) = tool_start
            && self.seen_tool_calls.insert(tool_call_id.clone())
        {
            frames.push(AguiSpecFrame {
                event_name: "TOOL_CALL_START",
                payload: serde_json::json!({
                    "type": "TOOL_CALL_START",
                    "profile": "uar.agui/1",
                    "toolCallId": tool_call_id,
                    "toolCallName": tool_name,
                    "threadId": run_id,
                    "runId": run_id,
                }),
                ordinal: 0,
            });
            frames.extend(
                self.pending_tool_args
                    .remove(tool_call_id)
                    .unwrap_or_default(),
            );
            if matches!(event, NormalizedEvent::ToolEnd { .. }) {
                frames.push(AguiSpecFrame {
                    event_name: "TOOL_CALL_END",
                    payload: serde_json::json!({
                        "type": "TOOL_CALL_END",
                        "profile": "uar.agui/1",
                        "toolCallId": tool_call_id,
                        "toolCallName": tool_name,
                        "threadId": run_id,
                        "runId": run_id,
                    }),
                    ordinal: 0,
                });
            }
        }
        if let Some((event_name, payload)) = to_agui_spec_event(event) {
            frames.push(AguiSpecFrame {
                event_name,
                payload,
                ordinal: 0,
            });
        }
        for (ordinal, frame) in frames.iter_mut().enumerate() {
            frame.ordinal = ordinal as u64;
        }
        frames
    }
}

fn replay_snapshot_events(snapshot: AguiReplaySnapshot) -> Vec<Result<Event, Infallible>> {
    let source_id = snapshot.cursor.to_string();
    let mut frames = Vec::new();
    if let Some(state) = snapshot.state {
        let payload = enrich_agui_spec_payload(
            "STATE_SNAPSHOT",
            serde_json::json!({
                "type": "STATE_SNAPSHOT", "profile": "uar.agui/1",
                "threadId": snapshot.run_id, "runId": snapshot.run_id,
                "snapshot": state,
            }),
            &source_id,
            1,
        );
        frames.push(Ok(Event::default()
            .event("STATE_SNAPSHOT")
            .id(source_id.clone())
            .data(payload.to_string())));
    }
    let payload = enrich_agui_spec_payload(
        "MESSAGES_SNAPSHOT",
        serde_json::json!({
            "type": "MESSAGES_SNAPSHOT", "profile": "uar.agui/1",
            "threadId": snapshot.run_id, "runId": snapshot.run_id,
            "messages": snapshot.messages,
        }),
        &source_id,
        2,
    );
    frames.push(Ok(Event::default()
        .event("MESSAGES_SNAPSHOT")
        .id(source_id)
        .data(payload.to_string())));
    frames
}

pub(crate) fn build_sse_response<S>(
    stream: S,
    agui_spec: bool,
    replay_snapshot: Option<AguiReplaySnapshot>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>> + Send>
where
    S: Stream<Item = StreamEvent> + Send + 'static,
{
    let snapshot_stream = futures::stream::iter(
        replay_snapshot
            .filter(|_| agui_spec)
            .map(replay_snapshot_events)
            .unwrap_or_default(),
    );
    let mut agui_spec_projector = AguiSpecProjector::default();
    let event_stream = stream.flat_map(move |event| {
        let source_id = event.id.to_string();
        let frames = if agui_spec {
            agui_spec_projector
                .project(&event.event)
                .into_iter()
                .map(|frame| {
                    let payload = enrich_agui_spec_payload(
                        frame.event_name,
                        frame.payload,
                        &source_id,
                        frame.ordinal,
                    );
                    Ok(Event::default()
                        .event(frame.event_name)
                        .id(source_id.clone())
                        .data(payload.to_string()))
                })
                .collect()
        } else {
            to_agui_event(&event.event)
                .map(|(event_name, payload)| {
                    let json = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
                    vec![Ok(Event::default()
                        .event(event_name)
                        .id(source_id)
                        .data(json))]
                })
                .unwrap_or_default()
        };
        futures::stream::iter(frames)
    });

    Sse::new(snapshot_stream.chain(event_stream))
        .keep_alive(axum::response::sse::KeepAlive::new().interval(Duration::from_secs(15)))
}

pub fn to_agui_event(event: &NormalizedEvent) -> Option<(&'static str, serde_json::Value)> {
    match event {
        NormalizedEvent::RunStart { run_id, agent_id } => Some((
            "agui.stream.start",
            serde_json::json!({
                "kind": "stream",
                "phase": "start",
                "request_id": run_id,
                "agent_id": agent_id
            }),
        )),
        NormalizedEvent::ChatDelta { run_id, text_delta } => Some((
            "agui.message.delta",
            serde_json::json!({
                "kind": "message",
                "phase": "delta",
                "request_id": run_id,
                "delta": { "text": text_delta }
            }),
        )),
        NormalizedEvent::ThinkingDelta { run_id, text_delta } => Some((
            "agui.thinking.delta",
            serde_json::json!({
                "kind": "thinking",
                "phase": "delta",
                "request_id": run_id,
                "delta": { "text": text_delta }
            }),
        )),
        NormalizedEvent::ReasoningDelta { run_id, text_delta } => Some((
            "agui.reasoning.delta",
            serde_json::json!({
                "kind": "reasoning",
                "phase": "delta",
                "request_id": run_id,
                "delta": { "text": text_delta }
            }),
        )),
        NormalizedEvent::Citation { run_id, sources } => {
            if sources.is_empty() {
                return None;
            }
            let citations: Vec<serde_json::Value> = sources
                .iter()
                .enumerate()
                .map(|(index, source)| {
                    serde_json::json!({
                        "index": index,
                        "url": source.url,
                        "title": source.title,
                        "snippet": source.snippet
                    })
                })
                .collect();

            Some((
                "agui.citation.added",
                serde_json::json!({
                    "kind": "citation",
                    "phase": "added",
                    "request_id": run_id,
                    "citation": citations.first(),
                    "citations": citations
                }),
            ))
        }
        NormalizedEvent::RagCitations { run_id, citations } => {
            if citations.is_empty() {
                return None;
            }
            Some((
                "agui.rag_citations",
                serde_json::json!({
                    "kind": "rag_citations",
                    "phase": "added",
                    "request_id": run_id,
                    "citations": citations
                }),
            ))
        }
        NormalizedEvent::MemoryRecall { run_id, items } => {
            // Distinguish pre-call context hits (source == "memory_context") from
            // model-provided memory updates so clients can render them differently.
            let is_context_injection = items.first().is_some_and(|i| i.source == "memory_context");
            if is_context_injection {
                Some((
                    "agui.memory.recall",
                    serde_json::json!({
                        "kind": "memory",
                        "phase": "recall",
                        "request_id": run_id,
                        "items": items,
                        "count": items.len()
                    }),
                ))
            } else {
                // Backward-compatible path: model-provided memory updates.
                let value = serde_json::to_string(items).unwrap_or_default();
                Some((
                    "agui.memory.update",
                    serde_json::json!({
                        "kind": "memory",
                        "phase": "update",
                        "request_id": run_id,
                        "key": "recall",
                        "value": value,
                        "operation": "set"
                    }),
                ))
            }
        }
        NormalizedEvent::SkillActivated {
            run_id,
            skill_id,
            title,
            selection_method,
        } => Some((
            "agui.skill.activated",
            serde_json::json!({
                "kind": "skill",
                "phase": "activated",
                "request_id": run_id,
                "skill": {
                    "id": skill_id,
                    "title": title
                },
                "selection_method": selection_method
            }),
        )),
        NormalizedEvent::ToolDelta {
            run_id,
            call_index,
            tool_call_id,
            delta,
        } => Some((
            "agui.tool_call.delta",
            serde_json::json!({
                "kind": "tool_call",
                "phase": "delta",
                "request_id": run_id,
                "call_index": call_index,
                "id": tool_call_id,
                "delta": { "arguments": delta.to_string() }
            }),
        )),
        NormalizedEvent::ToolStart {
            run_id,
            call_index,
            tool_call_id,
            tool,
            input,
        } => Some((
            "agui.tool_call.complete",
            serde_json::json!({
                "kind": "tool_call",
                "phase": "complete",
                "request_id": run_id,
                "call_index": call_index,
                "id": tool_call_id,
                "name": tool,
                "arguments_json": input.to_string()
            }),
        )),
        NormalizedEvent::ToolEnd {
            run_id,
            call_index,
            tool_call_id,
            tool,
            output,
            ok,
        } => Some((
            "agui.tool_result",
            serde_json::json!({
                "kind": "tool_result",
                "request_id": run_id,
                "call_index": call_index,
                "id": tool_call_id,
                "name": tool,
                "content": output.to_string(),
                "success": ok
            }),
        )),
        NormalizedEvent::Artifact { run_id, artifact } => Some((
            "agui.artifact",
            serde_json::json!({
                "kind": "artifact",
                "phase": "complete",
                "request_id": run_id,
                "artifact_id": artifact.artifact_id,
                "artifact_type": artifact.artifact_type,
                "title": artifact.title,
                "content": artifact.content,
                "language": artifact.language,
                "metadata": artifact.metadata
            }),
        )),
        NormalizedEvent::ArtifactDisplay { run_id, artifact } => Some((
            "agui.artifact",
            serde_json::json!({
                "kind": "artifact",
                "phase": "complete",
                "request_id": run_id,
                "artifact_id": artifact.artifact_id,
                "artifact_type": artifact.artifact_type,
                "title": artifact.title,
                "content": artifact.content,
                "language": artifact.language,
                "metadata": artifact.metadata
            }),
        )),
        NormalizedEvent::ArtifactInputRequest { run_id, artifact } => Some((
            "agui.artifact_input_request",
            serde_json::json!({
                "kind": "artifact_input_request",
                "request_id": run_id,
                "artifact_id": artifact.artifact_id,
                "artifact_type": artifact.artifact_type,
                "title": artifact.title,
                "content": artifact.content,
                "metadata": artifact.metadata
            }),
        )),
        NormalizedEvent::Error {
            run_id,
            code,
            message,
        } => Some((
            "agui.error",
            serde_json::json!({
                "kind": "error",
                "request_id": run_id,
                "message": message,
                "code": code
            }),
        )),
        NormalizedEvent::RunDone { run_id } => Some((
            "agui.done",
            serde_json::json!({
                "kind": "done",
                "request_id": run_id
            }),
        )),
        NormalizedEvent::Cancelled { run_id } => Some((
            "agui.cancelled",
            serde_json::json!({
                "kind": "cancelled",
                "request_id": run_id
            }),
        )),
        NormalizedEvent::SycophancyFlagged {
            run_id,
            sycophancy_score,
            has_critical,
            correction_mandatory,
            classifications,
        } => Some((
            "agui.quality.sycophancy",
            serde_json::json!({
                "kind": "quality",
                "phase": "sycophancy",
                "request_id": run_id,
                "score": sycophancy_score,
                "has_critical": has_critical,
                "correction_mandatory": correction_mandatory,
                "classifications": classifications,
            }),
        )),
        NormalizedEvent::SycophancyCorrected {
            run_id,
            corrected_text,
        } => Some((
            "agui.quality.sycophancy_corrected",
            serde_json::json!({
                "kind": "quality",
                "phase": "sycophancy_corrected",
                "request_id": run_id,
                "corrected_text": corrected_text,
            }),
        )),
        NormalizedEvent::GuardrailFlagged {
            run_id,
            category,
            reason,
        } => Some((
            "agui.guardrail",
            serde_json::json!({
                "kind": "guardrail",
                "phase": "flagged",
                "request_id": run_id,
                "category": category,
                "reason": reason,
            }),
        )),
        NormalizedEvent::RunDoneWithUsage {
            run_id,
            input_tokens,
            output_tokens,
            total_tokens,
            cost_usd_estimate,
            model,
        } => Some((
            "agui.done",
            serde_json::json!({
                "kind": "done",
                "request_id": run_id,
                "usage": {
                    "input_tokens": input_tokens,
                    "output_tokens": output_tokens,
                    "total_tokens": total_tokens,
                    "cost_usd_estimate": cost_usd_estimate,
                    "model": model
                }
            }),
        )),
        NormalizedEvent::StatePatch { run_id, patch } => Some((
            "agui.state.patch",
            serde_json::json!({
                "kind": "state",
                "phase": "patch",
                "request_id": run_id,
                "patch": patch
            }),
        )),
        NormalizedEvent::ContextAction(action) => Some((
            "agui.context.update",
            serde_json::json!({
                "kind": "context",
                "phase": "update",
                "strategy": action.strategy,
                "messages_removed": action.messages_removed,
                "tokens_saved": action.tokens_saved,
                "was_applied": action.was_applied,
                "summary_generated": action.summary_generated
            }),
        )),
        NormalizedEvent::ToolCallApprovalRequired {
            run_id,
            call_index,
            tool_call_id,
            name,
            arguments_json,
            risk_reason,
        } => Some((
            "agui.tool_call.approval_required",
            serde_json::json!({
                "kind": "tool_call",
                "phase": "approval_required",
                "request_id": run_id,
                "call_index": call_index,
                "id": tool_call_id,
                "name": name,
                "arguments_json": arguments_json,
                "risk_reason": risk_reason
            }),
        )),
        NormalizedEvent::ToolCallDenied {
            run_id,
            call_index,
            tool_call_id,
            name,
            reason,
        } => Some((
            "agui.tool_call.denied",
            serde_json::json!({
                "kind": "tool_call", "phase": "denied", "request_id": run_id,
                "call_index": call_index, "id": tool_call_id, "name": name,
                "reason": reason
            }),
        )),
        NormalizedEvent::MemoryMutation {
            run_id,
            operation,
            memory_id,
            content,
            scope,
            memory_type,
        } => Some((
            "agui.memory.mutation",
            serde_json::json!({
                "kind": "memory",
                "phase": "mutation",
                "request_id": run_id,
                "operation": operation,
                "memory_id": memory_id,
                "content": content,
                "scope": scope,
                "memory_type": memory_type
            }),
        )),
        // Runtime step progress is delivered on the `runtime.*` entity bus
        // (see `to_runtime_entity_event`), not the agui surface.
        NormalizedEvent::RuntimeStep { .. } => None,
        NormalizedEvent::BudgetAlert {
            run_id,
            scope,
            scope_id,
            spent_usd,
            limit_usd,
            exceeded,
        } => Some((
            "agui.budget.alert",
            serde_json::json!({
                "kind": "budget",
                "phase": if *exceeded { "exceeded" } else { "warning" },
                "request_id": run_id,
                "scope": scope,
                "scope_id": scope_id,
                "spent_usd": spent_usd,
                "limit_usd": limit_usd
            }),
        )),
    }
}

/// Convert a [`NormalizedEvent`] into a `runtime.*` entity-bus event.
///
/// The returned event name + JSON payload are in the shape consumed by
/// `frontend/src/entities/runtime-ingest.ts` (`ingestRuntimeEvent`).
/// These are emitted alongside `agui.*` events in the SSE stream when
/// `stream_mode` is `dual` or when a `runtime` consumer is connected.
///
// The canonical transport-free AG-UI encoder + profile enricher now live in
// `super::adapters` so the embedded (non-`server`) runtime can reach them.
// Re-exported here for the SSE path and existing callers/tests.
pub use super::adapters::{enrich_agui_spec_payload, to_agui_spec_event};

pub fn to_runtime_entity_event(
    event: &NormalizedEvent,
) -> Option<(&'static str, serde_json::Value)> {
    match event {
        NormalizedEvent::RunStart { run_id, agent_id } => Some((
            "runtime.run",
            serde_json::json!({
                "type": "run_started",
                "id": run_id,
                "run_id": run_id,
                "agent_id": agent_id,
                "status": "running",
                "started_at": chrono::Utc::now().to_rfc3339(),
                "updated_at": chrono::Utc::now().to_rfc3339()
            }),
        )),
        NormalizedEvent::RunDone { run_id } | NormalizedEvent::RunDoneWithUsage { run_id, .. } => {
            let (input_tokens, output_tokens, total_tokens, cost_usd_estimate) = match event {
                NormalizedEvent::RunDoneWithUsage {
                    input_tokens,
                    output_tokens,
                    total_tokens,
                    cost_usd_estimate,
                    ..
                } => (
                    *input_tokens,
                    *output_tokens,
                    *total_tokens,
                    *cost_usd_estimate,
                ),
                _ => (None, None, None, None),
            };
            Some((
                "runtime.run",
                serde_json::json!({
                    "type": "run_finished",
                    "id": run_id,
                    "run_id": run_id,
                    "status": "completed",
                    "input_tokens": input_tokens,
                    "output_tokens": output_tokens,
                    "total_tokens": total_tokens,
                    "cost_usd_estimate": cost_usd_estimate,
                    "updated_at": chrono::Utc::now().to_rfc3339()
                }),
            ))
        }
        NormalizedEvent::Error {
            run_id,
            code,
            message,
        } => Some((
            "runtime.run",
            serde_json::json!({
                "type": "run_failed",
                "id": run_id,
                "run_id": run_id,
                "status": "failed",
                "error_code": code,
                "error_message": message,
                "updated_at": chrono::Utc::now().to_rfc3339()
            }),
        )),
        NormalizedEvent::ToolStart {
            run_id,
            call_index,
            tool_call_id,
            tool,
            input,
        } => Some((
            "runtime.tool_call",
            serde_json::json!({
                "type": "tool_call_started",
                "id": tool_call_id,
                "run_id": run_id,
                "call_index": call_index,
                "tool_call_id": tool_call_id,
                "name": tool,
                "arguments_json": input.to_string(),
                "status": "running",
                "updated_at": chrono::Utc::now().to_rfc3339()
            }),
        )),
        NormalizedEvent::ToolEnd {
            run_id,
            call_index,
            tool_call_id,
            tool,
            output,
            ok,
        } => Some((
            "runtime.tool_call",
            serde_json::json!({
                "type": if *ok { "tool_call_finished" } else { "tool_call_failed" },
                "id": tool_call_id,
                "run_id": run_id,
                "call_index": call_index,
                "tool_call_id": tool_call_id,
                "name": tool,
                "result": output.to_string(),
                "status": if *ok { "completed" } else { "failed" },
                "updated_at": chrono::Utc::now().to_rfc3339()
            }),
        )),
        NormalizedEvent::ToolCallApprovalRequired {
            run_id,
            call_index,
            tool_call_id,
            name,
            arguments_json,
            risk_reason,
        } => Some((
            "runtime.approval",
            serde_json::json!({
                "type": "approval_requested",
                "id": format!("approval:{tool_call_id}"),
                "run_id": run_id,
                "call_index": call_index,
                "tool_call_id": tool_call_id,
                "tool_name": name,
                "arguments_json": arguments_json,
                "risk_reason": risk_reason,
                "status": "pending",
                "updated_at": chrono::Utc::now().to_rfc3339()
            }),
        )),
        NormalizedEvent::ToolCallDenied {
            run_id,
            call_index,
            tool_call_id,
            name,
            reason,
        } => Some((
            "runtime.approval",
            serde_json::json!({
                "type": "approval_denied",
                "id": format!("approval:{tool_call_id}"),
                "run_id": run_id,
                "call_index": call_index,
                "tool_call_id": tool_call_id,
                "tool_name": name,
                "reason": reason,
                "status": "denied",
                "updated_at": chrono::Utc::now().to_rfc3339()
            }),
        )),
        NormalizedEvent::RuntimeStep { run_id, step, kind } => Some((
            "runtime.step",
            serde_json::json!({
                "type": format!("step_{kind}"),
                "id": format!("{run_id}-{step}"),
                "run_id": run_id,
                "step": step,
                "status": kind,
                "updated_at": chrono::Utc::now().to_rfc3339()
            }),
        )),
        NormalizedEvent::BudgetAlert {
            run_id,
            scope,
            scope_id,
            spent_usd,
            limit_usd,
            exceeded,
        } => Some((
            "runtime.budget_alert",
            serde_json::json!({
                "type": "budget_alert",
                "id": format!("budget:{scope}:{scope_id}:{run_id}"),
                "run_id": run_id,
                "scope": scope,
                "scope_id": scope_id,
                "spent_usd": spent_usd,
                "limit_usd": limit_usd,
                "exceeded": exceeded,
                "updated_at": chrono::Utc::now().to_rfc3339()
            }),
        )),
        // All other events don't produce a Runtime entity.
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::{
        AguiSpecProjector, build_agui_replay_snapshot, enrich_agui_spec_payload, to_agui_event,
        to_agui_spec_event, to_runtime_entity_event,
    };
    use crate::uar::domain::{
        context::{ContextAction, ContextStrategy},
        events::{NormalizedEvent, RagCitation, StatePatchOp},
    };
    use crate::uar::runtime::manager::StreamEvent;

    #[test]
    fn maps_skill_activation_event_with_selection_method() {
        let event = NormalizedEvent::SkillActivated {
            run_id: "run-1".to_string(),
            skill_id: "skills.weather".to_string(),
            title: "Weather Skill".to_string(),
            selection_method: "skill_service.keyword".to_string(),
        };

        let (name, payload) = to_agui_event(&event).expect("skill activation should map");
        assert_eq!(name, "agui.skill.activated");
        assert_eq!(payload["kind"], "skill");
        assert_eq!(payload["phase"], "activated");
        assert_eq!(payload["request_id"], "run-1");
        assert_eq!(payload["skill"]["id"], "skills.weather");
        assert_eq!(payload["skill"]["title"], "Weather Skill");
        assert_eq!(payload["selection_method"], "skill_service.keyword");
    }

    #[test]
    fn maps_rag_citation_with_knowledge_base_and_document_provenance() {
        let event = NormalizedEvent::RagCitations {
            run_id: "run-rag".to_string(),
            citations: vec![RagCitation {
                marker: 1,
                chunk_id: "chunk-1".to_string(),
                document_id: Some("doc-1".to_string()),
                knowledge_base_id: Some("kb-1".to_string()),
                document_name: "handbook.txt".to_string(),
                relevance_score: 0.91,
                snippet: "grounded content".to_string(),
            }],
        };

        let (name, payload) = to_agui_event(&event).expect("RAG citation should map");
        assert_eq!(name, "agui.rag_citations");
        assert_eq!(payload["request_id"], "run-rag");
        assert_eq!(payload["citations"][0]["knowledge_base_id"], "kb-1");
        assert_eq!(payload["citations"][0]["document_id"], "doc-1");
        assert_eq!(payload["citations"][0]["document_name"], "handbook.txt");
    }

    #[test]
    fn maps_runtime_step_started_to_runtime_entity() {
        let event = NormalizedEvent::RuntimeStep {
            run_id: "run-1".to_string(),
            step: 2,
            kind: "started".to_string(),
        };
        let (name, payload) = to_runtime_entity_event(&event).expect("step should map");
        assert_eq!(name, "runtime.step");
        assert_eq!(payload["type"], "step_started");
        assert_eq!(payload["id"], "run-1-2");
        assert_eq!(payload["run_id"], "run-1");
        assert_eq!(payload["step"], 2);
        // Step progress is not mirrored to the agui surface.
        assert!(to_agui_event(&event).is_none());
    }

    #[test]
    fn maps_runtime_step_finished_to_runtime_entity() {
        let event = NormalizedEvent::RuntimeStep {
            run_id: "run-9".to_string(),
            step: 5,
            kind: "finished".to_string(),
        };
        let (name, payload) = to_runtime_entity_event(&event).expect("step should map");
        assert_eq!(name, "runtime.step");
        assert_eq!(payload["type"], "step_finished");
        assert_eq!(payload["id"], "run-9-5");
    }

    #[test]
    fn maps_run_start_event_with_agent_id() {
        let event = NormalizedEvent::RunStart {
            run_id: "run-1".to_string(),
            agent_id: "orchestrator-agent".to_string(),
        };

        let (name, payload) = to_agui_event(&event).expect("run start should map");
        assert_eq!(name, "agui.stream.start");
        assert_eq!(payload["request_id"], "run-1");
        assert_eq!(payload["agent_id"], "orchestrator-agent");
    }

    #[test]
    fn maps_run_done_with_usage_includes_model() {
        let event = NormalizedEvent::RunDoneWithUsage {
            run_id: "run-1".to_string(),
            input_tokens: Some(120),
            output_tokens: Some(45),
            total_tokens: Some(165),
            cost_usd_estimate: None,
            model: Some("qwen3.7-max".to_string()),
        };

        let (name, payload) = to_agui_event(&event).expect("run done should map");
        assert_eq!(name, "agui.done");
        assert_eq!(payload["usage"]["input_tokens"], 120);
        assert_eq!(payload["usage"]["output_tokens"], 45);
        assert_eq!(payload["usage"]["total_tokens"], 165);
        assert_eq!(payload["usage"]["model"], "qwen3.7-max");
    }

    #[test]
    fn maps_context_action_event_with_strategy() {
        let event = NormalizedEvent::ContextAction(ContextAction {
            strategy: ContextStrategy::SlidingWindow,
            messages_removed: 5,
            tokens_saved: 2000,
            was_applied: true,
            summary_generated: false,
        });

        let (name, payload) = to_agui_event(&event).expect("context action should map");
        assert_eq!(name, "agui.context.update");
        assert_eq!(payload["kind"], "context");
        assert_eq!(payload["phase"], "update");
        assert_eq!(payload["strategy"], "sliding_window");
        assert_eq!(payload["messages_removed"], 5);
        assert_eq!(payload["tokens_saved"], 2000);
        assert_eq!(payload["was_applied"], true);
        assert_eq!(payload["summary_generated"], false);
    }

    #[test]
    fn maps_core_lifecycle_text_tool_state_and_error_to_agui_profile() {
        let cases = [
            (
                NormalizedEvent::RunStart {
                    run_id: "run-1".into(),
                    agent_id: "agent-1".into(),
                },
                "RUN_STARTED",
            ),
            (
                NormalizedEvent::ChatDelta {
                    run_id: "run-1".into(),
                    text_delta: "hello".into(),
                },
                "TEXT_MESSAGE_CONTENT",
            ),
            (
                NormalizedEvent::ThinkingDelta {
                    run_id: "run-1".into(),
                    text_delta: "reason".into(),
                },
                "REASONING_MESSAGE_CONTENT",
            ),
            (
                NormalizedEvent::ToolDelta {
                    run_id: "run-1".into(),
                    call_index: 0,
                    tool_call_id: "call-1".into(),
                    delta: serde_json::json!({"city": "Chi"}),
                },
                "TOOL_CALL_ARGS",
            ),
            (
                NormalizedEvent::ToolStart {
                    run_id: "run-1".into(),
                    call_index: 0,
                    tool_call_id: "call-1".into(),
                    tool: "weather".into(),
                    input: serde_json::json!({"city": "Chicago"}),
                },
                "TOOL_CALL_END",
            ),
            (
                NormalizedEvent::ToolEnd {
                    run_id: "run-1".into(),
                    call_index: 0,
                    tool_call_id: "call-1".into(),
                    tool: "weather".into(),
                    output: serde_json::json!({"temperature": 72}),
                    ok: true,
                },
                "TOOL_CALL_RESULT",
            ),
            (
                NormalizedEvent::StatePatch {
                    run_id: "run-1".into(),
                    patch: vec![StatePatchOp {
                        op: "replace".into(),
                        path: "/status".into(),
                        value: Some(serde_json::json!("ready")),
                    }],
                },
                "STATE_DELTA",
            ),
            (
                NormalizedEvent::Error {
                    run_id: "run-1".into(),
                    code: "PROVIDER_ERROR".into(),
                    message: "provider failed".into(),
                },
                "RUN_ERROR",
            ),
            (
                NormalizedEvent::RunDone {
                    run_id: "run-1".into(),
                },
                "RUN_FINISHED",
            ),
        ];

        for (index, (event, expected_type)) in cases.into_iter().enumerate() {
            let (event_type, payload) = to_agui_spec_event(&event).expect("event maps");
            assert_eq!(event_type, expected_type);
            assert_eq!(payload["type"], expected_type);
            assert_eq!(payload["profile"], "uar.agui/1");
            let enriched = enrich_agui_spec_payload(event_type, payload, "7", index as u64);
            assert_eq!(enriched["eventId"], format!("7:{index}"));
            assert!(enriched["sequence"].as_u64().is_some());
        }
    }

    #[test]
    fn replay_snapshot_matches_the_selected_cursor() {
        let history = vec![
            StreamEvent {
                id: 1,
                event: NormalizedEvent::StatePatch {
                    run_id: "run-1".into(),
                    patch: vec![StatePatchOp {
                        op: "replace".into(),
                        path: "/run".into(),
                        value: Some(serde_json::json!({ "status": "running" })),
                    }],
                },
            },
            StreamEvent {
                id: 2,
                event: NormalizedEvent::ChatDelta {
                    run_id: "run-1".into(),
                    text_delta: "hello".into(),
                },
            },
            StreamEvent {
                id: 3,
                event: NormalizedEvent::ChatDelta {
                    run_id: "run-1".into(),
                    text_delta: " world".into(),
                },
            },
        ];

        let snapshot = build_agui_replay_snapshot("run-1", &history, 2);

        assert_eq!(snapshot.cursor, 2);
        assert_eq!(
            snapshot.state.expect("state remains synchronized")["run"]["status"],
            "running"
        );
        assert_eq!(snapshot.messages.len(), 1);
        assert_eq!(snapshot.messages[0]["content"], "hello");
    }

    #[test]
    fn replay_snapshot_does_not_claim_state_after_an_invalid_patch() {
        let history = vec![StreamEvent {
            id: 1,
            event: NormalizedEvent::StatePatch {
                run_id: "run-1".into(),
                patch: vec![StatePatchOp {
                    op: "replace".into(),
                    path: "/missing".into(),
                    value: Some(serde_json::json!(true)),
                }],
            },
        }];

        assert!(
            build_agui_replay_snapshot("run-1", &history, 1)
                .state
                .is_none()
        );
    }

    #[test]
    fn replay_tool_projection_synthesizes_start_exactly_once() {
        let mut projector = AguiSpecProjector::default();
        for index in 0..8 {
            let delta = NormalizedEvent::ToolDelta {
                run_id: "run-1".into(),
                call_index: 0,
                tool_call_id: "call-1".into(),
                delta: serde_json::json!({ "chunk": index }),
            };
            assert!(projector.project(&delta).is_empty());
        }
        let end = NormalizedEvent::ToolStart {
            run_id: "run-1".into(),
            call_index: 0,
            tool_call_id: "call-1".into(),
            tool: "weather".into(),
            input: serde_json::json!({ "city": "Chicago" }),
        };

        let end_frames = projector.project(&end);

        assert_eq!(
            end_frames
                .iter()
                .map(|frame| frame.event_name)
                .collect::<Vec<_>>(),
            vec![
                "TOOL_CALL_START",
                "TOOL_CALL_ARGS",
                "TOOL_CALL_ARGS",
                "TOOL_CALL_ARGS",
                "TOOL_CALL_ARGS",
                "TOOL_CALL_ARGS",
                "TOOL_CALL_ARGS",
                "TOOL_CALL_ARGS",
                "TOOL_CALL_ARGS",
                "TOOL_CALL_END",
            ]
        );
        assert_eq!(
            end_frames
                .iter()
                .map(|frame| frame.ordinal)
                .collect::<Vec<_>>(),
            (0..10).collect::<Vec<_>>()
        );
        assert_eq!(end_frames[0].payload["toolCallName"], "weather");
        let enriched = end_frames
            .into_iter()
            .map(|frame| {
                enrich_agui_spec_payload(frame.event_name, frame.payload, "7", frame.ordinal)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            enriched
                .iter()
                .map(|payload| payload["eventId"].as_str().expect("event id"))
                .collect::<std::collections::HashSet<_>>()
                .len(),
            10
        );
        assert!(enriched.iter().all(|payload| payload["sequence"] == 112));
        let next = enrich_agui_spec_payload("TOOL_CALL_RESULT", serde_json::json!({}), "8", 0);
        assert_eq!(next["sequence"], 128);
    }
}
