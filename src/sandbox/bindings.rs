//! Immutable host configuration for sandbox execution. Child policies select
//! opaque grants from this capture; they never resolve environment variables,
//! mount paths, endpoints or credentials again.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::uar::domain::artifact::ToolExecutionMode;
use crate::uar::runtime::thread::policy_intersection::SandboxPermissions;

use super::{ExecutionMode, ExecutionRequest, SandboxConfig, SandboxRunner};

/// Exact backend and host environment values retained across a delegation tree.
/// This is host authority, not a deserializable artifact or tool argument.
#[derive(Clone)]
pub struct SandboxBinding {
    runner: Arc<dyn SandboxRunner>,
    config: SandboxConfig,
    environment: BTreeMap<String, String>,
    protected_environment: Arc<BTreeSet<String>>,
}

impl std::fmt::Debug for SandboxBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SandboxBinding")
            .field("network_enabled", &self.config.network_enabled)
            .field("environment_count", &self.environment.len())
            .finish_non_exhaustive()
    }
}

impl SandboxBinding {
    /// Capture the actual host configuration without starting a remote sandbox.
    /// Environment grants identify these values, not later host lookups.
    ///
    /// # Errors
    /// Rejects non-isolating runners and mount configurations whose read/write
    /// semantics are not represented by the current sandbox wire contract.
    pub fn capture(runner: Arc<dyn SandboxRunner>, config: SandboxConfig) -> anyhow::Result<Self> {
        anyhow::ensure!(
            runner.enforces_isolation(),
            "Sandbox backend does not enforce isolation"
        );
        // SandboxConfig::volumes is an untyped string map. No backend contract
        // in this runtime specifies read-only versus writable mounts; do not
        // claim it enforces FilesystemGrant by inventing a string convention.
        anyhow::ensure!(
            config.volumes.is_empty(),
            "Sandbox mount permission bindings are unsupported"
        );
        anyhow::ensure!(
            !config.network_enabled || runner.capabilities().supports_networking,
            "Sandbox backend cannot enforce the requested network configuration"
        );
        let names = config.env_vars.keys().cloned().collect::<BTreeSet<_>>();
        let environment = names
            .iter()
            .map(|name| (uuid::Uuid::new_v4().to_string(), name.clone()))
            .collect();
        Ok(Self {
            runner,
            config,
            environment,
            protected_environment: Arc::new(names),
        })
    }

    /// Report only authority physically represented by the captured config.
    /// Empty mount/environment collections mean no host resources are exposed;
    /// they are not a claim about direct tools running outside this sandbox.
    pub fn permissions(&self, execution_mode: ToolExecutionMode) -> SandboxPermissions {
        SandboxPermissions {
            execution_mode,
            network_enabled: self.config.network_enabled,
            filesystem: BTreeMap::new(),
            environment: self.environment.keys().cloned().collect(),
        }
    }

    /// Narrow the configuration, retaining exact backend and environment values.
    ///
    /// # Errors
    /// Rejects unknown grants, network widening and unsupported filesystem
    /// restrictions before a child can create or execute a sandbox.
    pub fn for_permissions(&self, permissions: &SandboxPermissions) -> anyhow::Result<Self> {
        anyhow::ensure!(
            !permissions.network_enabled || self.config.network_enabled,
            "Child sandbox network permission exceeds the captured host binding"
        );
        anyhow::ensure!(
            permissions.filesystem.is_empty(),
            "Sandbox filesystem grants are unsupported"
        );
        anyhow::ensure!(
            permissions
                .environment
                .iter()
                .all(|id| self.environment.contains_key(id)),
            "Child sandbox environment grant is not inherited"
        );
        let mut binding = self.clone();
        binding.config.network_enabled = permissions.network_enabled;
        binding
            .environment
            .retain(|id, _| permissions.environment.contains(id));
        let retained = binding.environment.values().collect::<BTreeSet<_>>();
        binding
            .config
            .env_vars
            .retain(|name, _| retained.contains(name));
        Ok(binding)
    }

    pub(crate) fn runner(&self) -> Arc<dyn SandboxRunner> {
        Arc::clone(&self.runner)
    }

    pub(crate) fn execution_config(
        &self,
        runner: &Arc<dyn SandboxRunner>,
        request: &ExecutionRequest,
    ) -> anyhow::Result<SandboxConfig> {
        anyhow::ensure!(
            Arc::ptr_eq(runner, &self.runner),
            "Sandbox operation selected another backend"
        );
        anyhow::ensure!(
            matches!(request.mode, ExecutionMode::Ephemeral),
            "Owned sandbox execution requires an ephemeral operation"
        );
        // Tool-authored variables are not host environment grants. They may not
        // override a captured value or reintroduce a binding removed by a child.
        anyhow::ensure!(
            !request
                .env
                .keys()
                .any(|name| self.protected_environment.contains(name)),
            "Sandbox request cannot replace a host environment binding"
        );
        Ok(self.config.clone())
    }
}
