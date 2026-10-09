//! Apply the existing persisted native-tool preferences before registration.

use anyhow::Context;

use crate::{config::NativeToolsConfig, uar::settings::manager::SettingsManager};

pub(crate) async fn startup_config(
    configured: &NativeToolsConfig,
    settings: Option<&SettingsManager>,
) -> anyhow::Result<NativeToolsConfig> {
    let Some(settings) = settings else {
        return Ok(configured.clone());
    };
    let mut values = serde_json::to_value(configured)?;
    for field in [
        "file_tools_enabled",
        "file_allowed_paths",
        "file_max_size_kb",
        "file_write_max_kb",
        "web_fetch_enabled",
        "web_fetch_timeout_secs",
        "web_fetch_max_size_kb",
        "web_fetch_allowed_domains",
        "terminal_exec_enabled",
        "terminal_shell",
        "terminal_timeout_secs",
        "terminal_use_sandbox",
        "session_search_enabled",
        "session_search_max_results",
    ] {
        if let Some(value) = settings
            .load_optional_persisted_value(&format!("native_tools.{field}"))
            .await
            .with_context(|| format!("could not load native-tools preference {field}"))?
        {
            values[field] = value;
        }
    }
    serde_json::from_value(values).context("persisted native-tools configuration is invalid")
}
