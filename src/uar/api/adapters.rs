//! Transport-free `NormalizedEvent` → wire-event adapters.
//!
//! This module is intentionally **ungated** (no `#[cfg(feature = "server")]`)
//! and imports nothing from a transport layer (no axum, no SSE). Every AG-UI /
//! A2UI encoder that turns a runtime event into a wire payload lives here, so it
//! is reachable by BOTH the HTTP/SSE server path AND the embedded, in-process
//! runtime (mobile/desktop hosts that never enable the `server` feature).
//!
//! Matches the AG-UI protocol's own architecture: its reference SDKs ship a
//! standalone `EventEncoder` (`@ag-ui/encoder`, `ag-ui-protocol`) that is
//! separate from the HTTP handler. `api/sse.rs` re-exports these functions and
//! only adds the Axum SSE framing on top. See `docs/adr/0012`.

use crate::uar::domain::events::NormalizedEvent;
use serde_json::{Value, json};

pub fn to_ag_ui(event: &NormalizedEvent) -> Value {
    match event {
        NormalizedEvent::ChatDelta { run_id, text_delta } => json!({
            "type": "token.delta",
            "id": run_id,
            "payload": { "delta": text_delta }
        }),
        NormalizedEvent::ToolStart {
            run_id,
            tool_call_id,
            tool,
            input,
            call_index,
        } => json!({
            "type": "tool.call",
            "id": run_id,
            "payload": {
                "call_id": tool_call_id,
                "tool": tool,
                "args": input,
                "call_index": call_index
            }
        }),
        NormalizedEvent::Artifact { run_id, artifact } => json!({
            "type": "ui.render",
            "id": run_id,
            "payload": {
                "schema": "a2ui.v1",
                "component": "artifact",
                "props": artifact
            }
        }),
        // Default fallthrough to raw event
        other => serde_json::to_value(other).unwrap_or(json!({"error": "serialization_failed"})),
    }
}

/// Map a `NormalizedEvent` to an **official AG-UI protocol** event
/// (`RUN_STARTED`, `TEXT_MESSAGE_CONTENT`, `TOOL_CALL_*`, `STATE_DELTA`, …)
/// instead of UAR's invented `agui.*` names (CH-21, fable §7 R6). This is what
/// CopilotKit / Microsoft Agent Framework / Oracle A2UI clients expect on the
/// wire. Payloads use official field names; UAR-only signals are named
/// `CUSTOM` extensions. This is intentionally separate from the deprecated
/// legacy `agui.*` mapping (`to_agui_event`).
///
/// Transport-free: this returns `(event_name, json)` and knows nothing about
/// SSE/WebSocket. `api/sse.rs::build_sse_response` frames the result for HTTP;
/// the embedded runtime encodes it directly. Returns `None` for events that
/// don't produce a wire frame.
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
                "language": artifact.language, "metadata": artifact.metadata,
                "sourceRunId": run_id
            }),
        ),
        NormalizedEvent::ArtifactInputRequest { run_id, artifact } => custom(
            "uar.artifact.input_required",
            Some(run_id),
            serde_json::json!({
                "artifactId": artifact.artifact_id,
                "artifactType": artifact.artifact_type,
                "title": artifact.title, "content": artifact.content,
                "metadata": artifact.metadata,
                "sourceRunId": run_id
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
    // Frames synthesized from one retained runtime event share its ordering
    // sequence. Their stable event ids carry the per-frame ordinal, so an
    // arbitrarily long tool-argument stream cannot overlap the next event's
    // sequence range.
    let sequence = source_id.parse::<u64>().unwrap_or(0).saturating_mul(16);
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
