# Spec Delta

## ADDED Requirements

### Requirement: AG-UI run input supports durable interruption and resume

UAR SHALL accept the selected pinned AG-UI `RunAgentInput` contract on a documented authenticated endpoint and emit conformant SSE events. A run awaiting a human interaction SHALL end its current stream with the specified `RUN_FINISHED` interruption outcome. A later `resume[]` SHALL resolve the exact existing host-owned interaction under current authority and preserve stable run/event attribution.

#### Scenario: Approval interrupts an AG-UI run

- **WHEN** a `RunAgentInput` run reaches a pending Required tool approval
- **THEN** the stream emits the selected spec events and a `RUN_FINISHED` interruption outcome identifying the durable pending interaction
- **AND** no tool effect is reported before a valid host decision

#### Scenario: Resume after restart

- **WHEN** the authorized client sends `resume[]` for that interaction after a fresh process starts
- **THEN** the host resolves the same durable decision and protected continuation at most once
- **AND** the resumed stream does not duplicate a logical effect or fabricate a completed prior run

#### Scenario: Invalid or stale resume

- **WHEN** `resume[]` names another owner's, expired or invalidated interaction
- **THEN** UAR denies continuation without disclosing private state or executing a tool
