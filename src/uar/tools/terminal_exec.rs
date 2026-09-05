//! Native terminal_exec tool — run shell commands in the configured sandbox.

use crate::uar::runtime::native_skill::NativeSkill;
use crate::uar::tools::descriptor::{ToolEffect, ToolSource};
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

    fn effect(&self) -> ToolEffect {
        ToolEffect::CodeExecution
    }

    fn sandbox_required(&self) -> bool {
        self.use_sandbox
    }

    fn supports_sandbox_execution(&self) -> bool {
        matches!(self.shell.as_str(), "sh" | "/bin/sh" | "bash" | "/bin/bash")
    }

    fn sandbox_request(&self, args: Value) -> anyhow::Result<crate::sandbox::ExecutionRequest> {
        let command = args
            .get("command")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("Missing terminal command"))?;
        // The runner's language protocol supports Bash. Invoke the configured
        // supported shell explicitly so sh is not silently changed to Bash and
        // arbitrary shell configuration cannot become a command interpolation.
        anyhow::ensure!(
            self.supports_sandbox_execution(),
            "Configured terminal shell has no sandbox adapter"
        );
        let code = format!(
            "exec {} -c '{}'",
            self.shell,
            command.replace('\'', "'\\''")
        );
        let env = match args.get("env") {
            Some(value) => serde_json::from_value(value.clone())?,
            None => std::collections::HashMap::new(),
        };
        Ok(crate::sandbox::ExecutionRequest {
            language: crate::sandbox::Language::Bash,
            code,
            stdin: None,
            env,
            cwd: args
                .get("working_dir")
                .and_then(Value::as_str)
                .map(str::to_owned),
            timeout_seconds: Some(
                args.get("timeout_secs")
                    .and_then(Value::as_u64)
                    .unwrap_or(self.timeout_secs)
                    .min(self.timeout_secs),
            ),
            mode: crate::sandbox::ExecutionMode::Ephemeral,
        })
    }

    fn source(&self) -> ToolSource {
        ToolSource::BuiltIn
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
        self.execute_in_scope(args, None).await
    }

    async fn execute_with_context(
        &self,
        args: Value,
        context: &crate::uar::runtime::native_skill::NativeExecutionContext,
    ) -> anyhow::Result<Value> {
        // A scope owns cleanup, not authorization. Delegated direct execution
        // remains denied by execute_native until its permission port exists.
        anyhow::ensure!(
            context.terminal_scope.is_some()
                || (context.verified_owner.is_none() && context.thread_policy.is_none()),
            "Verified terminal execution requires a host-owned process scope"
        );
        self.execute_in_scope(args, context.terminal_scope.as_ref())
            .await
    }
}

impl TerminalExecTool {
    async fn execute_in_scope(
        &self,
        args: Value,
        scope: Option<&super::terminal_process::TerminalRun>,
    ) -> anyhow::Result<Value> {
        let command = match args.get("command").and_then(Value::as_str) {
            Some(c) => c.to_string(),
            None => {
                return Ok(json!({"ok": false, "error": "Missing required parameter: command"}));
            }
        };
        let cmd_timeout = args
            .get("timeout_secs")
            .and_then(Value::as_u64)
            .unwrap_or(self.timeout_secs)
            .min(self.timeout_secs);
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
        cmd.kill_on_drop(true);
        if let Some(scope) = scope {
            return match scope.execute(cmd, Duration::from_secs(cmd_timeout)).await {
                Ok(output) => Ok(json!({
                    "ok": output.status.success(),
                    "exit_code": output.status.code().unwrap_or(-1),
                    "stdout": output.stdout,
                    "stderr": output.stderr,
                    "stdout_bytes": output.stdout_bytes,
                    "stderr_bytes": output.stderr_bytes,
                    "command": command
                })),
                Err(error) => {
                    Ok(json!({"ok": false, "error": error.to_string(), "command": command}))
                }
            };
        }
        // Compatibility for standalone tool callers outside RunManager. The
        // drop guard requests termination, but only a host scope proves joining.
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
