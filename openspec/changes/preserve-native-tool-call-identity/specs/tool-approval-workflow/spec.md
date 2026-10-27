## ADDED Requirements

### Requirement: Native authority binds a nonblank tool-call identity

Native admission SHALL require a nonblank model tool-call identity within the existing authority envelope before host preparation, approval or dispatch. A valid identity SHALL be preserved exactly through pending approval, protected checkpoint history, live or recovered continuation, and canonical receipt persistence. Existing ownership, policy, stale-decision and exact-once protections SHALL remain in force.

#### Scenario: Invalid native authority identity

- **WHEN** preparation receives an empty or whitespace-only model tool-call identity
- **THEN** preparation fails before approval and dispatch without inventing a receipt identity

#### Scenario: Approved continuation retains identity

- **WHEN** a valid native call is approved and resumed live or reconstructed from its checkpoint
- **THEN** preparation, pending history and the persisted canonical result bind the same authentic identity

#### Scenario: Invalid recovered binding

- **WHEN** owner, run, protected history, policy or approval state mismatches its binding
- **THEN** existing rejection and no-replay behavior remain unchanged

### Requirement: Cold approval recovery is independent of durable run inspection

Owner-authorized durable run inspection SHALL remain available after restart
without recreating a process-local run or granting continuation authority.
Approval decision and pending routes SHALL select live handling only for an
exact authenticated process-resident run. Durable history alone SHALL NOT select
the live waiter path. Existing full-principal/workspace, protected checkpoint,
current policy/resource renewal and immutable claim checks SHALL remain required.

#### Scenario: Durable metadata coexists with a cold pending approval

- **WHEN** an orderly source run has durable history and a pending protected approval, and its successor process has no resident source run
- **THEN** the verified owner can inspect the original run with200 and read the durable pending snapshot with resolvable false and recoveryAvailable true
- **AND** approval or denial follows the existing cold admission/decision contract, without inventing an original waiter

#### Scenario: Inspection grants no execution authority

- **WHEN** the verified owner reads a cold run and checkpoint metadata without requesting recovery
- **THEN** public run fields match independently persisted history, with no provider request, effect, active-run insertion or continuation claim
- **AND** unrelated subjects/tenants cannot inspect the run and invalid issuer credentials remain rejected by authenticated ingress

#### Scenario: Process-resident approval behavior is retained

- **WHEN** the exact authenticated owner addresses a run still held in the current process
- **THEN** the existing live approval handling remains in force and its response schema is unchanged
