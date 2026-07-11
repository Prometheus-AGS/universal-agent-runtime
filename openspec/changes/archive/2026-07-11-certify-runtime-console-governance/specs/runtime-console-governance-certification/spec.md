## ADDED Requirements

### Requirement: Denial is not approval
A Cedar `Deny` decision SHALL be non-overridable and SHALL NOT be represented as a pending approval.

#### Scenario: Forbidden tool
- **WHEN** Cedar denies a tool execution
- **THEN** execution stops, an auditable denial is emitted, and no approve control is rendered

### Requirement: Console shows runtime truth
Console entities SHALL originate from correlated API or runtime events, never illustrative placeholder data.

#### Scenario: Live run
- **WHEN** a run emits steps, tools and protocol events
- **THEN** Cockpit, Protocols and Run Detail update without manual refresh

### Requirement: UAR has one configurable HTTP default
The UAR HTTP server and all first-party deployment and client examples SHALL use
port `1906` when no override is supplied, while preserving `--port`, `PORT`, and
`UAR_SERVER__PORT` configuration overrides.

#### Scenario: Default startup
- **WHEN** UAR starts without an HTTP port override
- **THEN** it listens on port `1906` and first-party health checks and clients target port `1906`

#### Scenario: Operator override
- **WHEN** an operator supplies a supported HTTP port override
- **THEN** UAR listens on that port instead of `1906`
