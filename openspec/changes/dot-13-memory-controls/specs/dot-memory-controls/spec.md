# Spec Delta

## Purpose

Define assistant capture and retention are owner controlled for reliable always-on assistant operation and honest support claims.

## ADDED Requirements

### Requirement: Assistant capture and retention are owner controlled

Automatic assistant memory capture SHALL require owner opt-in and an explicit retention choice. Deletion or expiry SHALL immediately suppress content from retrieval/history and survive cleanup interruption.

#### Scenario: Capture consent changes

- **WHEN** automatic capture is disabled or consent is revoked while capture work is queued
- **THEN** no new assistant capture is committed under the stale policy

#### Scenario: Deletion across restart

- **WHEN** the owner deletes a memory and the process exits before index cleanup
- **THEN** content becomes immediately unavailable to retrieval/history and durable purge resumes after restart

### Requirement: Automatic capture retention is explicit

The retention input SHALL be either `{"kind":"days","days":7}`, the same shape with 30, 90 or 365 days, or `{"kind":"until_deleted"}`. Other durations, mixed forms and missing choices SHALL be rejected at enablement. Existing explicit user memories SHALL NOT acquire automatic-capture consent or retention by inference.

#### Scenario: Explicit supported retention choice

- **WHEN** an owner enables automatic capture with 7, 30, 90 or 365 days, or until-deleted
- **THEN** the durable policy records that explicit choice and revision; any other duration or mixed/missing form is rejected

### Requirement: Automatic memory expiry has an immutable anchor

A retention day SHALL mean 24 hours in UTC from the memory's immutable first durable capture timestamp. A record SHALL expire at or after its effective expiration timestamp, suppressing content on all read surfaces before physical cleanup.

#### Scenario: Exact expiration boundary

- **WHEN** an automatically captured memory reaches first durable capture plus its effective whole-day duration
- **THEN** retrieval, direct-ID reads, export and history suppress its content at that exact instant without waiting for physical index cleanup

### Requirement: Retention revisions cannot revive prior content

Shortening SHALL constrain existing automatic captures immediately. Extending retention or choosing until-deleted SHALL apply only to new captures and SHALL NOT extend existing expiry or revive deleted/expired content. Earlier shortening constraints SHALL persist durably across policy revisions, cleanup interruption and restart until affected content is purged.

#### Scenario: Shortening cannot be undone by an interrupted purge

- **WHEN** retention is shortened for existing automatic memories, the process exits before purge completes, and the owner later extends retention or chooses until-deleted
- **THEN** existing effective expiry remains constrained by the earlier shortening across runtimes and restart; expired content is not revived, and purge resumes

### Requirement: Capture commits use current policy

Queued automatic capture SHALL atomically recheck current consent, retention and policy revision at commit. A retry SHALL preserve the first durable capture timestamp and SHALL NOT restart retention.

#### Scenario: New captures use current retention

- **WHEN** queued capture commits after a retention policy revision
- **THEN** the host atomically checks current opt-in and revision and records current retention at first durable capture; stale consent cannot commit and retries preserve the original timestamp rather than restarting retention
