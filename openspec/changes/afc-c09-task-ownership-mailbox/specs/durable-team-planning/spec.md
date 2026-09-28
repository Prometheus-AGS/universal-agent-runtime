## ADDED Requirements

### Requirement: Planning tasks have fenced ownership
UAR SHALL persist member claims, reassignment and reviewer selection against expected team and task revisions, idempotent command identities and a monotonically increasing ownership epoch. Superseded owners SHALL NOT commit task or protected effect state under an old epoch. C09.2 claims SHALL NOT start a model turn.

#### Scenario: Competing claims
- **WHEN** two writers claim the same ready task at one revision
- **THEN** at most one commits, the loser receives a conflict, and a replay of the winning command returns its result

### Requirement: Team inbox receipts distinguish durable stages
UAR SHALL persist addressed messages under authenticated owner/workspace/team scope. A public sender SHALL be the authenticated operator. `queue-only` and `trigger-turn` SHALL persist intent without executing a model until budget admission exists. Accepted, delivered and processed SHALL be distinct durable stages.

#### Scenario: Restart with an accepted message
- **WHEN** an owner sends a message and restarts the sidecar before recipient delivery
- **THEN** the inbox retains one accepted message and does not report it as delivered or processed
