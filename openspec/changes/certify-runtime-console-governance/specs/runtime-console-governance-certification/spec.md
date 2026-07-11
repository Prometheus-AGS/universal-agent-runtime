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
