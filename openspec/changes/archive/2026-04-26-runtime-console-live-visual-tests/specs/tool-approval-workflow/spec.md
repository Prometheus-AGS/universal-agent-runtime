## ADDED Requirements

### Requirement: Approval surfaces are visually covered in the runtime console
The tool approval workflow SHALL have targeted visual coverage for approval request surfaces inside the runtime console.

#### Scenario: Approval surface is reachable from runtime navigation
- **WHEN** the operator navigates to the approvals surface from the runtime console shell
- **THEN** the page MUST show approval workflow content or an intended empty state
- **AND** the content MUST be visible without being hidden by navigation or contextual panels.

#### Scenario: Approval actions remain visually actionable
- **WHEN** approval actions are rendered in the runtime console
- **THEN** approve and reject controls MUST be visible, focusable, and non-overlapping with the approval details.

### Requirement: Approval visual coverage preserves policy behavior
The tool approval workflow SHALL keep its existing pause, approval, rejection, timeout, and dialog-content behavior while adding runtime console visual coverage.

#### Scenario: Existing approval behavior remains in scope
- **WHEN** runtime console visual tests are added
- **THEN** the existing approval policy and dialog scenarios MUST remain valid.
