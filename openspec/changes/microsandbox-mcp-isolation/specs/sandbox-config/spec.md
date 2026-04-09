## ADDED Requirements

### Requirement: Sandbox Environment Variables

The system SHALL support the following environment variables for sandbox configuration. Environment variables SHALL take precedence over config file values.

- `UAR_SANDBOX_RUNNER` — overrides auto-detected runner selection. Valid values: `"microsandbox"`, `"wasmtime"`, `"remote"`.
- `UAR_SANDBOX_REMOTE_URL` — URL of the remote sandbox service, required when runner is `"remote"`.
- `UAR_SANDBOX_DEFAULT_MEMORY_MIB` — default memory allocation for sandboxes in MiB (default 512).
- `UAR_SANDBOX_DEFAULT_TIMEOUT_SECS` — default execution timeout in seconds (default 300).
- `UAR_SANDBOX_NETWORK_ENABLED` — whether sandboxes have network access by default. Values: `"true"` or `"false"` (default `"false"`).
- `UAR_SANDBOX_MAX_CONCURRENT` — maximum number of concurrent active sandboxes (default 10).

#### Scenario: Environment variable overrides config file
- **WHEN** `UAR_SANDBOX_DEFAULT_MEMORY_MIB` is set to `"1024"` and `config.yaml` specifies `default_memory_mib: 256`
- **THEN** sandboxes SHALL be created with 1024 MiB of memory by default.

#### Scenario: Unset environment variables use defaults
- **WHEN** no `UAR_SANDBOX_*` environment variables are set and no config file sandbox section exists
- **THEN** the system SHALL use the compiled defaults: runner auto-detected, 512 MiB memory, 300s timeout, network disabled, 10 max concurrent.

### Requirement: Config File Sandbox Section

The system SHALL support a `sandbox` section in `config.yaml` for persistent sandbox configuration. The section SHALL support the following keys:

```yaml
sandbox:
  runner: microsandbox
  remote_url: https://sandbox.example.com
  default_memory_mib: 512
  default_timeout_secs: 300
  network_enabled: false
  max_concurrent: 10
  default_image: prometheus-ags/sandbox-python:latest
  volumes_dir: /var/lib/uar/volumes
```

#### Scenario: Config file values applied
- **WHEN** `config.yaml` contains a `sandbox` section with `default_memory_mib: 768`
- **THEN** sandboxes SHALL be created with 768 MiB of memory by default unless overridden by an environment variable.

#### Scenario: Missing sandbox section uses defaults
- **WHEN** `config.yaml` exists but contains no `sandbox` section
- **THEN** the system SHALL use the compiled defaults for all sandbox settings.

### Requirement: Configuration Precedence

Sandbox configuration SHALL follow the same precedence hierarchy as LLM configuration (highest to lowest):
1. CLI arguments (if added in the future).
2. `UAR_SANDBOX_*` environment variables.
3. `sandbox:` section in `config.yaml`.
4. Compiled defaults.

#### Scenario: Full precedence chain
- **WHEN** `UAR_SANDBOX_DEFAULT_TIMEOUT_SECS` is set to `"60"`, `config.yaml` specifies `default_timeout_secs: 120`, and the compiled default is 300
- **THEN** the effective timeout SHALL be 60 seconds.

#### Scenario: Config file fills gaps not covered by env vars
- **WHEN** `UAR_SANDBOX_DEFAULT_MEMORY_MIB` is set to `"1024"` but no other env vars are set, and `config.yaml` specifies `default_timeout_secs: 120` and `max_concurrent: 5`
- **THEN** memory SHALL be 1024 (from env), timeout SHALL be 120 (from config), and max_concurrent SHALL be 5 (from config).

### Requirement: SandboxConfig in AppConfig

The `AppConfig` struct SHALL include a `sandbox: SandboxSettings` field that aggregates all resolved sandbox configuration values. The `SandboxSettings` struct SHALL be populated during application initialization by merging values from environment variables, config file, and defaults according to the precedence rules.

#### Scenario: AppConfig exposes resolved sandbox settings
- **WHEN** the application initializes
- **THEN** `AppConfig.sandbox` SHALL contain fully resolved values for runner, memory, timeout, network, max_concurrent, and all other sandbox settings.

#### Scenario: Invalid configuration values rejected at startup
- **WHEN** `UAR_SANDBOX_DEFAULT_MEMORY_MIB` is set to `"not_a_number"`
- **THEN** the application SHALL fail to start with a clear error message indicating the invalid value.
