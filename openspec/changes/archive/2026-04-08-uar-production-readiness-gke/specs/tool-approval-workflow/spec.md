## ADDED Requirements

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
