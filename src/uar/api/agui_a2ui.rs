//! A2UI surfaces on the dedicated AG-UI route.
//!
//! Outbound: published A2UI batches become one cumulative `ACTIVITY_SNAPSHOT`
//! per surface, and earlier runs' live surfaces are restated at stream start.
//! Inbound: a `forwardedProps.a2uiAction` click is checked against the
//! conversation's live surfaces and becomes the run input. Everything here is
//! pure except `thread_record`; the trusted host still owns admission.
use std::borrow::Cow;

use axum::http::StatusCode;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::routes::RunApiState;
use crate::uar::{
    domain::events::NormalizedEvent, persistence::agui_runs::AguiRunBinding,
    runtime::manager::StreamEvent,
};

pub(super) const ARTIFACT_TYPE: &str = "a2ui";
const ACTIVITY_TYPE: &str = "a2ui-surface";
/// Earlier runs read when restating a conversation's surfaces.
const MAX_RESTATED_RUNS: usize = 64;
/// Restated frames use indices `1..=MAX_RESTATED_SURFACES` at the start cursor.
const MAX_RESTATED_SURFACES: usize = 48;

/// An HTTP refusal raised before any run is reserved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Rejection {
    pub status: StatusCode,
    pub code: &'static str,
}
impl Rejection {
    const INVALID: Self = Self { status: StatusCode::BAD_REQUEST, code: "invalid_a2ui_action" };
    const NOT_DECLARED: Self = Self { status: StatusCode::UNPROCESSABLE_ENTITY, code: "a2ui_action_not_declared" };
    const GRAPH_AGENT: Self = Self { status: StatusCode::UNPROCESSABLE_ENTITY, code: "a2ui_action_unsupported_for_graph_agent" };
    const WITH_TEXT: Self = Self { status: StatusCode::BAD_REQUEST, code: "a2ui_action_with_text" };
    pub(super) const WITH_RESUME: Self = Self { status: StatusCode::BAD_REQUEST, code: "a2ui_action_with_resume" };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OpKind { Create, Delete, Other }

/// One A2UI wire message addressed to a surface, with `profile` removed.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct SurfaceOp { surface_id: String, kind: OpKind, message: Value }

const SURFACE_KEYS: [(&str, OpKind); 4] = [
    ("createSurface", OpKind::Create), ("updateComponents", OpKind::Other),
    ("updateDataModel", OpKind::Other), ("deleteSurface", OpKind::Delete),
];

/// Parse newline-joined raw A2UI messages (`a2ui_output.rs` publication).
/// Lines that are not JSON objects with a surface id are skipped.
pub(super) fn parse_batch(content: &str) -> Vec<SurfaceOp> {
    content.lines().filter(|line| !line.trim().is_empty()).filter_map(|line| {
        let mut message: Value = serde_json::from_str(line).ok()?;
        let object = message.as_object_mut()?;
        object.remove("profile");
        let (surface_id, kind) = SURFACE_KEYS.iter().find_map(|(key, kind)| {
            object.get(*key)?.get("surfaceId")?.as_str().map(|id| (id.to_owned(), *kind))
        })?;
        Some(SurfaceOp { surface_id, kind, message })
    }).collect()
}

/// `a2ui:` plus the first 32 hex digits of SHA-256(thread, NUL, surface).
pub(super) fn surface_message_id(thread_id: &str, surface_id: &str) -> String {
    let digest = Sha256::digest(format!("{thread_id}\0{surface_id}").as_bytes());
    let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    format!("a2ui:{}", &hex[..32])
}

fn snapshot(thread_id: &str, entry: &SurfaceEntry) -> Value {
    json!({"type":"ACTIVITY_SNAPSHOT","messageId":surface_message_id(thread_id,&entry.surface_id),
        "activityType":ACTIVITY_TYPE,"content":{"a2ui_operations":entry.operations},"replace":true})
}

#[derive(Debug, Clone)]
struct SurfaceEntry {
    surface_id: String,
    /// Native run that most recently published to this surface.
    source_run_id: String,
    operations: Vec<Value>,
    deleted: bool,
}

/// A conversation's live surfaces in first-created order, each with its
/// cumulative operation list.
#[derive(Debug, Clone, Default)]
pub(super) struct SurfaceRecord { surfaces: Vec<SurfaceEntry> }

impl SurfaceRecord {
    /// Append `ops`; returns the touched surface ids in first-touched order.
    /// `createSurface` starts the surface's list over; a delete is kept in the
    /// list until the caller prunes.
    fn apply(&mut self, source_run_id: &str, ops: Vec<SurfaceOp>) -> Vec<String> {
        let mut touched: Vec<String> = Vec::new();
        for op in ops {
            let index = self.surfaces.iter().position(|entry| entry.surface_id == op.surface_id).unwrap_or_else(|| {
                self.surfaces.push(SurfaceEntry { surface_id: op.surface_id.clone(), source_run_id: source_run_id.to_owned(), operations: Vec::new(), deleted: false });
                self.surfaces.len() - 1
            });
            let entry = &mut self.surfaces[index];
            if op.kind == OpKind::Create { entry.operations.clear(); entry.deleted = false; }
            if op.kind == OpKind::Delete { entry.deleted = true; }
            entry.source_run_id = source_run_id.to_owned();
            entry.operations.push(op.message);
            if !touched.contains(&op.surface_id) { touched.push(op.surface_id); }
        }
        touched
    }

    /// Project one published batch: a snapshot per touched surface, then drop
    /// the surfaces the batch deleted (their final snapshot includes the delete).
    pub(super) fn publish(&mut self, thread_id: &str, source_run_id: &str, content: &str) -> Vec<Value> {
        let touched = self.apply(source_run_id, parse_batch(content));
        let frames = touched.iter()
            .filter_map(|id| self.surfaces.iter().find(|entry| &entry.surface_id == id))
            .map(|entry| snapshot(thread_id, entry)).collect();
        self.surfaces.retain(|entry| !entry.deleted);
        frames
    }

    /// Fold an earlier run's batch into the record without emitting.
    fn fold(&mut self, source_run_id: &str, content: &str) {
        self.apply(source_run_id, parse_batch(content));
        self.surfaces.retain(|entry| !entry.deleted);
    }

    /// Fold the events of one binding's range `start_cursor < id <= source_cursor`.
    pub(super) fn fold_events(&mut self, native_run_id: &str, start_cursor: u64, source_cursor: u64, events: &[StreamEvent]) {
        for event in events.iter().filter(|event| event.id > start_cursor && event.id <= source_cursor) {
            if let NormalizedEvent::ArtifactDisplay { artifact, .. } = &event.event
                && artifact.artifact_type == ARTIFACT_TYPE
            {
                self.fold(native_run_id, &artifact.content);
            }
        }
    }

    /// One snapshot per live surface, first-created order, capped.
    pub(super) fn restatement(&self, thread_id: &str) -> Vec<Value> {
        self.surfaces.iter().take(MAX_RESTATED_SURFACES).map(|entry| snapshot(thread_id, entry)).collect()
    }
}

/// Bindings before `ordinal`, the newest `MAX_RESTATED_RUNS`, oldest first.
fn earlier_bindings(mut all: Vec<AguiRunBinding>, ordinal: u64) -> Vec<AguiRunBinding> {
    all.retain(|binding| binding.ordinal < ordinal);
    all.sort_by_key(|binding| binding.ordinal);
    let skip = all.len().saturating_sub(MAX_RESTATED_RUNS);
    all.into_iter().skip(skip).collect()
}

/// The surfaces published by the conversation's runs before `row`. `None` when
/// the binding list cannot be read; a binding whose history cannot be read is
/// skipped.
pub(super) async fn thread_record(state: &RunApiState, row: &AguiRunBinding) -> Option<SurfaceRecord> {
    let db = state.manager.persistence.as_ref()?;
    let bindings = db.list_agui_runs(&row.owner, &row.agent_id, &row.thread_id).await.ok()?;
    let mut record = SurfaceRecord::default();
    for binding in earlier_bindings(bindings, row.ordinal) {
        let Some(terminal) = &binding.terminal else { continue };
        let Some(events) = state.manager.history_since(&binding.native_run_id, Some(binding.start_cursor)).await else {
            tracing::warn!(native_run_id = %binding.native_run_id, "A2UI restatement skipped a run without readable history");
            continue;
        };
        record.fold_events(&binding.native_run_id, binding.start_cursor, terminal.source_cursor, &events);
    }
    Some(record)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct A2uiAction {
    user_action: UserAction,
    #[serde(default)] data_model: Option<Value>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserAction {
    name: String,
    surface_id: String,
    source_component_id: String,
    #[serde(default)] context: Option<Value>,
    #[serde(default)] timestamp: Option<Value>,
    #[serde(default)] catalog_id: Option<Value>,
    #[serde(default)] user_message: Option<Value>,
}

/// Read top-level `forwardedProps.a2uiAction`. Unknown fields are tolerated.
pub(super) fn parse_action(forwarded_props: Option<&Value>) -> Result<Option<A2uiAction>, Rejection> {
    let Some(raw) = forwarded_props.and_then(|props| props.get("a2uiAction")) else { return Ok(None); };
    let action: A2uiAction = serde_json::from_value(raw.clone()).map_err(|_| Rejection::INVALID)?;
    let user = &action.user_action;
    if [&user.name, &user.surface_id, &user.source_component_id].iter().any(|field| field.trim().is_empty()) {
        return Err(Rejection::INVALID);
    }
    Ok(Some(action))
}

/// The newest definition of `component_id` on the surface must be a `Button`
/// whose `action.event.name` is `name`.
fn declares_button(operations: &[Value], component_id: &str, name: &str) -> bool {
    let mut newest: Option<&Value> = None;
    for operation in operations {
        let Some(components) = operation.pointer("/updateComponents/components").and_then(Value::as_array) else { continue };
        for component in components.iter().filter(|c| c.get("id").and_then(Value::as_str) == Some(component_id)) {
            newest = Some(component);
        }
    }
    newest.is_some_and(|c| c["component"] == "Button" && c.pointer("/action/event/name").and_then(Value::as_str) == Some(name))
}

impl A2uiAction {
    /// Native run id of the binding that published the surface, when a live
    /// surface of `record` declares this exact button and action name.
    pub(super) fn declared_source(&self, record: &SurfaceRecord) -> Result<String, Rejection> {
        let user = &self.user_action;
        record.surfaces.iter()
            .find(|entry| entry.surface_id == user.surface_id
                && declares_button(&entry.operations, &user.source_component_id, &user.name))
            .map(|entry| entry.source_run_id.clone()).ok_or(Rejection::NOT_DECLARED)
    }

    /// The run input, shaped like UAR's REST action path.
    pub(super) fn run_input(&self, source_run_id: &str) -> String {
        let user = &self.user_action;
        json!({"type":"a2ui.user_action","sourceRunId":source_run_id,"interaction":{
            "surfaceId":user.surface_id,"name":user.name,"sourceComponentId":user.source_component_id,
            "timestamp":user.timestamp,"context":user.context,"catalogId":user.catalog_id,
            "userMessage":user.user_message,"a2uiClientDataModel":self.data_model}}).to_string()
    }
}

/// Orchestrator-graph conversations publish surfaces in child runs this slice
/// does not follow, so they refuse actions.
pub(super) fn refuse_for_graph(uses_agent_graph: bool) -> Result<(), Rejection> {
    if uses_agent_graph { Err(Rejection::GRAPH_AGENT) } else { Ok(()) }
}

/// The messages a run is built from. An action request carries no new user
/// text: activity messages are dropped and the rest is history.
pub(super) fn request_messages(messages: &[Value], is_action: bool) -> Result<Cow<'_, [Value]>, Rejection> {
    if !is_action { return Ok(Cow::Borrowed(messages)); }
    let role = |message: &Value| message.get("role").and_then(Value::as_str).map(str::to_owned);
    let kept: Vec<Value> = messages.iter().filter(|m| role(m).as_deref() != Some("activity")).cloned().collect();
    if kept.last().and_then(role).as_deref() == Some("user") { return Err(Rejection::WITH_TEXT); }
    Ok(Cow::Owned(kept))
}
