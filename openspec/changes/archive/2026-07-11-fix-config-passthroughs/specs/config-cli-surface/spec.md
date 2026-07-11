## ADDED Requirements

### Requirement: Declared CLI/env config flags are applied

Every flag declared on the `Cli` struct SHALL be applied to the runtime
configuration when set, so a declared flag is never silently dropped. In
particular `--port` / `PORT` SHALL set `server.port` and `--jwt-required` /
`JWT_REQUIRED` SHALL set `security.jwt_required`.

#### Scenario: Port flag overrides the server port

- **Given** the server default port is 3000
- **When** the runtime is started with `--port 8123` (or `PORT=8123`)
- **Then** the effective `server.port` MUST be 8123

#### Scenario: JWT-required flag is honored

- **Given** `security.jwt_required` defaults to true
- **When** the runtime is started with `--jwt-required=false` (or
  `JWT_REQUIRED=false`)
- **Then** the effective `security.jwt_required` MUST be false, not silently
  ignored
