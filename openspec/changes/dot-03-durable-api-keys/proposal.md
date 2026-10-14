# Proposal

## Why

Server startup selects in-memory API-key storage. This change supplies the R7 acceptance guarantee for the always-on assistant runtime; source findings are not yet reproduced failures.

## What Changes

Wire durable production key storage for both existing database providers, retaining hashed secrets, authority metadata, expiry and owner identity. Read authoritative revocation/expiry before admission; storage outage cannot fall back to memory or stale allow decisions. Preserve explicit development mode.

**BREAKING for unsafe legacy behavior:** callers can no longer rely on subject-only ownership, volatile production authority or successful no-op durability; migration and explicit unsupported outcomes are required.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `auth-key-management`: API-key authority survives restart and revocation is current. Add a requirement without removing existing scenarios.

## Impact

Code ownership: `src/server.rs`, `src/uar/security/api_keys.rs`, `src/uar/persistence`, `migrations`. Dependencies: dot-02-principal-credentials. Runtime UX changes are truthful API/event outcomes and durable state; no new UI is included. Provider compatibility preserves existing routing and secret reattachment; no new provider integration. Realtime consumers must observe the same committed outcomes, not inferred in-memory success. KBD task state changes only through the phase's typed task lifecycle during Execute; this proposal does not implement behavior.

