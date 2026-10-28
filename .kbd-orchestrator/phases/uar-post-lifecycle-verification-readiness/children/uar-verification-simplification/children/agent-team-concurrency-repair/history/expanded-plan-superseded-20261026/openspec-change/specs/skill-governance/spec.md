# Spec Delta

## ADDED Requirements

### Requirement: Configured filesystem sources satisfy durable admission invariants

UAR SHALL reconcile enabled configured filesystem skill sources with durable selection metadata before admitting runs. Reconciliation SHALL preserve scoped enablement and API-owned overrides, cover every configured source identity, and report an incomplete result rather than publishing a ready registry whose required selection records are absent.

#### Scenario: ADMIT-01 filesystem skills admit the real team
- **WHEN** a durable runtime starts with configured filesystem skills and no corresponding stored selection rows
- **THEN** startup reconciles the required records before member admission
- **AND** the mathematics and writing members can enter their real configured-model runs without a storage-missing failure caused by those sources

#### Scenario: Stored overrides survive source reconciliation
- **WHEN** a stored skill override or scoped disable exists and its filesystem source is registered again
- **THEN** the stored override and effective scoped selection survive
- **AND** the runtime does not replace API-owned configuration with filesystem defaults

### Requirement: Skill admission exposes private categorized failure evidence

UAR SHALL retain an owner-scoped cause category distinguishing unavailable selection storage from registered metadata missing its durable record. It SHALL preserve the public admission-failure code, with additive redacted diagnostics correlated to the run and source identity. It SHALL NOT disclose skill bodies, credentials or foreign-owner state, or bypass reconciliation to manufacture success.

#### Scenario: Required resource is absent
- **WHEN** a private team binding requires an absent or mismatched exact skill reference
- **THEN** admission or preflight refuses before provider dispatch with a safe resource diagnostic
- **AND** the request is not counted as successful team execution

#### Scenario: Selection storage fails
- **WHEN** a real selection read or decode fails
- **THEN** the runtime retains the unavailable-storage category rather than asserting a missing record or successful empty selection
- **AND** current confirmed configuration is not reset
