//! Native file-system tools: file_read and file_write.

use crate::uar::runtime::native_skill::NativeSkill;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use tokio::fs;

fn path_allowed(target: &Path, allowed: &[String]) -> bool {
    if allowed.is_empty() { return false; }
    if allowed.iter().any(|p| p == "*") { return true; }
    allowed.iter().any(|prefix| target.starts_with(PathBuf::from(prefix)))
}

// =============================================================================
// FileReadTool
// =============================================================================

#[derive(Debug)]
pub struct FileReadTool {
    pub allowed_paths: Vec<String>,
    pub max_size_kb: u64,
}

#[async_trait]
impl NativeSkill for FileReadTool {
    fn name(&self) -> &str { "file_read" }
    fn description(&self) -> &str {
        "Read the contents of a file from the local filesystem."
    }
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["path"],
            "properties": {
                "path": { "type": "string", "description": "Absolute path to the file." },
                "offset_lines": { "type": "integer", "minimum": 0 },
                "limit_lines": { "type": "integer", "minimum": 1 }
            }
        })
    }
    async fn execute(&self, args: Value) -> anyhow::Result<Value> {
        let path_str = match args.get("path").and_then(Value::as_str) {
            Some(p) => p.to_string(),
            None => return Ok(json!({"ok": false, "error": "Missing required parameter: path"})),
        };
        let canonical = match std::fs::canonicalize(&path_str) {
            Ok(p) => p,
            Err(e) => return Ok(json!({"ok": false, "error": format!("Cannot resolve '{}': {}", path_str, e)})),
        };
        if !path_allowed(&canonical, &self.allowed_paths) {
            return Ok(json!({"ok": false, "error": format!("Path '{}' is not in the allowed paths list.", path_str)}));
        }
        let metadata = match fs::metadata(&canonical).await {
            Ok(m) => m,
            Err(e) => return Ok(json!({"ok": false, "error": format!("Cannot stat: {}", e)})),
        };
        let size_kb = metadata.len() / 1024;
        if size_kb > self.max_size_kb {
            return Ok(json!({"ok": false, "error": format!("File {}KB exceeds limit {}KB", size_kb, self.max_size_kb)}));
        }
        match fs::read_to_string(&canonical).await {
            Ok(content) => {
                let offset = args.get("offset_lines").and_then(Value::as_u64).unwrap_or(0) as usize;
                let limit = args.get("limit_lines").and_then(Value::as_u64);
                let lines: Vec<&str> = content.lines().collect();
                let sliced: Vec<&str> = match limit {
                    Some(n) => lines.iter().skip(offset).take(n as usize).copied().collect(),
                    None => lines.iter().skip(offset).copied().collect(),
                };
                Ok(json!({
                    "ok": true,
                    "path": path_str,
                    "content": sliced.join("\n"),
                    "total_lines": lines.len(),
                    "returned_lines": sliced.len()
                }))
            }
            Err(e) => Ok(json!({"ok": false, "error": format!("Read failed: {}", e)})),
        }
    }
}

// =============================================================================
// FileWriteTool
// =============================================================================

#[derive(Debug)]
pub struct FileWriteTool {
    pub allowed_paths: Vec<String>,
    pub max_size_kb: u64,
}

#[async_trait]
impl NativeSkill for FileWriteTool {
    fn name(&self) -> &str { "file_write" }
    fn description(&self) -> &str {
        "Write or overwrite a file on the local filesystem."
    }
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["path", "content"],
            "properties": {
                "path": { "type": "string" },
                "content": { "type": "string" },
                "append": { "type": "boolean" }
            }
        })
    }
    async fn execute(&self, args: Value) -> anyhow::Result<Value> {
        let path_str = match args.get("path").and_then(Value::as_str) {
            Some(p) => p.to_string(),
            None => return Ok(json!({"ok": false, "error": "Missing required parameter: path"})),
        };
        let content = match args.get("content").and_then(Value::as_str) {
            Some(c) => c.to_string(),
            None => return Ok(json!({"ok": false, "error": "Missing required parameter: content"})),
        };
        let append = args.get("append").and_then(Value::as_bool).unwrap_or(false);
        let content_kb = (content.len() as u64) / 1024;
        if content_kb > self.max_size_kb {
            return Ok(json!({"ok": false, "error": format!("Content {}KB exceeds limit {}KB", content_kb, self.max_size_kb)}));
        }
        let target = PathBuf::from(&path_str);
        let check_path = if target.exists() {
            match std::fs::canonicalize(&target) {
                Ok(p) => p,
                Err(e) => return Ok(json!({"ok": false, "error": format!("Cannot resolve: {}", e)})),
            }
        } else {
            let parent = target.parent().unwrap_or(Path::new("."));
            match std::fs::canonicalize(parent) {
                Ok(p) => p.join(target.file_name().unwrap_or_default()),
                Err(e) => return Ok(json!({"ok": false, "error": format!("Cannot resolve parent: {}", e)})),
            }
        };
        if !path_allowed(&check_path, &self.allowed_paths) {
            return Ok(json!({"ok": false, "error": format!("Path '{}' is not in the allowed paths list.", path_str)}));
        }
        if let Some(parent) = target.parent() {
            if let Err(e) = fs::create_dir_all(parent).await {
                return Ok(json!({"ok": false, "error": format!("Cannot create directories: {}", e)}));
            }
        }
        let result = if append {
            use tokio::io::AsyncWriteExt;
            match fs::OpenOptions::new().create(true).append(true).open(&target).await {
                Ok(mut f) => f.write_all(content.as_bytes()).await,
                Err(e) => return Ok(json!({"ok": false, "error": format!("Cannot open file: {}", e)})),
            }
        } else {
            fs::write(&target, &content).await
        };
        match result {
            Ok(()) => Ok(json!({
                "ok": true,
                "path": path_str,
                "bytes_written": content.len(),
                "mode": if append { "append" } else { "overwrite" }
            })),
            Err(e) => Ok(json!({"ok": false, "error": format!("Write failed: {}", e)})),
        }
    }
}
