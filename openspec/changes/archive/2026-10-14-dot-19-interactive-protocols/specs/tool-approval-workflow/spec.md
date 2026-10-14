# Spec Delta

## ADDED Requirements

### Requirement: Explicit Required tool approval survives local governance bypass

A tool descriptor with `ApprovalClass::Required` SHALL require the host's exact durable approval decision before execution, whether effective governance is enabled or eligible local governance is Off. Neither an A2A/AG-UI projection nor an MCP elicitation response SHALL create or replace that authorization. Current owner, descriptor, payload, binding, expiry and executor authority SHALL be rechecked on continuation.

#### Scenario: Required tool under governance Off

- **WHEN** an eligible local run with governance Off reaches an available tool whose descriptor approval class is Required
- **THEN** the host records a pending exact approval ID and the tool remains uninvoked until a current valid decision

#### Scenario: One valid decision

- **WHEN** two clients approve the same pending decision across processes
- **THEN** at most one protected continuation and effect is admitted and both clients observe the committed disposition

#### Scenario: Denial or stale authority

- **WHEN** the owner denies, the decision expires, or current authority/bindings no longer match
- **THEN** no tool effect occurs and the exposed disposition is denied, expired or invalidated rather than success

### Requirement: A2A input-required projects the host approval record

For an A2A-originated run awaiting a tool approval, UAR SHALL expose the exact durable approval ID and pending state through the selected A2A 1.0 input-required task representation. Approve/deny SHALL resolve the existing host decision once under current authority, not a separate A2A decision store.

#### Scenario: Pending A2A approval

- **WHEN** an A2A task reaches a Required tool before a decision
- **THEN** the task is input-required, identifies the exact pending approval and has not invoked the tool

#### Scenario: A2A denial

- **WHEN** the authorized owner denies that exact pending approval
- **THEN** the task reaches the selected terminal denial outcome without a tool effect
