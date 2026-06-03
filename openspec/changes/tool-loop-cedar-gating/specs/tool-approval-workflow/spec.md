## MODIFIED Requirements

### Requirement: Tool calls matching approval policy pause for user confirmation
A tool call SHALL pause the run and emit an approval request event to the
frontend when it requires approval. A tool requires approval when EITHER the
built-in risk heuristic flags it (destructive/write-oriented keyword) OR the
Cedar governance engine denies it (`is_tool_allowed(agent_id, tool_name)` returns
deny). With the default permit-all policy, only the heuristic triggers approval,
so existing behavior is unchanged; restrictive policies add policy-denied tools
to the set requiring approval. When no governance engine is configured, only the
heuristic applies.

#### Scenario: High-risk tool requires approval (heuristic)
- **WHEN** the agent calls a tool whose name matches the risk heuristic (e.g., `filesystem__delete`)
- **THEN** the run pauses and an `agui.tool_call.approval_required` event is emitted with tool name, arguments, and risk reason

#### Scenario: Policy-denied tool requires approval
- **WHEN** the agent calls a tool that the Cedar engine denies for the run's agent (e.g., a `forbid` rule on `execute_tool` for that tool)
- **THEN** the run pauses and an approval request is emitted (the human may approve), rather than the tool executing or being silently dropped

#### Scenario: Permit-all default does not change behavior
- **WHEN** the bundled permit-all policy is in effect and the heuristic does not match a tool
- **THEN** the tool is approved automatically (no approval prompt), exactly as before this change

#### Scenario: User approves tool call
- **WHEN** the user clicks "Approve" on the approval dialog
- **THEN** a POST to `/api/uar/runs/{run_id}/tool-approval` with `{"approved": true}` resumes the tool execution

#### Scenario: User rejects tool call
- **WHEN** the user clicks "Reject" on the approval dialog
- **THEN** a POST to `/api/uar/runs/{run_id}/tool-approval` with `{"approved": false}` cancels the tool call and the agent receives a rejection message

#### Scenario: Approval timeout
- **WHEN** no approval response is received within 5 minutes
- **THEN** the tool call is automatically rejected and the agent is notified
