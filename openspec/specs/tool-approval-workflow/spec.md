## Purpose

Define tool approval behavior and runtime console visual requirements for approval workflows.
## Requirements
### Requirement: Tool calls matching approval policy pause for user confirmation
When a tool call matches a Cedar policy rule marked as `requires_approval`, the run SHALL pause and emit an approval request event to the frontend.

#### Scenario: High-risk tool requires approval
- **WHEN** the agent calls a tool tagged as high-risk in Cedar policy (e.g., `filesystem__delete`)
- **THEN** the run pauses, an `agui.tool_call.approval_required` event is emitted with tool name, arguments, and risk reason

#### Scenario: User approves tool call
- **WHEN** the user clicks "Approve" on the approval dialog
- **THEN** a POST to `/api/uar/runs/{run_id}/tool-approval` with `{"approved": true}` resumes the tool execution

#### Scenario: User rejects tool call
- **WHEN** the user clicks "Reject" on the approval dialog
- **THEN** a POST to `/api/uar/runs/{run_id}/tool-approval` with `{"approved": false}` cancels the tool call and the agent receives a rejection message

#### Scenario: Approval timeout
- **WHEN** no approval response is received within 5 minutes
- **THEN** the tool call is automatically rejected and the agent is notified

### Requirement: Approval UI renders clearly
The frontend SHALL display a modal dialog showing the tool name, arguments, and risk assessment for approval requests.

#### Scenario: Approval dialog content
- **WHEN** an approval request is received for tool `filesystem__write` with arguments `{"path": "/etc/config"}`
- **THEN** the dialog shows the tool name, a formatted view of the arguments, the risk policy that triggered approval, and Approve/Reject buttons

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
