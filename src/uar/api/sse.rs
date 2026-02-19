use crate::uar::domain::events::NormalizedEvent;
use crate::uar::runtime::manager::StreamEvent;
use axum::response::sse::{Event, Sse};
use futures::{Stream, StreamExt};
use std::convert::Infallible;
use std::time::Duration;

pub fn build_sse_response<S>(stream: S) -> Sse<impl Stream<Item = Result<Event, Infallible>> + Send>
where
    S: Stream<Item = StreamEvent> + Send + 'static,
{
    let stream = stream.filter_map(|event| async move {
        let (event_name, payload) = to_agui_event(&event.event)?;

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
        NormalizedEvent::RunStart { run_id, .. } => Some((
            "agui.stream.start",
            serde_json::json!({
                "kind": "stream",
                "phase": "start",
                "request_id": run_id
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
        NormalizedEvent::MemoryRecall { run_id, items } => {
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
        NormalizedEvent::Artifact { .. } => None,
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
    }
}

#[cfg(test)]
mod tests {
    use super::to_agui_event;
    use crate::uar::domain::{
        context::{ContextAction, ContextStrategy},
        events::NormalizedEvent,
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
}
