## Purpose

Expose the native peer tools available to an admitted team attempt through the existing policy-governed model tool catalog.

## ADDED Requirements

### Requirement: Attempt-native inventory precedes policy resolution

The runtime SHALL include the four native peer identities in both tool-universe resolution passes only for a bound team attempt in the qualified team execution profile.

#### Scenario: Coordinator without filesystem effects

- **WHEN** a qualified coordinator attempt selects peer tools and no host filesystem tools
- **THEN** both policy resolutions consider team_roster, team_send, team_delegate and team_wait available before applying existing scope restrictions

#### Scenario: Ordinary agent

- **WHEN** a run has no bound team attempt
- **THEN** this inventory addition exposes no team peer tools to the run

### Requirement: Peer handlers preserve effective restrictions

The runtime SHALL register only peer tools retained by effective policy. Native attempt/session/owner checks and all kernel peer authority SHALL remain unchanged.

#### Scenario: Scope denies a peer tool

- **WHEN** a policy scope denies or excludes a peer identity
- **THEN** its handler is absent from the attempt registry and the model tool request

#### Scenario: Admitted peer operation

- **WHEN** the model calls an exposed peer tool
- **THEN** the existing attempt-bound native handler and kernel authority checks govern execution without routing through the host filesystem allowlist
