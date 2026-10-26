## ADDED Requirements

### Requirement: Confirmed terminal usage reconciles liability exactly once

Host and durable scope accounting SHALL replace reserved liability with authoritative priced terminal usage only when terminal evidence is present. Replaying identical terminal evidence SHALL be idempotent. Conflicting terminal evidence SHALL remain an error rather than silently changing settled usage.

#### Scenario: Confirmed usage settles a reservation

- **WHEN** a pending admitted call receives authoritative priced terminal usage with nonempty terminal evidence
- **THEN** accounting settles the call once and subsequent admission uses the resulting actual usage

#### Scenario: Terminal evidence is replayed or conflicts

- **WHEN** a settled call receives the same terminal payload again or a conflicting terminal payload
- **THEN** the identical replay leaves charges unchanged and the conflicting payload is rejected without silently refunding or charging the call again

### Requirement: Unknown effects retain durable liability

Accounting SHALL retain unresolved liability for cancellation, missing responses, unpriced outcomes and failed settlement until authoritative reconciliation. Restart or elapsed time SHALL NOT make unknown work free.

#### Scenario: Pending liability survives restart

- **WHEN** a call may have produced an upstream effect but lacks qualifying terminal evidence and the runtime restarts
- **THEN** its durable pending liability remains counted and an admission exceeding the remaining budget is refused before another effect

#### Scenario: Positive settlement control

- **WHEN** authoritative terminal evidence reconciles a previously pending call within the scope budget
- **THEN** the next eligible admission can proceed using settled actuals rather than an indefinitely retained reservation
