## ADDED Requirements

### Requirement: Replayed run status transitions update status surfaces
The agent status UI SHALL reflect replayed runtime run and run step state transitions in chat-adjacent and runtime console status surfaces.

#### Scenario: Replayed running status is visible
- **WHEN** a replayed run or run step event sets status to `running` or `waiting`
- **THEN** the relevant runtime console status surface MUST display the active status without a manual refresh.

#### Scenario: Replayed terminal status is visible
- **WHEN** a replayed run or run step event updates status to `completed`, `failed`, or `cancelled`
- **THEN** the relevant runtime console status surface MUST display the terminal status without leaving stale active-state indicators.

#### Scenario: Status replay preserves chat status behavior
- **WHEN** runtime run status replay tests are added
- **THEN** the existing chat status requirements for thinking, tool execution, searching, completion, and transition behavior MUST remain unchanged.
