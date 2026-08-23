## ADDED Requirements

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
