## ADDED Requirements

### Requirement: Local persistence dependencies are version-verified before runtime readiness
The native deployment process SHALL verify SurrealDB server 3.2.4, install hash-identical dependency binaries, restart SurrealDB, Surreal Memory, and UAR in dependency order, and withhold success until each dependency proves health and persistence.

#### Scenario: Native services restart on the verified dependency baseline
- **WHEN** the locally packaged release is deployed
- **THEN** SurrealDB reports 3.2.4 and passes authenticated create/read/query evidence before Surreal Memory passes write/restart/read persistence and UAR is declared ready

#### Scenario: Dependency bootstrap fails
- **WHEN** a restarted dependency fails its health or persistence gate
- **THEN** deployment success is not reported and the captured binaries and prior service state are restored
