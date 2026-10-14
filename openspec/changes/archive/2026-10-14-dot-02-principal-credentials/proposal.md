# Proposal

## Why

Credential APIs and resolver collapse the verified identity to user_id. This change supplies the R2 acceptance guarantee for the always-on assistant runtime; source findings are not yet reproduced failures.

## What Changes

Introduce a host-derived versioned issuer/tenant/subject key in a namespace disjoint from legacy subjects, including explicit local issuer namespace. Carry it through create/list/read/delete/resolve/use and caches on SurrealDB and Postgres. Quarantine ambiguous legacy rows; provide auditable authenticated mapping with conflict detection and resumable migration, preserving ciphertext and scope precedence.

**BREAKING for unsafe legacy behavior:** callers can no longer rely on subject-only ownership, volatile production authority or successful no-op durability; migration and explicit unsupported outcomes are required.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `tenant-isolation`: Provider credentials use verified complete owner identity. Add a requirement without removing existing scenarios.

## Impact

Code ownership: `src/uar/security/claims.rs`, `src/uar/security/middleware.rs`, `src/uar/api/credentials.rs`, `src/uar/security/credentials/resolver.rs`, `src/uar/security/credentials/store.rs`, `migrations`. Dependencies: none. Runtime UX changes are truthful API/event outcomes and durable state; no new UI is included. Provider compatibility preserves existing routing and secret reattachment; no new provider integration. Realtime consumers must observe the same committed outcomes, not inferred in-memory success. KBD task state changes only through the phase's typed task lifecycle during Execute; this proposal does not implement behavior.

