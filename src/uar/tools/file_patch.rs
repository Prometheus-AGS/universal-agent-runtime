//! Native file_patch tool — apply targeted text replacements to files.

use crate::uar::runtime::native_skill::NativeSkill;
use crate::uar::tools::descriptor::{ToolEffect, ToolSource};
use async_trait::async_trait;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use tokio::fs;

fn path_allowed(target: &Path, allowed: &[String]) -> bool {
    if allowed.is_empty() {
        return false;
    }
    if allowed.iter().any(|p| p == "*") {
        return true;
    }
    allowed
        .iter()
        .any(|prefix| target.starts_with(PathBuf::from(prefix)))
}

#[derive(Debug)]
pub struct FilePatchTool {
    pub allowed_paths: Vec<String>,
    pub max_size_kb: u64,
}

#[async_trait]
impl NativeSkill for FilePatchTool {
    fn name(&self) -> &str {
        "file_patch"
    }
    fn description(&self) -> &str {
        "Apply a targeted text replacement to a file — replaces old_string with new_string. \
         Fails if old_string is not found or appears more than once."
    }
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["path", "old_string", "new_string"],
            "properties": {
                "path": { "type": "string" },
                "old_string": { "type": "string" },
                "new_string": { "type": "string" },
                "allow_multiple": { "type": "boolean" }
            }
        })
    }
    fn effect(&self) -> ToolEffect {
        ToolEffect::ExternalMutation
    }
    fn source(&self) -> ToolSource {
        ToolSource::BuiltIn
    }
    async fn execute(&self, args: Value) -> anyhow::Result<Value> {
        let path_str = match args.get("path").and_then(Value::as_str) {
            Some(p) => p.to_string(),
            None => return Ok(json!({"ok": false, "error": "Missing required parameter: path"})),
        };
        let old_string = match args.get("old_string").and_then(Value::as_str) {
            Some(s) => s.to_string(),
            None => {
                return Ok(json!({"ok": false, "error": "Missing required parameter: old_string"}));
            }
        };
        let new_string = match args.get("new_string").and_then(Value::as_str) {
            Some(s) => s.to_string(),
            None => {
                return Ok(json!({"ok": false, "error": "Missing required parameter: new_string"}));
            }
        };
        let allow_multiple = args
            .get("allow_multiple")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let canonical = match std::fs::canonicalize(&path_str) {
            Ok(p) => p,
            Err(e) => {
                return Ok(json!({"ok": false, "error": format!("Cannot resolve path: {}", e)}));
            }
        };
        if !path_allowed(&canonical, &self.allowed_paths) {
            return Ok(
                json!({"ok": false, "error": format!("Path '{}' is not in the allowed paths list.", path_str)}),
            );
        }
        if let Ok(meta) = fs::metadata(&canonical).await {
            if meta.len() / 1024 > self.max_size_kb {
                return Ok(json!({"ok": false, "error": "File exceeds size limit for patching"}));
            }
        }
        let content = match fs::read_to_string(&canonical).await {
            Ok(c) => c,
            Err(e) => return Ok(json!({"ok": false, "error": format!("Read failed: {}", e)})),
        };
        let occurrences = content.matches(old_string.as_str()).count();
        if occurrences == 0 {
            return Ok(json!({"ok": false, "error": "old_string not found in file"}));
        }
        if occurrences > 1 && !allow_multiple {
            return Ok(json!({
                "ok": false,
                "error": format!(
                    "old_string appears {} times. Use allow_multiple=true to replace all.", occurrences
                )
            }));
        }
        let patched = if allow_multiple {
            content.replace(old_string.as_str(), new_string.as_str())
        } else {
            content.replacen(old_string.as_str(), new_string.as_str(), 1)
        };
        match fs::write(&canonical, &patched).await {
            Ok(()) => Ok(json!({
                "ok": true,
                "path": path_str,
                "replacements": occurrences,
                "bytes_written": patched.len()
            })),
            Err(e) => Ok(json!({"ok": false, "error": format!("Write failed: {}", e)})),
        }
    }
}
