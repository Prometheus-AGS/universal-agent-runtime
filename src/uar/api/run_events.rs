//! Owner-scoped, inert snapshots of the existing bounded public run history.

use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::{routes::RunApiState, sse::to_agui_event};
use crate::uar::security::claims::UserContext;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RunEventQuery {
    #[serde(default)]
    after: u64,
}

/// The payload is the existing public SSE projection, not internal authority.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PublicRunEvent {
    event_id: u64,
    event_name: &'static str,
    data: Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RunEventGap {
    RetentionGap,
    CursorAhead,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RunEventSnapshot {
    version: u32,
    run_id: String,
    after: u64,
    cursor: u64,
    retention: &'static str,
    first_available_event_id: Option<u64>,
    gap_reason: Option<RunEventGap>,
    events: Vec<PublicRunEvent>,
}

pub(crate) async fn snapshot(
    State(state): State<Arc<RunApiState>>,
    Extension(user): Extension<UserContext>,
    Path(run_id): Path<String>,
    Query(query): Query<RunEventQuery>,
) -> Result<Json<RunEventSnapshot>, StatusCode> {
    if state
        .manager
        .get_run_for_context(&user, &run_id)
        .await
        .is_none()
    {
        return Err(StatusCode::NOT_FOUND);
    }
    let history = state
        .manager
        .history_since(&run_id, None)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;
    let first_available_event_id = history.first().map(|event| event.id);
    let cursor = history.last().map_or(0, |event| event.id);
    let mut gap_reason = (query.after > cursor).then_some(RunEventGap::CursorAhead);
    let mut previous = query.after;
    let mut events = Vec::new();
    for event in history.iter().filter(|event| event.id > query.after) {
        if event.id != previous.saturating_add(1) {
            gap_reason = Some(RunEventGap::RetentionGap);
        }
        previous = event.id;
        if let Some((event_name, data)) = to_agui_event(&event.event) {
            events.push(PublicRunEvent {
                event_id: event.id,
                event_name,
                data,
            });
        }
    }
    Ok(Json(RunEventSnapshot {
        version: 1,
        run_id,
        after: query.after,
        cursor,
        retention: "process-local-bounded",
        first_available_event_id,
        gap_reason,
        events,
    }))
}

pub(crate) fn snapshot_schema() -> Value {
    json!({
        "type": "object", "additionalProperties": false,
        "required": ["version", "runId", "after", "cursor", "retention", "firstAvailableEventId", "gapReason", "events"],
        "properties": {
            "version": {"type": "integer", "enum": [1]},
            "runId": {"type": "string"},
            "after": {"type": "integer", "minimum": 0},
            "cursor": {"type": "integer", "minimum": 0},
            "retention": {"type": "string", "enum": ["process-local-bounded"]},
            "firstAvailableEventId": {"type": ["integer", "null"], "minimum": 1},
            "gapReason": {"type": ["string", "null"], "enum": ["retention-gap", "cursor-ahead", null]},
            "events": {
                "type": "array", "items": {
                    "type": "object", "additionalProperties": false,
                    "required": ["eventId", "eventName", "data"],
                    "properties": {
                        "eventId": {"type": "integer", "minimum": 1},
                        "eventName": {"type": "string", "description": "Existing public AG-UI SSE event name"},
                        "data": {"description": "Existing public payload for eventName; no internal admission or credential objects"}
                    }
                }
            }
        }
    })
}
