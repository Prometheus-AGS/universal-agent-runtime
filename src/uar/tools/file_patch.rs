//! Native file_patch tool — apply targeted text replacements to files.

use crate::uar::runtime::native_skill::NativeSkill;
use crate::uar::tools::descriptor::{ToolEffect, ToolSource};
use async_trait::async_trait;
use serde_json::{Value, json};
use tokio::fs;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

use super::file_tools::{
    ConfinedOpenMode, DelegatedFileRoots, check_delegated_file_policy, file_limit_bytes,
    open_confined_file, path_allowed, read_bounded_file,
};

#[derive(Debug)]
pub struct FilePatchTool {
    pub allowed_paths: Vec<String>,
    pub max_size_kb: u64,
    delegated_roots: DelegatedFileRoots,
}

impl FilePatchTool {
    pub(crate) fn new(
        allowed_paths: Vec<String>,
        max_size_kb: u64,
        delegated_roots: DelegatedFileRoots,
    ) -> Self {
        Self {
            allowed_paths,
            max_size_kb,
            delegated_roots,
        }
    }

    async fn execute_inner(&self, args: Value, confined: bool) -> anyhow::Result<Value> {
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

        let mut file = if confined {
            match open_confined_file(&path_str, &self.delegated_roots, ConfinedOpenMode::Patch)
                .await
            {
                Ok(file) => file,
                Err(error) => {
                    return Ok(
                        json!({"ok": false, "error": format!("Cannot open confined file: {error}")}),
                    );
                }
            }
        } else {
            let canonical = match std::fs::canonicalize(&path_str) {
                Ok(path) => path,
                Err(error) => {
                    return Ok(json!({
                        "ok": false,
                        "error": format!("Cannot resolve path: {error}")
                    }));
                }
            };
            if !path_allowed(&canonical, &self.allowed_paths) {
                return Ok(json!({
                    "ok": false,
                    "error": format!("Path '{path_str}' is not in the allowed paths list.")
                }));
            }
            match fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&canonical)
                .await
            {
                Ok(file) => file,
                Err(error) => {
                    return Ok(json!({"ok": false, "error": format!("Cannot open file: {error}")}));
                }
            }
        };
        let max_bytes = file_limit_bytes(self.max_size_kb)?;
        let content = match read_bounded_file(&mut file, max_bytes).await {
            Ok(content) => content,
            Err(error) => {
                return Ok(json!({"ok": false, "error": format!("Read failed: {error}")}));
            }
        };
        let occurrences = content.matches(old_string.as_str()).count();
        if occurrences == 0 {
            return Ok(json!({"ok": false, "error": "old_string not found in file"}));
        }
        if occurrences > 1 && !allow_multiple {
            return Ok(json!({
                "ok": false,
                "error": format!(
                    "old_string appears {occurrences} times. Use allow_multiple=true to replace all."
                )
            }));
        }
        // Check expansion before String::replace allocates its output. Input
        // bounds alone do not constrain a repeated replacement's result.
        let replacement_count = if allow_multiple { occurrences } else { 1 };
        let patched_size = old_string
            .len()
            .checked_mul(replacement_count)
            .and_then(|removed| content.len().checked_sub(removed))
            .and_then(|kept| {
                new_string
                    .len()
                    .checked_mul(replacement_count)
                    .and_then(|added| kept.checked_add(added))
            });
        if patched_size.is_none_or(|size| size as u64 > max_bytes) {
            return Ok(json!({"ok": false, "error": "Patched file would exceed size limit"}));
        }
        let patched = if allow_multiple {
            content.replace(old_string.as_str(), new_string.as_str())
        } else {
            content.replacen(old_string.as_str(), new_string.as_str(), 1)
        };
        // Keep the capability-opened handle across read/modify/write. Replacing
        // the pathname cannot redirect this write to a different file.
        let result: std::io::Result<()> = async {
            file.seek(std::io::SeekFrom::Start(0)).await?;
            file.write_all(patched.as_bytes()).await?;
            file.flush().await?;
            file.set_len(patched.len() as u64).await
        }
        .await;
        match result {
            Ok(()) => Ok(json!({
                "ok": true,
                "path": path_str,
                "replacements": occurrences,
                "bytes_written": patched.len()
            })),
            Err(error) => Ok(json!({"ok": false, "error": format!("Write failed: {error}")})),
        }
    }
}

#[async_trait]
impl NativeSkill for FilePatchTool {
    fn check_thread_policy(
        &self,
        _policy: &crate::uar::runtime::thread::policy_intersection::ThreadPolicy,
    ) -> anyhow::Result<()> {
        check_delegated_file_policy(&self.allowed_paths, &self.delegated_roots)
    }

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
    async fn execute_with_context(
        &self,
        args: Value,
        context: &crate::uar::runtime::native_skill::NativeExecutionContext,
    ) -> anyhow::Result<Value> {
        self.execute_inner(args, context.thread_policy.is_some())
            .await
    }
    async fn execute(&self, args: Value) -> anyhow::Result<Value> {
        self.execute_inner(args, false).await
    }
}
