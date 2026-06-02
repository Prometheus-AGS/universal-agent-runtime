## MODIFIED Requirements

### Requirement: Tool calls matching approval policy pause for user confirmation
When a tool call matches a Cedar policy rule marked as `requires_approval`, the run SHALL pause and emit an approval request event to the frontend. A paused run SHALL also terminate if its run is cancelled while awaiting approval, resolving the pending approval as aborted so the orchestrator does not block indefinitely.

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

#### Scenario: Run cancelled while awaiting approval
- **WHEN** a run is paused on a tool-approval gate and the run is cancelled (via the cancel endpoint, last-subscriber drop, or shutdown)
- **THEN** the pending approval is resolved as aborted, the orchestrator unblocks, no tool execution occurs, and the run terminates with a `cancelled` outcome
