# Spec Delta

## MODIFIED Requirements

### Requirement: Governance-disabled local runs bypass governance decisions

While an eligible local runtime has `governance.enabled` set to `false`, it SHALL bypass automatic Cedar/risk decisions for eligible local root calls with effective approval policy Auto and a non-Required descriptor. Explicit Ask/Deny policy, current host authority and an explicit tool descriptor `ApprovalClass::Required` SHALL remain binding; Required SHALL still require its host-owned durable approval before execution. Tool registration and selection, argument validation, transport behavior, provider behavior, and ordinary execution failures SHALL remain unchanged. When governance is enabled, the complete existing governance and approval behavior SHALL apply.

#### Scenario: Available web search executes while governance is Off

- **WHEN** an eligible local anonymous run calls an available configured `web_search` tool while `governance.enabled` is `false` and the descriptor does not require approval
- **THEN** the tool executes without a governance-denied or approval-required outcome

#### Scenario: Automatic governance bypass preserves explicit policy

- **WHEN** an eligible local root tool call has effective approval policy Auto, a non-Required descriptor and `governance.enabled` is `false`
- **THEN** automatic Cedar/risk outcomes are bypassed and the available configured tool proceeds to execution
- **AND** changing effective policy to explicit Ask or Deny still requires its decision or denies the call, respectively

#### Scenario: Descriptor Required remains binding

- **WHEN** an eligible local run with governance Off reaches an available configured tool with `ApprovalClass::Required`
- **THEN** the tool pauses for the exact durable host approval and cannot execute through the governance bypass

#### Scenario: Disabled or unselected tool remains unavailable

- **WHEN** a tool is not registered, not selected, or otherwise unavailable while local governance is Off
- **THEN** the runtime does not invent or expose the tool and reports the ordinary capability or execution failure

#### Scenario: Enabled governance preserves denial semantics

- **WHEN** `governance.enabled` is `true` and Cedar denies a tool execution
- **THEN** execution stops, an auditable denial is emitted, and human approval cannot override it

### Requirement: Inactive governance posts one warning per process

The runtime SHALL emit an operator-visible structured warning the first time governance is observed inactive in a process. It SHALL emit no additional inactive-governance warning in that process for later settings reads, requests, runs, tool calls, or repeated Off transitions. A new process SHALL be eligible to emit its own single warning if governance is inactive. The warning SHALL describe the eligible automatic Cedar/risk bypass accurately and SHALL NOT claim explicit Ask/Deny, host authority or descriptor Required approval is bypassed.

#### Scenario: Process starts with governance Off

- **WHEN** an eligible local process starts with `governance.enabled` set to `false`
- **THEN** it emits exactly one structured warning about inactive governance and the continuing descriptor Required approval boundary

#### Scenario: Governance is disabled after startup

- **WHEN** a process starts with governance On and an eligible local operator later changes `governance.enabled` to `false`
- **THEN** the process emits its one inactive-governance warning when the Off state is first applied

#### Scenario: Repeated local activity does not repeat the warning

- **WHEN** a process has already emitted its inactive-governance warning and then serves additional requests, runs, tool calls, settings reads, or On-to-Off transitions
- **THEN** no additional inactive-governance warning is emitted by that process
