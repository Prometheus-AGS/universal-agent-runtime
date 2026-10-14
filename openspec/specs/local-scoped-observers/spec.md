# local-scoped-observers Specification

## Purpose
TBD - created by archiving change afc-c07-local-observers. Update Purpose after archive.

## Requirements

### Requirement: Committed source occurrence

A durable source occurrence MUST have stable identity and provenance, and MUST be committed atomically with its source state. Ephemeral token streams MUST NOT be presented as durable occurrences.

#### Scenario: Crash after source commit

- **WHEN** the process exits after an instance transition commits and before observer admission
- **THEN** the occurrence remains discoverable from the outbox with the same identity after restart.

### Requirement: Independent scoped subscriptions

Each observer MUST retain its own revision, source/conversation intersection, projection grant, cursor, inbox admission and acknowledgement. Delivery MUST recheck current authority before disclosure or execution.

#### Scenario: Two observers with different grants

- **WHEN** two observers watch overlapping sources but one lacks a grant for a conversation
- **THEN** each advances independently and the restricted observer never sees that conversation's projection.

### Requirement: Bounded delivery and honest recovery

The runtime MUST expose pause, backlog, retention gap and dead-letter state. Replayed occurrences MUST NOT re-execute source effects, and uncertain observer effects MUST obey the existing C02/C06 reconciliation boundary.

#### Scenario: Restart at admission or acknowledgement

- **WHEN** the process exits at either boundary and later resumes
- **THEN** a stable admission identity recovers the same observer command receipt or reports a visible gap/failure without silently dropping accepted work.

### Requirement: Follow-up decisions and delivery remain governed and durable

Follow-ups SHALL derive only from authorized durable occurrences under current user-controlled rules. Decisions, suppressions and delivery state SHALL be durable; external dispatch uncertainty SHALL never be hidden by automatic retries.

#### Scenario: Policy suppresses follow-up

- **WHEN** an observed event matches a rule but consent, source grant or destination permission is revoked
- **THEN** no disclosure or effect occurs and an owner-visible scoped suppression reason is retained

#### Scenario: Delivery interrupted

- **WHEN** a committed follow-up is interrupted during delivery
- **THEN** the durable inbox item is recovered once logically, while uncertain external dispatch is held for reconciliation rather than retried blindly

#### Scenario: Overnight quiet hours

- **WHEN** a rule uses daily 22:00–07:00 quiet hours in its explicit IANA zone
- **THEN** 22:00 is suppressed and 07:00 is permitted, with overnight and repeated-hour membership evaluated at the candidate real instant and queued work rechecked after policy edits
