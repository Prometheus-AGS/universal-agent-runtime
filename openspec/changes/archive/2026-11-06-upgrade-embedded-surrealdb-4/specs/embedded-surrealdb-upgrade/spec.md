## ADDED Requirements

### Requirement: Embedded 4.0 artifact preserves persistence behavior

The Linux amd64 UAR OCI service SHALL use its pinned 4.0 embedded SurrealKV dependency and preserve the established persistence API, owner boundaries, supported transaction safeguards, typed value semantics and contractual query behavior.

#### Scenario: SDB-1 fresh volume persists owner records

- **WHEN** the exact candidate starts with a fresh dedicated embedded volume behind the trusted gateway and an owner writes a record
- **THEN** that owner can read it before and after restart, and another owner cannot read it

#### Scenario: The installed API starts with the existing OpenAPI document

- **WHEN** the exact Linux candidate initializes its API and documentation routes
- **THEN** it reaches readiness and publishes the existing OpenAPI document, including unconstrained A2UI action inputs and AG-UI snapshot event data

#### Scenario: SDB-3 migrated records preserve their semantics

- **WHEN** the representative 3.3 dataset is transferred using the approved migration procedure
- **THEN** existing API operations preserve its owner visibility, identity, supported value and relationship semantics, contractual ordering and supported search results, and a forced transaction failure leaves no partial durable state after reopen

### Requirement: Explicit offline migration and separate restoration

UAR SHALL refuse a 3.3 store at normal 4.0 startup and require an explicit offline migration procedure using a separate destination. The supported-source manifest SHALL bind the exact 3.3 artifact, patches, format, schema, configuration and data classes covered by the release claim. The operator SHALL freeze writes to establish a recovery point and retain an independently restorable 3.3 baseline. If writes resume before rollback, the operator SHALL record their disposition and any accepted writes that would be lost before restoring service. A converted or partial 4.0 store SHALL be ineligible as a 3.3 rollback target.

#### Scenario: SDB-2 migration preserves the original store

- **WHEN** normal 4.0 startup sees an unconverted 3.3 copy and the operator then follows the documented migration procedure on separate storage
- **THEN** normal startup refuses conversion and the refused copy still opens with 3.3 without a format or content change, the migration preserves the agreed data, and the untouched 3.3 source stays reopenable by its original artifact

#### Scenario: SDB-5 recovery after interrupted migration

- **WHEN** migration is interrupted and the operator follows the recovery procedure
- **THEN** the failed destination is isolated and the pinned 3.3 artifact serves the recorded baseline from a distinct restored volume
- **AND** the upgrade destination remains marked ineligible as a source for offline migration or restore even if its legacy manifest was copied before interruption

#### Scenario: SDB-5 rollback accounts for accepted writes after migration

- **WHEN** a disposable 4.0 candidate accepts a write after the recorded recovery point and rollback is attempted without a disposition for that write
- **THEN** rollback refuses before serving 3.3 and preserves the candidate and backup
- **AND WHEN** the disposition and any loss are recorded
- **THEN** recovery on a distinct 3.3 volume returns the declared baseline and reports the excluded write

### Requirement: Embedded engine shutdown honors runtime ownership

UAR SHALL stop admission and settle admitted persistence producers before closing its owned embedded engine. Reachable cloned clients, active subscriptions and storage workers SHALL be covered by the close contract. Retain existing protections until equivalent behavior is established. A successful shutdown SHALL permit an independent reopen. Releasing a borrowed client SHALL leave its host-owned connection usable. Failure or uncertainty SHALL remain explicit and SHALL NOT authorize automatic replay.

#### Scenario: SDB-4 shutdown drains before close

- **WHEN** shutdown starts with an admitted producer still active
- **THEN** the owned engine does not acknowledge close before the producer settles, successful close permits immediate independent reopen, and releasing a borrowed handle leaves its host usable
