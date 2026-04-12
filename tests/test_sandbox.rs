/// Microsandbox integration tests.
///
/// These tests require the `sandbox-microsandbox` feature flag AND a working
/// hypervisor (KVM on Linux, HVF on macOS). They are ignored by default and
/// must be run explicitly:
///
///   cargo test --features sandbox-microsandbox --test test_sandbox -- --ignored

#[cfg(feature = "sandbox-microsandbox")]
mod microsandbox_tests {
    use universal_agent_runtime::sandbox::{
        runner::SandboxRunner,
        types::{ExecutionMode, ExecutionRequest, Language, SandboxConfig},
    };
    use universal_agent_runtime::sandbox::microsandbox_runner::MicrosandboxRunner;

    #[tokio::test]
    #[ignore = "requires hypervisor and microsandbox daemon"]
    async fn test_sandbox_creates_and_executes_command() {
        // This test requires a running microsandbox daemon and hypervisor support.
        // It is marked #[ignore] so CI doesn't run it automatically.
        let config = universal_agent_runtime::config::AppConfig::default_for_test();
        let runner = MicrosandboxRunner::new(&config);

        let sandbox_config = SandboxConfig {
            image: "ubuntu:22.04".to_string(),
            memory_mib: 512,
            cpus: 1,
            network_enabled: false,
            volumes: std::collections::HashMap::new(),
            env_vars: std::collections::HashMap::new(),
            timeout_secs: 30,
        };

        let handle = runner.create(sandbox_config).await.expect("sandbox create");
        assert!(!handle.id.is_empty());

        let result = runner
            .execute(
                &handle,
                ExecutionRequest {
                    language: Language::Bash,
                    code: "echo hello_from_sandbox".to_string(),
                    stdin: None,
                    env: std::collections::HashMap::new(),
                    cwd: None,
                    timeout_seconds: Some(10),
                    mode: ExecutionMode::Ephemeral,
                },
            )
            .await
            .expect("execute");

        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("hello_from_sandbox"));

        runner.destroy(handle).await.expect("destroy");
    }
}

#[test]
fn sandbox_test_module_compiles() {
    // Minimal smoke-test that this file compiles without sandbox feature.
}
