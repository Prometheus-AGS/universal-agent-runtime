## ADDED Requirements

### Requirement: Terminal KBD runs continue through an explicit successor boundary
After a KBD run reaches `completed`, `cancelled`, or `failed`, new phase work SHALL begin only after an operator-signed, causally ordered successor-run event. The successor boundary MUST preserve project identity and immutable audit while resetting run-scoped position, phases, completion, decisions, blockers, and claims.

#### Scenario: New work follows a terminal run
- **WHEN** an operator starts a new phase after a terminal run
- **THEN** one successor run is committed before the phase is created
- **AND** the authoritative waypoint names the successor run and requested phase rather than exposing the former phase as current work

#### Scenario: Successor state is projected
- **WHEN** the successor event and requested phase creation commit
- **THEN** the waypoint has a fresh plan revision and completion counters with no stale active path, checkpoint, decision, blocker, or claim from the former run
- **AND** the former run remains available through immutable audit history

#### Scenario: Current run is not terminal
- **WHEN** a successor start is attempted from a non-terminal lifecycle
- **THEN** the runtime rejects the command without creating a new run

#### Scenario: Terminal rollover projection fails
- **WHEN** the successor event commits but its compatibility projections fail
- **THEN** the emergency PAUSE valve remains active
- **AND** new phase creation does not proceed until the projection succeeds
