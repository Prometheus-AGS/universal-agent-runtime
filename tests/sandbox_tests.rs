//! Integration tests for the sandbox module.

#[cfg(test)]
mod sandbox_types {
    use std::collections::HashMap;
    use universal_agent_runtime::sandbox::types::*;

    #[test]
    fn default_sandbox_config_is_sane() {
        let config = SandboxConfig::default();
        assert_eq!(config.memory_mib, 512);
        assert_eq!(config.cpus, 1);
        assert!(!config.network_enabled);
        assert_eq!(config.timeout_secs, 300);
    }

    #[test]
    fn language_command_mapping() {
        assert_eq!(Language::Bash.command(), "bash");
        assert_eq!(Language::Python.command(), "python3");
        assert_eq!(Language::Node.command(), "node");
    }

    #[test]
    fn language_exec_args() {
        let args = Language::Bash.exec_args("echo hello");
        assert_eq!(args, vec!["-c", "echo hello"]);

        let args = Language::Python.exec_args("print('hi')");
        assert_eq!(args, vec!["-c", "print('hi')"]);
    }

    #[test]
    fn execution_request_serializes() {
        let req = ExecutionRequest {
            language: Language::Python,
            code: "print('hello')".to_string(),
            stdin: None,
            env: HashMap::new(),
            cwd: None,
            timeout_seconds: Some(30),
            mode: ExecutionMode::Ephemeral,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("python"));
        assert!(json.contains("print"));
    }

    #[test]
    fn execution_mode_serializes() {
        let mode = ExecutionMode::Session {
            session_id: "sess_123".to_string(),
        };
        let json = serde_json::to_string(&mode).unwrap();
        assert!(json.contains("session"));
        assert!(json.contains("sess_123"));
    }
}

#[cfg(test)]
mod sandbox_runner {
    use universal_agent_runtime::sandbox::runner::*;

    #[test]
    fn runner_type_variants() {
        assert_ne!(RunnerType::MicroVm, RunnerType::Wasmtime);
        assert_ne!(RunnerType::Wasmtime, RunnerType::Remote);
    }

    #[test]
    fn capabilities_struct() {
        let caps = RunnerCapabilities {
            supports_long_running: true,
            supports_networking: false,
            max_execution_seconds: Some(300),
            runner_type: RunnerType::Wasmtime,
        };
        assert!(!caps.supports_networking);
        assert_eq!(caps.runner_type, RunnerType::Wasmtime);
    }
}

#[cfg(test)]
mod wasmtime_runner {
    use std::collections::HashMap;
    use universal_agent_runtime::sandbox::runner::{RunnerType, SandboxRunner};
    use universal_agent_runtime::sandbox::types::*;
    use universal_agent_runtime::sandbox::wasmtime_runner::WasmtimeRunner;

    #[test]
    fn wasmtime_runner_capabilities() {
        let runner = WasmtimeRunner::new();
        let caps = runner.capabilities();
        assert_eq!(caps.runner_type, RunnerType::Wasmtime);
        assert!(!caps.supports_long_running);
        assert!(!caps.supports_networking);
    }

    #[tokio::test]
    async fn wasmtime_runner_executes_bash() {
        let runner = WasmtimeRunner::new();
        let handle = runner
            .create(SandboxConfig::default())
            .await
            .expect("create should succeed");

        let result = runner
            .execute(
                &handle,
                ExecutionRequest {
                    language: Language::Bash,
                    code: "echo hello_sandbox".to_string(),
                    stdin: None,
                    env: HashMap::new(),
                    cwd: None,
                    timeout_seconds: Some(10),
                    mode: ExecutionMode::Ephemeral,
                },
            )
            .await
            .expect("execute should succeed");

        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("hello_sandbox"));
        assert!(result.execution_time_ms < 5000);

        runner
            .destroy(handle)
            .await
            .expect("destroy should succeed");
    }

    /// Test 13.6: Platform fallback — WasmtimeRunner works without microsandbox feature.
    #[tokio::test]
    async fn wasmtime_runner_works_as_fallback() {
        let runner = WasmtimeRunner::new();

        // Should be able to execute Python if available
        let handle = runner.create(SandboxConfig::default()).await.unwrap();
        let result = runner
            .execute(
                &handle,
                ExecutionRequest {
                    language: Language::Bash,
                    code: "echo fallback_works".to_string(),
                    stdin: None,
                    env: HashMap::new(),
                    cwd: None,
                    timeout_seconds: Some(5),
                    mode: ExecutionMode::Ephemeral,
                },
            )
            .await
            .unwrap();

        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("fallback_works"));
    }
}

#[cfg(test)]
mod session_manager {
    use std::sync::Arc;
    use universal_agent_runtime::sandbox::session_manager::SessionManager;
    use universal_agent_runtime::sandbox::types::SandboxConfig;
    use universal_agent_runtime::sandbox::wasmtime_runner::WasmtimeRunner;

    #[tokio::test]
    async fn session_reuse() {
        let runner = Arc::new(WasmtimeRunner::new());
        let mgr = SessionManager::new(runner, 1800, 10);

        // First call creates a sandbox
        let handle1 = mgr
            .get_or_create("sess_1", SandboxConfig::default())
            .await
            .unwrap();

        // Second call should reuse the same sandbox
        let handle2 = mgr
            .get_or_create("sess_1", SandboxConfig::default())
            .await
            .unwrap();

        assert_eq!(handle1.id, handle2.id);
        assert_eq!(mgr.active_count(), 1);
    }

    #[tokio::test]
    async fn different_sessions_get_different_sandboxes() {
        let runner = Arc::new(WasmtimeRunner::new());
        let mgr = SessionManager::new(runner, 1800, 10);

        let h1 = mgr
            .get_or_create("sess_a", SandboxConfig::default())
            .await
            .unwrap();
        let h2 = mgr
            .get_or_create("sess_b", SandboxConfig::default())
            .await
            .unwrap();

        assert_ne!(h1.id, h2.id);
        assert_eq!(mgr.active_count(), 2);
    }

    #[tokio::test]
    async fn capacity_limit_enforced() {
        let runner = Arc::new(WasmtimeRunner::new());
        let mgr = SessionManager::new(runner, 1800, 2); // max 2

        mgr.get_or_create("s1", SandboxConfig::default())
            .await
            .unwrap();
        mgr.get_or_create("s2", SandboxConfig::default())
            .await
            .unwrap();

        // Third should fail
        let result = mgr.get_or_create("s3", SandboxConfig::default()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn destroy_session_removes_entry() {
        let runner = Arc::new(WasmtimeRunner::new());
        let mgr = SessionManager::new(runner, 1800, 10);

        mgr.get_or_create("sess_del", SandboxConfig::default())
            .await
            .unwrap();
        assert_eq!(mgr.active_count(), 1);

        mgr.destroy_session("sess_del").await.unwrap();
        assert_eq!(mgr.active_count(), 0);
    }
}
