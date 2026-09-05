//! Model-only capability that delegates activation to the trusted host.

use std::sync::Arc;

use serde_json::{Value, json};
use tokio::sync::Mutex;

use crate::uar::runtime::native_skill::NativeSkill;
use crate::uar::runtime::skills::activation::{ActivationContext, InvokeType, activate};
use crate::uar::tools::descriptor::{Exposure, ToolSource};

pub struct ActivateSkillTool {
    context: Arc<Mutex<ActivationContext>>,
    thread_policy: Option<Arc<crate::uar::runtime::thread::policy_intersection::ThreadPolicy>>,
}

impl std::fmt::Debug for ActivateSkillTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActivateSkillTool").finish_non_exhaustive()
    }
}

impl ActivateSkillTool {
    pub fn new(context: Arc<Mutex<ActivationContext>>) -> Self {
        Self {
            context,
            thread_policy: None,
        }
    }

    /// Retain the policy used by the host to construct this turn's activation
    /// context; parent-bound handlers are never reused for child execution.
    pub(crate) fn with_thread_policy(
        mut self,
        policy: Option<Arc<crate::uar::runtime::thread::policy_intersection::ThreadPolicy>>,
    ) -> Self {
        self.thread_policy = policy;
        self
    }
}

#[async_trait::async_trait]
impl NativeSkill for ActivateSkillTool {
    fn name(&self) -> &str {
        "activate_skill"
    }

    fn description(&self) -> &str {
        "Activate an eligible skill from the catalog. Its instructions and permitted tools become available on the next model step."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {"skill_id": {"type": "string", "minLength": 1}},
            "required": ["skill_id"],
            "additionalProperties": false
        })
    }

    fn exposure(&self) -> Exposure {
        Exposure::ModelOnly
    }

    fn source(&self) -> ToolSource {
        ToolSource::BuiltIn
    }

    fn check_thread_policy(
        &self,
        policy: &crate::uar::runtime::thread::policy_intersection::ThreadPolicy,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.thread_policy
                .as_ref()
                .is_some_and(|bound| std::ptr::eq(bound.as_ref(), policy)),
            "Skill activation is not bound to this delegated turn"
        );
        Ok(())
    }

    async fn execute(&self, args: Value) -> anyhow::Result<Value> {
        let skill_id = args
            .get("skill_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("skill_id must be a string"))?;
        let mut context = self.context.lock().await;
        match activate(skill_id, &mut context, InvokeType::Model).await {
            Ok(activated) => Ok(json!({
                "status": "activated",
                "skill_id": activated.skill.skill_id,
                "invoke_type": activated.invoke_type,
                "sequence": activated.sequence,
            })),
            Err(failure) => Ok(json!({"status": "activation_failed", "failure": failure})),
        }
    }
}
