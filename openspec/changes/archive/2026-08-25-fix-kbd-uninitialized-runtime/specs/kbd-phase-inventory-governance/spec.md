## ADDED Requirements

### Requirement: Registered UAR projects initialize on their first typed KBD mutation
A UAR checkout with canonical project and replica identity but no signed KBD runtime event SHALL remain uninitialized during read-only status inspection and SHALL create exactly one signed runtime initialization boundary when its first typed mutation is submitted. Compatible legacy lifecycle, phase, plan revision, and exact-next-work state MUST be preserved before the requested mutation is applied.

#### Scenario: Status inspects registered empty UAR state
- **WHEN** a registered UAR checkout has no signed KBD runtime event and status is requested
- **THEN** status reports pending automatic initialization without changing canonical history or recommending an unrelated migration

#### Scenario: First typed phase mutation follows legacy state
- **WHEN** the registered checkout has compatible legacy waypoint and phase projections and receives its first typed mutation
- **THEN** one initialization boundary imports that compatible state before the mutation commits
- **AND** the canonical runtime path is included if initialization fails

#### Scenario: Later phase mutation reuses the initialized run
- **WHEN** another typed mutation follows successful automatic initialization
- **THEN** it reuses the same run without creating a second initialization event

#### Scenario: Canonical validation rejects a mutation
- **WHEN** canonical KBD validation rejects a typed mutation
- **THEN** the CLI exits non-zero and does not record the rejected command as committed
