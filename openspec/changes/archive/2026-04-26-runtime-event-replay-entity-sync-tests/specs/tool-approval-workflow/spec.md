## ADDED Requirements

### Requirement: Replayed approvals appear in runtime console
The tool approval workflow SHALL expose replayed approval events in the runtime console with current approval state and action context.

#### Scenario: Pending replayed approval is visible
- **WHEN** a replayed approval request event is ingested for a runtime run
- **THEN** the approvals surface MUST show the approval id, run id, associated tool call when available, pending status, and approval actions.

#### Scenario: Updated replayed approval state is visible
- **WHEN** a replayed approval update event changes an existing approval from pending to approved, denied, or expired
- **THEN** the approvals surface MUST show the updated status without a manual refresh.

#### Scenario: Approval replay preserves policy behavior
- **WHEN** approval replay tests are added
- **THEN** the existing policy-driven pause, approval, rejection, timeout, and dialog-content requirements MUST remain valid.
