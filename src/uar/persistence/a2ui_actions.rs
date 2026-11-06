//! Immutable host admission receipts. A missing execution is uncertain, never
//! permission to execute the same logical interaction again.
use super::agui_runs::AguiRunBinding;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct A2uiActionReceipt {
    pub owner: String,
    pub agent_id: String,
    pub thread_id: String,
    pub interaction_id: String,
    pub source_run_id: String,
    pub surface_id: String,
    pub surface_generation: String,
    pub component_id: String,
    pub action_name: String,
    pub semantic_digest: String,
    pub continuation_run_id: String,
    pub wire_binding: Option<AguiRunBinding>,
}

impl A2uiActionReceipt {
    pub fn key(owner: &str, agent: &str, thread: &str, interaction: &str) -> String {
        digest(&serde_json::json!([owner, agent, thread, interaction]))
    }
    pub fn storage_key(&self) -> String {
        Self::key(
            &self.owner,
            &self.agent_id,
            &self.thread_id,
            &self.interaction_id,
        )
    }
    pub fn validate(&self) -> anyhow::Result<()> {
        for id in [
            &self.owner,
            &self.agent_id,
            &self.thread_id,
            &self.interaction_id,
            &self.source_run_id,
            &self.surface_id,
            &self.surface_generation,
            &self.component_id,
            &self.action_name,
            &self.continuation_run_id,
        ] {
            anyhow::ensure!(valid_id(id), "invalid_a2ui_interaction_identity");
        }
        anyhow::ensure!(
            self.semantic_digest.len() == 64
                && self
                    .semantic_digest
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit()),
            "invalid_a2ui_interaction_digest"
        );
        if let Some(binding) = &self.wire_binding {
            binding.validate()?;
            anyhow::ensure!(
                binding.owner == self.owner
                    && binding.agent_id == self.agent_id
                    && binding.session_id == self.thread_id
                    && binding.native_run_id == self.continuation_run_id
                    && binding.action_interaction_id.as_deref()
                        == Some(self.interaction_id.as_str()),
                "a2ui_interaction_binding_conflict"
            );
        }
        Ok(())
    }
}

pub fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 512 && id.trim() == id && !id.chars().any(char::is_control)
}

/// JSON objects are sorted recursively so transport object ordering cannot
/// create a second interpretation of the same semantic payload.
pub fn digest(value: &Value) -> String {
    fn canonical(value: &Value) -> Value {
        match value {
            Value::Object(object) => {
                let ordered: std::collections::BTreeMap<_, _> = object
                    .iter()
                    .map(|(key, value)| (key.clone(), canonical(value)))
                    .collect();
                serde_json::json!(ordered)
            }
            Value::Array(array) => Value::Array(array.iter().map(canonical).collect()),
            other => other.clone(),
        }
    }
    Sha256::digest(canonical(value).to_string().as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn semantic_object_order_is_stable_and_context_changes_conflict() {
        assert_eq!(
            digest(&serde_json::json!({"context":{"a":1,"b":2},"generation":"g"})),
            digest(&serde_json::json!({"generation":"g","context":{"b":2,"a":1}}))
        );
        assert_ne!(
            digest(&serde_json::json!({"context":{"a":1}})),
            digest(&serde_json::json!({"context":{"a":2}}))
        );
    }
    #[test]
    fn receipt_identity_separates_verified_owner_thread_and_intent() {
        let key = A2uiActionReceipt::key("owner", "agent", "thread", "click");
        assert_ne!(
            key,
            A2uiActionReceipt::key("foreign", "agent", "thread", "click")
        );
        assert_ne!(
            key,
            A2uiActionReceipt::key("owner", "agent", "other", "click")
        );
        assert_ne!(
            key,
            A2uiActionReceipt::key("owner", "agent", "thread", "new-click")
        );
    }
}
