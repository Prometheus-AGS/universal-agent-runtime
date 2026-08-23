# Native Service Deployment Specification

## Purpose

Define how UAR is installed, configured, operated, and verified as a native service on macOS, Linux, and Windows.

## Requirements

### Requirement: Native supervisors use platform-owned lifecycle contracts
UAR SHALL ship a macOS user LaunchAgent named `com.prometheus.universal-agent-runtime`, a Linux systemd unit named `uar.service`, and a Windows native SCM service. Each supervisor SHALL start the release server directly, restart it after unexpected failure, and route an ordinary stop into UAR's graceful cancellation path.

#### Scenario: Supported platform package is inspected
- **WHEN** an operator inspects a shipped native package
- **THEN** its supervisor identity, direct executable, working directory, environment file, start behavior, and graceful stop behavior are explicit

### Requirement: Native defaults are local-machine-only
Every installed UAR network listener SHALL inherit the configured `server.host`, whose native default is loopback. A local-only HTTP setting SHALL NOT leave A2A gRPC bound to a wildcard address.

#### Scenario: Installed server exposes HTTP and A2A
- **WHEN** the native service runs with the shipped loopback configuration
- **THEN** HTTP port 1906 and A2A gRPC port 50051 listen only on loopback addresses

### Requirement: Native installation preserves operator authority
Installation and upgrade SHALL preserve existing configuration, database state, selected default model, and API/UI-created provider settings. Before changing an existing configuration, the installer SHALL create a backup and merge only absent phase-owned entries rather than replacing the file.

#### Scenario: Existing configuration is upgraded
- **WHEN** an operator installs over an existing `~/.uar/config.yaml`
- **THEN** a backup exists beneath `~/.prometheus/backups/uar/`, existing values remain unchanged, and only missing phase-owned entries are added

### Requirement: Operational logs live beneath deployment-specific .prometheus paths
Service stdout, stderr, and UAR operational logs SHALL be written below `~/.prometheus/logs/universal-agent-runtime/` on macOS, `/var/lib/uar/.prometheus/logs/` on Linux, and `%ProgramData%\Prometheus\UniversalAgentRuntime\.prometheus\logs\` on Windows. Database-engine metadata files remain database state.

#### Scenario: Service emits operational output
- **WHEN** the native service starts, serves a request, and stops
- **THEN** its supervisor and runtime output is retained only beneath the platform's required `.prometheus/logs` directory

### Requirement: Native deployment claims are platform-scoped
Verification SHALL report server-full, macOS runtime deployment, Linux template validation, and Windows compile/template validation separately. A macOS host SHALL NOT imply an observed Linux or Windows runtime deployment.

#### Scenario: Phase completes on macOS
- **WHEN** macOS runtime checks and Linux/Windows structure checks pass
- **THEN** evidence names each platform, profile, source SHA, command, observed output, and limitation separately

### Requirement: Packaging and bootstrap use one fixed interface
Native installers SHALL use `service.env` on macOS, `/etc/uar/uar.env` on Linux, and `%ProgramData%\Prometheus\UniversalAgentRuntime\config\uar.env` on Windows. Provider environment generation and additive YAML merge SHALL be implemented only by the bootstrap change, return nonzero without replacing a prior good file on failure, and never print credential values.

#### Scenario: Installer refreshes provider configuration
- **WHEN** a native installer invokes the shared provider environment and YAML merge entrypoints
- **THEN** it supplies explicit output/config/env/proxy paths, receives exit 0 only after a restrictive atomic write, and contains no duplicate credential or merge implementation

### Requirement: Explicit service environment files fail closed
UAR SHALL accept `--env-file <path>` and the equivalent `UAR_ENV_FILE`. When either explicitly selects a file, UAR SHALL load it before telemetry and configuration; unreadable or invalid content SHALL terminate startup with a contextual error.

#### Scenario: Selected environment file is invalid
- **WHEN** a service starts with an explicitly selected unreadable or malformed environment file
- **THEN** UAR exits nonzero before opening network listeners

### Requirement: Service logging supports an explicit file
When `UAR_LOG_FILE` is set, UAR SHALL open that path and direct operational tracing to it. Failure to open the selected path SHALL terminate startup rather than silently discarding service output.

#### Scenario: Service has no usable standard streams
- **WHEN** a supervisor starts UAR with a valid `UAR_LOG_FILE`
- **THEN** startup and request tracing is appended to the selected file

### Requirement: Windows service controls use graceful cancellation
The Windows-only `service` command SHALL register with SCM, report Start Pending, Running, Stop Pending, and Stopped states, and convert SCM Stop and Shutdown controls into the existing graceful server cancellation path.

#### Scenario: SCM requests stop
- **WHEN** Windows SCM sends Stop or Shutdown to a running UAR service
- **THEN** UAR reports Stop Pending, cancels through its graceful runtime path, and reports Stopped after the server completes

### Requirement: Native packages provide reversible lifecycle entrypoints
Each native package SHALL provide install and uninstall entrypoints plus documented start, stop, restart, upgrade, and credential-refresh operations. Uninstall SHALL preserve mutable state by default.

#### Scenario: Operator uninstalls a native service
- **WHEN** the uninstall entrypoint runs without an explicit destructive-state option
- **THEN** the supervisor registration and program files are removed while configuration, database state, backups, and logs remain recoverable

### Requirement: Platform packages use native filesystem conventions
macOS SHALL install a user service beneath `~/.uar`; Linux SHALL use `/etc/uar` for configuration and `/var/lib/uar` for state; Windows SHALL separate `%ProgramFiles%` program files from `%ProgramData%` state. Every package SHALL use the platform log directory defined by the native-service contract.

#### Scenario: Installer renders a service definition
- **WHEN** a supported installer creates its native service definition
- **THEN** executable, working-directory, configuration, environment, state, and log paths match the platform contract

### Requirement: Native service definitions restart only after failure
The macOS and Linux native service definitions SHALL start automatically and restart after unexpected failure without turning an operator-requested graceful stop into an uncontrolled restart loop.

#### Scenario: Native process exits unexpectedly
- **WHEN** UAR exits unexpectedly after its supervisor considers it running
- **THEN** the native supervisor restarts it according to its platform failure policy

### Requirement: Service credentials are generated from an allowlist
The bootstrap SHALL copy only approved canonical provider credential variables into the service environment, SHALL use aliases only when the canonical variable is absent, SHALL set restrictive file permissions, and SHALL never print values.

#### Scenario: Canonical and alias values both exist
- **WHEN** the source environment contains both `KIMI_API_KEY` and `KIMI_CODING_API_KEY`
- **THEN** the service environment retains the canonical `KIMI_API_KEY` value and does not expose either value in output

### Requirement: Native YAML enables both loopback listeners
The native YAML SHALL set `server.host` to `127.0.0.1`, HTTP port to 1906, and A2A gRPC port to 50051 for the server-full installed service.

#### Scenario: Native YAML is installed
- **WHEN** server-full starts from the merged native configuration
- **THEN** both configured listener ports establish at least one loopback LISTEN socket and absence of either listener is a verification failure

### Requirement: Alias mapping does not cross endpoints
The bootstrap MAY map Kimi Coding aliases to KIMI, MINIMAX_KEY to MINIMAX, and Qwen aliases to DASHSCOPE. It SHALL NOT map another endpoint's credential into MOONSHOT or ZAI.

#### Scenario: Only a nonmatching credential exists
- **WHEN** no canonical or approved alias exists for a provider
- **THEN** that provider credential and its conditional YAML seed are omitted

#### Scenario: Multiple aliases exist without the canonical name
- **WHEN** both Kimi aliases exist without `KIMI_API_KEY`, or both Qwen aliases exist without `DASHSCOPE_API_KEY`
- **THEN** `KIMI_CODING_API_KEY` wins over `KIMI_CODING_KEY`, and `QWEN_API_KEY` wins over `QWEN_TOKEN_PLAN_API_KEY`

### Requirement: macOS installed release is functionally verified
After code completion, the phase SHALL build and install the release binary and React bundle, load the LaunchAgent on port 1906, and observe health, readiness, UI/static assets, loopback-only listeners, provider/model visibility, genuine inference, persistence across one restart, database access, graceful shutdown, and required logging.

#### Scenario: Installed LaunchAgent is restarted
- **WHEN** the operator restarts the LaunchAgent after successful inference
- **THEN** it becomes ready again with configuration, provider visibility, database access, operational logging, and genuine inference intact

### Requirement: Native Alibaba configuration uses the released Qwen flagship
When an Alibaba credential is present, native bootstrap SHALL seed `qwen3.8-max` with the documented one-million-token context and 131,072-token maximum output. It SHALL migrate only the exact obsolete native selection `alibaba/qwen3.7-max`, malformed credential reference `QWEN_TOKENPLAN_API_KEY`, and phase-owned `qwen3-coder-plus` seed. Other operator selections and custom Alibaba provider blocks SHALL remain unchanged.

#### Scenario: Interrupted native installation is refreshed
- **WHEN** the existing native YAML contains the exact obsolete Alibaba values observed during this phase
- **THEN** refresh selects `alibaba/qwen3.8-max`, refers to canonical `DASHSCOPE_API_KEY`, updates the phase-owned provider seed, and leaves all unrelated YAML unchanged

#### Scenario: Operator owns a different Alibaba configuration
- **WHEN** the existing model, credential reference, or Alibaba provider block does not exactly match the obsolete phase values
- **THEN** bootstrap preserves that operator-owned value rather than applying a broad Qwen migration

### Requirement: Newly released native models enter through pinned catalog sources
The model API SHALL continue to use the compile-time catalog. When the pinned catalog sources have added a required newly released model, UAR SHALL advance the `models.dev` and `liter-llm` gitlinks and regenerate the reviewed offline UAR snapshot rather than introduce a second configured-model overlay.

#### Scenario: Updated catalog contains Qwen 3.8-Max
- **WHEN** the pinned `models.dev` and `liter-llm` commits contain Alibaba `qwen3.8-max`
- **THEN** the reviewed offline snapshot and release build expose that model through `/api/models` and the Models UI without changing the endpoint implementation or either submodule's source
