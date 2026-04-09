## ADDED Requirements

### Requirement: SandboxRunner Trait

The crate SHALL define an async trait `SandboxRunner` that abstracts sandbox lifecycle and execution operations. All methods SHALL return `Result<T, SandboxError>`. The trait SHALL be object-safe and `Send + Sync` so it can be stored behind `Arc<dyn SandboxRunner>`.

The trait SHALL declare the following async methods:
- `create(config: SandboxConfig) -> Result<SandboxHandle, SandboxError>` — provisions a new sandbox from the given configuration.
- `execute(handle: &SandboxHandle, request: ExecutionRequest) -> Result<ExecutionResult, SandboxError>` — runs code or a command inside an existing sandbox.
- `write_file(handle: &SandboxHandle, path: &Path, content: &[u8]) -> Result<(), SandboxError>` — writes bytes to a file inside the sandbox filesystem.
- `read_file(handle: &SandboxHandle, path: &Path) -> Result<Vec<u8>, SandboxError>` — reads a file from the sandbox filesystem and returns its contents.
- `destroy(handle: SandboxHandle) -> Result<(), SandboxError>` — tears down the sandbox and releases all associated resources.

#### Scenario: Creating and destroying a sandbox through the trait
- **WHEN** a caller invokes `create` with a valid `SandboxConfig`
- **THEN** the method SHALL return `Ok(SandboxHandle)` containing a unique opaque identifier and a reference to the runner that created it.

#### Scenario: Destroying a sandbox releases resources
- **WHEN** a caller invokes `destroy` with a previously created `SandboxHandle`
- **THEN** the method SHALL return `Ok(())` and all resources (VM, volumes, network) associated with that handle SHALL be released.

#### Scenario: Executing code inside a sandbox
- **WHEN** a caller invokes `execute` with a valid handle and an `ExecutionRequest`
- **THEN** the method SHALL return `Ok(ExecutionResult)` containing exit_code, stdout, stderr, and execution_time_ms.

#### Scenario: Operating on a destroyed sandbox returns an error
- **WHEN** a caller invokes `execute`, `write_file`, or `read_file` on a handle whose sandbox has already been destroyed
- **THEN** the method SHALL return `Err(SandboxError::NotFound)`.

### Requirement: RunnerCapabilities Struct

The crate SHALL define a `RunnerCapabilities` struct that describes what a given `SandboxRunner` implementation supports. The `SandboxRunner` trait SHALL include a synchronous method `capabilities() -> RunnerCapabilities`.

The struct SHALL include at minimum:
- `supports_network: bool` — whether the runner can provide network access to sandboxes.
- `supports_volumes: bool` — whether the runner supports persistent named volumes.
- `supports_snapshot: bool` — whether the runner can snapshot and restore sandbox state.
- `max_memory_mib: Option<u32>` — the maximum memory a sandbox can be allocated, if bounded.
- `supported_languages: Vec<Language>` — languages the runner can execute.

#### Scenario: Querying runner capabilities
- **WHEN** a caller invokes `capabilities()` on any `SandboxRunner` implementation
- **THEN** the method SHALL return a `RunnerCapabilities` value that accurately reflects the implementation's support matrix.

#### Scenario: Capability-gated feature access
- **WHEN** `capabilities().supports_network` is `false` and a `SandboxConfig` is provided with `network_enabled: true`
- **THEN** the `create` method SHALL return `Err(SandboxError::UnsupportedCapability)` rather than silently ignoring the flag.
