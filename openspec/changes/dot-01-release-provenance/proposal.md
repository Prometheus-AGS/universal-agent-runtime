# Proposal

## Why

The frozen-install and schema fixes already landed; policy pins disagree with effective sources and Docker omits --locked. This change supplies the R1 acceptance guarantee for the always-on assistant runtime; source findings are not yet reproduced failures.

## What Changes

Prepare an exact intended-versus-effective pin reconciliation for the operator; do not edit versions.toml. Once policy resolution is recorded, align source/lock/build inputs, retain frozen pnpm installs and enforce locked Cargo resolution without redoing landed fixes. Preserve embedded schema inclusion and deployment-only workflow policy.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `offline-reproducible-build`: Release artifacts have traceable locked inputs. Add a requirement without removing existing scenarios.

## Impact

Code ownership: `versions.toml`, `Cargo.toml`, `Cargo.lock`, `Dockerfile`, `.dockerignore`, `.github/workflows/deploy.yml`. Dependencies: none. Runtime UX changes are truthful API/event outcomes and durable state; no new UI is included. Provider compatibility preserves existing routing and secret reattachment; no new provider integration. Realtime consumers must observe the same committed outcomes, not inferred in-memory success. KBD task state changes only through the phase's typed task lifecycle during Execute; this proposal does not implement behavior.

