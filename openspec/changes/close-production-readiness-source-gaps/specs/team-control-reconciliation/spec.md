## ADDED Requirements

### Requirement: Preserve team policy and uncertain liabilities
Team control advertisement SHALL reflect usable authoritativepolicy and typed recoverable versus terminal refusals. Reconciliation SHALL reuse authoritative owner/trustedhost usage/effect receipts, retain unknown liabilities and immutable settled outputs, and never replay effects or invent free usage.

#### Scenario: A cancelled attempt is reconciled repeatedly after restart
- **WHEN** A cancelled attempt is reconciled repeatedly after restart
- **THEN** Only identical authoritative evidence settles original IDs idempotently; unknown/conflicting/foreign evidence remains refused or reserved with explicit budget impact

#### Scenario: The installed Linux team recovery case observes a real restart
- **WHEN** The trusted host stops and starts the case-owned Linux amd64 runtime between the before and after observations
- **THEN** The case verifies a clean stop and a later start on the bound image, container and volume before comparing original team, attempt, accounting and effect identities
- **AND** Missing or mismatched fixture ownership fails before team admission, and the case exchanges only nonsecret selectors and lifecycle observations with the host

#### Scenario: A selected team control is unavailable to the current attempt
- **WHEN** current team eligibility, governance or approval policy withholds a selected attempt-bound team control
- **THEN** the actor captures only registered team controls, the task can run without the withheld control, and unrelated unbound tools still fail closed

#### Scenario: Concurrent team parents contend on embedded durable call admission
- **WHEN** embedded SurrealKV confirms that a pre-dispatch admission transaction aborted with a typed transaction conflict
- **THEN** the host can retry admission within a finite bound using the same call identity and reservation, with fresh scope and funding reads and the original generation checks
- **AND** an unknown commit outcome, stale generation or exhausted retry bound prevents dispatch, and previously submitted work and settlement stay outside the retry

#### Scenario: Concurrent calls share accounting scopes on owned embedded SurrealKV
- **WHEN** concurrent calls admit or record usage through the same owned embedded SurrealKV provider
- **THEN** the provider serializes those accounting writes from their first read through the transaction result and any settlement readback
- **AND** provider I/O stays outside the guard, durable compare-and-swap checks still apply, and uncertain usage remains pending
