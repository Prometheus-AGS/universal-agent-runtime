## ADDED Requirements

### Requirement: Streamed tool identities remain authentic and stable

The runtime SHALL preserve the exact first nonblank provider tool-call identity for each stream index across absent or blank continuation identities. Repeated identical identities SHALL be accepted. A conflicting nonblank identity SHALL fail through the existing typed provider-stream error path, without printing input identities or emitting a successful completion for that call.

#### Scenario: Empty continuation after authentic identity

- **WHEN** a typed provider stream supplies an authentic ID and later absent or empty ID fragments
- **THEN** its completed call retains the original exact identity and complete arguments

#### Scenario: Conflicting continuation

- **WHEN** one stream index receives two distinct nonblank IDs
- **THEN** the run fails with a bounded fixed error and no native tool is executed

### Requirement: Native batch identity is valid before effects

The runtime SHALL validate all native call identities before approval, scheduling or execution of any call in that batch, including before any early no-tool completion path. Missing, blank or duplicate native IDs SHALL fail closed without fabricated identity, approval or tool result. Existing caller-owned host identity allocation SHALL remain unchanged.

#### Scenario: Later invalid native call

- **WHEN** a valid native call is followed by a missing, blank or duplicate native identity
- **THEN** no call in the batch executes and no approval gate is invoked

#### Scenario: Missing completion identity

- **WHEN** a normalized driver emits native tool deltas without a valid identity and finishes without a completion event
- **THEN** the run fails before effects rather than reporting success without the call

#### Scenario: Caller-owned tool compatibility

- **WHEN** caller-owned tools use the existing host continuation identity path
- **THEN** that allocation and its durable continuation bindings remain unchanged, including supported blank or absent provider continuation identities

### Requirement: Published native metadata agrees with admitted identity

Native provisional deltas SHALL use the reconciled exact nonblank identity once known, or omit identity while unresolved. The runtime SHALL preserve each original stream index and argument fragment bytes. Native completions SHALL be published exactly once per assembled stream index, using final reconciled identity/name/arguments, only after the terminal stream and native batch identity validation plus applicable caller checks complete. The same identity SHALL correlate manager ToolStart/ToolEnd, assistant/result dialogue pairs, provider history and canonical receipts. Caller-owned host UUID allocation and durable publication SHALL remain separate and unchanged.

#### Scenario: Blank continuation and sparse interleaving

- **WHEN** sparse interleaved native indexes receive authentic identities followed by absent, blank or identical identity fragments and repeated completions
- **THEN** provisional metadata retains authentic identities once known, and each validated native call completes once with its original index and final arguments
- **AND** real manager event and dialogue histories correlate the same exact call/result identities

#### Scenario: Completion followed by conflicting continuation

- **WHEN** an apparent native completion is followed by a distinct nonblank identity before terminal EOF, including after a Done event
- **THEN** the existing typed failure is emitted without a successful native completion, approval, effect or canonical result for that rejected batch

#### Scenario: Incomplete or failed terminal stream

- **WHEN** a pending native completion is followed by a normalized error, typed stream failure or EOF without the required terminal event
- **THEN** no successful completion, approval or native effect is published for that pending batch

#### Scenario: Configured caller tools with native publication

- **WHEN** caller tools are configured and a native batch passes the existing identity, choice and schema checks
- **THEN** native metadata retains its authentic identity and original index while caller metadata remains governed by the separate host-owned durable handoff
