## ADDED Requirements

### Requirement: Release and ordinary CI use one supported matrix
Release validation SHALL use the same frontend toolchain, Cargo bundles, warning policy and test commands as ordinary CI.

#### Scenario: Toolchain drift
- **WHEN** release automation references a different package manager, Node major, or unsupported Cargo combination
- **THEN** workflow policy validation fails

### Requirement: Advertised artifacts start successfully
Every advertised platform artifact SHALL be installed from its archive, started, and pass readiness/health checks in CI.

#### Scenario: Windows artifact
- **WHEN** Windows is listed as supported
- **THEN** a Windows-native job compiles, installs, starts and health-checks the packaged executable
