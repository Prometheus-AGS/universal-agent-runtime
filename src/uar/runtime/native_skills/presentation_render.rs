//! Template preparation uses an immutable host capture and performs no writes.

use crate::uar::runtime::native_skill::{NativeExecutionContext, NativeSkill};
use crate::uar::tools::descriptor::{ToolEffect, ToolSource};

pub const PRESENTATION_RENDER_NAME: &str = "presentation_render";

#[derive(Debug)]
pub struct PresentationRenderSkill;

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RenderArguments {
    template_id: String,
    #[serde(default)]
    data: serde_json::Map<String, serde_json::Value>,
}

#[async_trait::async_trait]
impl NativeSkill for PresentationRenderSkill {
    fn name(&self) -> &str {
        PRESENTATION_RENDER_NAME
    }

    fn description(&self) -> &str {
        "Prepare an eligible reusable UI template from this run's frozen catalog. Supply its template_id and declarative data. The host controls publication and surface identity; preparation does not prove client display."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object", "required": ["template_id"],
            "properties": {
                "template_id": { "type": "string", "minLength": 1 },
                "data": { "type": "object", "additionalProperties": true },
            },
            "additionalProperties": false,
        })
    }

    fn effect(&self) -> ToolEffect {
        ToolEffect::ReadOnly
    }
    fn source(&self) -> ToolSource {
        ToolSource::BuiltIn
    }

    fn check_thread_policy(
        &self,
        policy: &crate::uar::runtime::thread::policy_intersection::ThreadPolicy,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            policy
                .effective()
                .tools
                .ids
                .iter()
                .any(|name| name == PRESENTATION_RENDER_NAME)
                && !policy.effective().presentations.ids.is_empty(),
            "Delegated policy does not permit template preparation"
        );
        Ok(())
    }

    async fn execute(&self, _args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        anyhow::bail!("Template preparation requires a captured host context")
    }

    async fn execute_with_context(
        &self,
        args: serde_json::Value,
        context: &NativeExecutionContext,
    ) -> anyhow::Result<serde_json::Value> {
        let args: RenderArguments = serde_json::from_value(args)?;
        let snapshot = context
            .presentations
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No Presentation snapshot is bound to this run"))?
            .snapshot();
        anyhow::ensure!(
            snapshot.owner().is_some() && snapshot.owner() == context.verified_owner.as_ref(),
            "Presentation snapshot owner does not match this run"
        );
        if let Some(policy) = &context.thread_policy {
            anyhow::ensure!(
                policy
                    .effective()
                    .presentations
                    .ids
                    .contains(&args.template_id),
                "Template is outside the delegated Presentation ceiling"
            );
        }
        snapshot.prepare(&args.template_id, &args.data)
    }
}
