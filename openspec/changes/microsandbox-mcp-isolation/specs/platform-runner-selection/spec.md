## ADDED Requirements

### Requirement: Auto-Detect Hardware Virtualization

The system SHALL automatically detect available hardware virtualization support at startup. On Linux, it SHALL check for the presence of `/dev/kvm`. On macOS, it SHALL check for Hypervisor.framework (HVF) availability.

#### Scenario: KVM detected on Linux
- **WHEN** the application starts on a Linux host and `/dev/kvm` exists and is accessible
- **THEN** the system SHALL report KVM as available and select `MicrosandboxRunner` as the default runner.

#### Scenario: No hardware virtualization available
- **WHEN** the application starts on a Linux host and `/dev/kvm` does not exist
- **THEN** the system SHALL report that hardware virtualization is unavailable and proceed to the fallback chain.

### Requirement: Fallback Chain

The system SHALL implement a runner selection fallback chain in the following priority order:
1. `microsandbox` — used when hardware virtualization (KVM or HVF) is available.
2. `wasmtime` — used as a lightweight alternative when no hardware VM support exists.
3. `remote` — delegates sandbox execution to a remote service when local execution is not possible.

The system SHALL select the first available runner in the chain.

#### Scenario: Fallback from microsandbox to wasmtime
- **WHEN** hardware virtualization is not available but the Wasmtime runtime is compiled in
- **THEN** the system SHALL select the Wasmtime-based runner as the active `SandboxRunner`.

#### Scenario: Fallback to remote runner
- **WHEN** neither hardware virtualization nor Wasmtime is available, and `UAR_SANDBOX_REMOTE_URL` is configured
- **THEN** the system SHALL select the remote runner that delegates execution to the configured URL.

#### Scenario: No runner available
- **WHEN** no runner in the fallback chain is available and no remote URL is configured
- **THEN** the system SHALL log a warning and sandbox tools SHALL return `Err(SandboxError::NoRunnerAvailable)` when invoked.

### Requirement: Runner Override via Environment Variable

The system SHALL support an explicit override of runner selection via the `UAR_SANDBOX_RUNNER` environment variable. Valid values SHALL be `microsandbox`, `wasmtime`, and `remote`. When set, the system SHALL skip auto-detection and use the specified runner directly.

#### Scenario: Override selects microsandbox
- **WHEN** `UAR_SANDBOX_RUNNER` is set to `"microsandbox"` and KVM is available
- **THEN** the system SHALL use `MicrosandboxRunner` regardless of the auto-detection result.

#### Scenario: Override to unavailable runner returns error
- **WHEN** `UAR_SANDBOX_RUNNER` is set to `"microsandbox"` but KVM is not available
- **THEN** the system SHALL return an error at startup indicating the requested runner is not available on this platform.

### Requirement: Remote Runner Configuration

When the remote runner is selected (either by fallback or override), the system SHALL require the `UAR_SANDBOX_REMOTE_URL` environment variable to be set to a valid URL of the remote sandbox service.

#### Scenario: Remote runner with valid URL
- **WHEN** `UAR_SANDBOX_RUNNER` is `"remote"` and `UAR_SANDBOX_REMOTE_URL` is `"https://sandbox.example.com"`
- **THEN** the system SHALL configure the remote runner to delegate all sandbox operations to that URL.

#### Scenario: Remote runner without URL fails
- **WHEN** the remote runner is selected but `UAR_SANDBOX_REMOTE_URL` is not set
- **THEN** the system SHALL return an error at startup indicating that the remote URL is required.
