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

pub(crate) const ARTIFACT_TYPE: &str = "a2ui";
const ACTIVITY_TYPE: &str = "a2ui-surface";
/// Earlier runs read when restating a conversation's surfaces.
const MAX_RESTATED_RUNS: usize = 64;
/// Restated frames use indices `1..=MAX_RESTATED_SURFACES` at the start cursor.
const MAX_RESTATED_SURFACES: usize = 48;

/// An HTTP refusal raised before any run is reserved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Rejection {
    pub status: StatusCode,
    pub code: &'static str,
}
impl Rejection {
    const INVALID: Self = Self {
        status: StatusCode::BAD_REQUEST,
        code: "invalid_a2ui_action",
    };
    const NOT_DECLARED: Self = Self {
        status: StatusCode::UNPROCESSABLE_ENTITY,
        code: "a2ui_action_not_declared",
    };
    const GRAPH_AGENT: Self = Self {
        status: StatusCode::UNPROCESSABLE_ENTITY,
        code: "a2ui_action_unsupported_for_graph_agent",
    };
    const WITH_TEXT: Self = Self {
        status: StatusCode::BAD_REQUEST,
        code: "a2ui_action_with_text",
    };
    pub(crate) const WITH_RESUME: Self = Self {
        status: StatusCode::BAD_REQUEST,
        code: "a2ui_action_with_resume",
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OpKind {
    Create,
    Delete,
    Other,
}

/// One A2UI wire message addressed to a surface, with `profile` removed.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SurfaceOp {
    surface_id: String,
    kind: OpKind,
    message: Value,
}

const SURFACE_KEYS: [(&str, OpKind); 4] = [
    ("createSurface", OpKind::Create),
    ("updateComponents", OpKind::Other),
    ("updateDataModel", OpKind::Other),
    ("deleteSurface", OpKind::Delete),
];

/// Parse newline-joined raw A2UI messages (`a2ui_output.rs` publication).
/// Lines that are not JSON objects with a surface id are skipped.
pub(crate) fn parse_batch(content: &str) -> Vec<SurfaceOp> {
    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let mut message: Value = serde_json::from_str(line).ok()?;
            let object = message.as_object_mut()?;
            object.remove("profile");
            let (surface_id, kind) = SURFACE_KEYS.iter().find_map(|(key, kind)| {
                object
                    .get(*key)?
                    .get("surfaceId")?
                    .as_str()
                    .map(|id| (id.to_owned(), *kind))
            })?;
            Some(SurfaceOp {
                surface_id,
                kind,
                message,
            })
        })
        .collect()
}

fn same_operation(left: &SurfaceOp, right: &SurfaceOp) -> bool {
    left.surface_id == right.surface_id
        && left.kind == right.kind
        && SURFACE_KEYS.iter().any(|(key, _)| {
            left.message
                .get(*key)
                .is_some_and(|payload| right.message.get(*key) == Some(payload))
        })
}

/// Accept only the exact host surface-patch representation, never nested
/// strings or arbitrary JSON-Patch paths as action declarations.
fn surface_patch(operation: &crate::uar::domain::events::StatePatchOp) -> Option<SurfaceOp> {
    let message = crate::uar::a2ui::protocol::surface_patch_message(operation)?;
    parse_batch(&message.raw.to_string()).into_iter().next()
}

/// `a2ui:` plus the first 32 hex digits of SHA-256(thread, NUL, surface).
pub(crate) fn surface_message_id(thread_id: &str, surface_id: &str) -> String {
    let digest = Sha256::digest(format!("{thread_id}\0{surface_id}").as_bytes());
    let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    format!("a2ui:{}", &hex[..32])
}

/// Validate the original host stamp; never grant authority to unstamped replay.
fn artifact_generations(run_id: &str, cursor: u64, operations: &[SurfaceOp], metadata: &Value) -> Value {
    let mut generations = serde_json::Map::new();
    for operation in operations {
        let expected = crate::uar::a2ui::protocol::surface_generation(run_id, cursor, &operation.surface_id);
        if metadata.get("surface_generations").and_then(|map| map.get(&operation.surface_id))
            .and_then(Value::as_str) == Some(expected.as_str()) {
            generations.insert(operation.surface_id.clone(), json!(expected));
        }
    }
    Value::Object(generations)
}

fn snapshot(thread_id: &str, entry: &SurfaceEntry) -> Value {
    json!({"type":"ACTIVITY_SNAPSHOT","messageId":surface_message_id(thread_id,&entry.surface_id),
        "activityType":ACTIVITY_TYPE,"content":{"a2ui_operations":entry.operations,"surface_generation":entry.generation},"replace":true})
}

#[derive(Debug, Clone)]
struct SurfaceEntry {
    generation: String,
    surface_id: String,
    /// Native run that most recently published to this surface.
    source_run_id: String,
    operations: Vec<Value>,
    deleted: bool,
}

/// A conversation's live surfaces in first-created order, each with its
/// cumulative operation list.
#[derive(Debug, Clone, Default)]
pub(crate) struct SurfaceRecord {
    surfaces: Vec<SurfaceEntry>,
    #[cfg(test)]
    next_test_cursor: u64,
    incomplete: Option<&'static str>,
    /// Only the latest bounded patch suffix is needed to collapse paired observations.
    pending_live: Vec<(String, u64, SurfaceOp)>,
}

impl SurfaceRecord {
    /// Append `ops`; returns the touched surface ids in first-touched order.
    /// `createSurface` starts the surface's list over; a delete is kept in the
    /// list until the caller prunes.
    fn apply(&mut self, source_run_id: &str, ops: Vec<SurfaceOp>, generations: &Value) -> Vec<String> {
        let mut touched: Vec<String> = Vec::new();
        for op in ops {
            let index = self
                .surfaces
                .iter()
                .position(|entry| entry.surface_id == op.surface_id)
                .unwrap_or_else(|| {
                    self.surfaces.push(SurfaceEntry {
                        generation: String::new(),
                        surface_id: op.surface_id.clone(),
                        source_run_id: source_run_id.to_owned(),
                        operations: Vec::new(),
                        deleted: false,
                    });
                    self.surfaces.len() - 1
                });
            let entry = &mut self.surfaces[index];
            if op.kind == OpKind::Create {
                entry.operations.clear();
                entry.deleted = false;
            }
            if op.kind == OpKind::Delete {
                entry.deleted = true;
            }
            // Missing historical stamps remain renderable, but cannot gain action authority.
            entry.generation = generations.get(&op.surface_id).and_then(Value::as_str)
                .unwrap_or_default().to_owned();
            entry.source_run_id = source_run_id.to_owned();
            entry.operations.push(op.message);
            if !touched.contains(&op.surface_id) {
                touched.push(op.surface_id);
            }
        }
        touched
    }

    /// Project one published batch: a snapshot per touched surface, then drop
    /// the surfaces the batch deleted (their final snapshot includes the delete).
    pub(crate) fn publish_artifact(
        &mut self,
        thread_id: &str,
        source_run_id: &str,
        content: &str,
        cursor: u64,
        metadata: &Value,
    ) -> Vec<Value> {
        let operations = parse_batch(content);
        let generations = artifact_generations(source_run_id, cursor, &operations, metadata);
        let oldest = cursor.saturating_sub(crate::uar::runtime::manager::EVENT_HISTORY_LIMIT as u64);
        self.pending_live.retain(|(run, id, _)| run == source_run_id && *id > oldest);
        let suffix = self.pending_live.len().checked_sub(operations.len());
        if suffix.is_some_and(|start| self.pending_live[start..].iter().zip(&operations)
            .all(|((_, _, left), right)| same_operation(left, right))) {
            // Live patches already applied this exact suffix. Undo only those
            // observations before applying their artifact's publication stamp.
            for operation in operations.iter().rev() {
                if let Some(entry) = self.surfaces.iter_mut().find(|entry| entry.surface_id == operation.surface_id) {
                    entry.operations.pop();
                }
            }
        }
        self.pending_live.clear();
        let touched = self.apply(source_run_id, operations, &generations);
        let frames = touched
            .iter()
            .filter_map(|id| self.surfaces.iter().find(|entry| &entry.surface_id == id))
            .map(|entry| snapshot(thread_id, entry))
            .collect();
        self.surfaces.retain(|entry| !entry.deleted);
        frames
    }

    /// Standalone patches also change live AG-UI authority. Keep the same
    /// bounded suffix used to collapse a following matching native artifact.
    pub(crate) fn publish_patch(&mut self, thread_id: &str, run_id: &str, cursor: u64,
        patch: &[crate::uar::domain::events::StatePatchOp]) -> Vec<Value> {
        let operations: Vec<_> = patch.iter().filter_map(surface_patch).collect();
        let mut generations = serde_json::Map::new();
        for operation in &operations {
            let generation = crate::uar::a2ui::protocol::surface_generation(run_id, cursor, &operation.surface_id);
            let path = crate::uar::a2ui::protocol::surface_generation_path(&operation.surface_id);
            if patch.iter().any(|op| op.op == "add" && op.path == path
                && op.value.as_ref().and_then(Value::as_str) == Some(generation.as_str())) {
                generations.insert(operation.surface_id.clone(), json!(generation));
            }
        }
        let oldest = cursor.saturating_sub(crate::uar::runtime::manager::EVENT_HISTORY_LIMIT as u64);
        self.pending_live.retain(|(run, id, _)| run == run_id && *id > oldest);
        self.pending_live.extend(operations.iter().cloned().map(|operation| (run_id.to_owned(), cursor, operation)));
        let touched = self.apply(run_id, operations, &Value::Object(generations));
        let frames = touched.iter().filter_map(|id| self.surfaces.iter().find(|entry| &entry.surface_id == id))
            .map(|entry| snapshot(thread_id, entry)).collect();
        self.surfaces.retain(|entry| !entry.deleted);
        frames
    }

    /// Fold the events of one binding's range `start_cursor < id <= source_cursor`.
    pub(crate) fn fold_events(
        &mut self,
        native_run_id: &str,
        start_cursor: u64,
        source_cursor: u64,
        events: &[StreamEvent],
    ) {
        // Live delivery and durable replay use the exact same ordered fold,
        // including split history ranges and matching patch/artifact suffixes.
        for event in events.iter().filter(|event| event.id > start_cursor && event.id <= source_cursor) {
            match &event.event {
                NormalizedEvent::StatePatch { patch, .. } => {
                    let _ = self.publish_patch("", native_run_id, event.id, patch);
                }
                NormalizedEvent::ArtifactDisplay { artifact, .. } if artifact.artifact_type == ARTIFACT_TYPE => {
                    let _ = self.publish_artifact("", native_run_id, &artifact.content, event.id, &artifact.metadata);
                }
                _ => {}
            }
        }
    }

    #[cfg(test)]
    fn publish(&mut self, thread_id: &str, run_id: &str, content: &str) -> Vec<Value> {
        self.next_test_cursor += 1;
        let mut event = NormalizedEvent::ArtifactDisplay {
            run_id: run_id.into(),
            artifact: crate::uar::domain::events::ArtifactPayload {
                artifact_id: "test".into(), artifact_type: ARTIFACT_TYPE.into(),
                title: "test".into(), content: content.into(), language: None, metadata: json!({}),
            },
        };
        crate::uar::a2ui::protocol::stamp_surface_publication(run_id, self.next_test_cursor, &mut event);
        let NormalizedEvent::ArtifactDisplay { artifact, .. } = event else { unreachable!() };
        self.publish_artifact(thread_id, run_id, content, self.next_test_cursor, &artifact.metadata)
    }

    pub(crate) fn mark_incomplete(&mut self, reason: &'static str) {
        self.incomplete = Some(reason);
    }

    pub(crate) fn touches_state_patch(
        &self,
        operation: &crate::uar::domain::events::StatePatchOp,
    ) -> bool {
        surface_patch(operation).is_some_and(|operation| {
            self.surfaces
                .iter()
                .any(|entry| entry.surface_id == operation.surface_id)
        })
    }
    pub(crate) fn touches_live_surface(&self, content: &str) -> bool {
        parse_batch(content).iter().any(|operation| {
            self.surfaces
                .iter()
                .any(|entry| entry.surface_id == operation.surface_id)
        })
    }
    pub(crate) fn authority_status(&self) -> &'static str {
        self.incomplete
            .unwrap_or(if self.surfaces.iter().any(|entry| entry.generation.is_empty()) {
                "unavailable"
            } else if self.surfaces.len() > MAX_RESTATED_SURFACES {
                "truncated"
            } else {
                "complete"
            })
    }
    pub(crate) fn authority(&self) -> Value {
        json!({"status":self.authority_status(),"authoritative":self.authority_status() == "complete",
            "max_prior_runs":MAX_RESTATED_RUNS,"max_surfaces":MAX_RESTATED_SURFACES,
            "recovery_hint":"Open a new conversation and request a fresh surface; reconcile an unconfirmed interaction before a new intent",
            "surfaces":self.surfaces.iter().take(MAX_RESTATED_SURFACES).map(|entry|json!({"surface_id":entry.surface_id,
                "surface_generation":entry.generation,"source_run_id":entry.source_run_id})).collect::<Vec<_>>()})
    }
    pub(crate) fn declared(
        &self,
        surface: &str,
        component: &str,
        name: &str,
        generation: Option<&str>,
    ) -> Result<(String, String), Rejection> {
        if self.authority_status() != "complete" {
            return Err(Rejection {
                status: StatusCode::CONFLICT,
                code: "a2ui_surface_authority_unavailable",
            });
        }
        self.surfaces
            .iter()
            .find(|entry| {
                entry.surface_id == surface
                    && !entry.generation.is_empty()
                    && generation.is_none_or(|generation| generation == entry.generation)
                    && declares_button(&entry.operations, component, name)
            })
            .map(|entry| (entry.source_run_id.clone(), entry.generation.clone()))
            .ok_or(Rejection::NOT_DECLARED)
    }

    /// One snapshot per live surface, first-created order, capped.
    pub(crate) fn restatement(&self, thread_id: &str) -> Vec<Value> {
        self.surfaces
            .iter()
            .take(MAX_RESTATED_SURFACES)
            .map(|entry| snapshot(thread_id, entry))
            .collect()
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
pub(crate) async fn thread_record(
    state: &RunApiState,
    user: &crate::uar::security::claims::UserContext,
    row: &AguiRunBinding,
) -> Option<SurfaceRecord> {
    let db = state.manager.persistence.as_ref()?;
    let bindings = db
        .list_agui_runs(&row.owner, &row.agent_id, &row.thread_id)
        .await
        .ok()?;
    let mut record = SurfaceRecord::default();
    if bindings
        .iter()
        .filter(|binding| binding.ordinal < row.ordinal)
        .count()
        > MAX_RESTATED_RUNS
    {
        record.mark_incomplete("expired");
    }
    for binding in earlier_bindings(bindings, row.ordinal) {
        let source = state.manager.get_run_for_context(user, &binding.native_run_id).await;
        let native_agent = super::agui::admitted_agent(&binding).ok();
        if binding.validate().is_err() || binding.owner != row.owner
            || binding.agent_id != row.agent_id || binding.thread_id != row.thread_id
            || binding.session_id != row.session_id || binding.workspace_id != row.workspace_id
            || source.as_ref().is_none_or(|source|
                Some(source.agent_id.as_str()) != native_agent
                    || source.conversation_id.as_deref() != Some(binding.session_id.as_str()))
        {
            record.mark_incomplete("unavailable");
            continue;
        }
        let Some(terminal) = &binding.terminal else {
            record.mark_incomplete("unavailable");
            continue;
        };
        let Some(events) = state
            .manager
            .history_since(&binding.native_run_id, Some(binding.start_cursor))
            .await
        else {
            tracing::warn!(native_run_id = %binding.native_run_id, "A2UI restatement skipped a run without readable history");
            record.mark_incomplete("unavailable");
            continue;
        };
        if terminal.source_cursor > binding.start_cursor
            && events
                .last()
                .is_none_or(|event| event.id < terminal.source_cursor)
        {
            record.mark_incomplete("unavailable");
        }
        if events
            .first()
            .is_some_and(|event| event.id > binding.start_cursor.saturating_add(1))
        {
            record.mark_incomplete("unavailable");
        }
        let mut expected = binding.start_cursor.saturating_add(1);
        for event in events
            .iter()
            .filter(|event| event.id > binding.start_cursor && event.id <= terminal.source_cursor)
        {
            if event.id != expected {
                record.mark_incomplete("unavailable");
            }
            expected = event.id.saturating_add(1);
        }
        record.fold_events(
            &binding.native_run_id,
            binding.start_cursor,
            terminal.source_cursor,
            &events,
        );
    }
    Some(record)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct A2uiAction {
    user_action: UserAction,
    #[serde(default, alias = "interaction_id")]
    pub(crate) interaction_id: Option<String>,
    #[serde(default, alias = "surface_generation")]
    surface_generation: Option<String>,
    #[serde(default)]
    data_model: Option<Value>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserAction {
    name: String,
    surface_id: String,
    source_component_id: String,
    #[serde(default)]
    context: Option<Value>,
    #[serde(default)]
    timestamp: Option<Value>,
    #[serde(default)]
    catalog_id: Option<Value>,
    #[serde(default)]
    user_message: Option<Value>,
}

/// Read top-level `forwardedProps.a2uiAction`. Unknown fields are tolerated.
pub(crate) fn parse_action(
    forwarded_props: Option<&Value>,
) -> Result<Option<A2uiAction>, Rejection> {
    let Some(raw) = forwarded_props.and_then(|props| props.get("a2uiAction")) else {
        return Ok(None);
    };
    let action: A2uiAction = serde_json::from_value(raw.clone()).map_err(|_| Rejection::INVALID)?;
    let user = &action.user_action;
    if action
        .interaction_id
        .as_ref()
        .is_some_and(|id| !crate::uar::persistence::a2ui_actions::valid_id(id))
        || (action.interaction_id.is_some()
            && action.surface_generation.as_ref().is_none_or(|generation| {
                !crate::uar::persistence::a2ui_actions::valid_id(generation)
            }))
    {
        return Err(Rejection::INVALID);
    }
    if [&user.name, &user.surface_id, &user.source_component_id]
        .iter()
        .any(|field| field.trim().is_empty())
    {
        return Err(Rejection::INVALID);
    }
    Ok(Some(action))
}

/// The newest definition of `component_id` on the surface must be a `Button`
/// whose `action.event.name` is `name`.
fn declares_button(operations: &[Value], component_id: &str, name: &str) -> bool {
    let mut newest: Option<&Value> = None;
    for operation in operations {
        let Some(components) = operation
            .pointer("/updateComponents/components")
            .and_then(Value::as_array)
        else {
            continue;
        };
        for component in components
            .iter()
            .filter(|c| c.get("id").and_then(Value::as_str) == Some(component_id))
        {
            newest = Some(component);
        }
    }
    newest.is_some_and(|c| {
        c["component"] == "Button"
            && c.pointer("/action/event/name").and_then(Value::as_str) == Some(name)
    })
}

impl A2uiAction {
    pub(crate) fn declared_identity(
        &self,
        record: &SurfaceRecord,
    ) -> Result<(String, String), Rejection> {
        let user = &self.user_action;
        record.declared(
            &user.surface_id,
            &user.source_component_id,
            &user.name,
            self.surface_generation.as_deref(),
        )
    }
    pub(crate) fn semantic_digest(&self) -> String {
        let user = &self.user_action;
        crate::uar::persistence::a2ui_actions::digest(&json!({"surface":user.surface_id,
            "generation":self.surface_generation,"component":user.source_component_id,"name":user.name,
            "context":user.context,"data_model":self.data_model,"catalog":user.catalog_id,"user_message":user.user_message}))
    }
    pub(crate) fn receipt(
        &self,
        row: &AguiRunBinding,
        source: &str,
        generation: &str,
    ) -> Option<crate::uar::persistence::a2ui_actions::A2uiActionReceipt> {
        let user = &self.user_action;
        Some(crate::uar::persistence::a2ui_actions::A2uiActionReceipt {
            owner: row.owner.clone(),
            agent_id: row.agent_id.clone(),
            thread_id: row.session_id.clone(),
            interaction_id: self.interaction_id.clone()?,
            source_run_id: source.to_owned(),
            surface_id: user.surface_id.clone(),
            surface_generation: generation.to_owned(),
            component_id: user.source_component_id.clone(),
            action_name: user.name.clone(),
            semantic_digest: self.semantic_digest(),
            continuation_run_id: row.native_run_id.clone(),
            wire_binding: Some(row.clone()),
        })
    }

    /// The run input, shaped like UAR's REST action path.
    pub(crate) fn run_input(&self, source_run_id: &str) -> String {
        let user = &self.user_action;
        json!({"type":"a2ui.user_action","sourceRunId":source_run_id,"interaction":{
            "surfaceId":user.surface_id,"name":user.name,"sourceComponentId":user.source_component_id,
            "timestamp":user.timestamp,"context":user.context,"catalogId":user.catalog_id,
            "userMessage":user.user_message,"a2uiClientDataModel":self.data_model}}).to_string()
    }
}

/// Orchestrator-graph conversations publish surfaces in child runs this slice
/// does not follow, so they refuse actions.
pub(crate) fn refuse_for_graph(uses_agent_graph: bool) -> Result<(), Rejection> {
    if uses_agent_graph {
        Err(Rejection::GRAPH_AGENT)
    } else {
        Ok(())
    }
}

/// The messages a run is built from. An action request carries no new user
/// text: activity messages are dropped and the rest is history.
pub(crate) fn request_messages(
    messages: &[Value],
    is_action: bool,
) -> Result<Cow<'_, [Value]>, Rejection> {
    if !is_action {
        return Ok(Cow::Borrowed(messages));
    }
    let role = |message: &Value| {
        message
            .get("role")
            .and_then(Value::as_str)
            .map(str::to_owned)
    };
    let kept: Vec<Value> = messages
        .iter()
        .filter(|m| role(m).as_deref() != Some("activity"))
        .cloned()
        .collect();
    if kept.last().and_then(role).as_deref() == Some("user") {
        return Err(Rejection::WITH_TEXT);
    }
    Ok(Cow::Owned(kept))
}

#[cfg(test)]
mod logical_action_tests {
    use super::*;

    fn surface() -> String {
        [json!({"createSurface":{"surfaceId":"ticket","catalogId":"catalog"}}),
            json!({"updateComponents":{"surfaceId":"ticket","components":[{"id":"confirm","component":"Button","action":{"event":{"name":"confirm"}}}]}})]
            .iter().map(Value::to_string).collect::<Vec<_>>().join("\n")
    }

    #[test]
    fn latest_exact_button_delete_and_generation_fence_actions() {
        let mut record = SurfaceRecord::default();
        record.publish("thread", "source", &surface());
        let (_, generation) = record
            .declared("ticket", "confirm", "confirm", None)
            .unwrap_or_else(|error| panic!("{error:?}"));
        assert!(
            record
                .declared("ticket", "other", "confirm", Some(&generation))
                .is_err()
        );
        assert!(
            record
                .declared("ticket", "confirm", "foreign", Some(&generation))
                .is_err()
        );
        assert!(
            record
                .declared("ticket", "confirm", "confirm", Some("stale"))
                .is_err()
        );
        record.publish("thread","source",&json!({"updateComponents":{"surfaceId":"ticket","components":[{"id":"confirm","component":"Text","text":"confirm"}]}}).to_string());
        assert!(
            record
                .declared("ticket", "confirm", "confirm", Some(&generation))
                .is_err()
        );
        record.publish("thread", "source", &surface());
        let (_, next_generation) = record
            .declared("ticket", "confirm", "confirm", None)
            .unwrap_or_else(|error| panic!("{error:?}"));
        assert_ne!(generation, next_generation);
        record.publish(
            "thread",
            "source",
            &json!({"deleteSurface":{"surfaceId":"ticket"}}).to_string(),
        );
        assert!(
            record
                .declared("ticket", "confirm", "confirm", Some(&next_generation))
                .is_err()
        );
    }

    #[test]
    fn surface_limit_and_missing_history_are_not_empty_authority() {
        let mut record = SurfaceRecord::default();
        for index in 0..49 {
            record.publish(
                "thread",
                "source",
                &json!({"createSurface":{"surfaceId":format!("s-{index}")}}).to_string(),
            );
        }
        assert_eq!(record.restatement("thread").len(), 48);
        assert_eq!(record.authority_status(), "truncated");
        assert_eq!(record.authority()["authoritative"], false);
        record.mark_incomplete("unavailable");
        assert_eq!(record.authority_status(), "unavailable");
        assert!(record.declared("s-0", "button", "action", None).is_err());
    }

    #[test]
    fn protected_actions_require_intent_and_generation_but_legacy_is_explicit() {
        let legacy = json!({"a2uiAction":{"userAction":{"name":"confirm","surfaceId":"ticket","sourceComponentId":"confirm"}}});
        assert!(
            parse_action(Some(&legacy))
                .unwrap_or_else(|error| panic!("{error:?}"))
                .unwrap_or_else(|| panic!("missing action"))
                .interaction_id
                .is_none()
        );
        let mut protected = legacy.clone();
        protected["a2uiAction"]["interaction_id"] = json!("intent-1");
        assert!(parse_action(Some(&protected)).is_err());
        protected["a2uiAction"]["surface_generation"] = json!("generation-1");
        let first = parse_action(Some(&protected))
            .unwrap_or_else(|error| panic!("{error:?}"))
            .unwrap_or_else(|| panic!("missing action"));
        protected["a2uiAction"]["userAction"]["timestamp"] = json!("transport retry time");
        assert_eq!(
            first.semantic_digest(),
            parse_action(Some(&protected))
                .unwrap_or_else(|error| panic!("{error:?}"))
                .unwrap_or_else(|| panic!("missing action"))
                .semantic_digest()
        );
        protected["a2uiAction"]["userAction"]["context"] = json!({"note":"changed"});
        assert_ne!(
            first.semantic_digest(),
            parse_action(Some(&protected))
                .unwrap_or_else(|error| panic!("{error:?}"))
                .unwrap_or_else(|| panic!("missing action"))
                .semantic_digest()
        );
    }
    #[test]
    fn trusted_patch_and_matching_artifact_fold_once_and_standalone_delete_remains() {
        use crate::uar::domain::events::{ArtifactPayload, StatePatchOp};
        let create = json!({"version":"v0.9","createSurface":{"surfaceId":"ticket","catalogId":"urn:uar:a2ui:catalog:1"}});
        let patch = StatePatchOp {
            op: "add".into(),
            path: "/a2ui/surfaces/ticket".into(),
            value: Some(create["createSurface"].clone()),
        };
        let mut events = vec![
            StreamEvent {
                id: 1,
                event: NormalizedEvent::StatePatch {
                    run_id: "source".into(),
                    patch: vec![patch],
                },
            },
            StreamEvent {
                id: 2,
                event: NormalizedEvent::ArtifactDisplay {
                    run_id: "source".into(),
                    artifact: ArtifactPayload {
                        artifact_id: "published".into(),
                        artifact_type: "a2ui".into(),
                        title: "surface".into(),
                        content: create.to_string(),
                        language: None,
                        metadata: json!({}),
                    },
                },
            },
        ];
        for event in &mut events {
            crate::uar::a2ui::protocol::stamp_surface_publication("source", event.id, &mut event.event);
        }
        let mut record = SurfaceRecord::default();
        record.fold_events("source", 0, 2, &events);
        assert_eq!(
            record.authority()["surfaces"][0]["surface_generation"],
            crate::uar::a2ui::protocol::surface_generation("source", 2, "ticket")
        );
        let deletion = StreamEvent {
            id: 3,
            event: NormalizedEvent::StatePatch {
                run_id: "source".into(),
                patch: vec![StatePatchOp {
                    op: "remove".into(),
                    path: "/a2ui/surfaces/ticket".into(),
                    value: None,
                }],
            },
        };
        record.fold_events("source", 2, 3, &[deletion]);
        assert_eq!(record.authority()["surfaces"], json!([]));
        let unrelated = StatePatchOp {
            op: "add".into(),
            path: "/untrusted/components".into(),
            value: Some(json!({"name":"confirm"})),
        };
        assert!(surface_patch(&unrelated).is_none());
    }
    #[test]
    fn same_ids_data_model_cross_run_delete_recreate_advance_publication_identity() {
        let mut record = SurfaceRecord::default();
        record.publish("thread", "source", &surface());
        let first = record.declared("ticket", "confirm", "confirm", None).unwrap().1;
        // Same component/action IDs with changed content are a new publication.
        record.publish("thread", "source", &json!({"updateComponents":{"surfaceId":"ticket","components":[
            {"id":"confirm","component":"Button","action":{"event":{"name":"confirm"}}},
            {"id":"label","component":"Text","text":"changed"}
        ]}}).to_string());
        let second = record.declared("ticket", "confirm", "confirm", None).unwrap().1;
        assert_ne!(first, second);
        assert!(record.declared("ticket", "confirm", "confirm", Some(&first)).is_err());
        record.publish("thread", "source", &json!({"updateDataModel":{"surfaceId":"ticket","path":"/answer","value":"new"}}).to_string());
        let third = record.declared("ticket", "confirm", "confirm", None).unwrap().1;
        assert_ne!(second, third);
        record.publish("thread", "next-source", &json!({"updateDataModel":{"surfaceId":"ticket","path":"/answer","value":"cross-run"}}).to_string());
        let (source, fourth) = record.declared("ticket", "confirm", "confirm", None).unwrap();
        assert_eq!(source, "next-source");
        assert_ne!(third, fourth);
        record.publish("thread", "next-source", &json!({"deleteSurface":{"surfaceId":"ticket"}}).to_string());
        assert!(record.declared("ticket", "confirm", "confirm", Some(&fourth)).is_err());
        record.publish("thread", "next-source", &surface());
        assert_ne!(fourth, record.declared("ticket", "confirm", "confirm", None).unwrap().1);
    }

    #[test]
    fn unstamped_native_history_never_gains_generation_authority() {
        let mut record = SurfaceRecord::default();
        let frames = record.publish_artifact("thread", "source", &surface(), 7, &json!({}));
        assert_eq!(frames[0]["content"]["surface_generation"], "");
        assert_eq!(record.authority_status(), "unavailable");
        assert_eq!(record.authority()["authoritative"], false);
        assert!(record.declared("ticket", "confirm", "confirm", None).is_err());
        let forged = json!({"surface_generations":{"ticket":"producer-claimed-latest"}});
        record.publish_artifact("thread", "source", &surface(), 8, &forged);
        assert!(record.declared("ticket", "confirm", "confirm", None).is_err());
    }

    #[test]
    fn standalone_patch_and_paired_artifact_live_replay_and_native_tokens_agree() {
        use crate::uar::domain::events::{ArtifactPayload, StatePatchOp};
        let content = [
            json!({"version":"v0.9","createSurface":{"surfaceId":"ticket","catalogId":"urn:uar:a2ui:catalog:1"}}),
            json!({"version":"v0.9","updateComponents":{"surfaceId":"ticket","components":[
                {"id":"root","component":"Column","children":["confirm"]},
                {"id":"confirm","component":"Button","child":"label","action":{"event":{"name":"confirm"}}},
                {"id":"label","component":"Text","text":"confirm"}
            ]}}),
        ];
        let mut events: Vec<StreamEvent> = content.iter().enumerate().map(|(index, wire)| {
            let message = crate::uar::a2ui::protocol::parse_message(wire.clone()).unwrap();
            StreamEvent { id: index as u64 + 1, event: NormalizedEvent::StatePatch {
                run_id: "source".into(), patch: vec![crate::uar::a2ui::realtime::surface_message_to_state_patch(
                    &message.surface_id, message.kind, message.payload)],
            }}
        }).collect();
        events.push(StreamEvent { id: 3, event: NormalizedEvent::ArtifactDisplay {
            run_id: "source".into(), artifact: ArtifactPayload {
                artifact_id: "native".into(), artifact_type: "a2ui".into(), title: "surface".into(),
                content: content.iter().map(Value::to_string).collect::<Vec<_>>().join("\n"),
                language: None, metadata: json!({"surface_generations":{"ticket":"untrusted"}}),
            },
        }});
        for event in &mut events {
            crate::uar::a2ui::protocol::stamp_surface_publication("source", event.id, &mut event.event);
        }
        let mut live = SurfaceRecord::default();
        let mut final_frames = Vec::new();
        for event in &events {
            final_frames = match &event.event {
                NormalizedEvent::StatePatch { patch, .. } => live.publish_patch("thread", "source", event.id, patch),
                NormalizedEvent::ArtifactDisplay { artifact, .. } => live.publish_artifact("thread", "source", &artifact.content, event.id, &artifact.metadata),
                _ => unreachable!(),
            };
        }
        let mut replay = SurfaceRecord::default();
        // A history read may split exactly between native patches and artifact.
        replay.fold_events("source", 0, 2, &events);
        replay.fold_events("source", 2, 3, &events);
        assert_eq!(live.authority(), replay.authority());
        assert_eq!(final_frames, replay.restatement("thread"));
        assert_eq!(final_frames[0]["content"]["a2ui_operations"].as_array().unwrap().len(), 2);
        let NormalizedEvent::ArtifactDisplay { artifact, .. } = &events[2].event else { unreachable!() };
        let native_generation = artifact.metadata["surface_generations"]["ticket"].as_str().unwrap();
        assert_eq!(live.declared("ticket", "confirm", "confirm", Some(native_generation)).unwrap().1, native_generation);
        assert_eq!(final_frames[0]["content"]["surface_generation"], native_generation);

        let mut update = StreamEvent { id: 4, event: NormalizedEvent::StatePatch {
            run_id: "source".into(), patch: vec![StatePatchOp {
                op: "replace".into(), path: "/a2ui/surfaces/ticket/dataModel".into(),
                value: Some(json!({"path":"/answer","value":"new"})),
            }],
        }};
        crate::uar::a2ui::protocol::stamp_surface_publication("source", 4, &mut update.event);
        let NormalizedEvent::StatePatch { patch, .. } = &update.event else { unreachable!() };
        let update_frames = live.publish_patch("thread", "source", 4, patch);
        replay.fold_events("source", 3, 4, &[update]);
        assert_eq!(live.authority(), replay.authority());
        assert_eq!(update_frames, replay.restatement("thread"));
        assert!(live.declared("ticket", "confirm", "confirm", Some(native_generation)).is_err());
    }

}
