## MODIFIED Requirements

### Requirement: Tool calls matching approval policy pause for user confirmation
When a tool call matches a Cedar policy rule marked as `requires_approval`, or the tool's descriptor approval class requires it under the effective run policy, the run SHALL pause and emit an approval request event to the frontend. The approval decision SHALL be derived from Cedar and the descriptor's approval class, never from the tool's name.

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
- **WHEN** no approval response is received within the configured approval timeout
- **THEN** the tool call is automatically rejected and the agent is notified

#### Scenario: Name does not decide approval
- **WHEN** a tool named `get_and_purge_records` is called with an `ExternalMutation` effect under `ToolApprovalPolicy::Auto`
- **THEN** the run pauses for approval because of its effect and policy, regardless of the `get_` prefix
