//! Native terminal_exec tool — run shell commands in the configured sandbox.

use crate::uar::runtime::native_skill::NativeSkill;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

#[derive(Debug)]
pub struct TerminalExecTool {
    pub shell: String,
    pub timeout_secs: u64,
    pub use_sandbox: bool,
}

#[async_trait]
impl NativeSkill for TerminalExecTool {
    fn name(&self) -> &str {
        "terminal_exec"
    }
    fn description(&self) -> &str {
        "Execute a shell command and return its stdout, stderr, and exit code."
    }
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["command"],
            "properties": {
                "command": { "type": "string" },
                "working_dir": { "type": "string" },
                "env": {
                    "type": "object",
                    "additionalProperties": { "type": "string" }
                },
                "timeout_secs": { "type": "integer", "minimum": 1 }
            }
        })
    }

    fn format_result(
        &self,
        result: &Value,
        policy: crate::uar::runtime::context::truncate::TruncationPolicy,
        model: &str,
    ) -> String {
        use crate::uar::runtime::context::truncate::formatted_truncate_for_model;

        let serialized = serde_json::to_string(result).unwrap_or_default();
        if formatted_truncate_for_model(&serialized, policy, model) == serialized {
            return serialized;
        }

        let stdout = result.get("stdout").and_then(Value::as_str).unwrap_or("");
        let stderr = result.get("stderr").and_then(Value::as_str).unwrap_or("");
        let exit_code = result
            .get("exit_code")
            .and_then(Value::as_i64)
            .unwrap_or(-1);
        let ok = result.get("ok").and_then(Value::as_bool).unwrap_or(false);
        let command = result.get("command").and_then(Value::as_str).unwrap_or("");
        let transcript = format!(
            "stdout:\n{stdout}\nstderr:\n{stderr}\nexit_code: {exit_code}\nok: {ok}\ncommand: {command}"
        );
        formatted_truncate_for_model(&transcript, policy, model)
    }

    async fn execute(&self, args: Value) -> anyhow::Result<Value> {
        let command = match args.get("command").and_then(Value::as_str) {
            Some(c) => c.to_string(),
            None => {
                return Ok(json!({"ok": false, "error": "Missing required parameter: command"}));
            }
        };
        let cmd_timeout = args
            .get("timeout_secs")
            .and_then(Value::as_u64)
            .unwrap_or(self.timeout_secs);
        if self.use_sandbox {
            return Ok(json!({
                "ok": false,
                "error": "Sandbox execution not yet wired. \
                          Set native_tools.terminal_use_sandbox = false for direct host execution (dev only)."
            }));
        }
        let mut cmd = Command::new(&self.shell);
        cmd.arg("-c").arg(&command);
        if let Some(dir) = args.get("working_dir").and_then(Value::as_str) {
            cmd.current_dir(dir);
        }
        if let Some(env_obj) = args.get("env").and_then(Value::as_object) {
            for (k, v) in env_obj {
                if let Some(val) = v.as_str() {
                    cmd.env(k, val);
                }
            }
        }
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        let result = timeout(Duration::from_secs(cmd_timeout), async {
            match cmd.output().await {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let exit_code = output.status.code().unwrap_or(-1);
                    Ok(json!({
                        "ok": exit_code == 0,
                        "exit_code": exit_code,
                        "stdout": stdout,
                        "stderr": stderr,
                        "command": command
                    }))
                }
                Err(e) => Ok(json!({"ok": false, "error": format!("Execution failed: {}", e)})),
            }
        })
        .await;
        match result {
            Ok(v) => v,
            Err(_) => Ok(json!({
                "ok": false,
                "error": format!("Command timed out after {}s", cmd_timeout),
                "command": command
            })),
        }
    }
}
