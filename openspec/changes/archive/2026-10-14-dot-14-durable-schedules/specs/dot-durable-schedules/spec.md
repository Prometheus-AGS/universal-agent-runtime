# Spec Delta

## Purpose

Define schedules persist intent and unique governed occurrences for reliable always-on assistant operation and honest support claims.

## ADDED Requirements

### Requirement: Schedules persist intent and unique governed occurrences

Schedules SHALL durably preserve owner intent, calendar policy, revision and occurrence identity. Concurrent scanners SHALL admit at most one logical occurrence under current authority, budget and approval requirements.

#### Scenario: Two scanners after downtime

- **WHEN** two runtimes find several due occurrences of the same non-overlapping schedule
- **THEN** cursor advancement and one latest coalesced occurrence commit atomically with a stable admission identity and visible skipped range

#### Scenario: DST and revocation

- **WHEN** a recurring local time is nonexistent or repeated, or its owner revokes the schedule before admission
- **THEN** nonexistent time is visibly skipped, repeated time selects the earlier instant, and revoked work is denied under current authority

#### Scenario: Validated recurrence revision

- **WHEN** an owner submits daily or weekly HH:mm plus IANA timezone, then changes that timezone
- **THEN** invalid forms are rejected, a valid edit creates a new revision, and unadmitted old-revision occurrences are cancelled without reinterpreting admitted work

### Requirement: Busy schedules coalesce behind one durable overlap guard

A stable owner/schedule guard SHALL span revisions and remain held while prior execution is queued, running, awaiting approval, suspended, cancelling or uncertain. While held, due occurrences SHALL coalesce into the latest current-revision pending occurrence with visible skipped history. Admission SHALL require safe prior resolution and fresh authority, revision, budget and approval checks.

#### Scenario: Multiple occurrences while busy

- **WHEN** several due instants occur while a prior run executes or awaits approval
- **THEN** competing scanners retain only the latest pending occurrence and do not admit another run until the prior run is safely resolved

#### Scenario: Uncertain predecessor after restart

- **WHEN** a process restarts with an uncertain prior effect and a coalesced pending occurrence
- **THEN** the durable guard remains held and the pending occurrence cannot execute until the predecessor is excluded and the effect is reconciled

#### Scenario: Edit during an active run

- **WHEN** a schedule revision changes while the prior run holds its guard
- **THEN** old unadmitted pending intent is cancelled, new-revision due instants may fill the pending slot, and no new-revision run overlaps the predecessor

#### Scenario: Safe release races a new due instant

- **WHEN** scanners race to update pending intent and admit work as the predecessor becomes safely terminal
- **THEN** one atomic current-revision admission consumes the latest eligible pending occurrence, records skipped occurrences, and preserves budget and approval checks


