//! System Info native skill — returns host system information.
//!
//! Provides agents with context about the runtime environment without
//! requiring any external tools or MCP servers.

use crate::uar::runtime::native_skill::NativeSkill;
use crate::uar::tools::descriptor::{ToolEffect, ToolSource};

/// Returns basic host system information.
#[derive(Debug)]
pub struct SystemInfoSkill;

#[async_trait::async_trait]
impl NativeSkill for SystemInfoSkill {
    fn name(&self) -> &str {
        "native_system_info"
    }

    fn description(&self) -> &str {
        "Returns basic host system information including OS, architecture, and CPU count."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
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
        _policy: &crate::uar::runtime::thread::policy_intersection::ThreadPolicy,
    ) -> anyhow::Result<()> {
        // Fixed platform/CPU facts, not process environment values or host paths.
        Ok(())
    }

    async fn execute(&self, _args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        tracing::debug!("Executing native system info skill");
        Ok(serde_json::json!({
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "family": std::env::consts::FAMILY,
            "cpu_count": num_cpus::get(),
        }))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[tokio::test]
    async fn test_system_info_returns_valid_data() {
        let skill = SystemInfoSkill;
        let result = skill.execute(serde_json::Value::Null).await.unwrap();

        assert!(result["os"].is_string());
        assert!(result["arch"].is_string());
        assert!(result["family"].is_string());
        assert!(result["cpu_count"].is_number());

        let cpu_count = result["cpu_count"].as_u64().unwrap();
        assert!(cpu_count > 0, "CPU count should be > 0");
    }

    #[test]
    fn test_system_info_metadata() {
        let skill = SystemInfoSkill;
        assert_eq!(skill.name(), "native_system_info");
        assert!(!skill.description().is_empty());
    }
}
