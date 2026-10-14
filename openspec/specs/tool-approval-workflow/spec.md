## Purpose

Define tool approval behavior and runtime console visual requirements for approval workflows.

## Requirements

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

### Requirement: Recovered approvals require current executable authority

Recovered approvals SHALL expose current state and authorize at most one continuation under current identity, policy and bindings. Restart SHALL NOT turn a stale decision into execution authority.

#### Scenario: Pending approval after restart

- **WHEN** an authorized owner reads a pending approval after its original process exits
- **THEN** the current approval and continuation disposition remain visible and execution still requires valid current authority

#### Scenario: Competing decisions

- **WHEN** two processes attempt to consume the same approval or its binding has changed
- **THEN** at most one valid continuation is admitted and stale or expired decisions cannot authorize effects

#### Scenario: Pending checkpoint publication

- **WHEN** the trusted host publishes a recoverable pending challenge
- **THEN** its protected pre-effect checkpoint is already committed and the approval row contains only its stable checkpoint identity and binding digest
- **AND** a checkpoint with a pending marker cannot execute through ordinary resume without its exact current decision

#### Scenario: Orderly shutdown while pending

- **WHEN** the original host cancels and joins execution during orderly shutdown
- **THEN** it preserves only checkpoint-linked pending challenges after acknowledging cancellation of their prepared admission
- **AND** failed cancellation or uncertain effects block predecessor exclusion and the original expiry remains unchanged

#### Scenario: Current authority changed

- **WHEN** current owner, policy, descriptor, payload or binding cannot renew a recovered pending call
- **THEN** no effect executes and stale executable mapping becomes visibly invalidated
- **AND** unsupported storage or a persistence outage is an explicit unavailable outcome, never an invented approval

#### Scenario: Recovered block contains sibling calls

- **WHEN** a protected unresolved assistant call block contains the selected call and remaining siblings
- **THEN** the host executes calls in original order through current admission and each sibling's own approval
- **AND** model continuation receives only complete real results without fabricated history repair

#### Scenario: Decision admission response

- **WHEN** a fresh approval request admits a continuation before its async gate commits a decision
- **THEN** the response distinguishes continuation admission from durable decision state and execution success remains observable through the new run

#### Scenario: Unknown recovered effect

- **WHEN** the recovered executor commits its decision and loses effect certainty
- **THEN** its immutable continuation claim remains held and another decision cannot replay the original call

#### Scenario: Expired denied or unbound decision

- **WHEN** an original challenge expires, is denied or has no protected executable mapping
- **THEN** the current API exposes expired, denied or invalidated state and no effect executes
- **AND** restart does not renew the timeout or assign legacy issuer authority

#### Scenario: Denial without executable continuation

- **WHEN** an authenticated owner denies a current pending challenge without a protected continuation or on an unsupported resume backend
- **THEN** the pending decision can atomically become denied without a continuation claim or effect
- **AND** an approval still requires its protected executable mapping

#### Scenario: Deadline passes during decision persistence

- **WHEN** an approve or deny intent reaches storage after its immutable expiry, including after awaiting a writer lock
- **THEN** the atomic transition commits expired with no human decision and cannot commit approved or denied
- **AND** the host exposes the actual committed state and executes no effect from the lost decision

#### Scenario: Required invalidation persistence fails

- **WHEN** current authority rejects recovery but the required invalidation cannot be durably committed
- **THEN** the API or run events expose approval persistence unavailable without claiming a committed invalidation
- **AND** the acquired continuation claim remains held and retry cannot replay the call
