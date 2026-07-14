use crate::uar::domain::events::NormalizedEvent;
use crate::uar::runtime::manager::StreamEvent;
use axum::response::sse::{Event, Sse};
use futures::{Stream, StreamExt};
use std::convert::Infallible;
use std::time::Duration;

pub fn build_sse_response<S>(
    stream: S,
    agui_spec: bool,
) -> Sse<impl Stream<Item = Result<Event, Infallible>> + Send>
where
    S: Stream<Item = StreamEvent> + Send + 'static,
{
    let stream = stream.filter_map(move |event| async move {
        let (event_name, payload) = if agui_spec {
            let (event_name, payload) = to_agui_spec_event(&event.event)?;
            (
                event_name,
                enrich_agui_spec_payload(event_name, payload, &event.id.to_string(), 8),
            )
        } else {
            to_agui_event(&event.event)?
        };

        let json = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
        let sse_event = Event::default()
            .event(event_name)
            .id(event.id.to_string())
            .data(json);

        Some(Ok(sse_event))
    });

    Sse::new(stream)
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
/// Returns `None` for events that don't produce a `Runtime*` entity.
/// Map a `NormalizedEvent` to an **official AG-UI protocol** event
/// (`RUN_STARTED`, `TEXT_MESSAGE_CONTENT`, `TOOL_CALL_*`, `STATE_DELTA`, …)
/// instead of UAR's invented `agui.*` names (CH-21, fable §7 R6). This is what
/// CopilotKit / Microsoft Agent Framework / Oracle A2UI clients expect on the
/// wire. Payloads use official field names; UAR-only signals are named
/// `CUSTOM` extensions. This is intentionally separate from the deprecated
/// legacy `agui.*` mapping.
#[must_use]
pub fn to_agui_spec_event(event: &NormalizedEvent) -> Option<(&'static str, serde_json::Value)> {
    const PROFILE: &str = "uar.agui/1";
    let custom = |name: &'static str, run_id: Option<&str>, value: serde_json::Value| {
        (
            "CUSTOM",
            serde_json::json!({
                "type": "CUSTOM",
                "profile": PROFILE,
                "name": name,
                "value": value,
                "runId": run_id,
                "threadId": run_id,
            }),
        )
    };

    let mapped = match event {
        NormalizedEvent::RunStart { run_id, agent_id } => (
            "RUN_STARTED",
            serde_json::json!({
                "type": "RUN_STARTED", "profile": PROFILE,
                "threadId": run_id, "runId": run_id,
                "input": { "agentId": agent_id }
            }),
        ),
        NormalizedEvent::ChatDelta { run_id, text_delta } => (
            "TEXT_MESSAGE_CONTENT",
            serde_json::json!({
                "type": "TEXT_MESSAGE_CONTENT", "profile": PROFILE,
                "messageId": format!("{run_id}:assistant"), "delta": text_delta,
                "threadId": run_id, "runId": run_id
            }),
        ),
        NormalizedEvent::ThinkingDelta { run_id, text_delta }
        | NormalizedEvent::ReasoningDelta { run_id, text_delta } => (
            "REASONING_MESSAGE_CONTENT",
            serde_json::json!({
                "type": "REASONING_MESSAGE_CONTENT", "profile": PROFILE,
                "messageId": format!("{run_id}:reasoning"), "delta": text_delta,
                "threadId": run_id, "runId": run_id
            }),
        ),
        NormalizedEvent::Citation { run_id, sources } => custom(
            "uar.citation.added",
            Some(run_id),
            serde_json::json!({ "citation": sources }),
        ),
        NormalizedEvent::RagCitations { run_id, citations } => custom(
            "uar.rag_citations",
            Some(run_id),
            serde_json::json!({ "citations": citations }),
        ),
        NormalizedEvent::MemoryRecall { run_id, items } => custom(
            "uar.memory.recall",
            Some(run_id),
            serde_json::json!({ "items": items, "count": items.len() }),
        ),
        NormalizedEvent::SkillActivated {
            run_id,
            skill_id,
            title,
            selection_method,
        } => custom(
            "uar.skill.activated",
            Some(run_id),
            serde_json::json!({
                "skill": { "id": skill_id, "title": title },
                "selectionMethod": selection_method
            }),
        ),
        NormalizedEvent::ToolDelta {
            run_id,
            tool_call_id,
            delta,
            ..
        } => (
            "TOOL_CALL_ARGS",
            serde_json::json!({
                "type": "TOOL_CALL_ARGS", "profile": PROFILE,
                "toolCallId": tool_call_id, "delta": delta.to_string(),
                "threadId": run_id, "runId": run_id
            }),
        ),
        NormalizedEvent::ToolStart {
            run_id,
            tool_call_id,
            tool,
            input,
            ..
        } => (
            "TOOL_CALL_END",
            serde_json::json!({
                "type": "TOOL_CALL_END", "profile": PROFILE,
                "toolCallId": tool_call_id, "toolCallName": tool,
                "arguments": input, "threadId": run_id, "runId": run_id
            }),
        ),
        NormalizedEvent::ToolEnd {
            run_id,
            tool_call_id,
            output,
            ..
        } => (
            "TOOL_CALL_RESULT",
            serde_json::json!({
                "type": "TOOL_CALL_RESULT", "profile": PROFILE,
                "messageId": format!("{run_id}:tool:{tool_call_id}"),
                "toolCallId": tool_call_id, "content": output.to_string(),
                "role": "tool", "threadId": run_id, "runId": run_id
            }),
        ),
        NormalizedEvent::Artifact { run_id, artifact }
        | NormalizedEvent::ArtifactDisplay { run_id, artifact } => custom(
            "uar.artifact.available",
            Some(run_id),
            serde_json::json!({
                "artifactId": artifact.artifact_id,
                "artifactType": artifact.artifact_type,
                "title": artifact.title, "content": artifact.content,
                "language": artifact.language, "metadata": artifact.metadata
            }),
        ),
        NormalizedEvent::ArtifactInputRequest { run_id, artifact } => custom(
            "uar.artifact.input_required",
            Some(run_id),
            serde_json::json!({
                "artifactId": artifact.artifact_id,
                "artifactType": artifact.artifact_type,
                "title": artifact.title, "content": artifact.content,
                "metadata": artifact.metadata
            }),
        ),
        NormalizedEvent::Error {
            run_id,
            code,
            message,
        } => (
            "RUN_ERROR",
            serde_json::json!({
                "type": "RUN_ERROR", "profile": PROFILE,
                "threadId": run_id, "runId": run_id,
                "code": code, "message": message
            }),
        ),
        NormalizedEvent::RunDone { run_id } => (
            "RUN_FINISHED",
            serde_json::json!({
                "type": "RUN_FINISHED", "profile": PROFILE,
                "threadId": run_id, "runId": run_id
            }),
        ),
        NormalizedEvent::Cancelled { run_id } => (
            "RUN_ERROR",
            serde_json::json!({
                "type": "RUN_ERROR", "profile": PROFILE,
                "threadId": run_id, "runId": run_id,
                "code": "CANCELLED", "message": "Run cancelled"
            }),
        ),
        NormalizedEvent::SycophancyFlagged {
            run_id,
            sycophancy_score,
            has_critical,
            correction_mandatory,
            classifications,
        } => custom(
            "uar.quality.sycophancy_flagged",
            Some(run_id),
            serde_json::json!({
                "score": sycophancy_score, "hasCritical": has_critical,
                "correctionMandatory": correction_mandatory,
                "classifications": classifications
            }),
        ),
        NormalizedEvent::SycophancyCorrected {
            run_id,
            corrected_text,
        } => custom(
            "uar.quality.sycophancy_corrected",
            Some(run_id),
            serde_json::json!({ "correctedText": corrected_text }),
        ),
        NormalizedEvent::GuardrailFlagged {
            run_id,
            category,
            reason,
        } => custom(
            "uar.guardrail.flagged",
            run_id.as_deref(),
            serde_json::json!({ "category": category, "reason": reason }),
        ),
        NormalizedEvent::RuntimeStep { run_id, step, kind } => {
            let event_type = if kind == "started" {
                "STEP_STARTED"
            } else {
                "STEP_FINISHED"
            };
            (
                event_type,
                serde_json::json!({
                    "type": event_type, "profile": PROFILE,
                    "stepName": format!("step-{step}"),
                    "threadId": run_id, "runId": run_id
                }),
            )
        }
        NormalizedEvent::StatePatch { run_id, patch } => (
            "STATE_DELTA",
            serde_json::json!({
                "type": "STATE_DELTA", "profile": PROFILE,
                "delta": patch, "threadId": run_id, "runId": run_id
            }),
        ),
        NormalizedEvent::ContextAction(action) => custom(
            "uar.context.updated",
            None,
            serde_json::json!({
                "strategy": action.strategy,
                "messagesRemoved": action.messages_removed,
                "tokensSaved": action.tokens_saved,
                "wasApplied": action.was_applied,
                "summaryGenerated": action.summary_generated
            }),
        ),
        NormalizedEvent::BudgetAlert {
            run_id,
            scope,
            scope_id,
            spent_usd,
            limit_usd,
            exceeded,
        } => custom(
            "uar.budget.alert",
            Some(run_id),
            serde_json::json!({
                "scope": scope, "scopeId": scope_id, "spentUsd": spent_usd,
                "limitUsd": limit_usd, "exceeded": exceeded
            }),
        ),
        NormalizedEvent::MemoryMutation {
            run_id,
            operation,
            memory_id,
            content,
            scope,
            memory_type,
        } => custom(
            "uar.memory.mutation",
            Some(run_id),
            serde_json::json!({
                "operation": operation, "memoryId": memory_id, "content": content,
                "scope": scope, "memoryType": memory_type
            }),
        ),
        NormalizedEvent::ToolCallApprovalRequired {
            run_id,
            tool_call_id,
            name,
            arguments_json,
            risk_reason,
            ..
        } => custom(
            "uar.tool.approval_required",
            Some(run_id),
            serde_json::json!({
                "toolCallId": tool_call_id, "name": name,
                "arguments": arguments_json, "riskReason": risk_reason
            }),
        ),
        NormalizedEvent::ToolCallDenied {
            run_id,
            tool_call_id,
            name,
            reason,
            ..
        } => custom(
            "uar.tool.denied",
            Some(run_id),
            serde_json::json!({
                "toolCallId": tool_call_id, "name": name, "reason": reason
            }),
        ),
        NormalizedEvent::RunDoneWithUsage {
            run_id,
            input_tokens,
            output_tokens,
            total_tokens,
            cost_usd_estimate,
            model,
        } => (
            "RUN_FINISHED",
            serde_json::json!({
                "type": "RUN_FINISHED", "profile": PROFILE,
                "threadId": run_id, "runId": run_id,
                "result": { "usage": {
                    "inputTokens": input_tokens, "outputTokens": output_tokens,
                    "totalTokens": total_tokens, "costUsdEstimate": cost_usd_estimate,
                    "model": model
                }}
            }),
        ),
    };
    Some(mapped)
}

/// Add the stable UAR profile metadata shared by live and replayed AG-UI frames.
#[must_use]
pub fn enrich_agui_spec_payload(
    event_type: &str,
    mut payload: serde_json::Value,
    source_id: &str,
    ordinal: u64,
) -> serde_json::Value {
    let sequence = source_id.parse::<u64>().unwrap_or(0).saturating_mul(16) + ordinal;
    if let Some(object) = payload.as_object_mut() {
        object.insert("type".to_string(), serde_json::json!(event_type));
        object.insert("profile".to_string(), serde_json::json!("uar.agui/1"));
        object.insert(
            "eventId".to_string(),
            serde_json::json!(format!("{source_id}:{ordinal}")),
        );
        object.insert("sequence".to_string(), serde_json::json!(sequence));
    }
    payload
}

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
        enrich_agui_spec_payload, to_agui_event, to_agui_spec_event, to_runtime_entity_event,
    };
    use crate::uar::domain::{
        context::{ContextAction, ContextStrategy},
        events::{NormalizedEvent, StatePatchOp},
    };

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
}
