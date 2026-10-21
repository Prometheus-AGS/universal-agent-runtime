## ADDED Requirements

### Requirement: Local embedding failover preserves one pinned vector space

The trusted host SHALL support explicitly configured local primary and backup
embedding instances. It SHALL validate their model, dimension, asset provenance
and inference semantics before admitting them as compatible members. A missing
or failed member SHALL NOT silently resolve through an ambient model directory.
Legacy single-backend configuration SHALL retain its established behavior.

#### Scenario: Live primary failure and backup continuity
- **WHEN** the active local primary reports an authoritative failure
- **THEN** the host selects a validated compatible backup without process restart
- **AND** subsequent ingestion and queries retain the same durable space binding
- **AND** selection and generation changes are observable through retained state
- **AND** no polling, retry sleep or fixed-duration wait establishes recovery.

#### Scenario: Explicit primary recovery
- **WHEN** the authorized host requests recovery for the observed generation
- **THEN** the host initializes and validates the selected local instance before
  publishing its new active generation
- **AND** admitted work on the prior instance remains owned until actual completion
- **AND** a stale concurrent command cannot overwrite a later generation.

#### Scenario: Incompatible candidate has the same dimension
- **WHEN** a candidate uses a different actual model or incompatible asset or
  inference identity even if its dimension matches
- **THEN** activation or query is rejected with an explicit space mismatch
- **AND** existing document, source and knowledge-base bindings remain intact.

### Requirement: Local model workers participate in host closure

Each accepted initialization and inference operation SHALL retain host admission
through actual blocking-worker completion. Finalization SHALL close producer
admission and join operations before dependent resources are released.

#### Scenario: A request waiter disappears during inference
- **WHEN** the caller drops its response waiter
- **THEN** the host retains the accepted worker and its dependency leases until
  completion, recording failure or uncertainty without an elapsed-time success.

### Requirement: Embedding acceptance uses real local inference

Fixture source SHALL exercise real locally supplied model assets, durable KB
bindings, live failover, backup ingestion and query, primary recovery, and a real
incompatible model-space refusal. Missing assets SHALL be an explicit unmet
prerequisite and SHALL NOT produce a passing or silently skipped scenario.

#### Scenario: A mirror restart receipt is presented
- **WHEN** only primary-to-backup-to-primary process restarts are evidenced
- **THEN** the receipt establishes restart resilience only and SHALL NOT satisfy
  live failover acceptance.

### Requirement: Boss candidate payload identifies immutable UAR source

The candidate integration SHALL bind the archive, every included file, platform,
source revision and required host lifecycle protocol to one immutable prepared
artifact. It SHALL NOT relabel a historical binary or another platform's payload
as the new lifecycle implementation. Existing release integrity checks SHALL
remain enforced.

#### Scenario: A payload predates the lifecycle source
- **WHEN** the selected archive or source pin does not identify the prepared
  lifecycle candidate
- **THEN** local candidate packaging is rejected rather than reporting success
  with the historical payload.

#### Scenario: Unrelated working-tree edits remain after the candidate commit
- **WHEN** an explicit candidate build selects an immutable source revision
- **THEN** its producer exports only that commit and its pinned gitlink objects
  into an isolated source directory and uses a fresh target directory
- **AND** unrelated working-tree edits are neither compiled nor deleted
- **AND** the record binds the commit, source tree, native platform, exact build
  command, executable and retained build-log digests
- **AND** legacy local and public release integrity requirements remain enforced.
