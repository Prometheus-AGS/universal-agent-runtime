## ADDED Requirements

### Requirement: SandboxConfig Struct

The crate SHALL define a `SandboxConfig` struct used to configure a new sandbox instance. The struct SHALL include the following fields:
- `image: String` — the OCI image reference to use as the sandbox base (e.g. `"prometheus-ags/sandbox-python:latest"`).
- `memory_mib: u32` — memory allocation in MiB (default 512).
- `cpus: u32` — number of virtual CPUs (default 1).
- `network_enabled: bool` — whether outbound network access is permitted (default `false`).
- `volumes: Vec<VolumeMount>` — named volumes to mount inside the sandbox.
- `env_vars: HashMap<String, String>` — environment variables injected into the sandbox.
- `timeout_secs: u64` — maximum wall-clock time before the sandbox is forcibly terminated (default 300).

The struct SHALL implement `Default` with the documented defaults above.

#### Scenario: Constructing a config with defaults
- **WHEN** a caller creates `SandboxConfig::default()`
- **THEN** the resulting config SHALL have `memory_mib` of 512, `cpus` of 1, `network_enabled` of false, empty `volumes` and `env_vars`, and `timeout_secs` of 300.

#### Scenario: Custom config overrides defaults
- **WHEN** a caller sets `memory_mib` to 1024 and `network_enabled` to true
- **THEN** only those fields SHALL differ from the defaults; all other fields SHALL retain their default values.

### Requirement: SandboxHandle Type

The crate SHALL define a `SandboxHandle` struct that acts as an opaque reference to a running sandbox. It SHALL contain:
- An opaque unique identifier (e.g. UUID or string).
- A reference to the `SandboxRunner` that created it (via `Arc<dyn SandboxRunner>`).

The handle SHALL implement `Clone`, `Debug`, `Eq`, and `Hash`.

#### Scenario: Handle uniqueness
- **WHEN** two sandboxes are created from the same `SandboxConfig`
- **THEN** their `SandboxHandle` values SHALL NOT be equal.

#### Scenario: Handle identity across clone
- **WHEN** a `SandboxHandle` is cloned
- **THEN** the clone SHALL be equal to the original and reference the same sandbox.

### Requirement: ExecutionRequest Struct

The crate SHALL define an `ExecutionRequest` struct for submitting work to a sandbox. It SHALL include:
- `language: Language` — the language runtime to use.
- `code: String` — the source code or command to execute.
- `stdin: Option<String>` — optional standard input to pipe to the process.
- `env: HashMap<String, String>` — additional environment variables for this execution only.
- `cwd: Option<String>` — working directory inside the sandbox.
- `timeout: Option<u64>` — per-execution timeout in seconds, overriding the sandbox default.
- `mode: ExecutionMode` — the execution lifecycle mode.

#### Scenario: Execution with stdin
- **WHEN** an `ExecutionRequest` is submitted with `stdin` set to `Some("hello\n")`
- **THEN** the sandbox process SHALL receive `"hello\n"` on its standard input.

#### Scenario: Per-execution timeout overrides sandbox timeout
- **WHEN** an `ExecutionRequest` specifies `timeout: Some(5)` against a sandbox with `timeout_secs: 300`
- **THEN** the execution SHALL be terminated after 5 seconds, not 300.

### Requirement: ExecutionResult Struct

The crate SHALL define an `ExecutionResult` struct returned after sandbox execution completes. It SHALL include:
- `exit_code: i32` — the process exit code.
- `stdout: String` — captured standard output.
- `stderr: String` — captured standard error.
- `execution_time_ms: u64` — wall-clock execution duration in milliseconds.

#### Scenario: Successful execution result
- **WHEN** a Python script prints `"42"` and exits with code 0
- **THEN** the `ExecutionResult` SHALL have `exit_code` of 0, `stdout` containing `"42"`, and an empty `stderr`.

#### Scenario: Failed execution result
- **WHEN** a command exits with code 1 and writes to stderr
- **THEN** the `ExecutionResult` SHALL have `exit_code` of 1 and `stderr` containing the error output.

### Requirement: Language Enum

The crate SHALL define a `Language` enum with at least the following variants:
- `Bash`
- `Python`
- `Rust`
- `Node`

The enum SHALL implement `Serialize`, `Deserialize`, `Clone`, `Debug`, `PartialEq`, and `Eq`.

#### Scenario: Deserializing language from JSON
- **WHEN** the JSON string `"python"` is deserialized
- **THEN** it SHALL produce `Language::Python`.

#### Scenario: Unknown language rejected
- **WHEN** the JSON string `"cobol"` is deserialized as a `Language`
- **THEN** deserialization SHALL fail with an error.

### Requirement: ExecutionMode Enum

The crate SHALL define an `ExecutionMode` enum with the following variants:
- `Ephemeral` — the sandbox is destroyed after execution completes.
- `Session { id: String }` — the sandbox persists for the duration of a session.
- `Project { id: String, repo: String }` — the sandbox persists with a project workspace and repository checkout.

#### Scenario: Ephemeral mode cleans up after execution
- **WHEN** an execution completes in `ExecutionMode::Ephemeral`
- **THEN** the sandbox SHALL be destroyed and no state SHALL persist.

#### Scenario: Session mode preserves state
- **WHEN** two sequential executions use `ExecutionMode::Session { id: "abc" }`
- **THEN** filesystem changes from the first execution SHALL be visible in the second.
