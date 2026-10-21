//! Schedule mutations are host tools, subject to normal effect approval.
use std::sync::{Arc, Weak};
use serde_json::{Value, json};
use crate::uar::{runtime::{native_skill::{NativeExecutionContext, NativeSkill, NativeSkillRegistry},
    schedules::ScheduleController}, tools::descriptor::{ApprovalClass, Exposure, ToolAssemblyError, ToolEffect, ToolSource}};

const NAMES: [&str; 7] = ["schedule_create", "schedule_list", "schedule_get", "schedule_update", "schedule_pause", "schedule_resume", "schedule_cancel"];
/// The registry must not keep the host controller alive after the host has
/// closed its lifecycle scope. Tool execution upgrades this command facade for
/// the duration of one admitted call.
struct ScheduleTool { controller: Weak<ScheduleController>, name: &'static str }

impl ScheduleTool {
    fn controller(&self) -> anyhow::Result<Arc<ScheduleController>> {
        self.controller
            .upgrade()
            .ok_or_else(|| anyhow::anyhow!("schedule_unavailable"))
    }
}

#[async_trait::async_trait]
impl NativeSkill for ScheduleTool {
    fn name(&self) -> &str { self.name }
    fn description(&self) -> &str {
        match self.name {
            "schedule_create" => "Create a durable one-shot, daily or weekly schedule for an owned logical agent. Local calendar gaps are skipped and repeated times use the earlier instant. Busy runs coalesce to latest pending.",
            "schedule_list" => "List your durable schedules, pending occurrences, active guard and skipped history in a workspace.",
            "schedule_get" => "Inspect a durable schedule and its current revision. Prompt and owner authorization are private.",
            "schedule_update" => "Replace schedule intent using expectedRevision. Cancels old pending intent while any admitted predecessor retains its overlap guard.",
            "schedule_pause" => "Pause new schedule admission using expectedRevision. Retains pending intent and any active run.",
            "schedule_resume" => "Resume schedule admission using expectedRevision. Backlog coalesces to the latest due occurrence.",
            _ => "Cancel future schedule intent using expectedRevision. Retains the tombstone and any admitted predecessor until safe resolution.",
        }
    }
    fn parameters_schema(&self) -> Value {
        let text = json!({"type":"string","minLength":1,"maxLength":256});
        let recurrence = json!({"oneOf":[
            {"type":"object","properties":{"kind":{"const":"once"},"at":{"type":"string","format":"date-time"}},"required":["kind","at"],"additionalProperties":false},
            {"type":"object","properties":{"kind":{"const":"daily"},"time":{"type":"string","pattern":"^[0-2][0-9]:[0-5][0-9]$"},"timezone":text},"required":["kind","time","timezone"],"additionalProperties":false},
            {"type":"object","properties":{"kind":{"const":"weekly"},"time":{"type":"string","pattern":"^[0-2][0-9]:[0-5][0-9]$"},"timezone":text,"weekdays":{"type":"array","items":{"type":"integer","minimum":1,"maximum":7},"minItems":1,"maxItems":7,"uniqueItems":true}},"required":["kind","time","timezone","weekdays"],"additionalProperties":false}
        ]});
        let schedule = json!({"type":"object","properties":{"instanceId":text,"recurrence":recurrence,"prompt":{"type":"string","minLength":1,"maxLength":262144}},"required":["instanceId","recurrence","prompt"],"additionalProperties":false});
        let mut properties = serde_json::Map::from_iter([("workspaceId".to_owned(), text.clone())]);
        let mut required = vec!["workspaceId"];
        if self.name != "schedule_create" && self.name != "schedule_list" {
            properties.insert("id".into(), text); required.push("id");
        }
        if self.name == "schedule_create" || self.name == "schedule_update" {
            properties.insert("schedule".into(), schedule); required.push("schedule");
        }
        if !matches!(self.name, "schedule_create" | "schedule_list" | "schedule_get") {
            properties.insert("expectedRevision".into(), json!({"type":"integer","minimum":1})); required.push("expectedRevision");
        }
        json!({"type":"object","properties":properties,"required":required,"additionalProperties":false})
    }
    fn effect(&self) -> ToolEffect { if matches!(self.name, "schedule_list" | "schedule_get") { ToolEffect::ReadOnly } else { ToolEffect::ExternalMutation } }
    fn approval_class(&self) -> ApprovalClass { if matches!(self.name, "schedule_list" | "schedule_get") { ApprovalClass::NotRequired } else { ApprovalClass::Required } }
    fn exposure(&self) -> Exposure { Exposure::Deferred }
    fn source(&self) -> ToolSource { ToolSource::BuiltIn }
    fn admission_owner(&self) -> crate::uar::persistence::tool_admission::AdmissionOwner {
        crate::uar::persistence::tool_admission::AdmissionOwner::UarRuntime
    }
    async fn execute(&self, _: Value) -> anyhow::Result<Value> { anyhow::bail!("schedule_denied") }
    async fn execute_with_context(&self, args: Value, context: &NativeExecutionContext) -> anyhow::Result<Value> {
        let owner = context.verified_owner.as_ref().ok_or_else(|| anyhow::anyhow!("schedule_denied"))?;
        let workspace = args["workspaceId"].as_str().ok_or_else(|| anyhow::anyhow!("schedule_invalid"))?;
        let id = args["id"].as_str().unwrap_or_default();
        let controller = self.controller()?;
        let result = match self.name {
            "schedule_list" => return Ok(serde_json::to_value(controller.list(owner, workspace).await.map_err(safe)?)?),
            "schedule_get" => controller.get(owner, workspace, id).await,
            "schedule_create" => controller.create(owner, workspace, serde_json::from_value(args["schedule"].clone())?).await,
            "schedule_update" => controller.revise(owner, workspace, id, revision(&args)?, serde_json::from_value(args["schedule"].clone())?).await,
            _ => controller.control(owner, workspace, id, revision(&args)?, self.name.trim_start_matches("schedule_")).await,
        };
        Ok(serde_json::to_value(result.map_err(safe)?)?)
    }
}

fn safe(error: crate::uar::runtime::schedules::ScheduleError) -> anyhow::Error { anyhow::anyhow!(error.code()) }
fn revision(args: &Value) -> anyhow::Result<u64> { args["expectedRevision"].as_u64().ok_or_else(|| anyhow::anyhow!("schedule_invalid")) }

pub async fn register(registry: &NativeSkillRegistry, controller: Arc<ScheduleController>) -> Result<(), ToolAssemblyError> {
    let controller = Arc::downgrade(&controller);
    for name in NAMES { registry.register(ScheduleTool { controller: controller.clone(), name }).await?; }
    Ok(())
}
