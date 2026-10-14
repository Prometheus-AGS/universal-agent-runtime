# auth-key-management Specification

## Purpose
TBD - created by archiving change fix-auth-revoke-key-error-surfacing. Update Purpose after archive.

## Requirements

### Requirement: Auth Key Mutation Failures Are Surfaced To The User

Every mutation on the admin auth/API-key page (create, revoke) SHALL surface a failure to the user via the page's visible error state, matching the pattern already used for the page's load/list operation. No mutation SHALL fail silently.

#### Scenario: Revoking a key fails

- **Given** `deleteAuthKey` rejects (network error, server error, or non-2xx response)
- **When** `revokeKey` handles the failure
- **Then** the store's `error` field MUST be set to a message describing the failure, which the page renders via its visible error component — the failure MUST NOT be silently discarded

#### Scenario: Creating a key fails

- **Given** `createAuthKey` rejects
- **When** `createKey` handles the failure
- **Then** the store's `error` field MUST be set (already the existing, correct behavior — this scenario documents the baseline the revoke path must match)

### Requirement: API-key authority survives restart and revocation is current

Production API-key authentication and management SHALL persist hashed key material, authority metadata, expiry and revocation, and enforce current authority across processes and restarts.

#### Scenario: Cross-process revocation

- **WHEN** one runtime revokes a key and another receives a new request using it
- **THEN** the second denies admission using durable current authority

#### Scenario: Durable storage outage

- **WHEN** production key storage cannot establish current validity
- **THEN** authentication and management fail explicitly without an in-memory or stale-authority fallback
