//! Owner opt-in and inbox tools use the ordinary host approval boundary.
use std::sync::{Arc, Weak};
use serde_json::{Value, json};
use crate::uar::{runtime::{native_skill::{NativeExecutionContext, NativeSkill, NativeSkillRegistry},
    followups::FollowupController}, tools::descriptor::{ApprovalClass, Exposure, ToolAssemblyError, ToolEffect, ToolSource}};

const NAMES: [&str; 9] = ["followup_create", "followup_list", "followup_get", "followup_update", "followup_enable", "followup_disable", "followup_revoke", "followup_inbox", "followup_acknowledge"];
/// The registry retains only a command facade. The trusted host owns the
/// controller and can close it without a registry/controller reference cycle.
struct FollowupTool { controller: Weak<FollowupController>, name: &'static str }
impl FollowupTool {
    fn read_only(&self) -> bool { matches!(self.name, "followup_list" | "followup_get" | "followup_inbox") }
    fn controller(&self) -> anyhow::Result<Arc<FollowupController>> {
        self.controller
            .upgrade()
            .ok_or_else(|| anyhow::anyhow!("followup_unavailable"))
    }
}
#[async_trait::async_trait]
impl NativeSkill for FollowupTool {
    fn name(&self) -> &str { self.name }
    fn description(&self) -> &str {
        match self.name {
            "followup_create" => "Create an owner follow-up rule for durable logical-agent event metadata. Disabled unless explicitly enabled. Delivery goes only to the authorized durable conversation inbox.",
            "followup_list" => "List your follow-up rules and durable admission, suppression and quiet-hour deferral decisions.",
            "followup_get" => "Read a follow-up rule and its revision without disclosing protected source content or credentials.",
            "followup_update" => "Revise follow-up consent, event selection or quiet hours using expectedRevision. Source and destination remain fixed; queued decisions recheck current policy.",
            "followup_enable" => "Opt in to future follow-ups using expectedRevision and current source and destination authority.",
            "followup_disable" => "Disable follow-ups and suppress queued decisions using expectedRevision.",
            "followup_revoke" => "Permanently revoke a follow-up rule and suppress queued decisions using expectedRevision.",
            "followup_inbox" => "Read currently authorized follow-up metadata in your durable conversation inbox.",
            _ => "Idempotently acknowledge a currently authorized conversation inbox item. This records acknowledgement, not proof that a human read it.",
        }
    }
    fn parameters_schema(&self) -> Value {
        let text = json!({"type":"string","minLength":1,"maxLength":256});
        let quiet = json!({"type":["object","null"],"properties":{"start":{"type":"string","pattern":"^[0-2][0-9]:[0-5][0-9]$"},"end":{"type":"string","pattern":"^[0-2][0-9]:[0-5][0-9]$"},"timezone":text},"required":["start","end","timezone"],"additionalProperties":false});
        let rule = json!({"type":"object","properties":{"sourceInstanceId":text,"destinationInstanceId":text,"conversationId":text,
            "eventKinds":{"type":"array","items":{"type":"string","minLength":1,"maxLength":128,"pattern":"^[A-Za-z0-9_]+$"},"minItems":1,"maxItems":32,"uniqueItems":true},
            "enabled":{"type":"boolean","default":false},"quietHours":quiet},
            "required":["sourceInstanceId","destinationInstanceId","conversationId","eventKinds"],"additionalProperties":false});
        let mut properties = serde_json::Map::from_iter([("workspaceId".to_owned(), text.clone())]);
        let mut required = vec!["workspaceId"];
        if matches!(self.name, "followup_inbox" | "followup_acknowledge") {
            properties.insert("conversationId".into(), text.clone()); required.push("conversationId");
            if self.name == "followup_acknowledge" { properties.insert("itemId".into(), text.clone()); required.push("itemId"); }
        } else {
            if !matches!(self.name, "followup_create" | "followup_list") { properties.insert("id".into(), text); required.push("id"); }
            if matches!(self.name, "followup_create" | "followup_update") { properties.insert("rule".into(), rule); required.push("rule"); }
            if !matches!(self.name, "followup_create" | "followup_list" | "followup_get") {
                properties.insert("expectedRevision".into(), json!({"type":"integer","minimum":1})); required.push("expectedRevision");
            }
        }
        json!({"type":"object","properties":properties,"required":required,"additionalProperties":false})
    }
    fn effect(&self) -> ToolEffect { if self.read_only() { ToolEffect::ReadOnly } else { ToolEffect::ExternalMutation } }
    fn approval_class(&self) -> ApprovalClass { if self.read_only() { ApprovalClass::NotRequired } else { ApprovalClass::Required } }
    fn exposure(&self) -> Exposure { Exposure::Deferred }
    fn source(&self) -> ToolSource { ToolSource::BuiltIn }
    fn admission_owner(&self) -> crate::uar::persistence::tool_admission::AdmissionOwner {
        crate::uar::persistence::tool_admission::AdmissionOwner::UarRuntime
    }
    async fn execute(&self, _: Value) -> anyhow::Result<Value> { anyhow::bail!("followup_denied") }
    async fn execute_with_context(&self, args: Value, context: &NativeExecutionContext) -> anyhow::Result<Value> {
        let owner = context.verified_owner.as_ref().ok_or_else(|| anyhow::anyhow!("followup_denied"))?;
        let workspace = args["workspaceId"].as_str().ok_or_else(|| anyhow::anyhow!("followup_invalid"))?;
        let id = args["id"].as_str().unwrap_or_default();
        let controller = self.controller()?;
        match self.name {
            "followup_list" => return Ok(serde_json::to_value(controller.list(owner, workspace).await.map_err(safe)?)?),
            "followup_inbox" => return Ok(serde_json::to_value(controller.inbox(owner, workspace, text(&args, "conversationId")?).await.map_err(safe)?)?),
            "followup_acknowledge" => {
                controller.acknowledge(owner, workspace, text(&args, "conversationId")?, text(&args, "itemId")?).await.map_err(safe)?;
                return Ok(json!({"acknowledged":true}));
            }
            _ => {},
        }
        let result = match self.name {
            "followup_get" => controller.get(owner, workspace, id).await,
            "followup_create" => controller.create(owner, workspace, serde_json::from_value(args["rule"].clone())?).await,
            "followup_update" => controller.revise(owner, workspace, id, revision(&args)?, serde_json::from_value(args["rule"].clone())?).await,
            _ => controller.control(owner, workspace, id, revision(&args)?, self.name.trim_start_matches("followup_")).await,
        };
        Ok(serde_json::to_value(result.map_err(safe)?)?)
    }
}
fn safe(error: crate::uar::runtime::followups::FollowupError) -> anyhow::Error { anyhow::anyhow!(error.code()) }
fn text<'a>(args: &'a Value, key: &str) -> anyhow::Result<&'a str> { args[key].as_str().ok_or_else(|| anyhow::anyhow!("followup_invalid")) }
fn revision(args: &Value) -> anyhow::Result<u64> { args["expectedRevision"].as_u64().ok_or_else(|| anyhow::anyhow!("followup_invalid")) }
pub async fn register(registry: &NativeSkillRegistry, controller: Arc<FollowupController>) -> Result<(), ToolAssemblyError> {
    let controller = Arc::downgrade(&controller);
    for name in NAMES { registry.register(FollowupTool { controller: controller.clone(), name }).await?; }
    Ok(())
}
